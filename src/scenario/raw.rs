/// Raw types used exclusively for RON authoring.
/// Good and recipe references are strings ("wheat") rather than numeric IDs.
/// The loader resolves these to runtime types after reading game_data.ron.
use crate::state::{
    game_data::{GameData, RegionDef, WealthLevel},
    sim_state::SimState,
};
use crate::types::{
    building::Building,
    channel::{ChannelDef, ChannelState},
    good::{GoodDef, MovementType, ShelfLife},
    ids::{
        BuildingId, ChannelId, GoodId, MagicProducerId, MarketNodeId, OwnerId, PopGroupId,
        RecipeId, RegionId,
    },
    inventory::Inventory,
    magic_producer::MagicProducer,
    market_node::MarketNodeDef,
    need_category::{NeedCategory, NeedEntry},
    pop_group::PopGroup,
    recipe::{InputScaling, RecipeDef, RecipeInput, RecipeOutput},
};
use serde::Deserialize;
use std::collections::HashMap;

// ── Raw good/recipe definitions ───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RawGoodDef {
    pub name: String,
    pub alpha: f64,
    pub shelf_life: ShelfLife,
    pub movement_type: MovementType,
    pub divisible: bool,
    pub storage_cost_per_tick: f64,
}

#[derive(Deserialize)]
pub struct RawRecipeInput {
    pub good: String,
    pub qty_per_unit: f64,
    pub scaling: InputScaling,
}

#[derive(Deserialize)]
pub struct RawRecipeOutput {
    pub good: String,
    pub qty_per_unit: f64,
}

#[derive(Deserialize)]
pub struct RawRecipeDef {
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<RawRecipeInput>,
    pub outputs: Vec<RawRecipeOutput>,
    pub reversible: bool,
}

// ── Raw need categories and wealth levels ─────────────────────────────────────

#[derive(Deserialize)]
pub struct RawNeedEntry {
    pub good: String,
    pub weight: f64,
    #[serde(default = "default_one")]
    pub max_allocation: f64,
    #[serde(default)]
    pub min_allocation: f64,
}

fn default_one() -> f64 { 1.0 }

#[derive(Deserialize)]
pub struct RawNeedCategory {
    pub name: String,
    pub entries: Vec<RawNeedEntry>,
}

/// Raw wealth level: maps category name → qty per pop per tick.
/// Resolved into WealthLevel.qty_per_pop (dense Vec indexed by category position).
#[derive(Deserialize)]
pub struct RawWealthLevel {
    pub tier: u8,
    /// category_name → qty demanded per person per tick at this tier.
    pub needs: HashMap<String, f64>,
}

#[derive(Deserialize)]
pub struct RawGameData {
    pub goods: Vec<RawGoodDef>,
    pub recipes: Vec<RawRecipeDef>,
    #[serde(default)]
    pub need_categories: Vec<RawNeedCategory>,
    #[serde(default)]
    pub wealth_levels: Vec<RawWealthLevel>,
    pub market_nodes: Vec<MarketNodeDef>,
    #[serde(default)]
    pub channels: Vec<ChannelDef>,
    pub regions: Vec<RegionDef>,
    #[serde(default)]
    pub currency_good: Option<String>,
}

// ── Raw sim state ─────────────────────────────────────────────────────────────

pub type NodePrices = HashMap<String, f64>;

#[derive(Deserialize)]
pub struct RawBuilding {
    pub id: BuildingId,
    pub region: RegionId,
    pub recipe: String,
    pub recipe_size: f64,
    pub chosen_size: f64,
    pub efficiency: f64,
    pub inventory: HashMap<String, f64>,
    pub transfer_target: Option<BuildingId>,
    #[serde(default)]
    pub owners: Vec<(OwnerId, f64)>,
    pub channel: Option<ChannelId>,
}

#[derive(Deserialize)]
pub struct RawPopGroup {
    pub id: PopGroupId,
    pub region: RegionId,
    pub size: f64,
    pub wealth: f64,
    pub inventory: HashMap<String, f64>,
    #[serde(default)]
    pub savings_target: f64,
}

#[derive(Deserialize)]
pub struct RawMagicProducer {
    pub id: MagicProducerId,
    pub node: MarketNodeId,
    pub good: String,
    pub qty_per_tick: f64,
}

#[derive(Deserialize)]
pub struct RawSimState {
    pub tick: u64,
    /// Optional per-node price overrides (index = node order in game_data).
    /// Omit to start all goods at 1.0 on all nodes.
    #[serde(default)]
    pub prices: Option<Vec<NodePrices>>,
    #[serde(default)]
    pub buildings: Vec<RawBuilding>,
    #[serde(default)]
    pub pop_groups: Vec<RawPopGroup>,
    #[serde(default)]
    pub channels: Vec<ChannelState>,
    #[serde(default)]
    pub magic_producers: Vec<RawMagicProducer>,
}

// ── Name resolver ─────────────────────────────────────────────────────────────

pub struct NameResolver {
    goods: HashMap<String, GoodId>,
    recipes: HashMap<String, RecipeId>,
    categories: HashMap<String, usize>,
    pub num_goods: usize,
    pub num_nodes: usize,
    pub num_categories: usize,
}

impl NameResolver {
    fn from_raw(raw: &RawGameData) -> Self {
        let goods = raw.goods.iter().enumerate()
            .map(|(i, g)| (g.name.clone(), GoodId(i as u32)))
            .collect();
        let recipes = raw.recipes.iter().enumerate()
            .map(|(i, r)| (r.name.clone(), RecipeId(i as u32)))
            .collect();
        let categories = raw.need_categories.iter().enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect();
        Self {
            goods,
            recipes,
            categories,
            num_goods: raw.goods.len(),
            num_nodes: raw.market_nodes.len(),
            num_categories: raw.need_categories.len(),
        }
    }

    fn good(&self, name: &str) -> crate::scenario::loader::Result<GoodId> {
        self.goods.get(name).copied()
            .ok_or_else(|| format!("unknown good: \"{name}\"").into())
    }

    fn recipe(&self, name: &str) -> crate::scenario::loader::Result<RecipeId> {
        self.recipes.get(name).copied()
            .ok_or_else(|| format!("unknown recipe: \"{name}\"").into())
    }

    fn category(&self, name: &str) -> crate::scenario::loader::Result<usize> {
        self.categories.get(name).copied()
            .ok_or_else(|| format!("unknown need category: \"{name}\"").into())
    }

    fn inventory(&self, raw: HashMap<String, f64>) -> crate::scenario::loader::Result<Inventory> {
        let mut inv = Inventory::default();
        for (name, qty) in raw {
            inv.add(self.good(&name)?, qty);
        }
        Ok(inv)
    }

    fn prices(&self, raw_prices: Option<Vec<NodePrices>>) -> crate::scenario::loader::Result<Vec<f64>> {
        // Default to 1.0; scenarios override via the prices field in starting_state.ron.
        let mut prices = vec![1.0f64; self.num_nodes * self.num_goods];
        if let Some(node_maps) = raw_prices {
            for (node_idx, map) in node_maps.into_iter().enumerate() {
                for (name, price) in map {
                    let good_id = self.good(&name)?;
                    prices[node_idx * self.num_goods + good_id.idx()] = price;
                }
            }
        }
        Ok(prices)
    }
}

// ── Resolution functions ──────────────────────────────────────────────────────

pub fn resolve_game_data(
    raw: RawGameData,
) -> crate::scenario::loader::Result<(GameData, NameResolver)> {
    let resolver = NameResolver::from_raw(&raw);

    let goods: Vec<GoodDef> = raw.goods.into_iter().enumerate()
        .map(|(i, g)| GoodDef {
            id: GoodId(i as u32),
            name: g.name,
            alpha: g.alpha,
            shelf_life: g.shelf_life,
            movement_type: g.movement_type,
            divisible: g.divisible,
            storage_cost_per_tick: g.storage_cost_per_tick,
        })
        .collect();

    let recipes = raw.recipes.into_iter().enumerate()
        .map(|(i, r)| -> crate::scenario::loader::Result<RecipeDef> {
            Ok(RecipeDef {
                id: RecipeId(i as u32),
                name: r.name,
                inputs: r.inputs.into_iter()
                    .map(|inp| -> crate::scenario::loader::Result<RecipeInput> {
                        Ok(RecipeInput { good: resolver.good(&inp.good)?, qty_per_unit: inp.qty_per_unit, scaling: inp.scaling })
                    })
                    .collect::<crate::scenario::loader::Result<_>>()?,
                outputs: r.outputs.into_iter()
                    .map(|out| -> crate::scenario::loader::Result<RecipeOutput> {
                        Ok(RecipeOutput { good: resolver.good(&out.good)?, qty_per_unit: out.qty_per_unit })
                    })
                    .collect::<crate::scenario::loader::Result<_>>()?,
                component_reqs: vec![],
                reversible: r.reversible,
            })
        })
        .collect::<crate::scenario::loader::Result<_>>()?;

    let need_categories: Vec<NeedCategory> = raw.need_categories.into_iter()
        .map(|cat| -> crate::scenario::loader::Result<NeedCategory> {
            Ok(NeedCategory {
                name: cat.name,
                entries: cat.entries.into_iter()
                    .map(|e| -> crate::scenario::loader::Result<NeedEntry> {
                        Ok(NeedEntry {
                            good: resolver.good(&e.good)?,
                            weight: e.weight,
                            max_allocation: e.max_allocation,
                            min_allocation: e.min_allocation,
                        })
                    })
                    .collect::<crate::scenario::loader::Result<_>>()?,
            })
        })
        .collect::<crate::scenario::loader::Result<_>>()?;

    // Resolve wealth levels: sparse HashMap<name, qty> → dense Vec<f64> by category index.
    let n_cats = resolver.num_categories;
    let mut wealth_levels: Vec<WealthLevel> = raw.wealth_levels.into_iter()
        .map(|wl| -> crate::scenario::loader::Result<WealthLevel> {
            let mut qty_per_pop = vec![0.0f64; n_cats];
            for (cat_name, qty) in wl.needs {
                let idx = resolver.category(&cat_name)?;
                qty_per_pop[idx] = qty;
            }
            Ok(WealthLevel { tier: wl.tier, qty_per_pop })
        })
        .collect::<crate::scenario::loader::Result<_>>()?;
    wealth_levels.sort_by_key(|wl| wl.tier);

    let currency_good = raw.currency_good.map(|n| resolver.good(&n)).transpose()?;

    let game_data = GameData {
        goods,
        recipes,
        need_categories,
        wealth_levels,
        market_nodes: raw.market_nodes,
        channels: raw.channels,
        regions: raw.regions,
        currency_good,
    };

    Ok((game_data, resolver))
}

pub fn resolve_sim_state(
    raw: RawSimState,
    resolver: &NameResolver,
    game_data: &GameData,
) -> crate::scenario::loader::Result<SimState> {
    let prices = resolver.prices(raw.prices)?;
    let mut state = SimState::from_prices(raw.tick, resolver.num_goods, resolver.num_nodes, prices);

    for b in raw.buildings {
        state.buildings.push(Building {
            id: b.id,
            region: b.region,
            recipe: resolver.recipe(&b.recipe)?,
            recipe_size: b.recipe_size,
            chosen_size: b.chosen_size,
            efficiency: b.efficiency,
            balance: 0.0,
            last_margin: 0.0,
            last_throughput: b.recipe_size,
            inventory: resolver.inventory(b.inventory)?,
            transfer_target: b.transfer_target,
            owners: b.owners,
            channel: b.channel,
        });
    }

    let default_sub = game_data.default_sub_state();
    for p in raw.pop_groups {
        state.pop_groups.push(PopGroup {
            id: p.id,
            region: p.region,
            size: p.size,
            wealth: p.wealth,
            inventory: resolver.inventory(p.inventory)?,
            savings_target: p.savings_target,
            sub_state: default_sub.clone(),
        });
    }

    state.channels = raw.channels;

    for mp in raw.magic_producers {
        state.magic_producers.push(MagicProducer {
            id: mp.id,
            node: mp.node,
            good: resolver.good(&mp.good)?,
            qty_per_tick: mp.qty_per_tick,
        });
    }

    Ok(state)
}
