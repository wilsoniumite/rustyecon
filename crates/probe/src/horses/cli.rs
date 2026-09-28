//! The options `horses-tape` and `horses` share: the instance, the tick length, the dial set and
//! changes to it, the rule variants and the one-sided rule, applied to a registered setup in
//! that order.

use super::setup::{dials, Setup};
use crate::setup::OneSided;
use rustyecon_engine::rustyecon_agents::{Assign, OrderRule};

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

/// The setup options: `--inst ID` (default h1), `--tpy N` (default 52), `--dials
/// c2g|c2g13|c2` (default the instance's), `--set KEY=VALUE` (repeatable; `rate.*`, `buffer.*`
/// and `adjust.*` scale a family, `tilt.*` sets every tilt), `--assign planned|expost`,
/// `--order target|held`, `--cover yes|none`, `--one-sided saturate|hold`. Options in
/// `takes_value` are kept, with their values, in `rest`.
pub fn parse(args: &[String], takes_value: &[&str]) -> Result<SetupArgs, String> {
    let mut inst = "h1".to_string();
    let mut tpy = 52;
    let mut dial_set: Option<String> = None;
    let mut sets: Vec<(String, f64)> = Vec::new();
    let mut one_sided = OneSided::Saturate;
    let mut assign: Option<Assign> = None;
    let mut order = OrderRule::Target;
    let mut cover = true;
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
            "--inst" | "--tpy" | "--dials" | "--set" | "--one-sided" | "--assign" | "--order"
            | "--cover" => {
                let v = val()?;
                given.push(a.clone());
                given.push(v.clone());
                match a.as_str() {
                    "--inst" => inst = v,
                    "--tpy" => tpy = v.parse().map_err(|_| "--tpy takes a whole number")?,
                    "--dials" => dial_set = Some(v),
                    "--set" => {
                        let (k, x) = v.split_once('=').ok_or("--set KEY=VALUE")?;
                        let x: f64 = x.parse().map_err(|_| format!("--set {v}: not a number"))?;
                        sets.push((k.to_string(), x));
                    }
                    "--assign" => {
                        assign = Some(match v.as_str() {
                            "planned" => Assign::Planned,
                            "expost" => Assign::ExPost,
                            x => return Err(format!("--assign {x}: planned or expost")),
                        })
                    }
                    "--order" => {
                        order = match v.as_str() {
                            "target" => OrderRule::Target,
                            "held" => OrderRule::Held,
                            x => return Err(format!("--order {x}: target or held")),
                        }
                    }
                    "--cover" => {
                        cover = match v.as_str() {
                            "yes" => true,
                            "none" => false,
                            x => return Err(format!("--cover {x}: yes or none")),
                        }
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
    if let Some(d) = dial_set {
        setup.dials = dials(&setup.instance, &d)?;
    }
    for (k, x) in sets {
        setup.dials.set(&k, x)?;
    }
    setup.one_sided = one_sided;
    if let Some(a) = assign {
        setup.assign = a;
    }
    setup.order = order;
    setup.cover = cover;
    Ok(SetupArgs { given, setup, rest })
}
