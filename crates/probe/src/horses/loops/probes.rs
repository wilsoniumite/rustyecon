//! The loop step's probes (LOOPS-RULES §8.6; HORSES-RULES §6.6, §6.7): the one-tick elasticity
//! probe on every market, which gives each market's time constant and so the run length's second
//! term; the kick set at a run's end, certify's (P2.2a's, [`crate::horses::kick`]); and the run
//! length's rule, L = max(20,000·tpy/52, 200·τ_max, 3·T6), each rounded up to a thousand ticks,
//! with T6 the mirror's (`lm_lin.json`).

use super::harness::run;
use super::perturb::Perturbation;
use super::{tape_ron, Setup};
use crate::horses::kick::kick_set_of;
use crate::markets::kick::KickSet;
use crate::markets::probes::Elasticity;
use rustyecon_core::num;

fn term(market: &str, f: f64) -> String {
    format!("p[{market}]*{f:?}")
}

/// One tick of `name` from `base`: every market's supply and demand.
fn one_tick(base: &Setup, name: &str) -> Result<(Vec<f64>, Vec<f64>), String> {
    let mut out = None;
    run(base, name, 1, &mut |r| {
        if out.is_none() {
            out = Some((r.row.supply.clone(), r.row.demand.clone()));
        }
    })?;
    out.ok_or_else(|| format!("{name}: no tick ran"))
}

/// A number of ticks rounded up to a thousand.
pub fn thousands(ticks: f64) -> u64 {
    let k = (ticks / 1000.0).ceil();
    if k.is_finite() && k > 0.0 {
        (k as u64) * 1000
    } else {
        u64::MAX
    }
}

/// The one-tick elasticity probe at step `h` in log price (PROBE-SPEC §4.8), every market: each
/// market's own-price elasticities of supply and demand from a central difference over one tick
/// from genesis, its price rate k a tick, and τ = 1/(k·|ε_d − ε_s|). `l` is 200·τ_max rounded
/// up to a thousand, at least the floor 20,000·tpy/52.
pub fn elasticity(base: &Setup, h: f64) -> Result<Elasticity, String> {
    let markets = base.instance.markets();
    let n = markets.len();
    let (up, dn) = (num::exp(h), num::exp(-h));
    let mut eps_s = vec![vec![0.0; n]; n];
    let mut eps_d = vec![vec![0.0; n]; n];
    for (j, m) in markets.iter().enumerate() {
        let (su, du) = one_tick(base, &term(m, up))?;
        let (sd, dd) = one_tick(base, &term(m, dn))?;
        for i in 0..n {
            eps_s[i][j] = (num::ln(su[i]) - num::ln(sd[i])) / (2.0 * h);
            eps_d[i][j] = (num::ln(du[i]) - num::ln(dd[i])) / (2.0 * h);
        }
    }
    let tpy = f64::from(base.tpy);
    let mut k = Vec::with_capacity(n);
    let mut multiplier = Vec::with_capacity(n);
    let mut tau = Vec::with_capacity(n);
    for (i, m) in markets.iter().enumerate() {
        let ki = base.dials.get(&format!("rate.{m}"))? / tpy;
        let gap = eps_d[i][i] - eps_s[i][i];
        k.push(ki);
        multiplier.push(1.0 + ki * gap);
        tau.push(if gap == 0.0 {
            f64::INFINITY
        } else {
            1.0 / (ki * gap.abs())
        });
    }
    let tau_max = tau
        .iter()
        .filter(|t| t.is_finite())
        .fold(0.0, |a: f64, &t| a.max(t));
    let l = thousands(200.0 * tau_max).max(floor(base.tpy));
    Ok(Elasticity {
        markets,
        eps_s,
        eps_d,
        k,
        multiplier,
        tau,
        tau_max,
        l,
    })
}

/// The run length's first term: 20,000·tpy/52 rounded up to a thousand.
pub fn floor(tpy: u32) -> u64 {
    thousands(20_000.0 * f64::from(tpy) / 52.0)
}

/// The mirror's 3·T6 at 52 ticks a year, `lm_lin.json`'s third term per instance (the registered
/// loop mirror, `D:/rustyecon-p2l/fix-report/loop-carry/model/lm_lin.json`; LOOP-SPEC §6): `None`
/// where the mirror's slowest root is not below 1 (LN1, LN3, LN6, LN7), which then run at LB1's
/// length (`lm_battery:70–74`).
pub fn mirror_three_t6(id: &str) -> Result<Option<u64>, String> {
    Ok(match id {
        "lb1" | "lf3" => Some(21_000),
        "lb2" => Some(17_000),
        "lb3" => Some(45_000),
        "lf1" => Some(29_000),
        "lf2" => Some(28_000),
        "lf5" => Some(16_000),
        "lf6" => Some(103_000),
        "lf7" => Some(23_000),
        "lf8" => Some(60_000),
        "lc1" => Some(19_000),
        "lw0" | "lw1" | "lw2" | "lw3" => Some(14_000),
        "ln2" => Some(16_000),
        "ln4" | "ln8" => Some(20_000),
        "ln9" => Some(19_000),
        "ln1" | "ln3" | "ln6" | "ln7" => None,
        _ => return Err(format!("no mirror T6 for {id}")),
    })
}

/// The run length's rule (LOOPS-RULES §8.6; decision 267): at `tpy` ticks a year, the largest of
/// the floor, the engine's 200·τ_max (`elastic`, already rounded) and the mirror's 3·T6 scaled by
/// tpy/52 and rounded up to a thousand. Where the mirror's T6 is infinite the instance runs at
/// `fallback`, LB1's L at the same tick length.
pub fn run_length(id: &str, tpy: u32, elastic: u64, fallback: Option<u64>) -> Result<u64, String> {
    match mirror_three_t6(id)? {
        Some(t6) => {
            let t6 = thousands(t6 as f64 * f64::from(tpy) / 52.0);
            Ok(floor(tpy).max(elastic).max(t6))
        }
        None => fallback.ok_or_else(|| format!("{id}: the mirror's T6 is infinite; give LB1's L")),
    }
}

/// The kick set at the end of the run `name` of `base`, scored for `ticks` ticks (and the ticks
/// before a dated shock), with horizon `horizon`, and its envelope (HORSES-RULES §6.6). Certify's
/// kick set walks `World::markets()`, so no plant is kicked.
pub fn kick_set(
    base: &Setup,
    name: &str,
    ticks: u64,
    horizon: u64,
) -> Result<(KickSet, Vec<f64>), String> {
    let pert = Perturbation::parse(name)?;
    let mut setup = base.clone();
    pert.apply(&mut setup, ticks)?;
    let text = tape_ron(&setup)?;
    kick_set_of(&text, pert.clock_start(ticks) + ticks, horizon, name)
}
