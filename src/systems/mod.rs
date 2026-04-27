pub mod auto_recipes;
pub mod clearing;
pub mod decisions;
pub mod pop_update;
pub mod price_update;
pub mod production;
pub mod transactions;

use crate::scenario::EventSchedule;
use crate::state::{apply_state_deltas, GameData, SimState};
use crate::types::delta::StateDelta;

/// Run one complete tick.
///
/// Phase order:
///   0  Events      — recurring (UBI etc.) + one-shot (shocks, unlocks)
///   1  Decisions   — agents post orders; pops snapshot their currency balance
///   2  Clearing    — aggregate supply/demand, compute fill rates and imbalances
///   3  Transactions— goods move, currency settles
///   4  Pop update  — consume received goods, adjust unbounded targets via savings ratio
///   5  Production  — buildings transform inputs into outputs
///   6  Auto-recipes— spoilage, maturity (stub; reserved for future)
///   7  Price update— posted prices for next tick from imbalances
pub fn run_tick(state: &mut SimState, game_data: &GameData, events: &EventSchedule) {
    // Phase 0 — Events (recurring + one-shot; UBI fires here as recurring delta)
    let event_deltas = events.deltas_for_tick(state.tick);
    apply_state_deltas(state, &event_deltas);

    // Phase 1 — Decisions
    let (decisions_deltas, orders) = decisions::run(state, game_data);
    apply_state_deltas(state, &decisions_deltas);

    // Phase 2 — Clearing
    let (clearing_deltas, fills) = clearing::run(state, game_data, &orders);
    apply_state_deltas(state, &clearing_deltas);

    // Phase 3 — Transactions
    let d = transactions::run(state, game_data, &orders, &fills);
    apply_state_deltas(state, &d);

    // Phase 4 — Pop update (consumption + savings-ratio target adjustment)
    let d = pop_update::run(state, game_data);
    apply_state_deltas(state, &d);

    // Phase 5 — Production
    let d = production::run(state, game_data);
    apply_state_deltas(state, &d);

    // Phase 6 — Auto-recipes (spoilage/maturity stub)
    let d = auto_recipes::run(state, game_data);
    apply_state_deltas(state, &d);

    // Phase 7 — Price update
    let d = price_update::run(state, game_data);
    apply_state_deltas(state, &d);

    apply_state_deltas(state, &[StateDelta::AdvanceTick]);
}
