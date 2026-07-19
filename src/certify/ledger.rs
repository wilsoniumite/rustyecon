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
/// The invariant is engine.md's, and the left-hand side must be an *independent
/// measurement* or the check is circular:
///
/// ```text
/// Σ inventory[g](close) − Σ inventory[g](open) == minted[g] − burned[g]
/// ```
///
/// `opening` and `closing` come from [`tally`] — a direct scan of every
/// inventory — never from the deltas being checked. Three quantities are
/// reconciled at tick close:
///
/// - `scan_delta` — measured change in Σ inventory (the truth).
/// - `declared` — the sum of provenance-tagged mint/burn lines.
///   [`Provenance::Transfer`] contributes nothing: moving units between
///   inventories does not change the total in existence.
/// - `posted` — accumulated from the deltas themselves (`+qty` per add,
///   `−actually removed` per remove). Cross-checked against `scan_delta` to
///   catch mutation paths that bypass the ledger entirely.
///
/// What this does and does not prove, stated plainly:
///
/// - It **does** catch unbalanced transfers (goods shipped but never received,
///   money credited but never paid), silent clamps, and any inventory mutation
///   that skips the delta path.
/// - It **cannot** validate a recipe's stoichiometry across *different* goods —
///   turning wheat into flour is a modelling choice, not an accounting identity,
///   so a cross-good transform is declared, not derived. A good that appears on
///   *both* sides of a recipe is therefore ledgered as a `Transfer` by
///   `production`, which is what stops a currency-in/currency-out recipe from
///   quietly minting money.
///
/// A breach panics the run (METHODOLOGY R3). Equality holds up to a tolerance
/// with two relative terms — one for the float error in what *moved*, one for the
/// error in *measuring* what is held, since `tally` sums thousands of lots — plus
/// a tiny absolute floor. See [`REL_FLOW`] and [`REL_STOCK`].
#[derive(Debug, Clone)]
pub struct ConservationLedger {
    posted: Vec<f64>,
    declared: Vec<f64>,
    minted: Vec<f64>,
    burned: Vec<f64>,
    /// Σ|qty| moved per good; sets the scale for the drift tolerance.
    gross: Vec<f64>,
    shortfalls: Vec<Shortfall>,
}

/// What one tick's ledger concluded, in the form the certificate consumes.
///
/// The tick has already asserted itself by the time this is produced — a breach
/// panics — so these are the *margins*: how close it ran to the limit, and what
/// was clamped. Without them "zero breaches" is indistinguishable from "just
/// inside tolerance, same sign, every tick".
#[derive(Debug, Clone, Copy, Default)]
pub struct TickAudit {
    pub max_abs_drift: f64,
    pub max_margin_ratio: f64,
    pub shortfalls: usize,
    pub shortfall_qty: f64,
}

impl TickAudit {
    /// Fold another tick's audit into a running worst-case for the run.
    pub fn absorb(&mut self, other: &TickAudit) {
        self.max_abs_drift = self.max_abs_drift.max(other.max_abs_drift);
        self.max_margin_ratio = self.max_margin_ratio.max(other.max_margin_ratio);
        self.shortfalls += other.shortfalls;
        self.shortfall_qty += other.shortfall_qty;
    }
}

/// Σ inventory per good, measured directly from state.
///
/// This is the independent left-hand side of the conservation identity: it is
/// deliberately computed by walking every inventory rather than by replaying the
/// delta stream, so a mutation that bypasses the ledger still shows up.
pub fn tally(inventories: &[crate::types::inventory::Inventory], num_goods: usize) -> Vec<f64> {
    let mut totals = vec![0.0; num_goods];
    for inv in inventories {
        for (good, qty) in inv.goods() {
            if good.idx() < num_goods {
                totals[good.idx()] += qty;
            }
        }
    }
    totals
}

/// Slack per unit of gross flow.
///
/// Clearing splits a market with `buyer_fill = min(1, s/d)` and
/// `seller_fill = min(1, d/s)`, so the two sides of a trade disagree by a few
/// ULP — about 1e-16 relative. 1e-12 leaves four orders of headroom over that
/// without being able to absorb a real leak.
const REL_FLOW: f64 = 1e-12;

/// Slack per unit of stock.
///
/// The opening and closing totals are *measured* by summing every lot of every
/// inventory, and long runs accumulate 10^3–10^4 lots on a single good, so the
/// measurement carries its own ~N·ε·S error (N·ε ≈ 2e-12 at N = 10^4). The flow
/// term cannot cover this: a good can hold large stock while barely trading, and
/// the scan noise scales with what is held, not with what moved.
const REL_STOCK: f64 = 1e-11;

/// Floor, so a good with neither stock nor flow still has a non-zero threshold.
/// Deliberately tiny: an absolute constant is unit-dependent and says nothing
/// meaningful about a good whose natural scale we do not know.
const ABS_TOLERANCE: f64 = 1e-12;

impl ConservationLedger {
    pub fn new(num_goods: usize) -> Self {
        Self {
            posted: vec![0.0; num_goods],
            declared: vec![0.0; num_goods],
            minted: vec![0.0; num_goods],
            burned: vec![0.0; num_goods],
            gross: vec![0.0; num_goods],
            shortfalls: Vec::new(),
        }
    }

    pub fn num_goods(&self) -> usize {
        self.posted.len()
    }

    /// Clear all tallies, keeping the allocation. Called at the top of each tick.
    pub fn reset(&mut self) {
        self.posted.iter_mut().for_each(|v| *v = 0.0);
        self.declared.iter_mut().for_each(|v| *v = 0.0);
        self.minted.iter_mut().for_each(|v| *v = 0.0);
        self.burned.iter_mut().for_each(|v| *v = 0.0);
        self.gross.iter_mut().for_each(|v| *v = 0.0);
        self.shortfalls.clear();
    }

    /// Post units entering an inventory. `qty` always lands in full — `add` never clamps.
    pub fn record_add(&mut self, good: GoodId, qty: f64, prov: Provenance) {
        let i = good.idx();
        debug_assert!(i < self.posted.len(), "good {i} outside ledger width");
        if i >= self.posted.len() {
            return;
        }
        self.posted[i] += qty;
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
        debug_assert!(i < self.posted.len(), "good {i} outside ledger width");
        if i >= self.posted.len() {
            return;
        }
        self.posted[i] -= removed;
        self.gross[i] += removed.abs();
        if prov.is_mint_or_burn() {
            self.declared[i] -= removed;
            self.burned[i] += removed;
        }
        // Relative to what was asked for: a bare absolute threshold would log
        // float noise as a shortfall on large removals and miss real ones on small.
        if requested - removed > ABS_TOLERANCE + REL_FLOW * requested.abs() {
            self.shortfalls.push(Shortfall { good, requested, removed });
        }
    }

    /// Drift tolerance for one good: a floor, plus slack for the float error in
    /// what moved (flow) and in the measurement of what is held (stock).
    fn tolerance(&self, i: usize, opening: &[f64], closing: &[f64]) -> f64 {
        let stock = opening[i].abs().max(closing[i].abs());
        ABS_TOLERANCE + REL_FLOW * self.gross[i] + REL_STOCK * stock
    }

    /// How close the tick ran to its limit, as `|drift| / tolerance` over all
    /// goods. Reported by the certificate so "zero breaches" is distinguishable
    /// from "99% of tolerance, same sign, every tick".
    pub fn max_margin_ratio(&self, opening: &[f64], closing: &[f64]) -> f64 {
        (0..self.posted.len())
            .map(|i| {
                let d = ((closing[i] - opening[i]) - self.declared[i]).abs();
                d / self.tolerance(i, opening, closing)
            })
            .fold(0.0, f64::max)
    }

    /// Per-good conservation error against a measured scan:
    /// `(closing − opening) − (minted − burned)`.
    pub fn drift(&self, good: GoodId, opening: &[f64], closing: &[f64]) -> f64 {
        let i = good.idx();
        (closing[i] - opening[i]) - self.declared[i]
    }

    /// Per-good disagreement between the measured scan and what the deltas
    /// claimed to move. Non-zero means something mutated an inventory without
    /// going through the ledger.
    pub fn unledgered(&self, good: GoodId, opening: &[f64], closing: &[f64]) -> f64 {
        let i = good.idx();
        (closing[i] - opening[i]) - self.posted[i]
    }

    /// The largest breach of the conservation identity, if any.
    /// Returns `(good, absolute drift)`.
    pub fn worst_breach(&self, opening: &[f64], closing: &[f64]) -> Option<(GoodId, f64)> {
        let mut worst: Option<(GoodId, f64, f64)> = None; // (good, |drift|, excess over tol)
        for i in 0..self.posted.len() {
            let d = ((closing[i] - opening[i]) - self.declared[i]).abs();
            let excess = d - self.tolerance(i, opening, closing);
            if excess > 0.0 && worst.map_or(true, |(_, _, w)| excess > w) {
                worst = Some((GoodId(i as u32), d, excess));
            }
        }
        worst.map(|(g, d, _)| (g, d))
    }

    /// The largest movement that reached an inventory without passing through
    /// the ledger, if any. Returns `(good, absolute gap)`.
    pub fn worst_unledgered(&self, opening: &[f64], closing: &[f64]) -> Option<(GoodId, f64)> {
        let mut worst: Option<(GoodId, f64, f64)> = None;
        for i in 0..self.posted.len() {
            let d = ((closing[i] - opening[i]) - self.posted[i]).abs();
            let excess = d - self.tolerance(i, opening, closing);
            if excess > 0.0 && worst.map_or(true, |(_, _, w)| excess > w) {
                worst = Some((GoodId(i as u32), d, excess));
            }
        }
        worst.map(|(g, d, _)| (g, d))
    }

    /// Largest absolute drift across all goods, breaching or not. Reported by the
    /// certificate's conservation battery.
    pub fn max_abs_drift(&self, opening: &[f64], closing: &[f64]) -> f64 {
        (0..self.posted.len())
            .map(|i| ((closing[i] - opening[i]) - self.declared[i]).abs())
            .fold(0.0, f64::max)
    }

    pub fn shortfalls(&self) -> &[Shortfall] {
        &self.shortfalls
    }

    /// Summarise this tick for the certificate.
    pub fn audit(&self, opening: &[f64], closing: &[f64]) -> TickAudit {
        TickAudit {
            max_abs_drift: self.max_abs_drift(opening, closing),
            max_margin_ratio: self.max_margin_ratio(opening, closing),
            shortfalls: self.shortfalls.len(),
            shortfall_qty: self.shortfalls.iter().map(|s| s.missing()).sum(),
        }
    }

    pub fn minted(&self, good: GoodId) -> f64 {
        self.minted[good.idx()]
    }

    pub fn burned(&self, good: GoodId) -> f64 {
        self.burned[good.idx()]
    }

    /// Panic if the tick failed to conserve (METHODOLOGY R3).
    ///
    /// `opening`/`closing` must come from [`tally`] — a direct scan of state, not
    /// from the delta stream — so the identity is checked against an independent
    /// measurement rather than against itself.
    ///
    /// A conservation failure is a bug by definition — money leaks, goods
    /// duplication, and phantom consumption must be impossible to miss — so this
    /// aborts the run rather than letting a corrupt trajectory reach a notebook.
    pub fn assert_conserved(&self, tick: u64, opening: &[f64], closing: &[f64]) {
        if let Some((good, gap)) = self.worst_unledgered(opening, closing) {
            let i = good.idx();
            panic!(
                "unledgered mutation at tick {tick}: good {} moved {gap:+.6e} outside the ledger\n  \
                 measured Σ change {:+.6e} != movements posted through apply {:+.6e}\n  \
                 (something wrote an inventory without going through a ledgered delta)",
                good.0,
                closing[i] - opening[i],
                self.posted[i],
            );
        }
        if let Some((good, drift)) = self.worst_breach(opening, closing) {
            let i = good.idx();
            // Shortfalls are the usual culprit: a removal that could not take what
            // it asked for leaves its counterparty add unmatched. Report them here
            // so the ledger line points at the cause instead of just the symptom.
            let lines: Vec<String> = self
                .shortfalls
                .iter()
                .filter(|s| s.good == good)
                .map(|s| format!("requested {:.6e}, removed {:.6e}", s.requested, s.removed))
                .collect();
            let shortfall_note = if lines.is_empty() {
                "  no shortfalls on this good".to_string()
            } else {
                format!(
                    "  {} shortfall line(s) on this good, total missing {:.6e}:\n    {}",
                    lines.len(),
                    self.shortfalls
                        .iter()
                        .filter(|s| s.good == good)
                        .map(|s| s.missing())
                        .sum::<f64>(),
                    lines.join("\n    ")
                )
            };
            panic!(
                "conservation failure at tick {tick}: good {} drifted {drift:+.6e}\n  \
                 measured Σ change {:+.6e} != declared mint/burn {:+.6e}\n  \
                 minted {:.6e}, burned {:.6e}, gross flow {:.6e}, tolerance {:.3e}\n  \
                 (an unbalanced transfer, or a create/destroy with no provenance line)\n{}",
                good.0,
                closing[i] - opening[i],
                self.declared[i],
                self.minted[i],
                self.burned[i],
                self.gross[i],
                self.tolerance(i, opening, closing),
                shortfall_note,
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
        // Seller ships 5, buyer receives 5; the measured total is unchanged.
        l.record_remove(g(0), 5.0, 5.0, Provenance::Transfer);
        l.record_add(g(0), 5.0, Provenance::Transfer);
        let (open, close) = (vec![10.0, 0.0], vec![10.0, 0.0]);
        assert!(l.worst_breach(&open, &close).is_none());
        l.assert_conserved(1, &open, &close);
    }

    #[test]
    fn unbalanced_transfer_is_a_breach() {
        let mut l = ConservationLedger::new(2);
        // Seller ships 5 but buyer only receives 3 — two units really vanish.
        l.record_remove(g(0), 5.0, 5.0, Provenance::Transfer);
        l.record_add(g(0), 3.0, Provenance::Transfer);
        let (open, close) = (vec![10.0, 0.0], vec![8.0, 0.0]);
        let (good, drift) = l.worst_breach(&open, &close).expect("must detect the leak");
        assert_eq!(good, g(0));
        assert!((drift - 2.0).abs() < 1e-12, "drift was {drift}");
    }

    #[test]
    fn declared_mint_and_burn_conserve() {
        let mut l = ConservationLedger::new(2);
        l.record_add(g(1), 7.0, Provenance::Production); // minted
        l.record_remove(g(1), 4.0, 4.0, Provenance::Consumption); // burned
        let (open, close) = (vec![0.0, 0.0], vec![0.0, 3.0]); // +7 −4
        assert!(l.worst_breach(&open, &close).is_none());
        assert_eq!(l.minted(g(1)), 7.0);
        assert_eq!(l.burned(g(1)), 4.0);
    }

    #[test]
    fn untagged_creation_is_a_breach() {
        let mut l = ConservationLedger::new(2);
        // Units appear with no provenance line and no counterparty removal.
        l.record_add(g(0), 9.0, Provenance::Transfer);
        let (open, close) = (vec![0.0, 0.0], vec![9.0, 0.0]);
        assert!(
            l.worst_breach(&open, &close).is_some(),
            "unexplained creation must be caught"
        );
    }

    #[test]
    fn stock_appearing_beyond_what_was_declared_is_a_breach() {
        // The whole point of measuring `closing` independently: a mint larger
        // than any delta accounted for cannot hide behind a self-declared tag.
        let mut l = ConservationLedger::new(2);
        l.record_add(g(0), 5.0, Provenance::Production);
        let (open, close) = (vec![0.0, 0.0], vec![8.0, 0.0]); // 3 unexplained units
        assert!(l.worst_breach(&open, &close).is_some());
    }

    #[test]
    fn mutation_that_skips_the_ledger_is_caught() {
        // No delta posted at all, yet the measured stock moved.
        let l = ConservationLedger::new(2);
        let (open, close) = (vec![0.0, 0.0], vec![4.0, 0.0]);
        assert!(
            l.worst_unledgered(&open, &close).is_some(),
            "a write that bypasses apply must be caught"
        );
    }

    #[test]
    fn tolerance_covers_scan_noise_on_large_stock() {
        // `tally` sums thousands of lots, so measuring a stock of ~1e3 carries
        // ~1e-9 of its own error. That must not be reported as a leak.
        let mut l = ConservationLedger::new(2);
        l.record_add(g(0), 1.0, Provenance::Transfer);
        l.record_remove(g(0), 1.0, 1.0, Provenance::Transfer);
        let open = vec![1.0e3, 0.0];
        let close = vec![1.0e3 + 1.0e-9, 0.0];
        assert!(
            l.worst_breach(&open, &close).is_none(),
            "measurement noise proportional to stock must be tolerated"
        );
    }

    #[test]
    fn tolerance_still_catches_a_real_leak_at_the_same_stock() {
        // Same stock scale, but a loss six orders larger than the scan noise.
        let mut l = ConservationLedger::new(2);
        l.record_add(g(0), 1.0, Provenance::Transfer);
        l.record_remove(g(0), 1.0, 1.0, Provenance::Transfer);
        let open = vec![1.0e3, 0.0];
        let close = vec![1.0e3 - 1.0e-3, 0.0];
        let (good, _) = l
            .worst_breach(&open, &close)
            .expect("a real leak must survive the stock-scaled tolerance");
        assert_eq!(good, g(0));
    }

    #[test]
    fn margin_ratio_reports_headroom() {
        let mut l = ConservationLedger::new(2);
        l.record_add(g(0), 1.0, Provenance::Transfer);
        l.record_remove(g(0), 1.0, 1.0, Provenance::Transfer);
        let open = vec![0.0, 0.0];
        let close = vec![0.0, 0.0];
        // Perfectly balanced: no drift, so no headroom consumed.
        assert_eq!(l.max_margin_ratio(&open, &close), 0.0);
        // Drifting to exactly the limit reports a ratio of 1.
        let close = vec![ABS_TOLERANCE, 0.0];
        let ratio = l.max_margin_ratio(&open, &close);
        assert!(ratio > 0.0 && ratio <= 1.0 + 1e-9, "ratio was {ratio}");
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
        let zero = vec![0.0, 0.0];
        assert!(l.worst_breach(&zero, &zero).is_none());
        assert!(l.shortfalls().is_empty());
        assert_eq!(l.max_abs_drift(&zero, &zero), 0.0);
    }
}
