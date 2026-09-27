//! Drivers (docs/GUI.md §3.3): the threads, clocks and channels a Runner lacks.
//!
//! [`ThreadDriver`] (G0, native) spawns one worker thread per run, so a baseline and a branch run
//! side by side. The worker owns the run's [`Runner`], times its slices, applies the speed cap,
//! and calls `advance` in slices of about 8 ms. `InlineDriver` (G2, the web) calls `advance`
//! inside `poll` instead. [`Host`] carries out the model's effects: it keeps one driver per run,
//! reads and writes files through [`platform`](crate::platform), and pumps observations back
//! into [`reduce`]. [`Frames`] keeps the smoke mode's CPU per frame.

mod frames;
mod host;
mod pace;

pub use frames::{FrameSummary, Frames};
pub use host::Host;

use crate::run::{Cmd, Obs, Runner};
use certify::Build;
use pace::Pace;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

#[cfg(doc)]
use crate::model::reduce;

/// How the model reaches a run: commands in, observations out.
pub trait Driver {
    /// Send the run a command.
    fn send(&mut self, c: Cmd);
    /// Append every observation that has arrived.
    fn poll(&mut self, out: &mut Vec<Obs>);
}

/// A Runner on its own worker thread. Dropping it stops the worker and waits for it.
pub struct ThreadDriver {
    tx: Option<Sender<Cmd>>,
    rx: Receiver<Obs>,
    worker: Option<JoinHandle<()>>,
    /// Whether the worker was told to stop.
    stopped: bool,
    /// Whether the worker's end has been reported.
    ended: bool,
}

impl ThreadDriver {
    /// Spawn a worker with an empty Runner. `wake` is called after every slice or command that
    /// reported something; the app passes `ctx.request_repaint()`.
    pub fn spawn(build: Build, wake: Arc<dyn Fn() + Send + Sync>) -> ThreadDriver {
        let (tx, commands) = channel::<Cmd>();
        let (sink, rx) = channel::<Obs>();
        let worker = std::thread::Builder::new()
            .name("rustyecon-run".to_string())
            .spawn(move || work(build, &commands, sink, wake))
            .expect("the run's worker thread starts");
        ThreadDriver {
            tx: Some(tx),
            rx,
            worker: Some(worker),
            stopped: false,
            ended: false,
        }
    }
}

impl Driver for ThreadDriver {
    fn send(&mut self, c: Cmd) {
        if let Some(tx) = &self.tx {
            self.stopped |= matches!(c, Cmd::Stop);
            // A worker that has ended takes no more commands; its run is over.
            let _ = tx.send(c);
        }
    }

    /// Every observation that has arrived, and once, after the last, [`Obs::Ended`] if the
    /// worker ended without being told to stop: a Runner that panicked says nothing, and its
    /// run would otherwise look as if it ran on.
    fn poll(&mut self, out: &mut Vec<Obs>) {
        loop {
            match self.rx.try_recv() {
                Ok(o) => out.push(o),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    if !self.stopped && !self.ended {
                        out.push(Obs::Ended);
                    }
                    self.ended = true;
                    return;
                }
            }
        }
    }
}

impl Drop for ThreadDriver {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(Cmd::Stop);
        }
        if let Some(w) = self.worker.take() {
            // A worker that panicked has said so through `poll`, as `Obs::Ended`.
            let _ = w.join();
        }
    }
}

/// The worker: take commands, and while the run is busy, run slices at the paced budget.
fn work(
    build: Build,
    commands: &Receiver<Cmd>,
    sink: Sender<Obs>,
    wake: Arc<dyn Fn() + Send + Sync>,
) {
    let mut runner = Runner::new(
        build,
        Box::new(move |o| {
            // A driver that is gone reads nothing more.
            let _ = sink.send(o);
        }),
        Box::new(move || wake()),
    );
    let mut pace = Pace::default();
    let mut busy = false;
    loop {
        let mut next = if busy {
            match commands.try_recv() {
                Ok(c) => Some(c),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => return,
            }
        } else {
            match commands.recv() {
                Ok(c) => Some(c),
                Err(_) => return,
            }
        };
        while let Some(c) = next {
            match &c {
                Cmd::Stop => {
                    runner.handle(c);
                    return;
                }
                Cmd::Run { max_tps, .. } => pace.start(*max_tps, Instant::now()),
                Cmd::Step(_) => pace.start(None, Instant::now()),
                _ => {}
            }
            runner.handle(c);
            next = match commands.try_recv() {
                Ok(c) => Some(c),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => return,
            };
        }
        busy = runner.busy();
        if !busy {
            continue;
        }
        let budget = pace.budget(Instant::now());
        if budget == 0 {
            // Ahead of the speed cap: wait for the next tick, a slice at most, then look again.
            std::thread::sleep(pace.wait(Instant::now()));
            continue;
        }
        let t0 = Instant::now();
        let p = runner.advance(budget);
        pace.record(p.ran, t0.elapsed());
        busy = p.busy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Poll until `d` has reported `n` observations, a few seconds at most.
    fn poll_for(d: &mut ThreadDriver, n: usize) -> Vec<Obs> {
        let t0 = Instant::now();
        let mut out = Vec::new();
        while out.len() < n && t0.elapsed() < Duration::from_secs(10) {
            d.poll(&mut out);
            std::thread::sleep(Duration::from_millis(1));
        }
        out
    }

    #[test]
    fn a_worker_that_ends_unasked_is_reported_once() {
        // A worker that ends with no Stop, as a Runner that panics does, drops its sink and
        // says nothing. The driver reports its end once, after what it had said.
        let (tx, _commands) = channel::<Cmd>();
        let (sink, rx) = channel::<Obs>();
        let worker = std::thread::spawn(move || {
            let _ = sink.send(Obs::Running { tick: 0 });
        });
        let mut d = ThreadDriver {
            tx: Some(tx),
            rx,
            worker: Some(worker),
            stopped: false,
            ended: false,
        };
        let seen = poll_for(&mut d, 2);
        assert!(
            matches!(seen[..], [Obs::Running { tick: 0 }, Obs::Ended]),
            "{seen:?}"
        );
        let mut again = Vec::new();
        d.poll(&mut again);
        assert!(again.is_empty(), "reported once: {again:?}");
    }

    #[test]
    fn a_worker_told_to_stop_ends_in_silence() {
        let mut d = ThreadDriver::spawn(crate::build(), Arc::new(|| {}));
        d.send(Cmd::Stop);
        d.worker
            .take()
            .expect("the worker")
            .join()
            .expect("it stops");
        let mut seen = Vec::new();
        d.poll(&mut seen);
        assert!(seen.is_empty(), "{seen:?}");
    }
}
