//! The seam and the scripted actor (docs/ENGINE.md §4 and §11, agents), on the gate world.
//!
//! Every comparison here is exact: the scripted actor's guarantees (no overdraw, no over-budget,
//! no shortfall) are statements about floating-point results, not approximations.

use rustyecon_agents::{
    ActorState, AgentDelta, Agents, Cast, Decision, RawSellQty, RawSpec, ScriptState, Spec,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, Clock, CoreError, GoodId, Holder, Ledger, LoadError,
    LoadErrorKind, NodeId, Phase, Provenance, RatePerYear, SimState, StateDelta, Tape, World,
};
use rustyecon_markets::{admit, Side};

const GATE: &str = include_str!("../../../tapes/gate.ron");

type W = World<Agents>;
type S = SimState<Agents>;
type Delta = StateDelta<Agents>;

fn load_text(text: &str) -> Result<(W, S), LoadError> {
    resolve(&Tape::<Agents>::from_ron(text)?)
}

fn load() -> (W, S) {
    load_text(GATE).expect("the gate tape loads")
}

fn load_tape(t: &Tape<Agents>) -> Result<(W, S), LoadError> {
    resolve(t)
}

fn gate_tape() -> Tape<Agents> {
    Tape::from_ron(GATE).expect("the gate tape parses")
}

/// Replace exactly one occurrence of `from` in the gate tape.
fn edit(from: &str, to: &str) -> String {
    assert_eq!(GATE.matches(from).count(), 1, "{from:?} must occur once");
    GATE.replacen(from, to, 1)
}

fn actor(w: &W, key: &str) -> ActorId {
    w.id_of::<ActorId>(key).expect("a gate actor")
}

fn good(w: &W, key: &str) -> GoodId {
    w.id_of::<GoodId>(key).expect("a gate good")
}

fn node(w: &W, key: &str) -> NodeId {
    w.id_of::<NodeId>(key).expect("a gate node")
}

fn held(s: &S, a: ActorId, g: GoodId) -> f64 {
    s.holding(Holder::Actor(a)).map_or(0.0, |inv| inv.get(g))
}

/// Apply deltas in a phase under a throwaway ledger.
fn apply_in(s: &mut S, w: &W, phase: Phase, ds: &[Delta]) -> Result<Vec<f64>, CoreError> {
    let mut l = Ledger::open(s, w)?;
    apply(s, w, phase, ds, &mut l)
}

fn mint(s: &mut S, w: &W, a: ActorId, g: GoodId, qty: f64) {
    let d = StateDelta::Mint {
        to: Holder::Actor(a),
        good: g,
        qty,
        prov: Provenance::Event,
    };
    apply_in(s, w, Phase::Events, &[d]).expect("the mint applies");
}

fn burn_all(s: &mut S, w: &W, a: ActorId, g: GoodId) {
    let d = StateDelta::Burn {
        from: Holder::Actor(a),
        good: g,
        amount: Amount::All,
        prov: Provenance::Event,
    };
    apply_in(s, w, Phase::Events, &[d]).expect("the burn applies");
}

fn age(s: &mut S, w: &W, a: ActorId) {
    let d = StateDelta::Age {
        holder: Holder::Actor(a),
    };
    apply_in(s, w, Phase::Upkeep, &[d]).expect("ageing applies");
}

/// A deterministic stream of awkward quantities: products of small decimal fractions and powers
/// of ten, the values whose float sums and products round.
struct Awkward(u64);

impl Awkward {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let a = (self.0 >> 33) % 997;
        let b = (self.0 >> 13) % 7;
        let scale = [1e-3, 0.1, 0.3, 1.0, 7.0, 1e3, 1e6][b as usize];
        (a as f64 + 0.1) * scale
    }
}

#[test]
fn leontief_never_overdraws() {
    // x = min(flow(capacity), min_k max_scale(held_k, a_k)): every burn a_k·x fits its holding,
    // whatever the lots, and x is the largest scale that does. Coefficients 0.3 and 0.7 make
    // every product round; the pops' bread is held in several lots of different lives.
    let text = edit(
        r#"inputs: [("grain", 2.0), ("fuel", 1.0)], outputs: [("bread", 3.0)], capacity: "mill.capacity""#,
        r#"inputs: [("grain", 0.3), ("fuel", 0.7)], outputs: [("bread", 3.0)], capacity: "mill.capacity""#,
    );
    for capacity in ["52.0", "1e12"] {
        let text = text.replace(
            r#"(key: "mill.capacity", value: 52.0"#,
            &format!(r#"(key: "mill.capacity", value: {capacity}"#),
        );
        let (w, genesis) = load_text(&text).expect("the variant loads");
        let cast = Cast::new(&w).expect("the cast builds");
        let (mill, workers) = (actor(&w, "mill"), actor(&w, "workers"));
        let (grain, fuel, bread) = (good(&w, "grain"), good(&w, "fuel"), good(&w, "bread"));
        let cap = w.clock.flow(rustyecon_core::FlowPerYear(
            genesis.param(w.id_of("mill.capacity").unwrap()).unwrap(),
        ));
        let mut q = Awkward(7);
        for _ in 0..400 {
            let mut s = genesis.clone();
            burn_all(&mut s, &w, mill, grain);
            burn_all(&mut s, &w, mill, fuel);
            mint(&mut s, &w, mill, grain, q.next());
            mint(&mut s, &w, mill, grain, q.next());
            mint(&mut s, &w, mill, fuel, q.next());
            // Three bread lots of three different lives at the workers.
            mint(&mut s, &w, workers, bread, q.next());
            age(&mut s, &w, workers);
            mint(&mut s, &w, workers, bread, q.next());
            age(&mut s, &w, workers);
            mint(&mut s, &w, workers, bread, q.next());
            assert!(s.holding(Holder::Actor(workers)).unwrap().lots(bread).len() >= 2);
            let (hg, hf, hb) = (
                held(&s, mill, grain),
                held(&s, mill, fuel),
                held(&s, workers, bread),
            );
            let mut ds = cast.produce(mill, &s, &w).expect("the mill produces");
            ds.extend(cast.produce(workers, &s, &w).expect("the workers eat"));
            // Every burn applies: none falls short, whatever the lots.
            let moved = apply_in(&mut s, &w, Phase::Production, &ds).expect("no burn falls short");
            // The mill's scale is the largest x with fl(a_k·x) within every holding, or the
            // capacity.
            let x = cap
                .min(num::max_scale(hg, 0.3).unwrap())
                .min(num::max_scale(hf, 0.7).unwrap());
            assert!(0.3 * x <= hg && 0.7 * x <= hf);
            assert!(x == cap || 0.3 * x.next_up() > hg || 0.7 * x.next_up() > hf);
            let expected = [
                (Holder::Actor(mill), fuel, 0.7 * x),
                (Holder::Actor(mill), grain, 0.3 * x),
                (Holder::Actor(mill), bread, 3.0 * x),
            ];
            for (h, g, q) in expected {
                let found = ds.iter().any(|d| match d {
                    StateDelta::Burn {
                        from,
                        good,
                        amount: Amount::Qty(b),
                        prov: Provenance::Production,
                    } => *from == h && *good == g && *b == q,
                    StateDelta::Mint {
                        to,
                        good,
                        qty,
                        prov: Provenance::Production,
                    } => *to == h && *good == g && *qty == q,
                    _ => false,
                });
                assert!(found, "{h} {g} {q} in {ds:?}");
            }
            // The workers' consumption burns at most what they held, across their lots.
            let eaten: f64 = ds
                .iter()
                .zip(&moved)
                .filter(|(d, _)| {
                    matches!(
                        d,
                        StateDelta::Burn {
                            prov: Provenance::Consumption,
                            ..
                        }
                    )
                })
                .map(|(_, m)| *m)
                .sum();
            assert!(eaten > 0.0 && eaten <= hb);
        }
    }
}

#[test]
fn scripted_actor_reads_only_its_view() {
    // R13: a decision depends on the actor's own holding and state, the posted book and the
    // params. Changing other actors' holdings and state, and the cleared volumes, changes no
    // actor's decision or production.
    let (w, s) = load();
    let cast = Cast::new(&w).unwrap();
    let everyone: Vec<ActorId> = cast.actors().collect();
    let decide = |s: &S| -> Vec<(Decision, Vec<Delta>)> {
        everyone
            .iter()
            .map(|&a| {
                (
                    cast.decide(a, s, &w).unwrap(),
                    cast.produce(a, s, &w).unwrap(),
                )
            })
            .collect()
    };
    for &me in &everyone {
        let base = decide(&s);
        let mut other = s.clone();
        for &a in everyone.iter().filter(|&&a| a != me) {
            for g in ["coin", "grain", "fuel", "bread"] {
                mint(&mut other, &w, a, good(&w, g), 17.25);
            }
            let wake = StateDelta::Actor(AgentDelta::SetActive {
                actor: a,
                active: true,
            });
            apply_in(&mut other, &w, Phase::Events, &[wake]).unwrap();
        }
        for (n, g) in w.markets().collect::<Vec<_>>() {
            let v = StateDelta::SetVolumes {
                node: n,
                good: g,
                supply: 123.0,
                demand: 456.0,
            };
            apply_in(&mut other, &w, Phase::Clearing, &[v]).unwrap();
        }
        let changed = decide(&other);
        let i = everyone.iter().position(|&a| a == me).unwrap();
        assert_eq!(
            changed[i], base[i],
            "{me}'s decision read something not its own"
        );
        // Its own holding does change it: the view is not empty.
        let mut own = s.clone();
        mint(&mut own, &w, me, good(&w, "coin"), 1000.0);
        mint(&mut own, &w, me, good(&w, "grain"), 1000.0);
        mint(&mut own, &w, me, good(&w, "fuel"), 1000.0);
        mint(&mut own, &w, me, good(&w, "bread"), 1000.0);
        if me != actor(&w, "oven") {
            assert_ne!(decide(&own)[i], base[i], "{me} ignored its own holding");
        }
    }
}

#[test]
fn extreme_spend_rate_never_overbudgets() {
    // At v·Δ = 40 per tick, share = −expm1(−40) rounds to exactly 1.0: the payout takes all the
    // cash, or the budgets do. Payouts still apply, and every budget still passes admission, for
    // awkward cash values.
    let clock = Clock {
        start: rustyecon_core::Date::new(1750, 1, 1).unwrap(),
        ticks_per_year: 52,
    };
    assert_eq!(clock.share(RatePerYear(2080.0)), 1.0);
    // A variant where only spending is extreme, and one where payouts take everything first.
    let mut t = gate_tape();
    for p in &mut t.params {
        if p.key.as_str().ends_with(".spend") {
            p.value = 2080.0;
        }
    }
    let mut all_payout = t.clone();
    for p in &mut all_payout.params {
        if p.key.as_str().ends_with(".payout") {
            p.value = 2080.0;
        }
    }
    for tape in [t, all_payout] {
        let (w, genesis) = load_tape(&tape).expect("the variant loads");
        let cast = Cast::new(&w).unwrap();
        let coin = good(&w, "coin");
        let mut q = Awkward(11);
        for _ in 0..300 {
            let mut s = genesis.clone();
            for a in cast.actors().collect::<Vec<_>>() {
                burn_all(&mut s, &w, a, coin);
                mint(&mut s, &w, a, coin, q.next());
                mint(&mut s, &w, a, coin, q.next());
            }
            let mut deltas = Vec::new();
            let mut orders = Vec::new();
            for a in cast.actors() {
                let d = cast.decide(a, &s, &w).unwrap();
                // The budgets are at most the cash the payouts left, walked by subtraction.
                let cash = held(&s, a, coin);
                let paid: f64 = d
                    .deltas
                    .iter()
                    .map(|d| match d {
                        StateDelta::Transfer {
                            amount: Amount::Qty(q),
                            ..
                        } => *q,
                        _ => 0.0,
                    })
                    .fold(0.0, |acc, q| acc + q);
                assert!(paid <= cash);
                let mut rem = cash;
                for d in &d.deltas {
                    if let StateDelta::Transfer {
                        amount: Amount::Qty(q),
                        ..
                    } = d
                    {
                        rem -= q;
                    }
                }
                for o in &d.orders {
                    if let Side::Buy { budget } = o.side {
                        assert!(budget <= rem, "{a} budgets {budget} with {rem} left");
                        rem -= budget;
                    }
                }
                deltas.extend(d.deltas);
                orders.extend(d.orders);
            }
            apply_in(&mut s, &w, Phase::Decisions, &deltas).expect("the payouts apply");
            admit(orders, &s, &w).expect("every budget passes admission");
        }
    }
}

#[test]
fn weights_must_sum_exactly_to_one() {
    // Buy and payout weights are folded left to right in canonical order, and the fold must be
    // exactly 1.0 (docs/ENGINE.md §2.6): no tolerance, and file order does not matter.
    let fold = |ws: &[f64]| ws.iter().fold(0.0, |acc, w| acc + w);
    // A payout whose weights add to 0.9999: refused, naming the path.
    let e = load_text(&edit(
        r#"to: [("workers", 0.75), ("pensioners", 0.25)]"#,
        r#"to: [("workers", 0.75), ("pensioners", 0.2499)]"#,
    ))
    .unwrap_err();
    assert_eq!(e.path, "actors[farm].spec.payout.to");
    assert!(matches!(e.kind, LoadErrorKind::WeightsNotOne { .. }));
    // The mill's four buy lines, canonically (town/fuel, town/grain, village/fuel,
    // village/grain). Find weights whose canonical fold is exactly 1 and whose reverse fold is
    // not, and weights for the opposite case.
    let mut candidates: Vec<[f64; 4]> = Vec::new();
    for set in [
        [0.1, 0.2, 0.3, 0.4],
        [0.1, 0.1, 0.1, 0.7],
        [0.1, 0.2, 0.2, 0.5],
        [0.05, 0.15, 0.3, 0.5],
    ] {
        for p in 0..24usize {
            let mut rest: Vec<f64> = set.to_vec();
            let mut ws = [0.0; 4];
            let mut code = p;
            for (i, slot) in ws.iter_mut().enumerate() {
                let k = code % (4 - i);
                code /= 4 - i;
                *slot = rest.remove(k);
            }
            candidates.push(ws);
        }
    }
    let rev = |ws: &[f64; 4]| {
        let mut r = *ws;
        r.reverse();
        r
    };
    let good_one = candidates
        .iter()
        .find(|ws| fold(&ws[..]) == 1.0 && fold(&rev(ws)) != 1.0)
        .expect("weights that add to 1 in canonical order only");
    let bad_one = candidates
        .iter()
        .find(|ws| fold(&ws[..]) != 1.0 && fold(&rev(ws)) == 1.0)
        .expect("weights that add to 1 in reverse order only");
    let with_weights = |ws: &[f64; 4]| {
        let mut t = gate_tape();
        let mill = t
            .actors
            .iter_mut()
            .find(|a| a.key.as_str() == "mill")
            .unwrap();
        let RawSpec::Scripted(s) = &mut mill.spec;
        // The file lists them in reverse canonical order; canonical order is by (node, good).
        s.buy
            .sort_by(|a, b| (&b.node, &b.good).cmp(&(&a.node, &a.good)));
        for (line, w) in s.buy.iter_mut().zip(rev(ws)) {
            line.weight = w;
        }
        t
    };
    load_tape(&with_weights(good_one)).expect("a canonical fold of exactly 1 loads");
    let e = load_tape(&with_weights(bad_one)).unwrap_err();
    assert_eq!(e.path, "actors[mill].spec.buy");
    assert_eq!(
        e.kind,
        LoadErrorKind::WeightsNotOne {
            sum: fold(&bad_one[..])
        }
    );
}

#[test]
fn dormant_actor_does_nothing() {
    // The oven is declared dormant: with cash, inputs and bread it decides and produces nothing,
    // until a SetActive wakes it.
    let (w, mut s) = load();
    let cast = Cast::new(&w).unwrap();
    let oven = actor(&w, "oven");
    for g in ["grain", "fuel", "bread"] {
        mint(&mut s, &w, oven, good(&w, g), 40.0);
    }
    assert_eq!(
        s.ext().get(&oven),
        Some(&ActorState::Scripted(ScriptState { active: false }))
    );
    assert_eq!(cast.decide(oven, &s, &w).unwrap(), Decision::default());
    assert!(cast.produce(oven, &s, &w).unwrap().is_empty());
    assert!(cast.upkeep(oven, &s, &w).unwrap().is_empty());
    let wake = StateDelta::Actor(AgentDelta::SetActive {
        actor: oven,
        active: true,
    });
    apply_in(&mut s, &w, Phase::Events, &[wake]).unwrap();
    let d = cast.decide(oven, &s, &w).unwrap();
    assert_eq!(d.orders.len(), 3, "two buys and a sell once awake");
    assert!(!d.deltas.is_empty(), "the payout");
    assert!(!cast.produce(oven, &s, &w).unwrap().is_empty());
}

#[test]
fn scripted_specs_are_checked_at_load() {
    // Every spec error is a load error that names its tape path (N2).
    let cases: [(&str, &str, &str); 7] = [
        (
            r#"(node: "village", good: "grain", qty: "mill.buy.grain.village", weight: 0.375)"#,
            r#"(node: "village", good: "salt", qty: "mill.buy.grain.village", weight: 0.375)"#,
            "actors[mill].spec.buy[village/salt].good",
        ),
        (
            r#"(node: "town", good: "grain", qty: "mill.buy.grain.town", weight: 0.125)"#,
            r#"(node: "town", good: "coin", qty: "mill.buy.grain.town", weight: 0.125)"#,
            "actors[mill].spec.buy[town/coin].good",
        ),
        (
            r#"(node: "town", good: "bread", qty: Flow("mill.sell.town"))"#,
            r#"(node: "town", good: "bread", qty: Flow("mill.buy.fuel.town.x"))"#,
            "actors[mill].spec.sell[town/bread].qty",
        ),
        (
            r#"outputs: [("grain", 1.0)], capacity: "farm.capacity""#,
            r#"outputs: [("grain", 0.0)], capacity: "farm.capacity""#,
            "actors[farm].spec.recipe.outputs[grain]",
        ),
        (
            r#"inputs: [("bread", 1.0)], outputs: [], capacity: "workers.eat""#,
            r#"inputs: [("bread", 1.0)], outputs: [("bread", 1.0)], capacity: "workers.eat""#,
            "actors[workers].spec.recipe.outputs[bread]",
        ),
        (
            r#"            spend: Some("workers.spend"),"#,
            r#"            spend: None,"#,
            "actors[workers].spec.spend",
        ),
        (
            r#"(node: "town", good: "bread", qty: "workers.buy.bread", weight: 1.0)"#,
            r#"(node: "town", good: "bread", qty: "farm.payout", weight: 1.0)"#,
            "actors[workers].spec.buy[town/bread].qty",
        ),
    ];
    for (from, to, path) in cases {
        let e = load_text(&edit(from, to)).unwrap_err();
        assert_eq!(e.path, path, "{e}");
    }
    // A missing Option field is an error, not a silent None.
    let e = load_text(&edit(
        "            payout: None,\n         ))),\n        (key: \"pensioners\"",
        "         ))),\n        (key: \"pensioners\"",
    ))
    .unwrap_err();
    assert!(matches!(e.kind, LoadErrorKind::Parse(_)), "{e}");
    // The checks that need the whole world: a buy at a node of another currency, a payout to
    // oneself.
    let t = edit(
        r#"(key: "village", currency: "coin")"#,
        r#"(key: "village", currency: "florin")"#,
    )
    .replace(
        r#"(key: "coin", life: Indefinite, price_rate: None),"#,
        r#"(key: "coin", life: Indefinite, price_rate: None), (key: "florin", life: Indefinite, price_rate: None),"#,
    );
    let (w, _) = load_text(&t).expect("the two-currency variant resolves");
    let e = Cast::new(&w).unwrap_err();
    assert_eq!(e.path, "actors[mill].spec.buy[village/fuel].node");
    let (w, _) = load_text(&edit(
        r#"payout: Some((rate: "mine.payout", to: [("workers", 1.0)]))"#,
        r#"payout: Some((rate: "mine.payout", to: [("mine", 1.0)]))"#,
    ))
    .expect("the self-payout variant resolves");
    assert_eq!(
        Cast::new(&w).unwrap_err().path,
        "actors[mine].spec.payout.to[mine]"
    );
}

#[test]
fn specs_round_trip_in_canonical_form() {
    // to_ron writes the spec's own lists in canonical order; the result parses to an equal
    // canonical tape with the same world_id, and reordering the spec's lists changes nothing.
    let t = gate_tape();
    let (w, _) = load_tape(&t).unwrap();
    let back: Tape<Agents> = Tape::from_ron(&t.to_ron()).expect("to_ron parses");
    assert_eq!(back, t.canonical());
    assert_eq!(load_tape(&back).unwrap().0.world_id, w.world_id);
    let mut shuffled = t.clone();
    for a in &mut shuffled.actors {
        let RawSpec::Scripted(s) = &mut a.spec;
        s.buy.reverse();
        s.sell.reverse();
        if let Some(r) = &mut s.recipe {
            r.inputs.reverse();
        }
        if let Some(p) = &mut s.payout {
            p.to.reverse();
        }
    }
    assert_eq!(load_tape(&shuffled).unwrap().0.world_id, w.world_id);
    // The resolved spec is in canonical order.
    let mill = w.actor(actor(&w, "mill")).unwrap();
    let Spec::Scripted(s) = &mill.spec;
    let keys: Vec<(NodeId, GoodId)> = s.buy.iter().map(|l| (l.node, l.good)).collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
    assert_eq!(keys[0], (node(&w, "town"), good(&w, "fuel")));
    assert!(matches!(
        gate_tape()
            .actors
            .iter()
            .find(|a| a.key.as_str() == "farm")
            .map(|a| {
                let RawSpec::Scripted(s) = &a.spec;
                s.sell[1].qty.clone()
            }),
        Some(RawSellQty::AllHeld)
    ));
}
