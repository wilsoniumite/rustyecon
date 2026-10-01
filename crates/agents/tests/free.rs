//! The pops on a commons market (P2.4; docs/probe/FREE-RULES.md; the free scan's §6.3,
//! docs/probe/free/SPEC.md; decision 419): the workers' exit's optional `market`, on CT2's tape
//! `tapes/markets-ct2.ron`, and the free step's admission at a price of 0 as the roles meet it on
//! IL1's `tapes/markets-il1.ron`.
//!
//! As in `commons.rs`, one bar is relative and named here: `CLOSE`, 1e-14 relative, for a value
//! the test recomputes in another order of operations than the rule's.

use rustyecon_agents::{
    pop_market, ActorState, AgentDelta, Agents, Cast, Decision, PopMarket, WorkersState,
};
use rustyecon_core::num;
use rustyecon_core::{
    apply, resolve, ActorId, Amount, CoreError, FlowPerYear, GoodId, Holder, Ledger, LoadError,
    NodeId, ParamId, Phase, Provenance, RatePerYear, SimState, StateDelta, Tape, World,
};
use rustyecon_markets::{admit, Side};

const CT2: &str = include_str!("../../../tapes/markets-ct2.ron");
const IL1: &str = include_str!("../../../tapes/markets-il1.ron");
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
        .find(|o| o.good == g && matches!(o.side, Side::Buy { .. }))
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

/// F(e), recomputed as the free scan's mirror writes it (`fm.pop_decide`'s `n_of` over N).
fn f_of(chi: f64, w: f64, ps: f64, e: f64) -> f64 {
    // Python's min(max(y, 0), 1) keeps a NaN, as `clamp` does.
    (num::ln1p((w - e) / (ps + e)) / chi).clamp(0.0, 1.0)
}

/// The pop's rule written again from FREE-SPEC §6.3 and `fm.pop_decide`, apart from the role's:
/// (hours, share, bid, plots).
#[allow(clippy::too_many_arguments)]
fn written_again(
    n: f64,
    chi: f64,
    w: f64,
    ps: f64,
    pg: f64,
    r: f64,
    ro: f64,
    (s0, sf, h, to): (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    let dlt = s0 - sf;
    let plot_at_r = r * h < pg * dlt;
    let rhat = if plot_at_r { r } else { pg * dlt / h };
    if ro < rhat {
        let f = f_of(chi, w, ps, num::fma(-ro, h, pg * s0));
        let hours = n * f;
        (hours, f, h * (n - hours), 0.0)
    } else {
        let f = if plot_at_r {
            f_of(chi, w, ps, num::fma(-rhat, h, pg * s0))
        } else {
            f_of(chi, w, ps, pg * sf)
        };
        let hours = n * f;
        let g = h * (n - hours);
        let tp = if plot_at_r && g > to { g - to } else { 0.0 };
        (hours, f, g.min(to), tp)
    }
}

#[test]
fn pops_on_a_commons_market_rule() {
    // FREE-SPEC §6.5 test 6, at the rule level on CT2's tape: over 3,000 random states (every
    // price over twelve decades, the commons' at 0 in a tenth of them, the land rent and r_o on
    // both sides of r̂, coins over eighteen decades), each pop offers the hours of §6.3's rule and
    // its whole share of the commons, bids for `bid` of the commons at budget
    // min(r_o·bid, (C − P) − the baskets' budget), rents T_p of land at r with P = min(r·T_p, C)
    // where the plots spill, spends share(spend)·(C − P) on baskets, and keeps F in its state; the
    // numbers are the rule written again from the spec, bit for bit; the orders pass admission;
    // and in produce it burns its land and min(held, bid) of the commons.
    let (w, genesis, cast) = load(CT2);
    let pops = [("workers.wa", "wa"), ("workers.wb", "wb")];
    let (coin, labour, land, food, commons) = (
        good(&w, "coin"),
        good(&w, "labour"),
        good(&w, "land"),
        good(&w, "food"),
        good(&w, "commons"),
    );
    let items = ["manufactures", "food", "care", "shelter"];
    let at = |s: &S, g: GoodId| s.price(home(&w), g).unwrap();
    let mut d = Draws(0x2026_0930_f4ee);
    let mut regimes = [0usize; 4];
    for i in 0..3000 {
        let mut s = genesis.clone();
        for it in items.iter().chain(&["labour", "land"]) {
            set_price(&mut s, &w, good(&w, it), d.decades(-6.0, 6.0));
        }
        let ro = if i % 10 == 0 {
            0.0
        } else if i % 3 == 0 {
            // Near the land rent, on either side of it.
            at(&s, land) * d.decades(-0.3, 0.3)
        } else {
            d.decades(-6.0, 6.0)
        };
        set_price(&mut s, &w, commons, ro);
        let (key, k) = pops[i % 2];
        let pop = actor(&w, key);
        let c = if i % 13 == 0 {
            0.0
        } else {
            d.decades(-9.0, 9.0)
        };
        set_holding(&mut s, &w, pop, coin, c);
        let dec = cast.decide(pop, &s, &w).unwrap();
        let mut ps = 0.0;
        for it in items {
            ps += value(&s, &w, &format!("inst.{it}.weight")) * at(&s, good(&w, it));
        }
        let n = per_tick(&s, &w, &format!("inst.{k}.workers"));
        let chi = value(&s, &w, &format!("inst.{k}.chi_max"));
        let exit = (
            value(&s, &w, &format!("inst.{k}.exit.gross")),
            value(&s, &w, &format!("inst.{k}.exit.floor")),
            value(&s, &w, &format!("inst.{k}.exit.plot")),
            per_tick(&s, &w, &format!("inst.{k}.commons")),
        );
        let (wage, r, pg) = (at(&s, labour), at(&s, land), at(&s, food));
        let (hours, f, bid, tp) = written_again(n, chi, wage, ps, pg, r, ro, exit);
        // The rule the harness reads is the role's, bit for bit.
        let p: PopMarket = pop_market((n, chi), (wage, ps, pg, r, ro), exit);
        assert_eq!(
            (p.hours, p.share, p.bid, p.plots),
            (hours, f, bid, tp),
            "draw {i}"
        );
        assert_eq!(sell_of(&dec, labour), Some(hours), "draw {i}: the hours");
        assert_eq!(
            sell_of(&dec, commons),
            Some(exit.3),
            "draw {i}: its share offered"
        );
        let spend = w.clock.share(RatePerYear(value(&s, &w, "spend.workers")));
        let paid = if tp > 0.0 { (r * tp).min(c) } else { 0.0 };
        let budget = spend * (c - paid);
        match buy_of(&dec, land) {
            Some((q, b)) => {
                assert!(tp > 0.0, "draw {i}: land bought with no plot to rent");
                assert_eq!(q, tp, "draw {i}: T_p");
                assert!(b <= r * tp && b <= c, "draw {i}: the rent's budget");
            }
            None => assert_eq!(tp, 0.0, "draw {i}: no land order"),
        }
        match buy_of(&dec, commons) {
            Some((q, b)) => {
                assert_eq!(q, bid, "draw {i}: the commons bid");
                // Its rent, paid from what the land and the baskets leave: the baskets leave
                // (1 − share(spend)) of C − P, so the chain never cuts it below its want.
                let want = (ro * bid).min((c - paid) - budget);
                assert_eq!(b, want, "draw {i}: the commons' budget");
            }
            None => assert_eq!(bid, 0.0, "draw {i}: no commons bid"),
        }
        let mut total = 0.0;
        for it in items {
            let g = good(&w, it);
            let (qi, bi) = buy_of(&dec, g).expect("a basket item");
            let z = value(&s, &w, &format!("inst.{it}.weight"));
            assert!(close(qi, z * (budget / ps)), "draw {i}, {it}: {qi}");
            total += bi;
        }
        assert!(
            total <= budget * (1.0 + CLOSE),
            "draw {i}: the baskets' budget"
        );
        assert!(dec.deltas.iter().any(|x| matches!(
            x,
            StateDelta::Actor(AgentDelta::SetState {
                state: ActorState::Workers(WorkersState { share }),
                ..
            }) if *share == f
        )));
        // The whole order set passes admission: nothing is over budget or over posted.
        let mut s2 = s.clone();
        for x in &dec.deltas {
            if matches!(x, StateDelta::Mint { .. }) {
                apply_in(&mut s2, &w, Phase::Decisions, std::slice::from_ref(x)).unwrap();
            }
        }
        admit(dec.orders.clone(), &s2, &w).unwrap_or_else(|e| panic!("draw {i}: {e}"));
        regimes[if ro == 0.0 {
            0
        } else if tp > 0.0 {
            3
        } else if ro < p.cap {
            1
        } else {
            2
        }] += 1;
        // Produce: the land it holds and min(held, bid) of the commons, both as `Consumption`.
        let (hl, hc) = (d.decades(-3.0, 1.0), d.decades(-3.0, 1.0));
        set_holding(&mut s, &w, pop, land, hl);
        set_holding(&mut s, &w, pop, commons, hc);
        let out = cast.produce(pop, &s, &w).unwrap();
        let burned = |g: GoodId| {
            out.iter()
                .filter_map(|x| match x {
                    StateDelta::Burn {
                        good,
                        amount: Amount::Qty(q),
                        prov: Provenance::Consumption,
                        ..
                    } if *good == g => Some(*q),
                    _ => None,
                })
                .sum::<f64>()
        };
        assert_eq!(burned(land), hl, "draw {i}: the plots' land");
        assert_eq!(burned(commons), hc.min(bid), "draw {i}: the commons bought");
    }
    // Every branch was met: free, below r̂, at or above r̂, and spilling onto enclosed land.
    assert!(regimes.iter().all(|&k| k > 20), "{regimes:?}");
}

#[test]
fn commons_market_is_checked_at_load() {
    // The exit's `market` (FREE-SPEC §6.3; the registration's §3): a traded good, not the pop's
    // labour, a basket item, the exit good or the plots' land (resolve); `Instant` and with a
    // free step (`Cast::new`); not with `pace`. Each refusal is a `LoadError` at its path.
    let refuse = |text: String, path: &str, why: &str| {
        let e = match load_text(&text) {
            Err(e) => e,
            Ok((w, _)) => Cast::new(&w).expect_err("refused"),
        };
        assert_eq!(e.path, path, "{e}");
        assert!(e.to_string().contains(why), "{e}");
    };
    let wa = "commons: \"inst.wa.commons\", land: \"land\", market: Some(\"commons\"))";
    for (to, why) in [
        ("food", "a good named twice"),
        ("labour", "a good named twice"),
        ("land", "a good named twice"),
        ("manufactures", "a good named twice"),
    ] {
        refuse(
            CT2.replacen(
                wa,
                &format!("commons: \"inst.wa.commons\", land: \"land\", market: Some(\"{to}\"))"),
                1,
            ),
            "actors[workers.wa].spec.exit.market",
            why,
        );
    }
    // Not Instant.
    let lasting = CT2.replacen(
        "(key: \"commons\", life: Instant,",
        "(key: \"commons\", life: Years(\"life.one_tick\"),",
        1,
    );
    refuse(lasting, "actors[workers.wa].spec.exit.market", "Instant");
    // No free step.
    let priced = CT2.replacen(
        ", free: Some((reference: \"labour\", scale: \"free.commons\"))",
        "",
        1,
    );
    let priced = priced.replacen(
        "        (key: \"free.commons\", value: 0.5, unit: Dimensionless, basis: Assumed(\"FREE-SPEC: the scan's window 0.3–1 at C2m, c 0.5\")),\n",
        "",
        1,
    );
    let priced = priced.replacen(
        "(node: \"home\", good: \"commons\", price: 0.0)",
        "(node: \"home\", good: \"commons\", price: 0.5)",
        1,
    );
    refuse(priced, "actors[workers.wa].spec.exit.market", "free step");
    // With a pace.
    let paced = CT2
        .replacen(
            wa,
            "commons: \"inst.wa.commons\", land: \"land\", pace: Some((adjust: \"spend.workers\", share: 0.1)), market: Some(\"commons\"))",
            1,
        );
    refuse(
        paced,
        "actors[workers.wa].spec.exit.market",
        "does not pace: the two were not scanned together (FREE-SPEC §6.3)",
    );
    // A pop paid by `more` has its support (decision 395): CT2's wb loads, and a pop no
    // transfer names is refused.
    let (w, _, _) = load(CT2);
    assert!(w.actors.iter().any(|a| a.key.as_str() == "workers.wb"));
    let unpaid = CT2.replacen(
        "            more: [(to: \"workers.wb\", heads: \"inst.wb.workers\")],\n",
        "",
        1,
    );
    refuse(unpaid, "actors[workers.wb].spec.exit", "support");
}

#[test]
fn idle_land_is_taken_for_nothing() {
    // The free step as the roles meet it at IL1 (FREE-SPEC §6.2: nothing new in the roles): at
    // r = 0 the workers' rule takes the plot branch (0·h < p_g·Δ) and rents T_p = h·(N − n(e₀))
    // on the idle land with a budget of 0, which admission fills in full, and the provider, with
    // no coin, pays no transfer and posts no basket.
    let (w, s, cast) = load(IL1);
    let (workers, provider) = (actor(&w, "workers"), actor(&w, "provider"));
    let (land, labour, food) = (good(&w, "land"), good(&w, "labour"), good(&w, "food"));
    assert_eq!(s.price(home(&w), land), Some(0.0));
    let dec = cast.decide(workers, &s, &w).unwrap();
    let (q, b) = buy_of(&dec, land).expect("the plots' land");
    assert_eq!(b, 0.0);
    let n = per_tick(&s, &w, "inst.workers");
    let chi = value(&s, &w, "inst.chi_max");
    let (s0, h) = (
        value(&s, &w, "inst.exit.gross"),
        value(&s, &w, "inst.exit.plot"),
    );
    let mut ps = 0.0;
    for it in ["manufactures", "food", "care", "shelter"] {
        ps +=
            value(&s, &w, &format!("inst.{it}.weight")) * s.price(home(&w), good(&w, it)).unwrap();
    }
    let wage = s.price(home(&w), labour).unwrap();
    let pg = s.price(home(&w), food).unwrap();
    let f = f_of(chi, wage, ps, num::fma(-0.0, h, pg * s0));
    assert_eq!(sell_of(&dec, labour), Some(n * f));
    assert_eq!(q, h * (n - n * f));
    let mut s2 = s.clone();
    for x in &dec.deltas {
        if matches!(x, StateDelta::Mint { .. }) {
            apply_in(&mut s2, &w, Phase::Decisions, std::slice::from_ref(x)).unwrap();
        }
    }
    let lines = admit(dec.orders.clone(), &s2, &w).unwrap();
    let line = lines
        .iter()
        .find(|l| l.order.good == land)
        .expect("admitted");
    assert_eq!(line.feasible, q, "a free good is taken in full for nothing");
    let pd = cast.decide(provider, &s, &w).unwrap();
    assert!(pd
        .deltas
        .iter()
        .all(|x| !matches!(x, StateDelta::Transfer { .. })));
    assert!(pd
        .orders
        .iter()
        .all(|o| matches!(o.side, Side::Sell) || o.qty == 0.0));
}
