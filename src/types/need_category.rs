use crate::types::ids::GoodId;
use serde::{Deserialize, Serialize};

/// One substitutable good within a need category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedEntry {
    pub good: GoodId,
    /// Base preference weight — higher = more desirable within the category.
    pub weight: f64,
    /// Maximum fraction of this category's demand that can go to this good.
    /// Acts as a supply-share ceiling (Vic3: max_supply_share).
    pub max_allocation: f64,
    /// Guaranteed minimum fraction regardless of supply (Vic3: min_supply_share).
    pub min_allocation: f64,
}

/// A set of mutually substitutable goods serving the same need.
/// Pops allocate total category demand across entries; allocation shifts slowly each tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedCategory {
    pub name: String,
    pub entries: Vec<NeedEntry>,
}

impl NeedCategory {
    /// Weight-proportional target fractions, clipped to [min, max] per entry, then renormalised.
    pub fn target_fractions(&self) -> Vec<f64> {
        let n = self.entries.len();
        if n == 0 { return vec![]; }
        if n == 1 { return vec![1.0]; }

        let total_weight: f64 = self.entries.iter().map(|e| e.weight).sum();
        let mut fracs: Vec<f64> = self.entries.iter().map(|e| {
            let raw = if total_weight > 0.0 { e.weight / total_weight } else { 1.0 / n as f64 };
            raw.clamp(e.min_allocation, e.max_allocation)
        }).collect();

        let sum: f64 = fracs.iter().sum();
        if sum > 1e-12 {
            for f in fracs.iter_mut() { *f /= sum; }
        }
        fracs
    }
}
