//! The Runner (docs/GUI.md §3.3, U1): the one struct that owns a run's `Sim`.
//!
//! It has no clock and no thread (E2). A driver calls [`Runner::handle`] with each command and
//! [`Runner::advance`] with a tick budget; the Runner steps at most that many ticks, extracts
//! each, checks its breakpoints, takes the ring's checkpoints, hands the slice's observations to
//! its sink and then calls `wake`. However the commands and slices fall, the `Sim` steps one tick
//! at a time from the same start, so the hash stream is the tape's and nothing else's (U4).

use super::ring::ring_tick_at_or_after;
use super::{
    Breakpoint, Catalogue, Cmd, Extractor, Obs, ObsBatch, PauseReason, Refusal, ResumeFrom,
    RingCheckpoint, Snapshot,
};
use certify::{tape_hash, Build, Hex, RunKey};
use rustyecon_engine::prelude::*;

/// What a slice did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// The ticks it ran.
    pub ran: u32,
    /// Whether the run wants another slice: it is running or stepping.
    pub busy: bool,
}

/// What the run is doing between slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Paused,
    Running { until: Option<u64> },
    Stepping { left: u64 },
}

/// A loaded run.
struct Live {
    key: RunKey,
    tape: Tape,
    sim: Sim,
    extractor: Extractor,
    mode: Mode,
    /// This run's ring, `(state tick, Checkpoint::to_bytes)`, for deep inspection.
    ring: Vec<(u64, Vec<u8>)>,
    next_ring: Option<u64>,
    /// Snapshot ticks asked for and not yet reached, increasing.
    snapshots: Vec<u64>,
}

/// Owns one run's `Sim` and reports what it does.
pub struct Runner {
    build: Build,
    sink: Box<dyn FnMut(Obs) + Send>,
    wake: Box<dyn Fn() + Send + Sync>,
    breakpoints: Vec<Breakpoint>,
    catalogue: Catalogue,
    live: Option<Live>,
}

impl Runner {
    /// A Runner with no tape. `build` names the binary in every run key; `sink` takes each
    /// observation; `wake` is called after each slice or command that reported something (the
    /// app's `request_repaint`).
    pub fn new(
        build: Build,
        sink: Box<dyn FnMut(Obs) + Send>,
        wake: Box<dyn Fn() + Send + Sync>,
    ) -> Runner {
        Runner {
            build,
            sink,
            wake,
            breakpoints: Vec::new(),
            catalogue: Catalogue::Full,
            live: None,
        }
    }

    /// Carry out one command.
    pub fn handle(&mut self, c: Cmd) {
        let mut out = Vec::new();
        match c {
            Cmd::Load { tape, from } => self.load(*tape, from, &mut out),
            Cmd::Run { until, .. } => {
                if let Some(live) = self.runnable() {
                    let tick = live.sim.tick();
                    match until {
                        Some(u) if u <= tick => {
                            live.mode = Mode::Paused;
                            out.push(Obs::Paused {
                                tick,
                                why: PauseReason::Reached(u),
                            });
                        }
                        _ => {
                            live.mode = Mode::Running { until };
                            out.push(Obs::Running { tick });
                        }
                    }
                }
            }
            Cmd::Step(n) => {
                if let Some(live) = self.runnable() {
                    let tick = live.sim.tick();
                    if n == 0 {
                        live.mode = Mode::Paused;
                        out.push(Obs::Paused {
                            tick,
                            why: PauseReason::Stepped,
                        });
                    } else {
                        live.mode = Mode::Stepping { left: n };
                        out.push(Obs::Running { tick });
                    }
                }
            }
            Cmd::Pause => {
                if let Some(live) = self.live.as_mut() {
                    if live.mode != Mode::Paused {
                        live.mode = Mode::Paused;
                        out.push(Obs::Paused {
                            tick: live.sim.tick(),
                            why: PauseReason::Asked,
                        });
                    }
                }
            }
            Cmd::Breakpoints(b) => self.breakpoints = b,
            Cmd::Catalogue(c) => self.catalogue = c,
            Cmd::Snapshot(t) => {
                if let Some(live) = self.live.as_mut() {
                    live.snapshot(t, &mut out);
                }
            }
            Cmd::Stop => self.live = None,
        }
        self.emit(out);
    }

    /// Whether a run is loaded and running or stepping, so that `advance` would step it.
    pub fn busy(&self) -> bool {
        self.live.as_ref().is_some_and(|l| l.mode != Mode::Paused)
    }

    /// Run at most `max_ticks` ticks of a running or stepping run, as one slice: one batch, then
    /// any ring checkpoints and snapshots it reached, then a pause if it paused.
    pub fn advance(&mut self, max_ticks: u32) -> Progress {
        let Some(live) = self.live.as_mut() else {
            return Progress {
                ran: 0,
                busy: false,
            };
        };
        if live.mode == Mode::Paused {
            return Progress {
                ran: 0,
                busy: false,
            };
        }
        let known = u32::try_from(live.extractor.catalogue().len()).unwrap_or(u32::MAX);
        let mut new_series = Vec::new();
        let mut rows = Vec::new();
        let mut last = None;
        let mut after = Vec::new();
        let mut paused = None;
        let mut failed = None;
        let mut ran = 0;
        while ran < max_ticks {
            let tick = live.sim.tick();
            match live.mode {
                Mode::Running { until: Some(u) } if tick >= u => {
                    paused = Some(PauseReason::Reached(u));
                    break;
                }
                Mode::Stepping { left: 0 } => {
                    paused = Some(PauseReason::Stepped);
                    break;
                }
                _ => {}
            }
            match live.sim.step() {
                Ok(report) => {
                    ran += 1;
                    rows.push(live.extractor.row(&live.sim, &report, &mut new_series));
                    let hit = hit(&self.breakpoints, &live.sim.world().clock, &report);
                    last = Some(report);
                    if let Mode::Stepping { left } = &mut live.mode {
                        *left -= 1;
                    }
                    live.reached(&mut after);
                    if let Some(b) = hit {
                        paused = Some(PauseReason::Breakpoint(b));
                        break;
                    }
                }
                Err(error) => {
                    let last = live.sim.last_report().cloned().map(Box::new);
                    failed = Some(Obs::Failed { error, last });
                    paused = Some(if self.breakpoints.contains(&Breakpoint::OnError) {
                        PauseReason::Breakpoint(Breakpoint::OnError)
                    } else {
                        PauseReason::Failed
                    });
                    break;
                }
            }
        }
        // A slice that ran its budget exactly to a stop pauses now, not a slice later.
        if paused.is_none() {
            let tick = live.sim.tick();
            match live.mode {
                Mode::Running { until: Some(u) } if tick >= u => {
                    paused = Some(PauseReason::Reached(u));
                }
                Mode::Stepping { left: 0 } => paused = Some(PauseReason::Stepped),
                _ => {}
            }
        }
        let mut out = Vec::new();
        if let Some(last) = last {
            out.push(Obs::Batch(ObsBatch {
                known,
                new_series,
                rows,
                last: Box::new(last),
            }));
        }
        out.extend(failed);
        out.extend(after);
        if let Some(why) = paused {
            live.mode = Mode::Paused;
            out.push(Obs::Paused {
                tick: live.sim.tick(),
                why,
            });
        }
        let busy = live.mode != Mode::Paused;
        self.emit(out);
        Progress { ran, busy }
    }

    /// The live run, if it can step.
    fn runnable(&mut self) -> Option<&mut Live> {
        self.live
            .as_mut()
            .filter(|l| matches!(l.sim.status(), Status::Ready))
    }

    fn load(&mut self, tape: Tape, from: Option<ResumeFrom>, out: &mut Vec<Obs>) {
        self.live = None;
        let mut sim = None;
        if let Some(ResumeFrom::Ring(cp)) = from {
            match cp.checkpoint() {
                Err(e) => out.push(Obs::Refused(Refusal::Decode(e))),
                Ok(cp) => match Sim::resume(&tape, &cp) {
                    Ok(s) => sim = Some(s),
                    Err(e) => out.push(Obs::Refused(Refusal::Resume(e))),
                },
            }
        }
        let sim = match sim {
            Some(s) => s,
            None => match Sim::new(&tape) {
                Ok(s) => s,
                Err(e) => {
                    out.push(Obs::Refused(Refusal::Load(e)));
                    return;
                }
            },
        };
        let key = RunKey {
            build: self.build.clone(),
            tape_hash: Hex(tape_hash(&tape)),
            world_id: Hex(sim.world().world_id),
        };
        out.push(Obs::Loaded {
            run: key.clone(),
            tape: Box::new(tape.clone()),
            world: Box::new(sim.world().clone()),
            tick: sim.tick(),
            hash: sim.hash(),
        });
        let mut live = Live {
            key,
            extractor: Extractor::with(sim.world(), self.catalogue),
            next_ring: ring_tick_at_or_after(&sim.world().clock, sim.tick()),
            tape,
            sim,
            mode: Mode::Paused,
            ring: Vec::new(),
            snapshots: Vec::new(),
        };
        live.reached(out);
        self.live = Some(live);
    }

    fn emit(&mut self, out: Vec<Obs>) {
        if out.is_empty() {
            return;
        }
        for o in out {
            (self.sink)(o);
        }
        (self.wake)();
    }
}

impl Live {
    /// What the state's tick reached: a ring checkpoint, and any snapshot asked for there.
    fn reached(&mut self, out: &mut Vec<Obs>) {
        let tick = self.sim.tick();
        if self.next_ring == Some(tick) {
            if let Ok(cp) = self.sim.checkpoint() {
                let bytes = cp.to_bytes();
                self.ring.push((tick, bytes.clone()));
                out.push(Obs::Checkpointed(RingCheckpoint {
                    run: self.key.clone(),
                    tick,
                    prefix_id: cp.prefix_id(),
                    bytes,
                }));
            }
            self.next_ring = tick
                .checked_add(1)
                .and_then(|t| ring_tick_at_or_after(&self.sim.world().clock, t));
        }
        while self.snapshots.first() == Some(&tick) {
            self.snapshots.remove(0);
            out.push(Obs::Snapshot(Box::new(snapshot_of(&self.sim))));
        }
    }

    /// A snapshot of tick `t`: now, when the run reaches it, or from a scratch `Sim` resumed at
    /// the latest ring checkpoint at or before it, else from genesis. The scratch run never
    /// touches this run's `Sim`.
    ///
    /// A failed step leaves the `Sim` poisoned at the tick it began, holding what the failure
    /// left: the deltas of the tick that applied before it failed. No tick of the tape left that
    /// state, so a poisoned run's own state is never read (U3). Its current tick is rebuilt on a
    /// scratch `Sim` like any earlier one, and a later tick, which it will never reach, is not
    /// asked for.
    fn snapshot(&mut self, t: u64, out: &mut Vec<Obs>) {
        let tick = self.sim.tick();
        let ready = matches!(self.sim.status(), Status::Ready);
        if t == tick && ready {
            out.push(Obs::Snapshot(Box::new(snapshot_of(&self.sim))));
            return;
        }
        if t > tick {
            if !ready {
                return;
            }
            if let Err(i) = self.snapshots.binary_search(&t) {
                self.snapshots.insert(i, t);
            }
            return;
        }
        let resumed = self
            .ring
            .iter()
            .rev()
            .find(|(c, _)| *c <= t)
            .and_then(|(_, bytes)| Checkpoint::from_bytes(bytes).ok())
            .and_then(|cp| Sim::resume(&self.tape, &cp).ok());
        let Some(mut scratch) = resumed.or_else(|| Sim::new(&self.tape).ok()) else {
            return;
        };
        if scratch.run_until(t, &mut |_| {}).is_ok() && scratch.tick() == t {
            out.push(Obs::Snapshot(Box::new(snapshot_of(&scratch))));
        }
    }
}

/// The event or date breakpoint the tick `r` reports hits, if one does: an event breakpoint
/// whose key fired in the tick, else a date breakpoint whose date falls in it (G1). The error
/// breakpoint is the failed step's, not a report's.
fn hit(breakpoints: &[Breakpoint], clock: &Clock, r: &TickReport) -> Option<Breakpoint> {
    breakpoints
        .iter()
        .find(|b| match b {
            Breakpoint::OnEvent(k) => r.events.iter().any(|e| &e.key == k),
            _ => false,
        })
        .or_else(|| {
            breakpoints.iter().find(|b| match b {
                Breakpoint::OnDate(d) => clock.tick_of(*d).ok() == Some(r.tick),
                _ => false,
            })
        })
        .cloned()
}

/// Every holding, its lots, and every actor's state of `sim`, read without changing it.
fn snapshot_of(sim: &Sim) -> Snapshot {
    let actors = sim
        .world()
        .actors
        .iter()
        .filter_map(|a| sim.actor_state(a.id).map(|s| (a.id, *s)))
        .collect();
    let holdings = sim.observe_holdings();
    let lots = holdings
        .0
        .iter()
        .map(|&(h, g, _)| {
            let lots = sim.holding(h).map(|inv| inv.lots(g).to_vec());
            (h, g, lots.unwrap_or_default())
        })
        .collect();
    Snapshot {
        tick: sim.tick(),
        hash: sim.hash(),
        holdings,
        lots,
        actors,
    }
}
