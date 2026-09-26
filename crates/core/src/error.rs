//! Core's error types. Written by hand, each with a `Display` that reads as a ledger or load
//! line; every one is `Send + Sync + 'static` (E3). Nothing in core panics on bad input: a bad
//! delta, tape or checkpoint is one of these, in every build profile (N2).

use crate::clock::Date;
use crate::delta::{Phase, Provenance};
use crate::ids::{ActorId, GoodId, Holder, NodeId, ParamId};
use crate::ledger::{Breach, ShortfallLine};
use crate::units::Unit;
use std::fmt;

/// A delta, state or ledger check that failed. `apply` is atomic per delta: when it returns
/// one of these, nothing of the failing delta has applied.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// A good id outside the world.
    UnknownGood(GoodId),
    /// A node id outside the world.
    UnknownNode(NodeId),
    /// A holder that is not a declared actor, or an escrow of an unknown or currency market.
    UnknownHolder(Holder),
    /// A param id outside the registry.
    UnknownParam(ParamId),
    /// An actor id outside the world (an extension's delta, for example).
    UnknownActor(ActorId),
    /// A book write to a (node, currency) slot: a currency has no market.
    NoMarket {
        /// The node.
        node: NodeId,
        /// The currency.
        good: GoodId,
    },
    /// A value that is not finite, has its sign bit set (`-0.0` included), or is zero where a
    /// positive value is required (a price or an EMA).
    BadValue {
        /// Which value.
        what: &'static str,
        /// The value.
        value: f64,
    },
    /// A delta in a phase that may not carry it (docs/ENGINE.md §2.4).
    WrongPhase {
        /// The phase.
        phase: Phase,
        /// What was out of place.
        what: &'static str,
    },
    /// A `SetParam` on a fixed param.
    FixedParam(ParamId),
    /// A param read as a unit it is not registered with.
    UnitMismatch {
        /// The param.
        param: ParamId,
        /// Its registered unit.
        registered: Unit,
        /// The unit asked for.
        requested: Unit,
    },
    /// A mint or burn with a provenance reserved for core (`Spoilage` belongs to ageing).
    ReservedProvenance(Provenance),
    /// A transfer or burn asked for more than its source held. Nothing of it moved, the line is
    /// in the ledger, and the tick stops (N3).
    Shortfall(ShortfallLine),
    /// The tick did not conserve a good (R2).
    Conservation(Box<Breach>),
    /// An escrow was left between ticks.
    EscrowLeft {
        /// The market's node.
        node: NodeId,
        /// The market's good.
        good: GoodId,
    },
    /// A state or ledger whose shape does not fit the world.
    Shape(String),
    /// A failure an extension (`Ext`) reports.
    Ext(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::UnknownGood(g) => write!(f, "undefined {g}"),
            CoreError::UnknownNode(n) => write!(f, "undefined {n}"),
            CoreError::UnknownHolder(h) => write!(f, "undefined holder {h}"),
            CoreError::UnknownParam(p) => write!(f, "undefined {p}"),
            CoreError::UnknownActor(a) => write!(f, "undefined actor {a}"),
            CoreError::NoMarket { node, good } => {
                write!(f, "{good} is a currency and has no market at {node}")
            }
            CoreError::BadValue { what, value } => write!(
                f,
                "{what} is {value:e}: state values are finite with a clear sign bit"
            ),
            CoreError::WrongPhase { phase, what } => {
                write!(f, "{what} is not allowed in phase {phase}")
            }
            CoreError::FixedParam(p) => write!(f, "{p} is fixed and cannot be set"),
            CoreError::UnitMismatch {
                param,
                registered,
                requested,
            } => write!(
                f,
                "{param} is registered in {registered} and was read as {requested}"
            ),
            CoreError::ReservedProvenance(p) => {
                write!(f, "provenance {p:?} is reserved for core ageing")
            }
            CoreError::Shortfall(line) => write!(f, "{line}"),
            CoreError::Conservation(b) => write!(f, "{b}"),
            CoreError::EscrowLeft { node, good } => {
                write!(
                    f,
                    "the escrow of ({node}, {good}) is not empty between phases"
                )
            }
            CoreError::Shape(s) => write!(f, "shape: {s}"),
            CoreError::Ext(s) => write!(f, "extension: {s}"),
        }
    }
}

impl std::error::Error for CoreError {}

/// A tape that does not load. `path` names where, for example
/// `actors[mill].spec.buy[village/grain].qty`; errors that come from the RON parser itself (a
/// syntax error, an unknown or missing field, a malformed key or date) have no path, and their
/// message carries the parser's line and column instead.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadError {
    /// Where in the tape.
    pub path: String,
    /// What is wrong.
    pub kind: LoadErrorKind,
}

impl LoadError {
    /// A load error at `path`.
    pub fn new(path: impl Into<String>, kind: LoadErrorKind) -> LoadError {
        LoadError {
            path: path.into(),
            kind,
        }
    }
}

/// What is wrong with a tape (docs/ENGINE.md §2.6's list).
#[derive(Debug, Clone, PartialEq)]
pub enum LoadErrorKind {
    /// The RON parser rejected the text: a syntax error, an unknown field, a missing field, a
    /// malformed key or date. The message includes the line and column.
    Parse(String),
    /// The tape's schema version is not the one this loader reads.
    Schema {
        /// The tape's version.
        found: u32,
        /// The loader's version.
        expected: u32,
    },
    /// A key that names nothing of its kind.
    Unknown {
        /// The kind looked in.
        kind: &'static str,
        /// The key.
        key: String,
    },
    /// Two entries of one kind (or of one shared namespace) with the same key.
    Duplicate {
        /// The kind.
        kind: &'static str,
        /// The key.
        key: String,
    },
    /// A (node, non-currency good) without a genesis price.
    MissingPrice,
    /// A genesis price on a currency, which has no market.
    PriceOnCurrency,
    /// A number that is not finite, has its sign bit set, or is zero where a positive one is
    /// required.
    BadValue(f64),
    /// A param referenced with a unit other than its registered one.
    UnitMismatch {
        /// The param's key.
        key: String,
        /// Its registered unit.
        registered: Unit,
        /// The unit the reference needs.
        expected: Unit,
    },
    /// A registered param nothing references.
    UnusedParam,
    /// A `SetParam` whose target is fixed.
    SetParamOnFixed {
        /// The target's key.
        key: String,
    },
    /// A `SetParam` whose source has another unit than its target.
    SetParamAcrossUnits {
        /// The target's unit.
        param: Unit,
        /// The source's unit.
        to: Unit,
    },
    /// An event dated before the start.
    EventBeforeStart {
        /// The event's date.
        date: Date,
        /// The start.
        start: Date,
    },
    /// A recurring entry whose last date is before its first.
    LastBeforeFirst,
    /// An order line on a currency.
    CurrencyOrder,
    /// A currency on either side of a recipe: every cost is a good (R14), and money is made
    /// and destroyed only by tape events.
    CurrencyInRecipe,
    /// A ledger tolerance of 1 or more, which would pass a leak of the whole stock or flow and
    /// so switch conservation off (R2).
    ToleranceNotBelowOne(f64),
    /// A currency that is not `Indefinite` or has a `price_rate`.
    CurrencyGood,
    /// A non-currency good without a `price_rate`.
    NoPriceRate,
    /// A good on both sides of a recipe.
    GoodOnBothSides {
        /// The good's key.
        good: String,
    },
    /// A life or period that rounds to 0 ticks.
    ZeroTicks {
        /// The value in years.
        years: f64,
    },
    /// `Ratio` with `Saturate`, which does not load (docs/ENGINE.md §3.3).
    RatioWithSaturate,
    /// Weights whose left fold in canonical order is not exactly 1.
    WeightsNotOne {
        /// The fold.
        sum: f64,
    },
    /// Anything else, described.
    Invalid(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{}: {}", self.path, self.kind)
        }
    }
}

impl fmt::Display for LoadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoadErrorKind::*;
        match self {
            Parse(m) => write!(f, "{m}"),
            Schema { found, expected } => write!(
                f,
                "tape schema {found}, but this loader reads schema {expected} only"
            ),
            Unknown { kind, key } => write!(f, "no {kind} has the key {key:?}"),
            Duplicate { kind, key } => write!(f, "two {kind} entries have the key {key:?}"),
            MissingPrice => write!(f, "no genesis price"),
            PriceOnCurrency => write!(f, "a currency has no market and takes no genesis price"),
            BadValue(v) => write!(
                f,
                "{v:e} is not allowed: values are finite with a clear sign bit"
            ),
            UnitMismatch {
                key,
                registered,
                expected,
            } => write!(
                f,
                "{key} is registered in {registered}, but used as {expected}"
            ),
            UnusedParam => write!(f, "the param is registered but nothing references it"),
            SetParamOnFixed { key } => write!(f, "{key} is fixed and cannot be set"),
            SetParamAcrossUnits { param, to } => {
                write!(f, "cannot set a {param} param from a {to} param")
            }
            EventBeforeStart { date, start } => {
                write!(f, "{date} is before the start, {start}")
            }
            LastBeforeFirst => write!(f, "the last date is before the first"),
            CurrencyOrder => write!(f, "an order line on a currency"),
            CurrencyInRecipe => write!(
                f,
                "a currency cannot be a recipe input or output: costs are goods (R14)"
            ),
            ToleranceNotBelowOne(v) => write!(
                f,
                "a ledger tolerance of {v:e} is not below 1: it would pass a leak of the whole \
                 stock or flow"
            ),
            CurrencyGood => write!(f, "a currency must be Indefinite and have no price_rate"),
            NoPriceRate => write!(f, "a non-currency good needs a price_rate"),
            GoodOnBothSides { good } => write!(f, "{good} is on both sides of the recipe"),
            ZeroTicks { years } => write!(f, "{years:e} years rounds to 0 ticks"),
            RatioWithSaturate => write!(f, "the Ratio rule cannot be used with Saturate"),
            WeightsNotOne { sum } => write!(
                f,
                "weights fold to {sum:e}, not exactly 1 (add them in canonical order)"
            ),
            Invalid(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// A checkpoint that cannot be read.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckpointError {
    /// The bytes do not open with the checkpoint magic.
    Magic,
    /// The checkpoint's format is not the one this build reads. Checked before the state.
    Format {
        /// The checkpoint's format.
        found: u32,
        /// This build's.
        expected: u32,
    },
    /// The bytes or text do not decode, or hold a value core rejects (NaN, `-0.0`, a negative
    /// quantity, a book of the wrong shape).
    Decode(String),
    /// The decoded fields (`world_id`, `prefix_id`, the state and the run's ledger) do not hash
    /// to the digest stored with them: the checkpoint was edited or corrupted after it was saved.
    Digest {
        /// The digest the checkpoint carries.
        stored: u64,
        /// The hash of the fields it carries.
        computed: u64,
    },
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointError::Magic => write!(f, "not a rustyecon checkpoint (bad magic)"),
            CheckpointError::Format { found, expected } => write!(
                f,
                "checkpoint format {found}, but this build reads format {expected} only"
            ),
            CheckpointError::Decode(m) => write!(f, "checkpoint does not decode: {m}"),
            CheckpointError::Digest { stored, computed } => write!(
                f,
                "the checkpoint hashes to 0x{computed:016x}, not its digest 0x{stored:016x}: \
                 it was edited or corrupted after it was saved"
            ),
        }
    }
}

impl std::error::Error for CheckpointError {}
