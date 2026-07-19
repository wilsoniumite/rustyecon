//! Run telemetry in tidy long form (v2 Phase 2, spec architecture/engine.md).
//!
//! Replaces the v1 pair of write-only CSVs. Two things were wrong with those
//! beyond the format: nothing in the analysis layer ever read them (the Python
//! side reconstructed everything from per-tick RON checkpoints instead), and
//! they were written with `{:.6}`, which silently rounds a quantity of 5.7e-24
//! to zero and inflates a price of 2.24e18 into twenty digits of noise. Collapsed
//! markets are precisely the runs worth studying, and the v1 telemetry destroyed
//! them on the way out. Parquet stores the f64 the engine actually held.
//!
//! The table is one row per observation: `(tick, region, entity_kind, id, good,
//! metric, value)`. Adding a metric is adding rows, never a schema migration —
//! which is the property the research readouts of Phase 6/7 will need.
//!
//! What is emitted here is deliberately the *primitives* the analysis layer used
//! to reconstruct from checkpoints, plus the per-region series the engine already
//! computes for certification. Derived quantities that Python can compute from
//! primitives cheaply (imbalance, price index) stay derived — storing them would
//! be storing the same information twice and inviting the two copies to disagree.

use std::collections::HashMap;
use std::path::Path;

use crate::certify::MetricsCollector;
use crate::output::manifest::{Manifest, MetricDim, RunInfo, TelemetryInfo};
use crate::output::parquet::{Result, TelemetryWriter, DEFAULT_ROWS_PER_GROUP};
use crate::state::{GameData, SimState};
use crate::types::ids::{GoodId, MarketNodeId};

pub const TELEMETRY_FILE: &str = "telemetry.parquet";
pub const MANIFEST_FILE: &str = "manifest.json";

/// Every `(entity_kind, metric, per_good)` this module can emit.
///
/// Declared as data rather than left implicit in the emit code so the manifest
/// can advertise it and a reader can fail loudly on a metric that was never
/// written, instead of quietly returning an empty frame that looks like "the
/// economy did nothing".
const CATALOGUE: &[(&str, &str, bool)] = &[
    ("market", "price", true),
    ("market", "supply", true),
    ("market", "demand", true),
    ("building", "recipe_id", false),
    ("building", "recipe_size", false),
    ("building", "chosen_size", false),
    ("building", "efficiency", false),
    ("building", "balance", false),
    ("building", "last_margin", false),
    ("building", "last_throughput", false),
    ("building", "channel_id", false),
    ("building", "inventory", true),
    ("pop", "size", false),
    ("pop", "wealth", false),
    ("pop", "savings_target", false),
    ("pop", "is_employed", false),
    ("pop", "inventory", true),
    ("region", "velocity", false),
    ("region", "employment", false),
    ("region", "real_wage", false),
    ("region", "concentration", false),
    ("region", "bld_util", false),
    ("region", "real_income", false),
];

/// The region-level series, paired with their accessor into [`crate::certify::RegionSeries`].
const REGION_METRICS: &[&str] = &[
    "velocity",
    "employment",
    "real_wage",
    "concentration",
    "bld_util",
    "real_income",
];

/// Streams a run's observations to Parquet, then writes the manifest.
pub struct Telemetry {
    writer: Option<TelemetryWriter>,
    every: u64,
    num_nodes: usize,
    num_goods: usize,
    first_tick: Option<u64>,
    last_tick: u64,
    /// `InventoryId -> (building_id, region_id)`. Production rows are keyed by
    /// the *building* that owns the instance's input inventory, matching the
    /// identity the analysis layer has always used; a recipe instance has its
    /// own id, and conflating the two would silently renumber every building.
    inv_to_building: HashMap<u32, (u32, u32)>,
}

impl Telemetry {
    pub fn create(dir: &Path, every: u64) -> Result<Self> {
        let writer = TelemetryWriter::create(&dir.join(TELEMETRY_FILE), DEFAULT_ROWS_PER_GROUP)?;
        Ok(Self {
            writer: Some(writer),
            every: every.max(1),
            num_nodes: 0,
            num_goods: 0,
            first_tick: None,
            last_tick: 0,
            inv_to_building: HashMap::new(),
        })
    }

    fn due(&self, tick: u64) -> bool {
        // Tick 0 is always sampled: it is the price baseline every index is
        // relative to, and a subsampled run that skipped it would silently
        // rebase onto whatever tick happened to land first.
        tick == 0 || tick % self.every == 0
    }

    /// Sample the whole state for this tick.
    pub fn record(&mut self, state: &SimState, game_data: &GameData) -> Result<()> {
        let tick = state.tick;
        if !self.due(tick) {
            return Ok(());
        }
        self.num_nodes = game_data.num_nodes();
        self.num_goods = game_data.num_goods();
        self.first_tick.get_or_insert(tick);
        self.last_tick = tick;

        self.inv_to_building.clear();
        for b in &state.buildings {
            self.inv_to_building.insert(b.inventory.0, (b.id.0, b.region.0));
        }

        self.record_markets(state, game_data)?;
        self.record_production(state)?;
        self.record_pops(state)?;
        self.record_inventories(state)?;
        Ok(())
    }

    fn writer(&mut self) -> Result<&mut TelemetryWriter> {
        self.writer
            .as_mut()
            .ok_or_else(|| "telemetry already closed".into())
    }

    fn record_markets(&mut self, state: &SimState, _game_data: &GameData) -> Result<()> {
        let tick = state.tick;
        let (nodes, goods) = (self.num_nodes, self.num_goods);
        // Prices belong to a market node, and a national or world node serves no
        // single region — hence `region: None` here rather than a guess. The
        // manifest carries the node→region map for analysis that wants to join.
        for node_idx in 0..nodes {
            let node = MarketNodeId(node_idx as u32);
            for good_idx in 0..goods {
                let good = GoodId(good_idx as u32);
                let id = node_idx as u32;
                let g = Some(good_idx as u32);
                let (p, s, d) = (
                    state.price(node, good),
                    state.supply(node, good),
                    state.demand(node, good),
                );
                let w = self.writer()?;
                w.push(tick, None, "market", id, g, "price", p)?;
                w.push(tick, None, "market", id, g, "supply", s)?;
                w.push(tick, None, "market", id, g, "demand", d)?;
            }
        }
        Ok(())
    }

    fn record_production(&mut self, state: &SimState) -> Result<()> {
        let tick = state.tick;
        for ri in &state.recipe_instances {
            // Only capacity-controlled instances are "buildings" in the analysis
            // sense; dividend and always-run instances have no capacity state to
            // report and were never in the frame.
            let Some(cc) = ri.strategy_state.capacity_control() else {
                continue;
            };
            let Some(&(bld_id, region_id)) = self.inv_to_building.get(&ri.input_inv.0) else {
                continue;
            };
            // -1 for "no channel" is the convention the analysis layer already
            // reads; `value` is a float column, so it is written as -1.0.
            let channel = ri.channel.map(|c| c.0 as f64).unwrap_or(-1.0);
            let r = Some(region_id);
            let fields: [(&'static str, f64); 8] = [
                ("recipe_id", ri.recipe.0 as f64),
                ("recipe_size", ri.recipe_size),
                ("chosen_size", ri.chosen_size),
                ("efficiency", cc.efficiency),
                ("balance", cc.balance),
                ("last_margin", cc.last_margin),
                ("last_throughput", cc.last_throughput),
                ("channel_id", channel),
            ];
            let w = self.writer()?;
            for (metric, value) in fields {
                w.push(tick, r, "building", bld_id, None, metric, value)?;
            }
        }
        Ok(())
    }

    fn record_pops(&mut self, state: &SimState) -> Result<()> {
        let tick = state.tick;
        for p in &state.pop_groups {
            let r = Some(p.region.0);
            let fields: [(&'static str, f64); 4] = [
                ("size", p.size),
                ("wealth", p.wealth),
                ("savings_target", p.savings_target),
                ("is_employed", if p.is_employed { 1.0 } else { 0.0 }),
            ];
            let w = self.writer()?;
            for (metric, value) in fields {
                w.push(tick, r, "pop", p.id.0, None, metric, value)?;
            }
        }
        Ok(())
    }

    fn record_inventories(&mut self, state: &SimState) -> Result<()> {
        let tick = state.tick;
        // Zero holdings are omitted: the long format is sparse by nature, and a
        // missing row means "held none", which is what the analysis layer's
        // reindex-and-fill already assumes.
        for b in &state.buildings {
            let (id, r) = (b.id.0, Some(b.region.0));
            let held: Vec<(GoodId, f64)> = state
                .inventory(b.inventory)
                .goods()
                .into_iter()
                .filter(|&(_, q)| q > 0.0)
                .collect();
            let w = self.writer()?;
            for (good, qty) in held {
                w.push(tick, r, "building", id, Some(good.0), "inventory", qty)?;
            }
        }
        for p in &state.pop_groups {
            let (id, r) = (p.id.0, Some(p.region.0));
            let held: Vec<(GoodId, f64)> = state
                .inventory(p.inventory)
                .goods()
                .into_iter()
                .filter(|&(_, q)| q > 0.0)
                .collect();
            let w = self.writer()?;
            for (good, qty) in held {
                w.push(tick, r, "pop", id, Some(good.0), "inventory", qty)?;
            }
        }
        Ok(())
    }

    /// Emit the per-region series the certification stack just sampled.
    ///
    /// Reads the tail of each series rather than recomputing: these are the exact
    /// numbers the run is scored against, so telemetry and verdict cannot drift
    /// apart into two different answers about the same tick.
    pub fn record_region_metrics(&mut self, tick: u64, collector: &MetricsCollector) -> Result<()> {
        if !self.due(tick) {
            return Ok(());
        }
        for s in collector.series() {
            let id = s.region.0;
            let samples: Vec<(&'static str, f64)> = REGION_METRICS
                .iter()
                .zip([
                    s.velocity.last(),
                    s.employment.last(),
                    s.real_wage.last(),
                    s.concentration.last(),
                    s.bld_util.last(),
                    s.real_income.last(),
                ])
                .filter_map(|(&m, v)| v.map(|&v| (m, v)))
                .collect();
            let w = self.writer()?;
            for (metric, value) in samples {
                w.push(tick, Some(id), "region", id, None, metric, value)?;
            }
        }
        Ok(())
    }

    /// Close the Parquet file and write the manifest beside it.
    pub fn finish(mut self, dir: &Path, game_data: &GameData, run: RunInfo) -> Result<u64> {
        let writer = self.writer.take().ok_or("telemetry already closed")?;
        let rows = writer.close()?;
        let info = TelemetryInfo {
            file: TELEMETRY_FILE.to_string(),
            rows,
            every: self.every,
            first_tick: self.first_tick.unwrap_or(0),
            last_tick: self.last_tick,
            metrics: CATALOGUE
                .iter()
                .map(|&(kind, metric, per_good)| MetricDim {
                    entity_kind: kind.to_string(),
                    metric: metric.to_string(),
                    per_good,
                })
                .collect(),
        };
        Manifest::new(game_data, info, run).write(&dir.join(MANIFEST_FILE))?;
        Ok(rows)
    }
}
