use crate::state::{
    game_data::{GameData, RegionDef, WealthLevel},
    sim_state::SimState,
};
use crate::types::{
    building::Building,
    channel::{ChannelDef, ChannelState},
    good::{GoodDef, MovementType, ShelfLife},
    ids::{
        BuildingId, ChannelId, GoodId, InventoryId, MagicProducerId, MarketNodeId,
        PopGroupId, PopPairId, RecipeId, RecipeInstanceId, RegionId,
    },
    inventory::Inventory,
    magic_producer::MagicProducer,
    market_node::MarketNodeDef,
    need_category::{NeedCategory, NeedEntry},
    pop_group::{PopGroup, PopPair},
    recipe::{InputScaling, RecipeDef, RecipeInput, RecipeOutput, StrategyKind},
    recipe_instance::{CapacityControlState, DividendPayoutState, RecipeInstance, StrategyState},
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
    #[serde(default = "raw_default_strategy")]
    pub strategy: StrategyKind,
}

fn raw_default_strategy() -> StrategyKind { StrategyKind::CapacityControl }

/// An additional recipe instance not auto-generated from a building definition.
/// Used to wire dividend payouts: input comes from a building's inventory,
/// output goes to a specific pop group's inventory.
#[derive(Deserialize)]
pub struct RawExtraInstance {
    /// Name of the recipe this instance runs.
    pub recipe: String,
    /// The building whose inventory is used as input_inv (and region source).
    pub building: BuildingId,
    /// The pop group whose inventory receives outputs.
    pub output_pop: PopGroupId,
    /// Maximum recipe_size (for DividendPayout this is effectively unused).
    pub recipe_size: f64,
}

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
    /// Desk kernel dials. Required, with no code-side default: a scenario that
    /// omits the block fails to load rather than inheriting a constant from the
    /// source (METHODOLOGY R2).
    pub kernel: crate::state::game_data::KernelParams,
    #[serde(default)]
    pub need_categories: Vec<RawNeedCategory>,
    #[serde(default)]
    pub wealth_levels: Vec<RawWealthLevel>,
    pub market_nodes: Vec<MarketNodeDef>,
    #[serde(default)]
    pub channels: Vec<ChannelDef>,
    pub regions: Vec<RegionDef>,
}

// ── Raw sim state ─────────────────────────────────────────────────────────────

/// A flat map of good_name → price, applied uniformly to all nodes.
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
    /// The good this pop supplies as labour. Optional; if absent the pop posts no labour orders.
    #[serde(default)]
    pub labour_good: Option<String>,
    /// The pop's outside option in subsistence baskets per unit of labour.
    /// Required wherever the pop supplies labour, with no code default: it
    /// drives the participation margin, and a behavioural constant with a live
    /// consumer belongs in the tape (R2).
    #[serde(default)]
    pub parity: Option<f64>,
    /// Genesis value of the labour margin `pi`, the fraction of the pop's hours
    /// actually offered to the market. `None` means 1.0 — full participation.
    ///
    /// **Why this is optional rather than required, which is a departure from
    /// how `wealth` is handled.** `pi` was a hardcoded genesis of 1.0 in this
    /// loader until 2026-07-31, and the reason nobody noticed is structural: in
    /// any economy whose only primary input is labour, `sigma_pi = +1`
    /// identically at the zero-profit price vector, so the margin is pinned at
    /// the `pi = 1` corner and genesis is the rest point whatever the tape says.
    /// `data/scenarios/solv_labour` is the first world with a scarce second
    /// factor, so the first with an INTERIOR equilibrium margin (`pi* = 0.5`) —
    /// and a state variable that cannot be stated in the tape is one the tape
    /// cannot start at rest. Making the field required would have rewritten 28
    /// tapes to say the thing they already meant, so the missing-field meaning
    /// is the old behaviour exactly and no existing scenario changed.
    #[serde(default)]
    pub participation: Option<f64>,
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
    /// Starting price overrides applied uniformly to all nodes.
    /// Omit or leave empty to start all goods at 1.0 everywhere.
    #[serde(default)]
    pub prices: NodePrices,
    #[serde(default)]
    pub buildings: Vec<RawBuilding>,
    #[serde(default)]
    pub pop_groups: Vec<RawPopGroup>,
    #[serde(default)]
    pub channels: Vec<ChannelState>,
    #[serde(default)]
    pub magic_producers: Vec<RawMagicProducer>,
    /// Additional recipe instances beyond those auto-created from buildings.
    /// For dividend payouts, supply chains, and other cross-inventory transformations.
    #[serde(default)]
    pub extra_recipe_instances: Vec<RawExtraInstance>,
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
            inv.add(self.good(&name)?, qty, None);
        }
        Ok(inv)
    }

    fn prices(&self, raw_prices: NodePrices) -> crate::scenario::loader::Result<Vec<f64>> {
        let mut prices = vec![1.0f64; self.num_nodes * self.num_goods];
        for (name, price) in raw_prices {
            let good_id = self.good(&name)?;
            for node_idx in 0..self.num_nodes {
                prices[node_idx * self.num_goods + good_id.idx()] = price;
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
    // Checked here rather than at first use: a nonsensical dial otherwise
    // surfaces as a panic deep inside a rule thousands of ticks later, with
    // nothing in the message saying which dial.
    raw.kernel.validate()?;

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
                strategy: r.strategy,
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

    let game_data = GameData {
        goods,
        recipes,
        kernel: raw.kernel,
        need_categories,
        wealth_levels,
        market_nodes: raw.market_nodes,
        channels: raw.channels,
        regions: raw.regions,
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
        let inv_id = InventoryId(state.inventories.len() as u32);
        state.inventories.push(resolver.inventory(b.inventory)?);
        state.buildings.push(Building {
            id: b.id,
            region: b.region,
            inventory: inv_id,
        });
        let transfer_target = b.transfer_target.map(|t| RecipeInstanceId(t.0));
        let recipe_id = resolver.recipe(&b.recipe)?;
        // Honor the recipe's declared strategy; the strategy_state variant must
        // match RecipeDef.strategy or downstream systems silently mis-decide.
        let strategy_state = match game_data.recipe(recipe_id).strategy {
            StrategyKind::CapacityControl => StrategyState::CapacityControl(CapacityControlState {
                efficiency: b.efficiency,
                ..CapacityControlState::new(b.recipe_size)
            }),
            StrategyKind::DividendPayout { .. } => {
                StrategyState::DividendPayout(DividendPayoutState { smoothed_input_cost: 0.0 })
            }
            StrategyKind::AlwaysRun => StrategyState::AlwaysRun,
        };
        state.recipe_instances.push(RecipeInstance {
            id: RecipeInstanceId(b.id.0),
            region: b.region,
            recipe: recipe_id,
            input_inv: inv_id,
            output_inv: inv_id,
            recipe_size: b.recipe_size,
            chosen_size: b.chosen_size,
            channel: b.channel,
            transfer_target,
            strategy_state,
            // kernel.md: the fill EMA opens at 1.0, so a desk with no
            // history posts its full flow rather than opening muted.
            last_fill: 1.0,
        });
    }

    let default_sub = game_data.default_sub_state();
    // Each raw pop group becomes the employed half. An unemployed stub is auto-created
    // alongside it. IDs: employed = 0..n, unemployed = n..2n. All employed halves are
    // pushed first so that id.idx() == vec index for both ranges.
    let n_pops = raw.pop_groups.len() as u32;
    // (id, region, wealth, savings_target, labour_good, parity, participation)
    #[allow(clippy::type_complexity)]
    let mut stub_data: Vec<(PopGroupId, RegionId, f64, f64, Option<GoodId>, Option<f64>, f64)> =
        Vec::new();

    for p in raw.pop_groups {
        let employed_id = p.id;
        let unemployed_id = PopGroupId(n_pops + employed_id.0);
        let pair_id = PopPairId(employed_id.0);
        let labour_good = p.labour_good.map(|n| resolver.good(&n)).transpose()?;
        // A pop that sells hours must state what those hours are worth to it at
        // home, or the participation margin has nothing to compare against and
        // the labour market's imbalance keeps its price-independent pin.
        if labour_good.is_some() && p.parity.is_none() {
            return Err(format!(
                "pop {:?} supplies labour but registers no parity — see                  tools/derive_parity.py, which derives it from the tape's technology",
                p.id
            )
            .into());
        }
        if let Some(v) = p.parity {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("pop {:?}: parity must be finite and positive, got {v}", p.id).into());
            }
        }
        // The same bound Rule 2 clamps to every activation. Checked at load
        // because a tape asking for pi = 0 is asking for a labour market that
        // can never reopen, and it should say so here rather than 900 ticks in.
        let participation = p.participation.unwrap_or(1.0);
        if !(participation.is_finite() && participation > 0.0 && participation <= 1.0) {
            return Err(format!(
                "pop {:?}: participation must be in (0, 1], got {participation}",
                p.id
            )
            .into());
        }

        let inv_id = InventoryId(state.inventories.len() as u32);
        state.inventories.push(resolver.inventory(p.inventory)?);

        state.pop_groups.push(PopGroup {
            id: employed_id,
            region: p.region,
            size: p.size,
            wealth: p.wealth,
            inventory: inv_id,
            savings_target: p.savings_target,
            sub_state: default_sub.clone(),
            ema_spending: 0.0,
            ema_balance: 0.0,
            prev_spend_error: 0.0,
            labour_good,
            is_employed: true,
            last_labour_fill_rate: 1.0,
            // Full participation unless the tape says otherwise; the margin
            // moves it from there. See RawPopGroup::participation.
            participation,
            parity: p.parity,
        });

        stub_data.push((
            unemployed_id,
            p.region,
            p.wealth,
            p.savings_target,
            labour_good,
            p.parity,
            participation,
        ));

        state.pop_pairs.push(PopPair {
            id: pair_id,
            employed: employed_id,
            unemployed: unemployed_id,
        });
    }

    // Push all unemployed stubs after employed halves so id.idx() == vec position.
    for (unemployed_id, region, wealth, savings_target, labour_good, parity, participation) in
        stub_data
    {
        let inv_id = InventoryId(state.inventories.len() as u32);
        state.inventories.push(Inventory::default());
        state.pop_groups.push(PopGroup {
            id: unemployed_id,
            region,
            size: 0.0,
            wealth,
            inventory: inv_id,
            savings_target,
            sub_state: default_sub.clone(),
            ema_spending: 0.0,
            ema_balance: 0.0,
            prev_spend_error: 0.0,
            labour_good,
            is_employed: false,
            last_labour_fill_rate: 1.0,
            // Both halves of a pair share one outside option and one margin, so
            // the stub inherits the employed half's genesis pi rather than
            // opening at a different one.
            participation,
            parity,
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

    for raw in raw.extra_recipe_instances {
        let recipe_id = resolver.recipe(&raw.recipe)?;
        let building = state.buildings.iter().find(|b| b.id == raw.building)
            .ok_or_else(|| format!("extra_recipe_instance references unknown building {:?}", raw.building))?;
        let building_inv = building.inventory;
        let building_region = building.region;
        let pop_inv = state.pop_groups.get(raw.output_pop.idx())
            .ok_or_else(|| format!("extra_recipe_instance references unknown pop {:?}", raw.output_pop))?
            .inventory;
        let ri_id = RecipeInstanceId(state.recipe_instances.len() as u32);
        let strategy_state = match &game_data.recipe(recipe_id).strategy {
            StrategyKind::CapacityControl => StrategyState::CapacityControl(
                CapacityControlState::new(raw.recipe_size)
            ),
            StrategyKind::DividendPayout { .. } => StrategyState::DividendPayout(
                DividendPayoutState { smoothed_input_cost: 0.0 }
            ),
            StrategyKind::AlwaysRun => StrategyState::AlwaysRun,
        };
        state.recipe_instances.push(RecipeInstance {
            id: ri_id,
            region: building_region,
            recipe: recipe_id,
            input_inv: building_inv,
            output_inv: pop_inv,
            recipe_size: raw.recipe_size,
            chosen_size: 0.0,
            channel: None,
            transfer_target: None,
            strategy_state,
            // kernel.md: the fill EMA opens at 1.0, so a desk with no
            // history posts its full flow rather than opening muted.
            last_fill: 1.0,
        });
    }

    // Warm-start smoothed_input_cost for DividendPayout instances.
    // Without this, the reserve = 26 × 0 = 0 on the first tick, so buildings
    // pay out their entire starting capital before production has begun.
    // We initialise to the sum of all CapacityControl instances on the same
    // inventory's weekly input cost at recipe_size throughput and starting prices.
    let ri_count = state.recipe_instances.len();
    for i in 0..ri_count {
        if !matches!(state.recipe_instances[i].strategy_state, StrategyState::DividendPayout(_)) {
            continue;
        }
        let input_inv = state.recipe_instances[i].input_inv;
        let region = state.recipe_instances[i].region;
        let node = game_data.region(region).market_node;

        let warm_cost: f64 = state.recipe_instances.iter()
            .filter(|other| other.input_inv == input_inv && other.strategy_state.capacity_control().is_some())
            .map(|other| {
                let recipe = game_data.recipe(other.recipe);
                recipe.inputs.iter()
                    .map(|inp| inp.desired(other.recipe_size, other.recipe_size) * state.price(node, inp.good))
                    .sum::<f64>()
            })
            .sum();

        if let StrategyState::DividendPayout(ds) = &mut state.recipe_instances[i].strategy_state {
            ds.smoothed_input_cost = warm_cost;
        }
    }

    Ok(state)
}
