//! Shared fixtures for the markets tests: the fixture tape and its variants, ids by key, order
//! builders, and the market phases of one tick on a world with no behaviour.

// Each test file is its own crate and uses part of this module.
#![allow(dead_code)]

use rustyecon_core::{
    apply, resolve, state_hash, ActorId, ClassId, CoreError, GoodId, Holder, Ledger, LoadError,
    NoExt, NodeId, Phase, SimState, StateDelta, Tape, TickAudit, World,
};
use rustyecon_markets::{
    admit, clear, settle, update_prices, Fills, Line, Order, OrderError, PriceError, RationLine,
    SettleLine, Side,
};

/// The fixture tape.
pub const FIXTURE: &str = include_str!("../../testdata/markets.ron");

pub type W = World<NoExt>;
pub type S = SimState<NoExt>;
pub type Delta = StateDelta<NoExt>;

/// Load a tape text, expecting it to resolve.
pub fn load_text(text: &str) -> (W, S) {
    try_load(text).expect("the tape loads")
}

/// Load a tape text.
pub fn try_load(text: &str) -> Result<(W, S), LoadError> {
    resolve(&Tape::<NoExt>::from_ron(text)?)
}

/// The fixture.
pub fn load() -> (W, S) {
    load_text(FIXTURE)
}

/// Replace exactly one occurrence of `from` in `text`.
pub fn edit_text(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "{from:?} must occur once");
    text.replacen(from, to, 1)
}

/// Replace exactly one occurrence of `from` in the fixture.
pub fn edit(from: &str, to: &str) -> String {
    edit_text(FIXTURE, from, to)
}

/// The fixture with its genesis holdings replaced by `holdings`, the body of the list.
pub fn with_holdings(text: &str, holdings: &str) -> String {
    let start = text.find("holdings: [").expect("a holdings list");
    let end = start + text[start..].find("\n        ],").expect("the list's end");
    format!("{}holdings: [{holdings}{}", &text[..start], &text[end..])
}

pub fn actor(w: &W, key: &str) -> ActorId {
    w.id_of::<ActorId>(key).expect("a fixture actor")
}

pub fn holder(w: &W, key: &str) -> Holder {
    Holder::Actor(actor(w, key))
}

pub fn good(w: &W, key: &str) -> GoodId {
    w.id_of::<GoodId>(key).expect("a fixture good")
}

pub fn node(w: &W, key: &str) -> NodeId {
    w.id_of::<NodeId>(key).expect("a fixture node")
}

pub fn class(w: &W, key: &str) -> ClassId {
    w.id_of::<ClassId>(key).expect("a fixture class")
}

/// An actor's registered class.
pub fn class_of(w: &W, a: ActorId) -> ClassId {
    w.actor(a).expect("a declared actor").class
}

/// A buy of `qty` of `what` at `at`, with `budget` in the node's currency.
pub fn buy(w: &W, who: &str, at: &str, what: &str, qty: f64, budget: f64) -> Order {
    let a = actor(w, who);
    Order {
        actor: a,
        class: class_of(w, a),
        node: node(w, at),
        good: good(w, what),
        qty,
        side: Side::Buy { budget },
    }
}

/// A sell of `qty` of `what` at `at`.
pub fn sell(w: &W, who: &str, at: &str, what: &str, qty: f64) -> Order {
    let a = actor(w, who);
    Order {
        actor: a,
        class: class_of(w, a),
        node: node(w, at),
        good: good(w, what),
        qty,
        side: Side::Sell,
    }
}

/// What `h` holds of `g`.
pub fn held(s: &S, h: Holder, g: GoodId) -> f64 {
    s.holding(h).map_or(0.0, |inv| inv.get(g))
}

/// Apply deltas in one phase under a throwaway ledger, for setting up a state.
pub fn apply_in(s: &mut S, w: &W, phase: Phase, ds: &[Delta]) -> Result<Vec<f64>, CoreError> {
    let mut l = Ledger::open(s, w)?;
    apply(s, w, phase, ds, &mut l)
}

/// Post a market's price directly, as phase 6 would.
pub fn set_price(s: &mut S, w: &W, n: NodeId, g: GoodId, price: f64) {
    apply_in(
        s,
        w,
        Phase::Prices,
        &[StateDelta::SetPrice {
            node: n,
            good: g,
            price,
        }],
    )
    .expect("the price is set");
}

/// Record a market's volumes directly, as phase 2 would.
pub fn set_volumes(s: &mut S, w: &W, n: NodeId, g: GoodId, supply: f64, demand: f64) {
    let d = StateDelta::SetVolumes {
        node: n,
        good: g,
        supply,
        demand,
    };
    apply_in(s, w, Phase::Clearing, &[d]).expect("the volumes are set");
}

/// Phase 6 alone: update the prices, apply them and advance the tick. Returns the deltas.
pub fn price_step(s: &mut S, w: &W) -> Result<Vec<Delta>, PriceError> {
    let mut ds = update_prices(s, w)?;
    let out = ds.clone();
    ds.push(StateDelta::AdvanceTick);
    apply_in(s, w, Phase::Prices, &ds).expect("the price deltas apply");
    Ok(out)
}

/// Why a market tick stopped.
#[derive(Debug)]
pub enum Failure {
    Order(OrderError),
    Core(CoreError),
    Price(PriceError),
}

/// What one market tick produced.
#[derive(Debug)]
pub struct Tick {
    pub lines: Vec<Line>,
    pub volumes: Vec<Delta>,
    pub fills: Fills,
    pub plan: Vec<Delta>,
    pub moved: Vec<f64>,
    pub settled: Vec<SettleLine>,
    pub rationing: Vec<RationLine>,
    pub prices: Vec<Delta>,
    pub audit: TickAudit,
    /// Whether an escrow was left right after phase 3.
    pub escrow_left: bool,
    pub hash: u64,
}

/// The market phases of one tick under one ledger, as the engine runs them: admit and clear
/// (phase 2), settle, apply and realize (phase 3), update the prices and advance (phase 6), and
/// close the ledger.
pub fn market_tick(s: &mut S, w: &W, orders: Vec<Order>) -> Result<Tick, Failure> {
    let mut l = Ledger::open(s, w).map_err(Failure::Core)?;
    let lines = admit(orders, s, w).map_err(Failure::Order)?;
    let (volumes, fills) = clear(&lines, w).map_err(Failure::Order)?;
    apply(s, w, Phase::Clearing, &volumes, &mut l).map_err(Failure::Core)?;
    let plan = settle(&lines, &fills, s, w).map_err(Failure::Order)?;
    let deltas = plan.deltas().to_vec();
    let moved = apply(s, w, Phase::Settlement, &deltas, &mut l).map_err(Failure::Core)?;
    let escrow_left = s.holdings().keys().any(|h| matches!(h, Holder::Escrow(..)));
    let (settled, rationing) = plan.realize(&moved).map_err(Failure::Order)?;
    let prices = update_prices(s, w).map_err(Failure::Price)?;
    let mut last = prices.clone();
    last.push(StateDelta::AdvanceTick);
    apply(s, w, Phase::Prices, &last, &mut l).map_err(Failure::Core)?;
    let audit = l.close(s, w).map_err(Failure::Core)?;
    Ok(Tick {
        lines,
        volumes,
        fills,
        plan: deltas,
        moved,
        settled,
        rationing,
        prices,
        audit,
        escrow_left,
        hash: state_hash(s),
    })
}

/// The settle line of `a` on `side` for (`n`, `g`).
pub fn line_of(t: &Tick, a: ActorId, n: NodeId, g: GoodId, buy: bool) -> &SettleLine {
    use rustyecon_markets::SideTag;
    let side = if buy { SideTag::Buy } else { SideTag::Sell };
    t.settled
        .iter()
        .find(|l| l.actor == a && l.node == n && l.good == g && l.side == side)
        .expect("a settle line")
}
