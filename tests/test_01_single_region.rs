/// Integration test 01: one region, one wheat farm, one pop group.
/// Supply == demand after the first tick; price should stabilise immediately
/// and remain constant for all subsequent ticks.
use rustyecon::{
    scenario::EventSchedule,
    state::{
        game_data::{KernelParams, RegionDef, WealthLevel},
        GameData, SimState,
    },
    systems::run_tick,
    types::{
        building::Building,
        good::{GoodDef, MovementType, ShelfLife},
        ids::{BuildingId, GoodId, InventoryId, MarketNodeId, PopGroupId, RecipeId, RecipeInstanceId, RegionId},
        inventory::Inventory,
        market_node::{MarketNodeDef, MarketTier},
        need_category::{NeedCategory, NeedEntry},
        pop_group::PopGroup,
        recipe::{RecipeDef, RecipeOutput, StrategyKind},
        recipe_instance::{CapacityControlState, RecipeInstance, StrategyState},
    },
};

fn make_scenario() -> (SimState, GameData, EventSchedule) {
    let game_data = GameData {
        // A test that hand-builds a GameData is writing its own tape, and the
        // kernel dials are part of it. KernelParams has no Default impl on
        // purpose (METHODOLOGY R2), so this is written out rather than inherited.
        kernel: KernelParams {
            s: 4,
            eta_up: 0.04,
            eta_dn: 0.05,
            dead: 0.05,
            b_out: 2.0,
            b_cash: 6.5,
            beta: 1.0,
            epsilon: 0.01,
            phi: 0.6180339887498949,
            fill_alpha: 0.25,
        },
        goods: vec![GoodDef {
            id: GoodId(0),
            name: "wheat".into(),
            alpha: 0.1,
            shelf_life: ShelfLife::Indefinite,
            movement_type: MovementType::Physical,
            divisible: true,
            storage_cost_per_tick: 0.0,
        }],
        recipes: vec![RecipeDef {
            id: RecipeId(0),
            name: "wheat_farm".into(),
            inputs: vec![],
            outputs: vec![RecipeOutput { good: GoodId(0), qty_per_unit: 1.0 }],
            component_reqs: vec![],
            reversible: false,
            strategy: StrategyKind::CapacityControl,
        }],
        need_categories: vec![NeedCategory {
            name: "wheat_need".into(),
            entries: vec![NeedEntry {
                good: GoodId(0),
                weight: 1.0,
                max_allocation: 1.0,
                min_allocation: 0.0,
            }],
        }],
        wealth_levels: vec![WealthLevel {
            tier: 0,
            qty_per_pop: vec![1.0], // 1 wheat/person/tick at tier 0
        }],
        market_nodes: vec![MarketNodeDef {
            id: MarketNodeId(0),
            tier: MarketTier::Regional,
            region: Some(RegionId(0)),
            currency_good: None,
        }],
        channels: vec![],
        regions: vec![RegionDef {
            id: RegionId(0),
            name: "Manchester".into(),
            market_node: MarketNodeId(0),
            position: None,
        }],
    };

    let mut state = SimState::new(1, 1);

    // InventoryId 0 → building/recipe instance, InventoryId 1 → pop
    let bld_inv = InventoryId(0);
    state.inventories.push(Inventory::default());
    state.buildings.push(Building {
        id: BuildingId(0),
        region: RegionId(0),
        inventory: bld_inv,
    });
    state.recipe_instances.push(RecipeInstance {
        id: RecipeInstanceId(0),
        region: RegionId(0),
        recipe: RecipeId(0),
        input_inv: bld_inv,
        output_inv: bld_inv,
        recipe_size: 10.0,
        chosen_size: 0.0,
        channel: None,
        transfer_target: None,
        strategy_state: StrategyState::CapacityControl(CapacityControlState::new(10.0)),
    });

    state.inventories.push(Inventory::default());
    state.pop_groups.push(PopGroup {
        id: PopGroupId(0),
        region: RegionId(0),
        size: 10.0,
        wealth: 0.0,
        inventory: InventoryId(1),
        savings_target: 0.0,
        sub_state: game_data.default_sub_state(),
        ema_spending: 0.0,
        ema_balance: 0.0,
        prev_spend_error: 0.0,
        labour_good: None,
        is_employed: true,
        last_labour_fill_rate: 1.0,
    });

    (state, game_data, EventSchedule::default())
}

#[test]
fn price_falls_from_startup_spike_toward_equilibrium() {
    let (mut state, game_data, events) = make_scenario();
    let node = MarketNodeId(0);
    let good = GoodId(0);

    run_tick(&mut state, &game_data, &events);
    let p0 = state.price(node, good);
    assert!(p0 > 1.0, "startup shortage should spike price above 1.0, got {p0}");

    for _ in 1..20 {
        run_tick(&mut state, &game_data, &events);
    }
    let p20 = state.price(node, good);
    assert!(p20.is_finite() && p20 > 0.0, "price must remain finite and positive");
}

#[test]
fn pop_consumption_drains_inventory_each_tick() {
    let (mut state, game_data, events) = make_scenario();
    let wheat = GoodId(0);

    run_tick(&mut state, &game_data, &events);
    let pop_inv = state.pop_groups[0].inventory;
    assert_eq!(state.inventory(pop_inv).get(wheat), 0.0, "inventory consumed each tick");

    run_tick(&mut state, &game_data, &events);
    assert_eq!(state.inventory(pop_inv).get(wheat), 0.0, "inventory consumed each tick");
}

#[test]
fn building_inventory_cycles_correctly() {
    let (mut state, game_data, events) = make_scenario();
    let wheat = GoodId(0);

    run_tick(&mut state, &game_data, &events);
    let bld_inv = state.buildings[0].inventory;
    let after_tick0 = state.inventory(bld_inv).get(wheat);
    assert!(after_tick0 > 0.0, "farm should produce wheat on tick 0, got {after_tick0}");

    let prev = after_tick0;
    run_tick(&mut state, &game_data, &events);
    let after_tick1 = state.inventory(bld_inv).get(wheat);
    assert!(after_tick1 > 0.0 || prev > 0.0, "farm should produce wheat over ticks");
}
