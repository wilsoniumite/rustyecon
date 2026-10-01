//! MARKETS-SPEC §7.8's probes, run on the engine before registration: the one-tick elasticity
//! probe (PROBE-SPEC §4.8, extended to every market), which gives each market's time constant
//! and so the run length L of §6.4's rule, and the open-loop probe, prices frozen.

use super::harness::run;
use super::setup::{genesis, Setup};
use rustyecon_core::num;

/// The one-tick elasticities at the oracle point: for each market's price moved by e^(±h) at
/// genesis, the central difference of ln S and ln D of every market over 2h, after one tick.
#[derive(Debug, Clone, PartialEq)]
pub struct Elasticity {
    /// The markets, in order.
    pub markets: Vec<String>,
    /// ε^S[i][j]: market i's supply against market j's price.
    pub eps_s: Vec<Vec<f64>>,
    /// ε^D[i][j]: market i's demand against market j's price.
    pub eps_d: Vec<Vec<f64>>,
    /// Each market's price step per unit imbalance per tick, k = rate/tpy.
    pub k: Vec<f64>,
    /// Each market's own-price multiplier, 1 + k·(ε^D_ii − ε^S_ii).
    pub multiplier: Vec<f64>,
    /// τ_i = 1/(k·|ε^D_ii − ε^S_ii|), in ticks.
    pub tau: Vec<f64>,
    /// The largest τ.
    pub tau_max: f64,
    /// L by PROBE-SPEC §4.4's rule: 200·τ_max rounded up to a thousand ticks, at least 20,000.
    pub l: u64,
}

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

/// The one-tick elasticity probe at step `h` in log price. A market posting 0 at genesis, a free
/// good (P2.4; the free scan's `elasticity_free.py`), has no log price to move: e^(±h)·0 is 0, so
/// its two runs are the undisplaced one and its own elasticities read 0, and it is left out of
/// τ_max with τ 0, since its price stays 0 while its supply exceeds its demand, whatever the others
/// do.
pub fn elasticity(base: &Setup, h: f64) -> Result<Elasticity, String> {
    let markets = base.instance.markets();
    let n = markets.len();
    let (up, dn) = (num::exp(h), num::exp(-h));
    let at_zero: Vec<bool> = genesis(base)?.prices.iter().map(|&p| p == 0.0).collect();
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
        tau.push(if at_zero[i] {
            0.0
        } else if gap == 0.0 {
            f64::INFINITY
        } else {
            1.0 / (ki * gap.abs())
        });
    }
    let tau_max = tau.iter().fold(0.0, |a: f64, &t| a.max(t));
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

/// PROBE-SPEC §4.4's run length: 200·τ_max rounded up to a thousand ticks, and at least the
/// probe's 20,000.
pub fn run_length(tau_max: f64) -> u64 {
    let thousands = (200.0 * tau_max / 1000.0).ceil();
    let l = if thousands.is_finite() && thousands > 0.0 {
        (thousands as u64) * 1000
    } else {
        u64::MAX
    };
    l.max(crate::protocol::RUN_TICKS)
}

/// The open-loop probe: every price rate 0, so prices stay where genesis put them; for each
/// market's price moved by e^(±h) at genesis, the response d ln(D/S)/d ln p of every market and
/// d ln(coin)/d ln p of every actor at each lag (ticks after genesis).
#[derive(Debug, Clone, PartialEq)]
pub struct OpenLoop {
    /// The markets, in order.
    pub markets: Vec<String>,
    /// The lags.
    pub lags: Vec<u64>,
    /// response[j][lag][i]: market i's d ln(D/S) per unit d ln p_j at the lag.
    pub response: Vec<Vec<Vec<f64>>>,
    /// coin[j][lag][a]: actor a's d ln(coin) per unit d ln p_j at the lag.
    pub coin: Vec<Vec<Vec<f64>>>,
    /// The frozen, undisplaced run's ln(D/S) per market at each lag: the quantity loop's own
    /// motion from rounding at the point.
    pub hold: Vec<Vec<f64>>,
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

/// The open-loop probe at step `h`, over `lags`.
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
