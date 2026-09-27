//! The eframe app (docs/GUI.md §3.2): it drains the drivers into `reduce`, draws, and supplies
//! the wake callback, `ctx.request_repaint()`, that each run's worker calls after a slice.
//!
//! Smoke mode (§8.1) opens a tape and runs it to a tick with every price plotted, at a speed cap
//! of ten model years a second so that frames are drawn while the plots stream. It records the
//! CPU each frame took, running and then for a tail of paused frames, prints p50, p90 and max of
//! each with the panes drawn, and closes the window. It keeps no session, so the user's
//! `session.ron` and `layout.ron` are left alone.

use crate::drive::{FrameSummary, Frames, Host};
use crate::model::{Intent, Model};
use crate::platform::{self, Files};
use crate::run::{PauseReason, RunStatus};
use crate::ui::charts::Chart;
use crate::ui::plots::DrawnLine;
use crate::ui::{self, layout, layout::Pane};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// What the app starts with.
#[derive(Debug, Clone, Default)]
pub struct Launch {
    /// A tape to open. Without one, the session's tapes open again.
    pub tape: Option<String>,
    /// Where `session.ron` and `layout.ron` live; `None` keeps no session.
    pub files: Option<Files>,
    /// Smoke mode: run the tape to this state tick, report CPU per frame, and close.
    pub smoke: Option<u64>,
}

/// Smoke mode's progress.
struct Smoke {
    until: u64,
    started: Option<Instant>,
    /// Frames drawn while the run ran.
    running: Frames,
    /// Frames drawn after it reached `until`.
    paused: Frames,
    panes: BTreeSet<Pane>,
    report: Option<String>,
}

/// How long smoke mode waits for its run before it reports what it has.
const SMOKE_PATIENCE: Duration = Duration::from_secs(600);
/// Frames smoke mode draws after the run pauses, so a paused frame is measured too.
const SMOKE_TAIL: usize = 120;
/// Smoke mode's speed cap, in model years a second.
const SMOKE_YEARS_A_SECOND: u32 = 10;

/// The GUI.
pub struct GuiApp {
    model: Model,
    host: Host,
    tree: egui_tiles::Tree<Pane>,
    ui: ui::State,
    smoke: Option<Smoke>,
}

impl GuiApp {
    /// Read the session and the layout, then open the tape asked for, or the session's tapes.
    pub fn new(ctx: &egui::Context, launch: Launch) -> GuiApp {
        let wake_ctx = ctx.clone();
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || wake_ctx.request_repaint());
        let mut model = Model::default();
        let mut host = Host::new(crate::build(), wake, launch.files.clone());
        // The date experiments made in this session are stamped with (D3).
        if let Some(d) = platform::today() {
            host.act(&mut model, Intent::Today(d));
        }
        let mut tree = layout::default_tree();
        if let Some(files) = &launch.files {
            if let Some(text) = files.read_session() {
                host.act(&mut model, Intent::SessionRead(text));
            }
            match files.read_layout() {
                None => {}
                Some(Ok(text)) => match layout::from_ron(&text) {
                    Ok(t) => tree = t,
                    Err(why) => {
                        let kept = match files.set_aside(platform::LAYOUT) {
                            Ok(p) => format!("kept as {}", p.display()),
                            Err(e) => format!("not set aside: {e}"),
                        };
                        host.act(
                            &mut model,
                            Intent::FileFailed {
                                what: format!("layout.ron does not read ({kept}); a new layout"),
                                why,
                            },
                        );
                    }
                },
                Some(Err(why)) => host.act(
                    &mut model,
                    Intent::FileFailed {
                        what: "cannot read layout.ron".to_string(),
                        why,
                    },
                ),
            }
        }
        let tapes = match launch.tape {
            Some(t) => vec![t],
            None => model.session.bases.iter().map(|b| b.path.clone()).collect(),
        };
        for t in tapes {
            host.act(&mut model, Intent::Open(t));
        }
        let smoke = launch.smoke.map(|until| Smoke {
            until,
            started: None,
            running: Frames::default(),
            paused: Frames::default(),
            panes: BTreeSet::new(),
            report: None,
        });
        GuiApp {
            model,
            host,
            tree,
            ui: ui::State::default(),
            smoke,
        }
    }

    /// The model, read-only.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// Do what a panel would: reduce `i` and carry out its effects. Tests and scripts drive
    /// the app through this, as the user does through the panels.
    pub fn act(&mut self, i: Intent) {
        self.host.act(&mut self.model, i);
    }

    /// The panels' state: the plot cache and the panes drawn.
    pub fn ui_state(&self) -> &ui::State {
        &self.ui
    }

    /// Every line the plots lent egui in the last frame, as egui received it.
    pub fn drawn(&self) -> Vec<DrawnLine> {
        self.ui.plots.drawn()
    }

    /// Every other chart the last frame drew (the lab's curve and sweep, a market's
    /// waterfall), with what each lent egui and the transform it painted with (G1).
    pub fn charts(&self) -> Vec<Chart> {
        self.ui.charts.drawn()
    }

    /// Smoke mode's report, once it has one.
    pub fn smoke_report(&self) -> Option<&str> {
        self.smoke.as_ref().and_then(|s| s.report.as_deref())
    }

    fn save_layout(&mut self) {
        if let Some(files) = self.host.files().cloned() {
            if let Err(why) = files.write_layout(&layout::to_ron(&self.tree)) {
                self.host.act(
                    &mut self.model,
                    Intent::FileFailed {
                        what: "cannot save layout.ron".to_string(),
                        why,
                    },
                );
            }
        }
    }

    /// A picture egui took of a frame whose banner says it is never citable: encode it as a
    /// PNG whose text chunks say so too, and write it to a new file in `snapshots/` beside the
    /// session (G1). A picture nobody asked for is ignored; with no session directory there is
    /// nowhere to write, and the log says so.
    pub fn save_snapshot(&mut self, image: &egui::ColorImage) {
        if !matches!(self.ui.snapshot, ui::Snap::Waiting(_)) {
            return;
        }
        self.ui.snapshot = ui::Snap::Idle;
        let marks = ui::snapshot_marks(&self.model, &self.ui.lab);
        let rgba: Vec<u8> = image
            .pixels
            .iter()
            .flat_map(|c| c.to_srgba_unmultiplied())
            .collect();
        let [w, h] = image.size;
        let result = u32::try_from(w)
            .ok()
            .zip(u32::try_from(h).ok())
            .ok_or_else(|| "the picture is too large".to_string())
            .and_then(|(w, h)| platform::snapshot::png(&rgba, w, h, &marks))
            .and_then(|bytes| {
                let dir = self
                    .host
                    .files()
                    .map(|f| f.dir().to_path_buf())
                    .ok_or("no session directory to write it in")?;
                let date = self
                    .model
                    .today()
                    .map_or_else(|| "undated".to_string(), |d| d.to_string());
                platform::snapshot::write(&dir, &date, &bytes)
            });
        let i = match result {
            Ok(path) => Intent::Note(format!(
                "snapshot saved as {}: never citable, as its banner and its text say",
                path.display()
            )),
            Err(why) => Intent::FileFailed {
                what: "the snapshot was not saved".to_string(),
                why,
            },
        };
        self.host.act(&mut self.model, i);
    }

    /// Smoke mode, once a frame: start the run when the tape has loaded, record each frame's
    /// CPU while it runs and for a tail after it pauses, then report and close.
    fn smoke_frame(&mut self, ctx: &egui::Context, cpu: Option<f32>) {
        let Some(s) = self.smoke.as_mut() else {
            return;
        };
        if s.report.is_some() {
            return;
        }
        ctx.request_repaint();
        let run = self.model.focused();
        let status = run.map(|r| r.store.status());
        let Some(t0) = s.started else {
            if matches!(status, Some(RunStatus::Paused { why: None, .. })) {
                let tpy = run
                    .and_then(|r| r.store.world())
                    .map_or(1, |w| w.clock.ticks_per_year);
                s.started = Some(Instant::now());
                let until = Some(s.until);
                let cap = tpy.saturating_mul(SMOKE_YEARS_A_SECOND);
                self.host.act(&mut self.model, Intent::Speed(Some(cap)));
                self.host.act(&mut self.model, Intent::Run { until });
            }
            return;
        };
        let reached = matches!(
            status,
            Some(RunStatus::Paused {
                why: Some(PauseReason::Reached(_)),
                ..
            })
        );
        let stuck = matches!(
            status,
            Some(RunStatus::Poisoned { .. } | RunStatus::Stopped | RunStatus::Ended)
        );
        if let Some(c) = cpu {
            if reached || stuck {
                s.paused.record(f64::from(c));
            } else {
                s.running.record(f64::from(c));
            }
        }
        s.panes.extend(self.ui.panes_drawn().iter().copied());
        if s.paused.len() >= SMOKE_TAIL || t0.elapsed() > SMOKE_PATIENCE {
            let tick = self.model.focused().map_or(0, |r| r.store.tick());
            let report = smoke_line(s, tick, t0.elapsed(), status);
            println!("{report}");
            s.report = Some(report);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn frames_line(f: Option<FrameSummary>) -> String {
    match f {
        Some(f) => format!(
            "{} frames, p50 {:.2} ms, p90 {:.2} ms, max {:.2} ms",
            f.frames, f.p50_ms, f.p90_ms, f.max_ms
        ),
        None => "no frame timed".to_string(),
    }
}

fn smoke_line(s: &Smoke, tick: u64, took: Duration, status: Option<RunStatus>) -> String {
    let panes: Vec<&str> = s.panes.iter().map(|p| p.title()).collect();
    format!(
        "smoke: tick {tick} in {:.1} s at {SMOKE_YEARS_A_SECOND} years/s, {status:?}; CPU per \
         frame running: {}; paused: {}; panes drawn: {}",
        took.as_secs_f64(),
        frames_line(s.running.summary()),
        frames_line(s.paused.summary()),
        panes.join(", ")
    )
}

impl eframe::App for GuiApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.host.pump(&mut self.model);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let shot = ui.ctx().input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = shot {
            self.save_snapshot(&image);
        }
        let intents = ui::draw(ui, &self.model, &mut self.tree, &mut self.ui);
        for i in intents {
            self.host.act(&mut self.model, i);
        }
        let ctx = ui.ctx().clone();
        self.smoke_frame(&ctx, frame.info().cpu_usage);
    }

    /// The layout is saved when the window closes; the session whenever it changes.
    fn on_exit(&mut self) {
        self.save_layout();
    }
}
