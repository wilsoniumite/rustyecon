//! Conservation ledger integration tests (v2 Phase 1, docs/architecture/engine.md).
//!
//! Conservation is asserted, not assumed (METHODOLOGY R3). `run_tick` closes every
//! tick by reconciling the observed change in Σ inventory against the sum of the
//! provenance-tagged mint/burn lines; a breach panics. These tests cover both
//! directions: that real scenarios conserve, and that the ledger actually catches
//! a leak rather than passing vacuously.

use std::path::PathBuf;

use rustyecon::{
    certify::ConservationLedger,
    scenario::loader,
    state::{apply_state_deltas_ledgered, SimState},
    systems::run_tick,
    types::{
        delta::StateDelta,
        ids::{GoodId, InventoryId},
        inventory::Inventory,
        provenance::Provenance,
    },
};

fn one_inventory_state() -> SimState {
    let mut state = SimState::new(2, 1);
    state.inventories.push(Inventory::default());
    state
}

#[test]
fn untagged_creation_is_caught_through_apply() {
    let mut state = one_inventory_state();
    let (inv, good) = (InventoryId(0), GoodId(0));

    // Units appear with no counterparty removal and no provenance line.
    let mut ledger = ConservationLedger::new(2);
    apply_state_deltas_ledgered(
        &mut state,
        &[StateDelta::AddToInventory { inv, good, qty: 5.0, life: None, prov: Provenance::Transfer }],
        &mut ledger,
    );
    assert!(
        ledger.worst_breach().is_some(),
        "an unpaired transfer-tagged add is unexplained creation"
    );

    // The same movement, declared as production, reconciles.
    let mut ledger = ConservationLedger::new(2);
    apply_state_deltas_ledgered(
        &mut state,
        &[StateDelta::AddToInventory { inv, good, qty: 5.0, life: None, prov: Provenance::Production }],
        &mut ledger,
    );
    assert!(ledger.worst_breach().is_none(), "a declared mint conserves");
}

#[test]
fn shortfall_is_captured_and_not_silently_clamped() {
    let mut state = one_inventory_state();
    state.inventories[0].add(GoodId(0), 2.0, None);

    let mut ledger = ConservationLedger::new(2);
    apply_state_deltas_ledgered(
        &mut state,
        &[StateDelta::RemoveFromInventory {
            inv: InventoryId(0),
            good: GoodId(0),
            qty: 7.0,
            prov: Provenance::Consumption,
        }],
        &mut ledger,
    );

    assert_eq!(ledger.shortfalls().len(), 1, "the clamp becomes a ledger line");
    assert!((ledger.shortfalls()[0].missing() - 5.0).abs() < 1e-12);
    // Only what was actually there is burned, so the books still balance.
    assert!((ledger.burned(GoodId(0)) - 2.0).abs() < 1e-12);
    assert!(ledger.worst_breach().is_none());
}

#[test]
fn unbalanced_transfer_through_apply_is_a_breach() {
    let mut state = one_inventory_state();
    state.inventories.push(Inventory::default());
    state.inventories[0].add(GoodId(1), 10.0, None);
    let mut ledger = ConservationLedger::new(2);

    // Seller ships 5 but the buyer is credited only 3 — two units vanish.
    apply_state_deltas_ledgered(
        &mut state,
        &[
            StateDelta::RemoveFromInventory {
                inv: InventoryId(0),
                good: GoodId(1),
                qty: 5.0,
                prov: Provenance::Transfer,
            },
            StateDelta::AddToInventory {
                inv: InventoryId(1),
                good: GoodId(1),
                qty: 3.0,
                life: None,
                prov: Provenance::Transfer,
            },
        ],
        &mut ledger,
    );

    let (good, drift) = ledger.worst_breach().expect("phantom consumption must be caught");
    assert_eq!(good, GoodId(1));
    assert!((drift - 2.0).abs() < 1e-12, "drift was {drift}");
}

#[test]
fn representative_scenarios_conserve_over_a_run() {
    // Spread across the factorial: differing labour supply, channel size, wheat
    // supply and start state, plus the un-generated baseline.
    for name in ["lr_00", "lr_11", "lr_23", "multi_region"] {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/scenarios")
            .join(name);
        let s = loader::load(&dir).unwrap_or_else(|e| panic!("{name} loads: {e}"));
        let (mut state, gd, events) = (s.state, s.game_data, s.events);

        // run_tick asserts the ledger at the close of every tick — a leak panics here.
        for _ in 0..120 {
            run_tick(&mut state, &gd, &events);
        }

        assert_eq!(state.tick, 120, "{name} advanced 120 ticks");
        // Non-vacuous: the economy actually holds and moves stock.
        let total: f64 = state
            .inventories
            .iter()
            .flat_map(|i| i.goods())
            .map(|(_, q)| q)
            .sum();
        assert!(total > 0.0, "{name} holds stock after 120 ticks");
    }
}
