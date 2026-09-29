//! The stock roles (P2.2; docs/probe/HORSES-RULES.md; HORSES-SPEC §2.6–§2.8, §3), on the stocks
//! probe's tapes: H1 and H2 (the capacity desk, the maker and P2.1's fodder desk on rule A), P7
//! (the owner desk holding its own horses) and R1a (the stocks layer off, the flow path).
//!
//! As in `many.rs`, most comparisons are exact. One bar is relative and named here: `CLOSE`,
//! 1e-12 relative, for a stock rule's order at the oracle's point, which the rules compute as a
//! small difference of rounded terms.

use rustyecon_agents::cast::view;
use rustyecon_agents::{
    ActorState, AgentDelta, Agents, CapacityState, Cast, Decision, GoodDeskState, MachDeskState,
    MakerState, OwnerState, RawSpec,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, ClockMethod, CoreError, FractionPerYear, GoodId, Holder,
    Ledger, LoadError, NodeId, ParamId, Phase, Provenance, SimState, StateDelta, Tape, World,
};
use rustyecon_markets::{admit, Order, Side};

const APPB: &str = include_str!("../../../tapes/appb.ron");
const I0: &str = include_str!("../../../tapes/markets-i0.ron");
const H1: &str = include_str!("../../../tapes/horses-h1.ron");
const H2: &str = include_str!("../../../tapes/horses-h2.ron");
const P7: &str = include_str!("../../../tapes/horses-p7.ron");
const R1A: &str = include_str!("../../../tapes/horses-r1a.ron");
const CLOSE: f64 = 1e-12;

type W = World<Agents>;
type S = SimState<Agents>;
type Delta = StateDelta<Agents>;

fn load_text(text: &str) -> Result<(W, S), LoadError> {
    resolve(&Tape::<Agents>::from_ron(text)?)
}

fn load(text: &str) -> (W, S, Cast) {
    let (w, s) = load_text(text).expect("the tape loads");
    let cast = Cast::new(&w).expect("the cast builds");
    (w, s, cast)
}

fn actor(w: &W, key: &str) -> ActorId {
    w.id_of::<ActorId>(key).expect("an actor")
}

fn good(w: &W, key: &str) -> GoodId {
    w.id_of::<GoodId>(key).expect("a good")
}

fn param(w: &W, key: &str) -> ParamId {
    w.id_of::<ParamId>(key).expect("a param")
}

fn home(w: &W) -> NodeId {
    w.id_of::<NodeId>("home").expect("the home node")
}

fn held(s: &S, a: ActorId, g: GoodId) -> f64 {
    s.holding(Holder::Actor(a)).map_or(0.0, |inv| inv.get(g))
}

fn apply_in(s: &mut S, w: &W, phase: Phase, ds: &[Delta]) -> Result<Vec<f64>, CoreError> {
    let mut l = Ledger::open(s, w)?;
    apply(s, w, phase, ds, &mut l)
}

fn set_price(s: &mut S, w: &W, g: GoodId, price: f64) {
    let d = StateDelta::SetPrice {
        node: home(w),
        good: g,
        price,
    };
    apply_in(s, w, Phase::Prices, &[d]).expect("the price applies");
}

fn set_param(s: &mut S, w: &W, key: &str, value: f64) {
    let d = StateDelta::SetParam {
        param: param(w, key),
        value,
    };
    apply_in(s, w, Phase::Events, &[d]).expect("the param applies");
}

fn set_holding(s: &mut S, w: &W, a: ActorId, g: GoodId, q: f64) {
    let mut ds = Vec::new();
    if held(s, a, g) > 0.0 {
        ds.push(StateDelta::Burn {
            from: Holder::Actor(a),
            good: g,
            amount: Amount::All,
            prov: Provenance::Event,
        });
    }
    if q > 0.0 {
        ds.push(StateDelta::Mint {
            to: Holder::Actor(a),
            good: g,
            qty: q,
            prov: Provenance::Event,
        });
    }
    apply_in(s, w, Phase::Events, &ds).expect("the holding applies");
}

fn set_state(s: &mut S, w: &W, a: ActorId, state: ActorState) {
    let d = StateDelta::Actor(AgentDelta::SetState { actor: a, state });
    apply_in(s, w, Phase::Decisions, &[d]).expect("a clean state");
}

fn state(s: &S, a: ActorId) -> ActorState {
    *s.ext().get(&a).expect("a state")
}

fn decide_and_admit(s: &mut S, w: &W, cast: &Cast) -> Vec<(ActorId, Decision)> {
    let out: Vec<(ActorId, Decision)> = cast
        .actors()
        .map(|a| (a, cast.decide(a, s, w).expect("decide runs")))
        .collect();
    let deltas: Vec<Delta> = out.iter().flat_map(|(_, d)| d.deltas.clone()).collect();
    let orders: Vec<Order> = out.iter().flat_map(|(_, d)| d.orders.clone()).collect();
    apply_in(s, w, Phase::Decisions, &deltas).expect("the decisions apply");
    admit(orders, s, w).expect("admission accepts every order");
    out
}

/// A deterministic stream: log-uniform draws over many binades.
struct Draws(u64);

impl Draws {
    fn unit(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn decades(&mut self, lo: f64, hi: f64) -> f64 {
        num::pow(10.0, lo + (hi - lo) * self.unit())
    }
}

/// Every non-currency good of a world, by key.
fn traded(w: &W) -> Vec<(String, GoodId)> {
    w.goods
        .iter()
        .enumerate()
        .map(|(i, g)| (g.key.to_string(), GoodId(i as u32)))
        .filter(|(_, g)| !w.is_currency(*g))
        .collect()
}

/// Every param of a world whose key starts with one of `prefixes`.
fn params_with(w: &W, prefixes: &[&str]) -> Vec<String> {
    w.registry
        .params()
        .iter()
        .map(|p| p.key.to_string())
        .filter(|k| prefixes.iter().any(|x| k.starts_with(x)))
        .collect()
}

/// A stock role's state restated as the older kind's it runs on the flow path.
fn as_older(s: ActorState) -> ActorState {
    match s {
        ActorState::Owner(o) => ActorState::GoodDesk(GoodDeskState {
            share: o.share,
            used: o.used,
            scale: o.scale,
            output: o.output,
        }),
        ActorState::Maker(m) => ActorState::MachDesk(MachDeskState {
            scale: m.scale,
            output: m.output,
        }),
        other => other,
    }
}

fn older_deltas(ds: Vec<Delta>) -> Vec<Delta> {
    ds.into_iter()
        .map(|d| match d {
            StateDelta::Actor(AgentDelta::SetState { actor, state }) => {
                StateDelta::Actor(AgentDelta::SetState {
                    actor,
                    state: as_older(state),
                })
            }
            other => other,
        })
        .collect()
}

#[test]
fn stock_roles_nest_the_flow_roles() {
    // HORSES-SPEC §2.6, §2.8 (SG6) at the rule level: on the flow path (R1a's tape, the horse
    // living one tick at δ = 1), the owner desk decides and produces exactly what P2.0's good
    // desk does (appb's tape), and the maker exactly what P2.1's type desk does (I0's tape),
    // order for order and delta for delta once their states are read as the older kinds', from
    // the same state: random prices, coins, holdings, techniques (corners included), spending,
    // turnover, technique rates and tilts. Neither wears at upkeep. The three tapes name the
    // same goods, classes and actors, so their ids agree.
    let (wr, gr, cr) = load(R1A);
    let (wa, ga, ca) = load(APPB);
    let (wi, gi, ci) = load(I0);
    for k in ["coin", "good", "labour", "land", "mach"] {
        assert_eq!(good(&wr, k), good(&wa, k));
        assert_eq!(good(&wr, k), good(&wi, k));
    }
    for k in ["desk.good", "desk.mach", "provider", "workers"] {
        assert_eq!(actor(&wr, k), actor(&wa, k));
        assert_eq!(actor(&wr, k), actor(&wi, k));
    }
    let (dg, dm) = (actor(&wr, "desk.good"), actor(&wr, "desk.mach"));
    // (R1a's key, appb's key, I0's key).
    let same_params = [
        ("spend.workers", "spend.workers", "spend.workers"),
        ("spend.provider", "spend.provider", "spend.provider"),
        (
            "buffer.desk.good.cash",
            "buffer.desk.good.cash",
            "buffer.desk.good.cash",
        ),
        (
            "buffer.desk.mach.cash",
            "buffer.desk.mach.cash",
            "buffer.desk.mach.cash",
        ),
        (
            "adjust.technique.good",
            "adjust.technique",
            "adjust.technique.good",
        ),
        ("tilt.desk.good", "tilt.desk.good", "tilt.desk.good"),
        ("tilt.desk.mach", "tilt.desk.mach", "tilt.desk.mach"),
    ];
    let goods = traded(&wr);
    let coin = good(&wr, "coin");
    let actors: Vec<ActorId> = cr.actors().collect();
    let mut d = Draws(0x2026_0928_0001);
    for i in 0..3000 {
        let (mut sr, mut sa, mut si) = (gr.clone(), ga.clone(), gi.clone());
        for (_, g) in &goods {
            let p = d.decades(-6.0, 6.0);
            set_price(&mut sr, &wr, *g, p);
            set_price(&mut sa, &wa, *g, p);
            set_price(&mut si, &wi, *g, p);
        }
        for &a in &actors {
            let c = if i % 17 == 0 {
                0.0
            } else {
                d.decades(-9.0, 9.0)
            };
            set_holding(&mut sr, &wr, a, coin, c);
            set_holding(&mut sa, &wa, a, coin, c);
            set_holding(&mut si, &wi, a, coin, c);
        }
        for (desk, out) in [("desk.good", "good"), ("desk.mach", "mach")] {
            let q = d.decades(-3.0, 3.0);
            for (s, w) in [(&mut sr, &wr), (&mut sa, &wa), (&mut si, &wi)] {
                set_holding(s, w, actor(w, desk), good(w, out), q);
            }
        }
        let share = match i % 5 {
            0 => 0.0,
            1 => 1.0,
            _ => d.unit(),
        };
        let ActorState::Owner(o) = state(&sr, dg) else {
            panic!("the owner desk's state")
        };
        set_state(
            &mut sr,
            &wr,
            dg,
            ActorState::Owner(OwnerState { share, ..o }),
        );
        let ActorState::GoodDesk(g) = state(&sa, dg) else {
            panic!("the good desk's state")
        };
        set_state(
            &mut sa,
            &wa,
            dg,
            ActorState::GoodDesk(GoodDeskState { share, ..g }),
        );
        for (kr, ka, ki) in same_params {
            let v = if i % 7 == 0 {
                1e4
            } else if ka.starts_with("tilt.") {
                8.0 * d.unit()
            } else {
                d.decades(-2.0, 3.0)
            };
            set_param(&mut sr, &wr, kr, v);
            set_param(&mut sa, &wa, ka, v);
            set_param(&mut si, &wi, ki, v);
        }
        for &a in &actors {
            let r = cr.decide(a, &sr, &wr).unwrap();
            let other = if a == dg {
                ca.decide(a, &sa, &wa).unwrap()
            } else {
                ci.decide(a, &si, &wi).unwrap()
            };
            assert_eq!(
                r.orders,
                other.orders,
                "draw {i}: {}",
                wr.key_of(a).unwrap()
            );
            assert_eq!(
                older_deltas(r.deltas),
                other.deltas,
                "draw {i}: {}",
                wr.key_of(a).unwrap()
            );
        }
        decide_and_admit(&mut sr, &wr, &cr);
        decide_and_admit(&mut sa, &wa, &ca);
        decide_and_admit(&mut si, &wi, &ci);
        for &a in &actors {
            for (_, g) in &goods {
                let q = if d.unit() < 0.1 {
                    0.0
                } else {
                    d.decades(-6.0, 3.0)
                };
                set_holding(&mut sr, &wr, a, *g, q);
                set_holding(&mut sa, &wa, a, *g, q);
                set_holding(&mut si, &wi, a, *g, q);
            }
            let r = older_deltas(cr.produce(a, &sr, &wr).unwrap());
            let other = if a == dg {
                ca.produce(a, &sa, &wa).unwrap()
            } else {
                ci.produce(a, &si, &wi).unwrap()
            };
            assert_eq!(r, other, "draw {i}: {}", wr.key_of(a).unwrap());
        }
        for a in [dg, dm] {
            assert!(cr.upkeep(a, &sr, &wr).unwrap().is_empty());
        }
    }
}

#[test]
fn stock_roles_never_overbudget_or_overdraw() {
    // RULES §2's arithmetic that cannot fail, for the stock kinds (HORSES-SPEC §8): whatever the
    // prices, coins, holdings, stock records and dials, every role's orders pass admission after
    // its own transfers apply, every burn in produce fits its holding, and every wear burn at
    // upkeep fits too. The draws span twelve decades of price, eighteen of coin and of stock,
    // spending shares that round to exactly 1, markups tilted by up to 8, and the maker's record
    // of its serving stock an ulp above what it holds, on H2 (the capacity desk and the maker,
    // fodder bought, no pasture) and P7 (the owner desk holding its horses).
    for text in [H2, P7] {
        let (w, genesis, cast) = load(text);
        let goods = traded(&w);
        let coin = good(&w, "coin");
        let horse = good(&w, "horse");
        let actors: Vec<ActorId> = cast.actors().collect();
        let rates = params_with(&w, &["spend.", "buffer.", "adjust."]);
        let tilts = params_with(&w, &["tilt."]);
        let mut d = Draws(0x2026_0928_0002);
        for i in 0..3000 {
            let mut s = genesis.clone();
            for (_, g) in &goods {
                set_price(&mut s, &w, *g, d.decades(-6.0, 6.0));
            }
            for &a in &actors {
                let c = if i % 17 == 0 {
                    0.0
                } else {
                    d.decades(-9.0, 9.0)
                };
                set_holding(&mut s, &w, a, coin, c);
                let key = w.key_of(a).unwrap().to_string();
                match state(&s, a) {
                    ActorState::Maker(m) => {
                        let h = d.decades(-9.0, 9.0);
                        set_holding(&mut s, &w, a, horse, h);
                        let own = match i % 4 {
                            0 => f64::from_bits(h.to_bits() + 1),
                            1 => h,
                            2 => 0.0,
                            _ => h * d.unit(),
                        };
                        set_state(&mut s, &w, a, ActorState::Maker(MakerState { own, ..m }));
                    }
                    ActorState::Capacity(_) => {
                        set_holding(&mut s, &w, a, horse, d.decades(-9.0, 9.0));
                        set_holding(&mut s, &w, a, good(&w, "traction"), d.decades(-3.0, 3.0));
                    }
                    ActorState::Owner(o) => {
                        set_holding(&mut s, &w, a, horse, d.decades(-9.0, 9.0));
                        let share = match i % 5 {
                            0 => 0.0,
                            1 => 1.0,
                            _ => d.unit(),
                        };
                        set_state(&mut s, &w, a, ActorState::Owner(OwnerState { share, ..o }));
                    }
                    ActorState::GoodDesk(g) => {
                        let share = match i % 5 {
                            0 => 0.0,
                            1 => 1.0,
                            _ => d.unit(),
                        };
                        set_state(
                            &mut s,
                            &w,
                            a,
                            ActorState::GoodDesk(GoodDeskState { share, ..g }),
                        );
                    }
                    _ => {}
                }
                if let Some(out) = key.strip_prefix("desk.") {
                    if out == "good" || out == "fodder" {
                        set_holding(&mut s, &w, a, good(&w, out), d.decades(-3.0, 3.0));
                    }
                }
            }
            for key in &rates {
                let v = if i % 7 == 0 {
                    1e4
                } else {
                    d.decades(-2.0, 3.0)
                };
                set_param(&mut s, &w, key, v);
            }
            for key in &tilts {
                set_param(&mut s, &w, key, 8.0 * d.unit());
            }
            decide_and_admit(&mut s, &w, &cast);
            // Produce on arbitrary inputs, the durable good included: each role burns within
            // what it holds; then upkeep's wear fits what is left.
            for &a in &actors {
                for (_, g) in &goods {
                    if *g == horse && d.unit() < 0.5 {
                        continue;
                    }
                    let q = if d.unit() < 0.1 {
                        0.0
                    } else {
                        d.decades(-6.0, 3.0)
                    };
                    set_holding(&mut s, &w, a, *g, q);
                }
            }
            let ds: Vec<Delta> = cast
                .actors()
                .flat_map(|a| cast.produce(a, &s, &w).expect("produce runs"))
                .collect();
            apply_in(&mut s, &w, Phase::Production, &ds).expect("no burn falls short");
            let ds: Vec<Delta> = cast
                .actors()
                .flat_map(|a| cast.upkeep(a, &s, &w).expect("upkeep runs"))
                .collect();
            apply_in(&mut s, &w, Phase::Upkeep, &ds).expect("no wear falls short");
        }
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= CLOSE * b.abs().max(a.abs())
}

#[test]
fn capacity_desk_replaces_wear_at_rest_and_stops_in_a_glut() {
    // HORSES-SPEC §2.7, §4.1: at the oracle's point the capacity desk runs every head it holds
    // (z = κ·H), targets the stock it holds (K* = H) and orders its wear (m = δ·H), and offers
    // every horse-day it holds; the maker moves δ of its serving stock into service. With its
    // heads doubled the stock is past 1.5 of its target and the order stops (a sign), while
    // every head still runs.
    let (w, s, cast) = load(H1);
    let (dc, dm) = (actor(&w, "desk.capacity"), actor(&w, "desk.maker"));
    let horse = good(&w, "horse");
    let clock = w.clock;
    let delta = clock.fraction(FractionPerYear(0.1));
    let h = held(&s, dc, horse);
    let d = cast.decide(dc, &s, &w).unwrap();
    let ActorState::Capacity(c) = (match &d.deltas[..] {
        [StateDelta::Actor(AgentDelta::SetState { state, .. })] => *state,
        other => panic!("{other:?}"),
    }) else {
        panic!("a capacity state")
    };
    assert_eq!(c.held, h);
    assert_eq!(c.run, h, "κ = 1 a tick: every head runs");
    assert!(close(c.target, h), "{} {h}", c.target);
    assert!(close(c.order, delta * h), "{} {}", c.order, delta * h);
    let sold: Vec<&Order> = d
        .orders
        .iter()
        .filter(|o| matches!(o.side, Side::Sell))
        .collect();
    assert_eq!(sold.len(), 1);
    assert_eq!(sold[0].qty, held(&s, dc, good(&w, "traction")));
    let m = cast.decide(dm, &s, &w).unwrap();
    let ActorState::Maker(ms) = (match &m.deltas[..] {
        [StateDelta::Actor(AgentDelta::SetState { state, .. })] => *state,
        other => panic!("{other:?}"),
    }) else {
        panic!("a maker state")
    };
    let ActorState::Maker(m0) = state(&s, dm) else {
        panic!("a maker state")
    };
    // serving = own + δ·serving at rest, so the keep is δ/(1 − δ) of the record.
    assert!(close(ms.serving - m0.own, delta * ms.serving));
    // A glut: twice the heads.
    let mut g = s.clone();
    set_holding(&mut g, &w, dc, horse, 2.0 * h);
    let d = cast.decide(dc, &g, &w).unwrap();
    let ActorState::Capacity(c) = (match &d.deltas[..] {
        [StateDelta::Actor(AgentDelta::SetState { state, .. })] => *state,
        other => panic!("{other:?}"),
    }) else {
        panic!("a capacity state")
    };
    assert_eq!(c.order, 0.0);
    // The outlay covers the running cost of every head: p_h/O is 3 at the point.
    assert_eq!(c.run, 2.0 * h);
    let bought: Vec<&Order> = d.orders.iter().filter(|o| o.good == horse).collect();
    assert_eq!(bought.len(), 1);
    assert_eq!(bought[0].qty, 0.0);
    // Its upkeep wears δ of what it held when it decided, and not what it bought since.
    let mut u = g.clone();
    decide_and_admit(&mut u, &w, &cast);
    set_holding(&mut u, &w, dc, horse, 3.0 * h);
    let ds = cast.upkeep(dc, &u, &w).unwrap();
    assert_eq!(
        ds,
        vec![StateDelta::Burn {
            from: Holder::Actor(dc),
            good: horse,
            amount: Amount::Qty(delta * (2.0 * h)),
            prov: Provenance::Depreciation,
        }]
    );
}

#[test]
fn stock_specs_are_checked_at_load() {
    // SG7 (HORSES-SPEC §3), each refusal with its path: a capacity desk's durable good that
    // lives one tick, or that wears all of it a tick; a maker's durable good worn all of it a
    // tick while it lives indefinitely, or living one tick while it wears less; the flow path
    // with a cover or a running recipe; a durable good that lives three ticks; a good desk or a
    // type desk that buys the durable good (M6); and a running good that is the stock itself.
    let edit = |text: &str, from: &str, to: &str| {
        assert!(text.contains(from), "{from:?}");
        text.replacen(from, to, 1)
    };
    let one_tick =
        r#"(key: "horse", life: Years("life.one_tick"), price_rate: Some("rate.horse")),"#;
    let indefinite = r#"(key: "horse", life: Indefinite, price_rate: Some("rate.horse")),"#;
    let cases = [
        (
            edit(H1, indefinite, one_tick),
            "actors[desk.capacity].spec.delta",
        ),
        (
            edit(
                H1,
                r#"(key: "inst.horse.delta", value: 0.1,"#,
                r#"(key: "inst.horse.delta", value: 1.0,"#,
            ),
            "actors[desk.capacity].spec.delta",
        ),
        (
            edit(
                P7,
                r#"(key: "inst.horse.delta", value: 0.04,"#,
                r#"(key: "inst.horse.delta", value: 1.0,"#,
            ),
            "actors[desk.good].spec.delta",
        ),
        (
            edit(
                R1A,
                r#"(key: "inst.mach.delta", value: 1.0,"#,
                r#"(key: "inst.mach.delta", value: 0.5,"#,
            ),
            "actors[desk.good].spec.delta",
        ),
        (
            edit(
                &edit(R1A, r#"cover: None,"#, r#"cover: Some("cover.mach"),"#),
                r#"(key: "life.one_tick","#,
                r#"(key: "cover.mach", value: 0.07692307692307693, unit: Years, basis: Assumed("four weeks")),
        (key: "life.one_tick","#,
            ),
            "actors[desk.mach].spec.cover",
        ),
        (
            edit(
                R1A,
                r#"(key: "inst.mach.run.labour", value: 0.0,"#,
                r#"(key: "inst.mach.run.labour", value: 0.5,"#,
            ),
            "actors[desk.mach].spec.running",
        ),
        (
            edit(
                &edit(
                    H1,
                    indefinite,
                    r#"(key: "horse", life: Years("life.three"), price_rate: Some("rate.horse")),"#,
                ),
                r#"(key: "life.one_tick","#,
                r#"(key: "life.three", value: 0.0577, unit: Years, basis: Assumed("three weeks")),
        (key: "life.one_tick","#,
            ),
            "actors[desk.capacity].spec.stock",
        ),
        (
            edit(
                H1,
                r#"output: "good", labour: "labour", mach: "traction","#,
                r#"output: "good", labour: "labour", mach: "horse","#,
            ),
            "actors[desk.good].spec.mach",
        ),
        (
            edit(
                H1,
                r#"recipe: (own: "inst.fodder.own", inputs: [],"#,
                r#"recipe: (own: "inst.fodder.own", inputs: [(good: "horse", coef: "inst.fodder.own")],"#,
            ),
            "actors[desk.fodder].spec.recipe.inputs",
        ),
        (
            edit(
                H1,
                r#"running: (goods: [(good: "fodder", coef: "inst.horse.run.fodder")], labour: "inst.horse.run.labour"),
            delta: "inst.horse.delta", adjust: "adjust.invest.capacity","#,
                r#"running: (goods: [(good: "horse", coef: "inst.horse.run.fodder")], labour: "inst.horse.run.labour"),
            delta: "inst.horse.delta", adjust: "adjust.invest.capacity","#,
            ),
            "actors[desk.capacity].spec.running.goods[horse].good",
        ),
    ];
    for (text, path) in cases {
        let err = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
        };
        assert_eq!(err.path, path, "{err}");
    }
}

/// `text` with `from` replaced by `to` everywhere, `from` present.
fn edit_all(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "{from:?}");
    text.replace(from, to)
}

/// Fodder that lives indefinitely, the life v2a.4 gives it.
fn fodder_stored(text: &str) -> String {
    edit_all(
        text,
        r#"(key: "fodder", life: Years("life.one_tick"),"#,
        r#"(key: "fodder", life: Indefinite,"#,
    )
}

/// The maker's horses fed on nothing it buys.
fn maker_unfed(text: &str) -> String {
    edit_all(
        text,
        r#"kappa: "inst.horse.kappa", running: (goods: [(good: "fodder", coef: "inst.horse.run.fodder")],"#,
        r#"kappa: "inst.horse.kappa", running: (goods: [],"#,
    )
}

fn refusal(text: &str) -> LoadError {
    match load_text(text) {
        Err(e) => e,
        Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
    }
}

#[test]
fn m6_nets_only_the_durable_good() {
    // O48 (decision 238): the capacity and owner desks net their holding of the durable good they
    // hold and of nothing else, so a storable running good they buy is refused at load, with its
    // path. The first tape is the fidelity review's (D:/rustyecon-p2g/review-fidelity/m6/): H1
    // with fodder stored and bought by the capacity desk alone. Before the fix it loaded and ran,
    // and its fodder desk held 49 times its genesis fodder at tick 520 as the price fell from 0.2
    // to 3.6e-6. The second is P7's owner desk, its fodder stored. The durable good itself stays
    // exempt: H1 and P7, whose horses live indefinitely, load.
    for text in [H1, P7] {
        let (w, _) = load_text(text).unwrap();
        Cast::new(&w).expect("the durable good is netted");
    }
    let cases = [
        (
            maker_unfed(&fodder_stored(H1)),
            "actors[desk.capacity].spec.running.goods",
        ),
        (
            maker_unfed(&fodder_stored(P7)),
            "actors[desk.good].spec.running",
        ),
    ];
    for (text, path) in cases {
        let err = refusal(&text);
        assert_eq!(err.path, path, "{err}");
    }
}

#[test]
fn m5_refuses_a_stored_good_offered_in_full() {
    // GOODS-CHAIN M5 and §3.2 fact 1: a seller of a storable good offers I/(1 + b_G); offered in
    // full, its stock has a unit root. So a kind that offers every unit it holds sells only a good
    // that lives at most one tick, each refusal with its path, in tapes where no buyer is refused
    // first: the fodder desk (a type desk) with fodder stored and bought by no one, every horse
    // fed on the good instead; the capacity desk with its horse-days stored and the good desk
    // working fodder in their place; and the good desk and the owner desk with the good stored and
    // the households buying fodder in its place. The maker, which offers its durable good under
    // its cover, is not held to it, its cover off included (P3 (v)'s control).
    let fed_on_the_good = |text: &str| {
        edit_all(
            text,
            r#"(good: "fodder", coef: "inst.horse.run.fodder")"#,
            r#"(good: "good", coef: "inst.horse.run.fodder")"#,
        )
    };
    let good_stored = |text: &str| {
        edit_all(
            &edit_all(
                text,
                r#"(key: "good", life: Years("life.one_tick"),"#,
                r#"(key: "good", life: Indefinite,"#,
            ),
            r#"basket: [(good: "good", weight: "inst.good.weight"),"#,
            r#"basket: [(good: "fodder", weight: "inst.good.weight"),"#,
        )
    };
    let cases = [
        (
            fed_on_the_good(&fodder_stored(H1)),
            "actors[desk.fodder].spec.output",
        ),
        (
            edit_all(
                &edit_all(
                    H1,
                    r#"(key: "traction", life: Years("life.one_tick"),"#,
                    r#"(key: "traction", life: Indefinite,"#,
                ),
                r#"output: "good", labour: "labour", mach: "traction","#,
                r#"output: "good", labour: "labour", mach: "fodder","#,
            ),
            "actors[desk.capacity].spec.hours",
        ),
        (good_stored(H1), "actors[desk.good].spec.output"),
        (good_stored(P7), "actors[desk.good].spec.output"),
    ];
    for (text, path) in cases {
        let err = refusal(&text);
        assert_eq!(err.path, path, "{err}");
    }
    // Its cover off, and the cover's param, which nothing reads then, dropped.
    let uncovered: String = edit_all(H1, r#"cover: Some("cover.maker"),"#, "cover: None,")
        .lines()
        .filter(|l| !l.trim_start().starts_with(r#"(key: "cover.maker","#))
        .map(|l| format!("{l}\n"))
        .collect();
    assert!(!uncovered.contains("cover.maker"));
    let (w, _) = load_text(&uncovered).unwrap();
    Cast::new(&w).expect("the maker sells under its own rule");
}

#[test]
fn stock_specs_round_trip_in_canonical_form() {
    // Every stocks-probe tape survives to_ron and a parse with the same world; its new kinds'
    // inline numbers (the maker's genesis record, the owner desk's genesis share) are listed.
    for text in [H1, H2, P7, R1A] {
        let t = Tape::<Agents>::from_ron(text).unwrap();
        let (w1, _) = resolve(&t).unwrap();
        Cast::new(&w1).unwrap();
        let back = Tape::<Agents>::from_ron(&t.to_ron()).unwrap();
        assert_eq!(back, t.canonical());
        let (w2, _) = resolve(&back).unwrap();
        assert_eq!(w1.world_id, w2.world_id);
        for a in &t.actors {
            let numbers = rustyecon_agents::spec::inline_numbers(&a.spec);
            match &a.spec {
                RawSpec::Maker(m) => assert_eq!(numbers[0], ("own".to_string(), m.own)),
                RawSpec::OwnerDesk(o) => {
                    assert_eq!(
                        numbers[0],
                        ("technique.share".to_string(), o.technique.share)
                    )
                }
                _ => {}
            }
        }
    }
}

#[test]
fn stock_roles_read_only_their_view() {
    // R13: a stock role's decision depends on posted prices, its own holding and state, and the
    // params. Changing every other actor's coin, horses and state leaves it bit for bit.
    for text in [H1, P7] {
        let (w, genesis, cast) = load(text);
        let coin = good(&w, "coin");
        let horse = good(&w, "horse");
        for a in cast.actors() {
            let before = cast.decide(a, &genesis, &w).unwrap();
            let mut s = genesis.clone();
            for b in cast.actors().filter(|&b| b != a) {
                set_holding(&mut s, &w, b, coin, 1e6);
                set_holding(&mut s, &w, b, horse, 123.0);
                match state(&s, b) {
                    ActorState::Capacity(c) => set_state(
                        &mut s,
                        &w,
                        b,
                        ActorState::Capacity(CapacityState { held: 7.0, ..c }),
                    ),
                    ActorState::Maker(m) => set_state(
                        &mut s,
                        &w,
                        b,
                        ActorState::Maker(MakerState {
                            own: 5.0,
                            serving: 9.0,
                            ..m
                        }),
                    ),
                    _ => {}
                }
            }
            assert_eq!(cast.decide(a, &s, &w).unwrap(), before, "{a}");
        }
    }
}

#[test]
fn stock_role_sites_name_their_methods() {
    // D10 item 4 on the stock kinds: κ is a flow (hours a head a year, `Clock::flow`), δ a
    // fraction (`Clock::fraction`), every rate a share of a stock (turnover, the technique, the
    // stock rules' adjustment), the maker's cover a span in whole ticks, and every coefficient
    // a value; each reads through the view as the clock converts it.
    for text in [H1, P7] {
        let (w, s, _) = load(text);
        let mut seen = std::collections::BTreeSet::new();
        for decl in &w.actors {
            let state = s.ext().get(&decl.id).unwrap();
            let v = view(decl, &s, &w, state).unwrap();
            for (path, site) in decl.spec.sites(&w) {
                let unit = w.registry.params()[site.param.idx()].unit;
                let want = match unit {
                    rustyecon_core::Unit::RatePerYear => ClockMethod::Share,
                    rustyecon_core::Unit::FlowPerYear => ClockMethod::Flow,
                    rustyecon_core::Unit::FractionPerYear => ClockMethod::Fraction,
                    rustyecon_core::Unit::Years => ClockMethod::Ticks,
                    _ => ClockMethod::Value,
                };
                assert_eq!(site.method, want, "{}: {path}", decl.key);
                site.per_tick(&v.params, v.clock).unwrap();
                seen.insert(site.method);
            }
        }
        assert!(seen.contains(&ClockMethod::Fraction) && seen.contains(&ClockMethod::Flow));
    }
}
