//! A host on the test's own thread, for the editor's tests: one Runner per run, advanced until
//! idle after every effect, and files read and written through `platform` as the real host
//! does, so a test drives the model through branches, saves and exports deterministically.

use super::{collecting, drain};
use rustyecon_gui::edit::lineage_path;
use rustyecon_gui::model::{reduce, Effect, Intent, Job, Model};
use rustyecon_gui::platform;
use rustyecon_gui::run::{Cmd, Obs, ResumeFrom, RunId, Runner};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// A Runner and what its sink collected.
type Collected = (Runner, Arc<Mutex<Vec<Obs>>>);

/// The host.
#[derive(Default)]
pub struct SyncHost {
    runners: BTreeMap<RunId, Collected>,
    /// Each `Load` sent, with the ring tick it resumes from, if any.
    pub loads: Vec<(RunId, Option<u64>)>,
    /// Each file job written, with its result.
    pub written: Vec<(Job, Result<Vec<String>, String>)>,
    slice: u32,
}

impl SyncHost {
    /// A host that advances runners `slice` ticks at a time.
    pub fn new(slice: u32) -> SyncHost {
        SyncHost {
            slice,
            ..SyncHost::default()
        }
    }

    /// Reduce `i`, carry out its effects, and settle every runner; returns the effects.
    pub fn act(&mut self, m: &mut Model, i: Intent) -> Vec<Effect> {
        let effects = reduce(m, i);
        self.apply(m, effects.clone());
        effects
    }

    fn apply(&mut self, m: &mut Model, effects: Vec<Effect>) {
        for e in effects {
            let answer = match e {
                Effect::Spawn(run) => {
                    self.runners.insert(run, collecting());
                    None
                }
                Effect::Send { run, cmd } => {
                    if let Cmd::Load { from, .. } = &cmd {
                        let tick = from.as_ref().map(|ResumeFrom::Ring(cp)| cp.tick());
                        self.loads.push((run, tick));
                    }
                    if let Some((r, _)) = self.runners.get_mut(&run) {
                        r.handle(cmd);
                    }
                    None
                }
                Effect::Close(run) => {
                    self.runners.remove(&run);
                    None
                }
                Effect::ReadTape(path) => Some(Intent::TapeRead {
                    text: platform::read_text(&path),
                    lineage: platform::read_if_there(lineage_path(&path)),
                    path,
                }),
                Effect::ReadAncestor { run, path } => Some(Intent::AncestorRead {
                    run,
                    text: platform::read_text(&path),
                    path,
                }),
                Effect::Write { job, files } => {
                    let result = platform::write_new(&files);
                    self.written.push((job.clone(), result.clone()));
                    Some(Intent::Written { job, result })
                }
                Effect::SaveSession
                | Effect::SetAsideSession
                | Effect::PickTape
                | Effect::PickSaveTape
                | Effect::PickExportDir => None,
            };
            if let Some(i) = answer {
                let e = reduce(m, i);
                self.apply(m, e);
            }
        }
        self.settle(m);
    }

    /// Advance every runner until idle, feeding each observation back.
    pub fn settle(&mut self, m: &mut Model) {
        loop {
            let mut arrived = Vec::new();
            for (id, (r, seen)) in &mut self.runners {
                while r.advance(self.slice).busy {}
                arrived.extend(drain(seen).into_iter().map(|o| (*id, o)));
            }
            if arrived.is_empty() {
                return;
            }
            for (run, obs) in arrived {
                let e = reduce(m, Intent::Observed { run, obs });
                self.apply(m, e);
            }
        }
    }
}
