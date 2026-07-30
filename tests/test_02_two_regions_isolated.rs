/// Integration test 02: two isolated regions, no active channel.
/// Both regions trade wheat. Manchester is balanced (supply=demand).
/// Liverpool is oversupplied — price falls. Without a channel they don't equalise.
use rustyecon::{
    scenario::EventSchedule,
    state::{
        game_data::{KernelParams, RegionDef, WealthLevel},
        GameData, SimState,
    },
    systems::run_tick,
    types::{
        building::Building,
        channel::{ChannelDef, ChannelState, ChannelType},
        good::{GoodDef, MovementType, ShelfLife},
        ids::{BuildingId, ChannelId, GoodId, InventoryId, MarketNodeId, PopGroupId, RecipeId, RecipeInstanceId, RegionId},
        inventory::Inventory,
        market_node::{MarketNodeDef, MarketTier},
        need_category::{NeedCategory, NeedEntry},
        pop_group::PopGroup,
        recipe::{RecipeDef, RecipeOutput, StrategyKind},
        recipe_instance::{CapacityControlState, RecipeInstance, StrategyState},
    },
};

const WHEAT: GoodId = GoodId(0);
const MANCHESTER: MarketNodeId = MarketNodeId(0);
const LIVERPOOL: MarketNodeId = MarketNodeId(1);

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
            id: WHEAT,
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
            outputs: vec![RecipeOutput { good: WHEAT, qty_per_unit: 1.0 }],
            component_reqs: vec![],
            reversible: false,
            strategy: StrategyKind::CapacityControl,
        }],
        need_categories: vec![NeedCategory {
            name: "wheat_need".into(),
            entries: vec![NeedEntry {
                good: WHEAT,
                weight: 1.0,
                max_allocation: 1.0,
                min_allocation: 0.0,
            }],
        }],
        wealth_levels: vec![WealthLevel {
            tier: 0,
            qty_per_pop: vec![1.0],
        }],
        market_nodes: vec![
            MarketNodeDef { id: MANCHESTER, tier: MarketTier::Regional, region: Some(RegionId(0)), currency_good: None },
            MarketNodeDef { id: LIVERPOOL,  tier: MarketTier::Regional, region: Some(RegionId(1)), currency_good: None },
        ],
        channels: vec![ChannelDef {
            id: ChannelId(0),
            from: MANCHESTER,
            to: LIVERPOOL,
            channel_type: ChannelType::Trade,
        }],
        regions: vec![
            RegionDef { id: RegionId(0), name: "Manchester".into(), market_node: MANCHESTER, position: None },
            RegionDef { id: RegionId(1), name: "Liverpool".into(),  market_node: LIVERPOOL, position: None  },
        ],
    };

    let mut state = SimState::new(1, 2);
    state.channels.push(ChannelState::default());

    let default_sub = game_data.default_sub_state();

    // Manchester: balanced (10 farm, 10 pops)
    // InventoryId 0: bld0, 1: pop0, 2: bld1, 3: pop1
    let man_inv = InventoryId(0);
    state.inventories.push(Inventory::default());
    state.buildings.push(Building { id: BuildingId(0), region: RegionId(0), inventory: man_inv });
    state.recipe_instances.push(RecipeInstance {
        id: RecipeInstanceId(0), region: RegionId(0), recipe: RecipeId(0),
        input_inv: man_inv, output_inv: man_inv,
        recipe_size: 10.0, chosen_size: 0.0, channel: None, transfer_target: None,
        strategy_state: StrategyState::CapacityControl(CapacityControlState::new(10.0)),
    });
    state.inventories.push(Inventory::default());
    state.pop_groups.push(PopGroup {
        id: PopGroupId(0), region: RegionId(0), size: 10.0, wealth: 0.0,
        inventory: InventoryId(1), savings_target: 0.0,
        sub_state: default_sub.clone(),
        ema_spending: 0.0, ema_balance: 0.0, prev_spend_error: 0.0,
        labour_good: None, is_employed: true, last_labour_fill_rate: 1.0,
    });

    // Liverpool: oversupplied (20 farm, 10 pops)
    let liv_inv = InventoryId(2);
    state.inventories.push(Inventory::default());
    state.buildings.push(Building { id: BuildingId(1), region: RegionId(1), inventory: liv_inv });
    state.recipe_instances.push(RecipeInstance {
        id: RecipeInstanceId(1), region: RegionId(1), recipe: RecipeId(0),
        input_inv: liv_inv, output_inv: liv_inv,
        recipe_size: 20.0, chosen_size: 0.0, channel: None, transfer_target: None,
        strategy_state: StrategyState::CapacityControl(CapacityControlState::new(20.0)),
    });
    state.inventories.push(Inventory::default());
    state.pop_groups.push(PopGroup {
        id: PopGroupId(1), region: RegionId(1), size: 10.0, wealth: 0.0,
        inventory: InventoryId(3), savings_target: 0.0,
        sub_state: default_sub,
        ema_spending: 0.0, ema_balance: 0.0, prev_spend_error: 0.0,
        labour_good: None, is_employed: true, last_labour_fill_rate: 1.0,
    });

    (state, game_data, EventSchedule::default())
}

#[test]
fn balanced_market_stabilises_oversupplied_market_falls() {
    let (mut state, game_data, events) = make_scenario();

    for _ in 0..50 { run_tick(&mut state, &game_data, &events); }

    let man_price = state.price(MANCHESTER, WHEAT);
    let liv_price = state.price(LIVERPOOL, WHEAT);

    assert!(man_price.is_finite() && man_price > 0.0,
        "Manchester price must remain positive and finite: {man_price}");
    assert!(liv_price < man_price,
        "Oversupplied Liverpool should price below balanced Manchester: man={man_price} liv={liv_price}");
}

#[test]
fn markets_are_independent() {
    let (mut state, game_data, events) = make_scenario();
    for _ in 0..50 { run_tick(&mut state, &game_data, &events); }

    let man_price = state.price(MANCHESTER, WHEAT);
    let liv_price = state.price(LIVERPOOL, WHEAT);

    assert!(man_price > liv_price,
        "isolated markets must diverge: Manchester {man_price} vs Liverpool {liv_price}");
}
