//! The trap's remedy (P2.4; docs/probe/TRAP-RULES.md; the trap scan's §5, docs/probe/trap/SPEC.md):
//! the workers' exit's optional `pace`, participation at a rate, on the paced commons tape
//! `tapes/markets-c1p.ron` and on C1's, which has none.
//!
//! As in `commons.rs`, most comparisons are exact: the tests recompute the rule's arithmetic in
//! the rule's own order of operations.

use rustyecon_agents::{
    workers_participation, ActorState, AgentDelta, AgentError, Agents, BasketWorkers, Cast,
    Decision, Spec, WorkersState,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, CoreError, FlowPerYear, GoodId, Holder, Ledger, LoadError,
    NodeId, ParamId, Phase, Provenance, RatePerYear, SimState, Site, StateDelta, Tape, World,
};
use rustyecon_markets::Side;

const C1: &str = include_str!("../../../tapes/markets-c1.ron");
const C1P: &str = include_str!("../../../tapes/markets-c1p.ron");

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

/// Set the workers' own state share, as their rule's decision sets it.
fn set_share(s: &mut S, w: &W, a: ActorId, share: f64) {
    let d = StateDelta::Actor(AgentDelta::SetState {
        actor: a,
        state: ActorState::Workers(WorkersState { share }),
    });
    apply_in(s, w, Phase::Decisions, &[d]).expect("the state applies");
}

fn share_of(s: &S, a: ActorId) -> f64 {
    match s.ext().get(&a) {
        Some(ActorState::Workers(st)) => st.share,
        other => panic!("not the workers' state: {other:?}"),
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

fn sell_of(d: &Decision, g: GoodId) -> Option<f64> {
    d.orders
        .iter()
        .find(|o| o.good == g && matches!(o.side, Side::Sell))
        .map(|o| o.qty)
}

/// The share a decision sets in the workers' state.
fn state_of(d: &Decision) -> f64 {
    d.deltas
        .iter()
        .find_map(|x| match x {
            StateDelta::Actor(AgentDelta::SetState {
                state: ActorState::Workers(st),
                ..
            }) => Some(st.share),
            _ => None,
        })
        .expect("the decision sets the workers' state")
}

/// The hours a decision mints for the workers.
fn minted(d: &Decision, labour: GoodId) -> Option<f64> {
    d.deltas.iter().find_map(|x| match x {
        StateDelta::Mint { good, qty, .. } if *good == labour => Some(*qty),
        _ => None,
    })
}

fn value(s: &S, w: &W, key: &str) -> f64 {
    s.param(param(w, key)).expect("a param")
}

fn per_tick(s: &S, w: &W, key: &str) -> f64 {
    w.clock.flow(FlowPerYear(value(s, w, key)))
}

fn the_workers(w: &W) -> BasketWorkers {
    w.actors
        .iter()
        .find_map(|a| match &a.spec {
            Spec::BasketWorkers(p) if a.key.as_str() == "workers" => Some(p.clone()),
            _ => None,
        })
        .expect("the workers")
}

/// The rule's participation at `s`'s posted prices and params, formed apart from the decision.
fn rule_at(w: &W, s: &S, p: &BasketWorkers) -> rustyecon_agents::Participation {
    let par = |site: Site| -> Result<f64, AgentError> {
        let v = s
            .param(site.param)
            .ok_or(AgentError::Core(CoreError::UnknownParam(site.param)))?;
        site.convert(&w.clock, v).map_err(AgentError::Core)
    };
    let pr = |g: GoodId| -> Result<f64, AgentError> {
        s.price(home(w), g)
            .ok_or(AgentError::Core(CoreError::Shape("no price".into())))
    };
    workers_participation(p, &par, &pr)
        .expect("the rule forms")
        .expect("an exit")
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

/// A random state of the tape's world: every price within a factor `spread` (in decades) of
/// genesis, the workers' coin over twelve decades (sometimes 0), the commons over two decades
/// around the tape's, the exit's s₀ and the heads within half a decade, and the workers' own
/// state share uniform on [0, 1].
fn draw_state(w: &W, genesis: &S, d: &mut Draws, spread: f64, i: usize) -> S {
    let mut s = genesis.clone();
    let workers = actor(w, "workers");
    let goods: Vec<GoodId> = (0..w.goods.len() as u32)
        .map(GoodId)
        .filter(|g| !w.is_currency(*g))
        .collect();
    for g in goods {
        let p = s.price(home(w), g).unwrap() * d.decades(-spread, spread);
        set_price(&mut s, w, g, p);
    }
    let c = if i.is_multiple_of(17) {
        0.0
    } else {
        d.decades(-6.0, 6.0)
    };
    set_coin(&mut s, w, workers, c);
    for (key, lo, hi) in [
        ("inst.commons", 0.3, 2.3),
        ("inst.exit.gross", -1.0, -0.2),
        ("inst.workers", 2.0, 2.6),
    ] {
        let v = d.decades(lo, hi);
        set_param(&mut s, w, key, v);
    }
    set_share(&mut s, w, workers, d.unit());
    s
}

#[test]
fn pace_absent_is_p23s() {
    // The trap scan's §5.5 test 1 (R1): with an exit and no pace (C1's tape) the workers decide
    // as P2.3's rule, whatever their own state share: over 3,000 random states (prices within a
    // decade and a half of genesis, coin, the commons, s₀, the heads, the state share), they offer
    // the rule's hours `workers_participation` gives, mint them, and set the rule's share; and
    // the same state with another own share gives the same decision, order for order and delta
    // for delta (the share is a record there, never read). C1's tape writes no pace.
    let (w, genesis, cast) = load(C1);
    let workers = actor(&w, "workers");
    let labour = good(&w, "labour");
    let spec = the_workers(&w);
    assert!(spec.exit.as_ref().is_some_and(|x| x.pace.is_none()));
    let mut d = Draws(0x2026_0930_7a01);
    for i in 0..3000 {
        let s = draw_state(&w, &genesis, &mut d, 1.5, i);
        let dec = cast.decide(workers, &s, &w).unwrap();
        let p = rule_at(&w, &s, &spec);
        assert_eq!(sell_of(&dec, labour), Some(p.hours), "draw {i}: the hours");
        assert_eq!(state_of(&dec), p.share, "draw {i}: the state");
        if p.hours > 0.0 {
            assert_eq!(minted(&dec, labour), Some(p.hours), "draw {i}: the mint");
        }
        let mut t = s.clone();
        set_share(&mut t, &w, workers, d.unit());
        assert_eq!(
            cast.decide(workers, &t, &w).unwrap(),
            dec,
            "draw {i}: the own share is read"
        );
    }
    let t = Tape::<Agents>::from_ron(C1).unwrap();
    assert!(!t.to_ron().contains("pace"));
}

#[test]
fn paced_share_moves_at_its_rate() {
    // The trap scan's §5.2 and §5.5 test 2, on C1P's tape: from the workers' own share s₀ and
    // posted prices, the decision offers and mints N·(s₀ + a·(F* − s₀)) hours, with F* the
    // rule's hours over N (`workers_participation`) and a = share(adjust) = −expm1(−rate/tpy),
    // sets its state to s₀ + a·(F* − s₀), and posts the rule's plots: where the plots spill (a
    // commons of 2.6 a year) one buy of the rule's T_p land at r with budget r·T_p, and none
    // where they do not. Over 3,000 random states, at the dial's 1.3 a year and at a rate set
    // by a dated `SetParam` (the param is live).
    let (w, genesis, cast) = load(C1P);
    let workers = actor(&w, "workers");
    let (labour, land) = (good(&w, "labour"), good(&w, "land"));
    let spec = the_workers(&w);
    assert!(spec.exit.as_ref().is_some_and(|x| x.pace.is_some()));
    let mut d = Draws(0x2026_0930_7a02);
    let (mut spill, mut none) = (0, 0);
    for i in 0..3000 {
        let mut s = draw_state(&w, &genesis, &mut d, 0.5, i);
        if i.is_multiple_of(3) {
            set_param(&mut s, &w, "inst.commons", 2.6);
        }
        if i.is_multiple_of(5) {
            set_param(
                &mut s,
                &w,
                "adjust.participation.workers",
                d.decades(-1.0, 2.0),
            );
        }
        let rate = value(&s, &w, "adjust.participation.workers");
        let a = w.clock.share(RatePerYear(rate));
        assert_eq!(a, -num::expm1(-(rate / 52.0)));
        let s0 = share_of(&s, workers);
        let p = rule_at(&w, &s, &spec);
        let n = per_tick(&s, &w, "inst.workers");
        assert_eq!(p.heads, n);
        let target = p.hours / n;
        let share = s0 + a * (target - s0);
        let dec = cast.decide(workers, &s, &w).unwrap();
        assert_eq!(
            sell_of(&dec, labour),
            Some(n * share),
            "draw {i}: the hours"
        );
        assert_eq!(state_of(&dec), share, "draw {i}: the state");
        if n * share > 0.0 {
            assert_eq!(minted(&dec, labour), Some(n * share), "draw {i}: the mint");
        }
        assert!((0.0..=1.0).contains(&share), "draw {i}: {share}");
        match buy_of(&dec, land) {
            Some((q, b)) => {
                spill += 1;
                assert!(p.plots > 0.0, "draw {i}: a land buy with no plots");
                assert_eq!(q, p.plots, "draw {i}: the rule's plots");
                let r = s.price(home(&w), land).unwrap();
                assert!(b <= r * p.plots, "draw {i}: the rent");
            }
            None => {
                none += 1;
                assert_eq!(p.plots, 0.0, "draw {i}: plots with no land buy");
            }
        }
    }
    assert!(spill > 300 && none > 300, "{spill} spill, {none} do not");
    // The site is listed with the others under the workers' spec.
    let sites = w
        .actors
        .iter()
        .find(|a| a.key.as_str() == "workers")
        .unwrap()
        .spec
        .sites(&w);
    assert!(
        sites.iter().any(|(k, _)| k == "exit.pace.adjust"),
        "{sites:?}"
    );
}

#[test]
fn pace_is_checked_at_load() {
    // The trap scan's §5.3 and §5.5 test 4: a genesis share outside [0, 1], or not a finite
    // number, is refused at `actors[workers].spec.exit.pace.share`; a rate param of the wrong
    // unit at `.adjust`; a negative rate by the registry, as every param's value. The block's
    // two fields are required and no other is read.
    let edit = |pairs: &[(&str, &str)]| {
        let mut t = C1P.to_string();
        for (from, to) in pairs {
            assert_eq!(t.matches(from).count(), 1, "{from:?}");
            t = t.replacen(from, to, 1);
        }
        t
    };
    // The block as written: `pace: Some((adjust: "…", share: S))`.
    let at = C1P.find("pace: Some((").expect("the pace");
    let end = at + C1P[at..].find("))").unwrap() + 2;
    let pace = &C1P[at..end];
    let share = pace
        .split("share: ")
        .nth(1)
        .unwrap()
        .trim_end_matches("))")
        .to_string();
    assert!(share.parse::<f64>().is_ok(), "{pace}");
    let with_share = |v: &str| {
        edit(&[(
            pace,
            &pace.replace(&format!("share: {share}"), &format!("share: {v}")),
        )])
    };
    let cases = [
        (with_share("-0.1"), "actors[workers].spec.exit.pace.share"),
        (with_share("1.5"), "actors[workers].spec.exit.pace.share"),
        (with_share("NaN"), "actors[workers].spec.exit.pace.share"),
        (with_share("inf"), "actors[workers].spec.exit.pace.share"),
        (
            edit(&[(
                pace,
                &pace.replace("adjust.participation.workers", "inst.exit.gross"),
            )]),
            "actors[workers].spec.exit.pace.adjust",
        ),
        (
            edit(&[(
                "(key: \"adjust.participation.workers\", value: 1.3, unit: RatePerYear,",
                "(key: \"adjust.participation.workers\", value: 1.3, unit: Dimensionless,",
            )]),
            "actors[workers].spec.exit.pace.adjust",
        ),
        (
            edit(&[(
                "(key: \"adjust.participation.workers\", value: 1.3,",
                "(key: \"adjust.participation.workers\", value: -1.3,",
            )]),
            "params[adjust.participation.workers].value",
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
        pace.replace(", share: ", ", rate: "),
        pace.replace(
            &format!("share: {share}"),
            &format!("share: {share}, lag: 1.0"),
        ),
        pace.replace(&format!(", share: {share}"), ""),
    ] {
        assert!(
            Tape::<Agents>::from_ron(&edit(&[(pace, &bad)])).is_err(),
            "{bad}"
        );
    }
    // The edges of [0, 1] load.
    for v in ["0.0", "1.0"] {
        let (w, _) = load_text(&with_share(v)).unwrap();
        Cast::new(&w).unwrap();
    }
}

#[test]
fn paced_genesis_state_is_the_tapes() {
    // The trap scan's §5.1 and §5.5 test 5: the paced workers' genesis state share is their
    // pace's `share`, as the tape writes it; without a pace it is 0.0, as before. The field
    // survives `to_ron` and a parse with the same world; a tape without it writes none, and the
    // resolved workers leave it out of `world_id`, so C1's world is the one the pre-build binary
    // recorded (build-trap's base streams, 2026-09-30, at e29b9c6).
    let (w, s, _) = load(C1P);
    let workers = actor(&w, "workers");
    let written: f64 = C1P
        .split("pace: Some((adjust: \"adjust.participation.workers\", share: ")
        .nth(1)
        .unwrap()
        .split(')')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(written > 0.1 && written < 0.2, "{written}");
    assert_eq!(share_of(&s, workers), written);
    let (wc, sc, _) = load(C1);
    assert_eq!(share_of(&sc, actor(&wc, "workers")), 0.0);
    // Round trip: C1P keeps its world and writes its pace back.
    let t = Tape::<Agents>::from_ron(C1P).unwrap();
    let back = Tape::<Agents>::from_ron(&t.to_ron()).unwrap();
    assert_eq!(back, t.canonical());
    let (w2, _) = resolve(&back).unwrap();
    assert_eq!(w.world_id, w2.world_id);
    let text = t.to_ron();
    assert!(text.contains("pace: Some("), "{text}");
    assert!(text.contains("adjust: \"adjust.participation.workers\""));
    // C1 keeps its world, and C1P's differs from it.
    assert_eq!(wc.world_id, 0xf63e_02fb_0360_98fb);
    assert_ne!(w.world_id, wc.world_id);
}
