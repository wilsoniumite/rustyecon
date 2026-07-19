//! Conservation ledger integration tests (v2 Phase 1, docs/architecture/engine.md).
//!
//! Conservation is asserted, not assumed (METHODOLOGY R3). `run_tick` closes every
//! tick by reconciling the observed change in Σ inventory against the sum of the
//! provenance-tagged mint/burn lines; a breach panics. These tests cover both
//! directions: that real scenarios conserve, and that the ledger actually catches
//! a leak rather than passing vacuously.

use std::path::PathBuf;

use rustyecon::{
    certify::{ledger::tally, ConservationLedger},
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

const NUM_GOODS: usize = 2;

fn one_inventory_state() -> SimState {
    let mut state = SimState::new(NUM_GOODS, 1);
    state.inventories.push(Inventory::default());
    state
}

/// Apply `deltas` and reconcile against a real before/after scan of state —
/// the same independent measurement `run_tick` uses.
fn apply_and_reconcile(state: &mut SimState, deltas: &[StateDelta]) -> (ConservationLedger, Vec<f64>, Vec<f64>) {
    let opening = tally(&state.inventories, NUM_GOODS);
    let mut ledger = ConservationLedger::new(NUM_GOODS);
    apply_state_deltas_ledgered(state, deltas, &mut ledger);
    let closing = tally(&state.inventories, NUM_GOODS);
    (ledger, opening, closing)
}

#[test]
fn untagged_creation_is_caught_through_apply() {
    let mut state = one_inventory_state();
    let (inv, good) = (InventoryId(0), GoodId(0));

    // Units appear with no counterparty removal and no provenance line.
    let (ledger, open, close) = apply_and_reconcile(
        &mut state,
        &[StateDelta::AddToInventory { inv, good, qty: 5.0, life: None, prov: Provenance::Transfer }],
    );
    assert!(
        ledger.worst_breach(&open, &close).is_some(),
        "an unpaired transfer-tagged add is unexplained creation"
    );

    // The same movement, declared as production, reconciles.
    let (ledger, open, close) = apply_and_reconcile(
        &mut state,
        &[StateDelta::AddToInventory { inv, good, qty: 5.0, life: None, prov: Provenance::Production }],
    );
    assert!(ledger.worst_breach(&open, &close).is_none(), "a declared mint conserves");
}

#[test]
fn shortfall_is_captured_and_not_silently_clamped() {
    let mut state = one_inventory_state();
    state.inventories[0].add(GoodId(0), 2.0, None);

    let (ledger, open, close) = apply_and_reconcile(
        &mut state,
        &[StateDelta::RemoveFromInventory {
            inv: InventoryId(0),
            good: GoodId(0),
            qty: 7.0,
            prov: Provenance::Consumption,
        }],
    );

    // The clamp must surface as an explicit line — this is the signal, and
    // run_tick_ledgered hands it to the certificate rather than dropping it.
    assert_eq!(ledger.shortfalls().len(), 1, "the clamp becomes a ledger line");
    assert!((ledger.shortfalls()[0].missing() - 5.0).abs() < 1e-12);
    // Only what was actually there is burned, so the books themselves balance:
    // a shortfall is a distinct signal from a conservation breach.
    assert!((ledger.burned(GoodId(0)) - 2.0).abs() < 1e-12);
    assert!(ledger.worst_breach(&open, &close).is_none());
}

#[test]
fn unbalanced_transfer_through_apply_is_a_breach() {
    let mut state = one_inventory_state();
    state.inventories.push(Inventory::default());
    state.inventories[0].add(GoodId(1), 10.0, None);

    // Seller ships 5 but the buyer is credited only 3 — two units vanish.
    let (ledger, open, close) = apply_and_reconcile(
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
    );

    let (good, drift) = ledger
        .worst_breach(&open, &close)
        .expect("phantom consumption must be caught");
    assert_eq!(good, GoodId(1));
    assert!((drift - 2.0).abs() < 1e-12, "drift was {drift}");
}

#[test]
fn tape_injection_and_phantom_supply_are_declared() {
    // These two scenarios are the only ones exercising Provenance::Event (a UBI
    // injection from the tape) and Provenance::Magic (a MagicProducer selling
    // goods it does not hold, and swallowing the payment). Both are unpaired by
    // construction — currency and goods enter or leave with no counterparty — so
    // they are precisely the paths that must be *declared* rather than silently
    // minted or burned.
    //
    // Completing the run is the assertion: until these were tagged, both scenarios
    // panicked at tick 2 with ~12.6 of currency appearing from nowhere.
    for name in ["supply_chain", "big_region"] {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/scenarios")
            .join(name);
        let s = loader::load(&dir).unwrap_or_else(|e| panic!("{name} must load: {e}"));
        let (mut state, gd, events) = (s.state, s.game_data, s.events);

        // Guard against the failure mode that made the old corpus evidence
        // worthless: a scenario can only exercise these paths if its tape
        // actually fires and it actually has a phantom producer.
        assert!(
            !events.deltas_for_tick(1).is_empty(),
            "{name}: the recurring tape entry must fire, or Event is untested"
        );
        assert!(
            !state.magic_producers.is_empty(),
            "{name}: needs a MagicProducer, or Magic is untested"
        );

        for _ in 0..60 {
            run_tick(&mut state, &gd, &events);
        }
        assert_eq!(state.tick, 60, "{name} advanced");
    }
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
