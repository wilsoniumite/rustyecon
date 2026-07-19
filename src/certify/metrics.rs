//! Per-region metric series, computed in-engine (engine.md; METHODOLOGY R5).
//!
//! Ported from `notebooks/07_stability_suite.py`, where these were reconstructed
//! from per-tick RON checkpoints by `tools/readers.py`. Every input already lives
//! in `SimState` each tick — the notebook only rebuilt them because the engine
//! kept one tick in memory and emitted no metric telemetry. Collecting here means
//! a run scores itself, rather than a notebook scoring it afterwards.
//!
//! The series are the raw per-tick values for the whole run; the analysis window
//! is applied at scoring time, so a criteria file can widen or narrow the window
//! without re-running.

use crate::certify::criteria::{Criteria, Metric};
use crate::state::{GameData, SimState};
use crate::types::ids::{GoodId, RegionId};

/// One region's per-tick series. Index is tick order, starting at the first
/// recorded tick — [`MetricsCollector::record`] is called once per tick.
#[derive(Debug, Clone)]
pub struct RegionSeries {
    pub region: RegionId,
    pub name: String,
    pub velocity: Vec<f64>,
    pub employment: Vec<f64>,
    pub real_wage: Vec<f64>,
    pub concentration: Vec<f64>,
    pub bld_util: Vec<f64>,
    pub real_income: Vec<f64>,
    /// Per-building utilisation, for the DEAD_BUILDING warning.
    /// `building_util[i]` is the series for `building_ids[i]`.
    pub building_ids: Vec<u32>,
    pub building_util: Vec<Vec<f64>>,
}

impl RegionSeries {
    pub fn get(&self, metric: Metric) -> &[f64] {
        match metric {
            Metric::Velocity => &self.velocity,
            Metric::Employment => &self.employment,
            Metric::RealWage => &self.real_wage,
            Metric::Concentration => &self.concentration,
            Metric::BldUtil => &self.bld_util,
            Metric::RealIncome => &self.real_income,
        }
    }
}

/// Accumulates [`RegionSeries`] as a run proceeds.
pub struct MetricsCollector {
    regions: Vec<RegionSeries>,
    /// Deflator for RealWage / RealIncome, resolved from the criteria by name.
    staple: Option<GoodId>,
    /// RealWage numerator, resolved from the criteria by name.
    labour: Option<GoodId>,
}

/// Divide, yielding NaN when the denominator is not positive.
///
/// Mirrors the notebook's `.replace(0, np.nan)`: a ratio with no base is not
/// zero, it is unknown, and under the fail-closed NaN policy that is a failure
/// rather than a quiet pass.
fn ratio(num: f64, den: f64) -> f64 {
    if den > 0.0 {
        num / den
    } else {
        f64::NAN
    }
}

impl MetricsCollector {
    pub fn new(game_data: &GameData, criteria: &Criteria) -> Self {
        let by_name = |name: &str| {
            game_data
                .goods
                .iter()
                .find(|g| g.name == name)
                .map(|g| g.id)
        };
        let regions = game_data
            .regions
            .iter()
            .map(|r| RegionSeries {
                region: r.id,
                name: r.name.clone(),
                velocity: Vec::new(),
                employment: Vec::new(),
                real_wage: Vec::new(),
                concentration: Vec::new(),
                bld_util: Vec::new(),
                real_income: Vec::new(),
                building_ids: Vec::new(),
                building_util: Vec::new(),
            })
            .collect();
        Self {
            regions,
            staple: by_name(&criteria.staple_good),
            labour: by_name(&criteria.labour_good),
        }
    }

    pub fn series(&self) -> &[RegionSeries] {
        &self.regions
    }

    /// Sample every region's metrics for the tick just completed.
    pub fn record(&mut self, state: &SimState, game_data: &GameData) {
        for series in &mut self.regions {
            let region = series.region;
            let node = game_data.region(region).market_node;
            let currency = game_data.market_node(node).currency_good;

            // Transaction value and currency stocks.
            let mut txn_value = 0.0;
            for good_idx in 0..game_data.num_goods() {
                let good = GoodId(good_idx as u32);
                if currency == Some(good) {
                    continue;
                }
                let cleared = state.supply(node, good).min(state.demand(node, good));
                if cleared > 0.0 {
                    txn_value += cleared * state.price(node, good);
                }
            }

            let (mut bld_gbp, mut pop_gbp, mut emp_gbp) = (0.0, 0.0, 0.0);
            if let Some(cid) = currency {
                for b in &state.buildings {
                    if b.region == region {
                        bld_gbp += state.inventory(b.inventory).get(cid);
                    }
                }
                for p in &state.pop_groups {
                    if p.region == region {
                        let held = state.inventory(p.inventory).get(cid);
                        pop_gbp += held;
                        if p.is_employed {
                            emp_gbp += held;
                        }
                    }
                }
            }
            let total_gbp = bld_gbp + pop_gbp;

            // Employment.
            let mut emp_size = 0.0;
            let mut total_size = 0.0;
            for p in &state.pop_groups {
                if p.region == region {
                    total_size += p.size;
                    if p.is_employed {
                        emp_size += p.size;
                    }
                }
            }

            // Real wage: labour price deflated by the staple.
            let real_wage = match (self.labour, self.staple) {
                (Some(l), Some(s)) => ratio(state.price(node, l), state.price(node, s)),
                _ => f64::NAN,
            };

            // Utilisation over this region's non-channel production instances.
            let mut util_sum = 0.0;
            let mut util_n = 0usize;
            for ri in &state.recipe_instances {
                if ri.region != region
                    || ri.channel.is_some()
                    || ri.strategy_state.capacity_control().is_none()
                {
                    continue;
                }
                let u = ratio(ri.chosen_size, ri.recipe_size);
                let u = if u.is_finite() { u.clamp(0.0, 1.0) } else { f64::NAN };
                if u.is_finite() {
                    util_sum += u;
                    util_n += 1;
                }
                // Per-building series for the DEAD_BUILDING warning.
                let id = ri.id.0;
                let slot = match series.building_ids.iter().position(|&b| b == id) {
                    Some(i) => i,
                    None => {
                        series.building_ids.push(id);
                        series.building_util.push(Vec::new());
                        series.building_util.len() - 1
                    }
                };
                series.building_util[slot].push(u);
            }

            let real_income = match self.staple {
                Some(s) => ratio(ratio(emp_gbp, emp_size), state.price(node, s)),
                None => f64::NAN,
            };

            series.velocity.push(ratio(txn_value, total_gbp));
            series.concentration.push(ratio(bld_gbp, total_gbp));
            series.employment.push(ratio(emp_size, total_size));
            series.real_wage.push(real_wage);
            series.bld_util.push(if util_n > 0 {
                util_sum / util_n as f64
            } else {
                f64::NAN
            });
            series.real_income.push(real_income);
        }
    }
}
