//! The tape's raw schema, version 1: what a tape file holds, before resolution
//! (docs/ENGINE.md §2.6 and §5; `docs/TAPE.md`).
//!
//! Every field is required and none has a default: nothing here carries `#[serde(default)]`,
//! and each `Option` field goes through a `deserialize_with` helper so that serde treats it as
//! required too (it would otherwise read a missing `Option` as `None`). Unknown fields are
//! rejected. Keyed data is always a list of entries carrying a `key`, never a RON map, and any
//! list may be in any order: the loader sorts it. Units are named on each field; a key that
//! names a param says which unit that param must be registered with.
#![deny(missing_docs)]

use crate::clock::Date;
use crate::ids::{ActorKind, Key};
use crate::inventory::Amount;
use crate::registry::Basis;
use crate::units::Unit;
use crate::world::{OneSided, PriceRule};
use serde::{Deserialize, Deserializer, Serialize};

/// Read an `Option` field that must be written, as `None` or `Some(..)`: use it as
/// `#[serde(deserialize_with = "required")]` on every `Option` field of a raw type, including an
/// extension's, since serde would otherwise read a missing `Option` as `None`.
pub fn required<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

/// The tape's header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHeader {
    /// `String`. The tape's name. Required, no default. Not part of the world's identity.
    pub name: String,
    /// `Date`, `"YYYY-MM-DD"`. The date of tick 0. Required, no default.
    pub start: Date,
    /// `u32`, ticks per year, at least 1. The registered tick length (A13). Required, no
    /// default.
    pub ticks_per_year: u32,
    /// The market settings. Required, no default.
    pub market: RawMarket,
    /// The ledger's tolerances. Required, no default.
    pub ledger: RawLedger,
}

/// The market settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMarket {
    /// `Imbalance` or `Ratio`. The price rule (N13). Required, no default.
    pub rule: PriceRule,
    /// `Saturate` or `Hold`. What a one-sided market does to its price (F8). Required, no
    /// default. `Ratio` with `Saturate` does not load.
    pub one_sided: OneSided,
    /// Param key, unit `Years`, live. The EMA's time constant. Required, no default.
    pub ema_time_constant: Key,
}

/// The ledger's tolerances (A12): both relative, with no absolute term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLedger {
    /// Param key, unit `Dimensionless`, fixed. Slack per unit of gross flow. Required, no
    /// default.
    pub rel_flow: Key,
    /// Param key, unit `Dimensionless`, fixed. Slack per unit of stock. Required, no default.
    pub rel_stock: Key,
}

/// A registered param: one dial (R4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawParam {
    /// `Key`. Unique among params. Required, no default.
    pub key: Key,
    /// `f64`, in `unit`, finite and not negative. The genesis value. Required, no default.
    pub value: f64,
    /// `Unit`. Required, no default.
    pub unit: Unit,
    /// `Basis`. Where the value comes from (R4). Required, no default. Not part of the world's
    /// identity.
    pub basis: Basis,
}

/// A good.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGood {
    /// `Key`. Unique among goods. Required, no default.
    pub key: Key,
    /// `RawLife`. The shelf life. Required, no default. A currency must be `Indefinite`.
    pub life: RawLife,
    /// `Option<Key>`, a param of unit `RatePerYear`, live. The rate the good's price moves at.
    /// `None` exactly for a currency. Required (write `None` or `Some(..)`), no default.
    #[serde(deserialize_with = "required")]
    pub price_rate: Option<Key>,
}

/// A good's shelf life.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawLife {
    /// Never spoils.
    Indefinite,
    /// Dies at the ageing of the tick it was minted in.
    Instant,
    /// Param key, unit `Years`, fixed: the life, rounded to whole ticks; 0 ticks does not load.
    Years(Key),
}

/// A market node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawNode {
    /// `Key`. Unique among nodes. Required, no default.
    pub key: Key,
    /// Good key. The good prices are quoted in; it becomes a currency. Required, no default.
    pub currency: Key,
}

/// A static channel between two nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawChannel {
    /// `Key`. Unique among channels. Required, no default.
    pub key: Key,
    /// Node key. Required, no default.
    pub from: Key,
    /// Node key, not `from`. Required, no default.
    pub to: Key,
}

/// A declared actor. `A` is the extension's spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawActorEntry<A> {
    /// `Key`. Unique among actors; desks and pops share one namespace. Required, no default.
    pub key: Key,
    /// `Desk` or `Pop`. Required, no default.
    pub kind: ActorKind,
    /// Class key. Required, no default.
    pub class: Key,
    /// Node key. The actor's home. Required, no default.
    pub home: Key,
    /// `Basis` of the spec's inline numbers. Required, no default.
    pub basis: Basis,
    /// The extension's spec. Required, no default.
    pub spec: A,
}

/// The genesis state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGenesis {
    /// `Basis` of the genesis prices and stocks. Required, no default.
    pub basis: Basis,
    /// Exactly one price per (node, non-currency good). Required, no default.
    pub prices: Vec<RawPrice>,
    /// Holdings per actor; an actor not listed starts empty. Required, no default.
    pub holdings: Vec<RawHolding>,
}

/// One genesis price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPrice {
    /// Node key. Required, no default.
    pub node: Key,
    /// Good key, not a currency. Required, no default.
    pub good: Key,
    /// `f64`, in the node's currency per unit, finite and positive. The genesis EMA is this
    /// price. Required, no default.
    pub price: f64,
}

/// One actor's genesis holding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHolding {
    /// Actor key. Required, no default.
    pub holder: Key,
    /// `(good key, quantity)` pairs; each quantity is in units of its good, finite and not
    /// negative, and a perishable good starts as one lot with its full life. Required, no
    /// default.
    pub goods: Vec<(Key, f64)>,
}

/// A dated event. `A` is the extension's tape action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEvent<A> {
    /// `Key`. Unique among events and recurring entries together. Required, no default.
    pub key: Key,
    /// `Date`, not before `start`. It fires in the tick the date falls in. Required, no
    /// default.
    pub at: Date,
    /// `Basis` of the action's inline numbers. Required, no default.
    pub basis: Basis,
    /// The action. Required, no default.
    pub act: RawAct<A>,
}

/// A recurring entry, firing at `first`'s tick plus every whole `every` period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRecurring<A> {
    /// `Key`. Unique among events and recurring entries together. Required, no default.
    pub key: Key,
    /// `Date`, not before `start`. The first firing. Required, no default.
    pub first: Date,
    /// Param key, unit `Years`, fixed. The period, rounded to whole ticks; 0 ticks does not
    /// load. Required, no default.
    pub every: Key,
    /// `Option<Date>`, not before `first`. No firing after it. Required (write `None` or
    /// `Some(..)`), no default.
    #[serde(deserialize_with = "required")]
    pub last: Option<Date>,
    /// `Basis` of the action's inline numbers. Required, no default.
    pub basis: Basis,
    /// The action. Required, no default.
    pub act: RawAct<A>,
}

/// A tape action. `A` is the extension's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RawAct<A> {
    /// Create goods in an actor's holding, with provenance `Event`.
    Mint {
        /// Actor key. Required, no default.
        holder: Key,
        /// Good key. Required, no default.
        good: Key,
        /// `f64`, units of the good, finite and not negative. Required, no default.
        qty: f64,
    },
    /// Destroy goods in an actor's holding, with provenance `Event`. A shortfall stops the run.
    Burn {
        /// Actor key. Required, no default.
        holder: Key,
        /// Good key. Required, no default.
        good: Key,
        /// `Qty(f64)` (units of the good, finite and not negative) or `All`. Required, no
        /// default.
        amount: Amount,
    },
    /// Move goods between actors. A shortfall stops the run.
    Transfer {
        /// Actor key. Required, no default.
        from: Key,
        /// Actor key. Required, no default.
        to: Key,
        /// Good key. Required, no default.
        good: Key,
        /// `Qty(f64)` (units of the good, finite and not negative) or `All`. Required, no
        /// default.
        amount: Amount,
    },
    /// Set a live param to another registered param's value, so the new value keeps a basis.
    SetParam {
        /// Param key, not fixed. Required, no default.
        param: Key,
        /// Param key, the same unit as `param`; fixed by this use. Required, no default.
        to: Key,
    },
    /// The extension's action.
    Actor(A),
}
