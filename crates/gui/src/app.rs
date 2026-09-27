//! The eframe app (docs/GUI.md §3.2): it drains the drivers into `reduce`, draws, and supplies
//! the wake callback, `ctx.request_repaint()`, that each run's worker calls after a slice.

use crate::drive::Host;
use crate::model::{Intent, Model};
use crate::platform::{self, Files};
use crate::ui::{self, layout, layout::Pane};
use std::sync::Arc;

/// What the app starts with.
#[derive(Debug, Clone, Default)]
pub struct Launch {
    /// A tape to open. Without one, the session's tapes open again.
    pub tape: Option<String>,
    /// Where `session.ron` and `layout.ron` live; `None` keeps no session.
    pub files: Option<Files>,
}

/// The GUI.
pub struct GuiApp {
    model: Model,
    host: Host,
    tree: egui_tiles::Tree<Pane>,
}

impl GuiApp {
    /// Read the session and the layout, then open the tape asked for, or the session's tapes.
    pub fn new(ctx: &egui::Context, launch: Launch) -> GuiApp {
        let wake_ctx = ctx.clone();
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || wake_ctx.request_repaint());
        let mut model = Model::default();
        let mut host = Host::new(crate::build(), wake, launch.files.clone());
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
        GuiApp { model, host, tree }
    }

    /// The model, read-only.
    pub fn model(&self) -> &Model {
        &self.model
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
}

impl eframe::App for GuiApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.host.pump(&mut self.model);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let intents = ui::draw(ui, &self.model, &mut self.tree);
        for i in intents {
            self.host.act(&mut self.model, i);
        }
    }

    /// The layout is saved when the window closes; the session whenever it changes.
    fn on_exit(&mut self) {
        self.save_layout();
    }
}
