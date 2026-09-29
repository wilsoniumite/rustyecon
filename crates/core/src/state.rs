//! The simulation state (docs/ENGINE.md §2.3), salvaged from `v2p3: state/sim_state.rs` with
//! the v1 and July agent structs taken out (N14) and no default price anywhere: every market's
//! genesis price comes from the tape, and its genesis EMA is that price.
//!
//! Its fields are private to core, readers are public, and [`crate::apply()`] is the only writer.

use crate::error::CoreError;
use crate::ext::Ext;
use crate::ids::{GoodId, Holder, NodeId, ParamId};
use crate::inventory::Inventory;
use crate::num::is_clean;
use crate::registry::{Params, Registry};
use crate::world::{Life, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Price, EMA and cleared volumes per (node, good), flat at `node·n_goods + good`. A currency's
/// slots hold price 1 and EMA 1 by definition and volumes 0, and never change; so do an untraded
/// good's (amended at P2.2b.1), since it has no market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BookRepr")]
pub struct MarketBook {
    n_goods: u32,
    price: Vec<f64>,
    ema: Vec<f64>,
    supply: Vec<f64>,
    demand: Vec<f64>,
}

#[derive(Deserialize)]
struct BookRepr {
    n_goods: u32,
    price: Vec<f64>,
    ema: Vec<f64>,
    supply: Vec<f64>,
    demand: Vec<f64>,
}

impl TryFrom<BookRepr> for MarketBook {
    type Error = String;

    /// Loading rejects a book whose columns differ in length, and any value that is not finite
    /// or has its sign bit set, or a price or EMA that is not positive.
    fn try_from(r: BookRepr) -> Result<MarketBook, String> {
        let n = r.price.len();
        if r.ema.len() != n || r.supply.len() != n || r.demand.len() != n {
            return Err("book columns differ in length".into());
        }
        if (r.n_goods == 0 && n != 0) || (r.n_goods != 0 && !n.is_multiple_of(r.n_goods as usize)) {
            return Err("book length is not a multiple of its goods".into());
        }
        for (what, col, positive) in [
            ("price", &r.price, true),
            ("ema", &r.ema, true),
            ("supply", &r.supply, false),
            ("demand", &r.demand, false),
        ] {
            if let Some(v) = col
                .iter()
                .find(|&&v| !is_clean(v) || (positive && v == 0.0))
            {
                return Err(format!("book {what} {v:e} is not allowed"));
            }
        }
        Ok(MarketBook {
            n_goods: r.n_goods,
            price: r.price,
            ema: r.ema,
            supply: r.supply,
            demand: r.demand,
        })
    }
}

/// One market's quote.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quote {
    /// The posted price.
    pub price: f64,
    /// The price's EMA.
    pub ema: f64,
    /// Last clearing's supply, `S`.
    pub supply: f64,
    /// Last clearing's feasible demand, `D`.
    pub demand: f64,
}

impl MarketBook {
    pub(crate) fn new(n_nodes: usize, n_goods: usize) -> MarketBook {
        let len = n_nodes * n_goods;
        MarketBook {
            n_goods: u32::try_from(n_goods).unwrap_or(u32::MAX),
            price: vec![1.0; len],
            ema: vec![1.0; len],
            supply: vec![0.0; len],
            demand: vec![0.0; len],
        }
    }

    pub(crate) fn index(&self, n: NodeId, g: GoodId) -> Option<usize> {
        if g.0 >= self.n_goods {
            return None;
        }
        let i = n
            .idx()
            .checked_mul(self.n_goods as usize)?
            .checked_add(g.idx())?;
        (i < self.price.len()).then_some(i)
    }

    /// One market's quote.
    pub fn quote(&self, n: NodeId, g: GoodId) -> Option<Quote> {
        let i = self.index(n, g)?;
        Some(Quote {
            price: self.price[i],
            ema: self.ema[i],
            supply: self.supply[i],
            demand: self.demand[i],
        })
    }

    pub(crate) fn price(&self, i: usize) -> f64 {
        self.price[i]
    }

    pub(crate) fn set_price(&mut self, i: usize, v: f64) {
        self.price[i] = v;
    }

    pub(crate) fn set_ema(&mut self, i: usize, v: f64) {
        self.ema[i] = v;
    }

    pub(crate) fn set_volumes(&mut self, i: usize, supply: f64, demand: f64) {
        self.supply[i] = supply;
        self.demand[i] = demand;
    }

    fn slots(&self) -> usize {
        self.price.len()
    }
}

/// Deserialise a list of values, rejecting any that is not finite or has its sign bit set.
fn clean_values<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<f64>, D::Error> {
    let values = Vec::<f64>::deserialize(d)?;
    match values.iter().find(|&&v| !is_clean(v)) {
        Some(v) => Err(serde::de::Error::custom(format!(
            "param value {v:e} is not allowed"
        ))),
        None => Ok(values),
    }
}

/// The complete state of a run at one tick. Everything in it is hashed and checkpointed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct SimState<E: Ext> {
    pub(crate) tick: u64,
    /// The current value per [`ParamId`].
    #[serde(deserialize_with = "clean_values")]
    pub(crate) params: Vec<f64>,
    pub(crate) book: MarketBook,
    /// Every declared actor, even with nothing; an escrow only inside phase 3.
    pub(crate) holdings: BTreeMap<Holder, Inventory>,
    pub(crate) ext: E::State,
}

impl<E: Ext> SimState<E> {
    pub(crate) fn genesis(
        params: Vec<f64>,
        book: MarketBook,
        holdings: BTreeMap<Holder, Inventory>,
        ext: E::State,
    ) -> SimState<E> {
        SimState {
            tick: 0,
            params,
            book,
            holdings,
            ext,
        }
    }

    /// The tick this state is at: tick `t` runs on the state whose tick is `t`.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// A param's current value.
    pub fn param(&self, p: ParamId) -> Option<f64> {
        self.params.get(p.idx()).copied()
    }

    /// The current params, typed by unit through the world's registry.
    pub fn params<'a>(&'a self, registry: &'a Registry) -> Params<'a> {
        Params::new(&self.params, registry)
    }

    /// The current value of every param, by id.
    pub fn param_values(&self) -> &[f64] {
        &self.params
    }

    /// The market book.
    pub fn book(&self) -> &MarketBook {
        &self.book
    }

    /// A market's posted price.
    pub fn price(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.book.quote(n, g).map(|q| q.price)
    }

    /// A market's EMA.
    pub fn ema(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.book.quote(n, g).map(|q| q.ema)
    }

    /// A market's last supply, `S`.
    pub fn supply(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.book.quote(n, g).map(|q| q.supply)
    }

    /// A market's last feasible demand, `D`.
    pub fn demand(&self, n: NodeId, g: GoodId) -> Option<f64> {
        self.book.quote(n, g).map(|q| q.demand)
    }

    /// One holder's inventory. Every declared actor has one.
    pub fn holding(&self, h: Holder) -> Option<&Inventory> {
        self.holdings.get(&h)
    }

    /// Every holding, in [`Holder`] order.
    pub fn holdings(&self) -> &BTreeMap<Holder, Inventory> {
        &self.holdings
    }

    /// The extension state.
    pub fn ext(&self) -> &E::State {
        &self.ext
    }

    /// Check a state (a loaded checkpoint's, say) against a world: params, book and holdings
    /// have the world's shape; every declared actor holds an inventory and nothing else does (no
    /// escrow between ticks); every good is in range; every lot's life fits its good; currency
    /// slots are untouched; and the extension state passes [`Ext::validate`]. Values are
    /// already finite with clear sign bits, which deserialisation checks.
    pub fn validate(&self, w: &World<E>) -> Result<(), CoreError> {
        let shape = |m: String| Err(CoreError::Shape(m));
        if self.params.len() != w.registry.len() {
            return shape(format!(
                "{} param values for {} registered params",
                self.params.len(),
                w.registry.len()
            ));
        }
        if let Some(&v) = self.params.iter().find(|&&v| !is_clean(v)) {
            return Err(CoreError::BadValue {
                what: "param value",
                value: v,
            });
        }
        if self.book.n_goods as usize != w.n_goods()
            || self.book.slots() != w.n_goods() * w.n_nodes()
        {
            return shape(format!(
                "a book of {} slots over {} goods, for {} nodes and {} goods",
                self.book.slots(),
                self.book.n_goods,
                w.n_nodes(),
                w.n_goods()
            ));
        }
        for n in &w.nodes {
            for g in &w.goods {
                let q = self
                    .book
                    .quote(n.id, g.id)
                    .ok_or(CoreError::UnknownGood(g.id))?;
                if !w.has_market(g.id)
                    && (q.price != 1.0 || q.ema != 1.0 || q.supply != 0.0 || q.demand != 0.0)
                {
                    let what = if w.is_currency(g.id) {
                        "currency"
                    } else {
                        "untraded"
                    };
                    return shape(format!(
                        "the {what} slot of ({}, {}) was written",
                        n.key, g.key
                    ));
                }
            }
        }
        for decl in &w.actors {
            if !self.holdings.contains_key(&Holder::Actor(decl.id)) {
                return shape(format!("declared actor {} holds no inventory", decl.key));
            }
        }
        for (h, inv) in &self.holdings {
            match h {
                Holder::Actor(a) if w.actor(*a).is_some() => {}
                Holder::Actor(_) => return Err(CoreError::UnknownHolder(*h)),
                Holder::Escrow(n, g) => return Err(CoreError::EscrowLeft { node: *n, good: *g }),
            }
            for (g, _) in inv.goods() {
                let good = w.good(g).ok_or(CoreError::UnknownGood(g))?;
                let fits = |life: Option<u32>| match good.life {
                    Life::Indefinite => life.is_none(),
                    Life::Instant => life == Some(0),
                    Life::Ticks(l) => life.is_some_and(|n| n <= l),
                };
                if let Some(lot) = inv.lots(g).iter().find(|lot| !fits(lot.life)) {
                    return shape(format!(
                        "{h} holds {} with life {:?}, which {} cannot have",
                        lot.qty, lot.life, good.key
                    ));
                }
            }
        }
        E::validate(&self.ext, &w.actors)
    }
}
