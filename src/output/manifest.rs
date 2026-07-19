//! Run manifest accompanying the Parquet telemetry (v2 Phase 2, architecture/engine.md).
//!
//! engine.md specifies the manifest as run identity — git sha, tape hash, seed,
//! criteria hash. It carries the **dimension tables** as well, and that addition
//! is what actually delivers the Phase 2 gate: the long table holds integer ids,
//! so without a good_id→name map here the analysis layer would still have to
//! parse `game_data.ron` to say the word "flour". Identity alone would have left
//! a RON parser in the analysis path and called the gate met.
//!
//! JSON, not Parquet: it is a handful of kilobytes read once per run, and being
//! diffable matters more than being fast.

use serde::{Deserialize, Serialize};

use crate::state::GameData;

/// Bumped when the telemetry schema or manifest layout changes in a way that
/// existing readers cannot absorb. Readers check it and refuse rather than
/// mis-parsing an older run.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub telemetry: TelemetryInfo,
    pub run: RunInfo,
    pub goods: Vec<GoodDim>,
    pub recipes: Vec<RecipeDim>,
    pub regions: Vec<RegionDim>,
    pub market_nodes: Vec<NodeDim>,
    pub channels: Vec<ChannelDim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryInfo {
    /// Filename, relative to the manifest.
    pub file: String,
    pub rows: u64,
    /// Ticks between samples. 1 = every tick.
    pub every: u64,
    /// First and last tick actually sampled, so a reader can tell a subsampled
    /// run from a truncated one.
    pub first_tick: u64,
    pub last_tick: u64,
    /// Which `(entity_kind, metric)` pairs this run actually emitted. A reader
    /// asking for a metric that was never written gets an explicit answer
    /// instead of a silently empty frame.
    pub metrics: Vec<MetricDim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDim {
    pub entity_kind: String,
    pub metric: String,
    /// Whether rows for this metric carry a `good`.
    pub per_good: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInfo {
    pub run_id: String,
    pub git_sha: String,
    pub tape_sha: String,
    /// Hash of the scenario's `criteria.ron`, or `null` when it registers none.
    pub criteria_sha: Option<String>,
    pub seed: u64,
    pub scenario: String,
    pub ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodDim {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDim {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDim {
    pub id: u32,
    pub name: String,
    pub market_node: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDim {
    pub id: u32,
    pub tier: String,
    /// `None` for national and world nodes, which serve no single region.
    pub region: Option<u32>,
    pub currency_good: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDim {
    pub id: u32,
    pub from_node: u32,
    pub to_node: u32,
    pub channel_type: String,
}

impl Manifest {
    /// Build the dimension tables from `game_data`. Identity and telemetry stats
    /// are filled in by the caller once the run is over.
    pub fn new(game_data: &GameData, telemetry: TelemetryInfo, run: RunInfo) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            telemetry,
            run,
            goods: game_data
                .goods
                .iter()
                .map(|g| GoodDim {
                    id: g.id.0,
                    name: g.name.clone(),
                })
                .collect(),
            recipes: game_data
                .recipes
                .iter()
                .enumerate()
                .map(|(i, r)| RecipeDim {
                    id: i as u32,
                    name: r.name.clone(),
                })
                .collect(),
            regions: game_data
                .regions
                .iter()
                .map(|r| RegionDim {
                    id: r.id.0,
                    name: r.name.clone(),
                    market_node: r.market_node.0,
                })
                .collect(),
            market_nodes: game_data
                .market_nodes
                .iter()
                .map(|n| NodeDim {
                    id: n.id.0,
                    tier: format!("{:?}", n.tier),
                    region: n.region.map(|r| r.0),
                    currency_good: n.currency_good.map(|g| g.0),
                })
                .collect(),
            channels: game_data
                .channels
                .iter()
                .map(|c| ChannelDim {
                    id: c.id.0,
                    from_node: c.from.0,
                    to_node: c.to.0,
                    channel_type: format!("{:?}", c.channel_type),
                })
                .collect(),
        }
    }

    pub fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut text = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        text.push('\n');
        std::fs::write(path, text)
    }
}

/// Fingerprint a scenario's `criteria.ron`, matching `certificate::tape_sha`'s
/// FNV-1a so the two hashes are comparable artifacts.
pub fn criteria_sha(dir: &std::path::Path) -> Option<String> {
    let bytes = std::fs::read(dir.join("criteria.ron")).ok()?;
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    Some(format!("{hash:016x}"))
}
