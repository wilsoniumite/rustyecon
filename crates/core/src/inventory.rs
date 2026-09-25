//! Inventories of lots with lives (docs/ENGINE.md §2.2; REVIEW §2.2 defect 5; ADDENDUM N9).
//!
//! Salvaged from `v2p3: types/inventory.rs`. Lots keep their lives through checkpoints (defect 5,
//! fixed in July and kept), and lots of equal life now coalesce, so a good holds at most
//! `max_life + 1` lots and an indefinite good exactly one (N9: July's lots never merged, and run
//! cost grew with the horizon). `take` is atomic: it takes all it is asked for or nothing, so a
//! shortfall is never a silent clamp (defect 9).
//!
//! Lots are `f64`, so splitting a lot (`old − rest`) or merging into one (`a + b`) rounds, and
//! the rounding creates or destroys up to half an ulp of the larger operand: 1 taken from a lot
//! of 1e17 (ulp 16) leaves it at 1e17, and 6 merged into it vanish. `take` and `put` measure that
//! exactly, by TwoSum, and return it as `rounding`, so `apply` can declare it to the ledger with
//! the provenance `Rounding` (docs/ENGINE.md §2.2, §2.4). Nothing appears or vanishes unrecorded.

use crate::error::CoreError;
use crate::ids::GoodId;
use crate::num::{is_clean, two_sum};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A quantity of one good with one remaining life.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lot {
    /// The quantity: finite, with a clear sign bit, never zero inside an inventory.
    pub qty: f64,
    /// Ticks of life left: `None` is indefinite, and `Some(0)` dies at the next ageing (5a).
    pub life: Option<u32>,
}

/// How much a transfer or burn takes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Amount {
    /// Exactly this quantity, or nothing if the holder has less.
    Qty(f64),
    /// Whatever is there.
    All,
}

/// What a take moved out of an inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct Taken {
    /// The lots taken, with their lives, soonest-expiring first.
    pub lots: Vec<Lot>,
    /// What the split of the last lot created, exactly: the source keeps `fl(old − rest)` while
    /// `rest` leaves it, so `(fl(old − rest) + rest) − old` units appeared (negative: vanished).
    /// Zero when the take split nothing or the subtraction was exact; never `-0.0`.
    pub rounding: f64,
}

/// The quantity a rounded sum created: `fl(a + b) − (a + b)`, from TwoSum's exact error.
fn created(err: f64) -> f64 {
    if err == 0.0 {
        0.0
    } else {
        -err
    }
}

/// A take that asked for more than was held. Nothing moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shortfall {
    /// What was asked for.
    pub requested: f64,
    /// What was held.
    pub held: f64,
}

/// Why a take failed. Nothing moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TakeError {
    /// More was asked for than was held.
    Shortfall(Shortfall),
    /// The requested quantity was non-finite or had its sign bit set.
    Invalid(f64),
}

impl fmt::Display for TakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TakeError::Shortfall(s) => {
                write!(f, "asked for {:e}, held {:e}", s.requested, s.held)
            }
            TakeError::Invalid(q) => write!(f, "invalid quantity {q:e}"),
        }
    }
}

/// The order lots keep within a good: soonest-expiring first, indefinite last.
fn life_order(life: Option<u32>) -> (bool, u32) {
    match life {
        Some(n) => (false, n),
        None => (true, 0),
    }
}

/// A holder's goods: goods ascending, and within each good its lots by life, indefinite last, at
/// most one lot per life and none empty. Every value is finite with a clear sign bit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Inventory(Vec<(GoodId, Vec<Lot>)>);

impl Inventory {
    /// An empty inventory.
    pub fn new() -> Inventory {
        Inventory(Vec::new())
    }

    fn find(&self, g: GoodId) -> Result<usize, usize> {
        self.0.binary_search_by_key(&g, |&(good, _)| good)
    }

    /// The quantity of `g` held: the sum of its lots, a left fold in lot order.
    pub fn get(&self, g: GoodId) -> f64 {
        match self.find(g) {
            Ok(i) => self.0[i].1.iter().fold(0.0, |acc, lot| acc + lot.qty),
            Err(_) => 0.0,
        }
    }

    /// The lots of `g`, soonest-expiring first.
    pub fn lots(&self, g: GoodId) -> &[Lot] {
        match self.find(g) {
            Ok(i) => &self.0[i].1,
            Err(_) => &[],
        }
    }

    /// Every good held, ascending, with its quantity as [`Inventory::get`] computes it.
    pub fn goods(&self) -> impl Iterator<Item = (GoodId, f64)> + '_ {
        self.0
            .iter()
            .map(|(g, lots)| (*g, lots.iter().fold(0.0, |acc, lot| acc + lot.qty)))
    }

    /// Whether nothing is held.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The number of lots over every good.
    pub fn lot_count(&self) -> usize {
        self.0.iter().map(|(_, lots)| lots.len()).sum()
    }

    /// Add lots of `g`, each merging into the lot of equal life if there is one. A lot of
    /// exactly `+0.0` is dropped. Every quantity is checked first, so a rejected call changes
    /// nothing. Returns what the merges created by rounding (negative: destroyed), exactly per
    /// merge: `fl(a + b) − (a + b)`; 0 when every merge was exact.
    pub fn put(&mut self, g: GoodId, lots: Vec<Lot>) -> Result<f64, CoreError> {
        if let Some(bad) = lots.iter().find(|lot| !is_clean(lot.qty)) {
            return Err(CoreError::BadValue {
                what: "lot quantity",
                value: bad.qty,
            });
        }
        let mut rounding = 0.0;
        for lot in lots {
            rounding += self.merge(g, lot);
        }
        Ok(rounding)
    }

    /// Add `qty` of `g` with one life, as [`Inventory::put`] does.
    pub fn put_qty(&mut self, g: GoodId, qty: f64, life: Option<u32>) -> Result<f64, CoreError> {
        self.put(g, vec![Lot { qty, life }])
    }

    /// Merge one checked lot, and return what the merge created by rounding.
    fn merge(&mut self, g: GoodId, lot: Lot) -> f64 {
        if lot.qty == 0.0 {
            return 0.0;
        }
        let i = match self.find(g) {
            Ok(i) => i,
            Err(i) => {
                self.0.insert(i, (g, Vec::new()));
                i
            }
        };
        let lots = &mut self.0[i].1;
        match lots.binary_search_by_key(&life_order(lot.life), |l| life_order(l.life)) {
            Ok(j) => {
                let (sum, err) = two_sum(lots[j].qty, lot.qty);
                lots[j].qty = sum;
                created(err)
            }
            Err(j) => {
                lots.insert(j, lot);
                0.0
            }
        }
    }

    /// Take from `g`, soonest-expiring lots first, and return the lots taken with their lives.
    ///
    /// `Qty(q)` with `q > get(g)` is [`TakeError::Shortfall`] and changes nothing. `q == get(g)`
    /// and `All` take every lot. Otherwise whole lots go first and the last one is split, its
    /// remainder being `fl(lot.qty − rest)`; for a single lot (every currency) that is
    /// `fl(lot.qty − q)`, the subtraction admission mirrors. If rounding uses up the lots while a
    /// remainder is left, every lot is taken, so a request with `q <= get(g)` never falls short.
    ///
    /// The lots taken hold exactly `rest` of the split lot, so a split that rounds creates or
    /// destroys units; [`Taken::rounding`] says exactly how many. The whole lots taken are
    /// moved as they are, so their sum can differ from `q` by the rounding of `rest`.
    pub fn take(&mut self, g: GoodId, a: Amount) -> Result<Taken, TakeError> {
        let q = match a {
            Amount::All => return Ok(self.take_all(g)),
            Amount::Qty(q) => q,
        };
        if !is_clean(q) {
            return Err(TakeError::Invalid(q));
        }
        let held = self.get(g);
        if q > held {
            return Err(TakeError::Shortfall(Shortfall { requested: q, held }));
        }
        if q == 0.0 {
            return Ok(Taken {
                lots: Vec::new(),
                rounding: 0.0,
            });
        }
        if q == held {
            return Ok(self.take_all(g));
        }
        let Ok(i) = self.find(g) else {
            // Unreachable: 0 < q <= held means the good is present.
            return Ok(Taken {
                lots: Vec::new(),
                rounding: 0.0,
            });
        };
        let lots = &mut self.0[i].1;
        let mut rest = q;
        let mut whole = 0;
        let mut split = None;
        let mut rounding = 0.0;
        for lot in lots.iter_mut() {
            if lot.qty <= rest {
                rest -= lot.qty;
                whole += 1;
                if rest == 0.0 {
                    break;
                }
            } else {
                // lot.qty > rest, so the remainder is positive however it rounds.
                let (remainder, err) = two_sum(lot.qty, -rest);
                lot.qty = remainder;
                rounding = created(err);
                split = Some(Lot {
                    qty: rest,
                    life: lot.life,
                });
                break;
            }
        }
        let mut taken: Vec<Lot> = lots.drain(..whole).collect();
        taken.extend(split);
        if lots.is_empty() {
            self.0.remove(i);
        }
        Ok(Taken {
            lots: taken,
            rounding,
        })
    }

    fn take_all(&mut self, g: GoodId) -> Taken {
        let lots = match self.find(g) {
            Ok(i) => self.0.remove(i).1,
            Err(_) => Vec::new(),
        };
        Taken {
            lots,
            rounding: 0.0,
        }
    }

    /// Core ageing, phase 5a: drop every lot at `Some(0)`, then decrement the other finite
    /// lives. Returns what spoiled, per good ascending, so it can be posted to the ledger as
    /// `Spoilage`. A lot minted with life `L` in tick t can therefore sell in ticks t+1 … t+L.
    pub fn age(&mut self) -> Vec<(GoodId, f64)> {
        let mut spoiled = Vec::new();
        for (g, lots) in &mut self.0 {
            // Lots are coalesced and sorted, so a dying lot can only be the first.
            if lots.first().is_some_and(|lot| lot.life == Some(0)) {
                spoiled.push((*g, lots.remove(0).qty));
            }
            for lot in lots.iter_mut() {
                if let Some(n) = &mut lot.life {
                    *n = n.saturating_sub(1);
                }
            }
        }
        self.0.retain(|(_, lots)| !lots.is_empty());
        spoiled
    }

    /// Test access: the raw lots, for writes that bypass `apply`.
    #[cfg(test)]
    pub(crate) fn raw_mut(&mut self) -> &mut Vec<(GoodId, Vec<Lot>)> {
        &mut self.0
    }
}

/// The serialised form: `(good, [(qty, life), …])` per good.
type Repr = Vec<(GoodId, Vec<(f64, Option<u32>)>)>;

/// Serialised as `Vec<(GoodId, Vec<(qty, life)>)>`, July's checkpoint form, so lot lives survive
/// a save and a resume (defect 5).
impl Serialize for Inventory {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let repr: Repr = self
            .0
            .iter()
            .map(|(g, lots)| (*g, lots.iter().map(|l| (l.qty, l.life)).collect()))
            .collect();
        repr.serialize(s)
    }
}

/// Loading re-sorts and re-coalesces, drops `+0.0` lots, and rejects any quantity that is not
/// finite or whose sign bit is set (NaN and `-0.0` included).
impl<'de> Deserialize<'de> for Inventory {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let repr = Repr::deserialize(d)?;
        let mut inv = Inventory::new();
        for (g, lots) in repr {
            let lots: Vec<Lot> = lots
                .into_iter()
                .map(|(qty, life)| Lot { qty, life })
                .collect();
            inv.put(g, lots).map_err(serde::de::Error::custom)?;
        }
        Ok(inv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(n: u32) -> GoodId {
        GoodId(n)
    }

    fn inv_with(lots: &[(u32, f64, Option<u32>)]) -> Inventory {
        let mut inv = Inventory::new();
        for &(good, qty, life) in lots {
            inv.put_qty(g(good), qty, life).unwrap();
        }
        inv
    }

    #[test]
    fn add_and_get() {
        let inv = inv_with(&[(0, 10.0, None)]);
        assert_eq!(inv.get(g(0)), 10.0);
        assert_eq!(inv.get(g(1)), 0.0);
    }

    #[test]
    fn take_partial() {
        let mut inv = inv_with(&[(0, 10.0, None)]);
        let taken = inv.take(g(0), Amount::Qty(4.0)).unwrap();
        assert_eq!(taken.rounding, 0.0);
        assert_eq!(
            taken.lots,
            vec![Lot {
                qty: 4.0,
                life: None
            }]
        );
        assert_eq!(inv.get(g(0)), 6.0);
    }

    #[test]
    fn take_more_than_held_is_a_shortfall_and_moves_nothing() {
        let mut inv = inv_with(&[(0, 3.0, Some(2)), (0, 1.0, None)]);
        let before = inv.clone();
        let err = inv.take(g(0), Amount::Qty(10.0)).unwrap_err();
        assert_eq!(
            err,
            TakeError::Shortfall(Shortfall {
                requested: 10.0,
                held: 4.0
            })
        );
        assert_eq!(inv, before, "a failed take moves nothing");
        // July's `remove` clamped: it took the 3 and reported 3.
        assert_eq!(inv.get(g(0)), 4.0);
    }

    #[test]
    fn stays_sorted() {
        let inv = inv_with(&[(5, 1.0, None), (2, 2.0, None), (8, 3.0, None)]);
        let ids: Vec<_> = inv.goods().map(|(good, _)| good).collect();
        assert_eq!(ids, vec![g(2), g(5), g(8)]);
    }

    #[test]
    fn age_removes_expired_and_decrements() {
        let mut inv = inv_with(&[(0, 5.0, Some(0)), (0, 3.0, Some(1)), (0, 2.0, None)]);
        assert_eq!(inv.age(), vec![(g(0), 5.0)]);
        assert_eq!(inv.get(g(0)), 5.0); // 3.0 (now life 0) + 2.0 (indefinite)
        assert_eq!(
            inv.lots(g(0))[0],
            Lot {
                qty: 3.0,
                life: Some(0)
            }
        );
        assert_eq!(inv.age(), vec![(g(0), 3.0)]);
        assert_eq!(inv.get(g(0)), 2.0);
        assert_eq!(inv.age(), vec![]);
        assert_eq!(inv.get(g(0)), 2.0);
    }

    #[test]
    fn serde_preserves_lot_lives() {
        // A save and resume must not turn perishable lots indefinite (defect 5).
        let inv = inv_with(&[(1, 4.0, Some(3)), (1, 6.0, None), (0, 2.0, Some(1))]);
        let ron_text = ron::ser::to_string(&inv).unwrap();
        let from_ron: Inventory = ron::from_str(&ron_text).unwrap();
        let bin = bincode::serialize(&inv).unwrap();
        let from_bin: Inventory = bincode::deserialize(&bin).unwrap();
        for mut round in [from_ron, from_bin] {
            assert_eq!(round, inv);
            round.age(); // Some(3) -> Some(2), Some(1) -> Some(0)
            round.age(); // Some(0) on good 0 dies
            assert_eq!(
                round.get(g(0)),
                0.0,
                "the finite life survived the round trip"
            );
            assert_eq!(round.get(g(1)), 10.0);
            assert_eq!(round.lots(g(1))[0].life, Some(1));
        }
    }

    #[test]
    fn take_fefo() {
        let mut inv = inv_with(&[(0, 6.0, None), (0, 4.0, Some(1))]);
        let taken = inv.take(g(0), Amount::Qty(5.0)).unwrap();
        assert_eq!(
            taken.lots,
            vec![
                Lot {
                    qty: 4.0,
                    life: Some(1)
                },
                Lot {
                    qty: 1.0,
                    life: None
                }
            ],
            "the soonest-expiring lot goes first"
        );
        assert_eq!(
            inv.lots(g(0)),
            &[Lot {
                qty: 5.0,
                life: None
            }]
        );
    }

    #[test]
    fn lots_coalesce_by_life() {
        let mut inv = inv_with(&[(0, 2.0, Some(3)), (0, 3.0, Some(3)), (0, 1.0, None)]);
        inv.put_qty(g(0), 4.0, None).unwrap();
        assert_eq!(
            inv.lots(g(0)),
            &[
                Lot {
                    qty: 5.0,
                    life: Some(3)
                },
                Lot {
                    qty: 5.0,
                    life: None
                }
            ]
        );
        // A perishable good with life L minted every tick and aged every tick holds at most
        // L + 1 lots however long it runs; July's grew by one lot per tick (N9).
        let life = 3;
        let mut inv = Inventory::new();
        for _ in 0..1000 {
            inv.put_qty(g(1), 1.0, Some(life)).unwrap();
            inv.put_qty(g(2), 1.0, None).unwrap();
            assert!(inv.lots(g(1)).len() <= life as usize + 1);
            assert_eq!(inv.lots(g(2)).len(), 1, "an indefinite good holds one lot");
            inv.age();
        }
        assert_eq!(inv.lots(g(1)).len(), life as usize);
    }

    #[test]
    fn take_all_of_multi_lot_holding_never_falls_short() {
        let fresh = || inv_with(&[(0, 0.1, Some(1)), (0, 0.2, None)]);
        let held = fresh().get(g(0));
        assert_eq!(held, 0.1 + 0.2);
        for a in [Amount::Qty(held), Amount::All] {
            let mut inv = fresh();
            let taken = inv.take(g(0), a).unwrap();
            assert_eq!(taken.lots.len(), 2);
            assert!(inv.is_empty());
        }
        // Just under the held total: the sequential subtraction must not fall short either.
        for q in [0.3, held.next_down(), 0.1, 0.2] {
            let mut inv = fresh();
            let taken = inv.take(g(0), Amount::Qty(q)).unwrap();
            assert!(!taken.lots.is_empty(), "{q}");
        }
    }

    #[test]
    fn nonfinite_negative_or_negative_zero_qty_is_rejected() {
        let mut inv = inv_with(&[(0, 1.0, None)]);
        let before = inv.clone();
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, -0.0] {
            assert!(inv.put_qty(g(0), bad, None).is_err(), "put {bad}");
            assert!(
                inv.put(
                    g(0),
                    vec![
                        Lot {
                            qty: 1.0,
                            life: None
                        },
                        Lot {
                            qty: bad,
                            life: None
                        }
                    ]
                )
                .is_err(),
                "a bad lot rejects the whole put"
            );
            assert!(matches!(
                inv.take(g(0), Amount::Qty(bad)),
                Err(TakeError::Invalid(_))
            ));
            assert_eq!(inv, before, "{bad}");
        }
        // +0.0 is a legal quantity and leaves no lot behind.
        inv.put_qty(g(3), 0.0, None).unwrap();
        assert_eq!(inv.lots(g(3)), &[]);
        // Loading rejects the same values.
        for text in [
            "[((0), [(NaN, None)])]",
            "[((0), [(-0.0, None)])]",
            "[((0), [(-2.0, Some(1))])]",
            "[((0), [(inf, None)])]",
        ] {
            assert!(ron::from_str::<Inventory>(text).is_err(), "{text}");
        }
        // Loading re-sorts, re-coalesces and drops +0.0 lots.
        let loaded: Inventory = ron::from_str(
            "[((2), [(1.0, None)]), ((0), [(0.0, None), (1.0, Some(2)), (2.0, Some(2))])]",
        )
        .unwrap();
        assert_eq!(loaded, inv_with(&[(0, 3.0, Some(2)), (2, 1.0, None)]));
    }

    #[test]
    fn split_and_merge_rounding_is_measured() {
        // Lots are f64. One taken from a lot of 1e17 (ulp 16) leaves it at 1e17: the taker has
        // 1 and the source lost nothing, so one unit appeared, and the take says so.
        let mut inv = inv_with(&[(0, 1e17, None)]);
        let t = inv.take(g(0), Amount::Qty(1.0)).unwrap();
        assert_eq!(
            t.lots,
            vec![Lot {
                qty: 1.0,
                life: None
            }]
        );
        assert_eq!(t.rounding, 1.0);
        assert_eq!(inv.get(g(0)), 1e17);
        // Six merged into it vanish; nine round up to 16, creating seven.
        assert_eq!(inv.put_qty(g(0), 6.0, None).unwrap(), -6.0);
        assert_eq!(inv.get(g(0)), 1e17);
        assert_eq!(inv.put_qty(g(0), 9.0, None).unwrap(), 7.0);
        assert_eq!(inv.get(g(0)), 1e17 + 16.0);
        // Exact splits and merges, and takes of whole lots, create nothing; never -0.0.
        let mut inv = inv_with(&[(0, 16.0, None)]);
        assert_eq!(
            inv.take(g(0), Amount::Qty(6.25))
                .unwrap()
                .rounding
                .to_bits(),
            0
        );
        assert_eq!(inv.put_qty(g(0), 0.25, None).unwrap().to_bits(), 0);
        assert_eq!(inv.take(g(0), Amount::All).unwrap().rounding.to_bits(), 0);
        // In general, before + rounding = after + taken, exactly: checked in integers on
        // integer-valued lots up to 2^62, whose ulps reach 1024.
        let mut state = 0x2026_0927_u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state >> 2
        };
        let (mut rounded, int) = (0, |x: f64| x as i128);
        for _ in 0..10_000 {
            let old = (next() >> (next() % 60)).max(2) as f64;
            let rest = ((next() % (old as u64)).max(1)) as f64;
            let mut inv = inv_with(&[(0, old, None)]);
            let t = inv.take(g(0), Amount::Qty(rest)).unwrap();
            let left = inv.get(g(0));
            assert_eq!(
                int(old) + int(t.rounding),
                int(left) + int(t.lots[0].qty),
                "{old} - {rest}"
            );
            let back = inv.put(g(0), t.lots).unwrap();
            assert_eq!(
                int(left) + int(rest) + int(back),
                int(inv.get(g(0))),
                "{left} + {rest}"
            );
            if t.rounding != 0.0 || back != 0.0 {
                rounded += 1;
            }
        }
        assert!(rounded > 100, "the sample rounds: {rounded}");
    }

    #[test]
    fn instant_lot_dies_at_5a() {
        // An Instant good is minted with life Some(0): it trades in phases 2 and 3 of its tick
        // and dies at the first ageing.
        let mut inv = inv_with(&[(0, 8.0, Some(0)), (1, 1.0, None)]);
        assert_eq!(inv.get(g(0)), 8.0);
        assert_eq!(inv.age(), vec![(g(0), 8.0)]);
        assert_eq!(inv.get(g(0)), 0.0);
        assert_eq!(inv.lot_count(), 1);
    }
}
