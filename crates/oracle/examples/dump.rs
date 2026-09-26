//! Reads economies from stdin, one per line, and writes one result line per economy.
//!
//! ```text
//! echo "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1" \
//!   | cargo run --release --example dump
//! ```
//!
//! The line format and the loop are `oracle::dump` (`dump::run`). A line that is not
//! UTF-8 gives an `error=` line. A failure to read stdin or to write stdout is reported on
//! stderr and ends the run with exit status 1.

use std::io::{self, BufWriter};
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdout = BufWriter::new(io::stdout().lock());
    match oracle::dump::run(io::stdin().lock(), stdout) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("dump: {e}");
            ExitCode::FAILURE
        }
    }
}
