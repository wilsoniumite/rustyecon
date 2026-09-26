//! Writes the Appendix B tape, the registered setup at 52 ticks a year, to a path or stdout:
//!
//! ```sh
//! cargo run -p rustyecon-probe --bin appb-tape -- tapes/appb.ron
//! ```
//!
//! `--tpy N` writes the same setup at N ticks a year instead (for the tick-length checks).

use probe::setup::{tape_ron, Setup};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut tpy = 52;
    let mut path = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--tpy" {
            match args.next().and_then(|v| v.parse().ok()) {
                Some(v) => tpy = v,
                None => {
                    eprintln!("appb-tape: --tpy takes a whole number of ticks a year");
                    return ExitCode::FAILURE;
                }
            }
        } else {
            path = Some(a);
        }
    }
    let text = match tape_ron(&Setup::registered(tpy)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("appb-tape: {e}");
            return ExitCode::FAILURE;
        }
    };
    match path {
        Some(p) => {
            if let Err(e) = std::fs::write(&p, text) {
                eprintln!("appb-tape: cannot write {p}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
