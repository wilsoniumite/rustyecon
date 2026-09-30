//! The demo world, `worlds/demo-gb` (docs/demo/WORLD.md), and its compiled tape,
//! `tapes/demo-gb.ron` (D.2, 2026-09-27).
//!
//! The committed tape is exactly what the compiler writes from the committed tables; its nodes
//! are the atlas's regions; every basis carries the illustrative marker; the dials are the
//! probe's registered C2 and each county's genesis is the probe's own genesis rule at that
//! county's instance. The compiler refuses tables that break a rule, an abrupt history among
//! them. The long run, `demo_runs_to_1901` (ignored, run by name in scripts/gate.sh), runs the
//! tape through the first tick of 1901, so every step fires, and scores every county every tick
//! against its moving oracle point.

use oracle::Eq1a;
use probe::harness::Target;
use probe::protocol::{LIVE_FLOOR, TOL_FLOOR};
use probe::setup::{Dials, Setup};
use rustyecon_core::{fnv1a_64, num, Basis, FlowPerYear};
use rustyecon_engine::prelude::{ActorId, ActorState, GoodId, NodeId, ParamId, Sim, Tape};
use rustyecon_engine::registry;
use rustyecon_worldgen::atlas::Atlas;
use rustyecon_worldgen::compile::{clock, genesis, solve as checked_solve, Compiled, MAX_YEAR};
use rustyecon_worldgen::lens::{value, Measure, Readings};
use rustyecon_worldgen::tables::{self, Instance, Param};
use rustyecon_worldgen::{compile, Tables};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const WORLD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/demo-gb");
const TAPE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes/demo-gb.ron");
const MARK: &str = "illustrative demo, 2026-09-27: ";

/// The final state hash of the run through 1901-01-01's tick (state tick 7,852), and FNV-1a 64
/// over every tick's hash as 8 little-endian bytes, recorded at D.4 on WSL and Windows.
const FINAL_HASH: u64 = 0xfad8_80fe_08d0_6645;
const STREAM_HASH: u64 = 0xdb63_cc96_f769_fb3e;

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

fn atlas() -> Atlas {
    Atlas::gb().unwrap_or_else(|e| panic!("{e}"))
}

fn compiled() -> &'static Compiled {
    static C: OnceLock<Compiled> = OnceLock::new();
    C.get_or_init(|| compile(&tables(), &atlas()).unwrap_or_else(|e| panic!("{e}")))
}

fn tape_text() -> String {
    std::fs::read_to_string(TAPE).unwrap_or_else(|e| panic!("{TAPE}: {e}"))
}

fn tape() -> &'static Tape {
    static T: OnceLock<Tape> = OnceLock::new();
    T.get_or_init(|| Tape::from_ron(&tape_text()).unwrap_or_else(|e| panic!("{e}")))
}

#[test]
fn demo_tape_is_its_compilers_output() {
    // `rustyecon worldgen worlds/demo-gb --out tapes/demo-gb.ron` rewrites it.
    let committed = tape_text();
    let fresh = &compiled().tape;
    if committed != *fresh {
        let line = committed
            .lines()
            .zip(fresh.lines())
            .position(|(a, b)| a != b)
            .map_or(committed.lines().count().min(fresh.lines().count()), |i| i);
        panic!(
            "tapes/demo-gb.ron differs from the compiler's output from line {}: rerun `rustyecon \
             worldgen worlds/demo-gb --out tapes/demo-gb.ron`",
            line + 1
        );
    }
    assert!(!committed.contains('\r'));
}

#[test]
fn atlas_keys_equal_node_keys() {
    // docs/GUI.md §6 and §7.3: an atlas whose keys differ from the tape's node keys is refused.
    let a = atlas();
    let atlas_keys: Vec<&str> = a.regions.iter().map(|r| r.key.as_str()).collect();
    let mut nodes: Vec<&str> = tape().nodes.iter().map(|n| n.key.as_str()).collect();
    nodes.sort_unstable();
    assert_eq!(nodes, atlas_keys);
    assert_eq!(nodes.len(), 93);
    // Great Britain is 87 of them, Northern Ireland the other six.
    let ni = a
        .regions
        .iter()
        .filter(|r| r.country == rustyecon_worldgen::atlas::Country::NorthernIreland)
        .count();
    assert_eq!(ni, 6);
}

#[test]
fn every_basis_is_illustrative() {
    // R4, R5: nothing from this world can be scored or cited. The name carries the marker and
    // every basis on the tape is Assumed with it, the inline numbers' included.
    let t = tape();
    assert!(t.header.name.contains("[illustrative]"));
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
}

#[test]
fn the_tables_check_every_county_at_every_step() {
    // The compiler solves every county at genesis and after every step (docs/demo/WORLD.md
    // §3.3): Interior with one crossing, funded, participation short of saturation, and no date
    // moving the oracle's relative prices and technique by more than max_step (O14).
    let c = compiled();
    let s = &c.summary;
    println!("{s:#?}");
    assert_eq!(s.counties, 93);
    assert_eq!(s.events, 30_078);
    assert_eq!(s.step_dates, 25_480);
    assert!(s.min_funding.0 > 0.1, "{:?}", s.min_funding);
    assert!(s.min_participation.0 > 0.5 && s.max_participation.0 < 0.9);
    assert!(s.min_x.0 > 0.6 && s.max_x.0 < 0.95);
    assert!(s.max_step_prices.0 <= c.world.max_step);
    assert!(s.max_step_quantities.0 <= c.world.max_step);
    assert!(c.world.max_step <= 0.03 && c.world.max_step <= tables::MAX_STEP_CEILING);
    assert!(c.world.step_log <= tables::STEP_LOG_CEILING);
    // Over any trailing year, too (O14): the history's largest are well inside the bound.
    assert!(s.max_year_prices.0 <= MAX_YEAR && s.max_year_quantities.0 <= MAX_YEAR);
    assert!(
        s.max_year_prices.0.max(s.max_year_quantities.0) < 0.06,
        "{s:?}"
    );
    // The events on the tape are the plans' steps, one each.
    assert_eq!(tape().events.len(), s.events);
    // Lenses: 25, each with a fixed domain.
    assert_eq!(c.lenses.len(), 25);
}

#[test]
fn dials_are_the_probes_registered_c2() {
    let w = &compiled().world;
    let d = Dials::registered();
    for (key, value) in [
        ("rate.labour", d.rate_labour),
        ("rate.land", d.rate_land),
        ("rate.mach", d.rate_mach),
        ("rate.good", d.rate_good),
        ("price.ema_tc", d.ema_tc),
        ("adjust.technique", d.adjust_technique),
        ("spend.workers", d.spend_workers),
        ("spend.provider", d.spend_provider),
        ("buffer.desk.good.cash", d.turnover_good),
        ("buffer.desk.mach.cash", d.turnover_mach),
        ("tilt.desk.good", d.tilt_good),
        ("tilt.desk.mach", d.tilt_mach),
        ("ledger.rel_flow", 1e-12),
        ("ledger.rel_stock", 1e-11),
    ] {
        assert_eq!(w.dial(key), Some(value), "{key}");
    }
    // The compiler's own list of C2, which it refuses any other dial against, is the probe's.
    for (key, value) in tables::C2 {
        let theirs = match key {
            "rate.labour" => d.rate_labour,
            "rate.land" => d.rate_land,
            "rate.mach" => d.rate_mach,
            "rate.good" => d.rate_good,
            "adjust.technique" => d.adjust_technique,
            "spend.workers" => d.spend_workers,
            "spend.provider" => d.spend_provider,
            "buffer.desk.good.cash" => d.turnover_good,
            "buffer.desk.mach.cash" => d.turnover_mach,
            "tilt.desk.good" => d.tilt_good,
            "tilt.desk.mach" => d.tilt_mach,
            other => panic!("{other} is not one of the probe's dials"),
        };
        assert_eq!(value, theirs, "{key}");
    }
    assert_eq!(w.ticks_per_year, 52);
    assert_eq!(w.one_sided, "Saturate");
}

/// A county's readings at its oracle point `e` under instance `inst`, as a run at rest would
/// record them: prices with r = 1, every market cleared at its equilibrium volume, the desk's
/// share 1 − x*, the provider paying all it owes, nothing rationed, every market trading.
fn at_rest(e: &Eq1a, inst: &Instance) -> Readings {
    let c = clock(&compiled().world);
    let n = c.flow(FlowPerYear(inst[Param::Workers.index()]));
    let t = c.flow(FlowPerYear(inst[Param::Land.index()]));
    Readings {
        prices: [Some(e.v), Some(1.0), Some(e.p_m), Some(e.p)],
        cleared: [Some(e.n_a), Some(t), Some(e.k), Some(e.y)],
        params: inst.map(Some),
        n_tick: Some(n),
        share: Some(e.one_minus_x_star),
        due: Some(e.support_cost),
        paid: Some(e.support_cost),
        rationing: vec![(1.0, 1.0)],
        traded: vec![true],
    }
}

#[test]
fn lens_domains_hold_the_oracle_range() {
    // docs/demo/WORLD.md §6: each lens's domain is fixed before any run, from the oracle's range
    // over every county at genesis and after every step, with a margin for the transients. Here
    // that range is computed through the lenses' own measures (`lens::value`) at each county's
    // oracle point, and must lie inside the domain; the margins are printed. The oracle lenses
    // wait for crates/observe; shortfall, rationing and no.trade are 0 at every rest point.
    let lenses = rustyecon_worldgen::lens::demo_gb().unwrap_or_else(|e| panic!("{e}"));
    let mut out = Vec::new();
    for l in &lenses {
        let Some(m) = Measure::of(&l.key) else {
            panic!("{}: no measure", l.key)
        };
        if m.level().needs_observe() {
            continue;
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut at_lo, mut at_hi) = (String::new(), String::new());
        for plan in &compiled().counties {
            let first = at_rest(&plan.point, &plan.county.genesis);
            let genesis = (0, plan.point.clone());
            for (month, e) in std::iter::once(&genesis).chain(&plan.points) {
                let x = at_rest(e, &plan.instance_at(*month));
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
        // The margin each side, as a share of the domain's span (in log for a log scale).
        let span = |x: f64, y: f64| match l.scale {
            tables::Scale::SequentialLog => (num::ln(y) - num::ln(x)) / (num::ln(b) - num::ln(a)),
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
        out.push(line);
    }
    assert_eq!(out.len(), 23, "every lens but the two oracle lenses");
}

#[test]
fn genesis_is_the_probes_rule_at_each_county() {
    // Each county's genesis is probe::setup::genesis at its instance, with every price and coin
    // divided by p/r: the same oracle point bit for bit, the same ratios to rounding.
    let c = compiled();
    for plan in &c.counties {
        let g = &plan.county.genesis;
        let v = |p: Param| g[p.index()];
        let mut s = Setup::registered(52);
        s.instance = probe::setup::Instance {
            workers: v(Param::Workers),
            land: v(Param::Land),
            space: v(Param::Space),
            a: v(Param::A),
            lam: v(Param::Lam),
            b: v(Param::B),
            eta: v(Param::Eta),
            g0: v(Param::G0),
            g1: v(Param::G1),
            k: v(Param::K),
            chi_max: v(Param::ChiMax),
        };
        s.genesis_b = v(Param::B);
        let theirs = probe::setup::genesis(&s).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(theirs.point, plan.point, "{}", plan.county.key);
        let ours = genesis(&c.world, g, &plan.point);
        let p = plan.point.p;
        let close = |a: f64, b: f64| ((a * p) / b - 1.0).abs() < 1e-14;
        for i in 0..4 {
            assert!(
                close(ours.prices[i], theirs.prices[i]),
                "{} price {i}",
                plan.county.key
            );
            assert!(
                close(ours.coin[i], theirs.coin[i]),
                "{} coin {i}",
                plan.county.key
            );
        }
        assert_eq!(ours.prices[3], 1.0);
    }
}

/// Compile with one table's text changed, and return the error.
fn refused(file: &str, edit: impl Fn(&str) -> String) -> String {
    let mut t = tables();
    let slot = match file {
        "world.csv" => &mut t.world,
        "counties.csv" => &mut t.counties,
        "regions.csv" => &mut t.regions,
        "history.csv" => &mut t.history,
        "lenses.csv" => &mut t.lenses,
        _ => unreachable!(),
    };
    *slot = edit(slot);
    match compile(&t, &atlas()) {
        Ok(_) => panic!("{file}: the edit compiled"),
        Err(e) => e.to_string(),
    }
}

fn without_line(text: &str, starts: &str) -> String {
    text.lines()
        .filter(|l| !l.starts_with(starts))
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn the_compiler_refuses_bad_tables() {
    let has = |e: &str, what: &str| assert!(e.contains(what), "{e}");
    // Every region of the atlas has a row, and every row is a region.
    has(
        &refused("regions.csv", |t| without_line(t, "county.lan,")),
        "county.lan (Lancashire) has no row",
    );
    has(
        &refused("counties.csv", |t| without_line(t, "county.ant,")),
        "county.ant (Antrim) has no row",
    );
    has(
        &refused("regions.csv", |t| {
            t.replace("county.bdf,BDF", "county.xyz,BDF")
        }),
        "`county.xyz` is not a region of the atlas",
    );
    has(
        &refused("regions.csv", |t| t.replacen(",Lancashire,", ",Lancs,", 1)),
        "name is `Lancashire` in the atlas",
    );
    has(
        &refused("counties.csv", |t| {
            t.replacen(
                "Bedfordshire,england,1202.5",
                "Bedfordshire,england,1302.5",
                1,
            )
        }),
        "where the atlas measures",
    );
    // Units and ranges.
    has(
        &refused("world.csv", |t| {
            t.replace("rate.labour,5.2,RatePerYear", "rate.labour,5.2,Years")
        }),
        "not `RatePerYear`",
    );
    has(
        &refused("world.csv", |t| {
            t.replace("name,demo-gb [illustrative]", "name,demo-gb")
        }),
        "[illustrative]",
    );
    has(
        &refused("regions.csv", |t| {
            t.replacen(",0.3,0.03,0.8,", ",1.3,0.03,0.8,", 1)
        }),
        "out of range",
    );
    // Reserved columns: more categories wait for the many-market roles.
    has(
        &refused("regions.csv", |t| {
            t.replacen(",good,horse,,", ",good food,horse,,", 1)
        }),
        "WORLD.md §7",
    );
    // A path that does not start where the value is.
    has(
        &refused("history.csv", |t| {
            t.replace(
                "population.bdf.1801,1801,1851,county.bdf,workers,path,131.04,",
                "population.bdf.1801,1801,1851,county.bdf,workers,path,141.04,",
            )
        }),
        "where the value is",
    );
    // An abrupt history: coal's mineral rent tripled in a year moves the oracle too far (O14):
    // over its first year in Antrim, before any one date passes max_step.
    has(
        &refused("history.csv", |t| {
            t.replace(
                "coal.mineral.1,1750,1830,tag:coal,land,scale,1,1.3,",
                "coal.mineral.1,1750,1751,tag:coal,land,scale,1,3,",
            )
        }),
        "county.ant at 1750-11-01: the year to this date moves the oracle's quantities",
    );
    // Bedfordshire's η halved within 1800 moves its relative prices too far at one date.
    has(
        &refused("history.csv", |t| {
            format!("{t}verify.eta,1800,1801,county.bdf,eta,scale,1,0.5,1,,test,eta halves\n")
        }),
        "county.bdf at 1800-02-01: one date moves the oracle's relative prices and technique \
         (1 − x*, w/r, p_m/r, p/r) by 0.0652 in log, above max_step 0.03",
    );
    // Abrupt in quantities with prices still: Middlesex's N and T doubled within 1800 move no
    // relative price, but its output 0.064 in log a month (verify-world-r1's scenario a).
    let append = |rows: &'static str| move |t: &str| format!("{t}{rows}");
    has(
        &refused(
            "history.csv",
            append(
                "verify.n,1800,1801,county.mdx,workers,scale,1,2,1,,test,N doubles in a year\n\
                 verify.t,1800,1801,county.mdx,land,scale,1,2,1,,test,T doubles in a year\n",
            ),
        ),
        "moves the oracle's quantities (Y, K, N_a) by 0.0637 in log, above max_step",
    );
    // Abrupt over a year in steps each under max_step: Middlesex's T ×1.3 within 1800 moves
    // it 0.29 in a year (scenario b, which ran to a trough of 0.78 and D̂ 248).
    has(
        &refused(
            "history.csv",
            append("verify.t,1800,1801,county.mdx,land,scale,1,1.3,1,,test,T x1.3 in a year\n"),
        ),
        "the year to this date moves the oracle's",
    );
    // The bounds themselves are bounded: max_step 5 with the abrupt coal above (scenario c,
    // 323 dead ticks), and a step_log coarser than any stepping run.
    has(
        &refused("world.csv", |t| t.replace("max_step,0.03,", "max_step,5,")),
        "max_step 5 is above its ceiling 0.03",
    );
    has(
        &refused("world.csv", |t| {
            t.replace("step_log,0.01,", "step_log,0.05,")
        }),
        "step_log 0.05 is above its ceiling 0.02",
    );
    // The clock off the probe's registered 52 ticks a year (O39, D2.1): at 12 and at 4 a year
    // the tables are otherwise sound, so only the clock's check refuses them.
    for n in [12, 4] {
        has(
            &refused("world.csv", |t| {
                t.replace("ticks_per_year,52,", &format!("ticks_per_year,{n},"))
            }),
            &format!("ticks_per_year is {n}, not 52"),
        );
    }
    // A dial off the probe's registered C2.
    has(
        &refused("world.csv", |t| {
            t.replace("rate.labour,5.2,RatePerYear", "rate.labour,52,RatePerYear")
        }),
        "rate.labour is 52, not the probe's registered C2 value 5.2",
    );
    // The marker dropped from both cells does not make an unmarked tape of the same tables.
    has(
        &refused("world.csv", |t| {
            t.replace("name,demo-gb [illustrative]", "name,demo-gb")
                .replace(
                    "basis,\"illustrative demo, 2026-09-27\"",
                    "basis,\"demo, 2026-09-27\"",
                )
        }),
        "illustrative worlds only",
    );
    // A textile ramp that would start after it ends.
    has(
        &refused("regions.csv", |t| {
            t.replacen(",cotton,1768,", ",cotton,1890,", 1)
        }),
        "not before the ramp's end",
    );
    // A lens whose domain does not fit its scale.
    has(
        &refused("lenses.csv", |t| {
            t.replace("sequential-log,,0.4,2.4", "sequential-log,,0,2.4")
        }),
        "does not fit",
    );
    // Line endings.
    has(
        &refused("history.csv", |t| t.replace('\n', "\r\n")),
        "LF only",
    );
}

/// One county's ids and running record in the long run.
struct County {
    key: String,
    params: [ParamId; 11],
    desk_good: ActorId,
    desk_mach: ActorId,
    provider: ActorId,
    /// Market line indices, in the probe's order: labour, land, mach, good.
    lines: [usize; 4],
    inst: Instance,
    target: Target,
    dhat: Vec<f64>,
    dead: u64,
    longest_dead: u64,
    run_dead: u64,
    short: u64,
    trough: f64,
}

fn solve(inst: &Instance, sim: &Sim) -> (Eq1a, f64) {
    let c = sim.world().clock;
    let e = checked_solve("run", inst, &c).unwrap_or_else(|e| panic!("{e}"));
    let land = c.flow(FlowPerYear(inst[Param::Land.index()]));
    (e, land)
}

#[test]
#[ignore = "the long run, 15 to 30 s in release: scripts/gate.sh runs it by name"]
fn demo_runs_to_1901() {
    let t0 = std::time::Instant::now();
    let tape = tape();
    let mut sim = Sim::new(tape).unwrap_or_else(|e| panic!("{e}"));
    let w = sim.world().clone();
    let c = clock(&compiled().world);
    let end = c
        .tick_of(compiled().world.end)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(end, 7_851);
    // Through the horizon's own tick: the steps dated 1901-01-01 fire in report tick 7,851, so
    // the run to state tick 7,852 fires every event on the tape.
    let until = end + 1;
    let good = |k: &str| w.id_of::<GoodId>(k).unwrap_or_else(|| panic!("{k}"));
    let goods = [good("labour"), good("land"), good("mach"), good("good")];
    let mut counties: Vec<County> = Vec::new();
    for plan in &compiled().counties {
        let k = &plan.county.key;
        let actor = |r: &str| {
            w.id_of::<ActorId>(&format!("{k}.{r}"))
                .unwrap_or_else(|| panic!("{k}.{r}"))
        };
        let params = Param::ALL.map(|p| {
            w.id_of::<ParamId>(&format!("{k}.{}", p.column()))
                .unwrap_or_else(|| panic!("{k}.{}", p.column()))
        });
        let inst = plan.county.genesis;
        let (e, land) = solve(&inst, &sim);
        counties.push(County {
            key: k.clone(),
            params,
            desk_good: actor("desk.good"),
            desk_mach: actor("desk.mach"),
            provider: actor("provider"),
            lines: [0; 4],
            inst,
            target: Target::of(&e, land),
            dhat: Vec::with_capacity(until as usize),
            dead: 0,
            longest_dead: 0,
            run_dead: 0,
            short: 0,
            trough: f64::INFINITY,
        });
    }
    let mut stream = Vec::with_capacity(8 * until as usize);
    let mut solves = 0u64;
    let mut engine_secs = 0.0;
    let mut fired = 0usize;
    let mut fired_at_end = 0usize;
    for tick in 0..until {
        let t1 = std::time::Instant::now();
        let r = sim.step().unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        engine_secs += t1.elapsed().as_secs_f64();
        stream.extend_from_slice(&r.hash.to_le_bytes());
        fired += r.events.len();
        if tick == end {
            fired_at_end = r.events.len();
        }
        if tick == 0 {
            // The report lists every (node, good) market in (node, good) order.
            let index: BTreeMap<(NodeId, GoodId), usize> = r
                .markets
                .iter()
                .enumerate()
                .map(|(i, l)| ((l.node, l.good), i))
                .collect();
            assert_eq!(index.len(), 4 * counties.len());
            for cty in &mut counties {
                let node = w.id_of::<NodeId>(&cty.key).unwrap_or_else(|| panic!());
                cty.lines = goods.map(|g| index[&(node, g)]);
            }
        }
        for cty in &mut counties {
            // The params in force this tick, as the run holds them: a step changed some of
            // them in phase 0, so the target moves with them (R13: the oracle is solved here).
            let mut inst = cty.inst;
            for (i, &p) in cty.params.iter().enumerate() {
                inst[i] = sim.param(p).unwrap_or_else(|| panic!("{} param", cty.key));
            }
            if inst != cty.inst {
                let (e, land) = solve(&inst, &sim);
                cty.target = Target::of(&e, land);
                cty.inst = inst;
                solves += 1;
            }
            let l = cty.lines.map(|i| &r.markets[i]);
            let (used, q_good) = match sim.actor_state(cty.desk_good) {
                Some(ActorState::GoodDesk(s)) => (s.used, s.output),
                _ => panic!("{}: not a good desk", cty.key),
            };
            let q_mach = match sim.actor_state(cty.desk_mach) {
                Some(ActorState::MachDesk(s)) => s.output,
                _ => panic!("{}: not a machine desk", cty.key),
            };
            if let Some(ActorState::Provider(p)) = sim.actor_state(cty.provider) {
                if p.paid < p.due * (1.0 - 1e-9) {
                    cty.short += 1;
                }
            }
            let [lw, lr, lm, lg] = l;
            let obs = [
                lw.price / lr.price,
                lm.price / lr.price,
                lg.price / lr.price,
                used,
                lw.cleared,
                lr.cleared,
                lm.cleared,
                lg.cleared,
                q_good,
                q_mach,
            ];
            let tg = &cty.target;
            let d = (0..10)
                .map(|i| {
                    if obs[i] > 0.0 && obs[i].is_finite() {
                        num::ln(obs[i] / tg.obs[i]).abs()
                    } else {
                        f64::INFINITY
                    }
                })
                .fold(0.0, f64::max)
                / TOL_FLOOR;
            cty.dhat.push(d);
            let mut dead = false;
            for (m, line) in l.iter().enumerate() {
                let share = line.cleared / tg.volume(m);
                cty.trough = cty.trough.min(share);
                dead |= !line.trades() || share < LIVE_FLOOR;
            }
            if dead {
                cty.dead += 1;
                cty.run_dead += 1;
                cty.longest_dead = cty.longest_dead.max(cty.run_dead);
            } else {
                cty.run_dead = 0;
            }
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    let final_hash = sim.hash();
    let stream_hash = fnv1a_64(&stream);
    let median = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let mut medians = Vec::new();
    let mut worst = (0.0, String::new());
    let mut all = Vec::new();
    for cty in &mut counties {
        let max = cty.dhat.iter().copied().fold(0.0, f64::max);
        if max > worst.0 {
            worst = (max, cty.key.clone());
        }
        all.extend_from_slice(&cty.dhat);
        let m = median(&mut cty.dhat);
        medians.push((m, cty.key.clone()));
        println!(
            "{:<11} median D-hat {m:>6.2}  max {max:>6.1}  dead {}  shortfall {}  trough {:.3}",
            cty.key, cty.dead, cty.short, cty.trough
        );
    }
    medians.sort_by(|a, b| a.0.total_cmp(&b.0));
    let p90 = {
        all.sort_by(f64::total_cmp);
        all[all.len() * 9 / 10]
    };
    let overall = all[all.len() / 2];
    println!(
        "\n{} counties, {until} ticks, {fired} events fired ({fired_at_end} in the horizon's \
         tick): final hash 0x{final_hash:016x}, hash stream 0x{stream_hash:016x}",
        counties.len()
    );
    println!(
        "D-hat (multiples of the probe's 1e-3 in log) against each county's moving oracle point: \
         median over county-ticks {overall:.2}, p90 {p90:.2}, the median county's median {:.2} \
         ({}), the highest county median {:.2} ({}), the largest {:.1} ({})",
        medians[medians.len() / 2].0,
        medians[medians.len() / 2].1,
        medians[medians.len() - 1].0,
        medians[medians.len() - 1].1,
        worst.0,
        worst.1
    );
    println!(
        "{solves} oracle solves for the targets; {:.1} s in all, {engine_secs:.1} s stepping: \
         {:.0} ticks a second, {:.1} model years a second",
        secs,
        until as f64 / engine_secs,
        until as f64 / engine_secs / 52.0
    );
    // Every event on the tape fired, the horizon's among them; after the last tick run (the
    // first of 1901), every county's params are its plan's instance in force from 1901-01-01,
    // every ramp at its end.
    assert_eq!(fired, tape.events.len());
    assert!(
        fired_at_end > 0,
        "the steps dated 1901-01-01 fire in tick {end}"
    );
    let last_month = i64::from(compiled().world.end.y - compiled().world.start.y) * 12;
    for (cty, plan) in counties.iter().zip(&compiled().counties) {
        assert_eq!(cty.key, plan.county.key);
        assert_eq!(cty.inst, plan.instance_at(last_month), "{}", cty.key);
        assert!(plan.steps.iter().all(|s| s.month <= last_month));
    }
    // Liveness: no county's market dies, and no provider falls short, at any tick.
    for cty in &counties {
        assert_eq!(cty.dead, 0, "{}: {} dead ticks", cty.key, cty.dead);
        assert_eq!(cty.longest_dead, 0);
        assert_eq!(cty.short, 0, "{}: {} ticks short", cty.key, cty.short);
        assert!(cty.trough > 0.9, "{}: trough {}", cty.key, cty.trough);
    }
    // Near the moving target: every county's median gap is within 5% in log, and no tick's
    // gap passes 10%.
    assert!(medians[medians.len() - 1].0 < 50.0, "{medians:?}");
    assert!(worst.0 < 100.0, "{worst:?}");
    // Determinism: the same hashes on every machine.
    assert_eq!(
        (final_hash, stream_hash),
        (FINAL_HASH, STREAM_HASH),
        "final 0x{final_hash:016x}, stream 0x{stream_hash:016x}"
    );
}
