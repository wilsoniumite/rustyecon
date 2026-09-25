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

use crate::delta::{Phase, Provenance};
use crate::error::CoreError;
use crate::ext::Ext;
use crate::ids::{GoodId, Holder};
use crate::state::SimState;
use crate::units::Dimensionless;
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

/// One good's failure to conserve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Breach {
    /// The tick the ledger opened at.
    pub tick: u64,
    /// The good.
    pub good: GoodId,
    /// Its measured total at the open.
    pub opening: f64,
    /// Its measured total at the close.
    pub closing: f64,
    /// The signed sum of its mints and burns.
    pub declared: f64,
    /// The quantity of it moved, minted, burned or spoiled.
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
        write!(
            f,
            "conservation failure at tick {}: {} drifted {:+e} (tolerance {:e}); measured \
             change {:+e} from {:e} to {:e}, declared {:+e}, gross flow {:e}; lines {:?}",
            self.tick,
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
        })
    }

    /// Close the tick: fail on a recorded shortfall or a remaining escrow, then measure every
    /// good again and check it against what was declared, with the tolerances read from the
    /// current params.
    pub fn close<E: Ext>(self, s: &SimState<E>, w: &World<E>) -> Result<TickAudit, CoreError> {
        if let Some(line) = self.shortfall {
            return Err(CoreError::Shortfall(line));
        }
        no_escrow(s)?;
        let closing = walk(s, w)?;
        if closing.len() != self.opening.len() {
            return Err(CoreError::Shape(format!(
                "a ledger over {} goods closed on a world of {}",
                self.opening.len(),
                closing.len()
            )));
        }
        let params = s.params(&w.registry);
        let rel_flow = params.get::<Dimensionless>(w.tol.rel_flow)?.0;
        let rel_stock = params.get::<Dimensionless>(w.tol.rel_stock)?.0;
        let mut max_margin = 0.0_f64;
        let mut max_drift = 0.0_f64;
        for (i, (&open, &close)) in self.opening.iter().zip(&closing).enumerate() {
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
        }
        Ok(TickAudit {
            lines: self.lines.iter().map(|(&(g, p), &q)| (g, p, q)).collect(),
            max_margin,
            max_drift,
        })
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
    }

    /// Record the shortfall that stops the tick.
    pub(crate) fn record_shortfall(&mut self, line: ShortfallLine) {
        self.shortfall = Some(line);
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
