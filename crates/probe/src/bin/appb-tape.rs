//! Writes the Appendix B tape, the registered setup at 52 ticks a year, to a path or stdout:
//!
//! ```sh
//! cargo run -p rustyecon-probe --bin appb-tape -- tapes/appb.ron
//! ```
//!
//! Options (`probe::setup::TapeArgs`): `--tpy N` writes the setup at N ticks a year (for the
//! tick-length checks); `--set KEY=VALUE`, `--assign`, `--scale` and `--one-sided` vary it as
//! `probe` does; `--perturb NAME` applies a named run, with `--ticks L` dating a `b=B@dated`
//! shock at L/4. Certify's testdata is written this way (docs/CERTIFY.md §13), each file opening
//! with a comment naming its options.

use probe::setup::TapeArgs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (text, path) = match TapeArgs::parse(&args).and_then(|a| Ok((a.text()?, a.path))) {
        Ok(x) => x,
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
