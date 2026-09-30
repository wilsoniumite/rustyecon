//! Writes a markets probe tape (MARKETS-SPEC §2; docs/probe/MARKETS-RULES.md), the registered
//! setup of an instance at 52 ticks a year, to a path or stdout:
//!
//! ```sh
//! cargo run -p rustyecon-probe --bin markets-tape -- --inst i1 tapes/markets-i1.ron
//! ```
//!
//! Options (`probe::markets::cli`): `--inst ID` (i0, i1, i2, i3, l2, l3, g1, iw1, ic1, c1, c2,
//! c1n, c1p, c2p, c1pn; default i1),
//! `--tpy N`, `--dials c2m|c2l`, `--set KEY=VALUE`, `--one-sided saturate|hold`; `--perturb NAME`
//! applies a named run, with `--ticks L` dating a `C=V@dated` shock at L/4. With no option but
//! `--inst` the text is the registered tape, `tapes/markets-<id>.ron`; with any other, it opens
//! with a comment naming them.

use probe::markets::cli::parse;
use probe::markets::perturb::Perturbation;
use probe::markets::setup::tape_ron;
use probe::protocol::RUN_TICKS;
use std::process::ExitCode;

fn text(args: &[String]) -> Result<(String, Option<String>), String> {
    let a = parse(args, &["--perturb", "--ticks"])?;
    let mut setup = a.setup;
    let mut ticks = RUN_TICKS;
    let mut perturb = Vec::new();
    let mut path = None;
    let mut given: Vec<String> = a
        .given
        .chunks(2)
        .filter(|c| c[0] != "--inst")
        .flat_map(|c| c.to_vec())
        .collect();
    let mut it = a.rest.iter();
    while let Some(x) = it.next() {
        match x.as_str() {
            "--ticks" => {
                let v = it.next().ok_or("--ticks takes a value")?;
                ticks = v.parse().map_err(|_| "--ticks takes a whole number")?;
                given.extend([x.clone(), v.clone()]);
            }
            "--perturb" => {
                let v = it.next().ok_or("--perturb takes a value")?;
                perturb.push(v.clone());
                given.extend([x.clone(), v.clone()]);
            }
            o if o.starts_with("--") => return Err(format!("unknown option {o}")),
            p => path = Some(p.to_string()),
        }
    }
    for name in &perturb {
        Perturbation::parse(name)?.apply(&mut setup, ticks)?;
    }
    let tape = tape_ron(&setup)?;
    if given.is_empty() {
        return Ok((tape, path));
    }
    Ok((
        format!(
            "// Written by `markets-tape --inst {} {}` (crates/probe).\n{tape}",
            setup.instance.id,
            given.join(" ")
        ),
        path,
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (text, path) = match text(&args) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("markets-tape: {e}");
            return ExitCode::FAILURE;
        }
    };
    match path {
        Some(p) => {
            if let Err(e) = std::fs::write(&p, text) {
                eprintln!("markets-tape: cannot write {p}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
