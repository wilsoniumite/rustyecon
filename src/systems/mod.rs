pub mod clearing;
pub mod decisions;
pub mod pop_update;
pub mod price_update;
pub mod production;
pub mod transactions;

use crate::scenario::EventSchedule;
use crate::state::{apply_state_deltas, GameData, SimState};
use crate::types::delta::StateDelta;
use crate::types::ids::InventoryId;

/// Run one complete tick.
///
/// Phase order:
///   0  Events      — recurring + one-shot (shocks, unlocks)
///   1  Decisions   — agents post orders (reads last tick's state)
///   2  Clearing    — aggregate supply/demand, compute fill rates and imbalances
///   3  Transactions— goods move, currency settles
///   4  Pop update  — consume received goods, wealth drift, labour redistribution
///   5  Production  — buildings transform inputs into outputs (new stock enters inventory)
///   6  Spoilage    — advance lot life counters; remove expired lots from all inventories
///   7  Price update— posted prices for next tick from imbalances
pub fn run_tick(state: &mut SimState, game_data: &GameData, events: &EventSchedule) {
    run_tick_inner(state, game_data, events, &mut None);
}

/// Run one tick and return the exact ordered delta stream that was applied.
///
/// The stream is the concatenation, in apply order, of every phase's deltas plus
/// the closing `AdvanceTick`. Re-applying it against an equal state reproduces
/// the tick exactly (`apply_state_deltas` is a pure function of state + deltas),
/// which is the basis of the replay determinism test. `run_tick` is the thin,
/// zero-overhead wrapper that discards the stream.
pub fn run_tick_capture(
    state: &mut SimState,
    game_data: &GameData,
    events: &EventSchedule,
) -> Vec<StateDelta> {
    let mut stream = Vec::new();
    run_tick_inner(state, game_data, events, &mut Some(&mut stream));
    stream
}

/// Append `d` to the capture sink if one is present. When `sink` is `None`
/// (the normal `run_tick` path) this is a no-op and `d` is simply dropped —
/// no allocation, identical to the pre-capture behaviour.
fn record(sink: &mut Option<&mut Vec<StateDelta>>, d: Vec<StateDelta>) {
    if let Some(s) = sink.as_deref_mut() {
        s.extend(d);
    }
}

fn run_tick_inner(
    state: &mut SimState,
    game_data: &GameData,
    events: &EventSchedule,
    sink: &mut Option<&mut Vec<StateDelta>>,
) {
    // Phase 0 — Events (recurring + one-shot; UBI fires here as recurring delta)
    let event_deltas = events.deltas_for_tick(state.tick);
    apply_state_deltas(state, &event_deltas);
    record(sink, event_deltas);

    // Phase 1 — Decisions
    let (decisions_deltas, orders) = decisions::run(state, game_data);
    apply_state_deltas(state, &decisions_deltas);
    record(sink, decisions_deltas);

    // Phase 2 — Clearing
    let (clearing_deltas, fills) = clearing::run(state, game_data, &orders);
    apply_state_deltas(state, &clearing_deltas);
    record(sink, clearing_deltas);

    // Phase 3 — Transactions
    let d = transactions::run(state, game_data, &orders, &fills);
    apply_state_deltas(state, &d);
    record(sink, d);

    // Phase 4 — Pop update (consumption + savings-ratio target adjustment)
    let d = pop_update::run(state, game_data);
    apply_state_deltas(state, &d);
    record(sink, d);

    // Phase 5 — Production
    let d = production::run(state, game_data);
    apply_state_deltas(state, &d);
    record(sink, d);

    // Phase 6 — Spoilage: advance lot life counters on all inventories.
    // Running after production ensures newly produced lots are decremented but not
    // immediately removed (a lot with life=1 survives this tick and the next).
    // Emitted as deltas so the mutation flows through apply_state_deltas — the sole
    // writer — and is captured by the replayable/hashable delta stream.
    let spoil_deltas: Vec<StateDelta> = (0..state.inventories.len())
        .map(|i| StateDelta::SpoilInventory { inv: InventoryId(i as u32) })
        .collect();
    apply_state_deltas(state, &spoil_deltas);
    record(sink, spoil_deltas);

    // Phase 7 — Price update
    let d = price_update::run(state, game_data);
    apply_state_deltas(state, &d);
    record(sink, d);

    apply_state_deltas(state, &[StateDelta::AdvanceTick]);
    record(sink, vec![StateDelta::AdvanceTick]);
}
