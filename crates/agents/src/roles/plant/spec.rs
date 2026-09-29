//! The plant's spec (P2.2b; docs/probe/LOOPS-RULES.md §3.2; LOOP-SPEC §2.2): CAPACITY's plant,
//! y = P^(1−θ)·z^θ over a desk's own Leontief bundle z, carried by the type desk, the maker and
//! the capacity desk as an optional field. The field is left out of the raw and the resolved
//! spec when absent, so every tape written before it keeps its canonical text, `tape_hash` and
//! `world_id` (the maker's `reserve` set the pattern at L0.4).
//!
//! The plant is a good of its own, untraded (core's `untraded: true`): held, minted by its desk in
//! produce and burned at upkeep, never on a market. Every dial is a registered param referenced
//! by key and read at use time (R4, E1): θ and the size s are `Dimensionless` values, the wear
//! δ_p a `FractionPerYear` read as a `Fraction`, and the adjustment s_Kp a `RatePerYear` read as
//! a `Share`.
#![deny(missing_docs)]

use crate::roles::spec::live;
use rustyecon_core::{ClockMethod, GoodId, Key, LoadError, LoadErrorKind, Resolver, Site};
use serde::{Deserialize, Serialize};

/// How a plant is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PlantOrder {
    /// M3's rule: max(u·K\*_p + s_Kp·(K\*_p − P), 0), the target's wear and a share of the gap
    /// (the registered rule, `lm_mirror:163`).
    Target,
    /// max(u·P + (K\*_p − P), 0): the plant's own wear and the whole gap each tick (CAPACITY's
    /// negative control, LN6; `lm_mirror:165`).
    Gap,
}

/// What a plant's target is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PlantTarget {
    /// The herd's plant, (κ_p·κ)·K\*, K\* the capacity desk's target stock (decision 262): a
    /// capacity desk's only.
    Herd,
    /// kr·z, the plant units the bundles planned this tick use at the cheapest ratio (CAPACITY's
    /// K\*; LF6 on the capacity desk, and the only form on the type desk and the maker).
    Bundles,
}

/// A desk's plant, as the tape writes it: `plant: Some((good: "plant.fodder", theta: …, delta:
/// …, size: …, adjust: …, order: Target, target: Bundles))`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPlant {
    /// Good key: the plant, an untraded good no other actor's spec names. Required, no default.
    pub good: Key,
    /// Param key, unit `Dimensionless`, live: θ, the bundles' share, in (0, 1]. Required, no
    /// default.
    pub theta: Key,
    /// Param key, unit `FractionPerYear`, live: δ_p, the plant's wear a year, read as the
    /// fraction u a tick. Required, no default.
    pub delta: Key,
    /// Param key, unit `Dimensionless`, live: s, the bundles of the desk's own recipe one plant
    /// unit takes to build (the generator writes CAPACITY's s1). Required, no default.
    pub size: Key,
    /// Param key, unit `RatePerYear`, live: s_Kp, the share of the plant's gap closed each tick
    /// beyond wear (2·δ_p a year, as M3's). Required, no default.
    pub adjust: Key,
    /// How it is ordered. Required, no default.
    pub order: PlantOrder,
    /// Its target. Required, no default.
    pub target: PlantTarget,
}

/// A resolved plant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plant {
    /// The plant good, untraded.
    pub good: GoodId,
    /// θ, a live `Dimensionless` param.
    pub theta: Site,
    /// δ_p, a live `FractionPerYear` param read as a `Fraction`.
    pub delta: Site,
    /// s, a live `Dimensionless` param.
    pub size: Site,
    /// s_Kp, a live `RatePerYear` param read as a `Share`.
    pub adjust: Site,
    /// How it is ordered.
    pub order: PlantOrder,
    /// Its target.
    pub target: PlantTarget,
}

impl Plant {
    /// Its sites, each with its path under the spec (`plant.…`).
    pub(crate) fn sites(&self, out: &mut Vec<(String, Site)>) {
        out.push(("plant.theta".into(), self.theta));
        out.push(("plant.delta".into(), self.delta));
        out.push(("plant.size".into(), self.size));
        out.push(("plant.adjust".into(), self.adjust));
    }
}

/// Resolve a desk's optional plant under `plant`. Its good must be an untraded good; its params
/// have the units and methods of LOOPS-RULES §3.2, so another unit is a unit mismatch at the same
/// path. The checks that need the genesis values are the cast's (`Cast::new`).
pub fn resolve_plant(
    raw: &Option<RawPlant>,
    r: &mut Resolver<'_>,
) -> Result<Option<Plant>, LoadError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    r.enter("plant");
    let good = r.good(&raw.good, "good")?;
    if !r.is_untraded(good) {
        return Err(r.error(
            "good",
            LoadErrorKind::Invalid(
                "a plant is an untraded good: marked `untraded: true`, with no market".into(),
            ),
        ));
    }
    let theta = live(r, &raw.theta, ClockMethod::Value, "theta")?;
    let delta = live(r, &raw.delta, ClockMethod::Fraction, "delta")?;
    let size = live(r, &raw.size, ClockMethod::Value, "size")?;
    let adjust = live(r, &raw.adjust, ClockMethod::Share, "adjust")?;
    r.leave();
    Ok(Some(Plant {
        good,
        theta,
        delta,
        size,
        adjust,
        order: raw.order,
        target: raw.target,
    }))
}
