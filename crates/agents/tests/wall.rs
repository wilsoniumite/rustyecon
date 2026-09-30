//! The wall's additions to the many-market roles (P2.3; docs/probe/WALL-RULES.md; the wall
//! frame's §3, docs/probe/wall/SPEC.md): the category desk's optional `tail` and `reserved`, and
//! the provider's optional `more`, on the wall tape `tapes/markets-iw1.ron` and on I1's.
//!
//! As in `many.rs`, most comparisons are exact. One bar is relative and named here: `CLOSE`,
//! 1e-14 relative, for a value the test recomputes in another order of operations than the
//! rule's (a chain of a few products, sums and quotients, each within half an ulp).

use rustyecon_agents::{
    ActorState, AgentDelta, Agents, Cast, Decision, GoodDeskState, ProviderState,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, CoreError, FlowPerYear, GoodId, Holder, Ledger, LoadError,
    NodeId, ParamId, Phase, Provenance, RatePerYear, SimState, StateDelta, Tape, World,
};
use rustyecon_markets::Side;

const IW1: &str = include_str!("../../../tapes/markets-iw1.ron");
const I1: &str = include_str!("../../../tapes/markets-i1.ron");
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

fn set_share(s: &mut S, w: &W, a: ActorId, share: f64) {
    if let Some(ActorState::GoodDesk(st)) = s.ext().get(&a).copied() {
        let d = StateDelta::Actor(AgentDelta::SetState {
            actor: a,
            state: ActorState::GoodDesk(GoodDeskState { share, ..st }),
        });
        apply_in(s, w, Phase::Decisions, &[d]).expect("a clean state");
    }
}

fn buy_of(d: &Decision, g: GoodId) -> Option<(f64, f64)> {
    d.orders
        .iter()
        .find(|o| o.good == g)
        .and_then(|o| match o.side {
            Side::Buy { budget } => Some((o.qty, budget)),
            Side::Sell => None,
        })
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= CLOSE * a.abs().max(b.abs())
}

/// J(x) = η·(g0·x + g1·x²/2) on the wall's schedule (η 0.5, g0 0.2, g1 0.8, k 1).
fn j(x: f64) -> f64 {
    0.5 * (0.2 * x + 0.8 * x * x / 2.0)
}

#[test]
fn category_desk_buys_its_tail_and_reserved_hours() {
    // The wall frame's §3.2 on IW1's services desk (one segment of density 0.75, tail L^H 0.1,
    // the trained's reserved hours R 0.04, no direct land), at a technique held fixed: the
    // hours per unit are h = μ·s + L^H, whatever x, so hours are bought even at the wall
    // (s = 0); the unit cost is c = w·h + p_τ·M/θ + w_T·R; the desk buys h·q hours, (M/θ)·q
    // machine services and R·q of the trained's hours, each with budget p·(coef·q), q =
    // share(turnover)·coin/c. Produce is Leontief over hours, services and the reserved hours,
    // each in turn the binding one, and burns coef·y of each. Without the tail the hours at
    // s = 0 are 0; without the reserved input there is no order on `labour.trained` and the
    // cost, and so every quantity, is lower.
    let (w, genesis, cast) = load(IW1);
    let d = actor(&w, "desk.services");
    let (labour, mach, trained, land, coin) = (
        good(&w, "labour"),
        good(&w, "mach"),
        good(&w, "labour.trained"),
        good(&w, "land"),
        good(&w, "coin"),
    );
    let (pw, pm, pt, r) = (0.93, 0.71, 1.37, 1.09);
    for share in [0.0, 0.05, 0.4, 1.0] {
        let mut s = genesis.clone();
        set_param(&mut s, &w, "adjust.technique.services", 0.0);
        set_share(&mut s, &w, d, share);
        for (g, p) in [(labour, pw), (mach, pm), (trained, pt), (land, r)] {
            set_price(&mut s, &w, g, p);
        }
        let dec = cast.decide(d, &s, &w).unwrap();
        let x = 1.0 - share;
        let h = 0.75 * share + 0.1;
        let ms = 0.75 * j(x) / 1.0;
        let c = pw * h + pm * ms + pt * 0.04;
        let v = s.param(param(&w, "buffer.desk.services.cash")).unwrap();
        let q = w.clock.share(RatePerYear(v)) * held(&s, d, coin) / c;
        let (hours, hb) = buy_of(&dec, labour).expect("hours are bought, the tail's at the wall");
        assert!(
            close(hours, h * q),
            "s {share}: hours {hours} against {}",
            h * q
        );
        assert!(close(hb, pw * hours), "s {share}: the hours' budget");
        let (serv, _) = buy_of(&dec, mach).expect("services are posted");
        assert!(
            close(serv, ms * q),
            "s {share}: services {serv} against {}",
            ms * q
        );
        let (res, rb) = buy_of(&dec, trained).expect("the reserved hours are bought");
        assert!(
            close(res, 0.04 * q),
            "s {share}: reserved {res} against {}",
            0.04 * q
        );
        assert!(close(rb, pt * res), "s {share}: the reserved hours' budget");
        assert!(buy_of(&dec, land).is_none(), "no direct land");
        assert!(
            buy_of(&dec, good(&w, "labour.master")).is_none(),
            "services reserve no master's hours"
        );
    }
    // Produce at the wall's corner s = 0, each input in turn the binding one.
    for bind in 0..3 {
        let mut t = genesis.clone();
        set_share(&mut t, &w, d, 0.0);
        let big = 1e3;
        let hold = [
            (labour, if bind == 0 { 0.07 } else { big }),
            (mach, if bind == 1 { 0.07 } else { big }),
            (trained, if bind == 2 { 0.07 } else { big }),
        ];
        for (g, q) in hold {
            set_holding(&mut t, &w, d, g, q);
        }
        // At s = 0 the hours per unit are the tail alone, 0.75·0 + 0.1, and M/θ is μ·J(1).
        let coefs = [0.75 * 0.0 + 0.1, 0.75 * j(1.0), 0.04];
        let y = (0..3)
            .map(|i| num::max_scale(hold[i].1, coefs[i]).unwrap())
            .fold(f64::INFINITY, f64::min);
        let ds = cast.produce(d, &t, &w).unwrap();
        let services = good(&w, "services");
        let minted: Vec<f64> = ds
            .iter()
            .filter_map(|x| match x {
                StateDelta::Mint { good: g, qty, .. } if *g == services => Some(*qty),
                _ => None,
            })
            .collect();
        assert_eq!(minted, [y], "bind {bind}");
        for (g, coef) in [(labour, coefs[0]), (trained, coefs[2])] {
            assert!(
                ds.contains(&StateDelta::Burn {
                    from: Holder::Actor(d),
                    good: g,
                    amount: Amount::Qty(coef * y),
                    prov: Provenance::Production,
                }),
                "bind {bind}: burns {coef}·y of {g}"
            );
        }
        apply_in(&mut t, &w, Phase::Production, &ds).expect("the burns fit");
    }
}

#[test]
fn provider_pays_every_transfer_in_list_order() {
    // The wall frame's §3.3 on IW1's provider: N·P_s to the workers, then N_i·P_s to each
    // reserved pop in list order (the trained, the master), each from the coin the copy still
    // holds; its state holds the sums, due = ((N·P_s) + N_T·P_s) + N_M·P_s. With coin for the
    // first and half the second, the second is paid what is left and the third nothing (no
    // transfer is written), and nothing is left to spend on baskets.
    let (w, genesis, cast) = load(IW1);
    let p = actor(&w, "provider");
    let coin = good(&w, "coin");
    let price = |s: &S, k: &str| s.price(home(&w), good(&w, k)).unwrap();
    let ps = (0.0 + 1.0 * price(&genesis, "services"))
        + 1.0 * price(&genesis, "goods")
        + 1.0 * price(&genesis, "land");
    let heads = [130.0, 52.0, 26.0].map(|n| w.clock.flow(FlowPerYear(n)));
    let to = ["workers", "workers.trained", "workers.master"].map(|k| actor(&w, k));
    let transfers = |d: &Decision| -> Vec<(ActorId, f64)> {
        d.deltas
            .iter()
            .filter_map(|x| match x {
                StateDelta::Transfer {
                    to: Holder::Actor(a),
                    amount: Amount::Qty(q),
                    ..
                } => Some((*a, *q)),
                _ => None,
            })
            .collect()
    };
    let state = |d: &Decision| -> ProviderState {
        d.deltas
            .iter()
            .find_map(|x| match x {
                StateDelta::Actor(AgentDelta::SetState {
                    state: ActorState::Provider(st),
                    ..
                }) => Some(*st),
                _ => None,
            })
            .expect("the provider records its transfer")
    };
    let due: Vec<f64> = heads.iter().map(|n| n * ps).collect();
    let dec = cast.decide(p, &genesis, &w).unwrap();
    assert_eq!(
        transfers(&dec),
        vec![(to[0], due[0]), (to[1], due[1]), (to[2], due[2])]
    );
    let total = (due[0] + due[1]) + due[2];
    assert_eq!(
        state(&dec),
        ProviderState {
            due: total,
            paid: total
        }
    );
    let mut short = genesis.clone();
    let c = due[0] + 0.5 * due[1];
    set_holding(&mut short, &w, p, coin, c);
    let dec = cast.decide(p, &short, &w).unwrap();
    let left = c - due[0];
    assert_eq!(transfers(&dec), vec![(to[0], due[0]), (to[1], left)]);
    assert_eq!(
        state(&dec),
        ProviderState {
            due: total,
            paid: (due[0] + left) + 0.0
        }
    );
    assert!(dec.orders.iter().all(|o| match o.side {
        Side::Buy { budget } => budget == 0.0,
        Side::Sell => true,
    }));
}

#[test]
fn wall_specs_are_checked_at_load() {
    // The wall frame's §3.8: a reserved good named twice, or equal to the desk's labour; a
    // further transfer to the provider itself, to the workers `transfer` already pays, or to a
    // recipient named twice; and a tail param of the wrong unit. Each is refused with its path.
    let edit = |from: &str, to: &str| {
        assert!(IW1.contains(from), "{from:?}");
        IW1.replacen(from, to, 1)
    };
    let reserved =
        r#"reserved: [(good: "labour.trained", coef: "inst.services.reserved.trained")]"#;
    let more = r#"more: [(to: "workers.trained", heads: "inst.trained.workers"), (to: "workers.master", heads: "inst.master.workers")]"#;
    let cases = [
        (
            edit(
                reserved,
                r#"reserved: [(good: "labour.trained", coef: "inst.services.reserved.trained"), (good: "labour.trained", coef: "inst.services.reserved.trained")]"#,
            ),
            "actors[desk.services].spec.reserved[labour.trained].good",
        ),
        (
            edit(
                reserved,
                r#"reserved: [(good: "labour", coef: "inst.services.reserved.trained")]"#,
            ),
            "actors[desk.services].spec.reserved[labour].good",
        ),
        (
            edit(
                more,
                r#"more: [(to: "provider", heads: "inst.trained.workers"), (to: "workers.master", heads: "inst.master.workers")]"#,
            ),
            "actors[provider].spec.more[provider].to",
        ),
        (
            edit(
                more,
                r#"more: [(to: "workers", heads: "inst.trained.workers"), (to: "workers.master", heads: "inst.master.workers")]"#,
            ),
            "actors[provider].spec.more[workers].to",
        ),
        (
            edit(
                more,
                r#"more: [(to: "workers.trained", heads: "inst.trained.workers"), (to: "workers.trained", heads: "inst.master.workers")]"#,
            ),
            "actors[provider].spec.more[workers.trained].to",
        ),
        (
            edit(
                r#"(key: "inst.services.tail", value: 0.1, unit: Dimensionless,"#,
                r#"(key: "inst.services.tail", value: 0.1, unit: FlowPerYear,"#,
            ),
            "actors[desk.services].spec.tail",
        ),
    ];
    for (text, path) in cases {
        let err = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
        };
        assert_eq!(err.path, path, "{err}");
    }
    // The workers of `more` sell the reserved hours: an Instant good. A reserved good that
    // lives longer is refused as any bought good that outlives the tick is (M6).
    let long = edit(
        r#"(key: "labour.trained", life: Instant,"#,
        r#"(key: "labour.trained", life: Years("life.one_tick"),"#,
    );
    let (w, _) = load_text(&long).expect("the tape resolves");
    let err = Cast::new(&w).expect_err("a pop's endowment must be Instant");
    assert_eq!(err.path, "actors[workers.trained].spec.labour", "{err}");
}

/// I1's tape with the wall's fields at zero (the wall frame's §3.8): every category desk with a
/// tail of 0 and a reserved input at coefficient 0 on `zlabour`, and the provider with a further
/// transfer to `zworkers`, a pop of 0 heads that sells `zlabour`. Every new key sorts after the
/// old ones of its kind.
fn zeroed_i1() -> String {
    let mut t = I1.to_string();
    let mut rep = |from: &str, to: &str| {
        assert_eq!(t.matches(from).count(), 1, "{from:?}");
        t = t.replacen(from, to, 1);
    };
    rep(
        "    params: [\n",
        "    params: [\n        (key: \"zz.tail\", value: 0.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.reserved\", value: 0.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.heads\", value: 0.0, unit: FlowPerYear, basis: Assumed(\"test\")),\n        (key: \"zz.chi_max\", value: 1.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"rate.zlabour\", value: 5.2, unit: RatePerYear, basis: Assumed(\"test\")),\n",
    );
    rep(
        "    goods: [\n",
        "    goods: [\n        (key: \"zlabour\", life: Instant, price_rate: Some(\"rate.zlabour\")),\n",
    );
    rep(
        "\"owners\", \"workers\"],",
        "\"owners\", \"workers\", \"zworkers\"],",
    );
    for c in ["manufactures", "food", "care", "shelter"] {
        rep(
            &format!("tilt: \"tilt.desk.{c}\", payout: None)),\n"),
            &format!(
                "tilt: \"tilt.desk.{c}\", payout: None)),\n            tail: Some(\"zz.tail\"),\n            reserved: [(good: \"zlabour\", coef: \"zz.reserved\")],\n"
            ),
        );
    }
    rep(
        "            spend: \"spend.provider\",\n",
        "            spend: \"spend.provider\",\n            more: [(to: \"zworkers\", heads: \"zz.heads\")],\n",
    );
    let basket = I1
        .lines()
        .find(|l| l.trim_start().starts_with("basket:"))
        .expect("a basket");
    rep(
        "    ],\n    genesis: (\n",
        &format!(
            "        (key: \"zworkers\", kind: Pop, class: \"zworkers\", home: \"home\", basis: Assumed(\"test\"),\n         spec: BasketWorkers((\n            labour: \"zlabour\", heads: \"zz.heads\", chi_max: \"zz.chi_max\",\n{basket}\n            spend: \"spend.workers\",\n         ))),\n    ],\n    genesis: (\n"
        ),
    );
    rep(
        "        prices: [\n",
        "        prices: [\n            (node: \"home\", good: \"zlabour\", price: 1.0),\n",
    );
    t
}

#[test]
fn wall_fields_at_zero_leave_every_decision() {
    // The rule-level nesting of the wall frame's §3.8 (R1): I1's tape with a tail of 0 on
    // every category desk, a reserved input at coefficient 0 on each, and a further transfer to
    // a pop of 0 heads that sells a labour good of its own. The new good and pop sort after
    // every old key, so every old id is kept, and each old actor's `decide` and `produce` equal
    // the field-free tape's, order for order and delta for delta, from random states. A tail or
    // a reserved input that entered the cost, the orders or the Leontief other than by adding 0,
    // or a transfer of 0 that wrote a delta, would part them.
    let text = zeroed_i1();
    let (wb, gb) = load_text(&text).expect("the tape with the fields at zero loads");
    let cb = Cast::new(&wb).expect("its cast builds");
    let (wa, ga, ca) = load(I1);
    let old: Vec<ActorId> = ca.actors().collect();
    for &a in &old {
        assert_eq!(wa.key_of(a), wb.key_of(a), "the old ids are kept");
    }
    for k in [
        "coin",
        "labour",
        "land",
        "mach",
        "manufactures",
        "food",
        "care",
        "shelter",
    ] {
        assert_eq!(good(&wa, k), good(&wb, k));
    }
    let goods: Vec<GoodId> = (0..wa.goods.len() as u32)
        .map(GoodId)
        .filter(|g| !wa.is_currency(*g))
        .collect();
    let coin = good(&wa, "coin");
    let mut state = 0x2026_0930_0001u64;
    let mut unit = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    for i in 0..500 {
        let (mut sa, mut sb) = (ga.clone(), gb.clone());
        for &g in &goods {
            let p = num::pow(10.0, -3.0 + 6.0 * unit());
            set_price(&mut sa, &wa, g, p);
            set_price(&mut sb, &wb, g, p);
        }
        for &a in &old {
            let c = if i % 13 == 0 {
                0.0
            } else {
                num::pow(10.0, -4.0 + 8.0 * unit())
            };
            set_holding(&mut sa, &wa, a, coin, c);
            set_holding(&mut sb, &wb, a, coin, c);
            let share = match i % 4 {
                0 => 0.0,
                1 => 1.0,
                _ => unit(),
            };
            set_share(&mut sa, &wa, a, share);
            set_share(&mut sb, &wb, a, share);
        }
        for &a in &old {
            assert_eq!(
                ca.decide(a, &sa, &wa).unwrap(),
                cb.decide(a, &sb, &wb).unwrap(),
                "draw {i}: {}",
                wa.key_of(a).unwrap()
            );
            for &g in &goods {
                let q = if unit() < 0.1 {
                    0.0
                } else {
                    num::pow(10.0, -3.0 + 5.0 * unit())
                };
                set_holding(&mut sa, &wa, a, g, q);
                set_holding(&mut sb, &wb, a, g, q);
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

#[test]
fn wall_specs_round_trip_and_old_worlds_keep_their_ids() {
    // The three fields are optional and skipped when absent (R1): IW1 survives to_ron and a
    // parse with the same world and writes them back; a tape without them keeps its canonical
    // text; and the resolved actors leave them out of `world_id`, so I1's world is unchanged.
    let t = Tape::<Agents>::from_ron(IW1).unwrap();
    let (w1, _) = resolve(&t).unwrap();
    Cast::new(&w1).unwrap();
    let back = Tape::<Agents>::from_ron(&t.to_ron()).unwrap();
    assert_eq!(back, t.canonical());
    let (w2, _) = resolve(&back).unwrap();
    assert_eq!(w1.world_id, w2.world_id);
    let text = t.to_ron();
    assert!(text.contains("tail: Some(\"inst.services.tail\")"));
    assert_eq!(text.matches("reserved: [").count(), 2);
    assert!(text.contains("coef: \"inst.goods.reserved.master\""));
    assert!(text.contains("more: ["));
    let i1 = Tape::<Agents>::from_ron(I1).unwrap();
    let canon = i1.to_ron();
    for field in ["tail:", "reserved:", "more:"] {
        assert!(
            !canon.contains(field),
            "{field} is written on a tape without it"
        );
    }
    // I1's world_id as P2.1 recorded it (P2.2b's streams, unchanged since).
    let (wi, _) = resolve(&i1).unwrap();
    assert_eq!(wi.world_id, 0x965e_dbef_2664_b850);
}
