//! Writes a stocks-probe tape (HORSES-SPEC §2; docs/probe/HORSES-RULES.md), the registered setup
//! of an instance at 52 ticks a year, to a path or stdout:
//!
//! ```sh
//! cargo run -p rustyecon-probe --bin horses-tape -- --inst h1 tapes/horses-h1.ron
//! ```
//!
//! Options (`probe::horses::cli`): `--inst ID` (h1-h4, f1-f10, r1a, p7, p8; default h1),
//! `--tpy N`, `--dials c2g|c2g13|c2`, `--set KEY=VALUE`, `--assign planned|expost`, `--order
//! target|held`, `--cover yes|none`, `--one-sided saturate|hold`, `--reserve PSI` (the maker's
//! reservation, L0.4: the param `reserve.<maker>` and the maker's field); `--perturb NAME` applies a
//! named run, with `--ticks L` dating a dated shock at L/4. With no option but `--inst` the text
//! is the registered tape, `tapes/horses-<id>.ron`; with any other, it opens with a comment
//! naming them.
//!
//! A loop instance (P2.2b; docs/probe/LOOPS-RULES.md §7), `--inst lb1` and the rest of §7.1's
//! ids, writes `tapes/loops-<id>.ron` for the registered setup, with the options of
//! `probe::horses::cli::parse_loops`: `--tpy N`, `--dials c2g|c2g13`, `--one-sided
//! saturate|hold`, `--reserve PSI|none`, `--theta X`, `--plant-delta D` and `--fixed-plants`.

use probe::horses::cli::{inst_of, parse, parse_loops};
use probe::horses::loops::{tape_ron as loop_tape_ron, Instance as LoopInstance};
use probe::horses::perturb::Perturbation;
use probe::horses::setup::tape_ron;
use probe::protocol::RUN_TICKS;
use std::process::ExitCode;

/// A loop instance's tape, and the path to write it to.
fn loop_text(args: &[String]) -> Result<(String, Option<String>), String> {
    let a = parse_loops(args, &[])?;
    let given: Vec<String> = a
        .given
        .iter()
        .enumerate()
        .filter(|(i, x)| *x != "--inst" && !(*i > 0 && a.given[*i - 1] == "--inst"))
        .map(|(_, x)| x.clone())
        .collect();
    let mut path = None;
    for x in &a.rest {
        if x.starts_with("--") {
            return Err(format!("unknown option {x}"));
        }
        path = Some(x.clone());
    }
    let tape = loop_tape_ron(&a.setup)?;
    if given.is_empty() {
        return Ok((tape, path));
    }
    Ok((
        format!(
            "// Written by `horses-tape --inst {} {}` (crates/probe).\n{tape}",
            a.setup.instance.id,
            given.join(" ")
        ),
        path,
    ))
}

fn text(args: &[String]) -> Result<(String, Option<String>), String> {
    if inst_of(args).is_some_and(LoopInstance::is_loop) {
        return loop_text(args);
    }
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
            "// Written by `horses-tape --inst {} {}` (crates/probe).\n{tape}",
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
            eprintln!("horses-tape: {e}");
            return ExitCode::FAILURE;
        }
    };
    match path {
        Some(p) => {
            if let Err(e) = std::fs::write(&p, text) {
                eprintln!("horses-tape: cannot write {p}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{text}"),
    }
    ExitCode::SUCCESS
}
