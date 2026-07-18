use crate::types::ids::GoodId;
use serde::{Deserialize, Serialize};

/// A single lot of goods with an optional shelf-life counter.
#[derive(Debug, Clone)]
pub struct Lot {
    pub qty: f64,
    /// Remaining ticks this lot survives. `None` = indefinite.
    pub life: Option<u32>,
}

/// Sparse inventory: sorted Vec of (GoodId, lots), no empty entries stored.
///
/// Serializes/deserializes as a flat `Vec<(GoodId, f64)>` (summing lots) for
/// compatibility with existing scenario and checkpoint files. Lot details are
/// tracked in-memory only and are not persisted to disk.
#[derive(Debug, Clone, Default)]
pub struct Inventory(Vec<(GoodId, Vec<Lot>)>);

impl Inventory {
    pub fn get(&self, good: GoodId) -> f64 {
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => self.0[i].1.iter().map(|l| l.qty).sum(),
            Err(_) => 0.0,
        }
    }

    /// Add a new lot of `qty` units with the given shelf life.
    pub fn add(&mut self, good: GoodId, qty: f64, life: Option<u32>) {
        assert!(qty >= 0.0, "cannot add negative quantity");
        if qty == 0.0 { return; }
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => self.0[i].1.push(Lot { qty, life }),
            Err(i) => self.0.insert(i, (good, vec![Lot { qty, life }])),
        }
    }

    /// Removes up to `qty` of `good` FIFO (oldest lots first).
    /// Returns how much was actually removed.
    pub fn remove(&mut self, good: GoodId, qty: f64) -> f64 {
        assert!(qty >= 0.0, "cannot remove negative quantity");
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => {
                let mut remaining = qty;
                for lot in self.0[i].1.iter_mut() {
                    if remaining <= 0.0 { break; }
                    let take = remaining.min(lot.qty);
                    lot.qty -= take;
                    remaining -= take;
                }
                self.0[i].1.retain(|l| l.qty > 0.0);
                if self.0[i].1.is_empty() { self.0.remove(i); }
                qty - remaining
            }
            Err(_) => 0.0,
        }
    }

    /// True if this good has any quantity.
    pub fn contains(&self, good: GoodId) -> bool {
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => self.0[i].1.iter().any(|l| l.qty > 0.0),
            Err(_) => false,
        }
    }

    /// Returns all goods with their total quantity (summed across lots).
    pub fn goods(&self) -> Vec<(GoodId, f64)> {
        self.0.iter()
            .filter_map(|(g, lots)| {
                let total: f64 = lots.iter().map(|l| l.qty).sum();
                if total > 0.0 { Some((*g, total)) } else { None }
            })
            .collect()
    }

    pub fn total_goods(&self) -> usize {
        self.0.iter().filter(|(_, lots)| lots.iter().any(|l| l.qty > 0.0)).count()
    }

    /// Advance shelf-life counters by one tick.
    ///
    /// Step 1: Remove lots with `life == Some(0)` — they expired at the end of last tick.
    /// Step 2: Decrement remaining `Some(n)` lives by 1.
    ///
    /// A lot produced with `life = Some(1)` survives the tick it is created, is available
    /// for sale the following tick, and is removed at the next call to `spoil_lots`.
    pub fn spoil_lots(&mut self) {
        for (_, lots) in &mut self.0 {
            lots.retain(|l| l.life != Some(0));
            for lot in lots.iter_mut() {
                if let Some(n) = &mut lot.life {
                    *n = n.saturating_sub(1);
                }
            }
        }
        self.0.retain(|(_, lots)| !lots.is_empty());
    }
}

impl Serialize for Inventory {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.goods().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Inventory {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let flat = Vec::<(GoodId, f64)>::deserialize(deserializer)?;
        let inner = flat.into_iter()
            .filter(|&(_, qty)| qty > 0.0)
            .map(|(g, qty)| (g, vec![Lot { qty, life: None }]))
            .collect();
        Ok(Inventory(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ids::GoodId;

    #[test]
    fn add_and_get() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 10.0, None);
        assert_eq!(inv.get(g), 10.0);
    }

    #[test]
    fn remove_partial() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 10.0, None);
        let removed = inv.remove(g, 4.0);
        assert_eq!(removed, 4.0);
        assert_eq!(inv.get(g), 6.0);
    }

    #[test]
    fn remove_more_than_held() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 3.0, None);
        let removed = inv.remove(g, 10.0);
        assert_eq!(removed, 3.0);
        assert_eq!(inv.get(g), 0.0);
        assert!(!inv.contains(g));
    }

    #[test]
    fn stays_sorted() {
        let mut inv = Inventory::default();
        inv.add(GoodId(5), 1.0, None);
        inv.add(GoodId(2), 2.0, None);
        inv.add(GoodId(8), 3.0, None);
        let ids: Vec<_> = inv.goods().iter().map(|&(g, _)| g).collect();
        assert_eq!(ids, vec![GoodId(2), GoodId(5), GoodId(8)]);
    }

    #[test]
    fn spoil_lots_removes_expired_and_decrements() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 5.0, Some(0)); // already expired: removed on first call
        inv.add(g, 3.0, Some(1)); // survives this call, expires next
        inv.add(g, 2.0, None);    // indefinite: never spoils

        inv.spoil_lots();
        assert_eq!(inv.get(g), 5.0); // 3.0 (now life=0) + 2.0 (None)

        inv.spoil_lots();
        assert_eq!(inv.get(g), 2.0); // 3.0 lot expired; only indefinite remains
    }

    #[test]
    fn remove_fifo() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 4.0, Some(1)); // oldest lot (will expire sooner)
        inv.add(g, 6.0, None);    // newer lot

        // Removing 5.0 should exhaust the oldest lot first
        let removed = inv.remove(g, 5.0);
        assert_eq!(removed, 5.0);
        assert_eq!(inv.get(g), 5.0); // 1.0 remains from the None lot
    }
}
