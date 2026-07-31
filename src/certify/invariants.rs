//! The two things [kernel.md](../../docs/architecture/kernel.md)'s price
//! formation section says Phase 4 must **check rather than assume**.
//!
//! Both exist because everything in that section is derivation, and because the
//! project has already paid a phase for taking a derivation on trust: the legacy
//! engine posted `min(stock, 1.2 × demand)`, which pinned the imbalance at
//! `−1/6` exactly whenever the cap bound, independent of price, and no
//! certificate noticed for a whole corpus.
//!
//! 1. [`own_state_scan`] — no posted quantity may be computed from another
//!    agent's demand or fill.
//! 2. [`BalanceWatch`] — a settled market must reach `imbalance ≈ 0` rather than
//!    merely a smaller pin.

use std::collections::BTreeMap;

use crate::kernel::AgentArm;
use crate::state::{GameData, SimState};
use crate::types::{
    ids::{GoodId, MarketNodeId},
    order::OrderSide,
};

/// A spread this small over a whole scored window is not a fluctuating market,
/// it is a constant. Absolute rather than relative because imbalance already
/// lives in [−1, 1] and means the same thing at either end of it.
const PIN_SPREAD: f64 = 1e-12;

/// An imbalance this close to zero is balance. Below it a "pin" claim would be
/// about floating-point dust rather than about a market.
const PIN_LEVEL: f64 = 1e-9;

/// Fewer observations than this is not evidence of a constant, just a market
/// that barely traded.
const PIN_MIN_SAMPLES: u64 = 32;

/// How much of a market's history a single unbroken constant must cover before
/// it counts as a pin.
///
/// A *fraction*, not a tick count, so the test means the same thing over an
/// 850-tick corpus window and an 11,550-tick horizon. Phase 3 learned that the
/// hard way: `transient: 150` discards 15% of a 1,000-tick span and 1.3% of an
/// 11,700-tick one, and a certificate passed on the difference.
///
/// Whole-window constancy alone was the first version of this test, and it was
/// too weak to catch the defect the battery exists for. The legacy sell cap
/// pins imbalance at exactly -1/6 *whenever the cap binds* — a long unbroken
/// stretch inside a series that also does other things — so a test asking
/// "was it constant throughout" answered no and passed the run.
const PIN_RUN_FRACTION: f64 = 0.5;

// ── 1. Own-state posting ──────────────────────────────────────────────────────

/// Check that no posted quantity moves when *other agents'* market data does.
///
/// The invariant is about how a quantity was computed, which cannot be read off
/// the quantity — so this establishes it differentially instead. Last tick's
/// aggregated `supply` and `demand` are the only foreign-agent data a decision
/// can reach: everything else an agent reads is its own inventory, its own
/// scale, or a posted price. Perturb those two and re-run the decision phase
/// against an otherwise identical state; a rule that names only its own state
/// must return byte-identical orders.
///
/// Returns one line per violating `(node, good, side)`, empty when clean. The
/// perturbation is deterministic, so this adds nothing to the delta stream and
/// cannot affect the run — it operates on a clone and the clone is discarded.
pub fn own_state_scan(state: &SimState, game_data: &GameData, arm: AgentArm) -> Vec<String> {
    let (_, base) = crate::systems::decisions::run(state, game_data, arm);

    let mut probe = state.clone();
    for node_idx in 0..game_data.num_nodes() {
        let node = MarketNodeId(node_idx as u32);
        for good_idx in 0..game_data.num_goods() {
            let good = GoodId(good_idx as u32);
            // Affine and monotone, so it moves every market including the ones
            // sitting at zero — a rule reading demand cannot be invariant to it,
            // and a rule that does not read it cannot see it at all.
            probe.set_demand(node, good, 2.0 * state.demand(node, good) + 1.0);
            probe.set_supply(node, good, 0.5 * state.supply(node, good) + 1.0);
        }
    }
    let (_, perturbed) = crate::systems::decisions::run(&probe, game_data, arm);

    // Compared in aggregate per (node, good, side) rather than order by order:
    // the invariant is about quantities offered, not about the order in which a
    // loop happened to emit them.
    let fold = |orders: &[crate::types::order::Order]| {
        let mut m: BTreeMap<(u32, u32, bool), f64> = BTreeMap::new();
        for o in orders {
            *m.entry((o.node.0, o.good.0, matches!(o.side, OrderSide::Sell)))
                .or_insert(0.0) += o.qty;
        }
        m
    };
    let (a, b) = (fold(&base), fold(&perturbed));

    let mut out = Vec::new();
    for key in a.keys().chain(b.keys()).copied().collect::<std::collections::BTreeSet<_>>() {
        let (before, after) = (a.get(&key).copied().unwrap_or(0.0), b.get(&key).copied().unwrap_or(0.0));
        if (before - after).abs() > 1e-12 {
            let (node, good, sell) = key;
            out.push(format!(
                "node{node}/{} posted {} moved {before:.6} -> {after:.6} on foreign volumes alone",
                game_data.good(GoodId(good)).name,
                if sell { "supply" } else { "demand" },
            ));
        }
    }
    out
}

// ── 2. Market balance ─────────────────────────────────────────────────────────

/// Running mean and spread of each market's imbalance over the scored window.
///
/// The question is not "is the imbalance small" — a pin can be small — but "is
/// it *moving*". A market whose imbalance never varies is not clearing at some
/// level, it is reporting a number decided by something other than the market,
/// which is exactly what a demand-keyed posting rule produces. Phase 3 measured
/// the legacy engine at standard deviation **exactly zero** across 10,100
/// consecutive ticks with a mean of −1/6.
///
/// Welford, so this is O(1) per market rather than storing a series: at 673
/// regions the series would be the larger artifact.
#[derive(Debug, Clone, Default)]
pub struct BalanceWatch {
    num_goods: usize,
    count: Vec<u64>,
    mean: Vec<f64>,
    m2: Vec<f64>,
    /// Longest unbroken stretch of one value, and the value it held.
    run_len: Vec<u64>,
    run_value: Vec<f64>,
    max_run: Vec<u64>,
    max_run_value: Vec<f64>,
}

impl BalanceWatch {
    pub fn new(game_data: &GameData) -> Self {
        let n = game_data.num_nodes() * game_data.num_goods();
        Self {
            num_goods: game_data.num_goods(),
            count: vec![0; n],
            mean: vec![0.0; n],
            m2: vec![0.0; n],
            run_len: vec![0; n],
            run_value: vec![f64::NAN; n],
            max_run: vec![0; n],
            max_run_value: vec![f64::NAN; n],
        }
    }

    /// Take one tick's reading of every market that actually traded.
    ///
    /// A market with neither supply nor demand reports imbalance 0 by
    /// convention, and counting that convention as an observation would make
    /// every dead market look perfectly pinned at zero.
    pub fn observe(&mut self, state: &SimState, game_data: &GameData) {
        for node_idx in 0..game_data.num_nodes() {
            let node = MarketNodeId(node_idx as u32);
            for good_idx in 0..self.num_goods {
                let good = GoodId(good_idx as u32);
                if state.supply(node, good) <= 0.0 && state.demand(node, good) <= 0.0 {
                    continue;
                }
                let x = state.imbalance(node, good);
                if !x.is_finite() {
                    continue;
                }
                let i = node_idx * self.num_goods + good_idx;
                self.count[i] += 1;
                let delta = x - self.mean[i];
                self.mean[i] += delta / self.count[i] as f64;
                self.m2[i] += delta * (x - self.mean[i]);

                // The unbroken-stretch counter. Exact equality, not a
                // tolerance: a structural pin is arithmetic returning the same
                // number, not a series that happens to be quiet.
                if self.run_len[i] > 0 && self.run_value[i] == x {
                    self.run_len[i] += 1;
                } else {
                    self.run_len[i] = 1;
                    self.run_value[i] = x;
                }
                if self.run_len[i] > self.max_run[i] {
                    self.max_run[i] = self.run_len[i];
                    self.max_run_value[i] = x;
                }
            }
        }
    }

    fn std_dev(&self, i: usize) -> f64 {
        if self.count[i] < 2 {
            return f64::NAN;
        }
        (self.m2[i] / (self.count[i] - 1) as f64).sqrt()
    }

    /// Markets whose imbalance never moved and never reached zero.
    pub fn pinned(&self, game_data: &GameData) -> Vec<String> {
        let mut out = Vec::new();
        for node_idx in 0..game_data.num_nodes() {
            for good_idx in 0..self.num_goods {
                let i = node_idx * self.num_goods + good_idx;
                if self.count[i] < PIN_MIN_SAMPLES {
                    continue;
                }
                let name = &game_data.good(GoodId(good_idx as u32)).name;
                let sd = self.std_dev(i);
                // Two shapes of the same fault. Constant throughout: the market
                // never moved at all. Constant for a long stretch: the market
                // moved, but there is a regime inside it where the number was
                // decided by something other than the market — which is what a
                // demand-keyed posting rule produces, and it is the shape that
                // whole-window constancy misses.
                if sd.is_finite() && sd < PIN_SPREAD && self.mean[i].abs() > PIN_LEVEL {
                    out.push(format!(
                        "node{node_idx}/{name} pinned at {:.6} for all {} ticks (sd {:.1e})",
                        self.mean[i], self.count[i], sd
                    ));
                } else if self.max_run_value[i].abs() > PIN_LEVEL
                    && self.max_run[i] as f64 >= PIN_RUN_FRACTION * self.count[i] as f64
                {
                    out.push(format!(
                        "node{node_idx}/{name} pinned at {:.6} for {} of {} ticks unbroken",
                        self.max_run_value[i], self.max_run[i], self.count[i]
                    ));
                }
            }
        }
        out
    }

    /// The median |mean imbalance| across markets that traded — how close the
    /// typical market got to actually clearing.
    pub fn median_abs_imbalance(&self) -> f64 {
        let mut v: Vec<f64> = (0..self.mean.len())
            .filter(|i| self.count[*i] >= PIN_MIN_SAMPLES)
            .map(|i| self.mean[i].abs())
            .collect();
        if v.is_empty() {
            return f64::NAN;
        }
        v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        v[v.len() / 2]
    }

    pub fn markets_watched(&self) -> usize {
        self.count.iter().filter(|c| **c >= PIN_MIN_SAMPLES).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pin is a constant, and the test that matters is that a *small* constant
    /// still counts as one — the legacy defect's imbalance was −1/6, not −0.99,
    /// and a check keyed on magnitude would have let it through.
    #[test]
    fn a_constant_imbalance_is_a_pin_however_small() {
        for level in [-1.0 / 6.0, -0.001, 1e-6] {
            let mut w = BalanceWatch { num_goods: 1, count: vec![0], mean: vec![0.0], m2: vec![0.0] };
            for _ in 0..100 {
                w.count[0] += 1;
                let d = level - w.mean[0];
                w.mean[0] += d / w.count[0] as f64;
                w.m2[0] += d * (level - w.mean[0]);
            }
            assert!(w.std_dev(0) < PIN_SPREAD, "level {level}: sd {}", w.std_dev(0));
            assert!(w.mean[0].abs() > PIN_LEVEL, "level {level} should read as pinned");
        }
    }

    #[test]
    fn a_market_that_moves_is_not_pinned_and_one_at_zero_is_not_either() {
        // Fluctuating around a level: spread is real, so not a pin.
        let mut w = BalanceWatch { num_goods: 1, count: vec![0], mean: vec![0.0], m2: vec![0.0] };
        for i in 0..100 {
            let x = if i % 2 == 0 { -0.17 } else { -0.16 };
            w.count[0] += 1;
            let d = x - w.mean[0];
            w.mean[0] += d / w.count[0] as f64;
            w.m2[0] += d * (x - w.mean[0]);
        }
        assert!(w.std_dev(0) > PIN_SPREAD, "sd {}", w.std_dev(0));

        // Constant at exactly zero is a market that clears, which is the goal,
        // not a fault.
        let mut z = BalanceWatch { num_goods: 1, count: vec![0], mean: vec![0.0], m2: vec![0.0] };
        z.count[0] = 100;
        assert!(z.std_dev(0) < PIN_SPREAD);
        assert!(z.mean[0].abs() <= PIN_LEVEL, "a cleared market is not a pinned one");
    }
}
