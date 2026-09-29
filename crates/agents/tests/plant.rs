//! The planted desks (P2.2b; docs/probe/LOOPS-RULES.md §3–§6, §10), on the loop step's tapes:
//! LB1 (rule B, the capacity desk, the maker and the fodder desk each with a plant) and LW1 (the
//! flow control, a horse-day desk and the fodder desk with plants).
//!
//! Most comparisons are exact. A planted desk at θ = 1 is its kind to the bit; the rules' numbers
//! are literals the test forms from LOOP-SPEC §2.2–§2.5 in the evaluation order LOOPS-RULES §3
//! registers, so they agree to the bit too.

use rustyecon_agents::behaviour::{Behaviour, Decision};
use rustyecon_agents::cast::view;
use rustyecon_agents::roles::stock::rules::MakerRole;
use rustyecon_agents::{
    maker_reservation, ActorState, AgentDelta, Agents, CapacityDesk, CapacityState, Cast,
    GoodDeskState, MachDeskState, Maker, MakerState, OwnerState, PlantState, PlantedCapacity,
    PlantedCapacityState, PlantedMaker, PlantedMakerState, PlantedType, PlantedTypeState,
    ProviderState, ScriptState, Spec, TypeDesk, WorkersState,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, ClockMethod, CoreError, GoodId, Holder, Ledger, LoadError,
    LoadErrorKind, NodeId, ParamId, Phase, Provenance, RatePerYear, SimState, StateDelta, Tape,
    World,
};
use rustyecon_markets::{admit, Order, Side};

const LB1: &str = include_str!("../../../tapes/loops-lb1.ron");
const LW1: &str = include_str!("../../../tapes/loops-lw1.ron");
const H1: &str = include_str!("../../../tapes/horses-h1.ron");
const R1A: &str = include_str!("../../../tapes/horses-r1a.ron");

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

/// What loading `text` and building its cast refuses: the load's error, or the cast's.
fn refusal(text: &str) -> LoadError {
    match load_text(text) {
        Err(e) => e,
        Ok((w, _)) => match Cast::new(&w) {
            Err(e) => e,
            Ok(_) => panic!("the tape loads and casts"),
        },
    }
}

/// `text` with `old` replaced by `new`, once.
fn edit(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "{old}");
    text.replacen(old, new, 1)
}

fn actor(w: &W, key: &str) -> ActorId {
    w.id_of::<ActorId>(key).expect("an actor")
}

fn good(w: &W, key: &str) -> GoodId {
    w.id_of::<GoodId>(key).expect("a good")
}

fn param_id(w: &W, key: &str) -> ParamId {
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
        param: param_id(w, key),
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

fn price(s: &S, w: &W, g: &str) -> f64 {
    s.book().quote(home(w), good(w, g)).expect("a quote").price
}

/// A param's genesis value at its current setting, per tick by `method`.
fn per_tick(s: &S, w: &W, key: &str, method: ClockMethod) -> f64 {
    let v = s
        .params(&w.registry)
        .value(param_id(w, key))
        .expect("a value");
    method.per_tick(&w.clock, v).expect("a conversion")
}

fn value(s: &S, w: &W, key: &str) -> f64 {
    per_tick(s, w, key, ClockMethod::Value)
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

/// Every good with a market, by key.
fn traded(w: &W) -> Vec<(String, GoodId)> {
    w.goods
        .iter()
        .filter(|g| w.has_market(g.id))
        .map(|g| (g.key.to_string(), g.id))
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

/// A planted desk's plant record.
fn plant_of(st: ActorState) -> PlantState {
    match st {
        ActorState::PlantedType(p) => p.plant,
        ActorState::PlantedMaker(p) => p.plant,
        ActorState::PlantedCapacity(p) => p.plant,
        other => panic!("{other:?} is not planted"),
    }
}

/// A planted state with its plant record replaced.
fn with_plant(st: ActorState, plant: PlantState) -> ActorState {
    match st {
        ActorState::PlantedType(p) => ActorState::PlantedType(PlantedTypeState { plant, ..p }),
        ActorState::PlantedMaker(p) => ActorState::PlantedMaker(PlantedMakerState { plant, ..p }),
        ActorState::PlantedCapacity(p) => {
            ActorState::PlantedCapacity(PlantedCapacityState { plant, ..p })
        }
        other => panic!("{other:?} is not planted"),
    }
}

/// A planted state read as its kind's.
fn as_kind(st: ActorState) -> ActorState {
    match st {
        ActorState::PlantedType(p) => ActorState::MachDesk(p.desk),
        ActorState::PlantedMaker(p) => ActorState::Maker(p.desk),
        ActorState::PlantedCapacity(p) => ActorState::Capacity(p.desk),
        other => other,
    }
}

/// Deltas with every `SetState` read as its kind's, and every delta that moves `plant` left out.
fn kind_deltas(ds: Vec<Delta>, plant: GoodId) -> Vec<Delta> {
    ds.into_iter()
        .filter(|d| {
            !matches!(d, StateDelta::Mint { good, .. } | StateDelta::Burn { good, .. } if *good == plant)
        })
        .map(|d| match d {
            StateDelta::Actor(AgentDelta::SetState { actor, state }) => {
                StateDelta::Actor(AgentDelta::SetState {
                    actor,
                    state: as_kind(state),
                })
            }
            other => other,
        })
        .collect()
}

/// The planted desks of a tape: (actor, its plant good).
fn planted(w: &W) -> Vec<(ActorId, GoodId)> {
    w.actors
        .iter()
        .filter_map(|a| {
            let p = match &a.spec {
                Spec::TypeDesk(d) => d.plant,
                Spec::Maker(d) => d.plant,
                Spec::CapacityDesk(d) => d.plant,
                _ => None,
            }?;
            Some((a.id, p.good))
        })
        .collect()
}

/// A planted desk's bundle: each good with its coefficient now, in the kind's order (the type
/// desk's inputs, labour and land; the maker's build goods, labour and land at a = 0; the
/// capacity desk's running goods and labour).
fn bundle(s: &S, w: &W, a: ActorId) -> Vec<(GoodId, f64)> {
    let par = |site: rustyecon_core::Site| s.params(&w.registry).value(site.param).unwrap();
    match &w.actor(a).expect("a declared actor").spec {
        Spec::TypeDesk(t) => {
            let mut v: Vec<(GoodId, f64)> =
                t.inputs.iter().map(|x| (x.good, par(x.coef))).collect();
            v.push((t.labour, par(t.labour_coef)));
            v.push((t.land, par(t.land_coef)));
            v
        }
        Spec::Maker(m) => {
            let mut v: Vec<(GoodId, f64)> = m
                .build
                .goods
                .iter()
                .map(|x| (x.good, par(x.coef)))
                .collect();
            v.push((m.labour, par(m.build.labour)));
            v.push((m.land, par(m.build.land)));
            v
        }
        Spec::CapacityDesk(c) => {
            let mut v: Vec<(GoodId, f64)> = c
                .running
                .goods
                .iter()
                .map(|x| (x.good, par(x.coef)))
                .collect();
            v.push((c.labour, par(c.running.labour)));
            v
        }
        other => panic!("{other:?} is not a planted kind"),
    }
}

/// A planted desk's kind without its plant, and the planted role, each run on `s` with the state
/// each reads: the plant-free kind's decide, produce and upkeep beside the planted one's.
enum Pair {
    Type(TypeDesk, PlantedType),
    Maker(Box<MakerRole>, Box<PlantedMaker>),
    Capacity(CapacityDesk, PlantedCapacity),
}

type Hooks = (Decision, Vec<Delta>, Vec<Delta>);

impl Pair {
    fn of(w: &W, a: ActorId) -> Pair {
        let decl = w.actor(a).expect("a declared actor");
        match &decl.spec {
            Spec::TypeDesk(d) => {
                let p = d.plant.expect("a plant");
                let plain = TypeDesk {
                    plant: None,
                    ..d.clone()
                };
                Pair::Type(plain, PlantedType::new(d, &p))
            }
            Spec::Maker(d) => {
                let p = d.plant.expect("a plant");
                let plain = Maker {
                    plant: None,
                    ..d.clone()
                };
                Pair::Maker(
                    Box::new(MakerRole::new(&plain, false)),
                    Box::new(PlantedMaker::new(d, &p)),
                )
            }
            Spec::CapacityDesk(d) => {
                let p = d.plant.expect("a plant");
                let plain = CapacityDesk {
                    plant: None,
                    ..d.clone()
                };
                Pair::Capacity(plain, PlantedCapacity::new(d, &p))
            }
            other => panic!("{other:?} is not a planted kind"),
        }
    }

    /// (the plain kind's decide, produce, upkeep) and the planted one's, on `s`.
    fn run(&self, w: &W, s: &S, a: ActorId) -> (Hooks, Hooks) {
        let decl = w.actor(a).expect("a declared actor");
        let st = state(s, a);
        macro_rules! both {
            ($plain:expr, $planted:expr, $variant:ident, $desk:ty) => {{
                let ActorState::$variant(p) = st else {
                    panic!("{st:?}")
                };
                let desk: $desk = p.desk;
                let vp = view(decl, s, w, &desk).expect("a view");
                let vq = view(decl, s, w, &p).expect("a view");
                (
                    (
                        $plain.decide(&vp).expect("decide"),
                        $plain.produce(&vp).expect("produce"),
                        $plain.upkeep(&vp).expect("upkeep"),
                    ),
                    (
                        $planted.decide(&vq).expect("decide"),
                        $planted.produce(&vq).expect("produce"),
                        $planted.upkeep(&vq).expect("upkeep"),
                    ),
                )
            }};
        }
        match self {
            Pair::Type(plain, planted) => both!(plain, planted, PlantedType, MachDeskState),
            Pair::Maker(plain, planted) => both!(plain, planted, PlantedMaker, MakerState),
            Pair::Capacity(plain, planted) => {
                both!(plain, planted, PlantedCapacity, CapacityState)
            }
        }
    }
}

/// Random prices, coins, holdings of every good with a market and of every plant, plant records,
/// spending shares (1e4 a year, which rounds to a share of exactly 1, every seventh draw),
/// turnovers and tilts.
fn draw_state(w: &W, genesis: &S, d: &mut Draws, i: usize) -> S {
    let mut s = genesis.clone();
    let coin = good(w, "coin");
    for (_, g) in traded(w) {
        set_price(&mut s, w, g, d.decades(-6.0, 6.0));
    }
    for a in &w.actors {
        let c = if i.is_multiple_of(17) {
            0.0
        } else {
            d.decades(-9.0, 9.0)
        };
        set_holding(&mut s, w, a.id, coin, c);
        if a.id.kind() == rustyecon_core::ActorKind::Desk {
            for (_, g) in traded(w) {
                let q = if d.unit() < 0.1 {
                    0.0
                } else {
                    d.decades(-9.0, 9.0)
                };
                set_holding(&mut s, w, a.id, g, q);
            }
        }
    }
    for (a, p) in planted(w) {
        let q = if i.is_multiple_of(11) {
            0.0
        } else {
            d.decades(-9.0, 9.0)
        };
        set_holding(&mut s, w, a, p, q);
        let rec = PlantState {
            held: q,
            target: d.decades(-9.0, 9.0),
            order: if i.is_multiple_of(3) {
                0.0
            } else {
                d.decades(-9.0, 9.0)
            },
            run: d.decades(-9.0, 9.0),
            built: 0.0,
        };
        let st = state(&s, a);
        set_state(&mut s, w, a, with_plant(st, rec));
    }
    for k in params_with(w, &["spend.", "buffer.", "adjust."]) {
        let v = if i.is_multiple_of(7) {
            1e4
        } else {
            d.decades(-2.0, 3.0)
        };
        set_param(&mut s, w, &k, v);
    }
    for k in params_with(w, &["tilt."]) {
        set_param(&mut s, w, &k, 8.0 * d.unit());
    }
    s
}

#[test]
fn planted_at_theta_one_is_the_kind() {
    // LOOPS-RULES §3.9: at θ = 1 a planted type desk, maker and capacity desk make their kind's
    // orders, budgets, produce and upkeep deltas bit for bit, the plant's wear and the planted
    // state aside (c_full = c, kr = 0, no order, y = z_got = B, the budgets' total the outlay, one
    // burn per good), from 3,000 random states per tape: prices over twelve decades, coins,
    // stocks and plants over eighteen, plant records of any size, and spending shares that round
    // to 1.
    for text in [LB1, LW1] {
        let (w, mut genesis, _) = load(text);
        let desks = planted(&w);
        for k in params_with(&w, &["plant."]) {
            if k.ends_with(".theta") {
                set_param(&mut genesis, &w, &k, 1.0);
            }
        }
        let mut d = Draws(0x2026_0930_0001);
        for i in 0..3000 {
            let mut s = draw_state(&w, &genesis, &mut d, i);
            // Produce runs on the record decide leaves, which at θ = 1 orders nothing: the
            // drawn plan with no order.
            for &(a, _) in &desks {
                let st = state(&s, a);
                let rec = PlantState {
                    order: 0.0,
                    ..plant_of(st)
                };
                set_state(&mut s, &w, a, with_plant(st, rec));
            }
            for &(a, plant) in &desks {
                let pair = Pair::of(&w, a);
                let ((dp, pp, up), (dq, pq, uq)) = pair.run(&w, &s, a);
                let key = w.key_of(a).unwrap();
                assert_eq!(dp.orders, dq.orders, "draw {i}: {key}");
                assert_eq!(
                    dp.deltas,
                    kind_deltas(dq.deltas.clone(), plant),
                    "draw {i}: {key}"
                );
                // The decision's plant record: nothing targeted, nothing ordered.
                let rec = dq
                    .deltas
                    .iter()
                    .find_map(|x| match x {
                        StateDelta::Actor(AgentDelta::SetState { state, .. }) => {
                            Some(plant_of(*state))
                        }
                        _ => None,
                    })
                    .expect("a SetState");
                assert_eq!((rec.target, rec.order), (0.0, 0.0), "draw {i}: {key}");
                // Produce: the kind's burns and mints, and no plant built.
                assert_eq!(pp, kind_deltas(pq.clone(), plant), "draw {i}: {key}");
                assert!(
                    pq.iter()
                        .all(|x| !matches!(x, StateDelta::Mint { good, .. } if *good == plant)),
                    "draw {i}: {key}"
                );
                // Upkeep: the kind's, and the plant's wear beside it.
                assert_eq!(up, kind_deltas(uq, plant), "draw {i}: {key}");
            }
        }
    }
}

#[test]
fn old_states_keep_their_encoding() {
    // LOOPS-RULES §3.8: the planted variants are appended after `Owner`, so the eight old
    // variants keep their bincode encoding, which core hashes: the variant's index as a u32,
    // then its fields in order, little-endian.
    fn bytes(index: u32, fields: &[f64]) -> Vec<u8> {
        let mut v = index.to_le_bytes().to_vec();
        for f in fields {
            v.extend(f.to_le_bytes());
        }
        v
    }
    let enc = |s: &ActorState| bincode::serialize(s).expect("it encodes");
    let mut scripted = 0u32.to_le_bytes().to_vec();
    scripted.push(1);
    assert_eq!(
        enc(&ActorState::Scripted(ScriptState { active: true })),
        scripted
    );
    assert_eq!(
        enc(&ActorState::Provider(ProviderState {
            due: 1.5,
            paid: 0.25
        })),
        bytes(1, &[1.5, 0.25])
    );
    assert_eq!(
        enc(&ActorState::Workers(WorkersState { share: 0.5 })),
        bytes(2, &[0.5])
    );
    let g = GoodDeskState {
        share: 0.25,
        used: 0.5,
        scale: 2.0,
        output: 3.0,
    };
    assert_eq!(
        enc(&ActorState::GoodDesk(g)),
        bytes(3, &[0.25, 0.5, 2.0, 3.0])
    );
    let m = MachDeskState {
        scale: 2.0,
        output: 3.0,
    };
    assert_eq!(enc(&ActorState::MachDesk(m)), bytes(4, &[2.0, 3.0]));
    let mk = MakerState {
        scale: 1.0,
        output: 2.0,
        own: 3.0,
        serving: 4.0,
    };
    assert_eq!(enc(&ActorState::Maker(mk)), bytes(5, &[1.0, 2.0, 3.0, 4.0]));
    let c = CapacityState {
        scale: 1.0,
        output: 2.0,
        held: 3.0,
        target: 4.0,
        order: 5.0,
        run: 6.0,
    };
    assert_eq!(
        enc(&ActorState::Capacity(c)),
        bytes(6, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
    );
    let o = OwnerState {
        share: 0.25,
        used: 0.5,
        scale: 1.0,
        output: 2.0,
        serving: 3.0,
    };
    assert_eq!(
        enc(&ActorState::Owner(o)),
        bytes(7, &[0.25, 0.5, 1.0, 2.0, 3.0])
    );
    // The planted ones follow: the kind's state, then the plant's five numbers.
    let p = PlantState {
        held: 1.0,
        target: 2.0,
        order: 3.0,
        run: 4.0,
        built: 5.0,
    };
    let planted = ActorState::PlantedType(PlantedTypeState { desk: m, plant: p });
    assert_eq!(
        enc(&planted),
        bytes(8, &[2.0, 3.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    );
    assert_eq!(
        enc(&ActorState::PlantedMaker(PlantedMakerState {
            desk: mk,
            plant: p
        }))[..4],
        9u32.to_le_bytes()
    );
    assert_eq!(
        enc(&ActorState::PlantedCapacity(PlantedCapacityState {
            desk: c,
            plant: p
        }))[..4],
        10u32.to_le_bytes()
    );
}

#[test]
fn plants_are_checked_at_load() {
    // LOOPS-RULES §6: each refusal with its path.
    let at = |text: &str, path: &str| {
        let e = refusal(text);
        assert_eq!(e.path, path, "{e}");
        e
    };
    // Core: the plant good is untraded, so Indefinite with no price rate; without the flag a
    // good with no price rate is still a typo.
    let e = at(
        &edit(
            LB1,
            "(key: \"plant.fodder\", life: Indefinite, price_rate: None, untraded: true)",
            "(key: \"plant.fodder\", life: Indefinite, price_rate: None)",
        ),
        "goods[plant.fodder].price_rate",
    );
    assert_eq!(e.kind, LoadErrorKind::NoPriceRate);
    // The role: its plant names an untraded good.
    at(
        &edit(
            LB1,
            "plant: Some((good: \"plant.fodder\"",
            "plant: Some((good: \"fodder\"",
        ),
        "actors[desk.fodder].spec.plant.good",
    );
    // Its params have LOOPS-RULES §3.2's units.
    let e = at(
        &edit(
            LB1,
            "(key: \"plant.fodder.theta\", value: 0.8, unit: Dimensionless,",
            "(key: \"plant.fodder.theta\", value: 0.8, unit: FractionPerYear,",
        ),
        "actors[desk.fodder].spec.plant.theta",
    );
    assert!(matches!(e.kind, LoadErrorKind::UnitMismatch { .. }));
    let e = at(
        &edit(
            LB1,
            "(key: \"adjust.plant.fodder\", value: 0.2, unit: RatePerYear,",
            "(key: \"adjust.plant.fodder\", value: 0.2, unit: FractionPerYear,",
        ),
        "actors[desk.fodder].spec.plant.adjust",
    );
    assert!(matches!(e.kind, LoadErrorKind::UnitMismatch { .. }));
    // A plant good is no role's good: `traded` refuses it, and so does a scripted line.
    let e = at(
        &edit(
            LB1,
            "basket: [(good: \"good\", weight: \"inst.good.weight\"), (good: \"land\", weight: \
             \"inst.space.weight\")],\n            spend: \"spend.provider\",",
            "basket: [(good: \"plant.fodder\", weight: \"inst.good.weight\"), (good: \"land\", \
             weight: \"inst.space.weight\")],\n            spend: \"spend.provider\",",
        ),
        "actors[provider].spec.basket[plant.fodder].good",
    );
    assert_eq!(e.kind, LoadErrorKind::NoMarket);
    let e = at(
        &edit(
            LB1,
            "        (key: \"workers\", kind: Pop,",
            "        (key: \"trader\", kind: Pop, class: \"workers\", home: \"home\", basis: \
             Assumed(\"test\"), spec: Scripted((active: true, recipe: None, buy: [(node: \"home\", \
             good: \"plant.fodder\", qty: \"inst.land\", weight: 1.0)], sell: [], spend: \
             Some(\"spend.workers\"), payout: None))),\n        (key: \"workers\", kind: Pop,",
        ),
        "actors[trader].spec.buy[home/plant.fodder].good",
    );
    assert_eq!(e.kind, LoadErrorKind::NoMarket);
    // The cast's checks, at the genesis values.
    for (old, new, path) in [
        (
            "(key: \"plant.fodder.theta\", value: 0.8,",
            "(key: \"plant.fodder.theta\", value: 0.0,",
            "actors[desk.fodder].spec.plant.theta",
        ),
        (
            "(key: \"plant.fodder.theta\", value: 0.8,",
            "(key: \"plant.fodder.theta\", value: 1.5,",
            "actors[desk.fodder].spec.plant.theta",
        ),
        (
            "(key: \"plant.fodder.delta\", value: 0.1,",
            "(key: \"plant.fodder.delta\", value: 1.0,",
            "actors[desk.fodder].spec.plant.delta",
        ),
        (
            "(key: \"plant.fodder.size\", value: 40.47205917167808,",
            "(key: \"plant.fodder.size\", value: 0.0,",
            "actors[desk.fodder].spec.plant.size",
        ),
        (
            "adjust: \"adjust.plant.fodder\", order: Target, target: Bundles",
            "adjust: \"adjust.plant.fodder\", order: Target, target: Herd",
            "actors[desk.fodder].spec.plant.target",
        ),
        (
            "adjust: \"adjust.plant.maker\", order: Target, target: Bundles",
            "adjust: \"adjust.plant.maker\", order: Target, target: Herd",
            "actors[desk.maker].spec.plant.target",
        ),
        (
            "(key: \"inst.fodder.own\", value: 0.0,",
            "(key: \"inst.fodder.own\", value: 0.5,",
            "actors[desk.fodder].spec.recipe.own",
        ),
        (
            "(key: \"inst.horse.own_hours\", value: 0.0,",
            "(key: \"inst.horse.own_hours\", value: 0.1,",
            "actors[desk.maker].spec.own_hours",
        ),
        (
            "plant: Some((good: \"plant.maker\"",
            "plant: Some((good: \"plant.fodder\"",
            "actors[desk.maker].spec.plant.good",
        ),
        (
            "scale: Cash((turnover: \"buffer.desk.fodder.cash\", tilt: \"tilt.desk.fodder\", \
             payout: None)),",
            "scale: Step((up: \"rate.labour\", down: \"rate.labour\", dead: \"tilt.desk.fodder\", \
             buffer: \"buffer.desk.fodder.cash\", payout: (to: \"provider\", rate: \
             \"spend.provider\"), scale: 0.1)),",
            "actors[desk.fodder].spec.scale",
        ),
    ] {
        let e = at(&edit(LB1, old, new), path);
        assert!(matches!(e.kind, LoadErrorKind::Invalid(_)), "{e}");
    }
    // A bundle that costs nothing.
    let mut free = LB1.to_string();
    for (k, v) in [
        ("inst.fodder.traction", "2.0"),
        ("inst.fodder.labour", "8.0"),
        ("inst.fodder.land", "1.0"),
    ] {
        free = edit(
            &free,
            &format!("(key: \"{k}\", value: {v},"),
            &format!("(key: \"{k}\", value: 0.0,"),
        );
    }
    at(&free, "actors[desk.fodder].spec.plant");
    // A planted maker holds a stock: on the flow path it has no plant (R1a's maker, planted).
    let mut flow = edit(
        R1A,
        "        (key: \"coin\", life: Indefinite, price_rate: None),",
        "        (key: \"coin\", life: Indefinite, price_rate: None),\n        (key: \"plant.mach\", \
         life: Indefinite, price_rate: None, untraded: true),",
    );
    flow = edit(
        &flow,
        "    params: [",
        "    params: [\n        (key: \"plant.mach.theta\", value: 0.8, unit: Dimensionless, \
         basis: Assumed(\"test\")),\n        (key: \"plant.mach.delta\", value: 0.1, unit: \
         FractionPerYear, basis: Assumed(\"test\")),\n        (key: \"plant.mach.size\", value: \
         40.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \
         \"adjust.plant.mach\", value: 0.2, unit: RatePerYear, basis: Assumed(\"test\")),",
    );
    flow = edit(
        &flow,
        "            scale: Cash((turnover: \"buffer.desk.mach.cash\", tilt: \"tilt.desk.mach\", \
         payout: None)),",
        "            scale: Cash((turnover: \"buffer.desk.mach.cash\", tilt: \"tilt.desk.mach\", \
         payout: None)),\n            plant: Some((good: \"plant.mach\", theta: \
         \"plant.mach.theta\", delta: \"plant.mach.delta\", size: \"plant.mach.size\", adjust: \
         \"adjust.plant.mach\", order: Target, target: Bundles)),",
    );
    at(&flow, "actors[desk.mach].spec.plant");
    // `plant` on any other kind is an unknown field.
    let e = refusal(&edit(
        LB1,
        "            assign: ExPost,\n",
        "            assign: ExPost,\n            plant: None,\n",
    ));
    assert!(matches!(e.kind, LoadErrorKind::Parse(_)), "{e}");
    // The canonical text: none of the new fields without a plant, and a planted tape round-trips.
    let h1 = Tape::<Agents>::from_ron(H1).unwrap().to_ron();
    assert!(!h1.contains("plant") && !h1.contains("untraded"));
    let lb1 = Tape::<Agents>::from_ron(LB1).unwrap().to_ron();
    assert_eq!(Tape::<Agents>::from_ron(&lb1).unwrap().to_ron(), lb1);
    assert!(lb1.contains("untraded: true") && lb1.contains("plant: Some("));
    // `sites` lists the four params with their methods, as the registry does.
    let (w, _, _) = load(LB1);
    let fodder = w.actor(actor(&w, "desk.fodder")).unwrap();
    let sites = fodder.spec.sites(&w);
    for (path, key, method) in [
        ("plant.theta", "plant.fodder.theta", ClockMethod::Value),
        ("plant.delta", "plant.fodder.delta", ClockMethod::Fraction),
        ("plant.size", "plant.fodder.size", ClockMethod::Value),
        ("plant.adjust", "adjust.plant.fodder", ClockMethod::Share),
    ] {
        let site = sites
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, s)| *s)
            .expect("a site");
        assert_eq!(
            (site.param, site.method),
            (param_id(&w, key), method),
            "{path}"
        );
        let def = w.registry.get(param_id(&w, key)).unwrap();
        assert_eq!(def.sites.len(), 1, "{key}");
        assert_eq!(
            def.sites[0].path,
            format!("actors[desk.fodder].spec.{path}")
        );
        assert_eq!(def.sites[0].method, method);
    }
}

/// A plant's params as the test reads them: θ, u, s, s_p.
fn plant_params(s: &S, w: &W, d: &str, adjust: &str) -> (f64, f64, f64, f64) {
    (
        value(s, w, &format!("plant.{d}.theta")),
        per_tick(s, w, &format!("plant.{d}.delta"), ClockMethod::Fraction),
        value(s, w, &format!("plant.{d}.size")),
        per_tick(s, w, adjust, ClockMethod::Share),
    )
}

/// LOOP-SPEC §2.2's ratios, as the test writes them: (P_K, kr, c_full, ζ, κ_p).
fn ratios(theta: f64, u: f64, size: f64, c: f64) -> (f64, f64, f64, f64, f64) {
    let pk = size * c;
    let kr = ((1.0 - theta) * c) / ((theta * u) * pk);
    let full = c + (u * pk) * kr;
    (
        pk,
        kr,
        full,
        num::pow(kr, -(1.0 - theta)),
        num::pow(kr, theta),
    )
}

/// |a − b|/|b|.
fn rel(a: f64, b: f64) -> f64 {
    ((a - b) / b).abs()
}

fn sign(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

/// The buy order for `g` among `orders`: (qty, budget).
fn bought(orders: &[Order], g: GoodId) -> (f64, f64) {
    orders
        .iter()
        .find_map(|o| match o.side {
            Side::Buy { budget } if o.good == g => Some((o.qty, budget)),
            _ => None,
        })
        .expect("a buy order")
}

/// The budget chain as MARKETS-SPEC §2.6 cuts it, from `total` over `wants` in good order.
fn chain(total: f64, coin: f64, wants: &[(GoodId, f64)]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..wants.len()).collect();
    order.sort_by_key(|&i| wants[i].0);
    let mut out = vec![0.0; wants.len()];
    let (mut spent, mut left) = (0.0, coin);
    for (n, &i) in order.iter().enumerate() {
        let room = if n == 0 {
            total
        } else {
            num::max_remainder(total, spent).unwrap()
        };
        let b = wants[i].1.min(room).min(left);
        left -= b;
        spent += b;
        out[i] = b;
    }
    out
}

#[test]
fn planted_rules_are_the_spec() {
    // LOOPS-RULES §3–§4 on LB1: one tick of each planted kind at fixed states, each plant half
    // its target so it orders, the coin cap slack and then binding. Every quantity is a literal
    // formed here from LOOP-SPEC §2.2–§2.5: P_K, kr, c_full, O_f and zcap, q or z, K*_p (`Herd`
    // and `Bundles`), I (`Target` and `Gap`), the horse order after the plant's cost, the lines,
    // the budgets' total, B, the split, y, used, I′, the burns, the mints and the wear.
    // The cases: M3's rule with the coin cap slack; M3's rule closing the whole gap a tick
    // (s_Kp 1e4 a year), where the coin cap binds; `Gap` (LN6), which binds it too; and each
    // plant twice s1, off the size at which O_f is O (each plant at a quarter of its genesis, so
    // it still orders).
    for (case, order_rule) in [
        ("slack", "Target"),
        ("capped", "Target"),
        ("gap", "Gap"),
        ("off s1", "Target"),
    ] {
        let text = if order_rule == "Gap" {
            LB1.replace("order: Target, target:", "order: Gap, target:")
        } else {
            LB1.to_string()
        };
        let (w, genesis, cast) = load(&text);
        let coin = good(&w, "coin");
        let (fodder, horse, traction, labour, land) = (
            good(&w, "fodder"),
            good(&w, "horse"),
            good(&w, "traction"),
            good(&w, "labour"),
            good(&w, "land"),
        );
        let share = |key: &str, s: &S| {
            w.clock.share(RatePerYear(
                s.params(&w.registry).value(param_id(&w, key)).unwrap(),
            ))
        };
        let mut s = genesis.clone();
        let (w_, r, pf, ph, pk) = (
            price(&s, &w, "labour"),
            price(&s, &w, "land"),
            price(&s, &w, "fodder"),
            price(&s, &w, "traction"),
            price(&s, &w, "horse"),
        );
        for d in ["fodder", "maker", "capacity"] {
            let a = actor(&w, &format!("desk.{d}"));
            let p = good(&w, &format!("plant.{d}"));
            let part = if case == "off s1" { 0.25 } else { 0.5 };
            let half = held(&s, a, p) * part;
            set_holding(&mut s, &w, a, p, half);
            if case == "capped" {
                set_param(&mut s, &w, &format!("adjust.plant.{d}"), 1e4);
            }
            if case == "off s1" {
                let size = value(&s, &w, &format!("plant.{d}.size"));
                set_param(&mut s, &w, &format!("plant.{d}.size"), 2.0 * size);
            }
        }
        let binds = case == "capped" || case == "gap";
        let gap = order_rule == "Gap";
        let order_of = |u: f64, sp: f64, target: f64, p: f64| {
            if gap {
                sign((u * p) + (target - p))
            } else {
                sign((u * target) + (sp * (target - p)))
            }
        };
        let out = cast
            .decide(actor(&w, "desk.fodder"), &s, &w)
            .expect("decide");
        // The fodder desk (a type desk).
        {
            let a = actor(&w, "desk.fodder");
            let (th, u, sz, sp) = plant_params(&s, &w, "fodder", "adjust.plant.fodder");
            let (ah, lam, b) = (
                value(&s, &w, "inst.fodder.traction"),
                value(&s, &w, "inst.fodder.labour"),
                value(&s, &w, "inst.fodder.land"),
            );
            let c = ((0.0 + ah * ph) + lam * w_) + b * r;
            let coin_c = held(&s, a, coin);
            let outlay = share("buffer.desk.fodder.cash", &s) * coin_c;
            let (ppk, kr, full, _, _) = ratios(th, u, sz, c);
            let q = outlay / full;
            let p = held(&s, a, good(&w, "plant.fodder"));
            let target = kr * q;
            let i = order_of(u, sp, target, p).min(sign(coin_c - c * q) / ppk);
            let n = q + sz * i;
            assert!(i > 0.0, "{case}: the fodder desk orders");
            let wants = [
                (traction, ph * (ah * n)),
                (labour, w_ * (lam * n)),
                (land, r * (b * n)),
            ];
            let budgets = chain(outlay + ppk * i, coin_c, &wants);
            for (k, (g, qty)) in [(traction, ah * n), (labour, lam * n), (land, b * n)]
                .into_iter()
                .enumerate()
            {
                assert_eq!(
                    bought(&out.orders, g),
                    (qty, budgets[k]),
                    "{case}: fodder {k}"
                );
            }
            let rec = out
                .deltas
                .iter()
                .find_map(|x| match x {
                    StateDelta::Actor(AgentDelta::SetState { state, .. }) => Some(plant_of(*state)),
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                (rec.held, rec.target, rec.order, rec.run),
                (p, target, i, q),
                "{case}: the fodder desk's record"
            );
            assert_eq!(
                i == sign(coin_c - c * q) / ppk,
                binds,
                "{case}: the fodder desk's cap"
            );
        }
        // The capacity desk.
        let out = cast
            .decide(actor(&w, "desk.capacity"), &s, &w)
            .expect("decide");
        {
            let a = actor(&w, "desk.capacity");
            let (th, u, sz, sp) = plant_params(&s, &w, "capacity", "adjust.plant.capacity");
            let (fo, lo) = (
                value(&s, &w, "inst.horse.run.fodder"),
                value(&s, &w, "inst.horse.run.labour"),
            );
            let kappa = per_tick(&s, &w, "inst.horse.kappa", ClockMethod::Flow);
            let d = per_tick(&s, &w, "inst.horse.delta", ClockMethod::Fraction);
            let sk = per_tick(&s, &w, "adjust.invest.capacity", ClockMethod::Share);
            let o = (0.0 + fo * pf) + lo * w_;
            let coin_c = held(&s, a, coin);
            let h = held(&s, a, horse);
            let outlay = share("buffer.desk.capacity.cash", &s) * coin_c;
            let (ppk, _, _, zeta, kap) = ratios(th, u, sz, o);
            let o_f = (zeta * o) + ((u * ppk) * kap);
            // O_f is O at s1 (to rounding), and apart from it off s1.
            assert_eq!(
                rel(o_f, o) > 1e-3,
                case == "off s1",
                "{case}: O_f {o_f:e}, O {o:e}"
            );
            let p = held(&s, a, good(&w, "plant.capacity"));
            let zcap = num::pow((kappa * h) / num::pow(p, 1.0 - th), 1.0 / th);
            let z = zcap.min(outlay / o);
            let kstar = outlay / (kappa * o_f + d * pk);
            let want = sign(d * kstar + sk * (kstar - h));
            let target = (kap * kappa) * kstar;
            let rest = coin_c - o * z;
            let i = order_of(u, sp, target, p).min(sign(rest) / ppk);
            let m = want.min(sign(rest - ppk * i) / pk);
            let n = z + sz * i;
            assert!(i > 0.0, "{case}: the capacity desk orders");
            assert_eq!(
                i == sign(rest) / ppk,
                binds,
                "{case}: the capacity desk's cap"
            );
            let wants = [
                (fodder, pf * (fo * n)),
                (labour, w_ * (lo * n)),
                (horse, pk * m),
            ];
            let budgets = chain(coin_c, coin_c, &wants);
            for (k, (g, qty)) in [(fodder, fo * n), (labour, lo * n), (horse, m)]
                .into_iter()
                .enumerate()
            {
                assert_eq!(
                    bought(&out.orders, g),
                    (qty, budgets[k]),
                    "{case}: capacity {k}"
                );
            }
            let st = out
                .deltas
                .iter()
                .find_map(|x| match x {
                    StateDelta::Actor(AgentDelta::SetState { state, .. }) => Some(*state),
                    _ => None,
                })
                .unwrap();
            let ActorState::PlantedCapacity(st) = st else {
                panic!("{st:?}")
            };
            assert_eq!(
                (st.desk.held, st.desk.target, st.desk.order, st.desk.run),
                (h, kstar, m, z),
                "{case}"
            );
            assert_eq!(
                (st.plant.held, st.plant.target, st.plant.order, st.plant.run),
                (p, target, i, z),
                "{case}"
            );
        }
        // The maker.
        let out = cast
            .decide(actor(&w, "desk.maker"), &s, &w)
            .expect("decide");
        {
            let a = actor(&w, "desk.maker");
            let (th, u, sz, sp) = plant_params(&s, &w, "maker", "adjust.plant.maker");
            let (fb, lb, bb) = (
                value(&s, &w, "inst.horse.fodder"),
                value(&s, &w, "inst.horse.labour"),
                value(&s, &w, "inst.horse.land"),
            );
            let fo = value(&s, &w, "inst.horse.run.fodder");
            let lo = value(&s, &w, "inst.horse.run.labour");
            // a = 0: a·run_g + build_g, a·run_lab + build_lab.
            let (fm, lm) = (0.0 * fo + fb, 0.0 * lo + lb);
            let cm = ((0.0 + fm * pf) + lm * w_) + bb * r;
            let coin_c = held(&s, a, coin);
            let outlay = share("buffer.desk.maker.cash", &s) * coin_c;
            let (ppk, kr, full, _, _) = ratios(th, u, sz, cm);
            let q = outlay / full;
            let p = held(&s, a, good(&w, "plant.maker"));
            let target = kr * q;
            let i = order_of(u, sp, target, p).min(sign(coin_c - cm * q) / ppk);
            let n = q + sz * i;
            assert!(i > 0.0, "{case}: the maker orders");
            assert_eq!(
                i == sign(coin_c - cm * q) / ppk,
                binds,
                "{case}: the maker's cap"
            );
            let finished = held(&s, a, horse);
            let cover = per_tick(&s, &w, "cover.maker", ClockMethod::Ticks);
            let offer = sign(finished - 0.0 - cover * q * full / pk);
            let sold = out
                .orders
                .iter()
                .find(|o| o.side == Side::Sell)
                .expect("an offer");
            assert_eq!(
                sold.qty, offer,
                "{case}: the maker's offer at the full cost"
            );
            let wants = [
                (fodder, pf * (fm * n)),
                (labour, w_ * (lm * n)),
                (land, r * (bb * n)),
            ];
            let budgets = chain(outlay + ppk * i, coin_c, &wants);
            for (k, (g, qty)) in [(fodder, fm * n), (labour, lm * n), (land, bb * n)]
                .into_iter()
                .enumerate()
            {
                assert_eq!(
                    bought(&out.orders, g),
                    (qty, budgets[k]),
                    "{case}: maker {k}"
                );
            }
        }
        // Produce and upkeep, one tick, from the orders as admitted and filled at whatever the
        // markets give: every desk's B, split, y, used, I′, burns, mints and wear, from its
        // holdings and records after decide.
        decide_and_admit(&mut s, &w, &cast);
        for (d, herd) in [("fodder", false), ("maker", false), ("capacity", true)] {
            let a = actor(&w, &format!("desk.{d}"));
            let p = good(&w, &format!("plant.{d}"));
            let rec = plant_of(state(&s, a));
            let (th, u, sz, _) = plant_params(&s, &w, d, &format!("adjust.plant.{d}"));
            // Hold every bundle good at a known amount: the plan's bundles with a carry of
            // labour's share off, so labour binds.
            let tot = rec.run + sz * rec.order;
            let lists: Vec<(GoodId, f64)> = match d {
                "fodder" => vec![
                    (traction, value(&s, &w, "inst.fodder.traction")),
                    (labour, value(&s, &w, "inst.fodder.labour")),
                    (land, value(&s, &w, "inst.fodder.land")),
                ],
                "maker" => vec![
                    (fodder, value(&s, &w, "inst.horse.fodder")),
                    (labour, value(&s, &w, "inst.horse.labour")),
                    (land, value(&s, &w, "inst.horse.land")),
                ],
                _ => vec![
                    (fodder, value(&s, &w, "inst.horse.run.fodder")),
                    (labour, value(&s, &w, "inst.horse.run.labour")),
                ],
            };
            for &(g, coef) in &lists {
                let q = if g == labour {
                    coef * tot * 0.75
                } else {
                    coef * tot * 1.5
                };
                set_holding(&mut s, &w, a, g, q);
            }
            let bundles = lists
                .iter()
                .map(|&(g, coef)| num::max_scale(held(&s, a, g), coef).unwrap())
                .fold(f64::MAX, f64::min);
            let (zg, ig) = (
                ((bundles * rec.run) / tot).min(bundles),
                (bundles * rec.order) / tot,
            );
            let made = num::pow(rec.held, 1.0 - th) * num::pow(zg, th);
            let (y, used) = if !herd {
                (made, zg)
            } else {
                let kappa = per_tick(&s, &w, "inst.horse.kappa", ClockMethod::Flow);
                let ActorState::PlantedCapacity(c) = state(&s, a) else {
                    panic!()
                };
                let y = (kappa * c.desk.held).min(made);
                let used = num::pow(y / num::pow(rec.held, 1.0 - th), 1.0 / th).min(zg);
                (y, used)
            };
            let built =
                ig.min(num::max_scale(num::max_remainder(bundles, used).unwrap(), sz).unwrap());
            let total = used + sz * built;
            assert!(built > 0.0 && total <= bundles, "{case}: {d}");
            let out = cast.produce(a, &s, &w).expect("produce");
            let me = Holder::Actor(a);
            let output = match d {
                "fodder" => fodder,
                "maker" => horse,
                _ => traction,
            };
            let mut want: Vec<Delta> = lists
                .iter()
                .map(|&(g, coef)| StateDelta::Burn {
                    from: me,
                    good: g,
                    amount: Amount::Qty(coef * total),
                    prov: Provenance::Production,
                })
                .collect();
            want.push(StateDelta::Mint {
                to: me,
                good: output,
                qty: y,
                prov: Provenance::Production,
            });
            want.push(StateDelta::Mint {
                to: me,
                good: p,
                qty: built,
                prov: Provenance::Production,
            });
            let got: Vec<Delta> = out
                .iter()
                .filter(|x| !matches!(x, StateDelta::Actor(_)))
                .cloned()
                .collect();
            assert_eq!(got, want, "{case}: {d}'s produce");
            let after = with_plant(state(&s, a), PlantState { built, ..rec });
            let set = out.last().unwrap();
            assert_eq!(
                set,
                &StateDelta::Actor(AgentDelta::SetState {
                    actor: a,
                    state: match after {
                        ActorState::PlantedType(x) => ActorState::PlantedType(PlantedTypeState {
                            desk: MachDeskState {
                                output: y,
                                ..x.desk
                            },
                            ..x
                        }),
                        ActorState::PlantedMaker(x) =>
                            ActorState::PlantedMaker(PlantedMakerState {
                                desk: MakerState {
                                    output: y,
                                    ..x.desk
                                },
                                ..x
                            }),
                        ActorState::PlantedCapacity(x) => {
                            ActorState::PlantedCapacity(PlantedCapacityState {
                                desk: CapacityState {
                                    output: y,
                                    ..x.desk
                                },
                                ..x
                            })
                        }
                        other => other,
                    }
                }),
                "{case}: {d}'s record"
            );
            apply_in(&mut s, &w, Phase::Production, &out).expect("the produce applies");
            // Upkeep: the plant wears u·P as recorded at decide, never what was just built.
            let up = cast.upkeep(a, &s, &w).expect("upkeep");
            let wear = up
                .iter()
                .find_map(|x| match x {
                    StateDelta::Burn {
                        good,
                        amount: Amount::Qty(q),
                        prov: Provenance::Depreciation,
                        ..
                    } if *good == p => Some(*q),
                    _ => None,
                })
                .expect("the plant wears");
            assert_eq!(wear, u * rec.held, "{case}: {d}'s wear");
        }
    }
}

#[test]
fn planted_arithmetic_never_fails() {
    // LOOPS-RULES §3.5–§3.6's arithmetic that cannot fail: from 3,000 random states per tape,
    // with carried goods beyond the plan and bundle goods held one ulp either side of a bundle
    // count, admission accepts every order, every burn in produce fits its holding, and every
    // wear at upkeep fits too.
    for text in [LB1, LW1] {
        let (w, genesis, cast) = load(text);
        let desks = planted(&w);
        let actors: Vec<ActorId> = cast.actors().collect();
        let mut d = Draws(0x2026_0930_0002);
        for i in 0..3000 {
            let mut s = draw_state(&w, &genesis, &mut d, i);
            decide_and_admit(&mut s, &w, &cast);
            for &(a, _) in &desks {
                // Every bundle good held at B bundles, an ulp either side, or with a carry.
                let rec = plant_of(state(&s, a));
                let b = rec.run + d.unit() * rec.run;
                let list = bundle(&s, &w, a);
                for (g, coef) in list {
                    let q = coef * b;
                    let q = match i % 4 {
                        0 => f64::from_bits(q.to_bits() + 1),
                        1 if q > 0.0 => f64::from_bits(q.to_bits() - 1),
                        2 => q * (1.0 + 3.0 * d.unit()),
                        _ => q,
                    };
                    set_holding(&mut s, &w, a, g, q);
                }
            }
            for &a in &actors {
                let out = cast.produce(a, &s, &w).expect("produce runs");
                apply_in(&mut s, &w, Phase::Production, &out)
                    .unwrap_or_else(|e| panic!("draw {i}: {}: {e}", w.key_of(a).unwrap()));
            }
            for &a in &actors {
                let out = cast.upkeep(a, &s, &w).expect("upkeep runs");
                apply_in(&mut s, &w, Phase::Upkeep, &out)
                    .unwrap_or_else(|e| panic!("draw {i}: {}: {e}", w.key_of(a).unwrap()));
            }
        }
    }
}

#[test]
fn planted_desks_take_carried_goods() {
    // Decision 273, LOOPS-RULES §5: a planted desk reads its bundles over every unit it holds, a
    // carry included, bound by that tick's labour; the split is the plan's; with no plant order
    // all of B runs, beyond the plan too where the labour allows (the mirror's carry branch).
    let (w, genesis, cast) = load(LB1);
    let a = actor(&w, "desk.fodder");
    let (traction, labour, land, fodder) = (
        good(&w, "traction"),
        good(&w, "labour"),
        good(&w, "land"),
        good(&w, "fodder"),
    );
    let pgood = good(&w, "plant.fodder");
    for (order, hands) in [(0.0, 0.8), (0.0, 1.5), (1e-4, 0.8)] {
        let mut s = genesis.clone();
        let p = held(&s, a, pgood);
        let rec = PlantState {
            held: p,
            target: p,
            order,
            run: 0.05,
            built: 0.0,
        };
        let st = state(&s, a);
        set_state(&mut s, &w, a, with_plant(st, rec));
        let sz = value(&s, &w, "plant.fodder.size");
        let tot = rec.run + sz * order;
        // Horse-days carried beyond the plan (twice the plan's), land beyond it (three times),
        // and labour for 0.8 or 1.5 of the plan's bundles, so labour binds.
        set_holding(&mut s, &w, a, traction, 2.0 * tot * 2.0);
        set_holding(&mut s, &w, a, land, tot * 3.0);
        set_holding(&mut s, &w, a, labour, 8.0 * tot * hands);
        let b = num::max_scale(held(&s, a, labour), 8.0).unwrap();
        assert_eq!(b > tot, hands > 1.0);
        let out = cast.produce(a, &s, &w).expect("produce");
        let burned = |g: GoodId| {
            out.iter()
                .find_map(|x| match x {
                    StateDelta::Burn {
                        good,
                        amount: Amount::Qty(q),
                        ..
                    } if *good == g => Some(*q),
                    _ => None,
                })
                .unwrap_or(0.0)
        };
        let minted = |g: GoodId| {
            out.iter()
                .find_map(|x| match x {
                    StateDelta::Mint { good, qty, .. } if *good == g => Some(*qty),
                    _ => None,
                })
                .unwrap_or(0.0)
        };
        let th = value(&s, &w, "plant.fodder.theta");
        if order == 0.0 {
            // All of B runs: labour binds, the carried horse-days and land are read too.
            assert_eq!(burned(labour), 8.0 * b);
            assert_eq!(burned(traction), 2.0 * b);
            assert_eq!(burned(land), 1.0 * b);
            assert_eq!(minted(fodder), num::pow(p, 1.0 - th) * num::pow(b, th));
            assert_eq!(minted(pgood), 0.0);
        } else {
            let zg = ((b * rec.run) / tot).min(b);
            let ig = (b * order) / tot;
            let built = ig.min(num::max_scale(num::max_remainder(b, zg).unwrap(), sz).unwrap());
            assert_eq!(minted(fodder), num::pow(p, 1.0 - th) * num::pow(zg, th));
            assert_eq!(minted(pgood), built);
            assert_eq!(burned(labour), 8.0 * (zg + sz * built));
            assert!(zg + sz * built <= b);
        }
    }
    // The split never runs more than B (LOOPS-RULES §3.6 step 2): with an order too small to
    // register in z + s·I, fl(B·z)/fl(z + s·I) is fl(B·z)/z, which can round an ulp above B.
    // Find such a B, held as labour (it binds), and produce: the share run is at most B, and
    // every burn fits.
    let mut s = genesis.clone();
    let z = 0.05;
    let b = (1..100_000)
        .map(|k| num::max_scale(8.0 * (0.04 + f64::from(k) * 1e-7), 8.0).unwrap())
        .find(|&b| (b * z) / z > b)
        .expect("a bundle count whose share rounds above it");
    let p = held(&s, a, pgood);
    let rec = PlantState {
        held: p,
        target: p,
        order: z * 1e-30,
        run: z,
        built: 0.0,
    };
    let st = state(&s, a);
    set_state(&mut s, &w, a, with_plant(st, rec));
    set_holding(&mut s, &w, a, traction, 2.0 * z * 2.0);
    set_holding(&mut s, &w, a, land, z * 3.0);
    set_holding(&mut s, &w, a, labour, 8.0 * b);
    assert_eq!(num::max_scale(held(&s, a, labour), 8.0).unwrap(), b);
    let out = cast.produce(a, &s, &w).expect("produce runs");
    apply_in(&mut s, &w, Phase::Production, &out).expect("every burn fits");
}

#[test]
fn plant_wears_what_it_held_at_decide() {
    // LOOPS-RULES §3.7: upkeep burns u·P with P the plant recorded at decide; units built this
    // tick do not wear this tick; after the tick the plant is P + I′ − u·P. One tick on LB1 with
    // every plant at half its genesis, so each desk orders and builds.
    let (w, genesis, cast) = load(LB1);
    let mut s = genesis.clone();
    let desks = planted(&w);
    for &(a, p) in &desks {
        let half = held(&s, a, p) * 0.5;
        set_holding(&mut s, &w, a, p, half);
    }
    let before: Vec<f64> = desks.iter().map(|&(a, p)| held(&s, a, p)).collect();
    decide_and_admit(&mut s, &w, &cast);
    // Each desk holds its plan's bundles, running and build, as a full fill would give them.
    for &(a, _) in &desks {
        let rec = plant_of(state(&s, a));
        let size = value(
            &s,
            &w,
            &format!(
                "plant.{}.size",
                w.key_of(a).unwrap().as_str().strip_prefix("desk.").unwrap()
            ),
        );
        let tot = rec.run + size * rec.order;
        for (g, coef) in bundle(&s, &w, a) {
            set_holding(&mut s, &w, a, g, coef * tot);
        }
    }
    let actors: Vec<ActorId> = cast.actors().collect();
    for &a in &actors {
        let out = cast.produce(a, &s, &w).unwrap();
        apply_in(&mut s, &w, Phase::Production, &out).unwrap();
    }
    for (k, &(a, p)) in desks.iter().enumerate() {
        let rec = plant_of(state(&s, a));
        assert_eq!(rec.held, before[k]);
        assert!(rec.built > 0.0, "{}: builds", w.key_of(a).unwrap());
        let d = w
            .key_of(a)
            .unwrap()
            .as_str()
            .strip_prefix("desk.")
            .unwrap()
            .to_string();
        let u = per_tick(&s, &w, &format!("plant.{d}.delta"), ClockMethod::Fraction);
        let up = cast.upkeep(a, &s, &w).unwrap();
        let wear: Vec<f64> = up
            .iter()
            .filter_map(|x| match x {
                StateDelta::Burn {
                    good,
                    amount: Amount::Qty(q),
                    prov: Provenance::Depreciation,
                    ..
                } if *good == p => Some(*q),
                _ => None,
            })
            .collect();
        assert_eq!(wear, vec![u * rec.held], "{d}");
        apply_in(&mut s, &w, Phase::Upkeep, &up).unwrap();
        assert_eq!(held(&s, a, p), (rec.held + rec.built) - u * rec.held, "{d}");
    }
}

#[test]
fn fixed_plant_is_drs() {
    // LOOPS-RULES §3.9: at δ_p 0 (u = 0), over 200 random states, no plant is ordered and none
    // wears, and output is P^(1−θ)·B^θ from the plant held (at most κ·H on the capacity desk).
    for text in [LB1, LW1] {
        let (w, mut genesis, cast) = load(text);
        for k in params_with(&w, &["plant.", "adjust.plant."]) {
            if k.ends_with(".delta") || k.starts_with("adjust.plant.") {
                set_param(&mut genesis, &w, &k, 0.0);
            }
        }
        let desks = planted(&w);
        let mut d = Draws(0x2026_0930_0003);
        for i in 0..200 {
            let mut s = draw_state(&w, &genesis, &mut d, i);
            let out = decide_and_admit(&mut s, &w, &cast);
            for &(a, p) in &desks {
                let rec = plant_of(state(&s, a));
                assert_eq!((rec.order, rec.target), (0.0, 0.0), "draw {i}");
                let dec = &out.iter().find(|(x, _)| *x == a).unwrap().1;
                assert!(dec.orders.iter().all(|o| o.good != p));
                let pr = cast.produce(a, &s, &w).unwrap();
                assert!(pr
                    .iter()
                    .all(|x| !matches!(x, StateDelta::Mint { good, .. } if *good == p)));
                // B, the bundles held, and the output on the plant held.
                let decl = w.actor(a).unwrap();
                let list = bundle(&s, &w, a);
                let cap = match state(&s, a) {
                    ActorState::PlantedCapacity(st) => {
                        let kappa = per_tick(&s, &w, "inst.horse.kappa", ClockMethod::Flow);
                        Some(kappa * st.desk.held)
                    }
                    _ => None,
                };
                let b = list
                    .iter()
                    .filter(|(_, c)| *c > 0.0)
                    .map(|&(g, c)| num::max_scale(held(&s, a, g), c).unwrap())
                    .fold(f64::MAX, f64::min);
                let th = s
                    .params(&w.registry)
                    .value(match &decl.spec {
                        Spec::TypeDesk(t) => t.plant.unwrap().theta.param,
                        Spec::Maker(m) => m.plant.unwrap().theta.param,
                        Spec::CapacityDesk(c) => c.plant.unwrap().theta.param,
                        _ => unreachable!(),
                    })
                    .unwrap();
                let made = if b > 0.0 {
                    num::pow(rec.held, 1.0 - th) * num::pow(b, th)
                } else {
                    0.0
                };
                let want = cap.map_or(made, |c| c.min(made));
                let output = match &decl.spec {
                    Spec::TypeDesk(t) => t.output,
                    Spec::Maker(m) => m.output,
                    Spec::CapacityDesk(c) => c.hours,
                    _ => unreachable!(),
                };
                let got = pr
                    .iter()
                    .find_map(|x| match x {
                        StateDelta::Mint { good, qty, .. } if *good == output => Some(*qty),
                        _ => None,
                    })
                    .unwrap_or(0.0);
                assert_eq!(got, want, "draw {i}: {}", w.key_of(a).unwrap());
                apply_in(&mut s, &w, Phase::Production, &pr).unwrap();
                let up = cast.upkeep(a, &s, &w).unwrap();
                assert!(up
                    .iter()
                    .all(|x| !matches!(x, StateDelta::Burn { good, .. } if *good == p)));
            }
        }
    }
}

#[test]
fn a_planted_maker_reads_its_reservation_at_c_m() {
    // Decision 295: the planted maker's margin reads the bundle cost c_m, not c_full, and so does
    // its reservation. At a markup on c_m above ψ but below it on c_full, it offers; below ψ on
    // c_m it offers nothing; `maker_reservation` reads the same.
    let (w, genesis, cast) = load(LB1);
    let a = actor(&w, "desk.maker");
    let decl = w.actor(a).unwrap();
    let Spec::Maker(m) = &decl.spec else { panic!() };
    let horse = good(&w, "horse");
    let s0 = genesis.clone();
    let (w_, r, pf) = (
        price(&s0, &w, "labour"),
        price(&s0, &w, "land"),
        price(&s0, &w, "fodder"),
    );
    let cm = ((0.0 + 8.0 * pf) + 20.0 * w_) + value(&s0, &w, "inst.horse.land") * r;
    let (th, u, sz, _) = plant_params(&s0, &w, "maker", "adjust.plant.maker");
    let (_, _, full, _, _) = ratios(th, u, sz, cm);
    let psi = value(&s0, &w, "reserve.maker");
    assert!(full > cm);
    // Between ψ and ψ·c_full/c_m: above ψ on c_m, below it on c_full; then half ψ on c_m.
    for (markup, offers) in [((psi + psi * full / cm) / 2.0, true), (psi * 0.5, false)] {
        assert!(!offers || (markup > psi && markup * cm / full < psi));
        let mut s = s0.clone();
        let pk = markup * cm;
        set_price(&mut s, &w, horse, pk);
        // Enough finished heads that the cover leaves an offer.
        let heads = held(&s, a, horse) * 1e3;
        set_holding(&mut s, &w, a, horse, heads);
        let out = cast.decide(a, &s, &w).unwrap();
        let sold = out.orders.iter().find(|o| o.side == Side::Sell).unwrap();
        assert_eq!(sold.qty > 0.0, offers, "markup {markup} on c_m");
        let par = |site: rustyecon_core::Site| -> Result<f64, rustyecon_agents::AgentError> {
            Ok(site.per_tick(&s.params(&w.registry), &w.clock)?)
        };
        let pr = |g: GoodId| -> Result<f64, rustyecon_agents::AgentError> {
            Ok(s.book().quote(home(&w), g).unwrap().price)
        };
        let res = maker_reservation(m, &par, &pr).unwrap();
        assert_eq!(res.withholds, !offers);
        assert_eq!(res.markup, pk / cm);
    }
}
