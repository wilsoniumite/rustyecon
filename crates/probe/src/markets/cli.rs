//! The options `markets-tape` and `markets` share: the instance, the tick length, the dial set
//! and changes to it, and the one-sided rule, applied to a registered setup in that order.

use super::setup::{Dials, Setup};
use crate::setup::OneSided;

/// A setup's options, parsed from the command line, with what was not an option.
#[derive(Debug, Clone, PartialEq)]
pub struct SetupArgs {
    /// The options as given, in order.
    pub given: Vec<String>,
    /// The setup they make.
    pub setup: Setup,
    /// Everything that was not a setup option, in order.
    pub rest: Vec<String>,
}

/// The setup options: `--inst ID` (default i1), `--tpy N` (default 52), `--dials c2m|c2l`
/// (default c2m), `--set KEY=VALUE` (repeatable; `rate.*`, `buffer.*`, `adjust.*` and, at a
/// switch instance, `rate.switch.*` scale a family, `tilt.*` sets every tilt),
/// `--one-sided saturate|hold`. Options in `takes_value` are
/// kept, with their values, in `rest`.
pub fn parse(args: &[String], takes_value: &[&str]) -> Result<SetupArgs, String> {
    let mut inst = "i1".to_string();
    let mut tpy = 52;
    let mut dials = "c2m".to_string();
    let mut sets: Vec<(String, f64)> = Vec::new();
    let mut one_sided = OneSided::Saturate;
    let mut given = Vec::new();
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{a} takes a value"))
        };
        match a.as_str() {
            "--inst" | "--tpy" | "--dials" | "--set" | "--one-sided" => {
                let v = val()?;
                given.push(a.clone());
                given.push(v.clone());
                match a.as_str() {
                    "--inst" => inst = v,
                    "--tpy" => tpy = v.parse().map_err(|_| "--tpy takes a whole number")?,
                    "--dials" => dials = v,
                    "--set" => {
                        let (k, x) = v.split_once('=').ok_or("--set KEY=VALUE")?;
                        let x: f64 = x.parse().map_err(|_| format!("--set {v}: not a number"))?;
                        sets.push((k.to_string(), x));
                    }
                    _ => {
                        one_sided = match v.as_str() {
                            "saturate" => OneSided::Saturate,
                            "hold" => OneSided::Hold,
                            x => return Err(format!("--one-sided {x}: saturate or hold")),
                        }
                    }
                }
            }
            x if takes_value.contains(&x) => {
                let v = val()?;
                rest.push(a.clone());
                rest.push(v);
            }
            _ => rest.push(a.clone()),
        }
    }
    let mut setup = Setup::registered(&inst, tpy)?;
    setup.dials = Dials::named(&dials, &setup.instance)?;
    for (k, x) in sets {
        setup.dials.set(&k, x)?;
    }
    setup.one_sided = one_sided;
    Ok(SetupArgs { given, setup, rest })
}
