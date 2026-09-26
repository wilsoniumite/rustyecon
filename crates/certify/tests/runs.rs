//! `certify` on real runs (docs/CERTIFY.md §3, §6–§8, §13): the gate world, the probe's
//! Appendix B world, and certify's testdata, written by `appb-tape` (§13): `appb-bcycle.ron`
//! (four dated changes of b), `appb-freeze.ron` (desk turnover ×16 from w×1.05, converged and
//! frozen at an unstable point), `appb-buffer16.ron` (desk turnover ×16 from the exact genesis,
//! frozen there from the first tick; S2.5) and `appb-july.ron` (July's step rule from w×2, the
//! negative control).
//!
//! The bars (amended at S2.5). The gate's tests apply the gate's registered batteries, and the
//! appb tests appb's registered Runaway, Settles and Kick (the Kick's horizon shortened where a
//! test says so), each read from `criteria/<tape>-2026-09-26.ron`, so a test and the file cannot
//! drift apart. Until S2.5 this header said that no test applied a registered bar but appb's
//! kick. That was not so: from S2.3, before the criteria were committed at `1ee9b80`, the gate's
//! tests and the appb tests wrote the same values into their own criteria text (CERTIFY's S2.5
//! amendment names them). No value was tuned to a run: each bar's basis predates the session
//! (ENGINE §10, July's `invariants.rs`, PROBE-SPEC §4.5, REPORT §5). Test bars that are not
//! registered ones: segments of 20 years on appb runs other than appb's own, below bcycle's
//! 1,500 ticks, and runs shorter than a registered `until`. The constants below restate a
//! registered number where an assertion needs it, and `the_bars_are_the_registered_ones`
//! checks that they agree.

mod common;

use certify::kick::{deferred_tape, kick_segment, KICK_FACTOR};
use certify::{
    battery, certify, nonfinite_paths, tape_hash, BatteryId, BatterySpec, Certificate,
    CertificateError, Certified, CertifyError, Limit, RawCertificate, Reading, Scoring, Verdict,
};
use common::*;
use rustyecon_core::tape::raw::RawAct;
use rustyecon_engine::prelude::{Provenance, SideTag, Sim, Tape, TickReport};
use rustyecon_engine::rustyecon_agents::ActorState;

/// L: 20,000 ticks (RULES §3).
const L: u64 = 20_000;
/// One L of kick at 52 a year: 20,020 ticks, appb's registered horizon.
const HORIZON: f64 = 385.0;
/// appb's registered kick: its size, and the largest gain in the tail and at the peak.
const KICK: f64 = 1e-9;
const MAX_GAIN: f64 = 1e-3;
const MAX_PEAK: f64 = 1e6;
/// The registered runaway bounds of appb and the gate.
const APPB_BOUND: f64 = 1e6;
const GATE_BOUND: f64 = 1e3;
/// Both files' `rationed_below`.
const RATIONED: f64 = 1e-9;

fn run(t: &Tape, name: &str, until: u64, min_segment: f64, bats: &[String]) -> Certified {
    let text = criteria_text(
        name,
        tape_hash(t),
        &date_of(t, until),
        min_segment,
        bats,
        RATIONED,
    );
    let c = criteria(name, &text);
    certify(
        t,
        Scoring::Criteria {
            criteria: &c,
            file: &file(name),
        },
        &build(),
        &mut |_| {},
    )
    .unwrap_or_else(|e| panic!("certify runs: {e}"))
}

/// The gate's registered batteries, read from its criteria file.
fn gate_batteries() -> Vec<String> {
    registered("gate").batteries.iter().map(spec_ron).collect()
}

/// appb's registered Conservation, Determinism, Runaway, Settles and Kick, read from its
/// criteria file, with the Kick's horizon `horizon` years.
fn appb_batteries(horizon: f64) -> Vec<String> {
    let c = registered("appb");
    [
        BatteryId::Conservation,
        BatteryId::Determinism,
        BatteryId::Runaway,
        BatteryId::Settles,
        BatteryId::Kick,
    ]
    .iter()
    .map(|id| {
        let mut b = c.battery(*id).expect("registered").clone();
        if let BatterySpec::Kick { horizon: h, .. } = &mut b {
            h.value = horizon;
        }
        spec_ron(&b)
    })
    .collect()
}

#[test]
fn the_bars_are_the_registered_ones() {
    // The constants this file's assertions use restate registered numbers; they agree with the
    // files, and the helpers above read the files themselves.
    let (gate, appb) = (registered("gate"), registered("appb"));
    let bound = |c: &certify::Criteria| match c.battery(BatteryId::Runaway) {
        Some(BatterySpec::Runaway { bound }) => bound.value,
        _ => panic!("Runaway is registered"),
    };
    assert_eq!((bound(&gate), bound(&appb)), (GATE_BOUND, APPB_BOUND));
    match appb.battery(BatteryId::Kick) {
        Some(BatterySpec::Kick {
            size,
            horizon,
            max_gain,
            max_peak,
            ..
        }) => assert_eq!(
            (size.value, horizon.value, max_gain.value, max_peak.value),
            (KICK, HORIZON, MAX_GAIN, MAX_PEAK)
        ),
        _ => panic!("Kick is registered"),
    }
    for c in [&gate, &appb] {
        assert_eq!(c.reports.rationed_below.value, RATIONED);
        assert_eq!(c.allowed_price_shocks(), 0);
    }
}

fn readings<'a>(c: &'a Certificate, id: BatteryId, name: &str) -> Vec<&'a Reading> {
    c.battery(id)
        .unwrap_or_else(|| panic!("{id} ran"))
        .readings
        .iter()
        .filter(|r| r.name == name)
        .collect()
}

#[test]
fn unscored_run_never_passes() {
    // N4: July certified a run with no criteria PASS (`v2p3: runner.rs:473-478`). A run with no
    // criteria, and one scored against criteria registered for another tape, are UNSCORED;
    // the same criteria registered for this tape give PASS.
    let gate = tape(GATE);
    let unscored = certify(
        &gate,
        Scoring::Unscored { until: 2080 },
        &build(),
        &mut |_| {},
    )
    .unwrap()
    .certificate;
    assert_eq!(unscored.verdict(), Verdict::Unscored);
    assert_eq!(unscored.criteria(), None);
    assert!(unscored.failures()[0].starts_with("unscored"));
    let bats = gate_batteries();
    let other = criteria_text(
        "gate",
        tape_hash(&tape(APPB)),
        &date_of(&gate, 2080),
        1.0,
        &bats,
        RATIONED,
    );
    let other = criteria("gate", &other);
    let c = certify(
        &gate,
        Scoring::Criteria {
            criteria: &other,
            file: &file("gate"),
        },
        &build(),
        &mut |_| {},
    )
    .unwrap()
    .certificate;
    assert!(
        c.batteries().iter().all(|b| b.pass),
        "every battery ran and passed"
    );
    assert_eq!(c.verdict(), Verdict::Unscored);
    assert!(c.failures()[0].contains("registered for tape_hash"));
    let own = run(&gate, "gate", 2080, 1.0, &bats).certificate;
    assert_eq!(own.verdict(), Verdict::Pass, "{:?}", own.failures());
}

#[test]
fn certify_reads_the_whole_run() {
    // certify hands each battery the whole run (the verification's wiring minors R7, W1, W3 and
    // W6, which only the byte equality of the committed certificates caught): Trades scores
    // all 40 yearly windows, Balance every market in every segment, Determinism every resume,
    // and Conservation every tick, whose largest margin is the run's, not its first tick's.
    let gate = tape(GATE);
    let c = registered("gate");
    let mut margins = Vec::new();
    let out = certify(
        &gate,
        Scoring::Criteria {
            criteria: &c,
            file: &file("gate"),
        },
        &build(),
        &mut |r| margins.push(r.audit.max_margin),
    )
    .unwrap()
    .certificate;
    assert_eq!(out.verdict(), Verdict::Pass, "{:?}", out.failures());
    let one = |name: &str| {
        let r = readings(&out, battery_of(name), name);
        assert_eq!(r.len(), 1, "{name}");
        r[0].value
    };
    assert_eq!(one("trades.windows"), 40.0);
    let markets = sim(&gate).world().markets().count();
    assert_eq!(markets, 6);
    assert_eq!(
        readings(&out, BatteryId::Trades, "trades.silent_windows").len(),
        markets
    );
    let segments = out.segments().len();
    assert_eq!(segments, 5);
    assert_eq!(
        readings(&out, BatteryId::Balance, "balance.samples").len(),
        markets * segments
    );
    let resumes: Vec<&str> = readings(&out, BatteryId::Determinism, "determinism.resume")
        .iter()
        .map(|r| r.at.as_str())
        .collect();
    assert_eq!(resumes, ["tick 520", "tick 1040", "tick 1560"]);
    assert_eq!(margins.len(), 2080);
    let largest = margins.iter().copied().fold(0.0, f64::max);
    assert!(
        largest > margins[0],
        "the largest margin is not the first tick's"
    );
    assert_eq!(one("conservation.max_margin"), largest);
}

/// The battery a reading's name belongs to.
fn battery_of(name: &str) -> BatteryId {
    match name.split('.').next() {
        Some("conservation") => BatteryId::Conservation,
        Some("determinism") => BatteryId::Determinism,
        Some("runaway") => BatteryId::Runaway,
        Some("trades") => BatteryId::Trades,
        Some("balance") => BatteryId::Balance,
        Some("settles") => BatteryId::Settles,
        Some("kick") => BatteryId::Kick,
        _ => panic!("{name}"),
    }
}

#[test]
fn a_listed_battery_that_did_not_run_fails() {
    // N4: a run stopped early does not run the batteries that read a whole run, and each of them
    // fails as "did not run"; the certificate says why the run stopped, first.
    let gate = tape(GATE);
    let mut t = gate.clone();
    t.events.push(
        ron::from_str(&format!(
            "(key: \"theft\", at: \"{}\", basis: Assumed(\"test\"), act: Burn(holder: \
             \"workers\", good: \"coin\", amount: Qty(1e12)))",
            date_of(&gate, 600)
        ))
        .unwrap(),
    );
    let mut bats = gate_batteries();
    bats.push(settles(0.5, 0.9, 0.01, 1e-4));
    bats.push(kick(KICK, 10.0, 0.1, MAX_GAIN, MAX_PEAK));
    let c = run(&t, "gate", 2080, 1.0, &bats).certificate;
    assert_eq!(c.verdict(), Verdict::Fail);
    assert_eq!(c.reached(), 600);
    assert!(
        c.failures()[0].starts_with("the run stopped at tick 600 of 2080: tick 600, phase 0"),
        "{:?}",
        c.failures()
    );
    for id in [
        BatteryId::Determinism,
        BatteryId::Trades,
        BatteryId::Balance,
        BatteryId::Settles,
        BatteryId::Kick,
    ] {
        let b = c.battery(id).expect("a listed battery has a result");
        assert!(!b.pass && b.notes == ["did not run"], "{id}: {:?}", b.notes);
    }
    // Conservation and Runaway read what the run made.
    assert!(!c.battery(BatteryId::Conservation).unwrap().pass);
    assert!(c.battery(BatteryId::Runaway).unwrap().pass);
}

#[test]
fn runaway_bound_is_relative() {
    // A12: the runaway bound is a ratio to the genesis price. The gate with every genesis price,
    // every coin holding and the pension ×2^40 runs the same economy in other units, exactly
    // (a power of two scales every product and sum), and gives bit-identical readings.
    let scale = 1_099_511_627_776.0;
    let gate = tape(GATE);
    let mut big = gate.clone();
    for p in &mut big.genesis.prices {
        p.price *= scale;
    }
    for h in &mut big.genesis.holdings {
        for (g, q) in &mut h.goods {
            if g.as_str() == "coin" {
                *q *= scale;
            }
        }
    }
    for r in &mut big.recurring {
        if let RawAct::Mint { qty, .. } = &mut r.act {
            *qty *= scale;
        }
    }
    let bats = gate_batteries();
    let a = run(&gate, "gate", 2080, 1.0, &bats).certificate;
    let b = run(&big, "gate", 2080, 1.0, &bats).certificate;
    assert_ne!(a.final_hash(), b.final_hash(), "another run");
    let ra = &a.battery(BatteryId::Runaway).unwrap().readings;
    let rb = &b.battery(BatteryId::Runaway).unwrap().readings;
    assert_eq!(ra.len(), 12, "two readings for each of six markets");
    for (x, y) in ra.iter().zip(rb) {
        assert_eq!((&x.name, &x.at), (&y.name, &y.at));
        assert_eq!(x.value.to_bits(), y.value.to_bits(), "{} {}", x.name, x.at);
        assert_eq!(x.bar, y.bar);
    }
    // The bound is the criteria's, not code's: the gate's prices, which reach about 16 times
    // genesis (ENGINE §10), fail a registered bound of 4.
    let tight: Vec<String> = bats
        .iter()
        .map(|b| {
            if b.starts_with("Runaway") {
                runaway(4.0)
            } else {
                b.clone()
            }
        })
        .collect();
    let t = run(&gate, "gate", 2080, 1.0, &tight).certificate;
    let r = t.battery(BatteryId::Runaway).unwrap();
    assert!(!r.pass && r.notes.len() == 6, "{:?}", r.notes);
    for r in ra {
        let want = if r.name == "runaway.max_ratio" {
            Limit::AtMost(GATE_BOUND)
        } else {
            Limit::AtLeast(1.0 / GATE_BOUND)
        };
        assert_eq!(r.bar, Some(want));
    }
}

#[test]
fn runaway_detector_catches_a_runaway() {
    // A12, REPORT §6: the negative control, July's step rule from w×2, runs away; the relative
    // bound fails it and names the good market, the first to leave (tick 415, as the probe found).
    let t = tape(JULY);
    let c = run(
        &t,
        "appb",
        1000,
        5.0,
        &[conservation(), determinism(&[0.5]), runaway(APPB_BOUND)],
    )
    .certificate;
    assert_eq!(c.verdict(), Verdict::Fail);
    let r = c.battery(BatteryId::Runaway).unwrap();
    assert!(!r.pass);
    assert!(
        r.notes[0].starts_with("home/good left") && r.notes[0].contains("at tick 415"),
        "{:?}",
        r.notes
    );
    let good = readings(&c, BatteryId::Runaway, "runaway.max_ratio")
        .into_iter()
        .find(|x| x.at == "home/good")
        .unwrap();
    assert!(good.value > APPB_BOUND);
}

#[test]
fn kick_check_passes_a_stable_rest() {
    // REPORT §3, §5: at C2's rest, a 1e-9 kick in ± each of appb's four prices decays to near its
    // rounding floor; the eight kicks pass, with their realized sizes near 1e-9.
    let c = run(&tape(APPB), "appb", L, 20.0, &appb_batteries(HORIZON)).certificate;
    assert_eq!(c.verdict(), Verdict::Pass, "{:?}", c.failures());
    let tails = readings(&c, BatteryId::Kick, "kick.gain_tail");
    assert_eq!(tails.len(), 8);
    assert!(tails.iter().all(|r| r.value <= MAX_GAIN), "{tails:?}");
    for s in readings(&c, BatteryId::Kick, "kick.size") {
        assert!((s.value / KICK - 1.0).abs() < 1e-6, "{s:?}");
    }
    let h = readings(&c, BatteryId::Kick, "kick.horizon");
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].value, 20_020.0);
}

#[test]
fn kick_check_fails_a_rounding_freeze() {
    // REPORT §5: at desk turnover ×16 the classifier scored every battery run CONVERGED, but the
    // point they froze at is unstable. The run is at rest, so Settles passes; six of its eight
    // kicks swing out ×3e9 through dead markets and back, so Kick fails, at the peak, though
    // each tail reads as decayed.
    let c = run(&tape(FREEZE), "appb", L, 20.0, &appb_batteries(HORIZON)).certificate;
    assert!(c.battery(BatteryId::Settles).unwrap().pass);
    let k = c.battery(BatteryId::Kick).unwrap();
    assert!(!k.pass);
    assert!(k.notes[0].contains("did not decay"), "{:?}", k.notes);
    assert_eq!(c.verdict(), Verdict::Fail);
    let peaks = readings(&c, BatteryId::Kick, "kick.gain_peak");
    let tails = readings(&c, BatteryId::Kick, "kick.gain_tail");
    let swung: Vec<_> = peaks
        .iter()
        .zip(&tails)
        .filter(|(p, t)| p.value > MAX_PEAK && t.value <= MAX_GAIN)
        .collect();
    assert!(swung.len() >= 6, "{peaks:?}");
    // The verification's E3: this certificate with Kick's pass flag flipped, its verdict set to
    // PASS and its failures emptied read back PASS, since the seal took the flags on trust. The
    // flag must now hold its readings, and six peaks of 2.9e9 do not hold a bar of 1e6.
    let mut raw: RawCertificate = ron::from_str(&c.to_ron()).unwrap();
    for b in &mut raw.batteries {
        b.pass = true;
    }
    raw.verdict = Verdict::Pass;
    raw.failures.clear();
    assert!(matches!(
        Certificate::try_from(raw),
        Err(CertificateError::EditedVerdict { .. })
    ));
}

#[test]
fn a_history_whose_segments_return_passes() {
    // REPORT §6 criterion 3: bcycle(1500,4) changes b four times, 1,500 ticks apart; each
    // segment returns before the next change, so Settles and Kick pass in all five, each kick
    // resumed from its segment's end for the horizon's 2,080 ticks, longer than the segment.
    let t = tape(BCYCLE);
    let c = run(&t, "appb", 7500, 20.0, &appb_batteries(40.0)).certificate;
    assert_eq!(c.verdict(), Verdict::Pass, "{:?}", c.failures());
    let segs: Vec<(u64, u64)> = c.segments().iter().map(|s| (s.from, s.to)).collect();
    assert_eq!(
        segs,
        [
            (0, 1500),
            (1500, 3000),
            (3000, 4500),
            (4500, 6000),
            (6000, 7500)
        ]
    );
    for (k, s) in c.segments().iter().enumerate().skip(1) {
        assert_eq!(
            s.opened_by.iter().map(|x| x.as_str()).collect::<Vec<_>>(),
            [format!("b.shock.{k}")]
        );
    }
    let dead = readings(&c, BatteryId::Settles, "settles.dead_f");
    assert_eq!(dead.len(), 5);
    for k in 0..5u32 {
        let tails: Vec<_> = readings(&c, BatteryId::Kick, "kick.gain_tail")
            .into_iter()
            .filter(|r| r.segment == Some(k))
            .collect();
        assert_eq!(tails.len(), 8, "segment {k}");
    }
    for h in readings(&c, BatteryId::Kick, "kick.horizon") {
        assert_eq!(
            h.value, 2080.0,
            "H is ticks(horizon), not a share of the segment"
        );
    }
}

/// One tick of a run as the test reads it, independently of certify's `Obs`.
struct Tick {
    tick: u64,
    report: TickReport,
    provider: Option<(f64, f64)>,
}

fn ticks(t: &Tape, until: u64) -> Vec<Tick> {
    let mut s = sim(t);
    let provider = s
        .world()
        .actors
        .iter()
        .find(|a| a.key.as_str() == "provider")
        .map(|a| a.id);
    let mut out = Vec::new();
    while s.tick() < until {
        let r = s.step().unwrap();
        let p = provider.and_then(|a| match s.actor_state(a) {
            Some(ActorState::Provider(p)) => Some((p.due, p.paid)),
            _ => None,
        });
        out.push(Tick {
            tick: r.tick,
            report: r,
            provider: p,
        });
    }
    out
}

#[test]
fn transient_statistics_are_reported() {
    // REPORT §6 criterion 2, O14: troughs, dead ticks, fills, rationing, ticks with no
    // consumption, spoilage and the transfer shortfall are reported per segment, and each equals
    // a fold of the run's reports written here. In bcycle(1500,4), b′ = 0.8 (segment 3) cuts
    // the good's volume by more than 80% on the way and leaves the provider short; at the
    // 0.8 → 0.2 change the good market has no supply for a tick and nobody eats (REPORT §5).
    let t = tape(BCYCLE);
    let c = run(
        &t,
        "appb",
        7500,
        20.0,
        &[conservation(), determinism(&[0.5]), runaway(APPB_BOUND)],
    )
    .certificate;
    let run_ticks = ticks(&t, 7500);
    let w = sim(&t).world().clone();
    let get = |name: &str, at: &str, seg: u32| -> f64 {
        c.reports()
            .iter()
            .find(|r| r.name == name && r.at == at && r.segment == Some(seg))
            .unwrap_or_else(|| panic!("no report {name} at {at}, segment {seg}"))
            .value
    };
    let same = |x: f64, y: f64, what: &str| assert_eq!(x.to_bits(), y.to_bits(), "{what}: {x} {y}");
    let good_id = |k: &str| w.id_of::<rustyecon_engine::prelude::GoodId>(k).unwrap();
    let mut checked = 0;
    for (k, s) in c.segments().iter().enumerate() {
        let k32 = k as u32;
        let span: Vec<&Tick> = run_ticks
            .iter()
            .filter(|x| (s.from..s.to).contains(&x.tick))
            .collect();
        for (m, (node, good)) in w.markets().enumerate() {
            let at = format!("{}/{}", w.key_of(node).unwrap(), w.key_of(good).unwrap());
            let line = |x: &Tick| x.report.markets[m];
            let mut low = (f64::INFINITY, 0);
            let (mut dead, mut no_s, mut no_d, mut rat_b, mut rat_s) = (0.0, 0.0, 0.0, 0.0, 0.0);
            let (mut worst_b, mut worst_s) = (f64::INFINITY, f64::INFINITY);
            for x in &span {
                let l = line(x);
                if l.cleared < low.0 {
                    low = (l.cleared, x.tick);
                }
                let traded =
                    l.supply > 0.0 && l.demand > 0.0 && l.buyer_fill > 0.0 && l.seller_fill > 0.0;
                if !traded {
                    dead += 1.0;
                    if l.supply == 0.0 {
                        no_s += 1.0;
                    }
                    if l.demand == 0.0 {
                        no_d += 1.0;
                    }
                }
                if l.demand > 0.0 {
                    worst_b = worst_b.min(l.buyer_fill);
                    if l.buyer_fill < 1.0 - RATIONED {
                        rat_b += 1.0;
                    }
                }
                if l.supply > 0.0 {
                    worst_s = worst_s.min(l.seller_fill);
                    if l.seller_fill < 1.0 - RATIONED {
                        rat_s += 1.0;
                    }
                }
            }
            same(get("trough.cleared", &at, k32), low.0, "trough");
            same(get("trough.tick", &at, k32), low.1 as f64, "trough tick");
            same(
                get("trough.end", &at, k32),
                line(span.last().unwrap()).cleared,
                "end",
            );
            same(get("dead.ticks", &at, k32), dead, "dead");
            same(get("dead.no_supply", &at, k32), no_s, "no supply");
            same(get("dead.no_demand", &at, k32), no_d, "no demand");
            same(get("fill.worst_buyer", &at, k32), worst_b, "worst buyer");
            same(get("fill.worst_seller", &at, k32), worst_s, "worst seller");
            same(
                get("fill.rationed_buyer_ticks", &at, k32),
                rat_b,
                "rationed buyers",
            );
            same(
                get("fill.rationed_seller_ticks", &at, k32),
                rat_s,
                "rationed sellers",
            );
            checked += 10;
        }
        // Rationing per (market, class, side).
        let mut keys: Vec<(usize, usize, SideTag)> = span
            .iter()
            .flat_map(|x| {
                x.report.rationing.iter().map(|r| {
                    let m = w
                        .markets()
                        .position(|(n, g)| n == r.node && g == r.good)
                        .unwrap();
                    (m, r.class.idx(), r.side)
                })
            })
            .collect();
        keys.sort();
        keys.dedup();
        for (m, class, side) in keys {
            let (node, good) = w.markets().nth(m).unwrap();
            let at = format!(
                "{}/{}/{}/{}",
                w.key_of(node).unwrap(),
                w.key_of(good).unwrap(),
                w.classes[class],
                if side == SideTag::Buy { "buy" } else { "sell" }
            );
            let (mut budget, mut short, mut worst, mut ticks_below) =
                (0.0, 0.0, f64::INFINITY, 0.0);
            for x in &span {
                for r in x.report.rationing.iter().filter(|r| {
                    r.node == node && r.good == good && r.class.idx() == class && r.side == side
                }) {
                    budget += r.requested - r.feasible;
                    short += r.feasible - r.filled;
                    if r.feasible > 0.0 {
                        let f = r.filled / r.feasible;
                        worst = worst.min(f);
                        if f < 1.0 - RATIONED {
                            ticks_below += 1.0;
                        }
                    }
                }
            }
            same(get("ration.budget_short", &at, k32), budget, "budget");
            same(get("ration.market_short", &at, k32), short, "market short");
            same(get("ration.worst", &at, k32), worst, "worst");
            same(
                get("ration.rationed_ticks", &at, k32),
                ticks_below,
                "rationed",
            );
            checked += 4;
        }
        // Goods: no consumption, spoilage.
        for g in ["good", "land", "labour", "mach"] {
            let id = good_id(g);
            let sum_of = |x: &Tick, p: Provenance| {
                -x.report
                    .audit
                    .lines
                    .iter()
                    .filter(|(gg, pp, _)| *gg == id && *pp == p)
                    .fold(0.0, |a, (_, _, q)| a + q)
            };
            let spoiled = span
                .iter()
                .fold(0.0, |a, x| a + sum_of(x, Provenance::Spoilage));
            same(get("spoiled.total", g, k32), spoiled, "spoiled");
            let eaten = run_ticks
                .iter()
                .any(|x| sum_of(x, Provenance::Consumption) != 0.0);
            if eaten {
                let none = span
                    .iter()
                    .filter(|x| sum_of(x, Provenance::Consumption) == 0.0)
                    .count() as f64;
                same(
                    get("consumption.none_ticks", g, k32),
                    none,
                    "no consumption",
                );
                checked += 1;
            }
            checked += 1;
        }
        let (mut short, mut short_ticks) = (0.0, 0.0);
        for x in &span {
            let (due, paid) = x.provider.unwrap();
            short += due - paid;
            if paid < due {
                short_ticks += 1.0;
            }
        }
        same(get("transfer.short", "provider", k32), short, "transfer");
        same(
            get("transfer.short_ticks", "provider", k32),
            short_ticks,
            "short ticks",
        );
        checked += 2;
    }
    assert!(checked > 300, "{checked}");
    // What the segments show.
    let (trough, end) = (
        get("trough.cleared", "home/good", 3),
        get("trough.end", "home/good", 3),
    );
    assert!(
        trough < 0.2 * end,
        "b′ = 0.8 cuts the good by more than 80%: {trough} of {end}"
    );
    assert!(get("transfer.short", "provider", 3) > 0.0);
    assert!(get("fill.rationed_buyer_ticks", "home/good", 3) > 0.0);
    assert_eq!(get("dead.ticks", "home/good", 4), 1.0);
    assert_eq!(get("dead.no_supply", "home/good", 4), 1.0);
    assert!(get("consumption.none_ticks", "good", 4) >= 1.0);
    for k in 0..3 {
        assert_eq!(get("transfer.short", "provider", k), 0.0);
    }
}

#[test]
fn deferred_tape_keeps_every_param() {
    // §7 step 2: the kick's base continuation defers every dated event at or after T past the
    // horizon, and keeps it. A param whose only reference is a SetParam's target after T is
    // registered live; dropping the event with its params (the first draft) moves world_id.
    let gate = tape(GATE);
    let mut t = gate.clone();
    t.params.push(
        ron::from_str(
            "(key: \"dial.only\", value: 300.0, unit: FlowPerYear, basis: Assumed(\"test\"))",
        )
        .unwrap(),
    );
    t.events.push(
        ron::from_str(&format!(
            "(key: \"dial.set\", at: \"{}\", basis: Assumed(\"test\"), act: SetParam(param: \
             \"dial.only\", to: \"workers.buy.bread.high\"))",
            date_of(&gate, 1800)
        ))
        .unwrap(),
    );
    let base = sim(&t).world().clone();
    let d = deferred_tape(&t, 1040, 1040 + 2080).unwrap();
    let w = sim(&d).world().clone();
    assert_eq!(w.world_id, base.world_id);
    assert_eq!(w.registry.params(), base.registry.params());
    assert_eq!(w.prefix_id(1040), base.prefix_id(1040));
    let moved: Vec<&str> = d
        .events
        .iter()
        .filter(|e| e.at.to_string() == date_of(&t, 3120))
        .map(|e| e.key.as_str())
        .collect();
    assert_eq!(
        moved,
        ["mine.restored", "dial.set"],
        "the events at or after 1040"
    );
    assert_eq!(
        d.events
            .iter()
            .find(|e| e.key.as_str() == "oven.opens")
            .unwrap()
            .at,
        t.events
            .iter()
            .find(|e| e.key.as_str() == "oven.opens")
            .unwrap()
            .at,
        "an event before T stays: the oven opens at tick 948"
    );
    // The first draft dropped the events and their params instead.
    let mut dropped = t.clone();
    dropped.events.retain(|e| e.key.as_str() != "dial.set");
    dropped.params.retain(|p| p.key.as_str() != "dial.only");
    assert_ne!(sim(&dropped).world().world_id, base.world_id);
}

#[test]
fn a_failed_kick_is_a_fail_not_an_error() {
    // §7: every failure after the base tape loads is inside the certificate. The kick runner fed
    // another world's checkpoint fails Kick with a note; and a tape whose base continuation fails
    // (a recurring burn that first falls due inside the horizon) certifies as a sealed FAIL.
    let gate = tape(GATE);
    let gw = sim(&gate).world().clone();
    let mut a = sim(&tape(APPB));
    a.run_until(100, &mut |_| {}).unwrap();
    let foreign = a.checkpoint().unwrap();
    let bars = battery::KickBars {
        size: KICK,
        horizon: 520,
        tail: 52,
        max_gain: MAX_GAIN,
        max_peak: MAX_PEAK,
    };
    let ks = kick_segment(&gate, &gw, &foreign, 0, &bars, "gate-2026-09-26.ron");
    let e = ks.runs.as_ref().unwrap_err();
    assert!(e.contains("refused"), "{e}");
    let k = battery::kick(&[ks], &bars, &certify::Names::of(&gw));
    assert!(!k.pass && k.notes[0].contains("refused"), "{:?}", k.notes);
    // A burn that falls due at tick 1,010, after the scored 1,000.
    let appb = tape(APPB);
    let mut t = appb.clone();
    t.recurring.push(
        ron::from_str(&format!(
            "(key: \"drain\", first: \"{}\", every: \"life.one_tick\", last: None, basis: \
             Assumed(\"test\"), act: Burn(holder: \"workers\", good: \"coin\", amount: \
             Qty(1e15)))",
            date_of(&appb, 1010)
        ))
        .unwrap(),
    );
    let c = run(&t, "appb", 1000, 10.0, &appb_batteries(10.0)).certificate;
    assert_eq!(c.verdict(), Verdict::Fail);
    let k = c.battery(BatteryId::Kick).unwrap();
    assert!(!k.pass);
    assert!(
        k.notes[0].contains("the base continuation: the run failed"),
        "{:?}",
        k.notes
    );
    assert!(c.battery(BatteryId::Settles).unwrap().pass);
}

#[test]
fn reserved_keys_do_not_load() {
    // §7: keys under `certify.` are the kick's; a base tape that uses one does not certify.
    let gate = tape(GATE);
    let mut t = gate.clone();
    t.params.push(
        ron::from_str(&format!(
            "(key: \"{KICK_FACTOR}\", value: 1.0, unit: Dimensionless, basis: Assumed(\"test\"))"
        ))
        .unwrap(),
    );
    let e = certify(&t, Scoring::Unscored { until: 10 }, &build(), &mut |_| {}).unwrap_err();
    assert_eq!(e, CertifyError::Reserved(vec![KICK_FACTOR.to_string()]));
    let mut t = gate.clone();
    t.events[0].key = rustyecon_engine::prelude::Key::new("certify.mine").unwrap();
    assert!(matches!(
        certify(&t, Scoring::Unscored { until: 10 }, &build(), &mut |_| {}),
        Err(CertifyError::Reserved(_))
    ));
}

#[test]
fn price_shocks_are_counted() {
    // §6: a tape that sets its own prices says so. The gate with one dated ScalePrice certifies
    // with price_shocks 1. Unscored, the shock is no failure. Scored, it fails unless the
    // criteria register it (amended at S2.5): the gate's registered batteries fail the shocked
    // run on the count alone, and the same file registering one shock passes it.
    let gate = tape(GATE);
    let mut t = gate.clone();
    t.params.push(
        ron::from_str(
            "(key: \"shock.factor\", value: 1.5, unit: Dimensionless, basis: Assumed(\"test\"))",
        )
        .unwrap(),
    );
    t.events.push(
        ron::from_str(&format!(
            "(key: \"bread.shock\", at: \"{}\", basis: Assumed(\"test\"), act: ScalePrice(node: \
             \"village\", good: \"bread\", by: \"shock.factor\"))",
            date_of(&gate, 700)
        ))
        .unwrap(),
    );
    let out = certify(&t, Scoring::Unscored { until: 2080 }, &build(), &mut |_| {}).unwrap();
    assert_eq!(out.certificate.price_shocks(), 1);
    assert_eq!(out.certificate.verdict(), Verdict::Unscored);
    let plain = certify(
        &gate,
        Scoring::Unscored { until: 2080 },
        &build(),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(plain.certificate.price_shocks(), 0);
    assert_eq!(
        out.manifest.run.world_id, plain.manifest.run.world_id,
        "a shock keeps the world"
    );
    let text = criteria_text(
        "gate",
        tape_hash(&t),
        &date_of(&t, 2080),
        1.0,
        &gate_batteries(),
        RATIONED,
    );
    let scored = |text: &str| {
        let c = criteria("gate", text);
        certify(
            &t,
            Scoring::Criteria {
                criteria: &c,
                file: &file("gate"),
            },
            &build(),
            &mut |_| {},
        )
        .unwrap()
        .certificate
    };
    let none = scored(&text);
    assert_eq!(none.verdict(), Verdict::Fail);
    assert!(
        none.batteries().iter().all(|b| b.pass),
        "only the count fails it"
    );
    assert_eq!(
        none.failures(),
        [
            "the run fired 1 ScalePrice events, and its criteria allow 0: a tape that sets its \
          own prices certifies only as far as its criteria register (R3)"
        ]
    );
    let one = scored(&with_price_shocks(&text, 1));
    assert_eq!(one.verdict(), Verdict::Pass, "{:?}", one.failures());
    assert_eq!(one.criteria().map(|c| c.price_shocks), Some(1));
}

/// appb under July's step rule from w×2 to tick `n`, with a dated `ScalePrice` that resets a
/// posted price to its genesis value at every tick it has left `[lo, hi]` of it: the
/// verification's E1. Returns the tape and its count of shocks.
fn clamped(lo: f64, hi: f64, n: u64) -> (Tape, u64) {
    let base = tape(JULY);
    let mut t = base.clone();
    let mut s = sim(&t);
    let w = s.world().clone();
    let markets: Vec<_> = w.markets().collect();
    let p0: Vec<f64> = markets
        .iter()
        .map(|(a, b)| s.price(*a, *b).unwrap())
        .collect();
    let mut shocks = 0;
    for tick in 0..n {
        let mut added = false;
        for (m, (node, good)) in markets.iter().enumerate() {
            let p = s.price(*node, *good).unwrap();
            let r = p / p0[m];
            if !(lo..=hi).contains(&r) {
                let key = format!("clamp.f.{tick}.{m}");
                t.params.push(
                    ron::from_str(&format!(
                        "(key: \"{key}\", value: {:?}, unit: Dimensionless, basis: \
                         Assumed(\"test\"))",
                        p0[m] / p
                    ))
                    .unwrap(),
                );
                t.events.push(
                    ron::from_str(&format!(
                        "(key: \"clamp.e.{tick}.{m}\", at: \"{}\", basis: Assumed(\"test\"), \
                         act: ScalePrice(node: \"{}\", good: \"{}\", by: \"{key}\"))",
                        date_of(&base, tick),
                        w.key_of(*node).unwrap(),
                        w.key_of(*good).unwrap()
                    ))
                    .unwrap(),
                );
                added = true;
                shocks += 1;
            }
        }
        if added {
            let cp = s.checkpoint().unwrap();
            s = Sim::resume(&t, &cp).unwrap();
        }
        s.step().unwrap();
    }
    (t, shocks)
}

#[test]
fn a_tape_that_clamps_its_prices_does_not_pass() {
    // The verification's E1 (blocker): July's step rule from w×2 runs away at tick 415 and
    // stops at 765. Dated ScalePrice events, each its own dated shock, that reset a price to
    // genesis whenever it leaves [1/2, 2] of it keep every price inside the runaway bound, and
    // under the minimal legal criteria (Conservation, Determinism, Runaway) the clamped run
    // certified PASS: the loader refuses only a recurring ScalePrice, and nothing scored the
    // count. Now the count fails it; criteria that register every one of the shocks pass it,
    // which is the file saying so.
    let (t, shocks) = clamped(0.5, 2.0, 1000);
    assert!(shocks > 100, "{shocks}");
    let bats = [conservation(), determinism(&[0.5]), runaway(APPB_BOUND)];
    let text = criteria_text(
        "appb",
        tape_hash(&t),
        &date_of(&t, 1000),
        5.0,
        &bats,
        RATIONED,
    );
    let score = |text: &str| {
        let c = criteria("appb", text);
        certify(
            &t,
            Scoring::Criteria {
                criteria: &c,
                file: &file("appb"),
            },
            &build(),
            &mut |_| {},
        )
        .unwrap()
        .certificate
    };
    let c = score(&text);
    assert_eq!(c.price_shocks(), shocks);
    assert_eq!(c.reached(), 1000, "the clamps keep the run going");
    assert!(
        c.batteries().iter().all(|b| b.pass),
        "every battery passes the clamped run: {:?}",
        c.batteries()
    );
    assert_eq!(c.verdict(), Verdict::Fail);
    assert_eq!(
        c.failures(),
        [format!(
            "the run fired {shocks} ScalePrice events, and its criteria allow 0: a tape that \
             sets its own prices certifies only as far as its criteria register (R3)"
        )]
    );
    let one_short = score(&with_price_shocks(&text, shocks - 1));
    assert_eq!(one_short.verdict(), Verdict::Fail);
    let all = score(&with_price_shocks(&text, shocks));
    assert_eq!(all.verdict(), Verdict::Pass, "{:?}", all.failures());
}

/// appb at desk turnover ×16 from its exact genesis, with both desks' turnover restored to
/// appb's 5.2 a year by dated events at tick `at` (the verification's E4).
fn restored(at: u64) -> Tape {
    let base = tape(BUFFER16);
    let mut t = base.clone();
    for d in ["good", "mach"] {
        t.params.push(
            ron::from_str(&format!(
                "(key: \"restore.{d}\", value: 5.2, unit: RatePerYear, basis: Assumed(\"test\"))"
            ))
            .unwrap(),
        );
        t.events.push(
            ron::from_str(&format!(
                "(key: \"restore.{d}.at\", at: \"{}\", basis: Assumed(\"test\"), act: \
                 SetParam(param: \"buffer.desk.{d}.cash\", to: \"restore.{d}\"))",
                date_of(&base, at)
            ))
            .unwrap(),
        );
    }
    t
}

#[test]
fn a_merged_shock_is_kicked() {
    // The verification's E4 (blocker): appb at desk turnover ×16 rests at an unstable point,
    // frozen by rounding. A dated restore of the turnover at tick 1,100 closes a segment, and
    // the kick at its end, run under the deferred tape and so under ×16, fails. The same restore
    // at tick 1,000, closer than min_segment (20 years, 1,040 ticks) to the start, merged into
    // one segment, and the only kick ran at the end under the restored turnover: PASS, after 19
    // years at an unstable point. The kick now fires at every dated shock, merged or not.
    for (at, segments) in [(1000, 1), (1100, 2)] {
        let c = run(&restored(at), "appb", 3120, 20.0, &appb_batteries(40.0)).certificate;
        assert_eq!(c.segments().len(), segments, "restored at {at}");
        let k = c.battery(BatteryId::Kick).unwrap();
        assert!(!k.pass, "restored at {at}");
        assert_eq!(c.verdict(), Verdict::Fail, "restored at {at}");
        let point = format!("at tick {at}");
        let peaks: Vec<&Reading> = readings(&c, BatteryId::Kick, "kick.gain_peak")
            .into_iter()
            .filter(|r| r.at.ends_with(&point))
            .collect();
        assert_eq!(peaks.len(), 8, "restored at {at}");
        assert!(
            peaks.iter().any(|r| r.value > MAX_PEAK),
            "restored at {at}: {peaks:?}"
        );
        assert!(
            peaks.iter().all(|r| r.segment == Some(0)),
            "the kick at {at} probes the first segment's regime"
        );
        // The kick at the end probes the restored turnover, which is stable.
        let end: Vec<&Reading> = readings(&c, BatteryId::Kick, "kick.gain_peak")
            .into_iter()
            .filter(|r| r.at.ends_with("at tick 3120"))
            .collect();
        assert_eq!(end.len(), 8);
        assert!(end.iter().all(|r| r.value <= MAX_PEAK), "{end:?}");
        assert!(
            c.battery(BatteryId::Settles).unwrap().pass,
            "restored at {at}"
        );
    }
}

#[test]
fn a_kick_moves_its_price_up_and_down() {
    // REPORT §6: a kick in ± each price. At the kick's tick the kicked market's posted price is
    // fl(p·(1 + size)) or fl(p·(1 − size)) of the base continuation's p, and no other market's
    // moves (the verification's minor: a − kick made a + kick passed every fast test).
    let gate = tape(GATE);
    let mut s = sim(&gate);
    s.run_until(1000, &mut |_| {}).unwrap();
    let cp = s.checkpoint().unwrap();
    let w = s.world().clone();
    let bars = battery::KickBars {
        size: KICK,
        horizon: 3,
        tail: 2,
        max_gain: MAX_GAIN,
        max_peak: MAX_PEAK,
    };
    let ks = kick_segment(&gate, &w, &cp, 0, &bars, "gate-2026-09-26.ron");
    let (base, runs) = ks.runs.as_ref().unwrap();
    assert_eq!(runs.len(), 12, "± each of six markets");
    for r in runs {
        let first = &r.prices.as_ref().unwrap()[0];
        let factor = match r.sign {
            battery::Sign::Up => 1.0 + KICK,
            battery::Sign::Down => 1.0 - KICK,
        };
        for (m, (&kicked, &p)) in first.iter().zip(&base[0]).enumerate() {
            let want = if m == r.market { p * factor } else { p };
            assert_eq!(kicked.to_bits(), want.to_bits(), "{:?} {m}", r.sign);
        }
        match r.sign {
            battery::Sign::Up => assert!(first[r.market] > base[0][r.market]),
            battery::Sign::Down => assert!(first[r.market] < base[0][r.market]),
        }
    }
}

#[test]
fn tape_hash_covers_every_input() {
    // N10, C2: tape_hash is FNV-1a 64 over the canonical to_ron, which holds everything the
    // loader reads. One edit of each kind moves it, the name and a basis text included, which
    // world_id leaves out; reformatting, comments, CRLF, list order and a round trip keep it.
    let h = |text: &str| tape_hash(&tape(text));
    let wid = |text: &str| sim(&tape(text)).world().world_id;
    let base = h(GATE);
    let rate = "(key: \"rate.bread\", value: 5.2,";
    let ulp = format!("(key: \"rate.bread\", value: {:?},", 5.2f64.next_up());
    let edits: [(&str, &str, bool); 6] = [
        ("name: \"gate\"", "name: \"gate2\"", false),
        ("Assumed(\"July REL_FLOW\")", "Assumed(\"July REL_FLOW.\")", false),
        (rate, &ulp, true),
        ("at: \"1760-03-01\"", "at: \"1760-03-02\"", false),
        ("(\"grain\", 2.0), (\"fuel\", 1.0)], outputs: [(\"bread\", 3.0)], capacity: \"mill.capacity\"", "(\"grain\", 2.5), (\"fuel\", 1.0)], outputs: [(\"bread\", 3.0)], capacity: \"mill.capacity\"", true),
        ("last: None", "last: Some(\"1780-01-01\")", false),
    ];
    for (from, to, world) in edits {
        assert_eq!(GATE.matches(from).count(), 1, "{from}");
        let edited = GATE.replacen(from, to, 1);
        assert_ne!(h(&edited), base, "{from} -> {to}");
        assert_eq!(wid(&edited) != wid(GATE), world, "world_id and {from}");
    }
    let reformatted = GATE.replace("    ", "  ").replace("),\n", "),\n\n");
    let commented = GATE.replace(
        "    goods: [",
        "    // a comment\n    goods: [ /* and another */",
    );
    let crlf = GATE.replace('\n', "\r\n");
    let mut reordered = tape(GATE);
    reordered.params.reverse();
    reordered.goods.reverse();
    reordered.actors.reverse();
    reordered.events.reverse();
    for text in [reformatted, commented, crlf, tape(GATE).to_ron()] {
        assert_eq!(h(&text), base);
    }
    assert_eq!(tape_hash(&reordered), base);
}

/// The RON float tokens of `text`, outside strings, with their byte ranges.
fn float_tokens(text: &str) -> Vec<(usize, usize)> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let (mut i, mut in_str) = (0, false);
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        let starts = (c == b'-' || c.is_ascii_digit())
            && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_'));
        if starts {
            let mut j = i + 1;
            while j < b.len()
                && (b[j].is_ascii_digit() || b[j] == b'.' || b[j] == b'e' || b[j] == b'-')
            {
                j += 1;
            }
            let tok = &text[i..j];
            if tok.contains('.') && tok.parse::<f64>().is_ok() {
                out.push((i, j));
            }
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

#[test]
fn finite_scan_reads_every_rendered_number() {
    // N12, §8: every number a certificate renders is scanned. Each float of a real PASS
    // certificate's RON, replaced by NaN, is refused by from_ron as NonFinite at that number's
    // path, not as a parse error.
    let c = run(&tape(GATE), "gate", 2080, 1.0, &gate_batteries()).certificate;
    assert_eq!(c.verdict(), Verdict::Pass);
    let text = c.to_ron();
    let tokens = float_tokens(&text);
    assert!(tokens.len() > 500, "{}", tokens.len());
    let mut paths = Vec::new();
    for (i, j) in tokens {
        let edited = format!("{}NaN{}", &text[..i], &text[j..]);
        let raw: RawCertificate = ron::from_str(&edited).expect("NaN parses");
        let want = nonfinite_paths(&raw);
        assert_eq!(want.len(), 1, "{}", &text[i..j]);
        assert_eq!(
            Certificate::from_ron(&edited),
            Err(CertificateError::NonFinite {
                path: want[0].clone()
            })
        );
        paths.push(want[0].clone());
    }
    paths.dedup();
    assert!(paths.iter().any(|p| p.starts_with("batteries[")));
    assert!(paths.iter().any(|p| p.starts_with("reports[")));
    assert!(paths.iter().any(|p| p.contains(".bar.")));
}

/// Every number in `text`: whitespace-separated tokens, stripped of brackets and punctuation,
/// that parse as a float.
fn numbers(text: &str) -> Vec<f64> {
    text.split(|c: char| c.is_whitespace() || "()[],;:=\"".contains(c))
        .filter_map(|t| t.parse::<f64>().ok())
        .collect()
}

#[test]
fn render_prints_only_serialised_numbers() {
    // §8: render prints serialised fields only and does no arithmetic, so every number it prints
    // is bit-equal to one the RON holds: no ratio, percentage or difference the finite scan
    // could not see.
    let pass = run(&tape(GATE), "gate", 2080, 1.0, &gate_batteries()).certificate;
    let fail = run(
        &tape(JULY),
        "appb",
        1000,
        5.0,
        &[conservation(), determinism(&[0.5]), runaway(APPB_BOUND)],
    )
    .certificate;
    for c in [pass, fail] {
        let held: Vec<u64> = numbers(&c.to_ron()).iter().map(|x| x.to_bits()).collect();
        let printed = numbers(&c.render());
        assert!(printed.len() > 50);
        for x in printed {
            assert!(
                held.contains(&x.to_bits()),
                "{x} is printed but not serialised"
            );
        }
    }
}
