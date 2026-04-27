use crate::types::ids::GoodId;
use serde::{Deserialize, Serialize};

/// Sparse inventory: sorted Vec of (GoodId, quantity), no zero entries stored.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory(Vec<(GoodId, f64)>);

impl Inventory {
    pub fn get(&self, good: GoodId) -> f64 {
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => self.0[i].1,
            Err(_) => 0.0,
        }
    }

    pub fn add(&mut self, good: GoodId, qty: f64) {
        assert!(qty >= 0.0, "cannot add negative quantity");
        if qty == 0.0 { return; }
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => self.0[i].1 += qty,
            Err(i) => self.0.insert(i, (good, qty)),
        }
    }

    /// Removes up to `qty` of `good`. Returns how much was actually removed.
    pub fn remove(&mut self, good: GoodId, qty: f64) -> f64 {
        assert!(qty >= 0.0, "cannot remove negative quantity");
        match self.0.binary_search_by_key(&good, |&(g, _)| g) {
            Ok(i) => {
                let removed = self.0[i].1.min(qty);
                self.0[i].1 -= removed;
                if self.0[i].1 <= 0.0 { self.0.remove(i); }
                removed
            }
            Err(_) => 0.0,
        }
    }

    /// True if this good has any quantity.
    pub fn contains(&self, good: GoodId) -> bool {
        self.0.binary_search_by_key(&good, |&(g, _)| g).is_ok()
    }

    pub fn goods(&self) -> &[(GoodId, f64)] { &self.0 }

    pub fn total_goods(&self) -> usize { self.0.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ids::GoodId;

    #[test]
    fn add_and_get() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 10.0);
        assert_eq!(inv.get(g), 10.0);
    }

    #[test]
    fn remove_partial() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 10.0);
        let removed = inv.remove(g, 4.0);
        assert_eq!(removed, 4.0);
        assert_eq!(inv.get(g), 6.0);
    }

    #[test]
    fn remove_more_than_held() {
        let mut inv = Inventory::default();
        let g = GoodId(0);
        inv.add(g, 3.0);
        let removed = inv.remove(g, 10.0);
        assert_eq!(removed, 3.0);
        assert_eq!(inv.get(g), 0.0);
        assert!(!inv.contains(g));
    }

    #[test]
    fn stays_sorted() {
        let mut inv = Inventory::default();
        inv.add(GoodId(5), 1.0);
        inv.add(GoodId(2), 2.0);
        inv.add(GoodId(8), 3.0);
        let ids: Vec<_> = inv.goods().iter().map(|&(g, _)| g).collect();
        assert_eq!(ids, vec![GoodId(2), GoodId(5), GoodId(8)]);
    }
}
