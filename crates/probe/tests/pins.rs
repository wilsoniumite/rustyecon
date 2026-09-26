//! The probe's summaries, pinned to its report (docs/CERTIFY.md §11, C11; S2.5). The probe reads
//! certify's oracle-free measures (the runaway bound, a market that traded, rationed fills,
//! spoilage, the provider's transfer, troughs, at rest in F and the dead-share rule), so each
//! has one definition. These tests show the move changed no class and no reported number of
//! docs/probe/results/battery.csv, the table REPORT §3 cites at `55c9e88`: the class, ticks to
//! tolerance, dead ticks, the worst fill, the troughs and the transfer shortfall, at the
//! precision the table prints them (`make_results.py` and `transient.py`, 2026-09-26).
//!
//! `probe_summaries_unchanged` runs three of the 57 rows and is fast.
//! `probe_battery_csv_unchanged` runs all 57, the negative control's family row (July's step
//! rule: 57 DIVERGED by the bound, at most 337 dead ticks) and the shock history bcycle(1500,80),
//! DEAD with 125 dead ticks in each b′ = 0.8 segment and 116 in each return to 0.4 (REPORT §3).
//! It is ignored and run by name in scripts/gate.sh. The troughs are compared as the table was
//! made: the summary's six significant digits first, then four decimals.

use probe::harness::{classify, run, Class, Record, Summary};
use probe::perturb::{battery, family};
use probe::protocol::RUN_TICKS;
use probe::setup::{ScaleRule, Setup};

const BATTERY: &str = include_str!("../../../docs/probe/results/battery.csv");
const FAMILIES: &str = include_str!("../../../docs/probe/results/families.csv");

/// Python's `%.3g`, as `make_results.py` printed the shortfall and the peak.
fn g3(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    let e = format!("{x:.2e}");
    let (mantissa, exp) = e.split_once('e').expect("an exponent");
    let exp: i32 = exp.parse().expect("an exponent");
    let strip = |s: String| {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    };
    if (-4..3).contains(&exp) {
        let decimals = usize::try_from(2 - exp).unwrap_or(0);
        strip(format!("{x:.decimals$}"))
    } else {
        let sign = if exp < 0 { '-' } else { '+' };
        format!(
            "{}e{sign}{:02}",
            strip(mantissa.to_string()),
            exp.unsigned_abs()
        )
    }
}

/// One run of a setup: its record, its summary, and the worst fill over every tick and market
/// side, as `transient.py` read it from the per-tick CSV.
struct Ran {
    rec: Record,
    sum: Summary,
    worst_fill: f64,
}

fn ran(setup: &Setup, name: &str) -> Ran {
    let mut worst_fill: f64 = 1.0;
    let rec = run(setup, name, RUN_TICKS, &mut |row| {
        for m in 0..4 {
            worst_fill = worst_fill.min(row.buyer_fill[m]).min(row.seller_fill[m]);
        }
    })
    .unwrap_or_else(|e| panic!("{name}: {e}"));
    let sum = classify(&rec);
    Ran {
        rec,
        sum,
        worst_fill,
    }
}

/// The row of battery.csv for `name`, by column.
fn row(name: &str) -> Vec<&'static str> {
    let mut lines = BATTERY.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    assert_eq!(header[0], "run");
    assert_eq!(header[14], "transfer_short_coin");
    lines
        .map(|l| l.split(',').collect::<Vec<_>>())
        .find(|r| r[0] == name)
        .unwrap_or_else(|| panic!("battery.csv has no row {name}"))
}

/// A run's summary as battery.csv prints it: class, ticks to tolerance, dead ticks, dead ticks
/// in W, worst fill, the four troughs and the transfer shortfall.
fn printed(r: &Ran) -> Vec<String> {
    let mut v = vec![
        r.sum.class.name().to_string(),
        r.sum
            .in_tol_from
            .map_or_else(|| "-".to_string(), |t| t.to_string()),
        r.sum.dead[0].to_string(),
        r.sum.dead[1].to_string(),
        format!("{:.4}", r.worst_fill),
    ];
    for (low, _) in r.rec.trough {
        let summary: f64 = format!("{low:.6e}").parse().expect("a float");
        v.push(format!("{summary:.4}"));
    }
    v.push(g3(r.rec.transfer_short));
    v
}

/// The same columns of battery.csv's row.
fn committed(name: &str) -> Vec<String> {
    let r = row(name);
    [2, 4, 7, 8, 9, 10, 11, 12, 13, 14]
        .iter()
        .map(|&i| r[i].to_string())
        .collect()
}

#[test]
fn python_g_is_matched() {
    // The formatter the pins compare with: Python's %.3g on the values battery.csv holds.
    for (x, s) in [
        (0.0, "0"),
        (4.4, "4.4"),
        (1.79, "1.79"),
        (255.0, "255"),
        (24.2, "24.2"),
        (770.4, "770"),
        (8.074, "8.07"),
        (2320.0, "2.32e+03"),
        (0.000123, "0.000123"),
    ] {
        assert_eq!(g3(x), s, "{x}");
    }
}

#[test]
fn probe_summaries_unchanged() {
    // Three rows of battery.csv, from Tiers 1 and 3: two CONVERGED runs with no dead tick, and
    // b′ = 0.8 from genesis, with 110 dead ticks, a worst fill of 0.19 and a shortfall of 770.
    let setup = Setup::registered(52);
    for name in ["w*1.05", "r*1.05", "b=0.8@genesis"] {
        let r = ran(&setup, name);
        assert_eq!(printed(&r), committed(name), "{name}");
    }
}

#[test]
#[ignore = "runs the probe's 57-run battery, its negative control and a 20,000-tick history, \
            about 15 s of CPU in release, on up to 16 threads: gate.sh runs it by name (C11)"]
fn probe_battery_csv_unchanged() {
    // All 57 rows of battery.csv; the negative control's row of families.csv; and the shock
    // history, whose class the moved dead-tick measures decide.
    let names: Vec<String> = battery().into_iter().map(|(n, _)| n).collect();
    assert_eq!(names.len(), 57);
    let registered = Setup::registered(52);
    let mut step = Setup::registered(52);
    step.scale = ScaleRule::july_step();
    let jobs = std::thread::available_parallelism().map_or(4, |n| n.get().min(16));
    let run_all = |setup: &Setup, names: &[String]| -> Vec<(String, Ran)> {
        let mut out: Vec<(usize, String, Ran)> = Vec::new();
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..jobs)
                .map(|j| {
                    s.spawn(move || {
                        names
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| i % jobs == j)
                            .map(|(i, n)| (i, n.clone(), ran(setup, n)))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            for h in handles {
                out.extend(h.join().expect("a run thread"));
            }
        });
        out.sort_by_key(|(i, _, _)| *i);
        out.into_iter().map(|(_, n, r)| (n, r)).collect()
    };
    for (name, r) in run_all(&registered, &names) {
        assert_eq!(printed(&r), committed(&name), "{name}");
    }
    // The negative control: July's step rule on the same 57 runs.
    let control = run_all(&step, &names);
    let fam = FAMILIES
        .lines()
        .find(|l| l.starts_with("negative control"))
        .expect("families.csv's negative control");
    let cols: Vec<&str> = fam.split(',').collect();
    let diverged = control
        .iter()
        .filter(|(_, r)| r.sum.class == Class::Diverged)
        .count();
    let most_dead = control.iter().map(|(_, r)| r.sum.dead[0]).max().unwrap();
    let most_dead_w = control.iter().map(|(_, r)| r.sum.dead[1]).max().unwrap();
    let least_good = control
        .iter()
        .map(|(_, r)| r.rec.trough[3].0)
        .fold(f64::INFINITY, f64::min);
    assert_eq!(
        [
            control.len().to_string(),
            diverged.to_string(),
            most_dead.to_string(),
            most_dead_w.to_string(),
            format!("{least_good:.3}"),
        ],
        [cols[1], cols[3], cols[9], cols[10], cols[11]],
        "{fam}"
    );
    // The shock history over 20,000 ticks: DEAD, 611 dead ticks, 368 in W; each b′ = 0.8
    // segment has 125 and each return to 0.4 has 116.
    let history = &family("history").unwrap()[0];
    let h = ran(&registered, history);
    let fam = FAMILIES
        .lines()
        .find(|l| l.starts_with("\"history bcycle(1500,80), 20,000 ticks\""))
        .expect("families.csv's history row");
    let cols: Vec<&str> = fam.rsplit(',').collect();
    assert_eq!(h.sum.class, Class::Dead);
    assert_eq!(
        [h.sum.dead[0].to_string(), h.sum.dead[1].to_string()],
        [cols[2], cols[1]],
        "{fam}"
    );
    let setup = &h.rec.setup;
    let mut segments: Vec<(u64, u64, f64, f64)> = Vec::new();
    let total = h.rec.dead.len() as u64;
    let mut from = 0;
    for t in 1..=total {
        if t == total || setup.b_at(t) != setup.b_at(from) {
            let before = if from == 0 {
                f64::NAN
            } else {
                setup.b_at(from - 1)
            };
            segments.push((from, t, setup.b_at(from), before));
            from = t;
        }
    }
    let dead_in = |a: u64, b: u64| {
        h.rec.dead[a as usize..b as usize]
            .iter()
            .filter(|&&d| d)
            .count()
    };
    let mut checked = (0, 0);
    for &(a, b, level, before) in &segments {
        if level == 0.8 {
            assert_eq!(dead_in(a, b), 125, "b′ = 0.8 over [{a}, {b})");
            checked.0 += 1;
        }
        if level == 0.4 && before == 0.2 {
            assert_eq!(dead_in(a, b), 116, "the return to 0.4 over [{a}, {b})");
            checked.1 += 1;
        }
    }
    assert!(checked.0 >= 2 && checked.1 >= 2, "{segments:?}");
}
