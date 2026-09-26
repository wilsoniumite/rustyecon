//! The probe's tape and harness (docs/probe/RULES.md; PROBE-SPEC §4): the committed tape is its
//! generator's output, it loads and runs deterministically, it conserves every tick, and mode A
//! holds at the oracle's point for the registered number of ticks.
//!
//! One bar is relative and named here: `COIN`, 1e-12 of the money stock, for the drift of the
//! total coin over a run. Coin moves only by transfers, whose splits and merges round by at most
//! half an ulp each and are declared as `Rounding` (docs/ENGINE.md §2.2); a few thousand ticks of
//! a few dozen transfers stay far inside it. Mode A's bars are PROBE-SPEC §4.6's, in
//! `probe::protocol`.

use probe::harness::{run, Stop, Target, OBSERVABLES};
use probe::perturb::{battery, family, Perturbation};
use probe::protocol::{RUN_TICKS, TOL_FLOOR};
use probe::setup::{equilibrium, tape_ron, Assign, OneSided, ScaleRule, Setup, TapeArgs};
use rustyecon_engine::prelude::{GoodId, Holder, ParamId, Provenance, Sim, Tape};

const APPB: &str = include_str!("../../../tapes/appb.ron");
const COIN: f64 = 1e-12;

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

#[test]
fn appb_tape_is_its_generators_output() {
    // tapes/appb.ron is `appb-tape`'s output for the registered setup at 52 ticks a year, byte
    // for byte, so its genesis is the oracle's point and nobody edits it by hand.
    let generated = tape_ron(&Setup::registered(52)).expect("the generator runs");
    assert!(
        generated == APPB,
        "tapes/appb.ron differs from its generator: run `cargo run -p rustyecon-probe --bin \
         appb-tape -- tapes/appb.ron`"
    );
}

#[test]
fn certify_testdata_is_generated() {
    // docs/CERTIFY.md §13: certify's testdata is `appb-tape`'s output for the options its first
    // line names, byte for byte, so its runs are the probe's setups and nobody edits them by
    // hand. With no option, `appb-tape` writes tapes/appb.ron.
    let cases: [(&str, &str, &[&str]); 3] = [
        (
            "appb-bcycle.ron",
            include_str!("../../certify/testdata/appb-bcycle.ron"),
            &["--perturb", "bcycle(1500,4)"],
        ),
        (
            "appb-freeze.ron",
            include_str!("../../certify/testdata/appb-freeze.ron"),
            &["--set", "buffer.*=16", "--perturb", "w*1.05"],
        ),
        (
            "appb-july.ron",
            include_str!("../../certify/testdata/appb-july.ron"),
            &["--scale", "step", "--perturb", "w*2"],
        ),
    ];
    for (name, text, args) in cases {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let generated = TapeArgs::parse(&args)
            .and_then(|a| a.text())
            .expect("the generator runs");
        assert!(
            generated == text,
            "crates/certify/testdata/{name} differs from its generator: run `cargo run -p              rustyecon-probe --bin appb-tape -- {} crates/certify/testdata/{name}`",
            args.join(" ")
        );
        assert!(text.starts_with(&format!("// Written by `appb-tape {}`", args.join(" "))));
    }
    let plain = TapeArgs::parse(&[]).and_then(|a| a.text()).unwrap();
    assert!(
        plain == APPB,
        "appb-tape with no option writes tapes/appb.ron"
    );
    assert!(TapeArgs::parse(&["--perturb".to_string()]).is_err());
    assert!(TapeArgs::parse(&["--bogus".to_string()]).is_err());
}

#[test]
fn appb_tape_loads_and_runs_deterministically() {
    // R8: two runs of the tape give identical reports and hash streams, and the state moves.
    let (mut a, mut b) = (sim(APPB), sim(APPB));
    let genesis = a.hash();
    for _ in 0..3000 {
        let ra = a.step().expect("the probe world runs");
        let rb = b.step().expect("the probe world runs");
        assert_eq!(ra, rb);
    }
    assert_eq!(a.hash(), b.hash());
    assert_ne!(a.hash(), genesis);
}

#[test]
fn appb_conserves_every_tick() {
    // R2 on the probe world, at rest and through a large transient (w ×2 at genesis, where
    // markets ration and machine services and the good spoil): every tick's ledger and the
    // run's close within their registered tolerances, and the money stock stays put.
    let mut setup = Setup::registered(52);
    Perturbation::parse("w*2+coin.desk.mach*0.1")
        .unwrap()
        .apply(&mut setup, 4000)
        .unwrap();
    for text in [APPB.to_string(), tape_ron(&setup).unwrap()] {
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
        let mut spoiled = 0;
        for _ in 0..4000 {
            let r = s.step().expect("no breach stops the run");
            assert!(r.audit.max_margin <= 1.0 && r.run.max_margin <= 1.0);
            assert!((money(&s) - m0).abs() <= COIN * m0);
            if r.audit
                .lines
                .iter()
                .any(|l| l.1 == Provenance::Spoilage && l.2 < 0.0)
            {
                spoiled += 1;
            }
        }
        assert!(spoiled > 0, "the probe world spoils what it does not sell");
    }
}

#[test]
fn appb_holds_at_the_oracle_point() {
    // Mode A (PROBE-SPEC §4.6): from the oracle's f64 point with each actor's stationary coin,
    // for L = RUN_TICKS ticks, every observable stays within 1e-9 of the oracle in log, every
    // market trades with every fill at least 1 − 1e-9, no machine service or good spoils beyond
    // 1e-9 of its volume, and the ledger is clean. A loop with growth g per tick above 1 would
    // cross 1e-9 within ln(1e6)/ln(g) ticks, so this is also a linear-stability test.
    let rec = run(&Setup::registered(52), "hold", RUN_TICKS, &mut |_| {}).unwrap();
    assert_eq!(rec.stop, Stop::Ran);
    assert_eq!(rec.dhat.len() as u64, RUN_TICKS);
    assert_eq!(rec.hold_failure, None);
    let worst = rec.dhat.iter().fold(0.0, |a: f64, &b| a.max(b)) * TOL_FLOOR;
    assert!(worst <= 1e-12, "the largest gap was {worst:e}");
}

#[test]
fn appb_variants_hold_at_the_oracle_point() {
    // Every registered variant rests at the same point: ex-post assignment, the ceiling payout
    // (inert at rest), the negative control (its margin is 0 at the point), Hold, a markup tilt,
    // and the tape at 12 and 365 ticks a year with its per-year dials restated by Clock. Mode A
    // over 2,000 ticks each; the full length is the harness's (docs/probe/RULES.md §6).
    let mut setups = Vec::new();
    for tpy in [12, 365] {
        setups.push(Setup::registered(tpy));
    }
    let mut s = Setup::registered(52);
    s.assign = Assign::ExPost;
    setups.push(s);
    for scale in [ScaleRule::ceiling(), ScaleRule::july_step()] {
        let mut s = Setup::registered(52);
        s.scale = scale;
        setups.push(s);
    }
    let mut s = Setup::registered(52);
    s.one_sided = OneSided::Hold;
    setups.push(s);
    let mut s = Setup::registered(52);
    s.dials.set("tilt.*", 1.0).unwrap();
    setups.push(s);
    for setup in setups {
        let rec = run(&setup, "hold", 2000, &mut |_| {}).unwrap();
        assert_eq!(rec.stop, Stop::Ran, "{setup:?}");
        assert_eq!(rec.hold_failure, None, "{setup:?}");
    }
}

#[test]
fn perturbations_parse_and_apply() {
    // PROBE-SPEC §4.7: 57 runs, 16 in Tier 1, 20 in Tier 2 and 21 in Tier 3, every name parses;
    // the shapes displace what the table says; a dated shock lands at L/4 and restarts the clock.
    let b = battery();
    assert_eq!(b.len(), 57);
    for (tier, n) in [(1, 16), (2, 20), (3, 21)] {
        assert_eq!(b.iter().filter(|(_, t)| *t == tier).count(), n);
    }
    for f in ["battery", "stocks", "joint2", "joint4", "history"] {
        for name in family(f).unwrap() {
            let mut s = Setup::registered(52);
            Perturbation::parse(&name)
                .and_then(|p| p.apply(&mut s, RUN_TICKS))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
    let mut s = Setup::registered(52);
    Perturbation::parse("JB(2)")
        .unwrap()
        .apply(&mut s, RUN_TICKS)
        .unwrap();
    let x = s.displace;
    assert_eq!((x.w, x.r, x.pm, x.p), (2.0, 1.0, 0.5, 2.0));
    assert_eq!(x.share, probe::setup::Share::Times(0.5));
    let mut s = Setup::registered(52);
    let dated = Perturbation::parse("b=0.8@dated").unwrap();
    dated.apply(&mut s, RUN_TICKS).unwrap();
    assert_eq!(dated.clock_start(RUN_TICKS), RUN_TICKS / 4);
    assert_eq!(
        (s.b_at(RUN_TICKS / 4 - 1), s.b_at(RUN_TICKS / 4)),
        (0.4, 0.8)
    );
    // The event's date falls in that tick, and the b in force changes there in the run.
    let mut sm = sim(&tape_ron(&s).unwrap());
    let b_param = sm.world().id_of::<ParamId>("inst.b").unwrap();
    for _ in 0..RUN_TICKS / 4 {
        sm.step().unwrap();
    }
    assert_eq!(sm.param(b_param), Some(0.4));
    sm.step().unwrap();
    assert_eq!(sm.param(b_param), Some(0.8));
}

#[test]
fn rows_carry_the_oracle_target() {
    // Every CSV row carries the oracle's values beside the observables, so gaps can be
    // recomputed: the target is Target::of the oracle's equilibrium at the b in force, each gap
    // is |ln(o/o*)|, and D̂ is the largest gap over the tolerance floor.
    let setup = Setup::registered(52);
    let e = equilibrium(&setup.instance, 0.4, 52).unwrap();
    let want = Target::of(&e, 10.0);
    let mut rows = Vec::new();
    run(&setup, "w*1.05", 60, &mut |r| rows.push(r.clone())).unwrap();
    assert_eq!(rows.len(), 60);
    for r in &rows {
        assert_eq!(r.target, want.obs);
        for i in 0..OBSERVABLES.len() {
            assert_eq!(
                r.gap[i],
                rustyecon_core::num::ln(r.obs[i] / r.target[i]).abs()
            );
        }
        let dhat = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g)) / TOL_FLOOR;
        assert_eq!(r.dhat, dhat);
        assert_eq!(
            r.csv().split(',').count(),
            probe::harness::csv_header().split(',').count()
        );
    }
    // w ×1.05 displaces v by 5% at genesis (r stays 1), which the first row shows.
    assert_eq!(rows[0].obs[0], e.v * 1.05);
}
