//! "Why is this price 12.3?" (docs/GUI.md §4, G1). Under `Imbalance` a price has memory,
//! p_t = p_0·exp(Σ k·x), so there are two answers, and this module gives both:
//!
//! - **The explainer:** the tick's step recomputed by markets' own `imbalance` and
//!   `next_price` (P0.4's public functions, which the engine's price update calls) from the
//!   tick's recorded inputs: the posted price p, S and D, and k, the price rate's per-tick log
//!   step, read at that tick through the good's own site (`Site::convert`, the conversion the
//!   run applies). Where the run records the next price, the explainer says whether its result
//!   equals it bit for bit.
//! - **The log waterfall:** ln(p_t/p_0) as Σ k·x over the ticks before t, a year at a time,
//!   with the residual, what the rule's steps do not explain: a one-sided market held, a price
//!   moved by an event, and rounding. The events fired in the span are marked, and those that
//!   act on this market's price or its rate are flagged. Under `Ratio`, which ignores k and
//!   holds a one-sided market by the rule itself, the steps are its own, ln(D/S) where both
//!   sides posted, the residual is events and rounding, and a rate set moves nothing (G1's
//!   verification).
//!
//! Every number is recorded, the result of markets' own function, or a display transform of
//! those: a product, a sum, a difference, a log through the engine's `num` (U6).

use super::{date, describe, report_tick};
use crate::run::{At, Measure, SeriesKey, Store};
use rustyecon_engine::num;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_markets::{imbalance, next_price};
use serde::Serialize;

/// Whether the world's price rule is `Ratio`, read by the name the tape gives it, as the
/// explainer shows it: the engine's prelude does not name the rule's type, and the GUI has no
/// edge to core (G1.1).
fn is_ratio(w: &World) -> bool {
    format!("{:?}", w.market.rule) == "Ratio"
}

/// The explainer at one tick.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExplainerVm {
    /// The report tick.
    pub tick: u64,
    /// The market's price rule, from the tape.
    pub rule: String,
    /// What a one-sided market does under it.
    pub one_sided: String,
    /// p: the posted price the tick's settlement used.
    pub price: f64,
    /// S, the sells.
    pub supply: f64,
    /// D, the feasible buys.
    pub demand: f64,
    /// x = imbalance(S, D) = (D − S)/max(S, D), markets' own.
    pub imbalance: f64,
    /// Whether exactly one side posted.
    pub one_side: bool,
    /// The good's price rate: its param.
    pub rate: Key,
    /// Its value at the tick, in its unit.
    pub rate_value: f64,
    /// Its unit.
    pub rate_unit: String,
    /// The conversion its site takes.
    pub method: String,
    /// k, the rate per tick by that conversion: `clock.log_step` of the rate.
    pub k: f64,
    /// k·x.
    pub kx: f64,
    /// next_price(rule, one_sided, p, k, S, D), markets' own.
    pub next: f64,
    /// The next price the run recorded, where its catalogue records it.
    pub recorded: Option<f64>,
    /// Whether `next` equals it bit for bit.
    pub equal: Option<bool>,
    /// ln(next/p), the step in the log price.
    pub log_step: Option<f64>,
}

fn series(store: &Store, measure: Measure, at: &At, tick: u64) -> Option<f64> {
    store
        .series(&SeriesKey {
            measure,
            at: at.clone(),
        })?
        .at(tick)
}

/// The price rate's site of a good, and its param's key and unit.
fn rate_of(w: &World, good: &Key) -> Result<(Site, Key, String), String> {
    let g = w
        .id_of::<GoodId>(good.as_str())
        .ok_or_else(|| format!("no good {good} in this run's world"))?;
    let site = w
        .good(g)
        .and_then(|d| d.price_rate)
        .ok_or_else(|| format!("{good} has no price rate"))?;
    let key = w
        .key_of(site.param)
        .cloned()
        .ok_or_else(|| format!("the price rate of {good} has no key"))?;
    let unit = w
        .registry
        .get(site.param)
        .map_or_else(String::new, |d| d.unit.to_string());
    Ok((site, key, unit))
}

/// The step of report tick `tick` at (node, good), recomputed from what the run recorded; or
/// why it cannot be.
pub fn explain(store: &Store, node: &Key, good: &Key, tick: u64) -> Result<ExplainerVm, String> {
    let w = store.world().ok_or("the tape is loading")?;
    let (site, rate, rate_unit) = rate_of(w, good)?;
    let at = At::Market {
        node: node.clone(),
        good: good.clone(),
    };
    let read = |m: Measure, what: &str| {
        series(store, m, &at, tick).ok_or_else(|| format!("{what} is not recorded at tick {tick}"))
    };
    let price = read(Measure::Price, "p")?;
    let supply = read(Measure::Supply, "S")?;
    let demand = read(Measure::Demand, "D")?;
    let rate_value = series(store, Measure::Param, &At::Param(rate.clone()), tick)
        .ok_or_else(|| format!("the rate {rate} is not recorded at tick {tick}"))?;
    let k = site
        .convert(&w.clock, rate_value)
        .map_err(|e| format!("the rate {rate} per tick: {e}"))?;
    let x = imbalance(supply, demand);
    let next = next_price(w.market.rule, w.market.one_sided, price, k, supply, demand);
    let recorded = series(store, Measure::NextPrice, &at, tick);
    Ok(ExplainerVm {
        tick,
        rule: format!("{:?}", w.market.rule),
        one_sided: format!("{:?}", w.market.one_sided),
        price,
        supply,
        demand,
        imbalance: x,
        one_side: (supply > 0.0) != (demand > 0.0),
        rate,
        rate_value,
        rate_unit,
        method: site.method.name().to_string(),
        k,
        kx: k * x,
        next,
        recorded,
        equal: recorded.map(|r| r.to_bits() == next.to_bits()),
        log_step: (next > 0.0 && price > 0.0).then(|| num::ln(next / price)),
    })
}

/// One bin of the waterfall: ticks `[from, to)`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WaterBinVm {
    /// Its first tick.
    pub from: u64,
    /// The tick after its last.
    pub to: u64,
    /// Its first tick's date, or its year.
    pub label: String,
    /// Σ k·x over its ticks.
    pub kx: f64,
    /// ln(p/p_0) at its end, tick `to`.
    pub level: f64,
    /// What its steps do not explain: the change of ln(p/p_0) over it, less Σ k·x.
    pub residual: f64,
}

/// An event fired in the waterfall's span.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WaterEventVm {
    /// The tick it fired in.
    pub tick: u64,
    /// Its key.
    pub key: Key,
    /// What it did.
    pub what: String,
    /// Whether it acts on this market's price or sets its rate.
    pub moves: bool,
}

/// ln(p_t/p_0) as Σ k·x at (node, good), from the record's first tick to the cursor's; under
/// `Ratio`, which ignores k, as Σ ln(D/S), the rule's own steps.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WaterfallVm {
    /// What each tick's step is: `k·x` under `Imbalance`, `ln(D/S)` under `Ratio` (0 where a
    /// side is empty, which the rule holds).
    pub term: String,
    /// The first tick, p_0's.
    pub start: u64,
    /// The cursor's report tick, p_t's.
    pub tick: u64,
    /// p_0.
    pub p0: f64,
    /// p_t.
    pub p: f64,
    /// ln(p_t/p_0).
    pub level: f64,
    /// Σ of the term over the ticks before t.
    pub explained: f64,
    /// `level` less `explained`.
    pub residual: f64,
    /// The ticks at which one side alone posted.
    pub one_sided: usize,
    /// The ticks without every input recorded, left out of Σ k·x.
    pub missing: usize,
    /// The bins, a year each when the span holds two years or more, else a tick each.
    pub bins: Vec<WaterBinVm>,
    /// The events fired in the span.
    pub events: Vec<WaterEventVm>,
}

/// The waterfall of (node, good) up to the cursor's report tick (`None`: live); or why there
/// is none.
pub fn waterfall(
    store: &Store,
    node: &Key,
    good: &Key,
    cursor: Option<u64>,
) -> Result<WaterfallVm, String> {
    let w = store.world().ok_or("the tape is loading")?;
    let tick = report_tick(store, cursor).ok_or("no tick has run")?;
    let (site, rate, _) = rate_of(w, good)?;
    let at = At::Market {
        node: node.clone(),
        good: good.clone(),
    };
    let key = |m: Measure| SeriesKey {
        measure: m,
        at: at.clone(),
    };
    let price = store
        .series(&key(Measure::Price))
        .ok_or("p is not recorded")?;
    let supply = store.series(&key(Measure::Supply));
    let demand = store.series(&key(Measure::Demand));
    let rates = store.series(&SeriesKey {
        measure: Measure::Param,
        at: At::Param(rate.clone()),
    });
    let start = store.start();
    let p0 = price
        .at(start)
        .ok_or("p is not recorded at the first tick")?;
    let p = price
        .at(tick)
        .ok_or_else(|| format!("p is not recorded at tick {tick}"))?;
    if !(p0 > 0.0 && p > 0.0) {
        return Err("a price is not positive".to_string());
    }
    let tpy = u64::from(w.clock.ticks_per_year.max(1));
    let yearly = tick - start >= 2 * tpy;
    let mut bins: Vec<WaterBinVm> = Vec::new();
    let (mut explained, mut one_sided, mut missing) = (0.0, 0, 0);
    let mut bin_kx = 0.0;
    let mut bin_from = start;
    let mut level_at_from = 0.0;
    let ratio = is_ratio(w);
    for t in start..tick {
        let volumes = supply
            .and_then(|s| s.at(t))
            .zip(demand.and_then(|s| s.at(t)));
        // The tick's step in the log price by the rule: k·x under `Imbalance`, from the rate at
        // the tick; ln(D/S) under `Ratio`, which ignores k and holds a one-sided market.
        let step = volumes.and_then(|(s, d)| {
            if ratio {
                Some(if s > 0.0 && d > 0.0 {
                    num::ln(d / s)
                } else {
                    0.0
                })
            } else {
                let v = rates.and_then(|r| r.at(t))?;
                let k = site.convert(&w.clock, v).ok()?;
                Some(k * imbalance(s, d))
            }
        });
        match (step, volumes) {
            (Some(x), Some((s, d))) => {
                explained += x;
                bin_kx += x;
                if (s > 0.0) != (d > 0.0) {
                    one_sided += 1;
                }
            }
            _ => missing += 1,
        }
        let end = t + 1;
        let close = if yearly {
            (end - start).is_multiple_of(tpy) || end == tick
        } else {
            true
        };
        if close {
            let level = price
                .at(end)
                .filter(|q| *q > 0.0)
                .map_or(level_at_from, |q| num::ln(q / p0));
            let label = if yearly {
                w.clock
                    .date_of(bin_from)
                    .map_or_else(|| bin_from.to_string(), |d| d.y.to_string())
            } else {
                date(w, bin_from)
            };
            bins.push(WaterBinVm {
                from: bin_from,
                to: end,
                label,
                kx: bin_kx,
                level,
                residual: (level - level_at_from) - bin_kx,
            });
            level_at_from = level;
            bin_kx = 0.0;
            bin_from = end;
        }
    }
    let rate_id = site.param;
    let (n, g) = (
        w.id_of::<NodeId>(node.as_str()),
        w.id_of::<GoodId>(good.as_str()),
    );
    let events = store
        .events()
        .iter()
        .filter(|(t, _)| *t >= start && *t <= tick)
        .map(|(t, e)| {
            let moves = match &e.action {
                StateDelta::ScalePrice {
                    node: en, good: eg, ..
                } => Some(*en) == n && Some(*eg) == g,
                // Under `Ratio` the rate moves nothing.
                StateDelta::SetParam { param, .. } => !ratio && *param == rate_id,
                _ => false,
            };
            WaterEventVm {
                tick: *t,
                key: e.key.clone(),
                what: describe(w, &e.action),
                moves,
            }
        })
        .collect();
    let level = num::ln(p / p0);
    Ok(WaterfallVm {
        term: if ratio { "ln(D/S)" } else { "k·x" }.to_string(),
        start,
        tick,
        p0,
        p,
        level,
        explained,
        residual: level - explained,
        one_sided,
        missing,
        bins,
        events,
    })
}
