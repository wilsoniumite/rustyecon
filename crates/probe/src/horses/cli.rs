//! The options `horses-tape` and `horses` share: the instance, the tick length, the dial set and
//! changes to it, the rule variants and the one-sided rule, applied to a registered setup in
//! that order; a loop instance's (P2.2b), which `horses-tape` takes when `--inst` names one; and
//! a demo county's (D2.4), `--inst demo:<key>@<year>`, read from `--counties PATH`.

use super::demo;
use super::loops::{dials as loop_dials, Setup as LoopSetup};
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
/// `--order target|held`, `--cover yes|none`, `--one-sided saturate|hold`, `--reserve PSI` (the
/// maker's reservation, L0.4; absent, the tape is P2.2a's), `--counties PATH` (the demo's county
/// table, D2.4: `--inst demo:<key>@<year>` names one of its rows, whose ψ is the reservation
/// unless `--reserve` gives another). Options in `takes_value` are kept, with their values, in
/// `rest`.
pub fn parse(args: &[String], takes_value: &[&str]) -> Result<SetupArgs, String> {
    let mut inst = "h1".to_string();
    let mut counties: Option<String> = None;
    let mut tpy = 52;
    let mut dial_set: Option<String> = None;
    let mut sets: Vec<(String, f64)> = Vec::new();
    let mut one_sided = OneSided::Saturate;
    let mut assign: Option<Assign> = None;
    let mut order = OrderRule::Target;
    let mut cover = true;
    let mut reserve: Option<f64> = None;
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
            | "--cover" | "--reserve" | "--counties" => {
                let v = val()?;
                given.push(a.clone());
                given.push(v.clone());
                match a.as_str() {
                    "--inst" => inst = v,
                    "--counties" => counties = Some(v),
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
                    "--reserve" => {
                        let psi: f64 = v
                            .parse()
                            .map_err(|_| format!("--reserve {v}: not a number"))?;
                        reserve = Some(psi);
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
    let mut setup = if demo::is_demo(&inst) {
        let path = counties.as_deref().ok_or_else(|| {
            format!("--inst {inst} needs --counties PATH, the demo's county table")
        })?;
        let row = demo::load(path, &inst)?;
        reserve = reserve.or(Some(row.psi));
        Setup::of(row.instance, tpy)?
    } else {
        if counties.is_some() {
            return Err(format!("--counties is for a demo instance, not {inst}"));
        }
        Setup::registered(&inst, tpy)?
    };
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
    setup.reserve = reserve;
    Ok(SetupArgs { given, setup, rest })
}

/// The instance `--inst` names in `args`, if any.
pub fn inst_of(args: &[String]) -> Option<&str> {
    args.iter()
        .position(|a| a == "--inst")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// A loop setup's options (P2.2b; LOOPS-RULES §8.4), parsed from the command line, with what was
/// not an option.
#[derive(Debug, Clone, PartialEq)]
pub struct LoopArgs {
    /// The options as given, in order.
    pub given: Vec<String>,
    /// The setup they make.
    pub setup: LoopSetup,
    /// Everything that was not a setup option, in order.
    pub rest: Vec<String>,
}

/// The loop setup options: `--inst ID` (a loop id), `--tpy N` (default 52), `--dials
/// c2g|c2g13`, `--one-sided saturate|hold`, `--reserve PSI|none` (the maker's reservation; the
/// instance's by default, 0.25 wet), `--theta X` and `--plant-delta D` (every plant's θ or δ_p a
/// year; its size, s_Kp and genesis follow), and `--fixed-plants` (δ_p 0 on every plant, genesis
/// at the design's plants), applied in that order. Options in `takes_value` are kept, with their
/// values, in `rest`.
pub fn parse_loops(args: &[String], takes_value: &[&str]) -> Result<LoopArgs, String> {
    let mut inst = "lb1".to_string();
    let mut tpy = 52;
    let mut dial_set: Option<String> = None;
    let mut one_sided = OneSided::Saturate;
    let mut reserve: Option<Option<f64>> = None;
    let mut theta: Option<f64> = None;
    let mut plant_delta: Option<f64> = None;
    let mut fixed = false;
    let mut given = Vec::new();
    let mut rest = Vec::new();
    let mut it = args.iter();
    let number = |a: &str, v: &str| -> Result<f64, String> {
        v.parse().map_err(|_| format!("{a} {v}: not a number"))
    };
    while let Some(a) = it.next() {
        let mut val = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{a} takes a value"))
        };
        match a.as_str() {
            "--inst" | "--tpy" | "--dials" | "--one-sided" | "--reserve" | "--theta"
            | "--plant-delta" => {
                let v = val()?;
                given.push(a.clone());
                given.push(v.clone());
                match a.as_str() {
                    "--inst" => inst = v,
                    "--tpy" => tpy = v.parse().map_err(|_| "--tpy takes a whole number")?,
                    "--dials" => dial_set = Some(v),
                    "--reserve" => {
                        reserve = Some(match v.as_str() {
                            "none" => None,
                            x => Some(number(a, x)?),
                        })
                    }
                    "--theta" => theta = Some(number(a, &v)?),
                    "--plant-delta" => plant_delta = Some(number(a, &v)?),
                    _ => {
                        one_sided = match v.as_str() {
                            "saturate" => OneSided::Saturate,
                            "hold" => OneSided::Hold,
                            x => return Err(format!("--one-sided {x}: saturate or hold")),
                        }
                    }
                }
            }
            "--fixed-plants" => {
                given.push(a.clone());
                fixed = true;
            }
            x if takes_value.contains(&x) => {
                let v = val()?;
                rest.push(a.clone());
                rest.push(v);
            }
            _ => rest.push(a.clone()),
        }
    }
    let mut setup = LoopSetup::registered(&inst, tpy)?;
    if let Some(d) = dial_set {
        setup.dials = loop_dials(&setup.instance, &d, setup.plant_delta)?;
    }
    setup.one_sided = one_sided;
    if let Some(r) = reserve {
        if r.is_some() && setup.instance.is_flow() {
            return Err("the flow control has no maker to hold a reservation".into());
        }
        setup.instance.reserve = r;
    }
    if let Some(t) = theta {
        setup = setup.with_theta(t);
    }
    if let Some(d) = plant_delta {
        setup = setup.with_plant_delta(d)?;
    }
    if fixed {
        setup = setup.with_fixed_plants()?;
    }
    Ok(LoopArgs { given, setup, rest })
}
