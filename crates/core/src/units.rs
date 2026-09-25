//! Time units (docs/ENGINE.md §2.1 and §6, ADDENDUM A13).
//!
//! Every registered param carries a [`Unit`], and every reader asks for one of the typed
//! newtypes below through [`crate::Params::get`], which checks the registered unit. The
//! conversions to per-tick values live in [`crate::Clock`], whose methods each take one newtype,
//! so a rate used as a flow does not compile.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The unit a param is registered with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Unit {
    /// A ratio, weight or tolerance. No time dimension.
    Dimensionless,
    /// A span of time in years: an EMA time constant, a period or a shelf life.
    Years,
    /// A flow of goods per year (a capacity or an order line).
    FlowPerYear,
    /// A continuous rate per year: a draw on a stock, or a price rate.
    RatePerYear,
    /// An effective annual rate, compounded.
    CompoundPerYear,
    /// A fraction of a stock lost per year (depreciation, for example).
    FractionPerYear,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A typed param value. One newtype per [`Unit`]; [`crate::Params::get`] returns one after
/// checking that the param was registered with that unit.
pub trait UnitKind: Copy + fmt::Debug + sealed::Sealed {
    /// The unit this type stands for.
    const UNIT: Unit;
    /// Wrap a raw value.
    fn from_value(v: f64) -> Self;
    /// The raw value, in this unit.
    fn value(self) -> f64;
}

macro_rules! unit_kind {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name(pub f64);

        impl sealed::Sealed for $name {}

        impl UnitKind for $name {
            const UNIT: Unit = Unit::$name;
            fn from_value(v: f64) -> Self {
                $name(v)
            }
            fn value(self) -> f64 {
                self.0
            }
        }
    };
}

unit_kind!(
    /// A dimensionless value: a ratio, weight or tolerance.
    Dimensionless
);
unit_kind!(
    /// A span of time in years.
    Years
);
unit_kind!(
    /// A flow of goods per year.
    FlowPerYear
);
unit_kind!(
    /// A continuous rate per year.
    RatePerYear
);
unit_kind!(
    /// An effective annual rate.
    CompoundPerYear
);
unit_kind!(
    /// An annual fraction.
    FractionPerYear
);
