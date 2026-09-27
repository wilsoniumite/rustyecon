//! The `rustyecon-gui` binary (docs/GUI.md §3.2): `rustyecon-gui [TAPE]`. It opens the tape
//! paused at tick 0 with every price plotted, or, with none, the tapes of the last session. The
//! session lives in `$RUSTYECON_GUI_DIR`, else the platform's configuration directory
//! (`platform`).
//!
//! `rustyecon-gui --smoke UNTIL TAPE` is the smoke mode (§8.1): it opens TAPE, runs it to state
//! tick UNTIL at ten model years a second with every price plotted, prints the CPU each frame
//! took (p50, p90, max) while running and then paused, with the panes drawn, and closes. It
//! reads and writes no session.

use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::platform::Files;
use std::process::ExitCode;

// mimalloc cut mesh building 3–10× and the 100k-point line by half on Windows (GUI.md §3.1).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const USAGE: &str = "usage: rustyecon-gui [TAPE]\n       rustyecon-gui --smoke UNTIL TAPE";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let launch = match args.as_slice() {
        [] => Launch {
            tape: None,
            files: Files::default_dir().map(Files::at),
            smoke: None,
        },
        [flag] if flag == "-h" || flag == "--help" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        [t] if !t.starts_with('-') => Launch {
            tape: Some(t.clone()),
            files: Files::default_dir().map(Files::at),
            smoke: None,
        },
        [flag, until, t] if flag == "--smoke" && !t.starts_with('-') => match until.parse() {
            Ok(u) => Launch {
                tape: Some(t.clone()),
                files: None,
                smoke: Some(u),
            },
            Err(e) => {
                eprintln!("rustyecon-gui: --smoke {until}: {e}\n{USAGE}");
                return ExitCode::from(1);
            }
        },
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(1);
        }
    };
    let smoke = launch.smoke.is_some();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rustyecon")
            .with_inner_size([1600.0, 1000.0]),
        ..eframe::NativeOptions::default()
    };
    let result = eframe::run_native(
        "rustyecon",
        options,
        Box::new(move |cc| Ok(Box::new(GuiApp::new(&cc.egui_ctx, launch)))),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("rustyecon-gui: {e}");
            ExitCode::from(if smoke { 2 } else { 1 })
        }
    }
}
