//! The type switch at the wall (P2.4; docs/probe/SWITCH-RULES.md; the switch scan's §3,
//! docs/probe/switch/SPEC.md): a reserved pop's optional `pool`, on the switch tape
//! `tapes/markets-is1.ron` and on IW1's, which has none.
//!
//! As in `pace.rs`, most comparisons are exact: the tests recompute the rule's arithmetic in the
//! scan's §3.3 order of operations, written apart from the rule. One bar is in ulps and named
//! here: `ULPS`, for the registered vectors where libm's `exp` is an ulp off glibc's (the
//! registration's §3).

use rustyecon_agents::{
    switch_split, ActorState, AgentDelta, Agents, BasketWorkers, Cast, Decision, Spec,
    SwitchWorkersState,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, CoreError, GoodId, Holder, Ledger, LoadError, NodeId, ParamId,
    Phase, Provenance, SimState, Site, StateDelta, Tape, World,
};
use rustyecon_markets::Side;

const IS1: &str = include_str!("../../../tapes/markets-is1.ron");
const IW1: &str = include_str!("../../../tapes/markets-iw1.ron");
const C1: &str = include_str!("../../../tapes/markets-c1.ron");
const VECTORS: &str = include_str!("../../../docs/probe/switch/registered/rule_vectors.json");
/// The registered vectors whose outputs libm's `exp` makes one or two ulps off glibc's.
const LIBM_EXP: [usize; 3] = [5, 13, 23];
const ULPS: f64 = 4.0;

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

fn set_coin(s: &mut S, w: &W, a: ActorId, q: f64) {
    let coin = good(w, "coin");
    let mut ds = Vec::new();
    if held(s, a, coin) > 0.0 {
        ds.push(StateDelta::Burn {
            from: Holder::Actor(a),
            good: coin,
            amount: Amount::All,
            prov: Provenance::Event,
        });
    }
    if q > 0.0 {
        ds.push(StateDelta::Mint {
            to: Holder::Actor(a),
            good: coin,
            qty: q,
            prov: Provenance::Event,
        });
    }
    apply_in(s, w, Phase::Events, &ds).expect("the coin applies");
}

/// Set a switch pop's pool share, as its rule's decision sets it.
fn set_pool(s: &mut S, w: &W, a: ActorId, pool: f64) {
    let st = match s.ext().get(&a) {
        Some(ActorState::SwitchWorkers(st)) => *st,
        other => panic!("not a switch pop's state: {other:?}"),
    };
    let d = StateDelta::Actor(AgentDelta::SetState {
        actor: a,
        state: ActorState::SwitchWorkers(SwitchWorkersState { pool, ..st }),
    });
    apply_in(s, w, Phase::Decisions, &[d]).expect("the state applies");
}

fn pool_of(s: &S, a: ActorId) -> f64 {
    match s.ext().get(&a) {
        Some(ActorState::SwitchWorkers(st)) => st.pool,
        other => panic!("not a switch pop's state: {other:?}"),
    }
}

fn sell_of(d: &Decision, g: GoodId) -> Option<f64> {
    d.orders
        .iter()
        .find(|o| o.good == g && matches!(o.side, Side::Sell))
        .map(|o| o.qty)
}

fn minted(d: &Decision, g: GoodId) -> Option<f64> {
    d.deltas.iter().find_map(|x| match x {
        StateDelta::Mint { good, qty, .. } if *good == g => Some(*qty),
        _ => None,
    })
}

/// The switch state a decision sets.
fn state_of(d: &Decision) -> SwitchWorkersState {
    match d.deltas.last() {
        Some(StateDelta::Actor(AgentDelta::SetState {
            state: ActorState::SwitchWorkers(st),
            ..
        })) => *st,
        other => panic!("the decision does not end on a switch state: {other:?}"),
    }
}

fn spec_of(w: &W, key: &str) -> BasketWorkers {
    w.actors
        .iter()
        .find_map(|a| match &a.spec {
            Spec::BasketWorkers(p) if a.key.as_str() == key => Some(p.clone()),
            _ => None,
        })
        .expect("the pop")
}

/// A site's per-tick value in `s`, as a rule reads it.
fn at(w: &W, s: &S, site: Site) -> f64 {
    site.convert(&w.clock, s.param(site.param).unwrap())
        .unwrap()
}

/// The scan's §3.3 steps 3–6, written apart from the rule, in its order of operations.
#[allow(clippy::too_many_arguments)]
fn rule(a: f64, eps: f64, w: f64, wi: f64, k: f64, ps: f64, chi: f64, n: f64) -> [f64; 6] {
    let e = eps * w;
    let g = num::ln(e / wi);
    let a1 = if g > 0.0 {
        a + (-num::expm1(-(k * g))) * (1.0 - a)
    } else if g < 0.0 {
        a * num::exp(k * g)
    } else {
        a
    };
    let v = wi + a1 * (e - wi);
    let f = (num::ln1p(v / ps) / chi).min(1.0);
    let hours = n * f;
    [g, a1, v, f, (1.0 - a1) * hours, eps * (a1 * hours)]
}

/// The registered vectors: inputs (w, w_i, P_s, a, ε, k, N, χ_max) and outputs (a', g, v, F,
/// the reserved hours, the pool's efficiency hours), each a Python `repr` double.
fn vectors() -> Vec<[f64; 14]> {
    let keys = [
        "w",
        "w_i",
        "p_s",
        "a",
        "eps",
        "k",
        "heads",
        "chi_max",
        "a1",
        "g",
        "v_eff",
        "F",
        "reserved_hours",
        "pool_efficiency_hours",
    ];
    VECTORS
        .split("\"w\": \"")
        .skip(1)
        .map(|chunk| {
            let chunk = format!("\"w\": \"{chunk}");
            let mut v = [0.0; 14];
            for (i, k) in keys.iter().enumerate() {
                let pat = format!("\"{k}\": \"");
                let from = chunk.find(&pat).expect("the field") + pat.len();
                let to = from + chunk[from..].find('"').unwrap();
                v[i] = chunk[from..to].parse().expect("a double");
            }
            v
        })
        .collect()
}

fn ulps_apart(x: f64, y: f64) -> f64 {
    if x == y {
        return 0.0;
    }
    let ulp = f64::EPSILON * y.abs().max(f64::MIN_POSITIVE);
    (x - y).abs() / ulp
}

#[test]
fn switch_rule_matches_the_registered_vectors() {
    // The scan's §3.10 test 1, as the registration's §3 reads it: the rule's steps 3–6
    // (`switch_split`, which the decision and the harness share) against the 24 registered
    // type-states. g, a', v, F and both offers equal the registered doubles bit for bit at 21 of
    // them; at the three where the share decays by exp(k·g), libm's `exp` is one ulp off glibc's
    // and the outputs part by one or two ulps (within 4). And `switch_split` is the §3.3 order
    // written again here, bit for bit, at every vector.
    let v = vectors();
    assert_eq!(v.len(), 24);
    let mut parted = Vec::new();
    for (n, x) in v.iter().enumerate() {
        let [w, wi, ps, a, eps, k, heads, chi] = [x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]];
        assert_eq!(k, 0.5, "vector {n}: k is rate.switch/tpy, 26/52");
        let s = switch_split(a, eps, w, wi, k, ps, chi, heads);
        let got = [s.pool, s.gap, s.wage, s.share, s.own, s.pooled];
        let again = rule(a, eps, w, wi, k, ps, chi, heads);
        assert_eq!(
            [again[1], again[0], again[2], again[3], again[4], again[5]].map(f64::to_bits),
            got.map(f64::to_bits),
            "vector {n}: the rule written again"
        );
        let want = [x[8], x[9], x[10], x[11], x[12], x[13]];
        assert_eq!(got[1].to_bits(), want[1].to_bits(), "vector {n}: g");
        if got.map(f64::to_bits) != want.map(f64::to_bits) {
            parted.push(n);
            for (i, (g, r)) in got.iter().zip(&want).enumerate() {
                assert!(
                    ulps_apart(*g, *r) <= ULPS,
                    "vector {n}, output {i}: {g:?} {r:?}"
                );
            }
        }
    }
    assert_eq!(parted, LIBM_EXP);
    // The corners: from a = 0 the pool pays more, a' = share(k·g); from a = 1 it pays less,
    // a' = e^(k·g); at a tie a' = a.
    let s = switch_split(0.0, 1.5, 1.0, 1.2, 0.5, 1.5, 2.0, 1.0);
    assert_eq!(s.pool, -num::expm1(-(0.5 * s.gap)));
    let s = switch_split(1.0, 1.5, 1.0, 1.8, 0.5, 1.5, 2.0, 1.0);
    assert_eq!(s.pool, num::exp(0.5 * s.gap));
    let s = switch_split(0.3, 1.5, 1.0, 1.5, 0.5, 1.5, 2.0, 1.0);
    assert_eq!((s.gap, s.pool), (0.0, 0.3));
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

fn traded(w: &W) -> Vec<GoodId> {
    (0..w.goods.len() as u32)
        .map(GoodId)
        .filter(|g| !w.is_currency(*g))
        .collect()
}

const POPS: [&str; 2] = ["workers.trained", "workers.master"];

#[test]
fn switch_off_is_the_reserved_pop() {
    // The scan's §3.10 test 2 (R1): with ε 0 the switch is structurally off. From 500 random
    // states (every price within a decade and a half of genesis, each pop's coin over twelve
    // decades or 0, its heads within half a decade, its pool share at 0, 1 or uniform), each
    // switch pop of IS1 decides exactly as IW1's reserved pop decides from the same state: the
    // same orders, and the same deltas but the last, which sets its state to P2.3's share with
    // its pool share unchanged. Without a pool (IW1) the state is `Workers`.
    let (ws, gs, cs) = load(IS1);
    let (ww, gw, cw) = load(IW1);
    let goods = traded(&ws);
    for g in &goods {
        assert_eq!(ws.key_of(*g), ww.key_of(*g), "the two worlds' goods");
    }
    for p in POPS {
        assert_eq!(actor(&ws, p), actor(&ww, p));
        assert!(matches!(
            gw.ext().get(&actor(&ww, p)),
            Some(ActorState::Workers(_))
        ));
        assert!(matches!(
            gs.ext().get(&actor(&ws, p)),
            Some(ActorState::SwitchWorkers(_))
        ));
    }
    let mut d = Draws(0x2026_0930_5701);
    for i in 0..500 {
        let (mut s, mut t) = (gs.clone(), gw.clone());
        for key in ["inst.trained.efficiency", "inst.master.efficiency"] {
            set_param(&mut s, &ws, key, 0.0);
        }
        for g in &goods {
            let f = d.decades(-1.5, 1.5);
            let p = gs.price(home(&ws), *g).unwrap() * f;
            set_price(&mut s, &ws, *g, p);
            set_price(&mut t, &ww, *g, p);
        }
        for p in POPS {
            let c = if i % 17 == 0 {
                0.0
            } else {
                d.decades(-6.0, 6.0)
            };
            set_coin(&mut s, &ws, actor(&ws, p), c);
            set_coin(&mut t, &ww, actor(&ww, p), c);
            let key = format!("inst.{}.workers", &p[8..]);
            let heads = d.decades(1.0, 2.0);
            set_param(&mut s, &ws, &key, heads);
            set_param(&mut t, &ww, &key, heads);
            let a = match i % 3 {
                0 => 0.0,
                1 => 1.0,
                _ => d.unit(),
            };
            set_pool(&mut s, &ws, actor(&ws, p), a);
        }
        for p in POPS {
            let (a_s, a_w) = (actor(&ws, p), actor(&ww, p));
            let x = cs.decide(a_s, &s, &ws).unwrap();
            let y = cw.decide(a_w, &t, &ww).unwrap();
            assert_eq!(x.orders, y.orders, "draw {i}: {p}'s orders");
            let n = y.deltas.len();
            assert_eq!(x.deltas.len(), n, "draw {i}: {p}'s deltas");
            assert_eq!(x.deltas[..n - 1], y.deltas[..n - 1], "draw {i}: {p}");
            let share = match &y.deltas[n - 1] {
                StateDelta::Actor(AgentDelta::SetState {
                    state: ActorState::Workers(st),
                    ..
                }) => st.share,
                other => panic!("{other:?}"),
            };
            assert_eq!(
                state_of(&x),
                SwitchWorkersState {
                    share,
                    pool: pool_of(&s, a_s)
                },
                "draw {i}: {p}'s state"
            );
        }
    }
}

#[test]
fn switch_moves_toward_the_better_market() {
    // The scan's §3.3 and §3.10 test 3, on IS1's tape: from its pool share a and posted prices a
    // switch pop moves a toward the market that pays more and splits its hours, as the §3.3
    // order written again here gives them: it offers (1 − a')·n of its own labour and mints them
    // where positive, offers and mints ε·(a'·n) of the pool's labour only where positive, and
    // sets its state to (F, a'). From a = 0 where the pool pays more a' = share(k·g); from a = 1
    // where it pays less a' = e^(k·g); at an exact tie (w_i = ε·w) a' = a; a' stays in [0, 1] at
    // gaps of twelve decades. Over 3,000 random states, at the dial's 26 a year and at rates set
    // by a dated `SetParam` (the params are live), ε drawn over a decade.
    let (w, genesis, cast) = load(IS1);
    let labour = good(&w, "labour");
    let mut d = Draws(0x2026_0930_5702);
    let (mut up, mut down, mut tie, mut pooled) = (0, 0, 0, 0);
    for i in 0..3000 {
        let mut s = genesis.clone();
        let spread = if i % 13 == 0 { 6.0 } else { 1.0 };
        for g in traded(&w) {
            let p = genesis.price(home(&w), g).unwrap() * d.decades(-spread, spread);
            set_price(&mut s, &w, g, p);
        }
        for p in POPS {
            let key = &p[8..];
            if i % 5 == 0 {
                set_param(
                    &mut s,
                    &w,
                    &format!("rate.switch.{key}"),
                    d.decades(-1.0, 3.0),
                );
            }
            if i % 7 == 0 {
                set_param(
                    &mut s,
                    &w,
                    &format!("inst.{key}.efficiency"),
                    d.decades(-0.5, 0.5),
                );
            }
            let a = match i % 4 {
                0 => 0.0,
                1 => 1.0,
                _ => d.unit(),
            };
            set_pool(&mut s, &w, actor(&w, p), a);
            set_coin(&mut s, &w, actor(&w, p), d.decades(-3.0, 3.0));
        }
        if i % 3 == 0 {
            // An exact tie for the trained: its wage the pool's ε·w, as the rule forms e.
            let spec = spec_of(&w, "workers.trained");
            let eps = at(&w, &s, spec.pool.unwrap().efficiency);
            let wage = s.price(home(&w), labour).unwrap();
            set_price(&mut s, &w, good(&w, "labour.trained"), eps * wage);
        }
        for p in POPS {
            let a_id = actor(&w, p);
            let spec = spec_of(&w, p);
            let pool = spec.pool.expect("a pool");
            let price = |g: GoodId| s.price(home(&w), g).unwrap();
            let mut ps = 0.0;
            for it in &spec.basket {
                ps += at(&w, &s, it.weight) * price(it.good);
            }
            let a = pool_of(&s, a_id);
            let eps = at(&w, &s, pool.efficiency);
            let k = at(&w, &s, pool.rate);
            // The rate is read as a log step, k = rate/tpy (the scan's §3.2).
            let rate = s
                .param(param(&w, &format!("rate.switch.{}", &p[8..])))
                .unwrap();
            assert_eq!(k, rate / 52.0, "draw {i}: {p}'s k");
            let chi = at(&w, &s, spec.chi_max);
            let n = at(&w, &s, spec.heads);
            let [g, a1, _, f, own, to_pool] =
                rule(a, eps, price(pool.good), price(spec.labour), k, ps, chi, n);
            let dec = cast.decide(a_id, &s, &w).unwrap();
            assert_eq!(sell_of(&dec, spec.labour), Some(own), "draw {i}: {p}'s own");
            assert_eq!(
                minted(&dec, spec.labour),
                (own > 0.0).then_some(own),
                "draw {i}: {p}'s own mint"
            );
            assert_eq!(
                sell_of(&dec, pool.good),
                (to_pool > 0.0).then_some(to_pool),
                "draw {i}: {p}'s pool offer"
            );
            assert_eq!(
                minted(&dec, pool.good),
                (to_pool > 0.0).then_some(to_pool),
                "draw {i}: {p}'s pool mint"
            );
            assert_eq!(
                state_of(&dec),
                SwitchWorkersState { share: f, pool: a1 },
                "draw {i}: {p}'s state"
            );
            assert!((0.0..=1.0).contains(&a1), "draw {i}: {p}: {a1}");
            if a == 0.0 && g > 0.0 {
                assert_eq!(a1, -num::expm1(-(k * g)), "draw {i}: {p} leaves its wall");
            }
            if a == 1.0 && g < 0.0 {
                assert_eq!(a1, num::exp(k * g), "draw {i}: {p} leaves the pool");
            }
            if g == 0.0 {
                assert_eq!(a1, a, "draw {i}: {p} at a tie");
                tie += 1;
            }
            up += usize::from(g > 0.0);
            down += usize::from(g < 0.0);
            pooled += usize::from(to_pool > 0.0);
        }
    }
    assert!(
        up > 500 && down > 500 && tie > 500 && pooled > 1000,
        "{up} {down} {tie} {pooled}"
    );
    // The sites are listed with the others under the pop's spec.
    let sites = w
        .actors
        .iter()
        .find(|a| a.key.as_str() == "workers.trained")
        .unwrap()
        .spec
        .sites(&w);
    for k in ["pool.efficiency", "pool.rate"] {
        assert!(sites.iter().any(|(p, _)| p == k), "{k}: {sites:?}");
    }
}

#[test]
fn switch_pool_is_checked_at_load() {
    // The scan's §3.6 and §3.10's load checks, each at its path: the pool's good declared,
    // `Instant`, not the pop's own labour and not a basket item (the registration's §3); ε a
    // `Dimensionless` param and the rate a `RatePerYear` one; the genesis share finite and in
    // [0, 1]; a negative rate the registry's; `pool` with `exit` refused. The block's four
    // fields are required and no other is read.
    let edit = |text: &str, pairs: &[(&str, &str)]| {
        let mut t = text.to_string();
        for (from, to) in pairs {
            assert_eq!(t.matches(from).count(), 1, "{from:?}");
            t = t.replacen(from, to, 1);
        }
        t
    };
    let pool = "pool: Some((good: \"labour\", efficiency: \"inst.trained.efficiency\", rate: \
                \"rate.switch.trained\", share: 0.0))";
    assert_eq!(
        IS1.matches(pool).count(),
        1,
        "the trained's pool as written"
    );
    let with = |from: &str, to: &str| edit(IS1, &[(pool, &pool.replace(from, to))]);
    let at = "actors[workers.trained].spec.pool";
    let cases = [
        (with("\"labour\"", "\"nolabour\""), format!("{at}.good")),
        (with("\"labour\"", "\"mach\""), format!("{at}.good")),
        (
            with("\"labour\"", "\"labour.trained\""),
            format!("{at}.good"),
        ),
        // A basket item: services (one tick, refused as a basket item before its life is read)
        // and land (the space in the basket, `Instant`, so only the basket check refuses it).
        (with("\"labour\"", "\"services\""), format!("{at}.good")),
        (with("\"labour\"", "\"land\""), format!("{at}.good")),
        (
            with("\"inst.trained.efficiency\"", "\"rate.switch.master\""),
            format!("{at}.efficiency"),
        ),
        (
            with("\"rate.switch.trained\"", "\"inst.master.efficiency\""),
            format!("{at}.rate"),
        ),
        (with("share: 0.0", "share: -0.1"), format!("{at}.share")),
        (with("share: 0.0", "share: 1.5"), format!("{at}.share")),
        (with("share: 0.0", "share: NaN"), format!("{at}.share")),
        (with("share: 0.0", "share: inf"), format!("{at}.share")),
        (
            edit(
                IS1,
                &[(
                    "(key: \"rate.switch.trained\", value: 26.0,",
                    "(key: \"rate.switch.trained\", value: -26.0,",
                )],
            ),
            "params[rate.switch.trained].value".to_string(),
        ),
        (
            edit(
                C1,
                &[(
                    "land: \"land\")),\n",
                    "land: \"land\")),\n            pool: Some((good: \"labour\", efficiency: \
                     \"inst.exit.gross\", rate: \"rate.labour\", share: 0.0)),\n",
                )],
            ),
            "actors[workers].spec.pool".to_string(),
        ),
    ];
    for (text, path) in cases {
        let err = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("the cast refuses it"),
        };
        assert_eq!(err.path, path, "{err}");
    }
    for bad in [
        pool.replace(", share: 0.0", ""),
        pool.replace("share: 0.0", "share: 0.0, lag: 1.0"),
        pool.replace("rate:", "adjust:"),
    ] {
        assert!(
            Tape::<Agents>::from_ron(&edit(IS1, &[(pool, &bad)])).is_err(),
            "{bad}"
        );
    }
    for v in ["0.0", "1.0", "0.25"] {
        let (w, _) = load_text(&with("share: 0.0", &format!("share: {v}"))).unwrap();
        Cast::new(&w).unwrap();
    }
}

#[test]
fn switch_state_is_checked() {
    // The scan's §3.4: `apply` and `validate` read both of a switch pop's state values: each
    // finite with a clear sign bit and at most 1, so a pool share or a participation share of
    // 1.5, NaN, −0.0 or −1 is refused and nothing changes; 0, 1 and a share between are taken.
    let (w, s0, _) = load(IS1);
    let a = actor(&w, "workers.trained");
    let set = |s: &mut S, share: f64, pool: f64| {
        let d = StateDelta::Actor(AgentDelta::SetState {
            actor: a,
            state: ActorState::SwitchWorkers(SwitchWorkersState { share, pool }),
        });
        apply_in(s, &w, Phase::Decisions, &[d])
    };
    for (share, pool) in [
        (0.5, 1.5),
        (0.5, f64::NAN),
        (0.5, -0.0),
        (0.5, -1.0),
        (1.5, 0.5),
        (f64::INFINITY, 0.5),
    ] {
        let mut s = s0.clone();
        assert!(set(&mut s, share, pool).is_err(), "({share}, {pool})");
        assert_eq!(s.ext().get(&a), s0.ext().get(&a));
    }
    for (share, pool) in [(0.0, 0.0), (1.0, 1.0), (0.3, 0.7)] {
        let mut s = s0.clone();
        set(&mut s, share, pool).expect("a clean state");
        assert_eq!(pool_of(&s, a), pool);
    }
    // A state of another kind is refused for a switch pop.
    let mut s = s0.clone();
    let d = StateDelta::Actor(AgentDelta::SetState {
        actor: a,
        state: ActorState::Workers(rustyecon_agents::WorkersState { share: 0.5 }),
    });
    assert!(apply_in(&mut s, &w, Phase::Decisions, &[d]).is_err());
}

#[test]
fn switch_genesis_state_is_the_tapes() {
    // The scan's §3.4 and §3.8: a switch pop's genesis state is `SwitchWorkers { share: 0, pool:
    // its pool's share }`, as the tape writes it (0.0 at IS1's base, 0.25 as edited); a pop
    // without a pool keeps `Workers`. The field survives `to_ron` and a parse with the same
    // world; a tape without it writes none, and the resolved pops leave it out of `world_id`, so
    // IW1's world is the one the pre-build binary recorded (build-switch's base streams,
    // 2026-09-30, at b7b9242).
    let (w, s, _) = load(IS1);
    for p in POPS {
        assert_eq!(
            s.ext().get(&actor(&w, p)),
            Some(&ActorState::SwitchWorkers(SwitchWorkersState {
                share: 0.0,
                pool: 0.0
            }))
        );
    }
    let edited = IS1.replacen(
        "rate: \"rate.switch.trained\", share: 0.0",
        "rate: \"rate.switch.trained\", share: 0.25",
        1,
    );
    let (we, se, _) = load(&edited);
    assert_eq!(pool_of(&se, actor(&we, "workers.trained")), 0.25);
    assert_eq!(pool_of(&se, actor(&we, "workers.master")), 0.0);
    let t = Tape::<Agents>::from_ron(IS1).unwrap();
    let back = Tape::<Agents>::from_ron(&t.to_ron()).unwrap();
    assert_eq!(back, t.canonical());
    let (w2, _) = resolve(&back).unwrap();
    assert_eq!(w.world_id, w2.world_id);
    assert!(t.to_ron().contains("pool: Some("));
    let (ww, sw, _) = load(IW1);
    assert!(matches!(
        sw.ext().get(&actor(&ww, "workers.trained")),
        Some(ActorState::Workers(_))
    ));
    assert!(!Tape::<Agents>::from_ron(IW1)
        .unwrap()
        .to_ron()
        .contains("pool: "));
    assert_eq!(ww.world_id, 0x7698_c09c_b7c5_4791);
    assert_ne!(w.world_id, ww.world_id);
}
