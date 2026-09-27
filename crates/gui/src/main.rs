//! The `rustyecon-gui` binary (docs/GUI.md §3.2): `rustyecon-gui [TAPE]`. It opens the tape
//! paused at tick 0, or, with none, the tapes of the last session. The session lives in
//! `$RUSTYECON_GUI_DIR`, else the platform's configuration directory (`platform`).

use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::platform::Files;
use std::process::ExitCode;

// mimalloc cut mesh building 3–10× and the 100k-point line by half on Windows (GUI.md §3.1).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const USAGE: &str = "usage: rustyecon-gui [TAPE]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tape = match args.as_slice() {
        [] => None,
        [flag] if flag == "-h" || flag == "--help" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        [t] if !t.starts_with('-') => Some(t.clone()),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(1);
        }
    };
    let launch = Launch {
        tape,
        files: Files::default_dir().map(Files::at),
    };
    let options = eframe::NativeOptions::default();
    let result = eframe::run_native(
        "rustyecon",
        options,
        Box::new(move |cc| Ok(Box::new(GuiApp::new(&cc.egui_ctx, launch)))),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("rustyecon-gui: {e}");
            ExitCode::from(1)
        }
    }
}
