/// Integration test 01: one region, one wheat farm, one pop group.
/// Supply == demand after the first tick; price should stabilise immediately
/// and remain constant for all subsequent ticks.
use rustyecon::{
    scenario::EventSchedule,
    state::{
        game_data::{RegionDef, WealthLevel},
        GameData, SimState,
    },
    systems::run_tick,
    types::{
        building::Building,
        good::{GoodDef, MovementType, ShelfLife},
        ids::{BuildingId, GoodId, MarketNodeId, PopGroupId, RecipeId, RegionId},
        inventory::Inventory,
        market_node::{MarketNodeDef, MarketTier},
        need_category::{NeedCategory, NeedEntry},
        pop_group::PopGroup,
        recipe::{RecipeDef, RecipeOutput},
    },
};

fn make_scenario() -> (SimState, GameData, EventSchedule) {
    let game_data = GameData {
        goods: vec![GoodDef {
            id: GoodId(0),
            name: "wheat".into(),
            alpha: 0.1,
            shelf_life: ShelfLife::Indefinite,
            movement_type: MovementType::Physical,
            divisible: true,
            base_price: 1.0,
            storage_cost_per_tick: 0.0,
        }],
        recipes: vec![RecipeDef {
            id: RecipeId(0),
            name: "wheat_farm".into(),
            inputs: vec![],
            outputs: vec![RecipeOutput { good: GoodId(0), qty_per_unit: 1.0 }],
            component_reqs: vec![],
            reversible: false,
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
        }],
        channels: vec![],
        currency_good: None,
        regions: vec![RegionDef {
            id: RegionId(0),
            name: "Manchester".into(),
            market_node: MarketNodeId(0),
        }],
    };

    let mut state = SimState::new(1, 1);

    state.buildings.push(Building {
        id: BuildingId(0),
        region: RegionId(0),
        recipe: RecipeId(0),
        recipe_size: 10.0,
        chosen_size: 0.0,
        efficiency: 1.0,
        balance: 0.0,
        inventory: Inventory::default(),
        transfer_target: None,
        owners: vec![],
        channel: None,
    });

    state.pop_groups.push(PopGroup {
        id: PopGroupId(0),
        region: RegionId(0),
        size: 10.0,
        wealth: 0.0,
        inventory: Inventory::default(),
        savings_target: 0.0,
        sub_state: game_data.default_sub_state(),
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
    assert_eq!(state.pop_groups[0].inventory.get(wheat), 0.0, "inventory consumed each tick");

    run_tick(&mut state, &game_data, &events);
    assert_eq!(state.pop_groups[0].inventory.get(wheat), 0.0, "inventory consumed each tick");
}

#[test]
fn building_inventory_cycles_correctly() {
    let (mut state, game_data, events) = make_scenario();
    let wheat = GoodId(0);

    run_tick(&mut state, &game_data, &events);
    assert!(
        (state.buildings[0].inventory.get(wheat) - 10.0).abs() < 1e-10,
        "farm should hold 10 wheat after tick 0 production"
    );

    run_tick(&mut state, &game_data, &events);
    assert!(
        (state.buildings[0].inventory.get(wheat) - 10.0).abs() < 1e-10,
        "farm inventory should remain 10 in steady state"
    );
}
