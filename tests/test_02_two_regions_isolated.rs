/// Integration test 02: two isolated regions, no active channel.
/// Both regions trade wheat. Manchester is balanced (supply=demand).
/// Liverpool is oversupplied — price falls. Without a channel they don't equalise.
use rustyecon::{
    scenario::EventSchedule,
    state::{
        game_data::{RegionDef, WealthLevel},
        GameData, SimState,
    },
    systems::run_tick,
    types::{
        building::Building,
        channel::{ChannelDef, ChannelState, ChannelType},
        good::{GoodDef, MovementType, ShelfLife},
        ids::{BuildingId, ChannelId, GoodId, MarketNodeId, PopGroupId, RecipeId, RegionId},
        inventory::Inventory,
        market_node::{MarketNodeDef, MarketTier},
        need_category::{NeedCategory, NeedEntry},
        pop_group::PopGroup,
        recipe::{RecipeDef, RecipeOutput},
    },
};

const WHEAT: GoodId = GoodId(0);
const MANCHESTER: MarketNodeId = MarketNodeId(0);
const LIVERPOOL: MarketNodeId = MarketNodeId(1);

fn make_scenario() -> (SimState, GameData, EventSchedule) {
    let game_data = GameData {
        goods: vec![GoodDef {
            id: WHEAT,
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
            outputs: vec![RecipeOutput { good: WHEAT, qty_per_unit: 1.0 }],
            component_reqs: vec![],
            reversible: false,
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
            MarketNodeDef { id: MANCHESTER, tier: MarketTier::Regional, region: Some(RegionId(0)) },
            MarketNodeDef { id: LIVERPOOL,  tier: MarketTier::Regional, region: Some(RegionId(1)) },
        ],
        channels: vec![ChannelDef {
            id: ChannelId(0),
            from: MANCHESTER,
            to: LIVERPOOL,
            channel_type: ChannelType::Trade,
            base_crossing_cost: 0.1,
        }],
        currency_good: None,
        regions: vec![
            RegionDef { id: RegionId(0), name: "Manchester".into(), market_node: MANCHESTER },
            RegionDef { id: RegionId(1), name: "Liverpool".into(),  market_node: LIVERPOOL  },
        ],
    };

    let mut state = SimState::new(1, 2);
    state.channels.push(ChannelState::default());

    let default_sub = game_data.default_sub_state();

    // Manchester: balanced (10 farm, 10 pops)
    state.buildings.push(Building {
        id: BuildingId(0), region: RegionId(0), recipe: RecipeId(0),
        recipe_size: 10.0, chosen_size: 0.0, efficiency: 1.0, balance: 0.0,
        inventory: Inventory::default(), transfer_target: None, owners: vec![], channel: None,
    });
    state.pop_groups.push(PopGroup {
        id: PopGroupId(0), region: RegionId(0), size: 10.0, wealth: 0.0,
        inventory: Inventory::default(), savings_target: 0.0,
        sub_state: default_sub.clone(),
    });

    // Liverpool: oversupplied (20 farm, 10 pops)
    state.buildings.push(Building {
        id: BuildingId(1), region: RegionId(1), recipe: RecipeId(0),
        recipe_size: 20.0, chosen_size: 0.0, efficiency: 1.0, balance: 0.0,
        inventory: Inventory::default(), transfer_target: None, owners: vec![], channel: None,
    });
    state.pop_groups.push(PopGroup {
        id: PopGroupId(1), region: RegionId(1), size: 10.0, wealth: 0.0,
        inventory: Inventory::default(), savings_target: 0.0,
        sub_state: default_sub,
    });

    (state, game_data, EventSchedule::default())
}

#[test]
fn balanced_market_stabilises_oversupplied_market_falls() {
    let (mut state, game_data, events) = make_scenario();

    run_tick(&mut state, &game_data, &events);
    let man_spike = state.price(MANCHESTER, WHEAT);
    assert!(man_spike > 1.0, "tick-0 shortage should spike Manchester wheat price");

    for _ in 1..50 { run_tick(&mut state, &game_data, &events); }

    let man_price = state.price(MANCHESTER, WHEAT);
    let liv_price = state.price(LIVERPOOL, WHEAT);

    assert!((man_price - man_spike).abs() < 1e-6,
        "Manchester should stabilise at spike level: {man_spike} → {man_price}");
    assert!(liv_price < 1.0, "Liverpool should fall under oversupply: {liv_price}");
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
