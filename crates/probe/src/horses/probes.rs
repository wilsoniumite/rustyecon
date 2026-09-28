//! HORSES-SPEC §7.8's probes, run on the engine before registration: the one-tick elasticity
//! probe (PROBE-SPEC §4.8, every market), which gives each market's time constant and so the
//! run length's second term, and the open-loop probe, prices frozen, which should show no
//! quantity loop (there is none in v2a.1). Each is the markets probe's
//! ([`crate::markets::probes`]) on the stocks probe's harness.

use super::harness::run;
use super::setup::Setup;
use crate::markets::probes::{run_length, Elasticity, OpenLoop};
use rustyecon_core::num;

fn term(market: &str, f: f64) -> String {
    format!("p[{market}]*{f:?}")
}

/// One tick of `name` from `base`: every market's supply and demand.
fn one_tick(base: &Setup, name: &str) -> Result<(Vec<f64>, Vec<f64>), String> {
    let mut out = None;
    run(base, name, 1, &mut |r| {
        if out.is_none() {
            out = Some((r.supply.clone(), r.demand.clone()));
        }
    })?;
    out.ok_or_else(|| format!("{name}: no tick ran"))
}

/// The one-tick elasticity probe at step `h` in log price. A market with no supply or demand at
/// a displaced tick (the horse market's supply is a stock) reads as it reads.
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
    let l = run_length(tau_max);
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

type Series = Vec<Vec<f64>>;

/// Each market's ln(D/S) and each actor's ln coin at each lag of a run.
fn imbalance(base: &Setup, name: &str, lags: &[u64]) -> Result<(Series, Series), String> {
    let last = lags.iter().copied().max().unwrap_or(0);
    let mut out = vec![Vec::new(); lags.len()];
    let mut coin = vec![Vec::new(); lags.len()];
    run(base, name, last + 1, &mut |r| {
        if let Some(i) = lags.iter().position(|&l| l == r.tick) {
            out[i] = r
                .supply
                .iter()
                .zip(&r.demand)
                .map(|(s, d)| num::ln(d / s))
                .collect();
            coin[i] = r.coin.iter().map(|&c| num::ln(c)).collect();
        }
    })?;
    Ok((out, coin))
}

fn per_unit(a: &Series, b: &Series, h: f64) -> Series {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.iter().zip(y).map(|(p, q)| (p - q) / (2.0 * h)).collect())
        .collect()
}

/// The open-loop probe at step `h`, over `lags`: every price rate 0, so prices stay where
/// genesis put them.
pub fn open_loop(base: &Setup, h: f64, lags: &[u64]) -> Result<OpenLoop, String> {
    let mut frozen = base.clone();
    frozen.dials.set("rate.*", 0.0)?;
    let markets = base.instance.markets();
    let (up, dn) = (num::exp(h), num::exp(-h));
    let mut response = Vec::with_capacity(markets.len());
    let mut coin = Vec::with_capacity(markets.len());
    for m in &markets {
        let (a, ac) = imbalance(&frozen, &term(m, up), lags)?;
        let (b, bc) = imbalance(&frozen, &term(m, dn), lags)?;
        response.push(per_unit(&a, &b, h));
        coin.push(per_unit(&ac, &bc, h));
    }
    let (hold, _) = imbalance(&frozen, "hold", lags)?;
    Ok(OpenLoop {
        markets,
        lags: lags.to_vec(),
        response,
        coin,
        hold,
    })
}
