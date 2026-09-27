//! The host of the drivers: it carries out the model's effects and pumps each run's
//! observations back into [`reduce`]. The app calls it every frame; tests call it headless.

use super::{Driver, ThreadDriver};
use crate::edit::lineage_path;
use crate::model::{reduce, Effect, Intent, Model};
use crate::platform::{self, Files};
use crate::run::RunId;
use certify::Build;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

/// One driver per run, and the files the session lives in.
pub struct Host {
    build: Build,
    wake: Arc<dyn Fn() + Send + Sync>,
    files: Option<Files>,
    drivers: BTreeMap<RunId, ThreadDriver>,
}

impl Host {
    /// A host whose runs name `build` and call `wake` when they report. With `files`, the
    /// session is saved there whenever it changes; without, never.
    pub fn new(build: Build, wake: Arc<dyn Fn() + Send + Sync>, files: Option<Files>) -> Host {
        Host {
            build,
            wake,
            files,
            drivers: BTreeMap::new(),
        }
    }

    /// Where the session lives, if anywhere.
    pub fn files(&self) -> Option<&Files> {
        self.files.as_ref()
    }

    /// Reduce `i` and carry out every effect it leads to.
    pub fn act(&mut self, m: &mut Model, i: Intent) {
        let effects = reduce(m, i);
        self.apply(m, effects);
    }

    /// Carry out `effects`, and any that the intents they answer with lead to, in order.
    pub fn apply(&mut self, m: &mut Model, effects: Vec<Effect>) {
        let mut queue: VecDeque<Effect> = effects.into();
        while let Some(e) = queue.pop_front() {
            let answer = match e {
                Effect::PickTape => platform::pick_tape().map(Intent::Open),
                Effect::ReadTape(path) => Some(Intent::TapeRead {
                    text: platform::read_text(&path),
                    lineage: platform::read_if_there(lineage_path(&path)),
                    path,
                }),
                Effect::PickSaveTape => platform::pick_save_tape().map(Intent::SaveTape),
                Effect::PickExportDir => platform::pick_dir().map(Intent::Export),
                Effect::Write { job, files } => Some(Intent::Written {
                    job,
                    result: platform::write_new(&files),
                }),
                Effect::Spawn(run) => {
                    let d = ThreadDriver::spawn(self.build.clone(), Arc::clone(&self.wake));
                    self.drivers.insert(run, d);
                    None
                }
                Effect::Send { run, cmd } => {
                    if let Some(d) = self.drivers.get_mut(&run) {
                        d.send(cmd);
                    }
                    None
                }
                Effect::Close(run) => {
                    // Dropping the driver stops its worker.
                    self.drivers.remove(&run);
                    None
                }
                Effect::SetAsideSession => {
                    self.files
                        .as_ref()
                        .map(|f| match f.set_aside(platform::SESSION) {
                            Ok(to) => {
                                Intent::Note(format!("the old session is kept as {}", to.display()))
                            }
                            Err(why) => Intent::FileFailed {
                                what: "cannot set session.ron aside".to_string(),
                                why,
                            },
                        })
                }
                Effect::SaveSession => self.files.as_ref().and_then(|f| {
                    f.write_session(&m.session.to_ron())
                        .err()
                        .map(|why| Intent::FileFailed {
                            what: format!("cannot save {}", f.session_path().display()),
                            why,
                        })
                }),
            };
            if let Some(i) = answer {
                queue.extend(reduce(m, i));
            }
        }
    }

    /// Hand every observation that has arrived to the model; returns how many there were.
    pub fn pump(&mut self, m: &mut Model) -> usize {
        let mut arrived = Vec::new();
        for (run, d) in &mut self.drivers {
            let mut obs = Vec::new();
            d.poll(&mut obs);
            arrived.extend(obs.into_iter().map(|o| (*run, o)));
        }
        let n = arrived.len();
        for (run, obs) in arrived {
            self.act(m, Intent::Observed { run, obs });
        }
        n
    }

    /// The number of live drivers.
    pub fn drivers(&self) -> usize {
        self.drivers.len()
    }
}
