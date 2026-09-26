//! The conservation ledger (docs/ENGINE.md §2.5; R2; REVIEW §2.2 defect 9; ADDENDUM N2, N3,
//! A12), salvaged from `v2p3: certify/ledger.rs`.
//!
//! One ledger opens before phase 0 and closes after phase 6. For every good,
//!
//! ```text
//! drift = (closing − opening) − declared,
//! ```
//!
//! where `opening` and `closing` are measured by walking every holding, independently of the
//! deltas being checked, and `declared` is the signed sum of the tick's mints and burns. A
//! transfer declares nothing, so a move that loses or creates units, or a write that bypasses
//! `apply`, shows up as drift. The tick passes when `|drift| <= tol` for every good, with
//!
//! ```text
//! tol = rel_flow·gross + rel_stock·max(|opening|, |closing|),
//! ```
//!
//! both tolerances registered, dimensionless and fixed. July's absolute floor is gone (A12):
//! with no stock and no flow the drift must be exactly zero. A shortfall is a ledger line and an
//! error that stops the tick (N3), never a clamp; July only logged burn shortfalls, and its
//! `run_tick` then discarded the audit. Every id is checked in every profile (N2): July skipped
//! an undefined good behind a `debug_assert`.
//!
//! What `f64` lots create or destroy when a split or a merge rounds is declared, measured
//! exactly, under the provenance `Rounding`, so the tolerance covers only the walk's own
//! rounding and the fold of the declared lines. A leak below each tick's tolerance could still
//! add up over a run, so a [`RunLedger`] folds every tick's ledger into run totals and checks
//! the same identity, with the same registered tolerances, over the whole run: the walks'
//! rounding telescopes, and a leak does not.

use crate::delta::{Phase, Provenance};
use crate::error::CoreError;
use crate::ext::Ext;
use crate::ids::{GoodId, Holder};
use crate::num::is_clean;
use crate::state::SimState;
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A transfer or burn that asked for more than its source held. Nothing of it moved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortfallLine {
    /// The tick.
    pub tick: u64,
    /// The phase.
    pub phase: Phase,
    /// The source.
    pub holder: Holder,
    /// The good.
    pub good: GoodId,
    /// What was asked for.
    pub requested: f64,
    /// What was held.
    pub held: f64,
    /// The burn's provenance; `None` for a transfer.
    pub prov: Option<Provenance>,
}

impl fmt::Display for ShortfallLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.prov {
            Some(p) => format!("a {p:?} burn"),
            None => "a transfer".to_string(),
        };
        write!(
            f,
            "shortfall at tick {}, phase {}: {what} asked {} for {:e} of {} and it held {:e} \
             (short {:e}); nothing moved",
            self.tick,
            self.phase,
            self.holder,
            self.requested,
            self.good,
            self.held,
            self.requested - self.held
        )
    }
}

/// One good's failure to conserve, over one tick or over a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Breach {
    /// The tick whose close found it.
    pub tick: u64,
    /// The tick the accounting opened at: `tick` for a tick's ledger, the run's first tick for
    /// a [`RunLedger`]'s.
    pub since: u64,
    /// The good.
    pub good: GoodId,
    /// Its measured total at the open.
    pub opening: f64,
    /// Its measured total at the close.
    pub closing: f64,
    /// The signed sum of its mints, burns, spoilage and rounding.
    pub declared: f64,
    /// The quantity of it moved, minted, burned, spoiled or rounded.
    pub gross: f64,
    /// `(closing − opening) − declared`.
    pub drift: f64,
    /// The tolerance it exceeded.
    pub tol: f64,
    /// Its ledger lines, by provenance.
    pub lines: Vec<(Provenance, f64)>,
}

impl fmt::Display for Breach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.since == self.tick {
            write!(f, "conservation failure at tick {}", self.tick)?;
        } else {
            write!(
                f,
                "run conservation failure over ticks {} to {}",
                self.since, self.tick
            )?;
        }
        write!(
            f,
            ": {} drifted {:+e} (tolerance {:e}); measured change {:+e} from {:e} to {:e}, \
             declared {:+e}, gross flow {:e}; lines {:?}",
            self.good,
            self.drift,
            self.tol,
            self.closing - self.opening,
            self.opening,
            self.closing,
            self.declared,
            self.gross,
            self.lines
        )
    }
}

/// What a closed tick's ledger found. It always reaches the caller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickAudit {
    /// The net declared quantity per (good, provenance): mints positive, burns and spoilage
    /// negative.
    pub lines: Vec<(GoodId, Provenance, f64)>,
    /// The largest `|drift|/tol` over goods, 0 where the drift is exactly 0; at most 1.
    pub max_margin: f64,
    /// The largest `|drift|` over goods.
    pub max_drift: f64,
}

/// One tick's conservation accounting.
#[derive(Debug, Clone, PartialEq)]
pub struct Ledger {
    tick: u64,
    opening: Vec<f64>,
    declared: Vec<f64>,
    gross: Vec<f64>,
    lines: BTreeMap<(GoodId, Provenance), f64>,
    shortfall: Option<ShortfallLine>,
    /// Every declaration in the order it was posted, before the lines fold it, so core's tests
    /// can check a delta's declarations against its lots exactly.
    #[cfg(test)]
    posts: Vec<(GoodId, Provenance, f64)>,
}

/// Measure every good's total over every holding, a left fold in holder order.
fn walk<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<Vec<f64>, CoreError> {
    let mut totals = vec![0.0; w.n_goods()];
    for (h, inv) in s.holdings() {
        if !w.is_holder(*h) {
            return Err(CoreError::UnknownHolder(*h));
        }
        for (g, q) in inv.goods() {
            let t = totals.get_mut(g.idx()).ok_or(CoreError::UnknownGood(g))?;
            *t += q;
        }
    }
    Ok(totals)
}

/// The registered tolerances, read from the current params through their `Value` sites.
fn tolerances<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<(f64, f64), CoreError> {
    let params = s.params(&w.registry);
    let rel_flow = w.tol.rel_flow.per_tick(&params, &w.clock)?;
    let rel_stock = w.tol.rel_stock.per_tick(&params, &w.clock)?;
    Ok((rel_flow, rel_stock))
}

/// The lines in (good, provenance) order.
fn flat(lines: &BTreeMap<(GoodId, Provenance), f64>) -> Vec<(GoodId, Provenance, f64)> {
    lines.iter().map(|(&(g, p), &q)| (g, p, q)).collect()
}

/// One accounting to check, a tick's or a run's: per good, what was measured at its open and
/// close, and what was declared and moved in between.
struct Totals<'a> {
    tick: u64,
    since: u64,
    opening: &'a [f64],
    closing: &'a [f64],
    declared: &'a [f64],
    gross: &'a [f64],
    lines: &'a BTreeMap<(GoodId, Provenance), f64>,
}

impl Totals<'_> {
    /// Check every good: `|drift| <= tol`, with `drift = (closing − opening) − declared` and
    /// `tol = rel_flow·gross + rel_stock·max(|opening|, |closing|)`. Returns the largest margin,
    /// the largest `|drift|` and each good's drift, or the first good's breach.
    fn check(&self, rel_flow: f64, rel_stock: f64) -> Result<(f64, f64, Vec<f64>), CoreError> {
        let n = self.opening.len();
        if [self.closing.len(), self.declared.len(), self.gross.len()] != [n, n, n] {
            return Err(CoreError::Shape(format!(
                "a ledger over {n} goods closed on a world of {}",
                self.closing.len()
            )));
        }
        let mut max_margin = 0.0_f64;
        let mut max_drift = 0.0_f64;
        let mut drifts = Vec::with_capacity(n);
        for (i, (&open, &close)) in self.opening.iter().zip(self.closing).enumerate() {
            let drift = (close - open) - self.declared[i];
            let tol = rel_flow * self.gross[i] + rel_stock * open.abs().max(close.abs());
            // NaN fails this comparison, so a NaN drift is a breach.
            let conserved = drift.abs() <= tol;
            if !conserved {
                let good = GoodId(u32::try_from(i).unwrap_or(u32::MAX));
                let lines = self
                    .lines
                    .iter()
                    .filter(|(&(g, _), _)| g == good)
                    .map(|(&(_, p), &q)| (p, q))
                    .collect();
                return Err(CoreError::Conservation(Box::new(Breach {
                    tick: self.tick,
                    since: self.since,
                    good,
                    opening: open,
                    closing: close,
                    declared: self.declared[i],
                    gross: self.gross[i],
                    drift,
                    tol,
                    lines,
                })));
            }
            let margin = if drift == 0.0 { 0.0 } else { drift.abs() / tol };
            max_margin = max_margin.max(margin);
            max_drift = max_drift.max(drift.abs());
            drifts.push(drift);
        }
        Ok((max_margin, max_drift, drifts))
    }
}

fn no_escrow<E: Ext>(s: &SimState<E>) -> Result<(), CoreError> {
    match s
        .holdings()
        .keys()
        .find(|h| matches!(h, Holder::Escrow(..)))
    {
        Some(Holder::Escrow(node, good)) => Err(CoreError::EscrowLeft {
            node: *node,
            good: *good,
        }),
        _ => Ok(()),
    }
}

impl Ledger {
    /// Open a tick's ledger on the state before phase 0: measure every good. An escrow left
    /// over, a holder outside the world or an undefined good is an error.
    pub fn open<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<Ledger, CoreError> {
        no_escrow(s)?;
        let opening = walk(s, w)?;
        let n = opening.len();
        Ok(Ledger {
            tick: s.tick(),
            opening,
            declared: vec![0.0; n],
            gross: vec![0.0; n],
            lines: BTreeMap::new(),
            shortfall: None,
            #[cfg(test)]
            posts: Vec::new(),
        })
    }

    /// Close the tick: fail on a recorded shortfall or a remaining escrow, then measure every
    /// good again and check it against what was declared, with the tolerances read from the
    /// current params.
    pub fn close<E: Ext>(self, s: &SimState<E>, w: &World<E>) -> Result<TickAudit, CoreError> {
        self.close_totals(s, w).map(|(audit, _)| audit)
    }

    /// [`Ledger::close`], also returning the closing walk.
    fn close_totals<E: Ext>(
        self,
        s: &SimState<E>,
        w: &World<E>,
    ) -> Result<(TickAudit, Vec<f64>), CoreError> {
        if let Some(line) = self.shortfall {
            return Err(CoreError::Shortfall(line));
        }
        no_escrow(s)?;
        let closing = walk(s, w)?;
        let (rel_flow, rel_stock) = tolerances(s, w)?;
        let totals = Totals {
            tick: self.tick,
            since: self.tick,
            opening: &self.opening,
            closing: &closing,
            declared: &self.declared,
            gross: &self.gross,
            lines: &self.lines,
        };
        let (max_margin, max_drift, _) = totals.check(rel_flow, rel_stock)?;
        let audit = TickAudit {
            lines: flat(&self.lines),
            max_margin,
            max_drift,
        };
        Ok((audit, closing))
    }

    /// The tick the ledger opened at.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// The net declared quantity per (good, provenance) so far.
    pub fn lines(&self) -> &BTreeMap<(GoodId, Provenance), f64> {
        &self.lines
    }

    /// The shortfall that stopped the tick, if one did.
    pub fn shortfall(&self) -> Option<&ShortfallLine> {
        self.shortfall.as_ref()
    }

    /// The number of goods the ledger covers.
    pub(crate) fn width(&self) -> usize {
        self.opening.len()
    }

    /// Post a quantity moved between holders.
    pub(crate) fn moved(&mut self, g: GoodId, q: f64) {
        self.gross[g.idx()] += q;
    }

    /// Post a mint (positive `q`) or a burn (negative `q`) with its provenance.
    pub(crate) fn declare(&mut self, g: GoodId, prov: Provenance, q: f64) {
        self.declared[g.idx()] += q;
        self.gross[g.idx()] += q.abs();
        *self.lines.entry((g, prov)).or_insert(0.0) += q;
        #[cfg(test)]
        self.posts.push((g, prov, q));
    }

    /// Every declaration posted so far, in order, unfolded.
    #[cfg(test)]
    pub(crate) fn posts(&self) -> &[(GoodId, Provenance, f64)] {
        &self.posts
    }

    /// Post what a split or a merge created by rounding (negative: destroyed), if anything.
    pub(crate) fn round(&mut self, g: GoodId, q: f64) {
        if q != 0.0 {
            self.declare(g, Provenance::Rounding, q);
        }
    }

    /// Record the shortfall that stops the tick.
    pub(crate) fn record_shortfall(&mut self, line: ShortfallLine) {
        self.shortfall = Some(line);
    }
}

/// What a run's ledger found so far: every tick's ledger since the run began, folded together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunAudit {
    /// The run's first tick: the `Sim`'s starting tick, genesis or a resumed checkpoint's.
    pub since: u64,
    /// The net declared quantity per (good, provenance) over the run: the ticks' lines summed.
    pub lines: Vec<(GoodId, Provenance, f64)>,
    /// Per good, in good order, the run's drift: `(closing − opening) − declared` from the
    /// run's first open to this close.
    pub drift: Vec<(GoodId, f64)>,
    /// The largest `|drift|/tol` over goods, with the run's gross flow in `tol`; at most 1.
    pub max_margin: f64,
}

/// A run's conservation accounting (R2). Each tick's [`Ledger`] passes through
/// [`RunLedger::close_tick`], which closes it and folds its declared quantities, gross flows and
/// lines into run totals. The run is then checked like a tick, from the run's opening walk to
/// this tick's closing walk, with the same registered tolerances and the run's gross flow. The
/// walks' own rounding telescopes, since each tick opens on the state the last one closed on,
/// so the run's drift stays at the size of one tick's; a leak below each tick's tolerance adds
/// up and is caught.
///
/// A checkpoint carries it, inside its digest (amended at P0.9, O8), so a resumed run continues
/// the audit of the run that reached the checkpoint: a leak that stops an uninterrupted run
/// stops the resumed one at the same tick, whatever the checkpoint cadence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "RunRepr", try_from = "RunRepr")]
pub struct RunLedger {
    since: u64,
    next: u64,
    opening: Vec<f64>,
    declared: Vec<f64>,
    gross: Vec<f64>,
    lines: BTreeMap<(GoodId, Provenance), f64>,
}

/// The serialised form of a [`RunLedger`]: its totals, with the lines as a list in (good,
/// provenance) order.
#[derive(Serialize, Deserialize)]
#[serde(rename = "RunLedger", deny_unknown_fields)]
struct RunRepr {
    since: u64,
    next: u64,
    opening: Vec<f64>,
    declared: Vec<f64>,
    gross: Vec<f64>,
    lines: Vec<(GoodId, Provenance, f64)>,
}

impl From<RunLedger> for RunRepr {
    fn from(r: RunLedger) -> RunRepr {
        RunRepr {
            since: r.since,
            next: r.next,
            opening: r.opening,
            declared: r.declared,
            gross: r.gross,
            lines: flat(&r.lines),
        }
    }
}

/// Decoding refuses columns of different lengths, a run that ends before it began, a total
/// that is not finite (or, for a walk or a gross flow, has its sign bit set), a good outside
/// the columns, and lines out of order or repeated.
impl TryFrom<RunRepr> for RunLedger {
    type Error = String;

    fn try_from(r: RunRepr) -> Result<RunLedger, String> {
        let n = r.opening.len();
        if r.declared.len() != n || r.gross.len() != n {
            return Err("the run ledger's columns differ in length".into());
        }
        if r.next < r.since {
            return Err(format!(
                "the run ledger's next tick {} is before its first {}",
                r.next, r.since
            ));
        }
        let clean = r.opening.iter().chain(&r.gross).all(|&v| is_clean(v));
        let finite = r.declared.iter().all(|v| v.is_finite());
        if !clean || !finite {
            return Err("the run ledger holds a value that is not allowed".into());
        }
        let mut lines = BTreeMap::new();
        let mut last = None;
        for (g, p, q) in r.lines {
            if g.idx() >= n || !q.is_finite() || last.is_some_and(|k| k >= (g, p)) {
                return Err(format!("the run ledger's line ({g}, {p:?}) is not allowed"));
            }
            last = Some((g, p));
            lines.insert((g, p), q);
        }
        Ok(RunLedger {
            since: r.since,
            next: r.next,
            opening: r.opening,
            declared: r.declared,
            gross: r.gross,
            lines,
        })
    }
}

impl RunLedger {
    /// Open a run's ledger on the state its first tick runs on: measure every good.
    pub fn open<E: Ext>(s: &SimState<E>, w: &World<E>) -> Result<RunLedger, CoreError> {
        no_escrow(s)?;
        let opening = walk(s, w)?;
        let n = opening.len();
        Ok(RunLedger {
            since: s.tick(),
            next: s.tick(),
            opening,
            declared: vec![0.0; n],
            gross: vec![0.0; n],
            lines: BTreeMap::new(),
        })
    }

    /// The run's first tick.
    pub fn since(&self) -> u64 {
        self.since
    }

    /// The tick whose ledger it closes next: the tick of the state it has reached.
    pub fn next(&self) -> u64 {
        self.next
    }

    /// Whether it can continue a run of `w` from a state at `tick`: it covers `w`'s goods and
    /// has closed every tick before `tick`. A checkpoint's is checked so on resume.
    pub fn fits<E: Ext>(&self, w: &World<E>, tick: u64) -> Result<(), CoreError> {
        if self.opening.len() != w.n_goods() {
            return Err(CoreError::Shape(format!(
                "a run ledger over {} goods on a world of {}",
                self.opening.len(),
                w.n_goods()
            )));
        }
        if self.next != tick {
            return Err(CoreError::Shape(format!(
                "a run ledger at tick {} beside a state at tick {tick}",
                self.next
            )));
        }
        Ok(())
    }

    /// Close the run's next tick: close `l` as [`Ledger::close`] does, fold it into the run,
    /// and check the run. A ledger of another tick than the next is a shape error; a tick's
    /// breach, or the run's, is [`CoreError::Conservation`].
    pub fn close_tick<E: Ext>(
        &mut self,
        l: Ledger,
        s: &SimState<E>,
        w: &World<E>,
    ) -> Result<(TickAudit, RunAudit), CoreError> {
        if l.tick != self.next || l.width() != self.opening.len() {
            return Err(CoreError::Shape(format!(
                "a run ledger at tick {} over {} goods given tick {}'s ledger over {}",
                self.next,
                self.opening.len(),
                l.tick,
                l.width()
            )));
        }
        let tick = l.tick;
        let (declared, gross, lines) = (l.declared.clone(), l.gross.clone(), l.lines.clone());
        let (audit, closing) = l.close_totals(s, w)?;
        for (run, q) in self.declared.iter_mut().zip(&declared) {
            *run += q;
        }
        for (run, q) in self.gross.iter_mut().zip(&gross) {
            *run += q;
        }
        for (key, q) in lines {
            *self.lines.entry(key).or_insert(0.0) += q;
        }
        self.next = tick.saturating_add(1);
        let (rel_flow, rel_stock) = tolerances(s, w)?;
        let totals = Totals {
            tick,
            since: self.since,
            opening: &self.opening,
            closing: &closing,
            declared: &self.declared,
            gross: &self.gross,
            lines: &self.lines,
        };
        let (max_margin, _, drifts) = totals.check(rel_flow, rel_stock)?;
        let run = RunAudit {
            since: self.since,
            lines: flat(&self.lines),
            drift: drifts
                .into_iter()
                .enumerate()
                .map(|(i, d)| (GoodId(u32::try_from(i).unwrap_or(u32::MAX)), d))
                .collect(),
            max_margin,
        };
        Ok((audit, run))
    }
}

#[cfg(test)]
mod tests {
    //! July's eleven ledger tests (`v2p3: certify/ledger.rs:359-488`), names kept, and the new
    //! ones of docs/ENGINE.md §11. An unbalanced move or an untagged creation can no longer be
    //! expressed as a delta (a transfer is atomic, a mint needs a provenance), so those tests
    //! write the state directly, which is exactly what the ledger exists to catch.

    use super::*;
    use crate::delta::StateDelta;
    use crate::inventory::Amount;
    use crate::testkit::{self, good, held, holder, tick, write_direct};
    use crate::NoExt;

    /// Bar for how close the largest passing drift sits to the limit. One ulp of the fixture's
    /// grain stock (16) is 3.6e-15, which is 2.2e-5 of its stock tolerance (1e-11 · 16), so the
    /// largest passing drift is within that fraction of the limit; 1e-4 bounds it with room.
    const LIMIT_BAR: f64 = 1e-4;

    fn transfer(from: Holder, to: Holder, good: GoodId, q: f64) -> StateDelta<NoExt> {
        StateDelta::Transfer {
            from,
            to,
            good,
            amount: Amount::Qty(q),
        }
    }

    fn breach(e: CoreError) -> Breach {
        match e {
            CoreError::Conservation(b) => *b,
            other => panic!("expected a conservation breach, got {other}"),
        }
    }

    #[test]
    fn balanced_transfer_conserves() {
        let (w, mut s) = testkit::load();
        let (farm, mill, grain) = (holder(&w, "farm"), holder(&w, "mill"), good(&w, "grain"));
        let audit = tick(&mut s, &w, |s, l| {
            crate::apply(
                s,
                &w,
                Phase::Decisions,
                &[transfer(farm, mill, grain, 5.0)],
                l,
            )?;
            Ok(())
        })
        .unwrap();
        assert_eq!(audit.max_margin, 0.0);
        assert_eq!(audit.max_drift, 0.0);
        assert!(audit.lines.is_empty(), "a transfer declares nothing");
        assert_eq!(held(&s, mill, grain), 5.0);
    }

    #[test]
    fn unbalanced_transfer_is_a_breach() {
        let (w, mut s) = testkit::load();
        let (farm, mill, grain) = (holder(&w, "farm"), holder(&w, "mill"), good(&w, "grain"));
        // The seller ships 5 but the buyer receives only 3: two units vanish.
        let err = tick(&mut s, &w, |s, _| {
            write_direct(s, farm, grain, -5.0);
            write_direct(s, mill, grain, 3.0);
            Ok(())
        })
        .unwrap_err();
        let b = breach(err);
        assert_eq!(b.good, grain);
        assert_eq!(b.drift, -2.0);
    }

    #[test]
    fn declared_mint_and_burn_conserve() {
        let (w, mut s) = testkit::load();
        let (mill, bread) = (holder(&w, "mill"), good(&w, "bread"));
        let audit = tick(&mut s, &w, |s, l| {
            let ds = [
                StateDelta::Mint {
                    to: mill,
                    good: bread,
                    qty: 7.0,
                    prov: Provenance::Production,
                },
                StateDelta::Burn {
                    from: mill,
                    good: bread,
                    amount: Amount::Qty(4.0),
                    prov: Provenance::Consumption,
                },
            ];
            crate::apply(s, &w, Phase::Production, &ds, l)?;
            Ok(())
        })
        .unwrap();
        assert_eq!(
            audit.lines,
            vec![
                (bread, Provenance::Production, 7.0),
                (bread, Provenance::Consumption, -4.0)
            ]
        );
        assert_eq!(held(&s, mill, bread), 6.0);
    }

    #[test]
    fn untagged_creation_is_a_breach() {
        let (w, mut s) = testkit::load();
        let (farm, grain) = (holder(&w, "farm"), good(&w, "grain"));
        let err = tick(&mut s, &w, |s, _| {
            write_direct(s, farm, grain, 9.0);
            Ok(())
        })
        .unwrap_err();
        assert_eq!(breach(err).drift, 9.0);
    }

    #[test]
    fn stock_appearing_beyond_what_was_declared_is_a_breach() {
        // Measuring `closing` independently is the point: a mint larger than any delta declared
        // cannot hide behind a self-declared tag.
        let (w, mut s) = testkit::load();
        let (farm, grain) = (holder(&w, "farm"), good(&w, "grain"));
        let err = tick(&mut s, &w, |s, l| {
            let mint = StateDelta::Mint {
                to: farm,
                good: grain,
                qty: 5.0,
                prov: Provenance::Production,
            };
            crate::apply(s, &w, Phase::Production, &[mint], l)?;
            write_direct(s, farm, grain, 3.0);
            Ok(())
        })
        .unwrap_err();
        let b = breach(err);
        assert_eq!((b.good, b.declared, b.drift), (grain, 5.0, 3.0));
        assert_eq!(b.lines, vec![(Provenance::Production, 5.0)]);
    }

    #[test]
    fn mutation_that_skips_the_ledger_is_caught() {
        // No delta at all, yet the measured stock moved.
        let (w, mut s) = testkit::load();
        let (workers, coin) = (holder(&w, "workers"), good(&w, "coin"));
        let err = tick(&mut s, &w, |s, _| {
            write_direct(s, workers, coin, 4.0);
            Ok(())
        })
        .unwrap_err();
        let b = breach(err);
        assert_eq!((b.good, b.drift), (coin, 4.0));
    }

    #[test]
    fn tolerance_covers_scan_noise_on_large_stock() {
        // Measuring a stock of about 1e3 carries its own rounding; a drift proportional to the
        // stock and far below a real leak must pass.
        let (w, mut s) = testkit::load();
        let (farm, mill, grain) = (holder(&w, "farm"), holder(&w, "mill"), good(&w, "grain"));
        write_direct(&mut s, farm, grain, 984.0);
        tick(&mut s, &w, |s, l| {
            crate::apply(
                s,
                &w,
                Phase::Decisions,
                &[transfer(farm, mill, grain, 1.0)],
                l,
            )?;
            write_direct(s, farm, grain, 1e-9);
            Ok(())
        })
        .expect("measurement noise proportional to stock is tolerated");
    }

    #[test]
    fn tolerance_still_catches_a_real_leak_at_the_same_stock() {
        let (w, mut s) = testkit::load();
        let (farm, mill, grain) = (holder(&w, "farm"), holder(&w, "mill"), good(&w, "grain"));
        write_direct(&mut s, farm, grain, 984.0);
        let err = tick(&mut s, &w, |s, l| {
            crate::apply(
                s,
                &w,
                Phase::Decisions,
                &[transfer(farm, mill, grain, 1.0)],
                l,
            )?;
            write_direct(s, farm, grain, -1e-3);
            Ok(())
        })
        .unwrap_err();
        assert_eq!(breach(err).good, grain);
    }

    #[test]
    fn margin_ratio_reports_headroom() {
        let (w, base) = testkit::load();
        let (farm, pensioners, grain) = (
            holder(&w, "farm"),
            holder(&w, "pensioners"),
            good(&w, "grain"),
        );
        // Balanced: no drift, no headroom used.
        let mut s = base.clone();
        let audit = tick(&mut s, &w, |s, l| {
            crate::apply(
                s,
                &w,
                Phase::Decisions,
                &[transfer(farm, pensioners, grain, 1.0)],
                l,
            )?;
            Ok(())
        })
        .unwrap();
        assert_eq!(audit.max_margin, 0.0);
        // Drift towards the limit in whole ulps of the stock (16): the largest passing drift
        // reports a margin just under 1, and one more ulp breaches.
        let ulp = 16.0_f64.next_up() - 16.0;
        let limit = 1e-11 * 16.0;
        let mut k = (limit / ulp) as u64 - 3;
        let mut last = None;
        loop {
            let mut s = base.clone();
            let d = k as f64 * ulp;
            match tick(&mut s, &w, |s, _| {
                write_direct(s, pensioners, grain, d);
                Ok(())
            }) {
                Ok(audit) => last = Some(audit.max_margin),
                Err(e) => {
                    let b = breach(e);
                    assert!(b.drift.abs() > b.tol);
                    break;
                }
            }
            k += 1;
        }
        let m = last.expect("some drift passed");
        assert!(m <= 1.0 && m > 1.0 - LIMIT_BAR, "margin {m}");
    }

    #[test]
    fn shortfall_is_recorded() {
        let (w, mut s) = testkit::load();
        let (pensioners, grain) = (holder(&w, "pensioners"), good(&w, "grain"));
        let mut l = Ledger::open(&s, &w).unwrap();
        let burn = StateDelta::Burn {
            from: pensioners,
            good: grain,
            amount: Amount::Qty(10.0),
            prov: Provenance::Consumption,
        };
        let err = crate::apply(&mut s, &w, Phase::Production, &[burn], &mut l).unwrap_err();
        let line = ShortfallLine {
            tick: 0,
            phase: Phase::Production,
            holder: pensioners,
            good: grain,
            requested: 10.0,
            held: 0.0,
            prov: Some(Provenance::Consumption),
        };
        assert_eq!(err, CoreError::Shortfall(line.clone()));
        assert_eq!(l.shortfall(), Some(&line));
        assert_eq!(l.close(&s, &w), Err(CoreError::Shortfall(line)));
    }

    #[test]
    fn reset_clears_state() {
        // July reset one ledger per tick; now each tick opens a fresh one, which has no lines.
        let (w, s) = testkit::load();
        let l = Ledger::open(&s, &w).unwrap();
        assert!(l.lines().is_empty());
        assert!(l.shortfall().is_none());
        let audit = l.close(&s, &w).unwrap();
        assert_eq!(
            audit,
            TickAudit {
                lines: vec![],
                max_margin: 0.0,
                max_drift: 0.0
            }
        );
    }

    #[test]
    fn undefined_good_is_an_error_in_release() {
        // July skipped an undefined good behind a debug_assert, so release builds ran on (N2).
        let (w, mut s) = testkit::load();
        let (farm, mill) = (holder(&w, "farm"), holder(&w, "mill"));
        let ghost = GoodId(99);
        let mut l = Ledger::open(&s, &w).unwrap();
        let bad: Vec<StateDelta<NoExt>> = vec![
            StateDelta::Mint {
                to: farm,
                good: ghost,
                qty: 1e6,
                prov: Provenance::Event,
            },
            transfer(farm, mill, ghost, 1.0),
            StateDelta::Burn {
                from: farm,
                good: ghost,
                amount: Amount::All,
                prov: Provenance::Event,
            },
        ];
        for d in bad {
            let e = crate::apply(&mut s, &w, Phase::Events, &[d], &mut l).unwrap_err();
            assert_eq!(e, CoreError::UnknownGood(ghost));
        }
        // A good that reached a holding without apply stops the ledger at either end.
        let mut t = s.clone();
        write_direct(&mut t, farm, ghost, 1.0);
        assert_eq!(
            Ledger::open(&t, &w).unwrap_err(),
            CoreError::UnknownGood(ghost)
        );
        assert_eq!(l.close(&t, &w).unwrap_err(), CoreError::UnknownGood(ghost));
    }

    #[test]
    fn nan_drift_is_a_breach() {
        let (w, mut s) = testkit::load();
        let (farm, grain) = (holder(&w, "farm"), good(&w, "grain"));
        let err = tick(&mut s, &w, |s, _| {
            write_direct(s, farm, grain, f64::NAN);
            Ok(())
        })
        .unwrap_err();
        assert!(breach(err).drift.is_nan());
    }

    #[test]
    fn zero_stock_zero_flow_requires_exact_zero() {
        // Nobody holds labour and nothing moves it, so its tolerance is exactly 0: July's
        // absolute floor is gone (A12), and even the smallest subnormal is a breach.
        let (w, mut s) = testkit::load();
        let (workers, labour) = (holder(&w, "workers"), good(&w, "labour"));
        let err = tick(&mut s.clone(), &w, |s, _| {
            write_direct(s, workers, labour, f64::from_bits(1));
            Ok(())
        })
        .unwrap_err();
        let b = breach(err);
        assert_eq!((b.good, b.tol), (labour, 0.0));
        let audit = tick(&mut s, &w, |_, _| Ok(())).unwrap();
        assert_eq!(audit.max_margin, 0.0);
    }

    #[test]
    fn margin_is_zero_when_drift_is_zero() {
        let (w, mut s) = testkit::load();
        let (farm, mill, grain, bread) = (
            holder(&w, "farm"),
            holder(&w, "mill"),
            good(&w, "grain"),
            good(&w, "bread"),
        );
        let audit = tick(&mut s, &w, |s, l| {
            crate::apply(
                s,
                &w,
                Phase::Decisions,
                &[transfer(farm, mill, grain, 2.5)],
                l,
            )?;
            let make = StateDelta::Mint {
                to: mill,
                good: bread,
                qty: 1.5,
                prov: Provenance::Production,
            };
            crate::apply(s, &w, Phase::Production, &[make], l)?;
            Ok(())
        })
        .unwrap();
        // Labour has no stock and no flow, 0/0 in principle; the margin is 0 by definition,
        // never NaN.
        assert_eq!(audit.max_margin.to_bits(), 0);
        assert_eq!(audit.max_drift.to_bits(), 0);
    }

    #[test]
    fn direct_state_write_is_caught() {
        let (w, mut s) = testkit::load();
        let (mill, coin, labour) = (holder(&w, "mill"), good(&w, "coin"), good(&w, "labour"));
        // Into an existing lot.
        let err = tick(&mut s.clone(), &w, |s, _| {
            write_direct(s, mill, coin, 1.0);
            Ok(())
        })
        .unwrap_err();
        assert_eq!(breach(err).good, coin);
        // A new lot of a good nobody held.
        let err = tick(&mut s, &w, |s, _| {
            write_direct(s, mill, labour, 0.5);
            Ok(())
        })
        .unwrap_err();
        assert_eq!(breach(err).good, labour);
    }

    #[test]
    fn run_ledger_catches_a_leak_below_each_ticks_tolerance() {
        // The farm holds the fixture's only grain, 16, so a tick's tolerance on grain is
        // 1e-11 · 16 = 1.6e-10 when nothing flows. A leak of 1e-10 a tick passes every tick's
        // check; over two ticks the run has drifted 2e-10, and the run ledger stops it.
        let (w, mut s) = testkit::load();
        let (farm, grain) = (holder(&w, "farm"), good(&w, "grain"));
        let leak = 1e-10;
        let mut run = RunLedger::open(&s, &w).unwrap();
        assert_eq!(run.since(), 0);
        let mut passed = Vec::new();
        let err = loop {
            // Ten ticks of the leak are 6.25 times the tolerance; a run ledger that has not
            // stopped by then never will.
            assert!(
                passed.len() < 10,
                "the run ledger let the leak run: {passed:?}"
            );
            let mut l = Ledger::open(&s, &w).unwrap();
            write_direct(&mut s, farm, grain, leak);
            crate::apply(
                &mut s,
                &w,
                Phase::Prices,
                &[StateDelta::AdvanceTick],
                &mut l,
            )
            .unwrap();
            match run.close_tick(l, &s, &w) {
                Ok((audit, r)) => passed.push((audit.max_margin, r.max_margin)),
                Err(e) => break e,
            }
        };
        assert_eq!(passed.len(), 1);
        let (tick_margin, run_margin) = passed[0];
        assert!(tick_margin > 0.5 && tick_margin <= 1.0, "{tick_margin}");
        assert_eq!(run_margin.to_bits(), tick_margin.to_bits());
        // The breach is the run's (since 0), found at tick 1, whose own check passed.
        let b = breach(err.clone());
        assert_eq!((b.good, b.since, b.tick), (grain, 0, 1));
        assert!(b.drift > b.tol && b.drift <= 2.0 * leak * (1.0 + LIMIT_BAR));
        assert!(err
            .to_string()
            .contains("run conservation failure over ticks 0 to 1"));
        // A tick's ledger closed alone passes the same leak.
        let mut l = Ledger::open(&s, &w).unwrap();
        write_direct(&mut s, farm, grain, leak);
        crate::apply(
            &mut s,
            &w,
            Phase::Prices,
            &[StateDelta::AdvanceTick],
            &mut l,
        )
        .unwrap();
        assert!(l.close(&s, &w).is_ok());

        // A clean run folds its ticks: the run's lines are the sums of the ticks' lines, and
        // the run's drift stays inside its tolerance. A ledger out of turn is refused.
        let (w, mut s) = testkit::load();
        let (mill, bread) = (holder(&w, "mill"), good(&w, "bread"));
        let mut run = RunLedger::open(&s, &w).unwrap();
        let mut made = 0.0;
        for t in 0..5 {
            let mut l = Ledger::open(&s, &w).unwrap();
            let make = StateDelta::Mint {
                to: mill,
                good: bread,
                qty: 0.1,
                prov: Provenance::Production,
            };
            crate::apply(&mut s, &w, Phase::Production, &[make], &mut l).unwrap();
            crate::apply(
                &mut s,
                &w,
                Phase::Prices,
                &[StateDelta::AdvanceTick],
                &mut l,
            )
            .unwrap();
            let (_, r) = run.close_tick(l, &s, &w).unwrap();
            made += 0.1;
            assert_eq!(r.since, 0);
            assert!(r.max_margin <= 1.0, "tick {t}: {}", r.max_margin);
            assert_eq!(r.drift.len(), w.n_goods());
            let line = r
                .lines
                .iter()
                .find(|&&(g, p, _)| g == bread && p == Provenance::Production)
                .map(|&(_, _, q)| q);
            assert_eq!(line, Some(made), "tick {t}");
        }
        let stale = Ledger::open(&testkit::load().1, &w).unwrap();
        assert!(matches!(
            run.close_tick(stale, &s, &w),
            Err(CoreError::Shape(_))
        ));
    }

    #[test]
    fn flow_tolerance_is_pinned_for_a_tick_and_a_run() {
        // O6 (P0.9): the flow term of the tolerance, rel_flow·gross, with gross = Σ moved +
        // Σ |declared|, exactly, for a tick and for a run. Nothing pinned it: counting a moved
        // quantity as q·q + q, or the run's fold as twice the ticks', widened A12's registered
        // tolerance and every test still passed. Here the flow term dominates, and a drift at
        // 0.9 of the tolerance passes while one at 1.1 breaches, with the breach's gross and
        // tolerance exactly what the formula gives.
        let (w, base) = testkit::load();
        let (farm, mill, pensioners, grain) = (
            holder(&w, "farm"),
            holder(&w, "mill"),
            holder(&w, "pensioners"),
            good(&w, "grain"),
        );
        let rel_flow = base.param(w.tol.rel_flow.param).unwrap();
        let rel_stock = base.param(w.tol.rel_stock.param).unwrap();
        assert_eq!((rel_flow, rel_stock), (1e-12, 1e-11));
        // `n` transfers of the farm's 1000 grain, to the mill and back: each moves exactly 1000.
        let shuttle = |s: &mut SimState<NoExt>, l: &mut Ledger, n: usize| {
            for k in 0..n {
                let (from, to) = if k % 2 == 0 {
                    (farm, mill)
                } else {
                    (mill, farm)
                };
                crate::apply(
                    s,
                    &w,
                    Phase::Decisions,
                    &[transfer(from, to, grain, 1000.0)],
                    l,
                )
                .unwrap();
            }
        };
        let advance = |s: &mut SimState<NoExt>, l: &mut Ledger| {
            crate::apply(s, &w, Phase::Prices, &[StateDelta::AdvanceTick], l).unwrap();
        };
        let at = |x: f64, want: f64| (x - want).abs() <= LIMIT_BAR * want;

        // A tick: a mint of 984 makes the farm's 16 into 1000, and 1000 transfers move 1e6.
        // gross = 984 + 1e6 exactly; the stock is 16 at the open and 1000 at the close, so
        // the flow term is 99% of the tolerance.
        let gross = 984.0 + 1e6;
        let tol = rel_flow * gross + rel_stock * 1000.0;
        let tick = |drift: f64| {
            let mut s = base.clone();
            let mut l = Ledger::open(&s, &w).unwrap();
            let mint = StateDelta::Mint {
                to: farm,
                good: grain,
                qty: 984.0,
                prov: Provenance::Production,
            };
            crate::apply(&mut s, &w, Phase::Production, &[mint], &mut l).unwrap();
            shuttle(&mut s, &mut l, 1000);
            write_direct(&mut s, pensioners, grain, drift);
            l.close(&s, &w)
        };
        let audit = tick(0.9 * tol).expect("0.9 of the tolerance passes");
        assert!(at(audit.max_margin, 0.9), "margin {}", audit.max_margin);
        let b = breach(tick(1.1 * tol).unwrap_err());
        assert_eq!((b.good, b.tick, b.since), (grain, 0, 0));
        assert_eq!((b.opening, b.declared, b.gross), (16.0, 984.0, gross));
        assert_eq!(
            b.tol.to_bits(),
            (rel_flow * b.gross + rel_stock * b.opening.abs().max(b.closing.abs())).to_bits()
        );
        assert!(at(b.tol, tol) && at(b.drift, 1.1 * tol), "{b}");

        // A run of two ticks, each moving 1e4 over a stock of 1000, so that flow and stock
        // weigh equally: a tick's tolerance is 2e-8, and the run's after two ticks is
        // rel_flow·2e4 + rel_stock·1000 = 3e-8, less than the ticks' 4e-8 (the stock term does
        // not add up over ticks, the flow term does). A drift of 0.45 of the run's tolerance
        // each tick passes both ticks and the run; 0.55 each passes both ticks and breaches the
        // run at its second tick.
        let run_tol = rel_flow * 2e4 + rel_stock * 1000.0;
        let run = |each: f64| -> Result<Vec<(TickAudit, RunAudit)>, CoreError> {
            let mut s = base.clone();
            testkit::apply_ok(
                &mut s,
                &w,
                Phase::Events,
                &[StateDelta::Mint {
                    to: farm,
                    good: grain,
                    qty: 984.0,
                    prov: Provenance::Event,
                }],
            );
            let mut r = RunLedger::open(&s, &w).unwrap();
            let mut audits = Vec::new();
            for _ in 0..2 {
                let mut l = Ledger::open(&s, &w).unwrap();
                shuttle(&mut s, &mut l, 10);
                write_direct(&mut s, pensioners, grain, each);
                advance(&mut s, &mut l);
                audits.push(r.close_tick(l, &s, &w)?);
            }
            Ok(audits)
        };
        let audits = run(0.45 * run_tol).expect("0.9 of the run's tolerance passes");
        for (t, (a, _)) in audits.iter().enumerate() {
            // A tick's drift is 0.45·3e-8 of its tolerance 2e-8.
            assert!(at(a.max_margin, 0.675), "tick {t}: {}", a.max_margin);
        }
        assert!(
            at(audits[1].1.max_margin, 0.9),
            "{}",
            audits[1].1.max_margin
        );
        let b = breach(run(0.55 * run_tol).unwrap_err());
        assert_eq!(
            (b.good, b.since, b.tick),
            (grain, 0, 1),
            "the run's breach, not a tick's"
        );
        assert_eq!((b.opening, b.declared, b.gross), (1000.0, 0.0, 2e4));
        assert_eq!(
            b.tol.to_bits(),
            (rel_flow * b.gross + rel_stock * b.opening.abs().max(b.closing.abs())).to_bits()
        );
        assert!(at(b.tol, run_tol) && at(b.drift, 1.1 * run_tol), "{b}");
    }

    #[test]
    fn burn_shortfall_stops_with_a_ledger_line() {
        // July declared only what a burn actually removed, so a 1e9 shortfall ran on (N3).
        let (w, mut s) = testkit::load();
        let (workers, coin) = (holder(&w, "workers"), good(&w, "coin"));
        let before = s.clone();
        let mut l = Ledger::open(&s, &w).unwrap();
        let burn = StateDelta::Burn {
            from: workers,
            good: coin,
            amount: Amount::Qty(1e9),
            prov: Provenance::Consumption,
        };
        let err = crate::apply(&mut s, &w, Phase::Production, &[burn], &mut l).unwrap_err();
        let CoreError::Shortfall(line) = &err else {
            panic!("expected a shortfall, got {err}");
        };
        assert_eq!((line.requested, line.held), (1e9, 20.0));
        let text = err.to_string();
        assert!(
            text.contains("shortfall at tick 0") && text.contains("Consumption"),
            "{text}"
        );
        assert_eq!(s, before, "no partial burn");
        assert!(l.lines().is_empty());
        assert!(
            matches!(l.close(&s, &w), Err(CoreError::Shortfall(_))),
            "the tick stops"
        );
        // `All` means whatever is there, and never falls short.
        let audit = tick(&mut s, &w, |s, l| {
            let all = StateDelta::Burn {
                from: workers,
                good: coin,
                amount: Amount::All,
                prov: Provenance::Consumption,
            };
            let moved = crate::apply(s, &w, Phase::Production, &[all], l)?;
            assert_eq!(moved, vec![20.0]);
            Ok(())
        })
        .unwrap();
        assert_eq!(audit.lines, vec![(coin, Provenance::Consumption, -20.0)]);
    }
}
