//! The trap's remedy on the engine (P2.4; docs/probe/TRAP-RULES.md; the trap scan's §5.4–§5.5,
//! docs/probe/trap/SPEC.md): the paced instances C1P, C2P and C1PN rest at the oracle's point at
//! every target and tick length, the pace leads C1's trap run out, the tapes are their
//! generator's output while every old tape keeps its ids, the harness reads the paced share
//! beside the rule's, and the dial, the grammar and the families are the registered ones.

use certify::{tape_hash, Obs};
use probe::markets::harness::{csv_header, mirror_regime, run, Row};
use probe::markets::instance::Instance;
use probe::markets::perturb::{battery, family, tier3s, Perturbation, PACE_FAMILY};
use probe::markets::setup::{genesis, tape_ron, Dials, PaceAt, Setup, PACE_KEY};
use probe::perturb::Start;
use probe::protocol::TOL_FLOOR;
use rustyecon_core::{CoreError, NodeId, ParamId, RatePerYear, Site};
use rustyecon_engine::prelude::{ActorId, GoodId, Sim, Tape};
use rustyecon_engine::rustyecon_agents::{
    workers_participation, ActorState, AgentError, BasketWorkers, Spec,
};

const C1: &str = include_str!("../../../tapes/markets-c1.ron");
const C2: &str = include_str!("../../../tapes/markets-c2.ron");
const C1P: &str = include_str!("../../../tapes/markets-c1p.ron");
const C2P: &str = include_str!("../../../tapes/markets-c2p.ron");
const RUNS: &str = include_str!("../../../docs/probe/trap/registered/runs.jsonl");

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn setup(id: &str, tpy: u32) -> Setup {
    Setup::registered(id, tpy).expect("the paced instance")
}

/// Every registered target of an instance: the base, then each cost coefficient at its four
/// values.
fn targets(base: &Instance) -> Vec<(String, Instance)> {
    let mut out = vec![("base".to_string(), base.clone())];
    for c in &base.coefs {
        for v in &c.values {
            let mut i = base.clone();
            i.set(&c.param, v.parse().unwrap()).unwrap();
            out.push((format!("{}={v}", c.name), i));
        }
    }
    out
}

/// A registered JSON line's string field.
fn field<'a>(line: &'a str, key: &str) -> &'a str {
    let at = line
        .find(&format!("\"{key}\": \""))
        .unwrap_or_else(|| panic!("no {key} in {line}"));
    let rest = &line[at + key.len() + 5..];
    &rest[..rest.find('"').unwrap()]
}

/// The registered runs of a set at an instance, named as the engine names them (the
/// registration's §3): the mirror's `exit.To` is `commons`, a desk's `coin.<d>` is
/// `coin.desk.<d>`, and its histories `cycle(C)` are `cycle(C,1500,80)`.
fn registered(inst: &str, set: &str) -> Vec<String> {
    let mut v: Vec<String> = RUNS
        .lines()
        .filter(|l| field(l, "inst") == inst && field(l, "set") == set)
        .map(|l| engine_name(field(l, "run")))
        .collect();
    v.sort();
    v
}

fn engine_name(run: &str) -> String {
    if let Some(c) = run.strip_prefix("cycle(").and_then(|r| r.strip_suffix(')')) {
        let c = if c == "exit.To" { "commons" } else { c };
        return format!("cycle({c},1500,80)");
    }
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
fn paced_rest_is_the_oracles() {
    // The trap scan's §7 and §5.5 test 3: unit 1e's point is a rest point of the paced map. Three
    // ticks from genesis at the oracle's f64 point leave every observable within 1e-12 of the
    // oracle in log, every market trading with both fills 1 to rounding, at C1P, C2P and C1PN and
    // every registered target of C1P and C2P, at 12, 52 and 365 ticks a year; the paced share is
    // within 1e-15 of the rule's share at the tick's prices on every tick, and the harness reads
    // the oracle's regime and plot rent.
    for tpy in [12, 52, 365] {
        for id in ["c1p", "c2p", "c1pn"] {
            let base = setup(id, tpy);
            let cases = if id == "c1pn" {
                vec![("base".to_string(), base.instance.clone())]
            } else {
                targets(&base.instance)
            };
            for (what, inst) in cases {
                let mut s = base.clone();
                s.instance = inst;
                let mut rows: Vec<Row> = Vec::new();
                let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
                assert_eq!(rec.hold_failure, None, "{id} {what} tpy {tpy}");
                let (at, share) = rec.genesis.pace.expect("a paced genesis");
                assert_eq!(at, share, "{id} {what}: genesis at S/N");
                let c = rec.genesis.point.commons.clone().unwrap();
                assert_eq!(rows.len(), 3);
                for r in &rows {
                    let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                    assert!(worst <= 1e-12, "{id} {what} tpy {tpy}: {worst:e}");
                    let p = r.plots.expect("the plots' readout");
                    let f = r.participation[0];
                    assert!(
                        (f - p.target).abs() <= 1e-15,
                        "{id} {what} tpy {tpy}: the share {f} against the rule's {}",
                        p.target
                    );
                    assert!((f - at).abs() <= 1e-15, "{id} {what} tpy {tpy}");
                    assert_eq!(mirror_regime(p.regime), c.regime.as_str());
                    assert!((p.rent - c.plot_rent).abs() <= 1e-12);
                }
            }
        }
    }
}

#[test]
fn paced_workers_leave_the_trap() {
    // The trap scan's §1.1, §3.5 and §5.5 test 6: at every price rate × 0.9, C1's `p[mach]*0.5`
    // falls into the subsistence trap (ticks with no hours offered, then every price past the
    // runaway bound at tick 325, as the mirror and P2.3.16's recheck have it), and C1P's, the
    // same run with the workers' participation at a rate, converges: in tolerance from tick 723
    // (the mirror's), every tick with hours offered.
    let name = "p[mach]*0.5";
    let at_rate = |id: &str| {
        let mut s = setup(id, 52);
        s.dials.set("rate.*", 0.9).unwrap();
        s
    };
    let mut dead_hours = 0;
    let c1 = run(&at_rate("c1"), name, 3000, &mut |r| {
        dead_hours += u64::from(r.supply[0] <= 0.0);
    })
    .unwrap();
    assert_eq!(c1.summary.class, probe::harness::Class::Diverged);
    assert!(c1.summary.why.contains("at tick 325"), "{}", c1.summary.why);
    assert!(dead_hours > 100, "{dead_hours} ticks without hours");
    let mut least: f64 = 1.0;
    let c1p = run(&at_rate("c1p"), name, 3000, &mut |r| {
        least = least.min(r.participation[0]);
    })
    .unwrap();
    assert_eq!(c1p.summary.class, probe::harness::Class::Converged);
    assert_eq!(c1p.summary.in_tol_from, Some(723));
    let pace = c1p.stats.pace.expect("the pace's readouts");
    assert_eq!(pace.zero_hours, 0);
    assert!(least > 0.0 && pace.low > 0.0, "{least} {}", pace.low);
}

#[test]
fn commons_paced_tapes_are_their_generators_output() {
    // The trap scan's §5.4 and §5.5 test 7: tapes/markets-c1p.ron and markets-c2p.ron are
    // `markets-tape --inst c1p` and `--inst c2p`'s output for the registered setup at 52 ticks a
    // year, byte for byte, their genesis unit 1e's point with the share at S/N. R1: C1's and C2's
    // tapes keep the text, `tape_hash` and `world_id` the pre-build binary recorded
    // (build-trap's base streams, 2026-09-30, at e29b9c6), and the paced tapes differ from them
    // in their workers' block, the dial and their names alone.
    for (id, text) in [("c1p", C1P), ("c2p", C2P)] {
        let generated = tape_ron(&setup(id, 52)).expect("the generator runs");
        assert!(
            generated == text,
            "tapes/markets-{id}.ron differs from its generator: run `cargo run -p \
             rustyecon-probe --bin markets-tape -- --inst {id} tapes/markets-{id}.ron`"
        );
    }
    for (text, hash, id) in [
        (C1, 0x3122_821b_16c9_b76f_u64, 0xf63e_02fb_0360_98fb_u64),
        (C2, 0x181a_824f_9455_2daf, 0x57b6_8567_1f6b_adc7),
    ] {
        assert_eq!(tape_hash(&Tape::from_ron(text).unwrap()), hash);
        assert_eq!(sim(text).world().world_id, id);
    }
    for (paced, plain) in [(C1P, C1), (C2P, C2)] {
        let body = |t: &str| -> Vec<String> {
            t.lines()
                .filter(|l| !l.starts_with("//"))
                .map(|l| l.to_string())
                .collect()
        };
        let (a, b) = (body(paced), body(plain));
        assert_eq!(a.len(), b.len() + 1, "one line more: the dial");
        let differ: Vec<&String> = a.iter().filter(|l| !b.contains(l)).collect();
        assert_eq!(differ.len(), 4, "{differ:?}");
        assert!(differ.iter().any(|l| l.contains("name: \"markets-c")));
        assert!(differ
            .iter()
            .any(|l| l.contains(&format!("(key: \"{PACE_KEY}\""))));
        assert!(differ.iter().any(|l| l.contains("pace: Some((adjust: ")));
        assert!(differ.iter().any(|l| l.contains("basis: Approximate(")));
    }
}

#[test]
fn harness_reads_the_paced_share() {
    // The trap scan's §5.4 and §5.5 test 8: on C1P displaced (the machine's price × 0.5, the
    // commons at 12.15 a year so the plots spill, and the share at 0.3 of S/N), the harness's
    // readout of every tick is what the workers did on that tick in an independent run of the
    // same tape. Before the tick their state holds s₀ and the rule at posted prices gives F*;
    // after it the labour supply is N·s, s = s₀ + a·(F* − s₀), the state holds s, the row's
    // `part_workers` is s and its `part_target` is F*, and the land they asked for is the rule's
    // T_p. The pace's readouts are the largest |ln(s/F*)|, the least s/F* and no tick without
    // hours; the CSV carries `part_target` last, at a paced instance only. At 52 and at 12 ticks
    // a year: at 52 N is 4, a power of 2, so hours/N is the rule's share to the bit; at 12 it is
    // 208/12, and `part_target` is hours/N as the workers form it, not the rule's share.
    for tpy in [52, 12] {
        readout_at(tpy);
    }
    assert!(!csv_header(&setup("c1", 52).instance).contains("part_target"));
    assert!(setup("c1", 52).instance.exit.is_some());
    assert!(run(&setup("c1", 52), "hold", 2, &mut |_| {})
        .unwrap()
        .stats
        .pace
        .is_none());
}

/// `harness_reads_the_paced_share` at `tpy` ticks a year.
fn readout_at(tpy: u32) {
    let name = "p[mach]*0.5+commons=12.15@genesis+part.workers*0.3";
    let base = setup("c1p", tpy);
    let mut s = base.clone();
    Perturbation::parse(name)
        .unwrap()
        .apply(&mut s, 600)
        .unwrap();
    let mut rows: Vec<Row> = Vec::new();
    let rec = run(&base, name, 600, &mut |r| rows.push(r.clone())).unwrap();
    let mut world = sim(&tape_ron(&s).unwrap());
    let workers = the_workers(&world);
    let (labour, land, class, home, who) = {
        let w = world.world();
        (
            w.id_of::<GoodId>("labour").unwrap(),
            w.id_of::<GoodId>("land").unwrap(),
            w.id_of::<rustyecon_engine::prelude::ClassId>("workers")
                .unwrap(),
            w.id_of::<NodeId>("home").unwrap(),
            w.id_of::<ActorId>("workers").unwrap(),
        )
    };
    let rate = world
        .param(world.world().id_of::<ParamId>(PACE_KEY).unwrap())
        .unwrap();
    let a = world.world().clock.share(RatePerYear(rate));
    let state = |w: &Sim| match w.actor_state(who) {
        Some(ActorState::Workers(s)) => s.share,
        _ => panic!("the workers' state"),
    };
    let (mut gap, mut low, mut spilled, mut parted) = (0.0_f64, f64::INFINITY, 0, 0);
    for row in &rows {
        let price = |g: GoodId| -> Result<f64, AgentError> {
            world
                .price(home, g)
                .ok_or(AgentError::Core(CoreError::Shape("no price".into())))
        };
        let rule = workers_participation(&workers, &par_of(&world), &price)
            .unwrap()
            .unwrap();
        let s0 = state(&world);
        let target = rule.hours / rule.heads;
        parted += usize::from(target != rule.share);
        let want = s0 + a * (target - s0);
        let report = world.step().unwrap();
        let o = Obs::of(&report, &world);
        let supply = o.markets.iter().find(|l| l.good == labour).unwrap().supply;
        assert_eq!(supply, rule.heads * want, "tick {}: the hours", row.tick);
        assert_eq!(state(&world), want, "tick {}: the state", row.tick);
        assert_eq!(
            row.participation[0], want,
            "tick {}: part_workers",
            row.tick
        );
        let p = row.plots.expect("the readout");
        assert_eq!(p.target, target, "tick {}: part_target", row.tick);
        let asked: f64 = report
            .rationing
            .iter()
            .filter(|l| {
                l.good == land
                    && l.class == class
                    && l.side == rustyecon_engine::prelude::SideTag::Buy
            })
            .map(|l| l.requested)
            .sum();
        assert_eq!(asked, rule.plots, "tick {}: the rule's plots", row.tick);
        spilled += usize::from(rule.plots > 0.0);
        gap = gap.max(rustyecon_core::num::ln(want / target).abs());
        low = low.min(want / target);
        let csv = row.csv(&s.instance);
        assert_eq!(
            csv.rsplit(',').next().unwrap(),
            format!("{target:?}"),
            "tick {}: the CSV's part_target",
            row.tick
        );
    }
    assert!(spilled > 100, "{spilled} ticks with plots rented");
    let pace = rec.stats.pace.expect("the pace's readouts");
    assert_eq!(pace.gap_max, gap);
    assert_eq!(pace.low, low);
    assert_eq!(pace.zero_hours, 0);
    assert!(gap > 0.5, "the share lags: {gap}");
    assert!(csv_header(&s.instance).ends_with(",part_target"));
    // At 12 ticks a year the rule's share and hours/N part in some ticks: the readout is the
    // latter, the workers' own F*.
    if tpy == 12 {
        assert!(parted > 0, "hours/N is the rule's share in every tick");
    }
}

#[test]
fn paced_grammar_dials_and_families_are_the_registered_ones() {
    // The trap registration's §3 and the scan's §5.4: the dial `adjust.participation.workers`,
    // 1.3 a year, joins C2m at a paced instance only, after the techniques', and `adjust.*`
    // scales it; `part.workers*F` sets the genesis share to min(F·S/N, 1) and `part.workers=V`
    // to V, both refused at an instance without a pace; a `part.workers` run starts as a stock,
    // at |ln(share/(S/N))| over the tolerance, or at its first year's largest D̂ with
    // `--first-year`. The families at C1P and C2P are the registered runs, name for name (the
    // pace family in SPEC §8 E4's order), C1PN's battery is the registered stress control's, and
    // every run parses, applies and writes its tape.
    let s = setup("c1p", 52);
    let d = &s.dials;
    assert_eq!(d.get(PACE_KEY), Ok(1.3));
    let at = d.values.iter().position(|x| x.key == PACE_KEY).unwrap();
    assert_eq!(d.values[at].unit, "RatePerYear");
    assert!(d.values[at - 1].key.starts_with("adjust.technique."));
    assert!(d.values[at + 1].key.starts_with("buffer."));
    assert!(Dials::c2m(&setup("c1", 52).instance).get(PACE_KEY).is_err());
    let mut e = s.dials.clone();
    e.set("adjust.*", 0.75).unwrap();
    assert_eq!(e.get(PACE_KEY), Ok(1.3 * 0.75));
    assert_eq!(e.get("adjust.technique.food"), Ok(2.6 * 0.75));
    e.set(PACE_KEY, 2.6).unwrap();
    assert_eq!(e.get(PACE_KEY), Ok(2.6));
    // The grammar.
    let g0 = genesis(&s).unwrap();
    let (sn, share) = g0.pace.unwrap();
    assert_eq!(sn, share);
    assert_eq!(sn, g0.point.pool / 4.0);
    let applied = |id: &str, name: &str| {
        let mut u = setup(id, 52);
        Perturbation::parse(name)
            .and_then(|p| p.apply(&mut u, 4000))
            .map(|_| u)
    };
    for (name, want) in [
        ("part.workers*0.5", sn * 0.5),
        ("part.workers*2", sn * 2.0),
        ("part.workers*20", 1.0),
        ("part.workers=0", 0.0),
        ("part.workers=1", 1.0),
    ] {
        let u = applied("c1p", name).unwrap();
        assert_eq!(genesis(&u).unwrap().pace, Some((sn, want)), "{name}");
        assert!(tape_ron(&u)
            .unwrap()
            .contains(&format!("share: {want:?}))")));
        assert_eq!(
            Perturbation::parse(name).unwrap().start(),
            Start::Stocks,
            "{name}"
        );
    }
    assert_eq!(
        applied("c1p", "part.workers*0.5").unwrap().displace.pace,
        PaceAt::Times(0.5)
    );
    assert!(genesis(&applied("c1p", "part.workers=1.5").unwrap()).is_err());
    for id in ["c1", "c2", "i1"] {
        assert!(applied(id, "part.workers*0.5").is_err(), "{id}");
    }
    let rec = run(&s, "part.workers*0.5", 1, &mut |_| {}).unwrap();
    assert_eq!(rec.d0, rustyecon_core::num::ln(0.5).abs() / TOL_FLOOR);
    let mut fy = s.clone();
    fy.first_year_d0 = true;
    let mut worst: f64 = 0.0;
    let rec = run(&fy, "part.workers*0.5", 52, &mut |r| {
        worst = worst.max(r.dhat)
    })
    .unwrap();
    assert_eq!(rec.d0, worst);
    assert!(worst > 1.0, "{worst}");
    // The pace's readouts count a tick without hours: at genesis no share and a rule that
    // offers none (food ten times dearer, so the exit is worth more than the wage), s = 0.
    let rec = run(&s, "part.workers=0+p[food]*10", 1, &mut |r| {
        assert_eq!(r.supply[0], 0.0);
        assert_eq!(r.plots.map(|p| p.target), Some(0.0));
    })
    .unwrap();
    let pace = rec.stats.pace.unwrap();
    assert_eq!((pace.zero_hours, pace.low, pace.gap_max), (1, 1.0, 0.0));
    // The families.
    for (id, key) in [("c1p", "C1P"), ("c2p", "C2P")] {
        let inst = setup(id, 52).instance;
        assert_eq!(
            family(&inst, 52, "pace").unwrap(),
            [
                "part.workers*0.1",
                "part.workers*0.5",
                "part.workers*2",
                "part.workers=0",
                "part.workers=1"
            ]
        );
        assert_eq!(family(&inst, 52, "pace").unwrap(), PACE_FAMILY);
        assert_eq!(
            sorted(family(&inst, 52, "pace").unwrap()),
            registered(key, "pace")
        );
        let b: Vec<String> = battery(&inst, 52)
            .unwrap()
            .into_iter()
            .map(|r| r.name)
            .collect();
        assert_eq!(sorted(b), registered(key, "battery"), "{id}");
        for fam in ["stocks", "joint2", "joint4", "basin", "history", "enclose"] {
            assert_eq!(
                sorted(family(&inst, 52, fam).unwrap()),
                registered(key, fam),
                "{id} {fam}"
            );
        }
        let t3s: Vec<String> = RUNS
            .lines()
            .filter(|l| field(l, "inst") == key && field(l, "set") == "tier3s")
            .map(|l| field(l, "engine_name").to_string())
            .collect();
        assert_eq!(sorted(tier3s(&inst)), sorted(t3s), "{id}");
        for f in [
            "battery", "tier3s", "stocks", "pace", "joint2", "basin", "history", "enclose",
        ] {
            for name in family(&inst, 52, f).unwrap() {
                let mut u = setup(id, 52);
                Perturbation::parse(&name)
                    .and_then(|p| p.apply(&mut u, 4000))
                    .unwrap_or_else(|e| panic!("{id} {name}: {e}"));
                tape_ron(&u).unwrap_or_else(|e| panic!("{id} {name}: {e}"));
            }
        }
    }
    let b: Vec<String> = battery(&setup("c1pn", 52).instance, 52)
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(sorted(b), registered("C1PN", "negctl"));
    assert!(family(&setup("c1", 52).instance, 52, "pace").is_err());
    assert_eq!(Instance::PACED_IDS, ["c1p", "c2p", "c1pn"]);
    for (id, of) in Instance::PACED_IDS.into_iter().zip(["c1", "c2", "c1n"]) {
        let i = Instance::named(id).unwrap();
        assert!(i.paced_exit(), "{id}");
        // The pace moves no point: the paced instance's oracle is its unpaced twin's.
        let twin = Instance::named(of).unwrap();
        assert_eq!(i.point(52).unwrap(), twin.point(52).unwrap(), "{id}");
        assert!(!twin.paced_exit());
    }
}

#[test]
fn markets_runs_the_pace_family_and_reports_the_pace() {
    // The command line (P2.4; the trap registration's §3): `markets family pace` runs SPEC §8
    // E4's five runs with their first year's largest D̂ as their start, as `family tier3s` does,
    // and `stats.tsv` carries the pace's readouts, `pace.gap_max`, `pace.low` and
    // `pace.zero_hours`, at a paced instance and at no other.
    let dir = std::env::temp_dir().join(format!("trap-bin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let markets = env!("CARGO_BIN_EXE_markets");
    let out = std::process::Command::new(markets)
        .args(["family", "pace", "--inst", "c1p", "--ticks", "60", "--csv"])
        .arg(dir.join("paced"))
        .output()
        .expect("markets runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let summary = std::fs::read_to_string(dir.join("paced/summary.tsv")).unwrap();
    let lines: Vec<&str> = summary.lines().skip(1).collect();
    assert_eq!(lines.len(), 5);
    let mut fy = setup("c1p", 52);
    fy.first_year_d0 = true;
    for (line, name) in lines.iter().zip(PACE_FAMILY) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols[0], name);
        let rec = run(&fy, name, 60, &mut |_| {}).unwrap();
        assert_eq!(
            cols[2],
            format!("{:.6e}", rec.d0),
            "{name}: the first-year start"
        );
    }
    let stats = std::fs::read_to_string(dir.join("paced/stats.tsv")).unwrap();
    for stat in ["pace.gap_max", "pace.low", "pace.zero_hours"] {
        assert_eq!(
            stats
                .lines()
                .filter(|l| l.split('\t').nth(1) == Some(stat))
                .count(),
            5,
            "{stat}"
        );
    }
    let out = std::process::Command::new(markets)
        .args(["run", "hold", "--inst", "c1", "--ticks", "2", "--csv"])
        .arg(dir.join("plain"))
        .output()
        .expect("markets runs");
    assert!(out.status.success());
    let stats = std::fs::read_to_string(dir.join("plain/stats.tsv")).unwrap();
    assert!(stats.contains("commons.switches") && !stats.contains("pace."));
    let _ = std::fs::remove_dir_all(&dir);
}
