//! The delta set, provenance and phases (docs/ENGINE.md §2.4).
//!
//! Salvaged from `v2p3: types/delta.rs`: its seven core arms, with July's paired Remove/Add
//! replaced by an atomic `Transfer` (it moves the lots it takes, or nothing when the source is
//! short) and by `Mint` and `Burn` with a required provenance, plus `SetParam`, and one
//! extension seam, `Actor`, in place of July's other 19 agent variants (N14). Lots are `f64`, so
//! splitting or merging one can round; `apply` declares what that creates or destroys as a
//! `Rounding` line, so no delta moves a unit without a provenance.

use crate::ext::Ext;
use crate::ids::{GoodId, Holder, NodeId, ParamId};
use crate::inventory::Amount;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The phases of a tick, in order (PLAN §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Phase {
    /// 0: the tape's dated and recurring events fire.
    Events,
    /// 1: actors decide: orders, payouts and Instant endowments.
    Decisions,
    /// 2: orders are admitted and markets clear.
    Clearing,
    /// 3: trades settle through the escrows.
    Settlement,
    /// 4: recipes run.
    Production,
    /// 5: core ageing (5a), then the upkeep hook (5b).
    Upkeep,
    /// 6: prices and EMAs update, and the tick advances.
    Prices,
    /// 7: the ledger closes and the state is hashed.
    Measure,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Phase::Events => "0 (events)",
            Phase::Decisions => "1 (decisions)",
            Phase::Clearing => "2 (clearing)",
            Phase::Settlement => "3 (settlement)",
            Phase::Production => "4 (production)",
            Phase::Upkeep => "5 (upkeep)",
            Phase::Prices => "6 (prices)",
            Phase::Measure => "7 (measure)",
        };
        f.write_str(name)
    }
}

/// Why a mint or burn changes the quantity of a good in existence. Every mint and burn carries
/// one; there is no default, and a transfer carries none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Provenance {
    /// A desk's recipe: inputs burned, outputs minted.
    Production,
    /// A pop's recipe inputs, burned.
    Consumption,
    /// Lots dropped by ageing. Reserved: only `Age` posts it.
    Spoilage,
    /// The output of a recipe with no inputs, and (Phase 2) Instant labour and parcel services.
    Endowment,
    /// Phase 3's capital wearing out. Declared now so the ledger line exists.
    Depreciation,
    /// Phase 3's capital being built. Declared now so the ledger line exists.
    Construction,
    /// A tape mint or burn.
    Event,
    /// What splitting or merging `f64` lots created or destroyed, measured exactly (TwoSum).
    /// Reserved: only `apply` posts it. Last, so the other provenances keep their encoding.
    Rounding,
}

/// Every change the state can undergo. `apply` is the only writer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum StateDelta<E: Ext> {
    /// Post a price for the next tick. Finite and positive. Phase 6 only.
    SetPrice {
        /// The market's node.
        node: NodeId,
        /// The market's good (not a currency).
        good: GoodId,
        /// The new price.
        price: f64,
    },
    /// Set a market's EMA. Finite and positive. Phase 6 only.
    SetEma {
        /// The market's node.
        node: NodeId,
        /// The market's good (not a currency).
        good: GoodId,
        /// The new EMA.
        ema: f64,
    },
    /// Record a market's cleared volumes, `S` and feasible `D`. Phase 2 only.
    SetVolumes {
        /// The market's node.
        node: NodeId,
        /// The market's good (not a currency).
        good: GoodId,
        /// Supply, `S`.
        supply: f64,
        /// Feasible demand, `D`.
        demand: f64,
    },
    /// Move lots, lives kept, from one holder to another. It is all or nothing: it moves the lots
    /// it takes, or nothing when the source is short. Splitting the source's last lot and merging
    /// into the destination's round, creating or destroying up to half an ulp of the larger
    /// operand; `apply` declares that exactly as a `Rounding` line (docs/ENGINE.md §2.4).
    Transfer {
        /// The source.
        from: Holder,
        /// The destination.
        to: Holder,
        /// The good.
        good: GoodId,
        /// How much.
        amount: Amount,
    },
    /// Create a quantity with the good's full life.
    Mint {
        /// The holder it is created in.
        to: Holder,
        /// The good.
        good: GoodId,
        /// How much.
        qty: f64,
        /// Why.
        prov: Provenance,
    },
    /// Destroy a quantity, soonest-expiring lots first. `All` means whatever is there; any
    /// other shortfall stops the tick.
    Burn {
        /// The holder it is destroyed in.
        from: Holder,
        /// The good.
        good: GoodId,
        /// How much.
        amount: Amount,
        /// Why.
        prov: Provenance,
    },
    /// Change a live param's current value. Its unit is the registry's. Phase 0 only.
    SetParam {
        /// The param.
        param: ParamId,
        /// Its new value.
        value: f64,
    },
    /// Age one holder's lots (5a). Phase 5 only.
    Age {
        /// The holder.
        holder: Holder,
    },
    /// An extension's delta.
    Actor(E::Delta),
    /// Advance the tick counter. Phase 6 only, last.
    AdvanceTick,
}

impl<E: Ext> StateDelta<E> {
    /// The variant's name, for error lines.
    pub fn name(&self) -> &'static str {
        match self {
            StateDelta::SetPrice { .. } => "SetPrice",
            StateDelta::SetEma { .. } => "SetEma",
            StateDelta::SetVolumes { .. } => "SetVolumes",
            StateDelta::Transfer { .. } => "Transfer",
            StateDelta::Mint { .. } => "Mint",
            StateDelta::Burn { .. } => "Burn",
            StateDelta::SetParam { .. } => "SetParam",
            StateDelta::Age { .. } => "Age",
            StateDelta::Actor(_) => "Actor",
            StateDelta::AdvanceTick => "AdvanceTick",
        }
    }
}
