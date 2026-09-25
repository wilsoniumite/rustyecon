//! The param registry (docs/ENGINE.md §2.1, R4).
//!
//! Every dial with a time unit, every behavioural rate and every tolerance is a registered param
//! with a unit and a basis. The genesis value, unit and basis live in the `World`; the current
//! value lives in `SimState`, is hashed, and changes only by a dated `SetParam`. Readers read at
//! use time through [`Params`], which checks the registered unit against the type asked for.

use crate::error::CoreError;
use crate::ids::{Key, ParamId};
use crate::units::{Unit, UnitKind};
use serde::{Deserialize, Serialize};

/// Where a number comes from (R4's five tags).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Basis {
    /// Measured, with the source and its vintage.
    Measured {
        /// The source.
        source: String,
        /// The source's vintage.
        vintage: String,
    },
    /// From the literature; the text names the reference.
    Literature(String),
    /// An approximation; the text says of what.
    Approximate(String),
    /// Fitted; `fit` names the fit's id.
    Fitted {
        /// The fit's id.
        fit: String,
    },
    /// Assumed; the text says why.
    Assumed(String),
}

/// One registered param.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamDef {
    /// Its dense id.
    pub id: ParamId,
    /// Its stable key.
    pub key: Key,
    /// Its unit.
    pub unit: Unit,
    /// Its value at genesis.
    pub genesis: f64,
    /// Where the value comes from.
    pub basis: Basis,
    /// Whether the loader turned it into structure (a shelf life, a recurring period, a value a
    /// `SetParam` copies) or it fixes what a past tick meant (a ledger tolerance). A fixed param
    /// cannot be the target of a `SetParam`.
    pub fixed: bool,
}

/// The registered params, by [`ParamId`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Registry {
    params: Vec<ParamDef>,
}

impl Registry {
    pub(crate) fn new(params: Vec<ParamDef>) -> Registry {
        Registry { params }
    }

    /// One param's definition.
    pub fn get(&self, p: ParamId) -> Option<&ParamDef> {
        self.params.get(p.idx())
    }

    /// Every definition, in id (key) order.
    pub fn params(&self) -> &[ParamDef] {
        &self.params
    }

    /// The number of params.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Whether no param is registered.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

/// The current param values, typed by unit. Built from a `SimState` and its world's registry by
/// [`crate::SimState::params`].
#[derive(Debug, Clone, Copy)]
pub struct Params<'a> {
    values: &'a [f64],
    registry: &'a Registry,
}

impl<'a> Params<'a> {
    pub(crate) fn new(values: &'a [f64], registry: &'a Registry) -> Params<'a> {
        Params { values, registry }
    }

    /// The current value of `p` as the unit type `U`. A param registered with another unit is
    /// [`CoreError::UnitMismatch`].
    pub fn get<U: UnitKind>(&self, p: ParamId) -> Result<U, CoreError> {
        let def = self.registry.get(p).ok_or(CoreError::UnknownParam(p))?;
        if def.unit != U::UNIT {
            return Err(CoreError::UnitMismatch {
                param: p,
                registered: def.unit,
                requested: U::UNIT,
            });
        }
        self.value(p).map(U::from_value)
    }

    /// The current value of `p`, untyped, for reports and the registry listing.
    pub fn value(&self, p: ParamId) -> Result<f64, CoreError> {
        self.values
            .get(p.idx())
            .copied()
            .ok_or(CoreError::UnknownParam(p))
    }

    /// The registry the values belong to.
    pub fn registry(&self) -> &'a Registry {
        self.registry
    }
}
