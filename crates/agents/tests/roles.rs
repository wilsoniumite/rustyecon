//! The Appendix B roles (P2.0; docs/probe/RULES.md, docs/ENGINE.md §4 as amended at P2.0), on the
//! probe's tape, `tapes/appb.ron`.
//!
//! Most comparisons are exact: the roles' guarantees (no order admission refuses, no burn that
//! falls short, a zero coefficient that does not bind) are statements about floating-point
//! results. Two bars are relative and named here:
//!
//! - `EXPOST`: 1e-12 of what was held. The ex-post cutoff is the boundary of a bisection over
//!   doubles, so the input that does not bind is left with a few ulps of rounding (a relative
//!   1e-16 of the scale times the sensitivities of J); 1e-12 is far above that and far below any
//!   economic quantity.
//! - `CLOSE`: 1e-14 relative, for a value the test recomputes in another order of operations
//!   than the rule's (a chain of a few products and quotients, each within half an ulp).

use rustyecon_agents::cast::view;
use rustyecon_agents::{
    ActorState, AgentDelta, Agents, Cast, Decision, GoodDeskState, ProviderState, RawScale, RawSpec,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, state_hash, ActorId, Amount, Clock, ClockMethod, CoreError, GoodId, Holder,
    Ledger, LoadError, LoadErrorKind, NodeId, ParamId, Phase, Provenance, RatePerYear, SimState,
    StateDelta, Tape, Unit, World,
};
use rustyecon_markets::{admit, Order, Side};
use std::collections::BTreeSet;

const APPB: &str = include_str!("../../../tapes/appb.ron");
const EXPOST: f64 = 1e-12;
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

/// Replace exactly one occurrence of `from` in the probe tape.
fn edit(from: &str, to: &str) -> String {
    assert_eq!(APPB.matches(from).count(), 1, "{from:?} must occur once");
    APPB.replacen(from, to, 1)
}

fn actor(w: &W, key: &str) -> ActorId {
    w.id_of::<ActorId>(key).expect("an appb actor")
}

fn good(w: &W, key: &str) -> GoodId {
    w.id_of::<GoodId>(key).expect("an appb good")
}

fn param(w: &W, key: &str) -> ParamId {
    w.id_of::<ParamId>(key).expect("an appb param")
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

/// Set an actor's holding of `g` to `q`: burn it all, then mint `q`.
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

fn set_state(s: &mut S, w: &W, a: ActorId, state: ActorState) -> Result<(), CoreError> {
    let d = StateDelta::Actor(AgentDelta::SetState { actor: a, state });
    apply_in(s, w, Phase::Decisions, &[d]).map(|_| ())
}

fn good_desk_state(s: &S, a: ActorId) -> GoodDeskState {
    match s.ext().get(&a) {
        Some(ActorState::GoodDesk(st)) => *st,
        other => panic!("not a good desk: {other:?}"),
    }
}

/// Every actor decides on `s`; the deltas apply in phase 1 and admission then accepts every
/// order, as the engine's tick does. Returns the decisions, by actor.
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
    /// 10^u with u uniform on [lo, hi].
    fn decades(&mut self, lo: f64, hi: f64) -> f64 {
        num::pow(10.0, lo + (hi - lo) * self.unit())
    }
}

#[test]
fn roles_never_overbudget_or_overdraw() {
    // Whatever the prices, coins, holdings, technique and dials, every role's orders pass
    // admission (no OverBudget, no OverPosted, no bad value) after its own transfers apply, and
    // every burn in produce fits its holding. The draws span twelve decades of price and coin,
    // spending shares that round to exactly 1, and markups tilted by up to 8.
    let (w, genesis, cast) = load(APPB);
    let (dg, dm, prov, wk) = (
        actor(&w, "desk.good"),
        actor(&w, "desk.mach"),
        actor(&w, "provider"),
        actor(&w, "workers"),
    );
    let (labour, land, mach, goods, coin) = (
        good(&w, "labour"),
        good(&w, "land"),
        good(&w, "mach"),
        good(&w, "good"),
        good(&w, "coin"),
    );
    let mut d = Draws(0x2026_0926_0002);
    for i in 0..3000 {
        let mut s = genesis.clone();
        for g in [labour, land, mach, goods] {
            set_price(&mut s, &w, g, d.decades(-6.0, 6.0));
        }
        for a in [dg, dm, prov, wk] {
            let c = if i % 17 == 0 {
                0.0
            } else {
                d.decades(-9.0, 9.0)
            };
            set_holding(&mut s, &w, a, coin, c);
        }
        set_holding(&mut s, &w, dg, goods, d.decades(-3.0, 3.0));
        set_holding(&mut s, &w, dm, mach, d.decades(-3.0, 3.0));
        let share = match i % 5 {
            0 => 0.0,
            1 => 1.0,
            _ => d.unit(),
        };
        let st = GoodDeskState {
            share,
            ..good_desk_state(&s, dg)
        };
        set_state(&mut s, &w, dg, ActorState::GoodDesk(st)).expect("a clean state");
        for key in [
            "spend.workers",
            "spend.provider",
            "buffer.desk.good.cash",
            "buffer.desk.mach.cash",
            "adjust.technique",
        ] {
            let v = if i % 7 == 0 {
                1e4
            } else {
                d.decades(-2.0, 3.0)
            };
            set_param(&mut s, &w, key, v);
        }
        set_param(&mut s, &w, "tilt.desk.good", 8.0 * d.unit());
        set_param(&mut s, &w, "tilt.desk.mach", 8.0 * d.unit());
        decide_and_admit(&mut s, &w, &cast);
        // Produce on arbitrary inputs: each role burns within what it holds.
        for (a, g) in [
            (dg, labour),
            (dg, mach),
            (dm, labour),
            (dm, land),
            (dm, mach),
            (prov, goods),
            (prov, land),
            (wk, goods),
            (wk, land),
        ] {
            let q = if d.unit() < 0.1 {
                0.0
            } else {
                d.decades(-6.0, 3.0)
            };
            set_holding(&mut s, &w, a, g, q);
        }
        let ds: Vec<Delta> = cast
            .actors()
            .flat_map(|a| cast.produce(a, &s, &w).expect("produce runs"))
            .collect();
        apply_in(&mut s, &w, Phase::Production, &ds).expect("no burn falls short");
    }
}

#[test]
fn good_desk_makes_output_at_either_corner() {
    // At x = 1 exactly (a human share of 0) the labour coefficient is 0 and does not bind: the
    // desk makes max_scale(M, J(1)) from machine services alone. At x = 0 exactly J(0) = 0 and
    // it makes L from hours alone. num::max_scale(0, 0) is 0, so a rule that kept the zero
    // coefficient in the min would make nothing.
    let (w, genesis, cast) = load(APPB);
    let dg = actor(&w, "desk.good");
    let (labour, mach, goods) = (good(&w, "labour"), good(&w, "mach"), good(&w, "good"));
    for (share, l, m) in [(0.0, 0.0, 3.7), (1.0, 1.3, 0.0)] {
        let mut s = genesis.clone();
        let st = GoodDeskState {
            share,
            ..good_desk_state(&s, dg)
        };
        set_state(&mut s, &w, dg, ActorState::GoodDesk(st)).unwrap();
        set_holding(&mut s, &w, dg, labour, l);
        set_holding(&mut s, &w, dg, mach, m);
        let ds = cast.produce(dg, &s, &w).unwrap();
        // J(1) = η(g0 + g1/(k + 1)) = 0.6 on the tape's schedule.
        let want = if share == 0.0 {
            num::max_scale(m, 0.2 + 0.8 / 2.0).unwrap()
        } else {
            l
        };
        let minted: Vec<f64> = ds
            .iter()
            .filter_map(|d| match d {
                StateDelta::Mint { good: g, qty, .. } if *g == goods => Some(*qty),
                _ => None,
            })
            .collect();
        assert_eq!(minted, [want], "share {share}: {ds:?}");
        assert!(want > 0.0);
        apply_in(&mut s, &w, Phase::Production, &ds).expect("the burns fit");
        assert_eq!(good_desk_state(&s, dg).output, want);
    }
}

#[test]
fn ex_post_assignment_uses_up_both_inputs() {
    // Under Assign::ExPost the desk assigns the tasks to what it holds: the cutoff x_u with
    // L·J(x_u) = M·(1 − x_u), so both inputs are used up (to EXPOST of what was held), whatever
    // the planned technique. With no hours every task goes to machines, and with no machine
    // services to people.
    let text = edit("assign: Planned", "assign: ExPost");
    let (w, genesis, cast) = load(&text);
    let dg = actor(&w, "desk.good");
    let (labour, mach) = (good(&w, "labour"), good(&w, "mach"));
    let mut d = Draws(41);
    for i in 0..500 {
        let mut s = genesis.clone();
        let (l, m) = match i {
            0 => (0.0, 2.5),
            1 => (1.5, 0.0),
            _ => (d.decades(-4.0, 4.0), d.decades(-4.0, 4.0)),
        };
        set_holding(&mut s, &w, dg, labour, l);
        set_holding(&mut s, &w, dg, mach, m);
        let ds = cast.produce(dg, &s, &w).unwrap();
        apply_in(&mut s, &w, Phase::Production, &ds).expect("the burns fit");
        let st = good_desk_state(&s, dg);
        match i {
            0 => assert_eq!(st.used, 0.0),
            1 => assert_eq!(st.used, 1.0),
            _ => {
                assert!(held(&s, dg, labour) <= EXPOST * l, "{l} {m}: hours left");
                assert!(held(&s, dg, mach) <= EXPOST * m, "{l} {m}: machines left");
            }
        }
        assert!(st.output > 0.0);
    }
}

#[test]
fn technique_moves_toward_the_task_measure() {
    // The target human share is 1 − (the measure of the tasks a machine does more cheaply at
    // posted prices): 1 − (w/(η·p_m) − g0)/g1 inside, 0 when w/p_m ≥ γ(1) = 1, 1 when
    // w/p_m ≤ γ(0) = 0.2. The plan closes share(adjust.technique) of its gap each tick.
    let (w, genesis, cast) = load(APPB);
    let dg = actor(&w, "desk.good");
    let (labour, mach) = (good(&w, "labour"), good(&w, "mach"));
    let clock: &Clock = &w.clock;
    let a = clock.share(RatePerYear(
        genesis.param(param(&w, "adjust.technique")).unwrap(),
    ));
    let s0 = good_desk_state(&genesis, dg).share;
    for (wage, pm, target) in [
        (0.9, 1.0, 1.0 - (0.9 - 0.2) / 0.8),
        (1.25, 1.0, 0.0),
        (1.0, 1.0, 0.0),
        (0.1, 1.0, 1.0),
        (0.2, 1.0, 1.0),
    ] {
        let mut s = genesis.clone();
        set_price(&mut s, &w, labour, wage);
        set_price(&mut s, &w, mach, pm);
        let d = cast.decide(dg, &s, &w).unwrap();
        let share = d
            .deltas
            .iter()
            .find_map(|d| match d {
                StateDelta::Actor(AgentDelta::SetState {
                    state: ActorState::GoodDesk(st),
                    ..
                }) => Some(st.share),
                _ => None,
            })
            .expect("the desk records its technique");
        let want = s0 + a * (target - s0);
        assert!(
            (share - want).abs() <= CLOSE * want,
            "w/p_m {}: {share} against {want}",
            wage / pm
        );
    }
}

#[test]
fn households_offer_the_cdf_and_the_provider_funds_one_basket_per_head() {
    // The workers mint and offer N·min(ln1p(w/P_s)/χ_max, 1) hours, exactly; the provider
    // transfers N·P_s at posted prices, or all its coin if that is less, and records both
    // (R12). Both buy baskets of one good and h space from one budget.
    let (w, genesis, cast) = load(APPB);
    let (prov, wk) = (actor(&w, "provider"), actor(&w, "workers"));
    let (labour, land, goods, coin) = (
        good(&w, "labour"),
        good(&w, "land"),
        good(&w, "good"),
        good(&w, "coin"),
    );
    let price = |s: &S, g| s.price(home(&w), g).unwrap();
    let n = 4.0;
    for provider_coin in [None, Some(3.0)] {
        let mut s = genesis.clone();
        if let Some(c) = provider_coin {
            set_holding(&mut s, &w, prov, coin, c);
        }
        let ps = price(&s, goods) + 1.0 * price(&s, land);
        let d = cast.decide(wk, &s, &w).unwrap();
        let hours = n * (num::ln1p(price(&s, labour) / ps) / 1.0).min(1.0);
        assert!(d.deltas.contains(&StateDelta::Mint {
            to: Holder::Actor(wk),
            good: labour,
            qty: hours,
            prov: Provenance::Endowment,
        }));
        assert!(d
            .orders
            .iter()
            .any(|o| o.good == labour && o.side == Side::Sell && o.qty == hours));
        // Baskets: the good and the space in the ratio 1 : h.
        let q = |g| {
            d.orders
                .iter()
                .find(|o| o.good == g && matches!(o.side, Side::Buy { .. }))
                .unwrap()
                .qty
        };
        assert_eq!(q(land), 1.0 * q(goods));
        let d = cast.decide(prov, &s, &w).unwrap();
        let due = n * ps;
        let paid = due.min(held(&s, prov, coin));
        let transfers: Vec<&Delta> = d
            .deltas
            .iter()
            .filter(|d| matches!(d, StateDelta::Transfer { .. }))
            .collect();
        assert_eq!(
            transfers,
            [&StateDelta::Transfer {
                from: Holder::Actor(prov),
                to: Holder::Actor(wk),
                good: coin,
                amount: Amount::Qty(paid),
            }]
        );
        assert!(d.deltas.contains(&StateDelta::Actor(AgentDelta::SetState {
            actor: prov,
            state: ActorState::Provider(ProviderState { due, paid }),
        })));
        assert_eq!(provider_coin.is_some(), paid < due);
        // It offers all T of its land and buys its own space on the market.
        assert!(d
            .orders
            .iter()
            .any(|o| o.good == land && o.side == Side::Sell && o.qty == 10.0));
    }
}

#[test]
fn cash_rule_outlay_follows_coin_and_markup() {
    // A desk spends share(v·μ^κ) of its coin on its two inputs, μ its markup at posted prices:
    // p/c for the good desk, p_m(1 − a)/(λw + br) for the machine desk. The two budgets sum to
    // that outlay, the second the largest remainder that keeps the sum within it. With κ = 0
    // (registered) the markup does not enter.
    let (w, genesis, cast) = load(APPB);
    let (dg, dm) = (actor(&w, "desk.good"), actor(&w, "desk.mach"));
    let (labour, mach, goods, land, coin) = (
        good(&w, "labour"),
        good(&w, "mach"),
        good(&w, "good"),
        good(&w, "land"),
        good(&w, "coin"),
    );
    for kappa in [0.0, 2.0] {
        let mut s = genesis.clone();
        set_param(&mut s, &w, "tilt.desk.good", kappa);
        set_param(&mut s, &w, "tilt.desk.mach", kappa);
        set_price(&mut s, &w, goods, 0.45);
        set_price(&mut s, &w, mach, 0.5);
        let v = s.param(param(&w, "buffer.desk.good.cash")).unwrap();
        let price = |g| s.price(home(&w), g).unwrap();
        let budgets = |d: &Decision| -> f64 {
            d.orders
                .iter()
                .filter_map(|o| match o.side {
                    Side::Buy { budget } => Some(budget),
                    Side::Sell => None,
                })
                .fold(0.0, |a, b| a + b)
        };
        // The good desk: its plan after this tick's technique step.
        let d = cast.decide(dg, &s, &w).unwrap();
        let st = d
            .deltas
            .iter()
            .find_map(|d| match d {
                StateDelta::Actor(AgentDelta::SetState {
                    state: ActorState::GoodDesk(st),
                    ..
                }) => Some(*st),
                _ => None,
            })
            .unwrap();
        let x = 1.0 - st.share;
        let c = st.share * price(labour) + (0.2 * x + 0.8 * x * x / 2.0) * price(mach);
        let mu = price(goods) / c;
        let outlay = w.clock.share(RatePerYear(v * num::pow(mu, kappa))) * held(&s, dg, coin);
        assert!((budgets(&d) - outlay).abs() <= CLOSE * outlay, "κ {kappa}");
        // The machine desk, in the net form.
        let d = cast.decide(dm, &s, &w).unwrap();
        let cm = 0.05 * price(labour) + 0.4 * price(land);
        let mu = price(mach) * (1.0 - 0.3) / cm;
        let outlay = w.clock.share(RatePerYear(v * num::pow(mu, kappa))) * held(&s, dm, coin);
        assert!((budgets(&d) - outlay).abs() <= CLOSE * outlay, "κ {kappa}");
        // It keeps a·q of the machine services it holds and offers the rest.
        let q = outlay / cm;
        let offered = d
            .orders
            .iter()
            .find(|o| o.good == mach && o.side == Side::Sell)
            .unwrap()
            .qty;
        let k = held(&s, dm, mach);
        assert!((offered - (k - (0.3 * q).min(k))).abs() <= CLOSE * k);
    }
}

#[test]
fn role_state_rejects_unclean_values() {
    // G10: the roles' state is hashed (R8), so SetState refuses a value that is not finite or
    // has its sign bit set, a share above 1, and a state of another kind; nothing changes.
    let (w, genesis, _) = load(APPB);
    let dg = actor(&w, "desk.good");
    let prov = actor(&w, "provider");
    let base = good_desk_state(&genesis, dg);
    let bad = [
        GoodDeskState {
            share: f64::NAN,
            ..base
        },
        GoodDeskState {
            share: -0.0,
            ..base
        },
        GoodDeskState {
            output: -1.0,
            ..base
        },
        GoodDeskState {
            scale: f64::INFINITY,
            ..base
        },
        GoodDeskState {
            used: 1.0 + f64::EPSILON,
            ..base
        },
    ];
    for st in bad {
        let mut s = genesis.clone();
        let hash = state_hash(&s);
        let e = set_state(&mut s, &w, dg, ActorState::GoodDesk(st)).unwrap_err();
        assert!(matches!(e, CoreError::BadValue { .. }), "{e:?}");
        assert_eq!(state_hash(&s), hash);
    }
    let mut s = genesis.clone();
    let wrong = ActorState::Provider(ProviderState {
        due: 1.0,
        paid: 1.0,
    });
    assert!(set_state(&mut s, &w, dg, wrong).is_err());
    assert!(set_state(&mut s, &w, prov, wrong).is_ok());
    let fine = GoodDeskState {
        share: 1.0,
        used: 0.0,
        scale: 3.5,
        output: 0.0,
    };
    set_state(&mut s, &w, dg, ActorState::GoodDesk(fine)).unwrap();
    assert_eq!(good_desk_state(&s, dg), fine);
}

#[test]
fn roles_read_only_their_view() {
    // R13: an actor's decision depends on posted prices, its own holding and state, and the
    // params. Changing every other actor's coin, stocks and state leaves it bit for bit.
    let (w, genesis, cast) = load(APPB);
    let coin = good(&w, "coin");
    let goods = good(&w, "good");
    for a in cast.actors() {
        let before = cast.decide(a, &genesis, &w).unwrap();
        let mut s = genesis.clone();
        for b in cast.actors().filter(|&b| b != a) {
            set_holding(&mut s, &w, b, coin, 1e6);
            set_holding(&mut s, &w, b, goods, 123.0);
            if let Some(ActorState::GoodDesk(st)) = s.ext().get(&b).copied() {
                let st = GoodDeskState { share: 0.5, ..st };
                set_state(&mut s, &w, b, ActorState::GoodDesk(st)).unwrap();
            }
        }
        assert_eq!(cast.decide(a, &s, &w).unwrap(), before, "{a}");
    }
}

#[test]
fn role_specs_are_checked_at_load() {
    // The load checks of the four kinds, each with its path: a desk role declared a Pop, an
    // endowment that is not Instant, a genesis share above 1, a transfer to itself, a param of
    // the wrong unit, a good named twice, and a currency traded.
    let cases: [(&str, &str, &str); 7] = [
        (
            r#"(key: "desk.good", kind: Desk,"#,
            r#"(key: "desk.good", kind: Pop,"#,
            "actors[desk.good].spec",
        ),
        (
            r#"(key: "labour", life: Instant,"#,
            r#"(key: "labour", life: Indefinite,"#,
            "actors[workers].spec.labour",
        ),
        (
            "technique: (adjust: \"adjust.technique\", share: 0.",
            "technique: (adjust: \"adjust.technique\", share: 1.",
            "actors[desk.good].spec.technique.share",
        ),
        (
            r#"transfer: (to: "workers""#,
            r#"transfer: (to: "provider""#,
            "actors[provider].spec.transfer.to",
        ),
        (
            r#"endowment: "inst.land""#,
            r#"endowment: "inst.space""#,
            "actors[provider].spec.endowment",
        ),
        (
            r#"output: "good", labour: "labour", mach: "mach""#,
            r#"output: "good", labour: "labour", mach: "labour""#,
            "actors[desk.good].spec.mach",
        ),
        (
            r#"output: "mach", labour: "labour", land: "land""#,
            r#"output: "coin", labour: "labour", land: "land""#,
            "actors[desk.mach].spec.output",
        ),
    ];
    for (from, to, path) in cases {
        let text = edit(from, to);
        let err = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
        };
        assert_eq!(err.path, path, "{to}: {err}");
        if to.contains("share: 1.") {
            assert!(matches!(err.kind, LoadErrorKind::BadValue(_)));
        }
    }
}

#[test]
fn role_specs_round_trip_in_canonical_form() {
    // Every kind and variant survives to_ron and a parse, with the same world.
    let variants = [
        APPB.to_string(),
        edit("assign: Planned", "assign: ExPost"),
        edit(
            r#"scale: Cash((turnover: "buffer.desk.good.cash", tilt: "tilt.desk.good", payout: None)),"#,
            r#"scale: Step((up: "step.up", down: "step.up", dead: "tilt.desk.good", buffer: "buffer.desk.good.cash", payout: (to: "provider", rate: "step.up"), scale: 7.5)),"#,
        )
        .replace(
            "    params: [",
            "    params: [\n        (key: \"step.up\", value: 2.6, unit: RatePerYear, basis: Assumed(\"test\")),",
        ),
        edit(
            r#"tilt: "tilt.desk.mach", payout: None"#,
            r#"tilt: "tilt.desk.mach", payout: Some((to: "provider", rate: "spend.provider", ceiling: "inst.space"))"#,
        ),
    ];
    for text in variants {
        let t = Tape::<Agents>::from_ron(&text).expect("the variant parses");
        let (w1, _) = resolve(&t).expect("the variant resolves");
        Cast::new(&w1).expect("the cast builds");
        let back = Tape::<Agents>::from_ron(&t.to_ron()).expect("to_ron parses");
        assert_eq!(back, t.canonical());
        let (w2, _) = resolve(&back).unwrap();
        assert_eq!(w1.world_id, w2.world_id);
        let kinds: Vec<&str> = t
            .actors
            .iter()
            .map(|a| match &a.spec {
                RawSpec::Provider(_) => "provider",
                RawSpec::Workers(_) => "workers",
                RawSpec::GoodDesk(d) => match d.scale {
                    RawScale::Cash(_) => "good cash",
                    RawScale::Step(_) => "good step",
                },
                RawSpec::MachDesk(_) => "mach",
                RawSpec::Scripted(_) => "scripted",
                RawSpec::BasketProvider(_)
                | RawSpec::BasketWorkers(_)
                | RawSpec::CategoryDesk(_)
                | RawSpec::TypeDesk(_) => "many",
            })
            .collect();
        assert_eq!(kinds.len(), 4);
        assert!(!kinds.contains(&"scripted"));
    }
}

#[test]
fn role_sites_name_their_methods() {
    // D10 item 4 (S2.2), docs/probe/RULES.md §2: a `RatePerYear` is a share of a stock where
    // a rule draws on one and a log step where it moves a scale; the site says which. Under
    // July's step rule (the negative control) both kinds sit in one desk: `up` and `down` step
    // the scale by exp(step·(m ∓ dead)), a log step; the buffer, the payout and every spending
    // and adjustment rate draw a share. The price rates are core's log steps (ENGINE §6).
    let step = |desk: &str| {
        format!(
            r#"scale: Step((up: "step.desk.{desk}.up", down: "step.desk.{desk}.down", dead: "dead.desk.{desk}", buffer: "buffer.desk.{desk}.cash", payout: (to: "provider", rate: "payout.desk.{desk}"), scale: 7.5)),"#
        )
    };
    let mut text = edit(
        r#"scale: Cash((turnover: "buffer.desk.good.cash", tilt: "tilt.desk.good", payout: None)),"#,
        &step("good"),
    );
    text = text.replacen(
        r#"scale: Cash((turnover: "buffer.desk.mach.cash", tilt: "tilt.desk.mach", payout: None)),"#,
        &step("mach"),
        1,
    );
    // The tilts belong to the cash rule; the step rule's own dials replace them.
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| !l.contains(r#"(key: "tilt.desk."#))
        .map(str::to_string)
        .collect();
    let at = lines.iter().position(|l| l == "    params: [").unwrap();
    for desk in ["good", "mach"] {
        for (key, value, unit) in [
            (format!("step.desk.{desk}.up"), 2.6, "RatePerYear"),
            (format!("step.desk.{desk}.down"), 1.3, "RatePerYear"),
            (format!("dead.desk.{desk}"), 0.01, "Dimensionless"),
            (format!("payout.desk.{desk}"), 5.2, "RatePerYear"),
        ] {
            lines.insert(
                at + 1,
                format!(
                    r#"        (key: "{key}", value: {value:?}, unit: {unit}, basis: Assumed("test")),"#
                ),
            );
        }
    }
    let (w, s, _) = load(&lines.join("\n"));
    let rates: Vec<(String, ClockMethod)> = w
        .registry
        .params()
        .iter()
        .filter(|p| p.unit == Unit::RatePerYear)
        .flat_map(|p| p.sites.iter().map(|s| (s.path.clone(), s.method)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let (share, log_step) = (ClockMethod::Share, ClockMethod::LogStep);
    let mut want: Vec<(String, ClockMethod)> = Vec::new();
    for desk in ["desk.good", "desk.mach"] {
        want.push((format!("actors[{desk}].spec.scale.buffer"), share));
        want.push((format!("actors[{desk}].spec.scale.down"), log_step));
        want.push((format!("actors[{desk}].spec.scale.payout.rate"), share));
        want.push((format!("actors[{desk}].spec.scale.up"), log_step));
    }
    want.push(("actors[desk.good].spec.technique.adjust".into(), share));
    want.push(("actors[provider].spec.spend".into(), share));
    want.push(("actors[workers].spec.spend".into(), share));
    for good in ["good", "labour", "land", "mach"] {
        want.push((format!("goods[{good}].price_rate"), log_step));
    }
    want.sort();
    assert_eq!(rates, want);
    // The spec holds the same method at each path, and reading it through the actor's view
    // gives the Clock's own conversion of the param's value.
    let mut checked = 0;
    for decl in &w.actors {
        let state = s.ext().get(&decl.id).unwrap();
        let v = view(decl, &s, &w, state).unwrap();
        for (path, site) in decl.spec.sites(&w) {
            let path = format!("actors[{}].spec.{path}", decl.key);
            let value = s.param(site.param).unwrap();
            let clock = &w.clock;
            let expected = match want.iter().find(|(p, _)| *p == path) {
                Some((_, m)) if *m == log_step => clock.log_step(RatePerYear(value)),
                Some(_) => clock.share(RatePerYear(value)),
                None => continue,
            };
            assert_eq!(site.per_tick(&v.params, v.clock), Ok(expected), "{path}");
            checked += 1;
        }
    }
    assert_eq!(checked, want.len() - 4, "every role rate site");
}
