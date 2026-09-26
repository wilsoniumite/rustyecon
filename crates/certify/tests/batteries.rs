//! The batteries on synthetic observations (docs/CERTIFY.md §5, §6, §13): each fails on what it
//! exists to catch, fails closed on a non-finite sample (N12), fails with "no samples" when it
//! has nothing to score, and scores windows relative to each segment (N15).
//!
//! Test bars, named once. Settles: W from half a segment, F from nine tenths, a hundredth of W
//! dead, and a band of 1e-4 in log (PROBE-SPEC §4.5's tol/10). BalanceWatch: July's bars
//! (`v2p3: certify/invariants.rs`: level 1e-9, spread 1e-12, 32 samples, half the run). Kick: a
//! gain of 1e-3 in the tail and 1e6 at the peak (the probe's tolerance, 1e-3 in log, over a 1e-9
//! kick). Runaway: 1e3.

mod common;

use certify::battery::{
    balance, conservation, determinism, kick, kick_gain, runaway, settles, trade_windows, trades,
    BalanceBars, BalanceWatch, DeterminismInputs, KickBars, KickSegment, KickedRun, ResumeRun,
    SettlesBars, Sign,
};
use certify::{segments, BatteryResult, Obs};
use common::*;
use rustyecon_core::num;
use rustyecon_engine::prelude::{Tape, TickReport};

const SETTLES: SettlesBars = SettlesBars {
    w_from: 0.5,
    f_from: 0.9,
    dead_share: 0.01,
    band: 1e-4,
};
const JULY: BalanceBars = BalanceBars {
    level: 1e-9,
    spread: 1e-12,
    min_samples: 32,
    run_share: 0.5,
};
const MAX_GAIN: f64 = 1e-3;
const MAX_PEAK: f64 = 1e6;
const SIZE: f64 = 1e-9;
const BOUND: f64 = 1e3;

fn failed(r: &BatteryResult, why: &str) {
    assert!(!r.pass, "{:?} passed; expected {why}", r.id);
    assert!(
        r.notes.iter().any(|n| n.contains(why)),
        "{:?}: no note says {why:?}: {:#?}",
        r.id,
        r.notes
    );
    assert!(!r.readings.is_empty(), "{:?} records no reading", r.id);
}

fn passed(r: &BatteryResult) {
    assert!(r.pass, "{:?} failed: {:#?}", r.id, r.notes);
    assert!(!r.readings.is_empty(), "{:?} records no reading", r.id);
}

fn bars(horizon: u64, tail: u64) -> KickBars {
    KickBars {
        size: SIZE,
        horizon,
        tail,
        max_gain: MAX_GAIN,
        max_peak: MAX_PEAK,
    }
}

/// One market's kick: the base price 1 on every tick, the kicked price `1 + gap(t)`.
fn one_kick(h: usize, gap: impl Fn(usize) -> f64) -> Vec<KickSegment> {
    let base = vec![vec![1.0]; h];
    let kicked = (0..h).map(|t| vec![1.0 + gap(t)]).collect();
    vec![KickSegment {
        segment: 0,
        at: 1000,
        runs: Ok((
            base,
            vec![KickedRun {
                market: 0,
                sign: Sign::Up,
                tape_hash: 7,
                prices: Ok(kicked),
            }],
        )),
    }]
}

fn kick_of(h: usize, tail: u64, gap: impl Fn(usize) -> f64) -> BatteryResult {
    kick(&one_kick(h, gap), &bars(h as u64, tail), &names(1))
}

#[test]
fn a_world_without_markets_fails_no_samples() {
    // No vacuous pass (§6): a battery with nothing to score fails, and says so, with a reading.
    let empty: Vec<Obs> = (0..100).map(|t| obs(t, Vec::new())).collect();
    let n = names(0);
    let segs = [seg(0, 100)];
    for r in [
        runaway(&empty, &[], BOUND, &n),
        trades(&empty, &trade_windows(100, 50), &n),
        balance(&empty, &segs, &JULY, &n),
        settles(&empty, &segs, &SETTLES, &n),
        kick(
            &[KickSegment {
                segment: 0,
                at: 100,
                runs: Ok((vec![Vec::new(); 10], Vec::new())),
            }],
            &bars(10, 2),
            &n,
        ),
        kick(&[], &bars(10, 2), &n),
        conservation(&[], 0, 100),
        determinism(&DeterminismInputs {
            base: Vec::new(),
            base_last: None,
            second: Ok(Vec::new()),
            replay: Ok(0),
            resumes: Vec::new(),
        }),
    ] {
        failed(&r, "no samples");
    }
    // A market, but no window or segment.
    let one = at_rest(100, |_, _| {});
    failed(&trades(&one, &[], &names(1)), "no samples");
    failed(&balance(&one, &[], &JULY, &names(1)), "no samples");
    failed(&settles(&one, &[], &SETTLES, &names(1)), "no samples");
}

#[test]
fn windows_are_relative_to_the_run() {
    // N15: July scored ticks 150–1000 of an 11,700-tick run (`analysis_end: 1000`). Here W and F
    // are shares of each segment, so an excursion at 0.95 of the run fails at any length.
    for n in [1000, 11_700] {
        let late = (n as f64 * 0.95) as u64;
        let spiked = at_rest(n, |t, m| {
            if t == late {
                m.price = 1.01;
            }
        });
        let r = settles(&spiked, &[seg(0, n)], &SETTLES, &names(1));
        failed(&r, "not at rest in F");
        passed(&settles(
            &at_rest(n, |_, _| {}),
            &[seg(0, n)],
            &SETTLES,
            &names(1),
        ));
        // Before W, a transient is not scored.
        let early = at_rest(n, |t, m| {
            if t == n / 4 {
                m.price = 1.01;
                m.supply = 0.0;
            }
        });
        passed(&settles(&early, &[seg(0, n)], &SETTLES, &names(1)));
    }
}

#[test]
fn settles_counts_dead_ticks_in_w_and_f() {
    // REPORT §6 criterion 1 and the probe's DEAD class: a dead tick is one on which some market
    // does not trade. W, the last half of a 1,000-tick segment, may hold ⌊0.01·500⌋ = 5 of them;
    // F, its last tenth, none.
    let dead_at = |ticks: Vec<u64>| {
        at_rest(1000, move |t, m| {
            if ticks.contains(&t) {
                *m = market(1.0, 1.0, 0.0, 0.0);
                m.cleared = 1.0;
            }
        })
    };
    let segs = [seg(0, 1000)];
    passed(&settles(
        &dead_at(vec![600, 601, 602, 603, 604]),
        &segs,
        &SETTLES,
        &names(1),
    ));
    failed(
        &settles(
            &dead_at(vec![600, 601, 602, 603, 604, 605]),
            &segs,
            &SETTLES,
            &names(1),
        ),
        "6 dead ticks in W",
    );
    failed(
        &settles(&dead_at(vec![950]), &segs, &SETTLES, &names(1)),
        "1 in F",
    );
    // Before W they are the transient's, and not scored.
    passed(&settles(
        &dead_at((100..200).collect()),
        &segs,
        &SETTLES,
        &names(1),
    ));
}

/// Three segments of 1,000 ticks, each opened by a jump to a new level that decays; the middle
/// one's level oscillates to the end if `restless`.
fn three(restless: bool) -> Vec<Obs> {
    at_rest(3000, |t, m| {
        let (k, s) = (t / 1000, (t % 1000) as f64);
        let level = [1.0, 2.0, 1.5][k as usize];
        m.price = level * (1.0 + 0.5 * num::exp(-s / 50.0));
        if restless && k == 1 {
            m.price = level * (1.0 + 0.01 * ((t % 7) as f64) / 7.0);
        }
    })
}

#[test]
fn windows_restart_at_each_dated_shock() {
    // REPORT §6 criterion 3: windows restart at each dated shock, so a history whose segments
    // each return passes, and one segment that never settles fails, though the same run scored
    // as one window, whose F falls in the last segment's rest, passes.
    let segs = [seg(0, 1000), seg(1000, 2000), seg(2000, 3000)];
    passed(&settles(&three(false), &segs, &SETTLES, &names(1)));
    let r = settles(&three(true), &segs, &SETTLES, &names(1));
    failed(&r, "segment 1: not at rest");
    assert!(!r
        .notes
        .iter()
        .any(|n| n.contains("segment 0") || n.contains("segment 2")));
    passed(&settles(&three(true), &[seg(0, 3000)], &SETTLES, &names(1)));
}

#[test]
fn short_segments_do_not_shrink_windows() {
    // C5: a dated event closes a segment only once it has lasted min_segment. With a no-op event
    // every 100 ticks of the gate world, the segments are still at least m = 520 long, so a slow
    // drift, 5e-6 in log a tick, fails Settles: over F it moves 3e-4 or more, where over the tenth
    // of a 100-tick segment it would move 5e-5, inside the band.
    let gate = tape(GATE);
    let mut t: Tape = gate.clone();
    let noop = |k: u64| {
        format!(
            "(key: \"noop.{k:02}\", at: \"{}\", basis: Assumed(\"test\"), act: Mint(holder: \
             \"workers\", good: \"coin\", qty: 0.0))",
            date_of(&gate, 100 * k)
        )
    };
    for k in 1..=20 {
        t.events
            .push(ron::from_str(&noop(k)).expect("a no-op event parses"));
    }
    let w = sim(&t).world().clone();
    let segs = segments(&w, 0, 2080, 520);
    let spans: Vec<(u64, u64)> = segs.iter().map(|s| (s.from, s.to)).collect();
    // The gate's own dated events are shocks too: the 1760 cut (tick 528) and later ones.
    assert!(segs.iter().all(|s| s.len() >= 520), "{spans:?}");
    assert_eq!(spans.first().map(|s| s.0), Some(0));
    assert_eq!(spans.last().map(|s| s.1), Some(2080));
    assert!(spans.windows(2).all(|p| p[0].1 == p[1].0), "{spans:?}");
    let merged: usize = segs.iter().map(|s| s.merged.len()).sum();
    let opened: usize = segs.iter().map(|s| s.opened_by.len()).sum();
    assert_eq!(
        merged + opened,
        24,
        "every dated event is named: 20 no-ops and the gate's 4"
    );
    let drift = at_rest(2080, |t, m| m.price = num::exp(5e-6 * t as f64));
    let r = settles(&drift, &segs, &SETTLES, &names(1));
    failed(&r, "not at rest in F");
    // The same drift over segments of 100 would pass: that is what the merge prevents.
    let hundreds: Vec<_> = (0..20).map(|k| seg(100 * k, 100 * (k + 1))).collect();
    passed(&settles(&drift, &hundreds, &SETTLES, &names(1)));
    // A kick whose gap first falls fast and then grows slowly: over H = 2,080 ticks (the
    // horizon's, not the segment's) its tail gain is far above 1; over the tenth of a short
    // segment it would read as decayed.
    let gap = |t: usize| SIZE * (num::exp(-(t as f64) / 5.0) + 1e-8 * num::exp(t as f64 / 100.0));
    failed(&kick_of(2080, 208, gap), "did not decay");
    passed(&kick_of(60, 6, gap));
}

#[test]
fn batteries_fail_closed_on_nonfinite_samples() {
    // N12: before any predicate, every field a battery reads is checked on every tick it scores;
    // the first non-finite one fails the battery, and the note names its tick and path. July's
    // statistics dropped NaN, `fold(0.0, f64::max)` dropped it, and BalanceWatch's `S > 0 or
    // D > 0` test skipped a tick with S = NaN.
    let n = 100;
    let segs = [seg(0, n)];
    let windows = trade_windows(n, 50);
    let moving = |t: u64, m: &mut certify::MarketObs| {
        m.demand = 1.0 + 0.01 * ((t % 3) as f64);
        m.buyer_fill = 1.0 / m.demand;
    };
    let base = at_rest(n, moving);
    passed(&conservation(&base, n, n));
    passed(&runaway(&base, &[1.0], BOUND, &names(1)));
    passed(&trades(&base, &windows, &names(1)));
    passed(&balance(&base, &segs, &JULY, &names(1)));
    passed(&settles(&base, &segs, &SETTLES, &names(1)));
    type Set = fn(&mut certify::MarketObs, f64);
    let fields: [(&str, Set); 7] = [
        ("price", |m, x| m.price = x),
        ("next_price", |m, x| m.next_price = x),
        ("supply", |m, x| m.supply = x),
        ("demand", |m, x| m.demand = x),
        ("cleared", |m, x| m.cleared = x),
        ("buyer_fill", |m, x| m.buyer_fill = x),
        ("seller_fill", |m, x| m.seller_fill = x),
    ];
    let at = 95;
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for (field, set) in fields {
            let mut o = base.clone();
            set(&mut o[at].markets[0], bad);
            let why = format!("at tick {at}, markets[m/0].{field}");
            let reads = |b: &str| match b {
                "Runaway" => field == "next_price",
                "Trades" => ["supply", "demand", "buyer_fill", "seller_fill"].contains(&field),
                "Balance" => ["supply", "demand"].contains(&field),
                _ => field != "next_price",
            };
            for (name, r) in [
                ("Runaway", runaway(&o, &[1.0], BOUND, &names(1))),
                ("Trades", trades(&o, &windows, &names(1))),
                ("Balance", balance(&o, &segs, &JULY, &names(1))),
                ("Settles", settles(&o, &segs, &SETTLES, &names(1))),
            ] {
                if reads(name) {
                    failed(&r, &why);
                }
            }
        }
        // Conservation reads each tick's margins.
        for (field, set) in [
            (
                "audit.max_margin",
                (|o: &mut Obs, x| o.margin = x) as fn(&mut Obs, f64),
            ),
            ("run.max_margin", |o: &mut Obs, x| o.run_margin = x),
        ] {
            let mut o = base.clone();
            set(&mut o[at], bad);
            failed(&conservation(&o, n, n), &format!("at tick {at}, {field}"));
        }
        // Runaway reads the genesis price.
        failed(
            &runaway(&base, &[bad], BOUND, &names(1)),
            "at tick 0, genesis[m/0].price",
        );
        // Balance: a one-sided tick whose other side is not a number is still read.
        for (s, d, field) in [(bad, 0.0, "supply"), (0.0, bad, "demand")] {
            let mut o = base.clone();
            o[at].markets[0].supply = s;
            o[at].markets[0].demand = d;
            failed(
                &balance(&o, &segs, &JULY, &names(1)),
                &format!("at tick {at}, markets[m/0].{field}"),
            );
        }
        // Kick: a kicked price, and the base continuation's.
        let r = kick_of(100, 10, |t| if t == 50 { bad } else { SIZE });
        failed(&r, "at tick 1050, kick[0][m/0 +].markets[m/0].price");
        let mut segs_k = one_kick(100, |t| SIZE * num::exp(-(t as f64)));
        if let Ok((base, _)) = &mut segs_k[0].runs {
            base[60][0] = bad;
        }
        failed(
            &kick(&segs_k, &bars(100, 10), &names(1)),
            "at tick 1060, kick[0].base.markets[m/0].price",
        );
    }
    // kick_gain alone carries a NaN price to its readings.
    let base = vec![vec![1.0]; 10];
    let mut kicked: Vec<Vec<f64>> = (0..10).map(|_| vec![1.0 + SIZE]).collect();
    kicked[9][0] = f64::NAN;
    assert!(kick_gain(&base, &kicked, 2).unwrap().gain_tail.is_nan());
    // Finite numbers no ratio can read fail too, where `!(x > bar)` would pass them: a price at
    // zero in F, a genesis price and posted price both at zero, and a kick whose prices fall to
    // zero in its tail.
    let zero = at_rest(n, |t, m| {
        if t == 95 {
            m.price = 0.0;
        }
    });
    failed(&settles(&zero, &segs, &SETTLES, &names(1)), "not at rest");
    let negative = at_rest(n, |t, m| {
        if t == 95 {
            m.price = -1.0;
        }
    });
    failed(
        &settles(&negative, &segs, &SETTLES, &names(1)),
        "not at rest",
    );
    let flat = at_rest(n, |_, m| m.next_price = 0.0);
    failed(&runaway(&flat, &[0.0], BOUND, &names(1)), "left");
    let mut to_zero = one_kick(100, |t| SIZE * num::exp(-(t as f64)));
    if let Ok((base, runs)) = &mut to_zero[0].runs {
        base[99][0] = 0.0;
        if let Ok(k) = &mut runs[0].prices {
            k[99][0] = 0.0;
        }
    }
    failed(&kick(&to_zero, &bars(100, 10), &names(1)), "did not decay");
}

#[test]
fn balance_watch_fails_on_a_nonfinite_imbalance() {
    // N12: July's BalanceWatch skipped a non-finite imbalance (`invariants.rs:167`), so a market
    // whose imbalance became NaN read as clean. Here it is counted and fails the verdict.
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut w = BalanceWatch::default();
        for t in 0..100 {
            w.observe(if t == 60 {
                bad
            } else {
                0.01 * ((t % 3) as f64)
            });
        }
        let e = w.judge(&JULY).expect_err("a non-finite imbalance fails");
        assert!(e.contains("non-finite"), "{e}");
    }
    let mut w = BalanceWatch::default();
    for t in 0..100 {
        w.observe(0.01 * ((t % 3) as f64));
    }
    assert!(w.judge(&JULY).is_ok());
}

#[test]
fn a_constant_imbalance_is_a_pin_however_small() {
    // Ported from July (`v2p3: certify/invariants.rs`): a pin is a constant, and a small
    // constant still counts; the legacy defect's imbalance was −1/6, not −0.99. July's bars, now
    // named here and read from criteria in a run.
    for level in [-1.0 / 6.0, -0.001, 1e-6] {
        let mut w = BalanceWatch::default();
        for _ in 0..100 {
            w.observe(level);
        }
        let sd = w.sd().unwrap();
        assert!(sd < JULY.spread, "level {level}: sd {sd}");
        assert!(w.mean().abs() > JULY.level, "level {level}");
        let e = w.judge(&JULY).expect_err("a pin");
        assert!(e.contains("pinned"), "{level}: {e}");
        // Nor need a pin repeat one exact value: a level alternating by one ulp has no run of
        // two, and its spread still says it never moved.
        let mut ulp = BalanceWatch::default();
        for i in 0..100 {
            ulp.observe(if i % 2 == 0 { level } else { level.next_up() });
        }
        assert_eq!(ulp.longest_run().0, 1);
        let e = ulp.judge(&JULY).expect_err("a pin by its spread");
        assert!(e.contains("for all 100 observations"), "{level}: {e}");
    }
}

#[test]
fn a_market_that_moves_is_not_pinned_and_one_at_zero_is_not_either() {
    // Ported from July: a real spread is not a pin, and a constant at exactly zero is a market
    // that clears, which is the goal. An unbroken stretch of one value over half the
    // observations is a pin even inside a series that also moves (the legacy sell cap's −1/6).
    let mut w = BalanceWatch::default();
    for i in 0..100 {
        w.observe(if i % 2 == 0 { -0.17 } else { -0.16 });
    }
    assert!(w.sd().unwrap() > JULY.spread);
    assert!(w.judge(&JULY).is_ok());
    let mut z = BalanceWatch::default();
    for _ in 0..100 {
        z.observe(0.0);
    }
    assert!(
        z.judge(&JULY).is_ok(),
        "a cleared market is not a pinned one"
    );
    let mut stretch = BalanceWatch::default();
    for i in 0..100 {
        stretch.observe(if i < 60 {
            -1.0 / 6.0
        } else {
            0.01 * ((i % 3) as f64)
        });
    }
    let e = stretch.judge(&JULY).expect_err("a stretch is a pin");
    assert!(e.contains("unbroken"), "{e}");
    let mut few = BalanceWatch::default();
    for _ in 0..31 {
        few.observe(0.2);
    }
    assert!(few.judge(&JULY).unwrap_err().contains("too few samples"));
}

#[test]
fn a_one_sided_market_is_observed() {
    // C13: a one-sided tick is an observation at ±1; only a tick with S = D = 0 is skipped, since
    // its 0 is a convention. A market with demand and no supply on every tick is not clearing:
    // it is pinned at +1, where dropping one-sided ticks would leave too few samples to say.
    let one_sided = at_rest(100, |_, m| {
        m.supply = 0.0;
        m.buyer_fill = 0.0;
        m.seller_fill = 0.0;
    });
    let r = balance(&one_sided, &[seg(0, 100)], &JULY, &names(1));
    failed(&r, "pinned at 1.0");
    let samples = r
        .readings
        .iter()
        .find(|x| x.name == "balance.samples")
        .unwrap();
    assert_eq!(samples.value, 100.0);
    // Ticks with S = D = 0 are not observations.
    let idle = at_rest(100, |t, m| {
        if t % 2 == 0 {
            *m = market(1.0, 0.0, 0.0, 0.0);
        } else {
            m.demand = 1.0 + 0.01 * ((t % 3) as f64);
        }
    });
    let r = balance(&idle, &[seg(0, 100)], &JULY, &names(1));
    let samples = r
        .readings
        .iter()
        .find(|x| x.name == "balance.samples")
        .unwrap();
    assert_eq!(samples.value, 50.0);
}

#[test]
fn conservation_fails_on_a_margin_over_one() {
    // R2: the engine stops a run on a breach; Conservation reads the same numbers a second time.
    // A margin of exactly 1 is the ledger's own pass; the next float up fails, for a tick and for
    // the run, and so does a NaN margin and a run short of its end.
    let n = 100;
    let with = |f: fn(&mut Obs)| {
        let mut o = at_rest(n, |_, _| {});
        f(&mut o[40]);
        o
    };
    passed(&conservation(&with(|o| o.margin = 1.0), n, n));
    passed(&conservation(&with(|o| o.run_margin = 1.0), n, n));
    failed(
        &conservation(&with(|o| o.margin = 1.0f64.next_up()), n, n),
        "a ledger margin above 1",
    );
    failed(
        &conservation(&with(|o| o.run_margin = 1.0f64.next_up()), n, n),
        "a ledger margin above 1",
    );
    failed(
        &conservation(&with(|o| o.margin = f64::NAN), n, n),
        "at tick 40",
    );
    let short = at_rest(n - 1, |_, _| {});
    failed(&conservation(&short, n - 1, n), "stopped at tick 99 of 100");
}

/// A short gate run's reports.
fn gate_reports(n: u64) -> Vec<TickReport> {
    let mut s = sim(&tape(GATE));
    let mut out = Vec::new();
    s.run_until(n, &mut |r| out.push(r.clone())).unwrap();
    out
}

#[test]
fn determinism_fails_on_one_differing_hash() {
    // R8: one flipped hash in the second run, one differing field in a resumed report, one in
    // its hashes, and a replay whose final hash differs, each fail; so do runs that fail.
    let reports = gate_reports(30);
    let hashes: Vec<u64> = reports.iter().map(|r| r.hash).collect();
    let last = reports.last().unwrap().clone();
    let good = DeterminismInputs {
        base: hashes.clone(),
        base_last: Some(last.clone()),
        second: Ok(hashes.clone()),
        replay: Ok(*hashes.last().unwrap()),
        resumes: vec![ResumeRun {
            at: 15,
            outcome: Ok((hashes[15..].to_vec(), last.clone())),
        }],
    };
    passed(&determinism(&good));
    let mut flipped = hashes.clone();
    flipped[7] ^= 1;
    let r = determinism(&DeterminismInputs {
        second: Ok(flipped.clone()),
        ..good.clone()
    });
    failed(&r, "differs");
    let repeat = r
        .readings
        .iter()
        .find(|x| x.name == "determinism.repeat")
        .unwrap();
    assert_eq!(repeat.value, 1.0);
    let mut edited = last.clone();
    edited.markets[2].cleared = edited.markets[2].cleared.next_up();
    let r = determinism(&DeterminismInputs {
        resumes: vec![ResumeRun {
            at: 15,
            outcome: Ok((hashes[15..].to_vec(), edited)),
        }],
        ..good.clone()
    });
    failed(&r, "differs");
    let resume = r
        .readings
        .iter()
        .find(|x| x.name == "determinism.resume")
        .unwrap();
    assert_eq!(resume.value, 1.0);
    let r = determinism(&DeterminismInputs {
        resumes: vec![ResumeRun {
            at: 15,
            outcome: Ok((
                {
                    let mut tail = hashes[15..].to_vec();
                    tail[5] ^= 1;
                    tail
                },
                last.clone(),
            )),
        }],
        ..good.clone()
    });
    failed(&r, "differs");
    failed(
        &determinism(&DeterminismInputs {
            replay: Ok(hashes[3]),
            ..good.clone()
        }),
        "differs",
    );
    for (inputs, why) in [
        (
            DeterminismInputs {
                second: Err("x".into()),
                ..good.clone()
            },
            "the second run failed",
        ),
        (
            DeterminismInputs {
                replay: Err("x".into()),
                ..good.clone()
            },
            "the replay audit failed",
        ),
        (
            DeterminismInputs {
                resumes: vec![ResumeRun {
                    at: 15,
                    outcome: Err("x".into()),
                }],
                ..good.clone()
            },
            "the resume at tick 15 failed",
        ),
    ] {
        failed(&determinism(&inputs), why);
    }
    // The gate reads every float of both reports.
    let mut nan = last.clone();
    nan.markets[0].price = f64::NAN;
    failed(
        &determinism(&DeterminismInputs {
            base_last: Some(nan),
            ..good.clone()
        }),
        "at tick 29, base.markets[0].price",
    );
}

#[test]
fn trades_fails_on_a_silent_window() {
    // Liveness: every market trades in every window, the last full window ending at the run's
    // end when E does not divide n. Over 250 ticks with E = 100 the windows are [0, 100),
    // [100, 200) and [150, 250), so a silence over ticks 150–249 is caught by the last one alone.
    assert_eq!(trade_windows(250, 100), [(0, 100), (100, 200), (150, 250)]);
    assert_eq!(trade_windows(200, 100), [(0, 100), (100, 200)]);
    assert_eq!(trade_windows(99, 100), []);
    let windows = trade_windows(250, 100);
    let silent = |from: u64, to: u64| {
        at_rest(250, move |t, m| {
            if (from..to).contains(&t) {
                *m = market(1.0, 0.0, 1.0, 0.0);
            }
        })
    };
    passed(&trades(&silent(0, 99), &windows, &names(1)));
    failed(
        &trades(&silent(100, 200), &windows, &names(1)),
        "[100, 200)",
    );
    failed(
        &trades(&silent(150, 250), &windows, &names(1)),
        "[150, 250)",
    );
    // A run that stopped early leaves its later windows silent.
    let short = at_rest(180, |_, _| {});
    failed(&trades(&short, &windows, &names(1)), "did not trade");
}

#[test]
fn a_zero_kick_fails() {
    // §7: the realized kick is g(T), the gap on the kicked run's first report; a kick rounding
    // ate, or a kicked run compared with itself, is no kick and fails.
    failed(&kick_of(100, 10, |_| 0.0), "a zero kick");
    failed(
        &kick_of(100, 10, |t| if t == 0 { 0.0 } else { SIZE }),
        "a zero kick",
    );
    passed(&kick_of(100, 10, |t| SIZE * num::exp(-(t as f64) / 3.0)));
}

#[test]
fn a_neutral_direction_fails() {
    // §7: a neutral direction keeps its gap near the kick, so its tail gain is near 1; and one
    // tick near zero on an orbit does not pass, since the tail is read over ⌈tail·H⌉ ticks. The
    // gain is over the realized kick, not the nominal size: a kick rounding shrank to 1e-13 that
    // stays at 1e-13 is neutral, though it is 1e-4 of the 1e-9 asked for.
    failed(&kick_of(100, 10, |_| SIZE), "did not decay");
    // A triangle orbit of period 20 with its zero on the horizon's last tick: H = 211.
    let orbit = |t: usize| SIZE * (((210 - t) % 20) as f64) / 10.0;
    assert_eq!((orbit(0), orbit(210)), (SIZE, 0.0));
    failed(&kick_of(211, 22, orbit), "did not decay");
    // Rounding left a tenth of a thousandth of the kick asked for, and it stays: neutral.
    failed(&kick_of(100, 10, |_| 1e-13), "did not decay");
    failed(
        &kick_of(100, 10, |t| SIZE * (1.0 + t as f64 / 10.0)),
        "did not decay",
    );
    // The readings: the realized kick, and the gains over it.
    let r = kick_of(100, 10, |t| SIZE * num::exp(-(t as f64) / 3.0));
    passed(&r);
    let tail = r
        .readings
        .iter()
        .find(|x| x.name == "kick.gain_tail")
        .unwrap();
    assert!(tail.value < MAX_GAIN && tail.bar == Some(MAX_GAIN));
}

#[test]
fn a_kick_that_swings_out_and_back_fails() {
    // Amended at S2.3: at desk turnover ×16 a 1e-9 kick swings out ×3e9, through dead markets,
    // and comes back to the unstable point it left, where rounding freezes it again, so its tail
    // gain reads as decayed. Its peak does not.
    let swing = |t: usize| {
        if (20..60).contains(&t) {
            1.0
        } else {
            SIZE * num::exp(-(t as f64) / 3.0)
        }
    };
    let r = kick_of(200, 20, swing);
    failed(&r, "did not decay");
    let peak = r
        .readings
        .iter()
        .find(|x| x.name == "kick.gain_peak")
        .unwrap();
    assert!(peak.value > MAX_PEAK && peak.bar == Some(MAX_PEAK));
    let tail = r
        .readings
        .iter()
        .find(|x| x.name == "kick.gain_tail")
        .unwrap();
    assert!(tail.value < MAX_GAIN);
    // A swing within the peak bar is transient amplification, which the tail then judges.
    passed(&kick_of(200, 20, |t| {
        if (20..60).contains(&t) {
            SIZE * 3.0
        } else {
            SIZE * num::exp(-(t as f64) / 3.0)
        }
    }));
}
