//! The commons' addition to the many-market roles (P2.3; docs/probe/COMMONS-RULES.md; the
//! commons frame's §3, docs/probe/commons/SPEC.md): the workers' optional `exit`, the priced exit
//! with a commons they hold, on the commons tape `tapes/markets-c1.ron` and on I1's.
//!
//! As in `many.rs`, most comparisons are exact. One bar is relative and named here: `CLOSE`,
//! 1e-14 relative, for a value the test recomputes in another order of operations than the
//! rule's (a chain of a few products, sums and quotients, each within half an ulp).

use rustyecon_agents::{ActorState, AgentDelta, Agents, Cast, Decision};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, CoreError, FlowPerYear, GoodId, Holder, Ledger, LoadError,
    NodeId, ParamId, Phase, Provenance, RatePerYear, SimState, StateDelta, Tape, World,
};
use rustyecon_markets::{admit, Order, Side};

const C1: &str = include_str!("../../../tapes/markets-c1.ron");
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

fn buy_of(d: &Decision, g: GoodId) -> Option<(f64, f64)> {
    d.orders
        .iter()
        .find(|o| o.good == g)
        .and_then(|o| match o.side {
            Side::Buy { budget } => Some((o.qty, budget)),
            Side::Sell => None,
        })
}

fn sell_of(d: &Decision, g: GoodId) -> Option<f64> {
    d.orders
        .iter()
        .find(|o| o.good == g && matches!(o.side, Side::Sell))
        .map(|o| o.qty)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= CLOSE * a.abs().max(b.abs())
}

fn per_tick(s: &S, w: &W, key: &str) -> f64 {
    w.clock
        .flow(FlowPerYear(s.param(param(w, key)).expect("a param")))
}

fn value(s: &S, w: &W, key: &str) -> f64 {
    s.param(param(w, key)).expect("a param")
}

/// A deterministic stream of units.
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

/// I1's tape with the workers' exit block: s₀, s̲ and h (`zz.exit.*`, Dimensionless) and the
/// commons T_o a year (`zz.commons`, FlowPerYear), exit good food and land the provider's. The
/// new param keys sort after every old one, so every old id is kept.
fn with_exit(s0: f64, sf: f64, h: f64, to: f64) -> String {
    let mut t = I1.to_string();
    let mut rep = |from: &str, to: &str| {
        assert_eq!(t.matches(from).count(), 1, "{from:?}");
        t = t.replacen(from, to, 1);
    };
    rep(
        "    params: [\n",
        &format!(
            "    params: [\n        (key: \"zz.exit.gross\", value: {s0:?}, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.exit.floor\", value: {sf:?}, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.exit.plot\", value: {h:?}, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.commons\", value: {to:?}, unit: FlowPerYear, basis: Assumed(\"test\")),\n"
        ),
    );
    rep(
        "            spend: \"spend.workers\",\n",
        "            spend: \"spend.workers\",\n            exit: Some((good: \"food\", gross: \"zz.exit.gross\", floor: \"zz.exit.floor\", plot: \"zz.exit.plot\", commons: \"zz.commons\", land: \"land\")),\n",
    );
    t
}

#[test]
fn workers_without_exit_are_p21s() {
    // The commons frame's §3.7 test 1 and §3.1's evaluation order, at the rule level (R1): with
    // `exit` absent (I1's tape) the workers offer N·min(ln1p(w/P_s)/χ_max, 1) hours bit for bit,
    // P_s summed from 0.0 in item order; and with an exit block switched off (s₀ = s̲ = 0, with
    // h 0 or 0.12 and a commons of 0 or 30 a year) they decide and produce exactly what the
    // exit-free workers do, order for order, delta for delta and state for state, over 3,000
    // random states: prices over twelve decades, coins, holdings, heads, χ_max and spending.
    // The switched-off workers hold no land (they never buy it), so land is left out of the
    // holdings drawn for them.
    let (wa, ga, ca) = load(I1);
    let workers = actor(&wa, "workers");
    let goods: Vec<GoodId> = (0..wa.goods.len() as u32)
        .map(GoodId)
        .filter(|g| !wa.is_currency(*g))
        .collect();
    let (coin, labour, land) = (good(&wa, "coin"), good(&wa, "labour"), good(&wa, "land"));
    let items = ["manufactures", "food", "care", "shelter"];
    let mut d = Draws(0x2026_0930_c001);
    for (k, (h, to)) in [(0.0, 0.0), (0.12, 30.0), (0.0, 30.0), (0.12, 0.0)]
        .into_iter()
        .enumerate()
    {
        let text = with_exit(0.0, 0.0, h, to);
        let (wb, gb, cb) = load(&text);
        assert_eq!(actor(&wb, "workers"), workers);
        for &g in &goods {
            assert_eq!(wb.key_of(g), wa.key_of(g), "the goods keep their ids");
        }
        for i in 0..750 {
            let (mut sa, mut sb) = (ga.clone(), gb.clone());
            for &g in &goods {
                let p = d.decades(-6.0, 6.0);
                set_price(&mut sa, &wa, g, p);
                set_price(&mut sb, &wb, g, p);
            }
            let c = if i % 17 == 0 {
                0.0
            } else {
                d.decades(-9.0, 9.0)
            };
            set_holding(&mut sa, &wa, workers, coin, c);
            set_holding(&mut sb, &wb, workers, coin, c);
            for (key, lo, hi) in [
                ("inst.workers", -2.0, 4.0),
                ("inst.chi_max", -2.0, 1.0),
                ("spend.workers", -2.0, 4.0),
            ] {
                let v = d.decades(lo, hi);
                set_param(&mut sa, &wa, key, v);
                set_param(&mut sb, &wb, key, v);
            }
            let da = ca.decide(workers, &sa, &wa).unwrap();
            let db = cb.decide(workers, &sb, &wb).unwrap();
            assert_eq!(da, db, "case {k}, draw {i}: the decision");
            // P2.1's hours, recomputed: P_s from 0.0 in item order, then N·min(F, 1).
            let mut ps = 0.0;
            for it in items {
                ps += value(&sa, &wa, &format!("inst.{it}.weight"))
                    * sa.price(home(&wa), good(&wa, it)).unwrap();
            }
            let w = sa.price(home(&wa), labour).unwrap();
            let f = (num::ln1p(w / ps) / value(&sa, &wa, "inst.chi_max")).min(1.0);
            let n = per_tick(&sa, &wa, "inst.workers");
            assert_eq!(
                sell_of(&da, labour),
                Some(n * f),
                "case {k}, draw {i}: the hours"
            );
            assert!(buy_of(&db, land).is_none(), "no plot is rented");
            for &g in &goods {
                if g == land {
                    continue;
                }
                let q = if d.unit() < 0.1 {
                    0.0
                } else {
                    d.decades(-6.0, 3.0)
                };
                set_holding(&mut sa, &wa, workers, g, q);
                set_holding(&mut sb, &wb, workers, g, q);
            }
            assert_eq!(
                ca.produce(workers, &sa, &wa).unwrap(),
                cb.produce(workers, &sb, &wb).unwrap(),
                "case {k}, draw {i}: produce"
            );
        }
    }
}

/// F(e) and the plots' rent as the commons frame's §3.1 writes them, recomputed here.
fn f_of(chi: f64, w: f64, ps: f64, e: f64) -> f64 {
    (num::ln1p((w - e) / (ps + e)) / chi).clamp(0.0, 1.0)
}

#[test]
fn plots_rent_enclosed_land() {
    // The commons frame's §3.1 and §3.2 on C1's tape, the regime set by the commons T_o at the
    // tape's genesis prices. Enclosed (T_o 2.6 a year): the workers offer n(r) hours, post one
    // buy of T_p = h·(N − n(r)) − T_o land at r with budget r·T_p, and their baskets' budget is
    // share(spend)·(C − r·T_p), so the chain's total is r·T_p + share(spend)·(C − r·T_p); their
    // state holds F(e_r). With too little coin for the rent the whole coin goes to the plots and
    // no basket is bought. Crowded (the tape's commons): N − T_o/h hours and no land order;
    // Commons (a commons of 500 a year): n(0) and no land order; the split (land at 3, where no
    // plot pays): the floor's n(p_g·s̲), no land order. In produce the workers burn what they
    // hold of the land as `Consumption`, all of it, beside their baskets.
    let (w, genesis, cast) = load(C1);
    let workers = actor(&w, "workers");
    let (coin, labour, land, food) = (
        good(&w, "coin"),
        good(&w, "labour"),
        good(&w, "land"),
        good(&w, "food"),
    );
    let items = ["manufactures", "food", "care", "shelter"];
    let at = |s: &S, g: GoodId| s.price(home(&w), g).unwrap();
    let n = per_tick(&genesis, &w, "inst.workers");
    let chi = value(&genesis, &w, "inst.chi_max");
    let (s0, h) = (
        value(&genesis, &w, "inst.exit.gross"),
        value(&genesis, &w, "inst.exit.plot"),
    );
    let ps_of = |s: &S| {
        let mut ps = 0.0;
        for it in items {
            ps += value(s, &w, &format!("inst.{it}.weight")) * at(s, good(&w, it));
        }
        ps
    };
    // Enclosed.
    let mut s = genesis.clone();
    set_param(&mut s, &w, "inst.commons", 2.6);
    let to = per_tick(&s, &w, "inst.commons");
    let (wage, r, pg, ps) = (at(&s, labour), at(&s, land), at(&s, food), ps_of(&s));
    let fr = f_of(chi, wage, ps, num::fma(-r, h, pg * s0));
    let nr = n * fr;
    assert!(r * h < pg * s0 && nr <= n - to / h, "the plots spill");
    let tp = h * (n - nr) - to;
    let dec = cast.decide(workers, &s, &w).unwrap();
    assert_eq!(sell_of(&dec, labour), Some(nr));
    let (q, b) = buy_of(&dec, land).expect("one buy of land for the plots");
    assert_eq!(q, tp);
    assert_eq!(b, r * tp);
    let c = held(&s, workers, coin);
    let spend = w.clock.share(RatePerYear(value(&s, &w, "spend.workers")));
    let budget = spend * (c - r * tp);
    let mut total = b;
    for it in items {
        let g = good(&w, it);
        let (qi, bi) = buy_of(&dec, g).expect("a basket item");
        let z = value(&s, &w, &format!("inst.{it}.weight"));
        assert!(close(qi, z * budget / ps), "{it}: {qi}");
        assert!(close(bi, at(&s, g) * qi), "{it}: its budget");
        total += bi;
    }
    assert!(close(total, r * tp + budget), "the chain's total");
    assert!(dec.deltas.iter().any(|x| matches!(
        x,
        StateDelta::Actor(AgentDelta::SetState {
            state: ActorState::Workers(st),
            ..
        }) if st.share == fr
    )));
    // The whole order set passes admission.
    let orders: Vec<Order> = dec.orders.clone();
    apply_in(&mut s, &w, Phase::Decisions, &dec.deltas).unwrap();
    admit(orders, &s, &w).expect("admission accepts the workers' orders");
    // Too little coin: all of it to the plots.
    let mut t = genesis.clone();
    set_param(&mut t, &w, "inst.commons", 2.6);
    set_holding(&mut t, &w, workers, coin, 0.5 * r * tp);
    let dec = cast.decide(workers, &t, &w).unwrap();
    let (q, b) = buy_of(&dec, land).unwrap();
    assert_eq!((q, b), (tp, 0.5 * r * tp));
    for it in items {
        assert_eq!(buy_of(&dec, good(&w, it)).unwrap().1, 0.0, "{it}");
    }
    // Produce: the land held is burned as Consumption, all of it.
    let mut u = genesis.clone();
    set_holding(&mut u, &w, workers, land, 0.37);
    let ds = cast.produce(workers, &u, &w).unwrap();
    let burns: Vec<&Delta> = ds
        .iter()
        .filter(|x| matches!(x, StateDelta::Burn { good, .. } if *good == land))
        .collect();
    assert_eq!(burns.len(), 1);
    assert!(matches!(
        burns[0],
        StateDelta::Burn {
            amount: Amount::Qty(q),
            prov: Provenance::Consumption,
            ..
        } if *q == 0.37
    ));
    apply_in(&mut u, &w, Phase::Production, &ds).expect("the burns fit");
    assert_eq!(held(&u, workers, land), 0.0);
    // Crowded at the tape's genesis.
    let s = genesis.clone();
    let to = per_tick(&s, &w, "inst.commons");
    let dec = cast.decide(workers, &s, &w).unwrap();
    assert_eq!(sell_of(&dec, labour), Some(n - to / h));
    assert!(buy_of(&dec, land).is_none());
    // Commons with room.
    let mut s = genesis.clone();
    set_param(&mut s, &w, "inst.commons", 500.0);
    let dec = cast.decide(workers, &s, &w).unwrap();
    let f0 = f_of(chi, at(&s, labour), ps_of(&s), at(&s, food) * s0);
    assert_eq!(sell_of(&dec, labour), Some(n * f0));
    assert!(buy_of(&dec, land).is_none());
    // The split: no plot pays at r = 3, the commons small.
    let mut s = genesis.clone();
    set_param(&mut s, &w, "inst.commons", 2.6);
    set_price(&mut s, &w, land, 3.0);
    let sf = value(&s, &w, "inst.exit.floor");
    assert!(3.0 * h >= at(&s, food) * (s0 - sf));
    let dec = cast.decide(workers, &s, &w).unwrap();
    let fl = f_of(chi, at(&s, labour), ps_of(&s), at(&s, food) * sf);
    assert_eq!(sell_of(&dec, labour), Some(n * fl));
    assert!(buy_of(&dec, land).is_none());
    // e_r̂ is rounded once, fma(−r, h, p_g·s₀) (the frame's §3.1): over 2,000 draws of the wage,
    // the rent and food's price within a factor 2 of genesis, at a commons of 2.6 a year, every
    // draw at which the plots spill offers n(e_r̂) hours to the bit and rents h·(N − n) − T_o.
    let mut d = Draws(0x2026_0930_c002);
    let mut spill = 0;
    for _ in 0..2000 {
        let mut s = genesis.clone();
        set_param(&mut s, &w, "inst.commons", 2.6);
        for g in [labour, land, food] {
            set_price(&mut s, &w, g, at(&genesis, g) * d.decades(-0.3, 0.3));
        }
        let (wage, r, pg, ps) = (at(&s, labour), at(&s, land), at(&s, food), ps_of(&s));
        let to = per_tick(&s, &w, "inst.commons");
        let nr = n * f_of(chi, wage, ps, num::fma(-r, h, pg * s0));
        if !(r * h < pg * s0 && nr <= n - to / h) {
            continue;
        }
        spill += 1;
        let dec = cast.decide(workers, &s, &w).unwrap();
        assert_eq!(sell_of(&dec, labour), Some(nr));
        assert_eq!(buy_of(&dec, land).map(|b| b.0), Some(h * (n - nr) - to));
    }
    assert!(spill > 1000, "{spill} draws spill");
}

#[test]
fn exit_is_checked_at_load() {
    // The commons frame's §3.4: an exit good that is not a basket good, or is the currency; the
    // plots' land a basket item, the workers' hours, not Instant, or not the land of the
    // provider that pays them; s₀, s̲, h or T_o negative; too little land for every plot
    // (T + T_o ≤ h·N at genesis); workers no provider pays; and exit params of the wrong unit.
    // Each is refused with its path; a negative value by the registry, as every param's.
    let edit = |pairs: &[(&str, &str)]| {
        let mut t = C1.to_string();
        for (from, to) in pairs {
            assert_eq!(t.matches(from).count(), 1, "{from:?}");
            t = t.replacen(from, to, 1);
        }
        t
    };
    let exit = r#"exit: Some((good: "food", gross: "inst.exit.gross", floor: "inst.exit.floor", plot: "inst.exit.plot", commons: "inst.commons", land: "land"))"#;
    let with = |from: &str, to: &str| edit(&[(exit, &exit.replace(from, to))]);
    let value_of = |key: &str, v: &str| {
        let line = C1
            .lines()
            .find(|l| l.contains(&format!("(key: \"{key}\", value: ")))
            .expect("the param's line");
        let old = line
            .split("value: ")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap();
        edit(&[(
            &format!("(key: \"{key}\", value: {old},"),
            &format!("(key: \"{key}\", value: {v},"),
        )])
    };
    let plots_good = edit(&[
        (
            "        (key: \"labour\", life: Instant,",
            "        (key: \"plots\", life: Instant, price_rate: Some(\"rate.land\")),\n        (key: \"labour\", life: Instant,",
        ),
        (
            "            (node: \"home\", good: \"land\", price: 1.0),",
            "            (node: \"home\", good: \"land\", price: 1.0),\n            (node: \"home\", good: \"plots\", price: 1.0),",
        ),
        (exit, &exit.replace("land: \"land\"", "land: \"plots\"")),
    ]);
    let cases = [
        (
            with("good: \"food\"", "good: \"mach\""),
            "actors[workers].spec.exit.good",
        ),
        (
            with("good: \"food\"", "good: \"coin\""),
            "actors[workers].spec.exit.good",
        ),
        (
            with("land: \"land\"", "land: \"food\""),
            "actors[workers].spec.exit.land",
        ),
        (
            with("land: \"land\"", "land: \"labour\""),
            "actors[workers].spec.exit.land",
        ),
        (
            with("land: \"land\"", "land: \"mach\""),
            "actors[workers].spec.exit.land",
        ),
        (plots_good, "actors[workers].spec.exit.land"),
        // Check 3, s₀, s̲, h and T_o finite and not negative, is the registry's: every param's
        // value is finite with a clear sign bit.
        (
            value_of("inst.exit.gross", "-0.1"),
            "params[inst.exit.gross].value",
        ),
        (
            value_of("inst.exit.floor", "-0.1"),
            "params[inst.exit.floor].value",
        ),
        (
            value_of("inst.exit.plot", "-0.1"),
            "params[inst.exit.plot].value",
        ),
        (
            value_of("inst.commons", "-1.0"),
            "params[inst.commons].value",
        ),
        (
            value_of("inst.exit.plot", "3.0"),
            "actors[workers].spec.exit.plot",
        ),
        (
            edit(&[(
                "transfer: (to: \"workers\", heads: \"inst.workers\")",
                "transfer: (to: \"desk.food\", heads: \"inst.workers\")",
            )]),
            "actors[workers].spec.exit",
        ),
        (
            edit(&[(
                "(key: \"inst.commons\", value: 24.3, unit: FlowPerYear,",
                "(key: \"inst.commons\", value: 24.3, unit: Dimensionless,",
            )]),
            "actors[workers].spec.exit.commons",
        ),
        (
            edit(&[(
                "(key: \"inst.exit.gross\", value: 0.3, unit: Dimensionless,",
                "(key: \"inst.exit.gross\", value: 0.3, unit: FlowPerYear,",
            )]),
            "actors[workers].spec.exit.gross",
        ),
    ];
    for (text, path) in cases {
        let err = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
        };
        assert_eq!(err.path, path, "{err}");
    }
    // A basket item as the plots' land is refused as such at resolve, before `Cast::new` would
    // refuse it as a land that is not Instant at the same path.
    match load_text(&with("land: \"land\"", "land: \"food\"")) {
        Err(err) => assert!(err.to_string().contains("or a basket item"), "{err}"),
        Ok(_) => panic!("resolve refuses a basket item as the plots' land"),
    }
    // A land that outlives the tick is refused as the plots' land, before M6's check of every
    // bought good would refuse it at the same path.
    let (w, _) = load_text(&with("land: \"land\"", "land: \"mach\"")).unwrap();
    let err = Cast::new(&w).expect_err("the cast refuses it");
    assert!(err.to_string().contains("an Instant good, rented"), "{err}");
    // The block's fields are required, and no other is read.
    for bad in [
        exit.replace(", land: \"land\"", ""),
        exit.replace("land: \"land\"", "land: \"land\", rent: \"land\""),
    ] {
        assert!(Tape::<Agents>::from_ron(&edit(&[(exit, &bad)])).is_err());
    }
    // At the tape's own values every check passes: T + T_o is 10.47 a tick against h·N 0.54.
    let (w, s) = load_text(C1).unwrap();
    Cast::new(&w).unwrap();
    assert!(
        per_tick(&s, &w, "inst.land") + per_tick(&s, &w, "inst.commons")
            > value(&s, &w, "inst.exit.plot") * per_tick(&s, &w, "inst.workers")
    );
}

#[test]
fn commons_specs_round_trip_and_old_worlds_keep_their_ids() {
    // The field is optional and skipped when absent (R1): C1 survives to_ron and a parse with
    // the same world and writes the block back; a tape without it keeps its canonical text; and
    // the resolved workers leave it out of `world_id`, so I1's world is unchanged.
    let t = Tape::<Agents>::from_ron(C1).unwrap();
    let (w1, _) = resolve(&t).unwrap();
    Cast::new(&w1).unwrap();
    let back = Tape::<Agents>::from_ron(&t.to_ron()).unwrap();
    assert_eq!(back, t.canonical());
    let (w2, _) = resolve(&back).unwrap();
    assert_eq!(w1.world_id, w2.world_id);
    let text = t.to_ron();
    assert!(text.contains("exit: Some("), "{text}");
    assert!(text.contains("commons: \"inst.commons\""));
    let i1 = Tape::<Agents>::from_ron(I1).unwrap();
    assert!(!i1.to_ron().contains("exit:"));
    let (wi, _) = resolve(&i1).unwrap();
    assert_eq!(wi.world_id, 0x965e_dbef_2664_b850);
    // C1 and I1 differ in their workers' block and params alone, so their worlds differ.
    assert_ne!(w1.world_id, wi.world_id);
}
