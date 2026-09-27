//! View-models (docs/GUI.md §3.2): one pure builder per panel. A builder reads run types (a
//! [`Store`](crate::run::Store), log lines) and returns plain data, `Serialize + Debug`, which a
//! panel in `ui/` draws and a golden test pins. No egui, and nothing from `model/`, so the
//! builders move into `crates/observe` with the Runner at G2 (D13).
//!
//! G0.1 builds the toolbar's identity, clock and health, and the log. The other panels' builders
//! join with their panels.

pub mod log;
pub mod toolbar;
