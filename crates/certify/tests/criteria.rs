//! Criteria (docs/CERTIFY.md §4; R4, C12, N4): every bar has a value, a unit and a basis, and
//! nothing has a default; the load checks refuse a file that could certify vacuously, and the
//! fit checks refuse one that does not fit its tape.
//!
//! The bars here are test bars on the gate world, 2,080 weekly ticks. None applies the gate's or
//! appb's registered bars (§13): those are committed in S2.4's second commit (the old S2.7).

mod common;

use certify::{Criteria, CriteriaError};
use common::*;
use rustyecon_engine::prelude::Clock;

/// The gate's run: 2,080 weekly ticks from 1750.
const GATE_TICKS: u64 = 2080;

fn clock() -> Clock {
    let t = tape(GATE);
    Clock {
        start: t.header.start,
        ticks_per_year: t.header.ticks_per_year,
    }
}

/// Every battery, with bars that fit the gate's clock.
fn all() -> Vec<String> {
    vec![
        conservation(),
        determinism(&[0.25, 0.5, 0.75]),
        runaway(1e3),
        trades(1.0),
        balance(1e-9, 1e-12, 32, 0.5),
        settles(0.5, 0.9, 0.01, 1e-4),
        kick(1e-9, 10.0, 0.1, 1e-3, 1e6),
    ]
}

fn text_with(batteries: &[String], until_tick: u64, min_segment: f64) -> String {
    let t = tape(GATE);
    criteria_text(
        "gate",
        certify::tape_hash(&t),
        &date_of(&t, until_tick),
        min_segment,
        batteries,
        1e-9,
    )
}

fn good() -> String {
    text_with(&all(), GATE_TICKS, 1.0)
}

fn load(text: &str) -> Result<Criteria, CriteriaError> {
    Criteria::from_ron(text, &file("gate"))
}

fn refused(text: &str) -> CriteriaError {
    match load(text) {
        Ok(_) => panic!("the criteria were expected not to load:\n{text}"),
        Err(e) => e,
    }
}

fn unfit(text: &str) -> CriteriaError {
    let c = load(text).unwrap_or_else(|e| panic!("the criteria load: {e}"));
    match c.fit(&clock()) {
        Ok(f) => panic!("the criteria were expected not to fit: {f:?}"),
        Err(e) => e,
    }
}

/// `batteries` with the one whose text starts with `name` replaced by `with`.
fn swap(name: &str, with: String) -> Vec<String> {
    all()
        .into_iter()
        .map(|b| if b.starts_with(name) { with.clone() } else { b })
        .collect()
}

#[test]
fn criteria_need_a_unit_and_basis_for_every_bar() {
    // R4: a bar is a value, a unit and a basis, with no default. A bare number, a bar without
    // its unit or basis, and a basis that says nothing do not load, and neither does any bar in
    // another unit than its field's.
    let good = good();
    let c = load(&good).expect("the good criteria load");
    assert!(c.fit(&clock()).is_ok());
    let bar = "min_segment: (value: 1.0, unit: Years, basis: Assumed(\"test\"))";
    assert!(good.contains(bar));
    for bad in [
        "min_segment: 1.0",
        "min_segment: (value: 1.0, basis: Assumed(\"test\"))",
        "min_segment: (value: 1.0, unit: Years)",
        "min_segment: (value: 1.0, unit: Years, basis: Assumed(\"test\"), extra: 1)",
    ] {
        let e = refused(&good.replacen(bar, bad, 1));
        assert!(e.path.is_empty(), "{bad}: a parse error, got {e}");
    }
    let e = refused(&good.replacen(bar, &bar.replace("\"test\"", "\" \""), 1));
    assert_eq!(e.path, "min_segment", "{e}");
    for x in ["NaN", "inf", "-inf"] {
        let e = refused(&good.replacen(bar, &bar.replace("1.0", x), 1));
        assert_eq!(e.path, "min_segment", "{x}: {e}");
    }
    // Every bar of the file, each with the wrong unit, is refused at its own path.
    let units: Vec<(usize, &str)> = good.match_indices("unit: ").collect();
    assert_eq!(units.len(), 19, "every bar of the file");
    for (k, (at, _)) in units.iter().enumerate() {
        let rest = &good[at + 6..];
        let unit = &rest[..rest.find(',').unwrap()];
        let wrong = if unit == "Gain" { "Years" } else { "Gain" };
        let text = format!("{}unit: {wrong}{}", &good[..*at], &rest[unit.len()..]);
        let e = refused(&text);
        assert!(
            e.why.contains("bar, where this field is"),
            "bar {k} ({unit} as {wrong}): {e}"
        );
    }
    // A count has its basis too.
    let count = "min_samples: (value: 32, basis: Assumed(\"test\"))";
    assert!(good.contains(count));
    assert!(refused(&good.replacen(count, "min_samples: 32", 1))
        .path
        .is_empty());
    // Every basis that says nothing is refused at its own path, not only a bar's (the
    // verification's coverage finding): the run's length, a count, and the price shocks.
    let blank = |from: &str, to: &str| {
        assert_eq!(good.matches(from).count(), 1, "{from}");
        refused(&good.replacen(from, to, 1)).path
    };
    assert_eq!(
        blank(
            "until_basis: Assumed(\"test\")",
            "until_basis: Assumed(\" \")"
        ),
        "until_basis"
    );
    assert_eq!(
        blank(count, "min_samples: (value: 32, basis: Assumed(\" \"))"),
        "batteries[Balance].min_samples"
    );
    let shocks = with_price_shocks(&good, 1);
    assert!(load(&shocks).is_ok());
    assert_eq!(load(&shocks).unwrap().allowed_price_shocks(), 1);
    assert_eq!(load(&good).unwrap().allowed_price_shocks(), 0);
    assert_eq!(
        refused(&shocks.replacen("basis: Assumed(\"test\"))),", "basis: Assumed(\"\"))),", 1)).path,
        "price_shocks"
    );
    // A value outside its unit's range, for every bar: each refused at its own path.
    let out_of_range: Vec<(Vec<String>, &str)> = vec![
        (swap("Runaway", runaway(1.0)), "batteries[Runaway].bound"),
        (swap("Trades", trades(0.0)), "batteries[Trades].every"),
        (
            swap("Balance", balance(0.0, 1e-12, 32, 0.5)),
            "batteries[Balance].level",
        ),
        (
            swap("Balance", balance(1.0, 1e-12, 32, 0.5)),
            "batteries[Balance].level",
        ),
        (
            swap("Balance", balance(1e-9, 0.0, 32, 0.5)),
            "batteries[Balance].spread",
        ),
        (
            swap("Balance", balance(1e-9, 1.0, 32, 0.5)),
            "batteries[Balance].spread",
        ),
        (
            swap("Balance", balance(1e-9, 1e-12, 32, 0.0)),
            "batteries[Balance].run_share",
        ),
        (
            swap("Balance", balance(1e-9, 1e-12, 32, 1.5)),
            "batteries[Balance].run_share",
        ),
        (
            swap("Settles", settles(-0.1, 0.9, 0.01, 1e-4)),
            "batteries[Settles].w_from",
        ),
        (
            swap("Settles", settles(0.5, 1.0, 0.01, 1e-4)),
            "batteries[Settles].f_from",
        ),
        (
            swap("Settles", settles(0.5, 0.9, 1.0, 1e-4)),
            "batteries[Settles].dead_share",
        ),
        (
            swap("Settles", settles(0.5, 0.9, -0.01, 1e-4)),
            "batteries[Settles].dead_share",
        ),
        (
            swap("Settles", settles(0.5, 0.9, 0.01, 0.0)),
            "batteries[Settles].band",
        ),
        (
            swap("Kick", kick(1e-9, 10.0, 0.0, 1e-3, 1e6)),
            "batteries[Kick].tail",
        ),
        (
            swap("Kick", kick(1e-9, 10.0, 1.5, 1e-3, 1e6)),
            "batteries[Kick].tail",
        ),
        (
            swap("Kick", kick(1e-9, 10.0, 0.1, 0.0, 1e6)),
            "batteries[Kick].max_gain",
        ),
    ];
    for (batteries, path) in out_of_range {
        let e = refused(&text_with(&batteries, GATE_TICKS, 1.0));
        assert_eq!(e.path, path, "{e}");
    }
    // The edges that load: a run of the whole segment, and a dead share of 0.
    assert!(load(&text_with(
        &swap("Balance", balance(1e-9, 1e-12, 32, 1.0)),
        GATE_TICKS,
        1.0
    ))
    .is_ok());
    assert!(load(&text_with(
        &swap("Settles", settles(0.5, 0.9, 0.0, 1e-4)),
        GATE_TICKS,
        1.0
    ))
    .is_ok());
    let rationed = "rationed_below: (value: 1e-9,";
    assert!(good.contains(rationed));
    for x in ["0.0", "1.0"] {
        assert_eq!(
            refused(&good.replacen(rationed, &format!("rationed_below: (value: {x},"), 1)).path,
            "reports.rationed_below",
            "{x}"
        );
    }
}

#[test]
fn criteria_date_matches_its_file() {
    // §4: a file is named <tape>-<date>.ron for the tape and date it holds, and one that
    // supersedes another names an earlier file for the same tape. A retune is a new dated file.
    let good = good();
    assert!(Criteria::from_ron(&good, "criteria/gate-2026-09-26.ron").is_ok());
    let e = Criteria::from_ron(&good, "gate-2026-09-27.ron").unwrap_err();
    assert_eq!(e.path, "date", "{e}");
    let e = Criteria::from_ron(&good, "appb-2026-09-26.ron").unwrap_err();
    assert_eq!(e.path, "tape.name", "{e}");
    for name in ["gate.ron", "gate-2026-9-26.ron", "gate-2026-09-26.txt", ""] {
        assert!(Criteria::from_ron(&good, name).is_err(), "{name}");
    }
    let sup = |s: &str| good.replacen("supersedes: None", &format!("supersedes: Some({s:?})"), 1);
    assert!(load(&sup("gate-2026-09-01.ron")).is_ok());
    assert!(load(&sup("criteria/gate-2025-12-31.ron")).is_ok());
    for bad in [
        "gate-2026-09-26.ron",
        "gate-2026-10-01.ron",
        "appb-2026-09-01.ron",
        "gate.ron",
    ] {
        assert_eq!(refused(&sup(bad)).path, "supersedes", "{bad}");
    }
    let e = refused(&good.replacen("format: 1", "format: 2", 1));
    assert_eq!(e.path, "format", "{e}");
    assert!(
        refused(&good.replacen("reason: \"test\"", "reason: \"\"", 1))
            .path
            .contains("reason")
    );
}

#[test]
fn criteria_round_trip() {
    // The criteria's identity is their canonical RON: to_ron reads back equal, with the same
    // hash, and any change to a bar, its basis included, changes the hash.
    let c = load(&good()).unwrap();
    let again = load(&c.to_ron()).expect("to_ron reads back");
    assert_eq!(again, c);
    assert_eq!(again.hash(), c.hash());
    assert!(c.to_ron().starts_with("Criteria(\n    format: 1,"));
    let moved = load(&good().replacen("Assumed(\"test\")", "Assumed(\"test.\")", 1)).unwrap();
    assert_ne!(moved.hash(), c.hash());
    let listed: Vec<String> = c.listed().iter().map(|b| b.to_string()).collect();
    assert_eq!(
        listed,
        [
            "Conservation",
            "Determinism",
            "Runaway",
            "Trades",
            "Balance",
            "Settles",
            "Kick"
        ]
    );
}

#[test]
fn criteria_without_runaway_do_not_certify() {
    // C12, N4: Conservation, Determinism and Runaway are in every criteria file, so no scored
    // run skips its ledger, its determinism or its runaway bound. A file without one does not
    // load, and no battery is listed twice.
    for missing in ["Conservation", "Determinism", "Runaway"] {
        let without: Vec<String> = all()
            .into_iter()
            .filter(|b| !b.starts_with(missing))
            .collect();
        let e = refused(&text_with(&without, GATE_TICKS, 1.0));
        assert!(e.why.contains(missing) && e.why.contains("C12"), "{e}");
    }
    // A certificate hand-edited so that its criteria list no Runaway is refused on readback:
    // resealed, it fails C12, which the file does not say.
    let gate = tape(GATE);
    let listed: Vec<String> = all()
        .into_iter()
        .filter(|b| !b.starts_with("Settles") && !b.starts_with("Kick"))
        .collect();
    let c = criteria("gate", &text_with(&listed, GATE_TICKS, 1.0));
    let cert = certify::certify(
        &gate,
        certify::Scoring::Criteria {
            criteria: &c,
            file: &file("gate"),
        },
        &build(),
        &mut |_| {},
    )
    .unwrap()
    .certificate;
    assert_eq!(
        cert.verdict(),
        certify::Verdict::Pass,
        "{:?}",
        cert.failures()
    );
    let text = cert.to_ron();
    let from = text.find("listed: [").unwrap();
    let to = from + text[from..].find(']').unwrap();
    let list = &text[from..to];
    assert!(list.contains("Runaway,"));
    let edited = format!(
        "{}{}{}",
        &text[..from],
        list.replacen("Runaway,", "", 1),
        &text[to..]
    );
    assert!(matches!(
        certify::Certificate::from_ron(&edited),
        Err(certify::CertificateError::EditedVerdict { .. })
    ));
    let mut twice = all();
    twice.push(runaway(1e4));
    assert_eq!(
        refused(&text_with(&twice, GATE_TICKS, 1.0)).path,
        "batteries[Runaway]"
    );
}

#[test]
fn settles_without_kick_does_not_load() {
    // C12: Settles comes only with Kick, so a rounding freeze at an unstable point cannot
    // certify as at rest; and Kick only with Settles.
    for (drop, keep) in [("Kick", "Settles"), ("Settles", "Kick")] {
        let without: Vec<String> = all().into_iter().filter(|b| !b.starts_with(drop)).collect();
        assert!(without.iter().any(|b| b.starts_with(keep)));
        let e = refused(&text_with(&without, GATE_TICKS, 1.0));
        assert!(e.why.contains("Settles and Kick"), "{e}");
    }
    let neither: Vec<String> = all()
        .into_iter()
        .filter(|b| !b.starts_with("Kick") && !b.starts_with("Settles"))
        .collect();
    assert!(load(&text_with(&neither, GATE_TICKS, 1.0)).is_ok());
}

#[test]
fn a_run_of_no_ticks_does_not_load() {
    // No vacuous pass: a run with `until` at the start holds no tick, and one shorter than a
    // segment holds no whole segment; neither fits.
    let e = unfit(&text_with(&all(), 0, 1.0));
    assert_eq!(e.path, "until", "{e}");
    let e = unfit(&text_with(&all(), 51, 1.0));
    assert_eq!(e.path, "until", "{e}");
    // One segment of 52 ticks is a run.
    let f = load(&text_with(&all(), 52, 1.0))
        .unwrap()
        .fit(&clock())
        .unwrap();
    assert_eq!((f.until, f.min_segment), (52, 52));
    // min_segment is a span above 0 years.
    assert_eq!(
        refused(&text_with(&all(), GATE_TICKS, 0.0)).path,
        "min_segment"
    );
}

#[test]
fn determinism_needs_a_resume_inside_the_run() {
    // No vacuous pass: at least one resume, each strictly inside the run, no two on one tick.
    let at = |shares: &[f64]| text_with(&swap("Determinism", determinism(shares)), GATE_TICKS, 1.0);
    assert!(refused(&at(&[])).path.ends_with("resume_at"));
    assert!(refused(&at(&[0.0])).path.ends_with("resume_at[0]"));
    assert!(refused(&at(&[1.0])).path.ends_with("resume_at[0]"));
    assert!(refused(&at(&[0.5, 1.0])).path.ends_with("resume_at[1]"));
    // 0.5 and 0.5002 of 2,080 ticks are both tick 1,040.
    let e = unfit(&at(&[0.5, 0.5002]));
    assert!(e.path.ends_with("resume_at[1]"), "{e}");
    // A share that floors to tick 0: 1e-4 of 2,080 ticks.
    let e = unfit(&at(&[1e-4]));
    assert!(e.path.ends_with("resume_at[0]"), "{e}");
    let f = load(&at(&[0.25, 0.5, 0.75]))
        .unwrap()
        .fit(&clock())
        .unwrap();
    assert_eq!(f.resume_at, [520, 1040, 1560]);
}

#[test]
fn kick_horizon_needs_two_ticks() {
    // No vacuous pass: a kick runs at least two ticks and reads at least two, and its size
    // moves a float.
    let with = |size: f64, horizon: f64, tail: f64| {
        text_with(
            &swap("Kick", kick(size, horizon, tail, 1e-3, 1e6)),
            GATE_TICKS,
            1.0,
        )
    };
    assert!(refused(&with(1e-9, 0.0, 0.1)).path.ends_with("horizon"));
    // One week of 52: H = 1.
    let e = unfit(&with(1e-9, 1.0 / 52.0, 1.0));
    assert!(e.path.ends_with("horizon"), "{e}");
    // H = 520 and a tail of 1/520: one tick.
    let e = unfit(&with(1e-9, 10.0, 1.0 / 520.0));
    assert!(e.path.ends_with("tail"), "{e}");
    let f = load(&with(1e-9, 10.0, 0.1)).unwrap().fit(&clock()).unwrap();
    assert_eq!((f.kick_horizon, f.kick_tail), (Some(520), Some(52)));
    // Below 2^-53, 1 ± size rounds to 1: no kick at all.
    for size in [1e-17, 0.0, 1.0] {
        assert!(
            refused(&with(size, 10.0, 0.1)).path.ends_with("size"),
            "{size}"
        );
    }
    // Gain and peak: a gain in (0, 1), a peak above 1.
    let bars =
        |g: f64, p: f64| text_with(&swap("Kick", kick(1e-9, 10.0, 0.1, g, p)), GATE_TICKS, 1.0);
    assert!(refused(&bars(1.0, 1e6)).path.ends_with("max_gain"));
    assert!(refused(&bars(1e-3, 1.0)).path.ends_with("max_peak"));
}

#[test]
fn balance_needs_two_samples() {
    // BalanceWatch needs two observations for a spread, and a segment must be able to hold the
    // registered number.
    let with = |min: u64| {
        text_with(
            &swap("Balance", balance(1e-9, 1e-12, min, 0.5)),
            GATE_TICKS,
            1.0,
        )
    };
    for min in [0, 1] {
        assert!(refused(&with(min)).path.ends_with("min_samples"), "{min}");
    }
    assert!(load(&with(2)).unwrap().fit(&clock()).is_ok());
    // A segment of one year holds 52 ticks: 53 samples cannot fit.
    assert!(load(&with(52)).unwrap().fit(&clock()).is_ok());
    let e = unfit(&with(53));
    assert!(e.path.ends_with("min_samples"), "{e}");
}

#[test]
fn trades_needs_one_full_window() {
    // A Trades window longer than the run is never scored in full.
    let with = |years: f64| text_with(&swap("Trades", trades(years)), GATE_TICKS, 1.0);
    let e = unfit(&with(41.0));
    assert!(e.path.ends_with("every"), "{e}");
    assert_eq!(
        load(&with(40.0))
            .unwrap()
            .fit(&clock())
            .unwrap()
            .trades_every,
        Some(GATE_TICKS)
    );
    // And F holds two ticks of the shortest segment: F from 0.99 of 52 ticks holds none.
    let e = unfit(&text_with(
        &swap("Settles", settles(0.5, 0.99, 0.01, 1e-4)),
        GATE_TICKS,
        1.0,
    ));
    assert!(e.path.ends_with("f_from"), "{e}");
    assert!(refused(&text_with(
        &swap("Settles", settles(0.9, 0.5, 0.01, 1e-4)),
        GATE_TICKS,
        1.0
    ))
    .path
    .ends_with("f_from"));
}
