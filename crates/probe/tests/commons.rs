//! The open-commons instances on the engine (P2.3; docs/probe/COMMONS-RULES.md; the commons
//! frame, docs/probe/commons/SPEC.md §3.7, §3.8): the tapes are their generator's output, the
//! harness's points are unit 1e's and the frame's 50-digit solve's, the workers' rule is unit
//! 1e's supply, the point is a rest point of the roles at every target and tick length, the exit
//! switched off nests I1's run, the harness reads the rule the workers act on, and the battery and
//! families are the registered ones, run name for run name.
//!
//! One bar is relative and named here, as in `markets.rs`: `COIN`, 1e-12 of the money stock, for
//! the drift of the total coin over a run. Mode A's bars are PROBE-SPEC §4.6's, in
//! `probe::protocol`.

use certify::{tape_hash, Obs};
use oracle::ParcelEconomy;
use probe::markets::harness::{
    mirror_regime, observables, run, shock_distance, target_distance, Row, Target,
};
use probe::markets::instance::Instance;
use probe::markets::perturb::{battery, family, g12, Perturbation};
use probe::markets::setup::{genesis, tape_ron, Setup, ShareAt, Shock};
use probe::protocol::{RUN_TICKS, TOL_FLOOR};
use rustyecon_core::{CoreError, NodeId, Site};
use rustyecon_engine::prelude::{ActorId, ClassId, GoodId, Holder, SideTag, Sim, Tape};
use rustyecon_engine::rustyecon_agents::{
    workers_participation, AgentError, BasketWorkers, PlotRegime, Spec,
};

const C1: &str = include_str!("../../../tapes/markets-c1.ron");
const C2: &str = include_str!("../../../tapes/markets-c2.ron");
const I1: &str = include_str!("../../../tapes/markets-i1.ron");
const POINTS: &str = include_str!("../../../docs/probe/commons/registered/points.json");
const BATTERY: &str = include_str!("../../../docs/probe/commons/registered/battery_v3.jsonl");
const FAMILIES: &str = include_str!("../../../docs/probe/commons/registered/families_v2.jsonl");
const TIER3S: &str = include_str!("../../../docs/probe/results/commons/tier3s/tier3s.jsonl");
const COIN: f64 = 1e-12;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn setup(id: &str, tpy: u32) -> Setup {
    Setup::registered(id, tpy).expect("the commons instance")
}

/// Every registered target of an instance: the base, then each cost coefficient at its four
/// values, as (the frame's key suffix, the instance).
fn targets(base: &Instance) -> Vec<(String, Instance)> {
    let mut out = vec![("base".to_string(), base.clone())];
    for c in &base.coefs {
        let frame = if c.name == "commons" {
            "exit.To"
        } else {
            &c.name
        };
        for (v, f) in c.values.iter().zip(["x1.1", "x0.9", "x2", "x0.5"]) {
            let mut i = base.clone();
            i.set(&c.param, v.parse().unwrap()).unwrap();
            out.push((format!("{frame} {f}"), i));
        }
    }
    out
}

/// A number field of one point of the frame's points.json (indent 1, one field a line).
fn point_field(key: &str, field: &str) -> f64 {
    let start = POINTS
        .find(&format!("\"{key}\": {{"))
        .unwrap_or_else(|| panic!("no point {key}"));
    let block = &POINTS[start..];
    let end = block.find("\n }").unwrap_or(block.len());
    let block = &block[..end];
    let at = block
        .find(&format!("\n  \"{field}\": "))
        .unwrap_or_else(|| panic!("{key}: no {field}"));
    // Past `\n  "<field>": `, the value runs to the comma or the line's end.
    let rest = &block[at + field.len() + 7..];
    let text = rest.split([',', '\n']).next().unwrap().trim();
    text.parse::<f64>().unwrap_or(f64::NAN)
}

/// A string field of one point of the frame's points.json.
fn point_text(key: &str, field: &str) -> String {
    let start = POINTS.find(&format!("\"{key}\": {{")).unwrap();
    let block = &POINTS[start..];
    let at = block.find(&format!("\n  \"{field}\": \"")).unwrap();
    // Past `\n  "<field>": "`, the value runs to the next quote.
    let rest = &block[at + field.len() + 8..];
    rest.split('"').next().unwrap().to_string()
}

/// A registered JSON line's string field.
fn field<'a>(line: &'a str, key: &str) -> &'a str {
    let at = line
        .find(&format!("\"{key}\": \""))
        .unwrap_or_else(|| panic!("no {key} in {line}"));
    let rest = &line[at + key.len() + 5..];
    &rest[..rest.find('"').unwrap()]
}

fn number(line: &str, key: &str) -> String {
    let at = line.find(&format!("\"{key}\": ")).unwrap();
    let rest = &line[at + key.len() + 4..];
    rest.split([',', '}']).next().unwrap().trim().to_string()
}

/// The registered runs of a set at an instance, named as the engine names them: the frame's
/// `exit.To` is the commons' coefficient `commons`, and a desk's coin `coin.<d>` is
/// `coin.desk.<d>` (the registration's §3).
fn registered(text: &str, name: &str, set: &str) -> Vec<String> {
    text.lines()
        .filter(|l| field(l, "name") == name && field(l, "set") == set)
        .map(|l| engine_name(field(l, "run")))
        .collect()
}

fn engine_name(run: &str) -> String {
    let run = run.replace("exit.To=", "commons=");
    match run.strip_prefix("coin.") {
        Some(rest)
            if !rest.starts_with("workers")
                && !rest.starts_with("provider")
                && !rest.starts_with("desk.") =>
        {
            format!("coin.desk.{rest}")
        }
        _ => run,
    }
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[test]
fn commons_tapes_are_their_generators_output() {
    // tapes/markets-c1.ron and markets-c2.ron are `markets-tape --inst c1` and `--inst c2`'s
    // output for the registered setup at 52 ticks a year, byte for byte (the commons frame's
    // §3.7 test 8), so their genesis is unit 1e's point and nobody edits them.
    for (id, text) in [("c1", C1), ("c2", C2)] {
        let generated = tape_ron(&setup(id, 52)).expect("the generator runs");
        assert!(
            generated == text,
            "tapes/markets-{id}.ron differs from its generator: run `cargo run -p \
             rustyecon-probe --bin markets-tape -- --inst {id} tapes/markets-{id}.ron`"
        );
    }
}

#[test]
fn commons_points_are_the_registered_ones() {
    // The commons frame's §2.3 and §3.8 (decision 399): the harness's point, unit 1e's
    // `ParcelEconomy` at the instance's per-tick values, is the frame's 50-digit solve at every
    // one of the 26 registered targets, which reads no oracle code: x*, v, P_s, Y, the supply S,
    // the plot rent r_o and the rented plots T_p within 1e-13 (relative, or absolute at 0), and
    // the same regime. Every target is interior, funded, and certified but b.food × 0.5.
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-13 * a.abs().max(b.abs()).max(1.0);
    let mut n = 0;
    for (id, key) in [("c1", "C1"), ("c2", "C2")] {
        for (what, inst) in targets(&setup(id, 52).instance) {
            let k = format!("{key} {what}");
            let e = inst.point(52).unwrap();
            let c = e.commons.as_ref().expect("unit 1e's readouts");
            for (got, field) in [
                (e.x_star, "x"),
                (e.v, "v"),
                (e.p_s, "Ps"),
                (e.y, "Y"),
                (e.pool, "S"),
                (c.rented, "T_p"),
            ] {
                let want = point_field(&k, field);
                assert!(close(got, want), "{k}: {field} {got:e} against {want:e}");
            }
            let ro = point_field(&k, "ro");
            let ro = if point_text(&k, "regime") == "Enclosed" {
                1.0
            } else {
                ro
            };
            assert!(
                close(c.plot_rent, ro),
                "{k}: r_o {} against {ro}",
                c.plot_rent
            );
            assert_eq!(c.regime, point_text(&k, "regime"), "{k}");
            assert_eq!(c.land_market, "Scarce", "{k}");
            assert!(c.funded, "{k}");
            assert_eq!(c.certified, !what.starts_with("b.food x0.5"), "{k}");
            n += 1;
        }
    }
    assert_eq!(n, 26);
}

#[test]
fn commons_genesis_is_unit_1e() {
    // The commons frame's §3.8: genesis prices are the point's ratios with r = 1 in market
    // order; the workers' coin is r·T_p + (N·P_s + w·S − r·T_p)/share(spend), which is P2.1's
    // formula where T_p = 0 (C1's base, Crowded) and pays the plots' rent first where the point
    // is Enclosed (C1 with its commons at 12.15 a year); and the markets and actors are I1's.
    let s = setup("c1", 52);
    let g = genesis(&s).unwrap();
    let e = &g.point;
    assert_eq!(
        s.instance.markets(),
        [
            "labour",
            "land",
            "mach",
            "manufactures",
            "food",
            "care",
            "shelter"
        ]
    );
    let mut prices = vec![e.v, 1.0];
    prices.extend(e.type_price.iter().copied());
    prices.extend(e.cat_price.iter().copied());
    assert_eq!(g.prices, prices);
    let share = |v: f64| -rustyecon_core::num::expm1(-v / 52.0);
    let n = 208.0 / 52.0;
    let tau = n * e.p_s;
    assert_eq!(g.stationary[6], (tau + e.v * e.pool) / share(13.0));
    let mut t = s.clone();
    t.instance.set("inst.commons", 12.15).unwrap();
    let g = genesis(&t).unwrap();
    let e = &g.point;
    let tp = e.commons.as_ref().unwrap().rented;
    assert!(tp > 0.2, "the plots spill: T_p {tp}");
    let tau = n * e.p_s;
    assert_eq!(
        g.stationary[6],
        tp + (tau + e.v * e.pool - tp) / share(13.0)
    );
    // Each household's target baskets are its money accounts (O102): the provider's T/P_s − N
    // with the plots' rent, the workers' N + (w·S − T_p)/P_s; they sum to Y.
    assert!((e.provider_baskets - (10.0 / e.p_s - n)).abs() <= 1e-12);
    assert!((e.worker_baskets - (n + (e.v * e.pool - tp) / e.p_s)).abs() <= 1e-12);
    assert!((e.provider_baskets + e.worker_baskets - e.y).abs() <= 1e-12 * e.y);
    assert_eq!(
        s.instance.actors(),
        [
            "desk.manufactures",
            "desk.food",
            "desk.care",
            "desk.shelter",
            "desk.mach",
            "provider",
            "workers"
        ]
    );
}

/// The workers' resolved spec on a world.
fn the_workers(sim: &Sim) -> BasketWorkers {
    sim.world()
        .actors
        .iter()
        .find_map(|a| match &a.spec {
            Spec::BasketWorkers(p) if a.key.as_str() == "workers" => Some(p.clone()),
            _ => None,
        })
        .expect("the workers")
}

fn par_of<'a>(sim: &'a Sim) -> impl Fn(Site) -> Result<f64, AgentError> + 'a {
    move |s: Site| {
        let v = sim
            .param(s.param)
            .ok_or(AgentError::Core(CoreError::UnknownParam(s.param)))?;
        s.convert(&sim.world().clock, v).map_err(AgentError::Core)
    }
}

#[test]
fn commons_rule_is_unit_1e_supply() {
    // The commons frame's §3.5 step 2 and §3.7 test 3: at points on the line of C1 and C2 and
    // of every target, x from 0.05 to 0.95 and x*, the workers' rule at the point's prices
    // (w = v, r = 1, each category's price, the exit good's) offers unit 1e's supply S
    // (`ParcelEconomy::at_with(x, τ)`'s n_s) within 1e-12, in unit 1e's regime (the split
    // labelled Crowded, as 1e names a commons full at the rent where a plot stops paying). The
    // points span the Commons, Crowded and Enclosed regimes; a constructed h of 0.5, where no
    // plot pays at r, gives the split.
    let mut seen = std::collections::BTreeSet::new();
    let mut cases: Vec<(String, Instance)> = Vec::new();
    for id in ["c1", "c2"] {
        cases.extend(targets(&setup(id, 52).instance));
    }
    let mut split = setup("c1", 52).instance;
    split.set("inst.exit.plot", 0.5).unwrap();
    cases.push(("split".into(), split));
    for (what, inst) in cases {
        let mut s = setup(&inst.id, 52);
        s.instance = inst.clone();
        let world = sim(&tape_ron(&s).unwrap());
        let workers = the_workers(&world);
        let e = ParcelEconomy::new(inst.parcel_params(52).unwrap()).unwrap();
        let x_star = inst.point(52).unwrap().x_star;
        let mut xs: Vec<f64> = (1..20).map(|k| f64::from(k) * 0.05).collect();
        xs.push(x_star);
        for x in xs {
            let p = e.at_with(x, 0);
            if p.point.short.is_some() || !p.point.n_s.is_finite() {
                continue;
            }
            let price = |g: GoodId| -> Result<f64, AgentError> {
                let key = world.world().key_of(g).map(|k| k.to_string()).unwrap();
                Ok(match key.as_str() {
                    "labour" => p.point.v,
                    "land" => 1.0,
                    "mach" => p.point.type_prices[0],
                    other => {
                        let j = inst.categories.iter().position(|c| c.key == other).unwrap();
                        p.point.prices[j]
                    }
                })
            };
            let r = workers_participation(&workers, &par_of(&world), &price)
                .unwrap()
                .expect("an exit");
            let want = p.point.n_s;
            assert!(
                (r.hours - want).abs() <= 1e-12 * want.abs().max(1e-3),
                "{what} x {x}: hours {} against 1e's {want}",
                r.hours
            );
            assert_eq!(
                mirror_regime(r.regime).to_string(),
                format!("{:?}", p.exit_land),
                "{what} x {x}"
            );
            if r.regime == PlotRegime::Enclosed {
                assert!(
                    (r.plots - p.rented_plots).abs() <= 1e-12,
                    "{what} x {x}: T_p"
                );
            }
            seen.insert(format!("{:?}", r.regime));
        }
    }
    assert_eq!(
        seen.into_iter().collect::<Vec<_>>(),
        ["Commons", "Crowded", "Enclosed", "Split"]
    );
}

#[test]
fn commons_rest_at_the_oracle() {
    // The commons frame's §3.5 and §3.7 test 4: unit 1e's point is a rest point of the roles'
    // map. Three ticks from genesis at the oracle's f64 point leave every observable within
    // 1e-12 of the oracle in log, every market trading with both fills 1 to rounding, at C1,
    // C2 and every registered target, at 12, 52 and 365 ticks a year; and the harness's
    // readouts, through the workers' own rule, give the oracle's regime and its plot rent over
    // r within 1e-12 (0 with room, r when the plots spill).
    for tpy in [12, 52, 365] {
        for id in ["c1", "c2"] {
            let base = setup(id, tpy);
            for (what, inst) in targets(&base.instance) {
                let mut s = base.clone();
                s.instance = inst;
                let mut rows: Vec<Row> = Vec::new();
                let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
                assert_eq!(rec.hold_failure, None, "{id} {what} tpy {tpy}");
                let c = rec.genesis.point.commons.clone().unwrap();
                for r in &rows {
                    let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                    assert!(worst <= 1e-12, "{id} {what} tpy {tpy}: {worst:e}");
                    let p = r.plots.expect("the plots' readout");
                    assert_eq!(
                        mirror_regime(p.regime),
                        c.regime.as_str(),
                        "{id} {what} tpy {tpy}"
                    );
                    assert!(
                        (p.rent - c.plot_rent).abs() <= 1e-12,
                        "{id} {what} tpy {tpy}: r_o/r {} against {}",
                        p.rent,
                        c.plot_rent
                    );
                    assert!((p.rented - c.rented).abs() <= 1e-12);
                }
            }
        }
    }
}

#[test]
fn commons_hold_at_the_oracle_point() {
    // Mode A (PROBE-SPEC §4.6; the commons frame's §5.5 E2) at 52 ticks a year for 20,000
    // ticks: every observable within 1e-9 of the oracle in log (in fact 1e-12), every market
    // trading with every fill at least 1 − 1e-9, no produced good spoiling beyond 1e-9 of its
    // volume, the plots in the oracle's regime every tick, and the provider's coin never below
    // its genesis coin by more than rounding.
    for (id, regime) in [("c1", PlotRegime::Crowded), ("c2", PlotRegime::Commons)] {
        let rec = run(&setup(id, 52), "hold", RUN_TICKS, &mut |r| {
            assert_eq!(r.plots.map(|p| p.regime), Some(regime), "tick {}", r.tick);
        })
        .unwrap();
        assert_eq!(rec.stop, probe::harness::Stop::Ran);
        assert_eq!(rec.hold_failure, None);
        let worst = rec.stats.peak.0 * TOL_FLOOR;
        assert!(worst <= 1e-12, "{id}: the largest gap was {worst:e}");
        let c = rec.stats.commons.expect("the commons' readouts");
        assert_eq!(c.switches, 0);
        assert!(c.provider_coin_low >= 1.0 - 1e-12);
    }
}

/// I1's tape with the workers' exit switched off: s₀ = s̲ = 0, the plot h and the commons
/// `to` a year, on params that sort after every old one.
fn switched_off(h: f64, to: f64) -> String {
    let mut t = I1.to_string();
    let mut rep = |from: &str, to: &str| {
        assert_eq!(t.matches(from).count(), 1, "{from:?}");
        t = t.replacen(from, to, 1);
    };
    rep(
        "    params: [\n",
        &format!(
            "    params: [\n        (key: \"zz.exit.gross\", value: 0.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.exit.floor\", value: 0.0, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.exit.plot\", value: {h:?}, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"zz.commons\", value: {to:?}, unit: FlowPerYear, basis: Assumed(\"test\")),\n"
        ),
    );
    rep(
        "            spend: \"spend.workers\",\n",
        "            spend: \"spend.workers\",\n            exit: Some((good: \"food\", gross: \"zz.exit.gross\", floor: \"zz.exit.floor\", plot: \"zz.exit.plot\", commons: \"zz.commons\", land: \"land\")),\n",
    );
    t
}

/// Every number of one tick of the markets and actors, by key: each market's posted price, next
/// price, S, D, cleared volume and fills, each actor's coin, and the workers' share.
fn numbers(sim: &Sim, obs: &Obs, markets: &[&str], actors: &[&str]) -> Vec<u64> {
    let w = sim.world();
    let coin = w.id_of::<GoodId>("coin").unwrap();
    let mut v = Vec::new();
    for m in markets {
        let g = w.id_of::<GoodId>(m).unwrap();
        let l = obs.markets.iter().find(|l| l.good == g).unwrap();
        v.extend([
            l.price,
            l.next_price,
            l.supply,
            l.demand,
            l.cleared,
            l.buyer_fill,
            l.seller_fill,
        ]);
    }
    for a in actors {
        let id = w.id_of::<ActorId>(a).unwrap();
        v.push(sim.holding(Holder::Actor(id)).map_or(0.0, |i| i.get(coin)));
    }
    let workers = w.id_of::<ActorId>("workers").unwrap();
    if let Some(rustyecon_engine::rustyecon_agents::ActorState::Workers(s)) =
        sim.actor_state(workers)
    {
        v.push(s.share);
    }
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn exit_switched_off_is_the_dependence_form() {
    // R1 at the run level (the commons frame's §3.7 test 2): I1's tape with an exit block of
    // s₀ = s̲ = 0 (h 0 and 0.12, a commons of 0 and 30 a year) gives every market's prices, S,
    // D, cleared volume and fills, every actor's coin and the workers' share bit for bit as I1's
    // tape does, for 2,000 ticks, from genesis and through w × 2. An exit that moved the hours,
    // the baskets or the state by an ulp would part them.
    let markets = [
        "labour",
        "land",
        "mach",
        "manufactures",
        "food",
        "care",
        "shelter",
    ];
    let actors = [
        "desk.manufactures",
        "desk.food",
        "desk.care",
        "desk.shelter",
        "desk.mach",
        "provider",
        "workers",
    ];
    let bump = |t: &str| {
        let key = "(node: \"home\", good: \"labour\", price: ";
        let start = t.find(key).unwrap() + key.len();
        let end = start + t[start..].find(')').unwrap();
        let p: f64 = t[start..end].parse().unwrap();
        format!("{}{:?}{}", &t[..start], p * 2.0, &t[end..])
    };
    for (h, to) in [(0.0, 0.0), (0.12, 30.0), (0.0, 30.0), (0.12, 0.0)] {
        let off = switched_off(h, to);
        for (a_text, b_text) in [(I1.to_string(), off.clone()), (bump(I1), bump(&off))] {
            let (mut a, mut b) = (sim(&a_text), sim(&b_text));
            for t in 0..2000 {
                let ra = a.step().unwrap();
                let rb = b.step().unwrap();
                let (oa, ob) = (Obs::of(&ra, &a), Obs::of(&rb, &b));
                assert!(
                    numbers(&a, &oa, &markets, &actors) == numbers(&b, &ob, &markets, &actors),
                    "h {h}, T_o {to}: tick {t} differs"
                );
            }
        }
    }
}

#[test]
fn harness_reads_the_rule_the_workers_act_on() {
    // The commons frame's §3.7 test 7 (as L0.7's maker readout): on C1 displaced (the commons
    // at 12.15 a year from genesis, so the plots spill, and the wage × 0.8 and land × 2, so the
    // regime moves), the harness's readout of every tick is what the workers did on that tick
    // in an independent run of the same tape: the labour market's supply is the rule's hours;
    // the land the workers' class asked for is the readout's T_p (none where it is 0); a
    // Crowded tick's hours are N − T_o/h; and the shadow rent read is the plot rent at which the
    // rule's supply is its hours, n(p_g·s₀ − r_o·h) = hours within 1e-12.
    let name = "commons=12.15@genesis+p[labour]*0.8+p[land]*2";
    let mut s = setup("c1", 52);
    Perturbation::parse(name)
        .unwrap()
        .apply(&mut s, 600)
        .unwrap();
    let mut rows: Vec<Row> = Vec::new();
    run(&setup("c1", 52), name, 600, &mut |r| rows.push(r.clone())).unwrap();
    let mut world = sim(&tape_ron(&s).unwrap());
    let workers = the_workers(&world);
    let (labour, land, food, class, home) = {
        let w = world.world();
        (
            w.id_of::<GoodId>("labour").unwrap(),
            w.id_of::<GoodId>("land").unwrap(),
            w.id_of::<GoodId>("food").unwrap(),
            w.id_of::<ClassId>("workers").unwrap(),
            w.id_of::<NodeId>("home").unwrap(),
        )
    };
    let mut visited = std::collections::BTreeSet::new();
    for row in &rows {
        let p = row.plots.expect("the readout");
        let price = |g: GoodId| -> Result<f64, AgentError> {
            world
                .price(home, g)
                .ok_or(AgentError::Core(CoreError::Shape("no price".into())))
        };
        let rule = workers_participation(&workers, &par_of(&world), &price)
            .unwrap()
            .unwrap();
        let (wage, r, pg) = (
            price(labour).unwrap(),
            price(land).unwrap(),
            price(food).unwrap(),
        );
        let report = world.step().unwrap();
        let o = Obs::of(&report, &world);
        let supply = o.markets.iter().find(|l| l.good == labour).unwrap().supply;
        assert_eq!(supply, rule.hours, "tick {}: the labour supply", row.tick);
        assert_eq!(
            (p.regime, p.rented),
            (rule.regime, rule.plots),
            "tick {}",
            row.tick
        );
        let asked: f64 = report
            .rationing
            .iter()
            .filter(|l| l.good == land && l.class == class && l.side == SideTag::Buy)
            .map(|l| l.requested)
            .sum();
        assert_eq!(
            asked, p.rented,
            "tick {}: the plots' land asked for",
            row.tick
        );
        if p.regime == PlotRegime::Crowded {
            assert_eq!(rule.hours, rule.crowded, "tick {}", row.tick);
            let e = pg * 0.3 - p.rent * r * 0.135;
            let f = (rustyecon_core::num::ln1p((wage - e) / (rule.basket_price + e)) / 1.0)
                .clamp(0.0, 1.0);
            let back = rule.heads * f;
            assert!(
                (back - rule.hours).abs() <= 1e-12 * rule.hours,
                "tick {}: n(r_o) {back} against {}",
                row.tick,
                rule.hours
            );
        }
        visited.insert(format!("{:?}", p.regime));
    }
    assert!(
        visited.len() >= 2,
        "the run visits several regimes: {visited:?}"
    );
    assert!(visited.contains("Enclosed"));
}

#[test]
fn commons_battery_and_families_are_the_registered_ones() {
    // The commons frame's §5.3–§5.4 and its registration's §3: the battery (115 runs: 30, 42 and
    // 43 by tier, with the registered slack runs), the stocks, joint, basin and enclose families
    // at C1 and C2, the negative control's battery and Tier 3S are the registered runs, name for
    // name (the frame's `exit.To` is `commons`, a desk's `coin.<d>` is `coin.desk.<d>`), with
    // the battery's tiers and slack flags; the history family cycles land.mach and the commons.
    // Every run parses, applies and writes its tape.
    for (id, key) in [("c1", "C1"), ("c2", "C2")] {
        let s = setup(id, 52);
        let inst = &s.instance;
        let b = battery(inst, 52).unwrap();
        let t = |k: u8| b.iter().filter(|r| r.tier == k).count();
        assert_eq!((b.len(), t(1), t(2), t(3)), (115, 30, 42, 43), "{id}");
        let names: Vec<String> = b.iter().map(|r| r.name.clone()).collect();
        assert_eq!(
            sorted(names.clone()),
            sorted(registered(BATTERY, key, "battery"))
        );
        for l in BATTERY
            .lines()
            .filter(|l| field(l, "name") == key && field(l, "set") == "battery")
        {
            let n = engine_name(field(l, "run"));
            let r = b.iter().find(|r| r.name == n).unwrap();
            assert_eq!(r.tier.to_string(), number(l, "tier"), "{id} {n}");
            assert_eq!(r.slack.to_string(), number(l, "slack"), "{id} {n}");
        }
        let tier3: Vec<String> = b
            .iter()
            .filter(|r| r.tier == 3)
            .map(|r| r.name.clone())
            .collect();
        assert_eq!(sorted(tier3), sorted(registered(BATTERY, key, "tier3x10")));
        for (fam, set, text) in [
            ("stocks", "stocks", BATTERY),
            ("enclose", "enclose", BATTERY),
            ("joint2", "joint2", FAMILIES),
            ("joint4", "joint4", FAMILIES),
            ("basin", "basin", FAMILIES),
        ] {
            assert_eq!(
                sorted(family(inst, 52, fam).unwrap()),
                sorted(registered(text, key, set)),
                "{id} {fam}"
            );
        }
        assert_eq!(
            family(inst, 52, "tier3s").unwrap(),
            registered(TIER3S, key, "tier3s")
        );
        assert_eq!(
            family(inst, 52, "history").unwrap(),
            ["cycle(land.mach,1500,80)", "cycle(commons,1500,80)"]
        );
        for f in [
            "battery", "tier3s", "stocks", "joint2", "joint4", "basin", "history", "enclose",
        ] {
            for name in family(inst, 52, f).unwrap() {
                let mut u = s.clone();
                Perturbation::parse(&name)
                    .and_then(|p| p.apply(&mut u, RUN_TICKS))
                    .unwrap_or_else(|e| panic!("{id} {name}: {e}"));
                tape_ron(&u).unwrap_or_else(|e| panic!("{id} {name}: {e}"));
            }
        }
    }
    let negctl = include_str!("../../../docs/probe/commons/registered/negctl.jsonl");
    let b: Vec<String> = battery(&setup("c1n", 52).instance, 52)
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(sorted(b), sorted(registered(negctl, "C1/chi025", "negctl")));
    assert_eq!(
        observables(&setup("c1", 52).instance),
        [
            "v",
            "pi.mach",
            "pi.manufactures",
            "pi.food",
            "pi.care",
            "pi.shelter",
            "s.manufactures",
            "s.food",
            "s.care",
            "s.shelter",
            "vol.labour",
            "vol.land",
            "vol.mach",
            "vol.manufactures",
            "vol.food",
            "vol.care",
            "vol.shelter",
            "y.manufactures",
            "y.food",
            "y.care",
            "y.shelter",
            "y.mach"
        ]
    );
    // The basin's factors are Python's `%.12g` of 1.05^j.
    assert_eq!(g12(0.122_704_401_080_2), "0.12270440108");
    assert_eq!(g12(8.147_252_993_471_526), "8.14725299347");
    assert_eq!(g12(1.05), "1.05");
    assert_eq!(g12(1.5e-5), "1.5e-05");
}

#[test]
fn commons_grammar_applies_as_named() {
    // The commons' terms (the registration's §3): `commons=V` sets `inst.commons`;
    // `enclose=F` moves F·T_o of the commons to the enclosed land, both params at genesis or
    // both dated at L/4, as `cm.shocked` computes them; `joint` draws in the mirror's market
    // order (labour, land, the categories, the type); a share scaled past 1 is 1 at the commons
    // and refused at I1.
    let s = setup("c1", 52);
    let applied = |name: &str| {
        let mut u = s.clone();
        Perturbation::parse(name)
            .unwrap()
            .apply(&mut u, 4000)
            .unwrap();
        u
    };
    assert_eq!(
        applied("commons=48.6@genesis").at_genesis,
        [("inst.commons".to_string(), 48.6)]
    );
    let u = applied("enclose=0.5@genesis");
    assert_eq!(
        u.at_genesis,
        [
            ("inst.commons".to_string(), 24.3 - 0.5 * 24.3),
            ("inst.land".to_string(), 520.0 + 0.5 * 24.3)
        ]
    );
    let u = applied("enclose=1@dated");
    assert_eq!(
        u.shocks,
        [
            Shock {
                tick: 1000,
                param: "inst.commons".into(),
                value: 0.0
            },
            Shock {
                tick: 1000,
                param: "inst.land".into(),
                value: 520.0 + 24.3
            }
        ]
    );
    assert_eq!(
        Perturbation::parse("enclose=1@dated")
            .unwrap()
            .clock_start(4000),
        1000
    );
    // Enclosure at genesis gives a tape whose land is T + F·T_o from tick 0, genesis at C1's.
    let text = tape_ron(&applied("enclose=1@genesis")).unwrap();
    assert!(text.contains(&format!("(key: \"inst.land\", value: {:?},", 520.0 + 24.3)));
    assert!(text.contains("(key: \"inst.commons\", value: 0.0,"));
    // joint(F,SEED): SplitMix64 from SEED, u on [−1, 1], F^u for each price in the mirror's
    // order, then 2^u for each share.
    let mut state: u64 = 7;
    let mut signed = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        2.0 * ((z >> 11) as f64 / (1u64 << 53) as f64) - 1.0
    };
    // Markets: labour, land, mach, manufactures, food, care, shelter.
    let mut want = [1.0; 7];
    for m in [0, 1, 3, 4, 5, 6, 2] {
        want[m] *= rustyecon_core::num::pow(4.0, signed());
    }
    assert_eq!(applied("joint(4,7)").displace.price, want);
    // A share past 1.
    let u = applied("s[food]*8");
    assert_eq!(u.displace.share[1], ShareAt::Times(8.0));
    assert_eq!(genesis(&u).unwrap().shares[1], 1.0);
    let mut i1 = Setup::registered("i1", 52).unwrap();
    Perturbation::parse("s[food]*8")
        .unwrap()
        .apply(&mut i1, 4000)
        .unwrap();
    assert!(genesis(&i1).is_err());
    // Under enclosure the land market's target is the enclosed land in force, T + F·T_o a tick.
    let mut rows: Vec<Row> = Vec::new();
    run(&s, "enclose=1@genesis", 1, &mut |r| rows.push(r.clone())).unwrap();
    let land = observables(&s.instance)
        .iter()
        .position(|o| o == "vol.land")
        .unwrap();
    assert_eq!(rows[0].target[land], (520.0 + 24.3) / 52.0);
    // A cost shock's start distance at the commons is the largest |ln| over every observable
    // between the two targets, over the tolerance, as the mirror's.
    // Where it differs from P2.1's point-based distance (`shock_distance`), the mirror's is read.
    let mut differ = Vec::new();
    for id in ["c1", "c2"] {
        let s = setup(id, 52);
        let base = &s.instance;
        let b = Target::of(base, &base.point(52).unwrap(), 10.0);
        let mut runs: Vec<(String, Instance)> = Vec::new();
        for c in &base.coefs {
            for v in &c.values {
                let mut i = base.clone();
                i.set(&c.param, v.parse().unwrap()).unwrap();
                runs.push((format!("{}={v}@genesis", c.name), i));
            }
        }
        for f in [0.5, 1.0] {
            let mut i = base.clone();
            let to = base.exit.as_ref().unwrap().commons;
            i.set("inst.commons", to - f * to).unwrap();
            i.set("inst.land", base.land + f * to).unwrap();
            runs.push((format!("enclose={f}@genesis"), i));
        }
        for (name, i) in runs {
            let rec = run(&s, &name, 1, &mut |_| {}).unwrap();
            let p = i.point(52).unwrap();
            let a = Target::of(&i, &p, i.land / 52.0);
            assert_eq!(rec.d0, target_distance(&a, &b), "{id} {name}");
            if shock_distance(&p, &b.point) != rec.d0 {
                differ.push(format!("{id} {name}"));
            }
        }
    }
    assert!(!differ.is_empty(), "the two distances agree at every shock");
}

#[test]
fn commons_dated_shocks_load_and_fire() {
    // A dated shock of the commons (`commons=V@dated`), enclosure dated (`enclose=F@dated`) and
    // the commons' history (`cycle(commons,P,N)`) set `FlowPerYear` params (`inst.commons`,
    // `inst.land`) at their tick (P2.3.12): the schedule param carries that unit, so the tape
    // loads, and from the shock's tick on the run's target is the shocked instance's. Before it
    // the schedule params were `Dimensionless` and `Sim::new` refused every such tape.
    let s = setup("c1", 52);
    let (ticks, at) = (400u64, 100u64);
    let land = observables(&s.instance)
        .iter()
        .position(|o| o == "vol.land")
        .unwrap();
    for (name, shocked) in [
        ("commons=48.6@dated", vec![("inst.commons", 48.6)]),
        (
            "enclose=0.5@dated",
            vec![
                ("inst.commons", 24.3 - 0.5 * 24.3),
                ("inst.land", 520.0 + 0.5 * 24.3),
            ],
        ),
        ("cycle(commons,100,3)", vec![("inst.commons", 24.3 * 1.1)]),
    ] {
        let mut u = s.clone();
        let p = Perturbation::parse(name).unwrap();
        p.apply(&mut u, ticks).unwrap();
        let text = tape_ron(&u).unwrap();
        let line = text
            .lines()
            .find(|l| l.contains("(key: \"inst.commons.shock.1\", value:"))
            .unwrap_or_else(|| panic!("{name}: no schedule param"));
        assert!(
            line.contains("unit: FlowPerYear,"),
            "{name}: the schedule param is not FlowPerYear: {line}"
        );
        Sim::new(&Tape::from_ron(&text).expect("the tape parses")).expect("the tape loads");
        let mut rows: Vec<Row> = Vec::new();
        let rec = run(&s, name, ticks, &mut |r| rows.push(r.clone())).unwrap();
        assert_eq!(rec.stop, probe::harness::Stop::Ran, "{name}");
        let mut target = s.instance.clone();
        for (k, v) in &shocked {
            target.set(k, *v).unwrap();
        }
        let want = Target::of(&target, &target.point(52).unwrap(), target.land / 52.0);
        let after = rows.iter().find(|r| r.tick == at).unwrap();
        let before = rows.iter().find(|r| r.tick == at - 1).unwrap();
        assert_eq!(after.target, want.obs, "{name}: the target after the shock");
        assert_ne!(
            before.target, after.target,
            "{name}: the shock moved nothing"
        );
        assert_eq!(after.target[land], target.land / 52.0, "{name}");
    }
}

#[test]
fn commons_conserve_and_run_deterministically() {
    // R8: two runs of C1's tape give identical reports and hash streams, and the state moves.
    // R2: at rest and through a transient whose plots rent enclosed land (the commons at 12.15
    // a year, w × 2 and every desk's coin × 0.1 at genesis), every tick's ledger and the run's
    // close within their registered tolerances, the money stock stays put (the plots' rent
    // moves coin from the workers to the provider), and the world spoils what it does not sell.
    let (mut a, mut b) = (sim(C1), sim(C1));
    let genesis_hash = a.hash();
    for _ in 0..500 {
        assert_eq!(a.step().unwrap(), b.step().unwrap());
    }
    assert_eq!(a.hash(), b.hash());
    assert_ne!(a.hash(), genesis_hash);
    let mut setup = setup("c1", 52);
    let mut name = "commons=12.15@genesis+w*2".to_string();
    for d in setup.instance.desks() {
        name.push_str(&format!("+coin.desk.{d}*0.1"));
    }
    Perturbation::parse(&name)
        .unwrap()
        .apply(&mut setup, 2000)
        .unwrap();
    for text in [C1.to_string(), C2.to_string(), tape_ron(&setup).unwrap()] {
        let mut s = sim(&text);
        let coin = s.world().id_of::<GoodId>("coin").unwrap();
        let money = |s: &Sim| {
            s.world()
                .actors
                .iter()
                .map(|a| s.holding(Holder::Actor(a.id)).map_or(0.0, |i| i.get(coin)))
                .fold(0.0, |x, y| x + y)
        };
        let m0 = money(&s);
        for _ in 0..2000 {
            let r = s.step().expect("no breach stops the run");
            assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0);
            assert!((money(&s) - m0).abs() <= COIN * m0);
        }
    }
}

#[test]
fn commons_leave_the_other_markets_tapes_ids() {
    // R1 for the workers' `exit`: `world_id` hashes the resolved actors, so an `exit` left in
    // the resolved workers when absent would move every markets tape's `world_id`. P2.1's tapes
    // are checked in `wall.rs`; the wall's IW1 keeps the ids the pre-build binary recorded
    // (build-commons' base streams, 2026-09-30, at 7ea124d).
    let iw1 = include_str!("../../../tapes/markets-iw1.ron");
    let t = Tape::from_ron(iw1).unwrap();
    assert_eq!(tape_hash(&t), 0xec2c_4d1c_8f62_ea28);
    assert_eq!(sim(iw1).world().world_id, 0x7698_c09c_b7c5_4791);
}
