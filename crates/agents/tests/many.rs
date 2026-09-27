//! The many-market roles (P2.1; docs/probe/MARKETS-RULES.md; MARKETS-SPEC §2), on the markets
//! probe's tapes: I0 (Appendix B in the new kinds), I2 (a basket with space, a chain of two
//! machine types) and L3 (four categories that buy land, two types that buy each other's
//! services).
//!
//! As in `roles.rs`, most comparisons are exact. One bar is relative and named here: `CLOSE`,
//! 1e-14 relative, for a value the test recomputes in another order of operations than the
//! rule's (a chain of a few products, sums and quotients, each within half an ulp).

use rustyecon_agents::cast::view;
use rustyecon_agents::{
    ActorState, AgentDelta, Agents, Cast, Decision, GoodDeskState, MachDeskState, ProviderState,
    RawSpec,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, ClockMethod, CoreError, GoodId, Holder, Ledger, LoadError,
    NodeId, ParamId, Phase, Provenance, RatePerYear, SimState, StateDelta, Tape, World,
};
use rustyecon_markets::{admit, Order, Side};

const APPB: &str = include_str!("../../../tapes/appb.ron");
const I0: &str = include_str!("../../../tapes/markets-i0.ron");
const I2: &str = include_str!("../../../tapes/markets-i2.ron");
const L3: &str = include_str!("../../../tapes/markets-l3.ron");
const CLOSE: f64 = 1e-14;

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

fn set_share(s: &mut S, w: &W, a: ActorId, share: f64) {
    if let Some(ActorState::GoodDesk(st)) = s.ext().get(&a).copied() {
        set_state(s, w, a, ActorState::GoodDesk(GoodDeskState { share, ..st }));
    }
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

#[test]
fn many_roles_never_overbudget_or_overdraw() {
    // RULES §2's arithmetic that cannot fail, for the four many-market kinds (MARKETS-SPEC
    // §2.6): whatever the prices, coins, holdings, techniques and dials, every role's orders
    // pass admission after its own transfers apply, and every burn in produce fits its holding.
    // The draws span twelve decades of price and coin, spending shares that round to exactly 1,
    // and markups tilted by up to 8, on I2 (space in the basket, a chain of types, a zero own
    // input) and L3 (desks that buy land, types that buy each other's services).
    for text in [I2, L3] {
        let (w, genesis, cast) = load(text);
        let goods = traded(&w);
        let coin = good(&w, "coin");
        let actors: Vec<ActorId> = cast.actors().collect();
        let rates = params_with(&w, &["spend.", "buffer.", "adjust."]);
        let tilts = params_with(&w, &["tilt."]);
        let mut d = Draws(0x2026_0927_0001);
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
                // Each desk's own output, as last tick's lot.
                let key = w.key_of(a).unwrap().to_string();
                if let Some(out) = key.strip_prefix("desk.") {
                    set_holding(&mut s, &w, a, good(&w, out), d.decades(-3.0, 3.0));
                }
                let share = match i % 5 {
                    0 => 0.0,
                    1 => 1.0,
                    _ => d.unit(),
                };
                set_share(&mut s, &w, a, share);
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
            // Produce on arbitrary inputs: each role burns within what it holds.
            for &a in &actors {
                for (_, g) in &goods {
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
        }
    }
}

#[test]
fn many_roles_nest_the_appendix_b_roles() {
    // MARKETS-SPEC §2.7 at the rule level: on Appendix B, each many-market kind (I0's tape)
    // decides and produces exactly what its Appendix B kind (appb's tape) does, order for order
    // and delta for delta, from the same state: random prices, coins, holdings, techniques
    // (corners included), spending, turnover, technique rates and tilts. The two tapes name the
    // same goods, classes and actors, so their ids agree.
    let (wa, ga, ca) = load(APPB);
    let (wb, gb, cb) = load(I0);
    for k in ["coin", "good", "labour", "land", "mach"] {
        assert_eq!(good(&wa, k), good(&wb, k));
    }
    for k in ["desk.good", "desk.mach", "provider", "workers"] {
        assert_eq!(actor(&wa, k), actor(&wb, k));
    }
    let same_params = [
        ("spend.workers", "spend.workers"),
        ("spend.provider", "spend.provider"),
        ("buffer.desk.good.cash", "buffer.desk.good.cash"),
        ("buffer.desk.mach.cash", "buffer.desk.mach.cash"),
        ("adjust.technique", "adjust.technique.good"),
    ];
    let goods = traded(&wa);
    let coin = good(&wa, "coin");
    let actors: Vec<ActorId> = ca.actors().collect();
    let mut d = Draws(0x2026_0927_0002);
    for i in 0..3000 {
        let (mut sa, mut sb) = (ga.clone(), gb.clone());
        for (_, g) in &goods {
            let p = d.decades(-6.0, 6.0);
            set_price(&mut sa, &wa, *g, p);
            set_price(&mut sb, &wb, *g, p);
        }
        for &a in &actors {
            let c = if i % 17 == 0 {
                0.0
            } else {
                d.decades(-9.0, 9.0)
            };
            set_holding(&mut sa, &wa, a, coin, c);
            set_holding(&mut sb, &wb, a, coin, c);
        }
        for (desk, out) in [("desk.good", "good"), ("desk.mach", "mach")] {
            let q = d.decades(-3.0, 3.0);
            set_holding(&mut sa, &wa, actor(&wa, desk), good(&wa, out), q);
            set_holding(&mut sb, &wb, actor(&wb, desk), good(&wb, out), q);
        }
        let share = match i % 5 {
            0 => 0.0,
            1 => 1.0,
            _ => d.unit(),
        };
        set_share(&mut sa, &wa, actor(&wa, "desk.good"), share);
        set_share(&mut sb, &wb, actor(&wb, "desk.good"), share);
        for (ka, kb) in same_params {
            let v = if i % 7 == 0 {
                1e4
            } else {
                d.decades(-2.0, 3.0)
            };
            set_param(&mut sa, &wa, ka, v);
            set_param(&mut sb, &wb, kb, v);
        }
        for k in ["tilt.desk.good", "tilt.desk.mach"] {
            let v = 8.0 * d.unit();
            set_param(&mut sa, &wa, k, v);
            set_param(&mut sb, &wb, k, v);
        }
        for &a in &actors {
            assert_eq!(
                ca.decide(a, &sa, &wa).unwrap(),
                cb.decide(a, &sb, &wb).unwrap(),
                "draw {i}: {}",
                wa.key_of(a).unwrap()
            );
        }
        decide_and_admit(&mut sa, &wa, &ca);
        decide_and_admit(&mut sb, &wb, &cb);
        for &a in &actors {
            for (_, g) in &goods {
                let q = if d.unit() < 0.1 {
                    0.0
                } else {
                    d.decades(-6.0, 3.0)
                };
                set_holding(&mut sa, &wa, a, *g, q);
                set_holding(&mut sb, &wb, a, *g, q);
            }
            assert_eq!(
                ca.produce(a, &sa, &wa).unwrap(),
                cb.produce(a, &sb, &wb).unwrap(),
                "draw {i}: {}",
                wa.key_of(a).unwrap()
            );
        }
    }
}

/// J(x) = g0·x + g1·x²/2 on the tapes' schedule (η = 1, k = 1).
fn j(x: f64) -> f64 {
    0.2 * x + 0.8 * x * x / 2.0
}

/// H and M per unit at threshold x, unit-1b.md §4.1, written from the formula.
fn tasks(edges: &[f64], mu: &[f64], x: f64) -> (f64, f64) {
    let (mut h, mut m) = (0.0, 0.0);
    for s in 0..mu.len() {
        let (lo, hi) = (edges[s], edges[s + 1]);
        h += mu[s] * (hi - x.max(lo)).max(0.0);
        m += mu[s] * (j(x.clamp(lo, hi)) - j(lo));
    }
    (h, m)
}

fn qty(d: &Decision, g: GoodId, side: Side) -> Option<f64> {
    d.orders
        .iter()
        .find(|o| o.good == g && std::mem::discriminant(&o.side) == std::mem::discriminant(&side))
        .map(|o| o.qty)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= CLOSE * a.abs().max(b.abs())
}

#[test]
fn category_desk_works_its_segments() {
    // MARKETS-SPEC §2.4 on L3's four categories (edges 0.4 and 0.75): at a technique held fixed
    // (adjust 0), each desk buys H_j(x)·q hours, (M_j(x)/θ)·q of the engine's services and
    // b_j·q land, with q = share(turnover)·coin/c_j and c_j = w·H_j + p_τ·M_j/θ + r·b_j. It
    // posts hours and services whenever it has tasks, at quantity 0 where x makes the
    // coefficient 0, and land only where b_j > 0 (manufactures buys none).
    let (w, genesis, cast) = load(L3);
    let buy = Side::Buy { budget: 0.0 };
    let (labour, land, engine) = (good(&w, "labour"), good(&w, "land"), good(&w, "engine"));
    let edges = [0.0, 0.4, 0.75, 1.0];
    let cats: [(&str, f64, [f64; 3]); 4] = [
        ("manufactures", 0.0, [2.0, 0.0, 0.0]),
        ("food", 0.6, [0.5, 1.5, 0.0]),
        ("care", 0.1, [0.0, 0.0, 1.0]),
        ("shelter", 1.0, [0.0, 0.4, 0.0]),
    ];
    let theta = 2.0;
    for (key, b, mu) in cats {
        let a = actor(&w, &format!("desk.{key}"));
        for x in [0.0, 0.1, 0.4, 0.55, 0.75, 0.9, 1.0] {
            let mut s = genesis.clone();
            set_param(&mut s, &w, &format!("adjust.technique.{key}"), 0.0);
            set_share(&mut s, &w, a, 1.0 - x);
            let dec = cast.decide(a, &s, &w).unwrap();
            let price = |g| s.price(home(&w), g).unwrap();
            let (h, m) = tasks(&edges, &mu, x);
            let c = price(labour) * h + price(engine) * (m / theta) + price(land) * b;
            let v = s
                .param(param(&w, &format!("buffer.desk.{key}.cash")))
                .unwrap();
            let q = w.clock.share(RatePerYear(v)) * held(&s, a, good(&w, "coin")) / c;
            let hours = qty(&dec, labour, buy).expect("hours are posted");
            let serv = qty(&dec, engine, buy).expect("services are posted");
            assert!(
                close(hours, h * q),
                "{key} x {x}: hours {hours} against {}",
                h * q
            );
            assert!(
                close(serv, m / theta * q),
                "{key} x {x}: {serv} against {}",
                m / theta * q
            );
            match qty(&dec, land, buy) {
                Some(r) => assert!(b > 0.0 && close(r, b * q), "{key} x {x}: land {r}"),
                None => assert_eq!(b, 0.0, "{key}: land is posted where b > 0"),
            }
        }
    }
}

#[test]
fn type_desk_buys_the_other_types_services() {
    // MARKETS-SPEC §2.5 on L3's loop: the engine keeps 0.1 of its own service and buys 0.5 of
    // power's, power buys 0.1 of the engine's; each desk's cash cost is Σ a_kl·p_l + λ·w + b·r,
    // its markup p_k(1 − a_kk)/c_k, its outlay share(turnover)·coin, and it keeps min(a_kk·q,
    // K) and offers the rest. Produce is Leontief over the nonzero coefficients, the own input
    // burned first.
    let (w, genesis, cast) = load(L3);
    let buy = Side::Buy { budget: 0.0 };
    let (labour, land, engine, power, coin) = (
        good(&w, "labour"),
        good(&w, "land"),
        good(&w, "engine"),
        good(&w, "power"),
        good(&w, "coin"),
    );
    let (de, dp) = (actor(&w, "desk.engine"), actor(&w, "desk.power"));
    let s = genesis.clone();
    let price = |g| s.price(home(&w), g).unwrap();
    let v = s.param(param(&w, "buffer.desk.engine.cash")).unwrap();
    for (desk, out, other, own, a_other, lam, b) in [
        (de, engine, power, 0.1, 0.5, 0.12, 0.4),
        (dp, power, engine, 0.0, 0.1, 0.12, 0.6),
    ] {
        let dec = cast.decide(desk, &s, &w).unwrap();
        let c = a_other * price(other) + lam * price(labour) + b * price(land);
        let q = w.clock.share(RatePerYear(v)) * held(&s, desk, coin) / c;
        let k = held(&s, desk, out);
        let offered = qty(&dec, out, Side::Sell).unwrap();
        assert!(close(offered, k - (own * q).min(k)));
        assert!(close(qty(&dec, other, buy).unwrap(), a_other * q));
        assert!(close(qty(&dec, labour, buy).unwrap(), lam * q));
        assert!(close(qty(&dec, land, buy).unwrap(), b * q));
        // Produce: every input but one in excess, each in turn the binding one.
        for bind in 0..4 {
            let mut t = genesis.clone();
            let big = 1e3;
            let hold = [
                (out, if bind == 0 { 0.7 } else { big }),
                (other, if bind == 1 { 0.7 } else { big }),
                (labour, if bind == 2 { 0.7 } else { big }),
                (land, if bind == 3 { 0.7 } else { big }),
            ];
            for (g, x) in hold {
                set_holding(&mut t, &w, desk, g, x);
            }
            let coefs = [own, a_other, lam, b];
            let y = if coefs[bind] > 0.0 {
                num::max_scale(0.7, coefs[bind]).unwrap()
            } else {
                // A zero own coefficient does not bind: the next smallest does.
                (1..4)
                    .map(|i| num::max_scale(big, coefs[i]).unwrap())
                    .fold(f64::INFINITY, f64::min)
            };
            let ds = cast.produce(desk, &t, &w).unwrap();
            let minted: Vec<f64> = ds
                .iter()
                .filter_map(|d| match d {
                    StateDelta::Mint { good: g, qty, .. } if *g == out => Some(*qty),
                    _ => None,
                })
                .collect();
            assert_eq!(minted, [y], "bind {bind}");
            apply_in(&mut t, &w, Phase::Production, &ds).expect("the burns fit");
            match t.ext().get(&desk) {
                Some(ActorState::MachDesk(MachDeskState { output, .. })) => {
                    assert_eq!(*output, y)
                }
                other => panic!("{other:?}"),
            }
        }
    }
}

#[test]
fn basket_households_buy_and_eat_their_basket() {
    // MARKETS-SPEC §2.3: P_s = Σ z_j·p_j from 0.0 in item order (space at r); the workers
    // offer N·min(ln1p(w/P_s)/χ_max, 1) hours; the provider transfers N·P_s, or all its coin if
    // less; each household buys z_j·n of every item, n = budget/P_s, and eats min_j(held_j/z_j)
    // baskets, burning z_j·n of each.
    for (text, items) in [
        (I2, vec![("good", 1.0), ("land", 1.0)]),
        (
            L3,
            vec![
                ("manufactures", 0.3),
                ("food", 1.0),
                ("care", 0.2),
                ("shelter", 0.8),
            ],
        ),
    ] {
        let (w, genesis, cast) = load(text);
        let (prov, wk) = (actor(&w, "provider"), actor(&w, "workers"));
        let (labour, coin) = (good(&w, "labour"), good(&w, "coin"));
        let s = genesis.clone();
        let price = |k: &str| s.price(home(&w), good(&w, k)).unwrap();
        let ps = items.iter().fold(0.0, |acc, (k, z)| acc + z * price(k));
        let n = 4.0;
        let d = cast.decide(wk, &s, &w).unwrap();
        let hours = n * (num::ln1p(price("labour") / ps) / 1.0).min(1.0);
        assert!(d
            .orders
            .iter()
            .any(|o| o.good == labour && o.side == Side::Sell && o.qty == hours));
        let budget = w
            .clock
            .share(RatePerYear(s.param(param(&w, "spend.workers")).unwrap()))
            * held(&s, wk, coin);
        for (k, z) in &items {
            let q = qty(&d, good(&w, k), Side::Buy { budget: 0.0 }).unwrap();
            assert!(close(q, z * budget / ps), "{k}");
        }
        let d = cast.decide(prov, &s, &w).unwrap();
        let due = n * ps;
        assert!(d.deltas.contains(&StateDelta::Actor(AgentDelta::SetState {
            actor: prov,
            state: ActorState::Provider(ProviderState {
                due,
                paid: due.min(held(&s, prov, coin)),
            }),
        })));
        // Eating: the second item binds at 0.5 of the others' baskets.
        let mut t = genesis.clone();
        for (i, (k, z)) in items.iter().enumerate() {
            let q = if i == 1 { 0.5 * z * 3.0 } else { z * 3.0 };
            set_holding(&mut t, &w, wk, good(&w, k), q);
        }
        let ds = cast.produce(wk, &t, &w).unwrap();
        let eaten = num::max_scale(0.5 * items[1].1 * 3.0, items[1].1).unwrap();
        for (k, z) in &items {
            assert!(ds.contains(&StateDelta::Burn {
                from: Holder::Actor(wk),
                good: good(&w, k),
                amount: Amount::Qty(z * eaten),
                prov: Provenance::Consumption,
            }));
        }
        apply_in(&mut t, &w, Phase::Production, &ds).expect("the burns fit");
    }
}

#[test]
fn many_specs_are_checked_at_load() {
    // The load checks of the four kinds, each with its path: densities that do not fit the
    // edges, a bought service that is the desk's own output, an item named twice, an empty
    // basket, and a desk role declared a Pop.
    let edit = |from: &str, to: &str| {
        assert!(L3.contains(from), "{from:?}");
        L3.replacen(from, to, 1)
    };
    let cases = [
        (
            edit(
                r#"density: ["inst.food.mu.1", "inst.food.mu.2", "inst.food.mu.3"]"#,
                r#"density: ["inst.food.mu.1", "inst.food.mu.2"]"#,
            ),
            "actors[desk.food].spec.line.density",
        ),
        (
            edit(
                r#"inputs: [(good: "power", coef: "inst.engine.in.power")]"#,
                r#"inputs: [(good: "engine", coef: "inst.engine.in.power")]"#,
            ),
            "actors[desk.engine].spec.recipe.inputs[engine].good",
        ),
        (
            edit(
                r#"(good: "care", weight: "inst.care.weight"), (good: "shelter""#,
                r#"(good: "food", weight: "inst.care.weight"), (good: "shelter""#,
            ),
            "actors[provider].spec.basket[food].good",
        ),
        (
            edit(
                r#"(key: "desk.food", kind: Desk,"#,
                r#"(key: "desk.food", kind: Pop,"#,
            ),
            "actors[desk.food].spec",
        ),
    ];
    for (text, path) in cases {
        let err = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
        };
        assert_eq!(err.path, path, "{err}");
    }
    let mut t = Tape::<Agents>::from_ron(L3).unwrap();
    for a in &mut t.actors {
        if let RawSpec::BasketWorkers(w) = &mut a.spec {
            w.basket.clear();
        }
    }
    let err = resolve(&t).expect_err("an empty basket");
    assert_eq!(err.path, "actors[workers].spec.basket");
}

#[test]
fn many_specs_round_trip_in_canonical_form() {
    // Every many-market tape survives to_ron and a parse with the same world, and its lists keep
    // the order written: the basket, the segments and the bought services are evaluation order.
    for text in [I0, I2, L3] {
        let t = Tape::<Agents>::from_ron(text).unwrap();
        let (w1, _) = resolve(&t).unwrap();
        Cast::new(&w1).unwrap();
        let back = Tape::<Agents>::from_ron(&t.to_ron()).unwrap();
        assert_eq!(back, t.canonical());
        let (w2, _) = resolve(&back).unwrap();
        assert_eq!(w1.world_id, w2.world_id);
        for (a, b) in t.actors.iter().zip(&t.canonical().actors) {
            if let (RawSpec::BasketWorkers(x), RawSpec::BasketWorkers(y)) = (&a.spec, &b.spec) {
                assert_eq!(x.basket, y.basket);
            }
        }
    }
    // A reordered basket is another tape: its world differs.
    let swapped = L3.replace(
        r#"basket: [(good: "manufactures", weight: "inst.manufactures.weight"), (good: "food", weight: "inst.food.weight"),"#,
        r#"basket: [(good: "food", weight: "inst.food.weight"), (good: "manufactures", weight: "inst.manufactures.weight"),"#,
    );
    assert_ne!(swapped, L3);
    let (a, _) = load_text(L3).unwrap();
    let (b, _) = load_text(&swapped).unwrap();
    assert_ne!(a.world_id, b.world_id);
}

#[test]
fn many_roles_read_only_their_view() {
    // R13: a many-market actor's decision depends on posted prices, its own holding and state,
    // and the params. Changing every other actor's coin, stocks and state leaves it bit for bit.
    let (w, genesis, cast) = load(L3);
    let coin = good(&w, "coin");
    let food = good(&w, "food");
    for a in cast.actors() {
        let before = cast.decide(a, &genesis, &w).unwrap();
        let mut s = genesis.clone();
        for b in cast.actors().filter(|&b| b != a) {
            set_holding(&mut s, &w, b, coin, 1e6);
            set_holding(&mut s, &w, b, food, 123.0);
            set_share(&mut s, &w, b, 0.5);
        }
        assert_eq!(cast.decide(a, &s, &w).unwrap(), before, "{a}");
    }
}

#[test]
fn many_role_sites_name_their_methods() {
    // D10 item 4 on the new kinds: every rate a many-market role reads is a share of a stock
    // (turnover, spending, the technique's adjustment), and every coefficient a value.
    let (w, s, _) = load(L3);
    for decl in &w.actors {
        let state = s.ext().get(&decl.id).unwrap();
        let v = view(decl, &s, &w, state).unwrap();
        for (path, site) in decl.spec.sites(&w) {
            let unit = w.registry.params()[site.param.idx()].unit;
            let want = match unit {
                rustyecon_core::Unit::RatePerYear => ClockMethod::Share,
                rustyecon_core::Unit::FlowPerYear => ClockMethod::Flow,
                _ => ClockMethod::Value,
            };
            assert_eq!(site.method, want, "{}: {path}", decl.key);
            site.per_tick(&v.params, v.clock).unwrap();
        }
    }
}
