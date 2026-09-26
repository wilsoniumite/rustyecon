//! The kick check (docs/CERTIFY.md §7; C1; REPORT §5 and §6 criterion 1): at the run's end and
//! at every dated shock (amended at S2.5), the base run's checkpoint resumes under tapes that
//! differ from the base only by one dated price shock, and the kick must decay.
//!
//! A kick is a tape edit with provenance, never a state edit (E1): [`deferred_tape`] moves every
//! dated event from the kick's tick on past the horizon, and [`kicked_tape`] adds a param only
//! the schedule reads and one `ScalePrice` event at the kick's tick. Neither changes the world or
//! the past before the kick, so the checkpoint resumes, and a kicked run equals the kicked tape's
//! run from genesis (`kicked_tape_keeps_world_and_past`, engine). Keys under `certify.` are
//! reserved for this.

use crate::battery::{KickBars, KickSegment, KickedRun, Sign};
use crate::manifest::tape_hash;
use crate::obs::Names;
use rustyecon_core::tape::raw::{RawAct, RawEvent, RawParam};
use rustyecon_core::{Basis, Clock, Key, Unit};
use rustyecon_engine::prelude::{Checkpoint, Sim, Tape, World};
use std::fmt;

/// The prefix of every key certify adds to a tape; a base tape that uses one does not certify.
pub const RESERVED: &str = "certify.";

/// The kick's event.
pub const KICK_EVENT: &str = "certify.kick";

/// The kick's factor, a param only the schedule reads.
pub const KICK_FACTOR: &str = "certify.kick.factor";

/// A kick that cannot be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KickError(pub String);

impl fmt::Display for KickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KickError {}

fn clock(t: &Tape) -> Clock {
    Clock {
        start: t.header.start,
        ticks_per_year: t.header.ticks_per_year,
    }
}

/// Every key of a tape under [`RESERVED`], by its kind: params, events and recurring entries,
/// goods, nodes, channels, classes and actors.
pub fn reserved_keys(t: &Tape) -> Vec<String> {
    let mut keys: Vec<&Key> = Vec::new();
    keys.extend(t.params.iter().map(|p| &p.key));
    keys.extend(t.events.iter().map(|e| &e.key));
    keys.extend(t.recurring.iter().map(|e| &e.key));
    keys.extend(t.goods.iter().map(|g| &g.key));
    keys.extend(t.nodes.iter().map(|n| &n.key));
    keys.extend(t.channels.iter().map(|c| &c.key));
    keys.extend(t.classes.iter());
    keys.extend(t.actors.iter().map(|a| &a.key));
    keys.iter()
        .filter(|k| k.as_str().starts_with(RESERVED))
        .map(|k| k.to_string())
        .collect()
}

/// The base tape with every dated event at a tick at or after `t` re-dated to `date_of(until)`,
/// a tick no kicked run reaches, and nothing else changed (§7 step 2). Every param keeps every
/// reference, so the registry and `world_id` are unchanged, and so is `prefix_id(t)`, which
/// covers the firings before `t`. Recurring entries stay.
pub fn deferred_tape(base: &Tape, t: u64, until: u64) -> Result<Tape, KickError> {
    let c = clock(base);
    let later = c
        .date_of(until)
        .ok_or_else(|| KickError(format!("tick {until} has no date")))?;
    let mut out = base.clone();
    for e in &mut out.events {
        let at = c
            .tick_of(e.at)
            .map_err(|err| KickError(format!("events[{}]: {err}", e.key)))?;
        if at >= t {
            e.at = later;
        }
    }
    Ok(out)
}

/// The deferred tape with a kick at `t`: the schedule param [`KICK_FACTOR`], valued `factor`
/// with `basis`, and the event [`KICK_EVENT`] at `date_of(t)` doing `ScalePrice(node, good, by:
/// KICK_FACTOR)` (§7 step 3). A tick whose first date falls in another tick cannot be kicked.
pub fn kicked_tape(
    deferred: &Tape,
    t: u64,
    node: &Key,
    good: &Key,
    factor: f64,
    basis: Basis,
) -> Result<Tape, KickError> {
    let c = clock(deferred);
    let date = c
        .date_of(t)
        .ok_or_else(|| KickError(format!("tick {t} has no date")))?;
    if c.tick_of(date).ok() != Some(t) {
        return Err(KickError(format!(
            "tick {t}'s first date, {date}, falls in another tick"
        )));
    }
    let key = |k: &str| Key::new(k).map_err(|e| KickError(e.to_string()));
    let mut out = deferred.clone();
    out.params.push(RawParam {
        key: key(KICK_FACTOR)?,
        value: factor,
        unit: Unit::Dimensionless,
        basis: basis.clone(),
    });
    out.events.push(RawEvent {
        key: key(KICK_EVENT)?,
        at: date,
        basis,
        act: RawAct::ScalePrice {
            node: node.clone(),
            good: good.clone(),
            by: key(KICK_FACTOR)?,
        },
    });
    Ok(out)
}

/// Every market's posted price on each of `h` ticks of a run of `tape` resumed from `cp`.
fn prices(tape: &Tape, cp: &Checkpoint, h: u64) -> Result<Vec<Vec<f64>>, String> {
    let mut sim = Sim::resume(tape, cp).map_err(|e| format!("the resume is refused: {e}"))?;
    let mut out = Vec::new();
    let until = cp.state().tick().saturating_add(h);
    sim.run_until(until, &mut |r| {
        out.push(r.markets.iter().map(|l| l.price).collect());
    })
    .map_err(|e| format!("the run failed: {e}"))?;
    Ok(out)
}

/// Whether a tape keeps the base world's identity and its past before `t`: the same `world_id`,
/// the same registry, and the same `prefix_id(t)`.
fn same_world(tape: &Tape, base: &World, t: u64) -> Result<(), String> {
    let w = Sim::new(tape).map_err(|e| format!("the tape does not load: {e}"))?;
    let w = w.world();
    if w.world_id != base.world_id {
        return Err(format!(
            "it moves world_id from 0x{:016x} to 0x{:016x}",
            base.world_id, w.world_id
        ));
    }
    if w.registry.params() != base.registry.params() {
        return Err("it changes the registry".to_string());
    }
    if w.prefix_id(t) != base.prefix_id(t) {
        return Err(format!("it changes prefix_id({t})"));
    }
    Ok(())
}

/// The kicks at one tick (§7): the base continuation and ± each market, each resumed from `cp`,
/// the base run's checkpoint at that tick (the run's end or a dated shock), for the horizon, and
/// `segment` the segment whose regime they probe. Every failure is
/// in the result, never an error: a checkpoint of another world, a tape that does not load, a
/// refused resume, a run that fails.
pub fn kick_segment(
    base: &Tape,
    world: &World,
    cp: &Checkpoint,
    segment: usize,
    bars: &KickBars,
    file: &str,
) -> KickSegment {
    let at = cp.state().tick();
    let runs = (|| {
        let deferred = deferred_tape(base, at, at.saturating_add(bars.horizon))
            .map_err(|e| format!("the deferred tape: {e}"))?;
        same_world(&deferred, world, at).map_err(|e| format!("the deferred tape: {e}"))?;
        let base_prices = prices(&deferred, cp, bars.horizon)
            .map_err(|e| format!("the base continuation: {e}"))?;
        let names = Names::of(world);
        let basis = Basis::Literature(format!(
            "{file}: the kick of docs/CERTIFY.md §7 and docs/probe/REPORT.md §5"
        ));
        let mut runs = Vec::new();
        for (m, (node, good)) in world.markets().enumerate() {
            let (Some(nk), Some(gk)) = (world.key_of(node), world.key_of(good)) else {
                return Err(format!("{} has no keys", names.market(m)));
            };
            for sign in [Sign::Up, Sign::Down] {
                let factor = match sign {
                    Sign::Up => 1.0 + bars.size,
                    Sign::Down => 1.0 - bars.size,
                };
                let run = kicked_tape(&deferred, at, nk, gk, factor, basis.clone())
                    .map_err(|e| e.to_string())
                    .and_then(|t| {
                        same_world(&t, world, at)?;
                        Ok(t)
                    });
                let (hash, prices) = match run {
                    Ok(t) => (tape_hash(&t), prices(&t, cp, bars.horizon)),
                    Err(e) => (0, Err(format!("the kicked tape: {e}"))),
                };
                runs.push(KickedRun {
                    market: m,
                    sign,
                    tape_hash: hash,
                    prices,
                });
            }
        }
        Ok((base_prices, runs))
    })();
    KickSegment { segment, at, runs }
}
