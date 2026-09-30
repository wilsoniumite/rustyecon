//! The demo's second pass (docs/demo/WORLD-V2.md; D2.2, 2026-09-30): stage v2a.1 of the goods
//! chain on every county, compiled from v1's tables with `--stage v2a1` into
//! `tapes/demo-gb-v2.ron`.
//!
//! - `demo_v2_tape_is_its_compilers_output`: the committed tape is the compiler's output.
//! - `v2_the_tables_check_every_county_at_every_step`: every county solved by 1g at genesis and
//!   after every step date, meeting v1's 1a point (R1b), funded, its chain within the bounds;
//!   33,332 events on v1's 25,480 county dates; every basis illustrative; the nodes the atlas's.
//! - `c2g_is_the_probes`: the stage's dials are P2.2a's registered C2g on F5, key by key.
//! - `rule_a_is_the_probes`: worldgen's rule A, chain and point are the probe harness's bit for
//!   bit on F5, F6 and ten counties (decision 329; O32 for rule A).
//! - `v2_genesis_is_the_horses_rule_at_each_county`: each county's genesis is
//!   `probe::horses::setup::genesis` on its instance, divided by p/r, to 1e-14.
//! - `the_stage_compiler_refuses_bad_tables`: dials off C2g, a machine type out of range, rule
//!   B, the wrong assignment, an unknown stage, and an unfunded date, each with its path.
//! - `v2_rests_at_every_county_oracle_point`: with no history, every county's 18 observables
//!   stay at its oracle point, within 1e-12 for three ticks and 1e-9 for two years (mode A
//!   over a short window), every market trading and the ledger closed every tick.
//! - `demo_v2_runs_deterministically`: two runs give one hash stream.
//! - `demo_v2_pin` (ignored, run by name in scripts/gate.sh): the tape through the first tick
//!   of 1901, every event fired and the ledger closed every tick, its final hash and hash stream.
//! - `lens_v2_domains_hold_the_oracle_range` (D2.3): every lens's domain holds the oracle's range
//!   at every county date; the engine's run is held by the GUI's
//!   `lens_v2_domains_hold_the_engines_long_run` (D2.5).
//! - `v2_flow_path_is_v1` (D2.3b): the stage on the flow path runs v1's tables and history with
//!   v1's prices, volumes and coins bit for bit (R1).
//! - `battery_instances_are_the_harness_instances` (D2.4): the battery's county table, each
//!   county's row in force on 1 January of the six years, is the plans' bit for bit; the
//!   harness's point there is the compiler's chain point; its battery drops b × 2 exactly where
//!   the registered targets are unfunded, 51,260 runs.
//! - `demo_v2_runs_to_1901` (D2.4; ignored, run by name in scripts/gate.sh): the long run scored
//!   against each county's moving oracle point as the registration's mirror scores it
//!   (WORLD-V2 §11.4), its table written for the scorer; no dead tick in any county.

use oracle::{ChainEconomy, Regime};
use probe::horses::instance::{Config, County as ProbeCounty, Instance as Horse, Keys};
use probe::horses::setup::{dials, genesis as probe_genesis, Setup};
use rustyecon_core::{fnv1a_64, num, Basis, Date, FlowPerYear};
use rustyecon_engine::prelude::{ActorId, ActorState, GoodId, NodeId, Sim, Tape};
use rustyecon_engine::registry;
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::chain::{self, ChainPoint, MachineType};
use rustyecon_worldgen::compile::{clock, Compiled, MAX_YEAR};
use rustyecon_worldgen::lens::{value, ChainReadings, Measure, Readings, WindowTick};
use rustyecon_worldgen::stage::{
    chain_params, compile_stage_parsed, parse_stage, stage_genesis, stage_instance, Stage,
    StageParam, C2G, MARKETS,
};
use rustyecon_worldgen::tables::{Instance, Param};
use rustyecon_worldgen::{compile_stage, StageTables, Tables};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const WORLD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/demo-gb");
const TAPE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes/demo-gb-v2.ron");
const MARK: &str = "illustrative demo v2a.1, 2026-09-30: ";

/// The final state hash of the run through 1901-01-01's tick (state tick 7,852), and FNV-1a 64
/// over every tick's hash as 8 little-endian bytes, recorded at D2.2 on WSL and Windows.
const FINAL_HASH: u64 = 0x45b7_c120_1f8a_e633;
const STREAM_HASH: u64 = 0xacb3_ca2b_ee63_b3bf;

fn read(name: &str) -> String {
    let path = format!("{WORLD}/{name}");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn tables() -> Tables {
    Tables {
        world: read("world.csv"),
        counties: read("counties.csv"),
        regions: read("regions.csv"),
        history: read("history.csv"),
        lenses: read("lenses.csv"),
    }
}

fn stage_tables() -> StageTables {
    StageTables {
        key: "v2a1".to_string(),
        machine_types: read("machine_types.csv"),
        stage: read("stage-v2a1.csv"),
        lenses: read("lenses-v2a1.csv"),
    }
}

fn atlas() -> Atlas {
    Atlas::gb().unwrap_or_else(|e| panic!("{e}"))
}

fn stage() -> Stage {
    parse_stage(&stage_tables()).unwrap_or_else(|e| panic!("{e}"))
}

fn compiled() -> &'static Compiled {
    static C: OnceLock<Compiled> = OnceLock::new();
    C.get_or_init(|| {
        compile_stage(&tables(), &stage_tables(), &atlas()).unwrap_or_else(|e| panic!("{e}"))
    })
}

fn tape_text() -> String {
    std::fs::read_to_string(TAPE).unwrap_or_else(|e| panic!("{TAPE}: {e}"))
}

fn tape() -> &'static Tape {
    static T: OnceLock<Tape> = OnceLock::new();
    T.get_or_init(|| Tape::from_ron(&tape_text()).unwrap_or_else(|e| panic!("{e}")))
}

#[test]
fn demo_v2_tape_is_its_compilers_output() {
    // `rustyecon worldgen worlds/demo-gb --stage v2a1 --out tapes/demo-gb-v2.ron` rewrites it.
    let committed = tape_text();
    let fresh = &compiled().tape;
    if committed != *fresh {
        let line = committed
            .lines()
            .zip(fresh.lines())
            .position(|(a, b)| a != b)
            .map_or(committed.lines().count().min(fresh.lines().count()), |i| i);
        panic!(
            "tapes/demo-gb-v2.ron differs from the compiler's output from line {}: rerun \
             `rustyecon worldgen worlds/demo-gb --stage v2a1 --out tapes/demo-gb-v2.ron`",
            line + 1
        );
    }
    assert!(!committed.contains('\r'));
}

#[test]
fn v2_the_tables_check_every_county_at_every_step() {
    // WORLD-V2 §3, §4, §9.2: every county at genesis and after every step date is solved by 1g,
    // Interior and funded, meeting v1's 1a point within 1e-13 (R1b), and the chain's own moves
    // are bounded as v1's oracle's are. The history is v1's, mapped: a b step is two events.
    let c = compiled();
    let s = &c.summary;
    println!("{s:#?}");
    assert_eq!(s.counties, 93);
    assert_eq!(s.step_dates, 25_480);
    assert_eq!(s.events, 33_332);
    assert_eq!(tape().events.len(), s.events);
    assert!(s.min_funding.0 > 0.1, "{:?}", s.min_funding);
    assert!(s.max_collapse.0 <= 1e-13, "{:?}", s.max_collapse);
    assert!(
        s.max_step_chain.0 <= c.world.max_step,
        "{:?}",
        s.max_step_chain
    );
    assert!(s.max_year_chain.0 <= MAX_YEAR, "{:?}", s.max_year_chain);
    for p in &c.counties {
        assert_eq!(p.chain.len(), p.points.len() + 1, "{}", p.county.key);
        assert!(p.chain.iter().all(|e| e.funded), "{}", p.county.key);
    }
    // The nodes are the atlas's regions.
    let mut nodes: Vec<&str> = tape().nodes.iter().map(|n| n.key.as_str()).collect();
    nodes.sort_unstable();
    let a = atlas();
    let keys: Vec<&str> = a.regions.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(nodes, keys);
    // R4, R5: every basis carries the marker, and the name its tag.
    let t = tape();
    assert_eq!(t.header.name, "demo-gb-v2 [illustrative]");
    let is_marked = |b: &Basis| matches!(b, Basis::Assumed(s) if s.starts_with(MARK));
    let lines = registry(t).unwrap_or_else(|e| panic!("{e}"));
    assert!(lines.len() > 30_000);
    for l in &lines {
        assert!(is_marked(&l.basis), "{}: {:?}", l.path, l.basis);
    }
    for a in &t.actors {
        assert!(is_marked(&a.basis), "{}", a.key);
    }
    assert!(is_marked(&t.genesis.basis));
    for e in &t.events {
        assert!(is_marked(&e.basis), "{}", e.key);
    }
    // Six actors a county, and v1's a, λ and b are not registered (decision 328).
    assert_eq!(t.actors.len(), 6 * 93);
    let params: Vec<&str> = t.params.iter().map(|p| p.key.as_str()).collect();
    for k in ["county.lan.a", "county.lan.lam", "county.lan.b"] {
        assert!(!params.contains(&k), "{k} is registered");
    }
    for q in StageParam::ALL {
        let k = format!("county.lan.{}", q.column());
        assert!(params.contains(&k.as_str()), "{k}");
    }
}

#[test]
fn c2g_is_the_probes() {
    // Decision 324: the stage's dials are P2.2a's registered C2g on F5, key by key, value and
    // unit, and the maker's reservation is IDLE's ψ 0.25 (decision 253).
    let s = stage();
    let f5 = Horse::named("f5").unwrap_or_else(|e| panic!("{e}"));
    let theirs = dials(&f5, "c2g").unwrap_or_else(|e| panic!("{e}"));
    for d in &theirs.values {
        let ours = s
            .dials
            .iter()
            .find(|x| x.key == d.key)
            .unwrap_or_else(|| panic!("the stage has no {}", d.key));
        assert_eq!(ours.value, d.value, "{}", d.key);
        assert_eq!(ours.unit, d.unit, "{}", d.key);
    }
    assert_eq!(s.dial("reserve.maker"), Some(0.25));
    // Every one the stage lists is the probe's or the ledger's or the reservation.
    for d in &s.dials {
        let known = theirs.values.iter().any(|x| x.key == d.key)
            || d.key.starts_with("ledger.")
            || d.key == "reserve.maker";
        assert!(known, "{} is not one of P2.2a's dials", d.key);
    }
    for (key, v) in C2G {
        assert_eq!(s.dial(key), Some(v), "{key}");
    }
    assert_eq!(
        s.dial("adjust.invest.capacity"),
        Some(2.0 * s.machine.delta)
    );
    assert_eq!(s.machine.delta, f5.delta);
    assert_eq!(s.machine.omega, f5.omega);
    assert_eq!(s.machine.kappa, f5.kappa);
}

/// The probe's wet instance for a county's v1 instance, under the stage's machine type.
fn horse(inst: &Instance, m: &MachineType) -> Horse {
    let v = |p: Param| inst[p.index()];
    Horse {
        id: "f5".into(),
        title: "a demo county".into(),
        county: ProbeCounty {
            workers: v(Param::Workers),
            land: v(Param::Land),
            space: v(Param::Space),
            chi_max: v(Param::ChiMax),
            eta: v(Param::Eta),
            g0: v(Param::G0),
            g1: v(Param::G1),
            k: v(Param::K),
            a: v(Param::A),
            lam: v(Param::Lam),
            b: v(Param::B),
        },
        county_basis: "docs/demo/WORLD.md".into(),
        delta: m.delta,
        omega: m.omega,
        kappa: m.kappa,
        config: Config::Wet,
        keys: Keys {
            horse: "horse".into(),
            maker: "maker".into(),
        },
        dials: "c2g".into(),
    }
}

/// The ten counties the mapping is held to by name, from the coal and cotton counties to the
/// Highlands, London's ring and Ulster.
const TEN: [&str; 10] = [
    "county.lan",
    "county.gla",
    "county.lks",
    "county.mdx",
    "county.sry",
    "county.sut",
    "county.ant",
    "county.nfk",
    "county.wil",
    "county.con",
];

#[test]
fn rule_a_is_the_probes() {
    // Decision 329 (O32 for rule A): worldgen computes rule A's coefficients, builds unit 1g's
    // chain and reads its point as the probe's harness does, bit for bit, on F5, F6 and ten
    // counties at genesis and in 1850, without depending on the probe crate.
    let c = compiled();
    let cl = clock(&c.world);
    let s = stage();
    let mut cases: Vec<(String, Horse)> = Vec::new();
    for id in ["f5", "f6"] {
        cases.push((
            id.into(),
            Horse::named(id).unwrap_or_else(|e| panic!("{e}")),
        ));
    }
    for k in TEN {
        let p = c
            .counties
            .iter()
            .find(|p| p.county.key == k)
            .unwrap_or_else(|| panic!("{k}"));
        for m in [0, 1200] {
            cases.push((
                format!("{k} month {m}"),
                horse(&p.instance_at(m), &s.machine),
            ));
        }
    }
    for (what, h) in &cases {
        let m = MachineType {
            delta: h.delta,
            omega: h.omega,
            kappa: h.kappa,
            ..s.machine.clone()
        };
        let n = h.county;
        let ours = chain::rule_a(&m, &cl, n.a, n.lam, n.b);
        let theirs = h.rule_a(52).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            ours.fodder_land.to_bits(),
            theirs.fodder_land.to_bits(),
            "{what}"
        );
        assert_eq!(
            ours.own_hours.to_bits(),
            theirs.own_hours.to_bits(),
            "{what}"
        );
        assert_eq!(ours.labour.to_bits(), theirs.labour.to_bits(), "{what}");
        assert_eq!(ours.pasture.to_bits(), theirs.pasture.to_bits(), "{what}");
        let mut inst: Instance = [0.0; 11];
        for (p, v) in [
            (Param::Workers, n.workers),
            (Param::Land, n.land),
            (Param::Space, n.space),
            (Param::Eta, n.eta),
            (Param::G0, n.g0),
            (Param::G1, n.g1),
            (Param::K, n.k),
            (Param::A, n.a),
            (Param::Lam, n.lam),
            (Param::B, n.b),
            (Param::ChiMax, n.chi_max),
        ] {
            inst[p.index()] = v;
        }
        let cp = chain_params(&inst, &m, &cl);
        let chain_ours = chain::goods_chain(&cp, &cl, 2);
        let chain_theirs = h.chain(52, 2).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(chain_ours, chain_theirs, "{what}");
        let e = chain::point(&cp, &cl).unwrap_or_else(|e| panic!("{what}: {e}"));
        let t = h.point(52).unwrap_or_else(|e| panic!("{what}: {e}"));
        for (name, a, b) in [
            ("x*", e.x_star, t.x_star),
            ("v", e.v, t.v),
            ("p_s", e.p_s, t.p_s),
            ("y", e.y, t.y),
            ("n_a", e.n_a, t.n_a),
            ("p", e.p, t.p),
            ("good", e.good, t.good),
            ("pf", e.pf, t.pf),
            ("pk", e.pk, t.pk),
            ("ph", e.ph, t.ph),
            ("o", e.o, t.o),
            ("qf", e.qf, t.qf),
            ("made", e.made, t.made),
            ("sold", e.sold, t.sold),
            ("hours", e.hours, t.hours),
            ("task_hours", e.task_hours, t.task_hours),
            ("capacity", e.capacity, t.capacity),
            ("serving", e.serving, t.serving),
            ("provider_baskets", e.provider_baskets, t.provider_baskets),
        ] {
            assert_eq!(a.to_bits(), b.to_bits(), "{what}: {name} {a} against {b}");
        }
        assert_eq!(e.funded, t.funded, "{what}");
        // The solve is the oracle's own: 1g's economy on the same chain.
        let q = ChainEconomy::new(chain_ours).unwrap_or_else(|e| panic!("{e}"));
        assert!(matches!(q.solve(), Ok(Regime::Interior(_))), "{what}");
    }
    assert_eq!(cases.len(), 22);
}

#[test]
fn v2_genesis_is_the_horses_rule_at_each_county() {
    // WORLD-V2 §5 (decision 325): each county's genesis is probe::horses::setup::genesis on the
    // same instance at C2g, with every price and coin divided by p/r: the same point bit for
    // bit, the same stocks, the same ratios to 1e-14.
    let c = compiled();
    let s = stage();
    for p in &c.counties {
        let h = horse(&p.county.genesis, &s.machine);
        let mut setup = Setup::registered("f5", 52).unwrap_or_else(|e| panic!("{e}"));
        setup.dials = dials(&h, "c2g").unwrap_or_else(|e| panic!("{e}"));
        setup.instance = h;
        setup.reserve = Some(0.25);
        let theirs = probe_genesis(&setup).unwrap_or_else(|e| panic!("{e}"));
        let e = &p.chain[0];
        assert_eq!(
            theirs.point.pk.to_bits(),
            e.pk.to_bits(),
            "{}",
            p.county.key
        );
        let ours =
            stage_genesis(&c.world, &s, &p.county.genesis, e).unwrap_or_else(|x| panic!("{x}"));
        let close = |a: f64, b: f64| ((a * e.p) / b - 1.0).abs() < 1e-14;
        for (i, market) in MARKETS.iter().enumerate() {
            assert!(
                close(ours.prices[i], theirs.prices[i]),
                "{} price of {market}",
                p.county.key
            );
            assert!(
                close(ours.coin[i], theirs.coin[i]),
                "{} coin {i}",
                p.county.key
            );
        }
        assert_eq!(ours.prices[5], 1.0);
        assert_eq!(ours.share, theirs.share);
        let stocks = [
            ours.good,
            ours.heads,
            ours.hours,
            ours.own,
            ours.finished,
            ours.fodder,
        ];
        assert_eq!(stocks.to_vec(), theirs.stock, "{}", p.county.key);
        assert_eq!(ours.band, theirs.band);
    }
}

/// Compile the stage with one of its tables' texts changed, and return the error.
fn refused(file: &str, edit: impl Fn(&str) -> String) -> String {
    let mut t = tables();
    let mut s = stage_tables();
    let slot = match file {
        "world.csv" => &mut t.world,
        "regions.csv" => &mut t.regions,
        "history.csv" => &mut t.history,
        "machine_types.csv" => &mut s.machine_types,
        "stage-v2a1.csv" => &mut s.stage,
        _ => unreachable!(),
    };
    *slot = edit(slot);
    match compile_stage(&t, &s, &atlas()) {
        Ok(_) => panic!("{file}: the edit compiled"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn the_stage_compiler_refuses_bad_tables() {
    let has = |e: &str, what: &str| assert!(e.contains(what), "{e}");
    let dial = |row: &'static str, to: &'static str| {
        move |t: &str| {
            assert!(t.contains(row), "{row}");
            t.replace(row, to)
        }
    };
    // Dials off C2g (decision 324): land's rate at C2's 1.3, s_K at 4δ, no reservation.
    has(
        &refused(
            "stage-v2a1.csv",
            dial("rate.land,0.1625,", "rate.land,1.3,"),
        ),
        "rate.land is 1.3, not 0.1625",
    );
    has(
        &refused(
            "stage-v2a1.csv",
            dial(
                "adjust.invest.capacity,0.16,",
                "adjust.invest.capacity,0.32,",
            ),
        ),
        "adjust.invest.capacity is 0.32, not 0.16",
    );
    has(
        &refused(
            "stage-v2a1.csv",
            dial("reserve.maker,0.25,", "reserve.maker,0,"),
        ),
        "reserve.maker is 0, not 0.25",
    );
    has(
        &refused("stage-v2a1.csv", dial("assign,ExPost,", "assign,Planned,")),
        "assign is `Planned`",
    );
    has(
        &refused(
            "stage-v2a1.csv",
            dial("name,demo-gb-v2 [illustrative],", "name,demo-gb-v2,"),
        ),
        "[illustrative]",
    );
    has(
        &refused(
            "stage-v2a1.csv",
            dial("machine_type,horse,", "machine_type,engine,"),
        ),
        "machine_types.csv has no `engine`",
    );
    // The machine type out of range, or rule B (decision 308).
    has(
        &refused("machine_types.csv", dial("horse,A,0.08,", "horse,A,1,")),
        "δ 1 a year is not in (0, 1)",
    );
    has(
        &refused(
            "machine_types.csv",
            dial("horse,A,0.08,52,0.85,", "horse,A,0.08,52,1,"),
        ),
        "ω 1 is not in [0, 1)",
    );
    has(
        &refused(
            "machine_types.csv",
            dial("horse,A,0.08,52,0.85,1,", "horse,A,0.08,52,0.85,2,"),
        ),
        "J_b is 1 tick",
    );
    has(
        &refused("machine_types.csv", dial("horse,A,", "horse,B,")),
        "rule B's",
    );
    // A county whose machine type is not the horse.
    has(
        &refused("regions.csv", |t| {
            t.replacen(",good,horse,,", ",good,engine,,", 1)
        }),
        "machine_types `horse`",
    );
    // The clock, as v1's (O39).
    has(
        &refused(
            "world.csv",
            dial("ticks_per_year,52,", "ticks_per_year,12,"),
        ),
        "ticks_per_year is 12, not 52",
    );
    // An unfunded date does not compile, and the error names the county and the date
    // (decision 326): Surrey, whose margin is v1's least (0.119 in April 1834), with its
    // population raised a further 30% over 1830-1834.
    let e = refused("history.csv", |t| {
        format!("{t}verify.sry,1830,1834,county.sry,workers,scale,1,1.3,12,,test,Surrey's N x1.3\n")
    });
    has(&e, "county.sry at 18");
    has(
        &e,
        "the provider cannot fund one basket per potential worker",
    );
    // An unknown stage.
    let mut st = stage_tables();
    st.key = "v2a4".into();
    let e = compile_stage(&tables(), &st, &atlas()).expect_err("an unknown stage");
    assert!(e.to_string().contains("no stage `v2a4`"), "{e}");
}

/// A county's ids and targets in a run.
struct Run {
    key: String,
    node: NodeId,
    actors: [ActorId; 4],
    lines: [usize; 6],
    /// The 18 observables' targets.
    target: [f64; 18],
}

/// The 18 observables of P2.2a (HORSES-SPEC §7.1) for a county at a tick: v; fodder's, the
/// horse's, the horse-day's and the good's prices over r; the good desk's used share; the six
/// markets' cleared volumes; the four desks' outputs; the tasks' heads and the maker's.
fn observables(sim: &Sim, r: &rustyecon_engine::prelude::TickReport, c: &Run) -> [f64; 18] {
    let l = c.lines.map(|i| &r.markets[i]);
    let pr = |i: usize| l[i].price / l[1].price;
    let (used, y_good) = match sim.actor_state(c.actors[0]) {
        Some(ActorState::GoodDesk(s)) => (s.used, s.output),
        other => panic!("{}: {other:?}", c.key),
    };
    let (held, y_hours) = match sim.actor_state(c.actors[1]) {
        Some(ActorState::Capacity(s)) => (s.held, s.output),
        other => panic!("{}: {other:?}", c.key),
    };
    let (serving, y_horse) = match sim.actor_state(c.actors[2]) {
        Some(ActorState::Maker(s)) => (s.serving, s.output),
        other => panic!("{}: {other:?}", c.key),
    };
    let y_fodder = match sim.actor_state(c.actors[3]) {
        Some(ActorState::MachDesk(s)) => s.output,
        other => panic!("{}: {other:?}", c.key),
    };
    [
        pr(0),
        pr(2),
        pr(3),
        pr(4),
        pr(5),
        used,
        l[0].cleared,
        l[1].cleared,
        l[2].cleared,
        l[3].cleared,
        l[4].cleared,
        l[5].cleared,
        y_good,
        y_hours,
        y_horse,
        y_fodder,
        held,
        serving,
    ]
}

/// Each county's ids and targets at its genesis point.
fn runs(sim: &Sim, c: &Compiled, r0: &rustyecon_engine::prelude::TickReport) -> Vec<Run> {
    let w = sim.world();
    let cl = clock(&c.world);
    let good = |k: &str| w.id_of::<GoodId>(k).unwrap_or_else(|| panic!("{k}"));
    let goods = MARKETS.map(good);
    let index: BTreeMap<(NodeId, GoodId), usize> = r0
        .markets
        .iter()
        .enumerate()
        .map(|(i, l)| ((l.node, l.good), i))
        .collect();
    c.counties
        .iter()
        .map(|p| {
            let k = &p.county.key;
            let node = w.id_of::<NodeId>(k).unwrap_or_else(|| panic!("{k}"));
            let actor = |r: &str| {
                w.id_of::<ActorId>(&format!("{k}.{r}"))
                    .unwrap_or_else(|| panic!("{k}.{r}"))
            };
            let e = &p.chain[0];
            let t = cl.flow(FlowPerYear(p.county.genesis[Param::Land.index()]));
            Run {
                key: k.clone(),
                node,
                actors: [
                    actor("desk.good"),
                    actor("desk.capacity"),
                    actor("desk.maker"),
                    actor("desk.fodder"),
                ],
                lines: goods.map(|g| index[&(node, g)]),
                target: target_of(e, t),
            }
        })
        .collect()
}

/// The 18 observables' targets at a chain point `e`, with land `t` a tick: the oracle's point in
/// [`observables`]' order (the horse market's volume is the heads sold, q_b·(1 − δ·a/κ)).
fn target_of(e: &ChainPoint, t: f64) -> [f64; 18] {
    [
        e.v,
        e.pf,
        e.pk,
        e.ph,
        e.p,
        e.one_minus_x,
        e.n_a,
        t,
        e.qf,
        e.sold,
        e.task_hours,
        e.good,
        e.good,
        e.task_hours,
        e.made,
        e.qf,
        e.capacity,
        e.serving,
    ]
}

#[test]
fn v2_rests_at_every_county_oracle_point() {
    // WORLD-V2 §11.5's E2 over a short window (mode A): with no history, every county stays at
    // its oracle point. Each of its 18 observables is within 1e-12 in log of 1g's value for
    // three ticks and within 1e-9 for two years; every market, the horse's too, trades every
    // tick; and the ledger closes every tick (conservation: the engine refuses a tick that does
    // not, and its largest margin is at most 1).
    let mut t = tables();
    t.history = t
        .history
        .lines()
        .next()
        .map_or(String::new(), |h| format!("{h}\n"));
    let c = compile_stage(&t, &stage_tables(), &atlas()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(c.summary.events, 0);
    let tape = Tape::from_ron(&c.tape).unwrap_or_else(|e| panic!("{e}"));
    let mut sim = Sim::new(&tape).unwrap_or_else(|e| panic!("{e}"));
    let mut runs_: Vec<Run> = Vec::new();
    let (mut worst3, mut worst) = ((0.0_f64, String::new()), (0.0_f64, String::new()));
    let ticks = 104;
    for tick in 0..ticks {
        let r = sim.step().unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        assert!(
            r.audit.max_margin <= 1.0,
            "tick {tick}: {}",
            r.audit.max_margin
        );
        if tick == 0 {
            runs_ = runs(&sim, &c, &r);
            assert_eq!(runs_.len(), 93);
        }
        for run in &runs_ {
            for &i in &run.lines {
                assert!(
                    r.markets[i].trades(),
                    "{} at {tick}: a market did not trade",
                    run.key
                );
                assert_eq!(r.markets[i].node, run.node);
            }
            let o = observables(&sim, &r, run);
            for (i, (x, y)) in o.iter().zip(&run.target).enumerate() {
                let d = num::ln(x / y).abs();
                assert!(
                    d.is_finite(),
                    "{} at {tick}: observable {i} is {x}",
                    run.key
                );
                let at = format!("{} observable {i} at tick {tick}", run.key);
                if tick < 3 && d > worst3.0 {
                    worst3 = (d, at.clone());
                }
                if d > worst.0 {
                    worst = (d, at);
                }
            }
        }
    }
    println!("three ticks: {worst3:?}; {ticks} ticks: {worst:?}");
    assert!(worst3.0 <= 1e-12, "{worst3:?}");
    assert!(worst.0 <= 1e-9, "{worst:?}");
}

#[test]
fn demo_v2_runs_deterministically() {
    // Two runs of the committed tape give one hash stream, tick by tick, for two years.
    let mut a = Sim::new(tape()).unwrap_or_else(|e| panic!("{e}"));
    let mut b = Sim::new(tape()).unwrap_or_else(|e| panic!("{e}"));
    for tick in 0..104 {
        let ha = a.step().unwrap_or_else(|e| panic!("{e}")).hash;
        let hb = b.step().unwrap_or_else(|e| panic!("{e}")).hash;
        assert_eq!(ha, hb, "tick {tick}");
    }
    assert_eq!(a.hash(), b.hash());
}

#[test]
#[ignore = "the long run, 20 to 40 s in release: scripts/gate.sh runs it by name"]
fn demo_v2_pin() {
    // The tape through the first tick of 1901, so every step fires: every event fired, the
    // ledger closed every tick, and the recorded hashes (decision 339). It scores nothing: the
    // long run's scoring against the registration is D2.4's (WORLD-V2 §11.4, §11.5).
    let t0 = std::time::Instant::now();
    let t = tape();
    let mut sim = Sim::new(t).unwrap_or_else(|e| panic!("{e}"));
    // The history's horizon (world.csv's `end`), read from the tape's own clock.
    let horizon = Date::parse("1901-01-01").unwrap_or_else(|e| panic!("{e}"));
    let end = sim
        .world()
        .clock
        .tick_of(horizon)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(end, 7_851);
    let until = end + 1;
    let mut stream = Vec::with_capacity(8 * until as usize);
    let mut fired = 0usize;
    let mut fired_at_end = 0usize;
    let mut margin = 0.0_f64;
    for tick in 0..until {
        let r = sim.step().unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        stream.extend_from_slice(&r.hash.to_le_bytes());
        fired += r.events.len();
        margin = margin.max(r.audit.max_margin);
        if tick == end {
            fired_at_end = r.events.len();
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    let final_hash = sim.hash();
    let stream_hash = fnv1a_64(&stream);
    println!(
        "{until} ticks, {fired} events fired ({fired_at_end} in the horizon's tick), largest \
         ledger margin {margin:e}: final hash 0x{final_hash:016x}, hash stream \
         0x{stream_hash:016x}; {secs:.1} s, {:.0} ticks a second",
        until as f64 / secs
    );
    assert_eq!(fired, t.events.len());
    assert!(fired_at_end > 0);
    assert!(margin <= 1.0);
    assert_eq!(
        (final_hash, stream_hash),
        (FINAL_HASH, STREAM_HASH),
        "final 0x{final_hash:016x}, stream 0x{stream_hash:016x}"
    );
}

/// A county's second-pass readings at its chain point `e` under instance `inst`, as a run at
/// rest would record them: prices with r = 1, every market cleared at its equilibrium volume,
/// the desks' outputs and stocks at rest, the capacity desk at its target, the provider paying
/// all it owes, nothing rationed, every market trading, the maker's markup 1.
fn at_rest_v2(e: &ChainPoint, inst: &Instance, s: &Stage) -> Readings {
    let c = clock(&compiled().world);
    let n = c.flow(FlowPerYear(inst[Param::Workers.index()]));
    let t = c.flow(FlowPerYear(inst[Param::Land.index()]));
    let si = stage_instance(inst, &s.machine, &c);
    let (kappa, delta) = s.machine.per_tick(&c);
    let mut params = inst.map(Some);
    for p in [Param::A, Param::Lam, Param::B] {
        params[p.index()] = None;
    }
    let coef = [
        StageParam::FodderLand,
        StageParam::OwnHours,
        StageParam::HorseLabour,
        StageParam::HorseLand,
    ]
    .map(|q| Some(si[q.index()]));
    let due = n * e.p_s;
    let mut chain = ChainReadings {
        price_fodder: Some(e.pf),
        price_horse: Some(e.pk),
        cleared_fodder: Some(e.qf),
        cleared_horse: Some(e.sold),
        horse_traded: Some(true),
        coef,
        kappa: Some(kappa),
        delta: Some(delta),
        run_fodder: Some(1.0),
        run_labour: Some(0.0),
        psi: s.dial("reserve.maker"),
        used: Some(e.one_minus_x),
        out_good: Some(e.good),
        out_hours: Some(e.task_hours),
        out_horse: Some(e.made),
        out_fodder: Some(e.qf),
        held: Some(e.capacity),
        target: Some(e.capacity),
        serving: Some(e.serving),
        clock: Some(c),
        window: Vec::new(),
    };
    let prices = [Some(e.v), Some(1.0), Some(e.ph), Some(e.p)];
    chain.window = vec![WindowTick {
        horse_traded: true,
        markup: chain.markup_inputs(&prices),
        psi: chain.psi,
    }];
    Readings {
        prices,
        cleared: [Some(e.n_a), Some(t), Some(e.task_hours), Some(e.good)],
        params,
        n_tick: Some(n),
        share: Some(e.one_minus_x),
        due: Some(due),
        paid: Some(due),
        rationing: vec![(1.0, 1.0)],
        traded: vec![true],
        chain: Some(chain),
    }
}

#[test]
fn lens_v2_domains_hold_the_oracle_range() {
    // WORLD-V2 §7 (decision 332): each lens's domain is fixed before any run, from the oracle's
    // range over every county at genesis and after every step date, with a margin. The range is
    // computed through the lenses' own measures at each county's chain point and must lie
    // inside the domain; the margins are printed. At rest the dynamic lenses read their rest
    // values (a markup 1 or 0, a ratio to target 0, a count 0), inside their domains too; the
    // three oracle lenses read 0 at rest and are held by the mirror's long run (§6) instead.
    let s = stage();
    let lenses = rustyecon_worldgen::lens::demo_gb_v2().unwrap_or_else(|e| panic!("{e}"));
    let mut lines = Vec::new();
    for l in &lenses {
        let m = Measure::of(&l.key).unwrap_or_else(|| panic!("{}: no measure", l.key));
        if m.level().needs_observe() {
            continue;
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut at_lo, mut at_hi) = (String::new(), String::new());
        for plan in &compiled().counties {
            let first = at_rest_v2(&plan.chain[0], &plan.county.genesis, &s);
            let months = std::iter::once(0).chain(plan.points.iter().map(|(m, _)| *m));
            for (month, e) in months.zip(&plan.chain) {
                let x = at_rest_v2(e, &plan.instance_at(month), &s);
                let v = value(m, &x, Some(&first)).unwrap_or_else(|e| panic!("{}: {e}", l.key));
                let at = format!("{} {}", plan.county.key, month);
                if v < lo {
                    (lo, at_lo) = (v, at.clone());
                }
                if v > hi {
                    (hi, at_hi) = (v, at);
                }
            }
        }
        let (a, b) = l.domain;
        let span = |x: f64, y: f64| match l.scale {
            rustyecon_worldgen::tables::Scale::SequentialLog => {
                (num::ln(y) - num::ln(x)) / (num::ln(b) - num::ln(a))
            }
            _ => (y - x) / (b - a),
        };
        let line = format!(
            "{:<22} domain [{a}, {b}]  oracle [{lo:.4} ({at_lo}), {hi:.4} ({at_hi})]  margins \
             {:.2} below, {:.2} above",
            l.key,
            span(a, lo),
            span(hi, b)
        );
        println!("{line}");
        assert!(a <= lo && hi <= b, "{line}");
        lines.push(line);
    }
    assert_eq!(lines.len(), 33, "every lens but the three oracle lenses");
}

#[test]
fn v2_flow_path_is_v1() {
    // R1 (WORLD-V2 §9.3): the stage with its machine on the flow path (δ = 1 a tick, ω 0, planned
    // assignment, the probe's C2), which runs the owner desk as P2.0's good desk and the maker as
    // the type desk (P2.2a's R1a, P2.1's I0 bit for bit), compiles v1's tables into a tape whose
    // run gives every county's prices, cleared volumes and coins equal to v1's tape's, bit for
    // bit, for ten years with v1's history firing. The actor kinds differ, so the hashes do.
    let mut s = stage();
    s.machine = MachineType {
        delta: 1.0,
        omega: 0.0,
        ..s.machine.clone()
    };
    s.assign = "Planned".into();
    s.name = "demo-gb flow path [illustrative]".into();
    let r1a = Horse::named("r1a").unwrap_or_else(|e| panic!("{e}"));
    let c2 = dials(&r1a, "c2").unwrap_or_else(|e| panic!("{e}"));
    let mut d: Vec<rustyecon_worldgen::tables::Dial> = s
        .dials
        .iter()
        .filter(|x| x.key.starts_with("ledger."))
        .cloned()
        .collect();
    d.extend(c2.values.iter().map(|x| rustyecon_worldgen::tables::Dial {
        key: x.key.clone(),
        value: x.value,
        unit: x.unit,
        note: x.basis.clone(),
    }));
    s.dials = d;
    let flow = compile_stage_parsed(&tables(), &s, &atlas()).unwrap_or_else(|e| panic!("{e}"));
    let v1 = rustyecon_worldgen::compile(&tables(), &atlas()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        flow.summary.events, v1.summary.events,
        "a b step is one event here"
    );
    let tf = Tape::from_ron(&flow.tape).unwrap_or_else(|e| panic!("{e}"));
    let t1 = Tape::from_ron(&v1.tape).unwrap_or_else(|e| panic!("{e}"));
    let mut a = Sim::new(&tf).unwrap_or_else(|e| panic!("{e}"));
    let mut b = Sim::new(&t1).unwrap_or_else(|e| panic!("{e}"));
    let (wa, wb) = (a.world().clone(), b.world().clone());
    let coin = |w: &rustyecon_engine::prelude::World| w.id_of::<GoodId>("coin").expect("coin");
    let actors = |w: &rustyecon_engine::prelude::World| -> Vec<ActorId> {
        let mut v = Vec::new();
        for n in &w.nodes {
            for r in ["desk.good", "desk.mach", "provider", "workers"] {
                v.push(
                    w.id_of::<ActorId>(&format!("{}.{r}", n.key))
                        .expect("an actor"),
                );
            }
        }
        v
    };
    let (aa, ab) = (actors(&wa), actors(&wb));
    let mut compared = 0usize;
    let mut fired = 0usize;
    for tick in 0..520 {
        let ra = a
            .step()
            .unwrap_or_else(|e| panic!("flow, tick {tick}: {e}"));
        let rb = b.step().unwrap_or_else(|e| panic!("v1, tick {tick}: {e}"));
        assert_eq!(ra.events.len(), rb.events.len(), "tick {tick}");
        fired += ra.events.len();
        let key = |w: &rustyecon_engine::prelude::World,
                   l: &rustyecon_engine::prelude::MarketLine| {
            (
                w.key_of(l.node).map(|k| k.as_str().to_string()),
                w.key_of(l.good).map(|k| k.as_str().to_string()),
            )
        };
        let lines = |w: &rustyecon_engine::prelude::World,
                     r: &rustyecon_engine::prelude::TickReport| {
            let mut v: Vec<_> = r
                .markets
                .iter()
                .map(|l| (key(w, l), l.price.to_bits(), l.cleared.to_bits()))
                .collect();
            v.sort();
            v
        };
        let (la, lb) = (lines(&wa, &ra), lines(&wb, &rb));
        assert_eq!(la.len(), 4 * 93);
        assert_eq!(la, lb, "tick {tick}: prices and cleared volumes");
        for (x, y) in aa.iter().zip(&ab) {
            let ca = a
                .holding(rustyecon_engine::prelude::Holder::Actor(*x))
                .map_or(0.0, |i| i.get(coin(&wa)));
            let cb = b
                .holding(rustyecon_engine::prelude::Holder::Actor(*y))
                .map_or(0.0, |i| i.get(coin(&wb)));
            assert_eq!(
                ca.to_bits(),
                cb.to_bits(),
                "tick {tick}: the coin of {:?}",
                wa.key_of(*x)
            );
            compared += 1;
        }
    }
    assert!(fired > 0, "v1's history fired in the ten years");
    println!("{compared} coins and 520 ticks of 372 market lines equal; {fired} events fired");
}

#[test]
fn battery_instances_are_the_harness_instances() {
    // WORLD-V2 §11.3 (D2.4): the battery's county table, which `rustyecon worldgen … --stage
    // v2a1 --instances PATH` writes and `horses --counties PATH` reads, holds each county's
    // instance in force on 1 January of the six years bit for bit, with the stage's machine type
    // and ψ. The harness's point at each row is the compiler's chain point in force from that
    // month, bit for bit, and its battery is P2.2a's 93 runs less the b × 2 pair exactly where
    // the registered targets (docs/demo/v2/targets.csv) are unfunded: 51,260 runs in all.
    use probe::horses::demo;
    use probe::horses::perturb::battery;
    use rustyecon_worldgen::stage::{battery_instances, BATTERY_YEARS};
    use std::collections::BTreeSet;
    let c = compiled();
    let s = stage();
    let text = battery_instances(c, &s).unwrap_or_else(|e| panic!("{e}"));
    let rows = demo::parse(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(rows.len(), 6 * 93);
    let registered = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/demo/v2/targets.csv"
    ))
    .unwrap_or_else(|e| panic!("targets.csv: {e}"));
    let mut unfunded: BTreeSet<(String, i32)> = BTreeSet::new();
    for line in registered.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        let year: i32 = f[2].parse().unwrap_or_else(|_| panic!("{line}"));
        if f[5] == "false" {
            assert_eq!(f[3], "2", "only b × 2 is ever unfunded: {line}");
            unfunded.insert((f[0].to_string(), year));
        }
    }
    assert_eq!(unfunded.len(), 317);
    let mut scored = 0usize;
    for p in &c.counties {
        let key = &p.county.key;
        for y in BATTERY_YEARS {
            let month = i64::from(y - c.world.start.y) * 12;
            let row =
                demo::find(&rows, &format!("demo:{key}@{y}")).unwrap_or_else(|e| panic!("{e}"));
            let h = &row.instance;
            let inst = p.instance_at(month);
            let n = h.county;
            let ours = [
                n.workers, n.land, n.space, n.eta, n.g0, n.g1, n.k, n.a, n.lam, n.b, n.chi_max,
            ];
            assert_eq!(ours.map(f64::to_bits), inst.map(f64::to_bits), "{key}@{y}");
            assert_eq!(
                (h.delta, h.omega, h.kappa, row.psi),
                (s.machine.delta, s.machine.omega, s.machine.kappa, 0.25),
                "{key}@{y}"
            );
            // The point: the compiler's chain point in force from the first of that January.
            let k = p.points.iter().take_while(|(m, _)| *m <= month).count();
            let e = &p.chain[k];
            let t = h.point(52).unwrap_or_else(|e| panic!("{key}@{y}: {e}"));
            for (what, a, b) in [
                ("x*", e.x_star, t.x_star),
                ("v", e.v, t.v),
                ("p", e.p, t.p),
                ("pf", e.pf, t.pf),
                ("pk", e.pk, t.pk),
                ("ph", e.ph, t.ph),
                ("y", e.y, t.y),
                ("made", e.made, t.made),
                ("capacity", e.capacity, t.capacity),
                ("serving", e.serving, t.serving),
            ] {
                assert_eq!(a.to_bits(), b.to_bits(), "{key}@{y}: {what}");
            }
            // The battery: the unfunded b × 2 pair dropped, as registered.
            let list = battery(h, 52).unwrap_or_else(|e| panic!("{key}@{y}: {e}"));
            let has_b2 = list.iter().any(|r| r.name == "b*2@genesis");
            assert_eq!(
                has_b2,
                !unfunded.contains(&(key.clone(), y)),
                "{key}@{y}: b × 2"
            );
            assert_eq!(list.len(), if has_b2 { 93 } else { 91 }, "{key}@{y}");
            scored += list.len();
        }
    }
    assert_eq!(scored, 51_260);
    // Through the command line's parse: the row's instance, C2g, and its ψ as the reservation.
    let path = std::env::temp_dir().join(format!("rustyecon-d24-{}.tsv", std::process::id()));
    std::fs::write(&path, &text).unwrap_or_else(|e| panic!("{e}"));
    let args: Vec<String> = [
        "--inst",
        "demo:county.lan@1850",
        "--counties",
        path.to_str().unwrap_or_else(|| panic!("a path")),
    ]
    .map(String::from)
    .to_vec();
    let a = probe::horses::cli::parse(&args, &[]).unwrap_or_else(|e| panic!("{e}"));
    let _ = std::fs::remove_file(&path);
    let row = demo::find(&rows, "demo:county.lan@1850").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(a.setup.instance, row.instance);
    assert_eq!(a.setup.reserve, Some(0.25));
    assert_eq!(
        a.setup.dials,
        dials(&row.instance, "c2g").unwrap_or_else(|e| panic!("{e}"))
    );
    assert!(a.rest.is_empty());
}

/// A county in the long run (WORLD-V2 §11.4): its ids, its targets by tick, and what the scorer
/// keeps, as the mirror's `d2_long.py` keeps it (registration §3.4).
struct Long {
    run: Run,
    name: String,
    provider: ActorId,
    maker: rustyecon_engine::rustyecon_agents::Maker,
    /// For genesis and each step date: the report tick it is in force from, the 18 targets, Y\*,
    /// every installed head at the point, and h.
    points: Vec<(u64, [f64; 18], f64, f64, f64)>,
    /// The index of the point in force.
    at: usize,
    dead: u64,
    dead_by: [u64; 5],
    idle: u64,
    zero_orders: u64,
    withheld: u64,
    short: u64,
    unpaid: f64,
    low: (f64, usize, u64),
    baskets: f64,
    heads_lo: (f64, u64),
    heads_hi: (f64, u64),
    hc_lo: f64,
    hc_hi: f64,
    plan_lo: f64,
    plan_hi: f64,
    pk_lo: f64,
    pk_hi: (f64, u64),
    mu_lo: (f64, u64),
    qr_lo: f64,
    qr_hi: f64,
    ph_over_o: f64,
    util_lo: f64,
    dh: Vec<f64>,
    fast: Vec<f64>,
    stock: Vec<f64>,
    end: [f64; 2],
}

impl Long {
    fn new(
        run: Run,
        name: String,
        provider: ActorId,
        maker: rustyecon_engine::rustyecon_agents::Maker,
        points: Vec<(u64, [f64; 18], f64, f64, f64)>,
        until: usize,
    ) -> Long {
        Long {
            run,
            name,
            provider,
            maker,
            points,
            at: 0,
            dead: 0,
            dead_by: [0; 5],
            idle: 0,
            zero_orders: 0,
            withheld: 0,
            short: 0,
            unpaid: 0.0,
            low: (f64::INFINITY, 0, 0),
            baskets: f64::INFINITY,
            heads_lo: (f64::INFINITY, 0),
            heads_hi: (0.0, 0),
            hc_lo: f64::INFINITY,
            hc_hi: 0.0,
            plan_lo: f64::INFINITY,
            plan_hi: 0.0,
            pk_lo: f64::INFINITY,
            pk_hi: (0.0, 0),
            mu_lo: (f64::INFINITY, 0),
            qr_lo: f64::INFINITY,
            qr_hi: f64::NEG_INFINITY,
            ph_over_o: f64::INFINITY,
            util_lo: f64::INFINITY,
            dh: Vec::with_capacity(until),
            fast: Vec::with_capacity(until),
            stock: Vec::with_capacity(until),
            end: [0.0; 2],
        }
    }
}

/// The markets a dead tick is judged on: indices into [`observables`], into a county's market
/// lines, and the mirror's names; labour, land, fodder, horse-days and the good (the horse
/// market is idle, not dead).
const DEAD: [(usize, usize, &str); 5] = [
    (6, 0, "L"),
    (7, 1, "R"),
    (8, 2, "Fo"),
    (10, 4, "H"),
    (11, 5, "G"),
];

/// The mirror's percentile: the sorted values' element at ⌊q·n⌋.
fn pct(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let i = ((q * s.len() as f64) as usize).min(s.len() - 1);
    s[i]
}

/// A long run, scored: every county's readouts, the scorer's two tables, and what the run did.
struct LongRun {
    counties: Vec<Long>,
    /// `long-run.csv`: a row a county, in the registered long-run.csv's first 22 columns, then
    /// the extras.
    table: String,
    /// `series.csv`: every 13th tick of every county.
    series: String,
    fired: usize,
    fired_at_end: usize,
    margin: f64,
    final_hash: u64,
    stream_hash: u64,
    secs: f64,
    sim: Sim,
}

/// Run the compiled world `c`'s tape `t` for `until` ticks and score every county every tick
/// against its own oracle point at the params in force (1g: the compiler's chain point after the
/// county's last step date), as the registration's mirror (`d2_long.py`) scores it: D̂ over
/// P2.2a's observables without the horse market's volume; dead ticks (labour, land, fodder,
/// horse-days or the good below half its oracle volume, or not trading); idle horse-market
/// ticks, ticks without an order and ticks the maker withheld; transfer shortfalls; troughs;
/// the heads against the oracle and the desk's own target; the horse's price and the two
/// markups. Nothing it reads reaches the run (R13).
fn long_run(c: &Compiled, t: &Tape, until: u64) -> LongRun {
    use probe::horses::harness::maker_readout;
    use rustyecon_engine::prelude::{ClassId, SideTag};
    use rustyecon_engine::rustyecon_agents::Spec;
    use rustyecon_worldgen::history;
    let t0 = std::time::Instant::now();
    let s = stage();
    let mut sim = Sim::new(t).unwrap_or_else(|e| panic!("{e}"));
    let w = sim.world().clone();
    let cl = clock(&c.world);
    let (kappa, delta) = s.machine.per_tick(&cl);
    let class = |k: &str| w.id_of::<ClassId>(k).unwrap_or_else(|| panic!("{k}"));
    let classes = [class("owners"), class("workers")];
    let good = |k: &str| w.id_of::<GoodId>(k).unwrap_or_else(|| panic!("{k}"));
    let goods = MARKETS.map(good);
    let items = [good("good"), good("land")];
    let mut counties: Vec<Long> = Vec::new();
    let mut stream = Vec::with_capacity(8 * until as usize);
    let mut fired = 0usize;
    let mut fired_at_end = 0usize;
    let mut margin = 0.0_f64;
    let mut series = String::from(
        "county,tick,dhat,prices_dhat,heads_dhat,heads_over_oracle,capacity_over_oracle,\
         capacity_over_desk_target,horse_price_over_target,maker_markup,hday_markup,\
         good_over_target\n",
    );
    for tick in 0..until {
        let r = sim.step().unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        stream.extend_from_slice(&r.hash.to_le_bytes());
        fired += r.events.len();
        margin = margin.max(r.audit.max_margin);
        if tick + 1 == until {
            fired_at_end = r.events.len();
        }
        if tick == 0 {
            for (run, p) in runs(&sim, c, &r).into_iter().zip(&c.counties) {
                assert_eq!(run.key, p.county.key);
                let k = &p.county.key;
                let provider = w
                    .id_of::<ActorId>(&format!("{k}.provider"))
                    .unwrap_or_else(|| panic!("{k}.provider"));
                let maker = w
                    .actors
                    .iter()
                    .find(|a| a.id == run.actors[2])
                    .and_then(|a| match &a.spec {
                        Spec::Maker(m) => Some(m.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("{k}: no maker"));
                let land_space = |inst: &Instance| {
                    (
                        cl.flow(FlowPerYear(inst[Param::Land.index()])),
                        inst[Param::Space.index()],
                    )
                };
                let mut points = Vec::with_capacity(p.chain.len());
                let (tl, h) = land_space(&p.county.genesis);
                let e = &p.chain[0];
                points.push((0, target_of(e, tl), e.y, e.hours / kappa, h));
                for (j, (m, _)) in p.points.iter().enumerate() {
                    let date =
                        Date::parse(&history::date(&c.world, *m)).unwrap_or_else(|e| panic!("{e}"));
                    let fires = cl.tick_of(date).unwrap_or_else(|e| panic!("{e}"));
                    let (tl, h) = land_space(&p.instance_at(*m));
                    let e = &p.chain[j + 1];
                    points.push((fires, target_of(e, tl), e.y, e.hours / kappa, h));
                }
                assert!(points.windows(2).all(|x| x[0].0 <= x[1].0), "{k}");
                counties.push(Long::new(
                    run,
                    p.county.name.clone(),
                    provider,
                    maker,
                    points,
                    until as usize,
                ));
            }
            assert_eq!(counties.len(), 93);
        }
        // What each household class bought of each basket item at each node this tick.
        let mut bought: BTreeMap<(NodeId, usize, usize), f64> = BTreeMap::new();
        for l in &r.rationing {
            if l.side != SideTag::Buy {
                continue;
            }
            let (Some(h), Some(i)) = (
                classes.iter().position(|x| *x == l.class),
                items.iter().position(|x| *x == l.good),
            ) else {
                continue;
            };
            *bought.entry((l.node, h, i)).or_insert(0.0) += l.filled;
        }
        for cty in &mut counties {
            while cty.at + 1 < cty.points.len() && cty.points[cty.at + 1].0 <= tick {
                cty.at += 1;
            }
            let (_, tg, y_star, heads_star, h) = cty.points[cty.at];
            cty.run.target = tg;
            let o = observables(&sim, &r, &cty.run);
            let gap: Vec<f64> = o
                .iter()
                .zip(&tg)
                .map(|(x, y)| {
                    if *x > 0.0 && x.is_finite() {
                        num::ln(x / y).abs()
                    } else {
                        f64::INFINITY
                    }
                })
                .collect();
            let max = |from: usize, to: usize| {
                (from..to)
                    .filter(|&i| i != 9)
                    .map(|i| gap[i])
                    .fold(0.0, f64::max)
                    / 1e-3
            };
            let dh = max(0, 18);
            cty.dh.push(dh);
            cty.fast.push(max(0, 6));
            cty.stock.push(max(16, 18));
            let l = cty.run.lines.map(|i| &r.markets[i]);
            let mut dead = false;
            for (m, &(i, line, _)) in DEAD.iter().enumerate() {
                let ratio = o[i] / tg[i];
                if ratio < cty.low.0 {
                    cty.low = (ratio, m, tick);
                }
                if ratio < 0.5 || !l[line].trades() {
                    cty.dead_by[m] += 1;
                    dead = true;
                }
            }
            cty.dead += u64::from(dead);
            cty.idle += u64::from(o[9] < 0.5 * tg[9]);
            let cap = match sim.actor_state(cty.run.actors[1]) {
                Some(ActorState::Capacity(x)) => *x,
                other => panic!("{}: {other:?}", cty.run.key),
            };
            cty.zero_orders += u64::from(cap.order <= 0.0);
            let price_of = |g: GoodId| goods.iter().position(|x| *x == g).map(|i| l[i].price);
            let res = maker_readout(&sim, &cty.maker, &price_of).unwrap_or_else(|e| panic!("{e}"));
            cty.withheld += u64::from(res.withholds);
            if res.markup < cty.mu_lo.0 {
                cty.mu_lo = (res.markup, tick);
            }
            if let Some(ActorState::Provider(pv)) = sim.actor_state(cty.provider) {
                if pv.paid < pv.due {
                    cty.short += 1;
                    cty.unpaid += (pv.due - pv.paid) / pv.due;
                }
            }
            // Baskets eaten: each household's scarcest item over its weight, as the harness
            // reads them (the good 1, land h).
            let mut eaten = 0.0;
            for hh in 0..2 {
                let got = |i: usize| bought.get(&(cty.run.node, hh, i)).copied().unwrap_or(0.0);
                let g = num::max_scale(got(0), 1.0).unwrap_or_else(|e| panic!("{e}"));
                let sp = num::max_scale(got(1), h).unwrap_or_else(|e| panic!("{e}"));
                eaten += g.min(sp);
            }
            cty.baskets = cty.baskets.min(eaten / y_star);
            let heads = (o[16] + o[17]) / heads_star;
            if heads < cty.heads_lo.0 {
                cty.heads_lo = (heads, tick);
            }
            if heads > cty.heads_hi.0 {
                cty.heads_hi = (heads, tick);
            }
            let hc = o[16] / tg[16];
            cty.hc_lo = cty.hc_lo.min(hc);
            cty.hc_hi = cty.hc_hi.max(hc);
            let plan = if cap.target > 0.0 {
                cap.held / cap.target
            } else {
                f64::INFINITY
            };
            cty.plan_lo = cty.plan_lo.min(plan);
            cty.plan_hi = cty.plan_hi.max(plan);
            let pk = o[2] / tg[2];
            cty.pk_lo = cty.pk_lo.min(pk);
            if pk > cty.pk_hi.0 {
                cty.pk_hi = (pk, tick);
            }
            // The horse-day's markup over its full cost at posted prices; the running cost is a
            // unit of fodder (run.fodder 1, run.labour 0), summed as the capacity desk sums it.
            let running = (0.0 + 1.0 * l[2].price) + 0.0 * l[0].price;
            let full = running + delta * l[3].price / kappa;
            let qr = l[4].price / full - 1.0;
            cty.qr_lo = cty.qr_lo.min(qr);
            cty.qr_hi = cty.qr_hi.max(qr);
            cty.ph_over_o = cty.ph_over_o.min(l[4].price / running);
            if cap.held > 0.0 {
                cty.util_lo = cty.util_lo.min(o[13] / (kappa * cap.held));
            }
            cty.end = [dh, heads];
            if tick % 13 == 0 {
                series.push_str(&format!(
                    "{},{tick},{dh:e},{:e},{:e},{heads:e},{hc:e},{plan:e},{pk:e},{:e},{qr:e},{:e}\n",
                    cty.run.key,
                    cty.fast[cty.fast.len() - 1],
                    cty.stock[cty.stock.len() - 1],
                    res.markup,
                    o[11] / tg[11]
                ));
            }
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    let final_hash = sim.hash();
    let stream_hash = fnv1a_64(&stream);
    let mut table = String::from(
        "county,name,dead,idle_horse,zero_orders,withheld,shortfall_ticks,\
         lowest_cleared_over_target,lowest_market,dhat_median,dhat_p90,dhat_max,\
         prices_dhat_median,heads_dhat_median,heads_over_oracle_min,heads_over_oracle_max,\
         heads_over_oracle_1901,capacity_over_desk_target_min,horse_price_over_target_max,\
         maker_markup_min,quasi_rent_max,dhat_1901,dead_L,dead_R,dead_Fo,dead_H,dead_G,\
         lowest_cleared_tick,dhat_max_tick,heads_min_tick,heads_max_tick,\
         capacity_over_oracle_min,capacity_over_oracle_max,capacity_over_desk_target_max,\
         horse_price_over_target_min,horse_price_max_tick,maker_markup_min_tick,\
         quasi_rent_min,lowest_baskets_over_Y,shortfall_unpaid_mean,hday_over_running_min,\
         utilisation_min\n",
    );
    for cty in &counties {
        let dmax = cty.dh.iter().copied().fold(0.0, f64::max);
        let dmax_at = cty.dh.iter().position(|&x| x == dmax).unwrap_or(0);
        let unpaid = if cty.short > 0 {
            cty.unpaid / cty.short as f64
        } else {
            0.0
        };
        let row = [
            cty.run.key.clone(),
            cty.name.clone(),
            cty.dead.to_string(),
            cty.idle.to_string(),
            cty.zero_orders.to_string(),
            cty.withheld.to_string(),
            cty.short.to_string(),
            format!("{:e}", cty.low.0),
            DEAD[cty.low.1].2.to_string(),
            format!("{:e}", pct(&cty.dh, 0.5)),
            format!("{:e}", pct(&cty.dh, 0.9)),
            format!("{dmax:e}"),
            format!("{:e}", pct(&cty.fast, 0.5)),
            format!("{:e}", pct(&cty.stock, 0.5)),
            format!("{:e}", cty.heads_lo.0),
            format!("{:e}", cty.heads_hi.0),
            format!("{:e}", cty.end[1]),
            format!("{:e}", cty.plan_lo),
            format!("{:e}", cty.pk_hi.0),
            format!("{:e}", cty.mu_lo.0),
            format!("{:e}", cty.qr_hi),
            format!("{:e}", cty.end[0]),
            cty.dead_by[0].to_string(),
            cty.dead_by[1].to_string(),
            cty.dead_by[2].to_string(),
            cty.dead_by[3].to_string(),
            cty.dead_by[4].to_string(),
            cty.low.2.to_string(),
            dmax_at.to_string(),
            cty.heads_lo.1.to_string(),
            cty.heads_hi.1.to_string(),
            format!("{:e}", cty.hc_lo),
            format!("{:e}", cty.hc_hi),
            format!("{:e}", cty.plan_hi),
            format!("{:e}", cty.pk_lo),
            cty.pk_hi.1.to_string(),
            cty.mu_lo.1.to_string(),
            format!("{:e}", cty.qr_lo),
            format!("{:e}", cty.baskets),
            format!("{unpaid:e}"),
            format!("{:e}", cty.ph_over_o),
            format!("{:e}", cty.util_lo),
        ];
        table.push_str(&row.join(","));
        table.push('\n');
    }
    LongRun {
        counties,
        table,
        series,
        fired,
        fired_at_end,
        margin,
        final_hash,
        stream_hash,
        secs,
        sim,
    }
}

#[test]
fn the_long_run_scorer_reads_the_rest_point() {
    // D2.4, before any scored run (decision 311): the long run's scorer on the stage with the
    // history removed, where every county rests at its oracle point, reads that point: D̂ at
    // rounding, the heads, the horse's price, the capacity desk's holding against its own
    // target, the baskets eaten and the maker's markup each 1 to 1e-9, the horse-day's markup 0,
    // and no dead, idle, no-order, withheld or short tick, for two years.
    let mut tb = tables();
    tb.history = tb
        .history
        .lines()
        .next()
        .map_or(String::new(), |h| format!("{h}\n"));
    let c = compile_stage(&tb, &stage_tables(), &atlas()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(c.summary.events, 0);
    let t = Tape::from_ron(&c.tape).unwrap_or_else(|e| panic!("{e}"));
    let lr = long_run(&c, &t, 104);
    assert_eq!(lr.fired, 0);
    assert!(lr.margin <= 1.0);
    let one = |x: f64, what: &str, k: &str| {
        assert!((x - 1.0).abs() <= 1e-9, "{k}: {what} {x}");
    };
    for cty in &lr.counties {
        let k = &cty.run.key;
        assert_eq!(cty.dh.len(), 104);
        let dmax = cty.dh.iter().copied().fold(0.0, f64::max);
        assert!(dmax <= 1e-6, "{k}: D-hat {dmax}");
        assert_eq!(
            (cty.dead, cty.idle, cty.zero_orders, cty.withheld, cty.short),
            (0, 0, 0, 0, 0),
            "{k}"
        );
        for (what, x) in [
            ("lowest cleared", cty.low.0),
            ("heads low", cty.heads_lo.0),
            ("heads high", cty.heads_hi.0),
            ("capacity low", cty.hc_lo),
            ("capacity high", cty.hc_hi),
            ("desk's target low", cty.plan_lo),
            ("desk's target high", cty.plan_hi),
            ("horse price low", cty.pk_lo),
            ("horse price high", cty.pk_hi.0),
            ("maker's markup", cty.mu_lo.0),
            ("baskets", cty.baskets),
            ("utilisation", cty.util_lo),
            ("heads at the end", cty.end[1]),
        ] {
            one(x, what, k);
        }
        assert!(
            cty.qr_lo.abs() <= 1e-9 && cty.qr_hi.abs() <= 1e-9,
            "{k}: quasi-rent"
        );
    }
    // A row a county, and every 13th tick of each in the series.
    assert_eq!(lr.table.lines().count(), 94);
    assert_eq!(lr.series.lines().count(), 1 + 93 * 8);
    let head: Vec<&str> = lr
        .table
        .lines()
        .next()
        .unwrap_or_default()
        .split(',')
        .collect();
    assert_eq!(head.len(), 42);
    assert!(lr.table.lines().all(|l| l.split(',').count() == 42));
}

#[test]
#[ignore = "the long run, scored: 1 to 2 minutes in release; scripts/gate.sh runs it by name"]
fn demo_v2_runs_to_1901() {
    // WORLD-V2 §11.4 (D2.4): the 93-node run from genesis through the first tick of 1901, scored
    // by `long_run` as the registration's mirror scores it. It prints a line a county, and with
    // DEMO_V2_LONG_OUT set it writes long-run.csv and series.csv there for the scorer
    // (docs/demo/v2/results/). It asserts what must hold (every event fired, the ledger closed,
    // the pin's hashes, the plans' params at the end) and the registration's refutation for the
    // long run: no dead tick in any county. Every other line is the scorer's (§11.5's E5).
    use rustyecon_engine::prelude::ParamId;
    let c = compiled();
    let s = stage();
    let t = tape();
    let cl = clock(&c.world);
    let end = cl.tick_of(c.world.end).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(end, 7_851);
    let until = end + 1;
    let lr = long_run(c, t, until);
    let mut all = Vec::new();
    for cty in &lr.counties {
        all.extend_from_slice(&cty.dh);
        println!(
            "{:<11} D-hat median {:>7.2} max {:>7.1}  dead {}  idle {}  withheld {}  short {}  \
             heads {:.3}-{:.3}  lowest {:.3} {}",
            cty.run.key,
            pct(&cty.dh, 0.5),
            cty.dh.iter().copied().fold(0.0, f64::max),
            cty.dead,
            cty.idle,
            cty.withheld,
            cty.short,
            cty.heads_lo.0,
            cty.heads_hi.0,
            cty.low.0,
            DEAD[cty.low.1].2
        );
    }
    let mut medians: Vec<f64> = lr.counties.iter().map(|x| pct(&x.dh, 0.5)).collect();
    medians.sort_by(f64::total_cmp);
    println!(
        "\n{} counties, {until} ticks, {} events fired ({} in the horizon's tick), largest \
         ledger margin {:e}: final hash 0x{:016x}, hash stream 0x{:016x}; the county medians' \
         median D-hat {:.2}, the county-ticks' {:.2}; {:.1} s",
        lr.counties.len(),
        lr.fired,
        lr.fired_at_end,
        lr.margin,
        lr.final_hash,
        lr.stream_hash,
        medians[medians.len() / 2],
        pct(&all, 0.5),
        lr.secs
    );
    if let Some(dir) = std::env::var_os("DEMO_V2_LONG_OUT") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        for (name, body) in [("long-run.csv", &lr.table), ("series.csv", &lr.series)] {
            std::fs::write(dir.join(name), body).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        println!("wrote {}", dir.display());
    }
    // Every event fired, the ledger closed every tick, and the run is the pinned one.
    assert_eq!(lr.fired, t.events.len());
    assert!(lr.fired_at_end > 0);
    assert!(lr.margin <= 1.0);
    assert_eq!(
        (lr.final_hash, lr.stream_hash),
        (FINAL_HASH, STREAM_HASH),
        "final 0x{:016x}, stream 0x{:016x}",
        lr.final_hash,
        lr.stream_hash
    );
    // At the end every county's params are its plan's instance in force from 1901-01-01, mapped.
    let w = lr.sim.world();
    let last = i64::from(c.world.end.y - c.world.start.y) * 12;
    for (cty, p) in lr.counties.iter().zip(&c.counties) {
        let want = stage_instance(&p.instance_at(last), &s.machine, &cl);
        for q in StageParam::ALL {
            let key = format!("{}.{}", cty.run.key, q.key(&s.machine.key));
            let id = w.id_of::<ParamId>(&key).unwrap_or_else(|| panic!("{key}"));
            let got = lr.sim.param(id).unwrap_or_else(|| panic!("{key}"));
            assert_eq!(got.to_bits(), want[q.index()].to_bits(), "{key}");
        }
    }
    // The registration's refutation for the long run (§4): a dead tick in any county.
    for cty in &lr.counties {
        assert_eq!(cty.dead, 0, "{}: {} dead ticks", cty.run.key, cty.dead);
    }
}
