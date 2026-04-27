pub mod loader;
pub mod raw;

use crate::types::delta::StateDelta;
use serde::{Deserialize, Serialize};

/// Scheduled and recurring state deltas.
/// One-shot events fire at a specific tick; recurring entries fire every N ticks.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EventSchedule {
    /// One-shot events: (tick_number, deltas).
    #[serde(default)]
    events: Vec<(u64, Vec<StateDelta>)>,
    /// Recurring events: (every_n_ticks, deltas). Fires whenever tick % n == 0.
    /// Use n=1 for every-tick effects like UBI.
    #[serde(default)]
    recurring: Vec<(u64, Vec<StateDelta>)>,
}

impl EventSchedule {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, tick: u64, deltas: Vec<StateDelta>) {
        match self.events.binary_search_by_key(&tick, |&(t, _)| t) {
            Ok(i) => self.events[i].1.extend(deltas),
            Err(i) => self.events.insert(i, (tick, deltas)),
        }
    }

    /// Returns all deltas that should fire at the given tick (one-shot + recurring).
    pub fn deltas_for_tick(&self, tick: u64) -> Vec<StateDelta> {
        let mut deltas = Vec::new();

        // One-shot events.
        if let Ok(i) = self.events.binary_search_by_key(&tick, |&(t, _)| t) {
            deltas.extend(self.events[i].1.iter().cloned());
        }

        // Recurring events.
        for (every, rec_deltas) in &self.recurring {
            if *every > 0 && tick % every == 0 {
                deltas.extend(rec_deltas.iter().cloned());
            }
        }

        deltas
    }
}
