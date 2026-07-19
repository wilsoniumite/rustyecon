use crate::types::ids::GoodId;
use crate::types::provenance::Provenance;

/// A removal that could not take the full requested amount.
///
/// engine.md: shortfalls become ledger lines, never silent clamps. A shortfall
/// on a `Transfer` is also a conservation break (units were shipped that did
/// not exist), so the drift assert catches it independently.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shortfall {
    pub good: GoodId,
    pub requested: f64,
    pub removed: f64,
}

impl Shortfall {
    pub fn missing(&self) -> f64 {
        self.requested - self.removed
    }
}

/// Per-tick conservation accounting (engine.md, "The conservation ledger").
///
/// Two independent tallies per good:
///
/// - `observed` — the true change in Σ inventory, accumulated from every add
///   (`+qty`) and every remove (`−actually removed`). Because removals post the
///   quantity that was really taken, a clamp cannot hide here.
/// - `declared` — the sum of provenance-tagged mint/burn lines. [`Provenance::Transfer`]
///   contributes nothing, since moving units between inventories does not change
///   the total in existence.
///
/// At tick end the two must agree. They diverge exactly when a transfer fails to
/// balance (goods shipped but never received, money credited but never paid) or
/// when something is created or destroyed with no provenance line — which is the
/// definition of a conservation bug, and panics the run (METHODOLOGY R3).
///
/// Equality is up to a tolerance scaled by gross flow: clearing splits a market
/// with `buyer_fill = min(1, s/d)` and `seller_fill = min(1, d/s)`, so the two
/// sides of a trade agree only to within a few ULP of f64.
#[derive(Debug, Clone)]
pub struct ConservationLedger {
    observed: Vec<f64>,
    declared: Vec<f64>,
    minted: Vec<f64>,
    burned: Vec<f64>,
    /// Σ|qty| moved per good; sets the scale for the drift tolerance.
    gross: Vec<f64>,
    shortfalls: Vec<Shortfall>,
}

/// Relative slack allowed per unit of gross flow, plus an absolute floor.
const REL_TOLERANCE: f64 = 1e-9;
const ABS_TOLERANCE: f64 = 1e-9;

impl ConservationLedger {
    pub fn new(num_goods: usize) -> Self {
        Self {
            observed: vec![0.0; num_goods],
            declared: vec![0.0; num_goods],
            minted: vec![0.0; num_goods],
            burned: vec![0.0; num_goods],
            gross: vec![0.0; num_goods],
            shortfalls: Vec::new(),
        }
    }

    /// Clear all tallies, keeping the allocation. Called at the top of each tick.
    pub fn reset(&mut self) {
        self.observed.iter_mut().for_each(|v| *v = 0.0);
        self.declared.iter_mut().for_each(|v| *v = 0.0);
        self.minted.iter_mut().for_each(|v| *v = 0.0);
        self.burned.iter_mut().for_each(|v| *v = 0.0);
        self.gross.iter_mut().for_each(|v| *v = 0.0);
        self.shortfalls.clear();
    }

    /// Post units entering an inventory. `qty` always lands in full — `add` never clamps.
    pub fn record_add(&mut self, good: GoodId, qty: f64, prov: Provenance) {
        let i = good.idx();
        if i >= self.observed.len() {
            return;
        }
        self.observed[i] += qty;
        self.gross[i] += qty.abs();
        if prov.is_mint_or_burn() {
            self.declared[i] += qty;
            self.minted[i] += qty;
        }
    }

    /// Post units leaving an inventory. `removed` is what was *actually* taken;
    /// any gap against `requested` is recorded as a shortfall ledger line.
    pub fn record_remove(&mut self, good: GoodId, requested: f64, removed: f64, prov: Provenance) {
        let i = good.idx();
        if i >= self.observed.len() {
            return;
        }
        self.observed[i] -= removed;
        self.gross[i] += removed.abs();
        if prov.is_mint_or_burn() {
            self.declared[i] -= removed;
            self.burned[i] += removed;
        }
        if requested - removed > ABS_TOLERANCE {
            self.shortfalls.push(Shortfall { good, requested, removed });
        }
    }

    /// Drift tolerance for one good, scaled by the gross flow through it.
    fn tolerance(&self, i: usize) -> f64 {
        ABS_TOLERANCE + REL_TOLERANCE * self.gross[i]
    }

    /// Per-good conservation error: observed change minus declared mint/burn.
    pub fn drift(&self, good: GoodId) -> f64 {
        let i = good.idx();
        self.observed[i] - self.declared[i]
    }

    /// The largest tolerance-normalised breach, if any good is out of balance.
    /// Returns `(good, absolute drift)`.
    pub fn worst_breach(&self) -> Option<(GoodId, f64)> {
        let mut worst: Option<(GoodId, f64, f64)> = None; // (good, |drift|, excess over tol)
        for i in 0..self.observed.len() {
            let d = (self.observed[i] - self.declared[i]).abs();
            let excess = d - self.tolerance(i);
            if excess > 0.0 && worst.map_or(true, |(_, _, w)| excess > w) {
                worst = Some((GoodId(i as u32), d, excess));
            }
        }
        worst.map(|(g, d, _)| (g, d))
    }

    /// Largest absolute drift across all goods, breaching or not. Reported by the
    /// certificate's conservation battery.
    pub fn max_abs_drift(&self) -> f64 {
        (0..self.observed.len())
            .map(|i| (self.observed[i] - self.declared[i]).abs())
            .fold(0.0, f64::max)
    }

    pub fn shortfalls(&self) -> &[Shortfall] {
        &self.shortfalls
    }

    pub fn minted(&self, good: GoodId) -> f64 {
        self.minted[good.idx()]
    }

    pub fn burned(&self, good: GoodId) -> f64 {
        self.burned[good.idx()]
    }

    /// Panic if any good is out of balance beyond tolerance (METHODOLOGY R3).
    ///
    /// A conservation failure is a bug by definition — money leaks, goods
    /// duplication, and phantom consumption must be impossible to miss — so this
    /// aborts the run rather than letting a corrupt trajectory reach a notebook.
    pub fn assert_conserved(&self, tick: u64) {
        if let Some((good, drift)) = self.worst_breach() {
            let i = good.idx();
            panic!(
                "conservation failure at tick {tick}: good {} drifted {drift:+.6e}\n  \
                 observed Σ change {:+.6e} != declared mint/burn {:+.6e}\n  \
                 minted {:.6e}, burned {:.6e}, gross flow {:.6e}, tolerance {:.3e}\n  \
                 (an unbalanced transfer, or a create/destroy with no provenance line)",
                good.0,
                self.observed[i],
                self.declared[i],
                self.minted[i],
                self.burned[i],
                self.gross[i],
                self.tolerance(i),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(n: u32) -> GoodId {
        GoodId(n)
    }

    #[test]
    fn balanced_transfer_conserves() {
        let mut l = ConservationLedger::new(2);
        // Seller ships 5, buyer receives 5.
        l.record_remove(g(0), 5.0, 5.0, Provenance::Transfer);
        l.record_add(g(0), 5.0, Provenance::Transfer);
        assert!(l.worst_breach().is_none());
        l.assert_conserved(1);
    }

    #[test]
    fn unbalanced_transfer_is_a_breach() {
        let mut l = ConservationLedger::new(2);
        // Seller ships 5 but buyer only receives 3 — two units vanish.
        l.record_remove(g(0), 5.0, 5.0, Provenance::Transfer);
        l.record_add(g(0), 3.0, Provenance::Transfer);
        let (good, drift) = l.worst_breach().expect("must detect the leak");
        assert_eq!(good, g(0));
        assert!((drift - 2.0).abs() < 1e-12, "drift was {drift}");
    }

    #[test]
    fn declared_mint_and_burn_conserve() {
        let mut l = ConservationLedger::new(2);
        l.record_add(g(1), 7.0, Provenance::Production); // minted
        l.record_remove(g(1), 4.0, 4.0, Provenance::Consumption); // burned
        assert!(l.worst_breach().is_none());
        assert_eq!(l.minted(g(1)), 7.0);
        assert_eq!(l.burned(g(1)), 4.0);
    }

    #[test]
    fn untagged_creation_is_a_breach() {
        let mut l = ConservationLedger::new(2);
        // Units appear with no provenance line and no counterparty removal.
        l.record_add(g(0), 9.0, Provenance::Transfer);
        assert!(l.worst_breach().is_some(), "unexplained creation must be caught");
    }

    #[test]
    fn shortfall_is_recorded() {
        let mut l = ConservationLedger::new(2);
        l.record_remove(g(0), 10.0, 4.0, Provenance::Consumption);
        assert_eq!(l.shortfalls().len(), 1);
        assert!((l.shortfalls()[0].missing() - 6.0).abs() < 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut l = ConservationLedger::new(2);
        l.record_add(g(0), 3.0, Provenance::Transfer);
        l.record_remove(g(0), 5.0, 1.0, Provenance::Consumption);
        l.reset();
        assert!(l.worst_breach().is_none());
        assert!(l.shortfalls().is_empty());
        assert_eq!(l.max_abs_drift(), 0.0);
    }
}
