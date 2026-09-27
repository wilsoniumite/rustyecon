//! The in-memory store and decimation (docs/GUI.md §3.4, §8.1): thinning keeps every column's
//! true extremes and draws only recorded points, a non-finite value stops ingestion with the
//! series and tick named (U10), and the store keeps the run's ticks contiguous.

mod common;

use common::{collecting, drain, tape_of, GATE};
use rustyecon_engine::prelude::*;
use rustyecon_gui::run::{
    decimate, At, Cmd, Decimator, IngestError, Measure, Obs, Origin, RunStatus, Runner, Series,
    SeriesKey, Store,
};
use rustyecon_gui::vm;

/// A deterministic walk of `n` ticks with one-tick spikes up and down and two gaps, by
/// xorshift: the kind of line whose spikes a naive thinning drops.
fn walk(n: u64) -> Vec<(u64, f64)> {
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut v = 0.0;
    let mut out = Vec::new();
    for t in 0..n {
        let r = next();
        if (5_000..5_100).contains(&t) || t == 70_000 {
            continue;
        }
        v += (r % 2_001) as f64 / 1_000.0 - 1.0;
        let x = match r % 1_000 {
            0 => v + 500.0,
            1 => v - 500.0,
            _ => v,
        };
        out.push((t, x));
    }
    out
}

#[test]
fn decimation_keeps_extremes() {
    // docs/GUI.md §3.4: the plot thins a series to about two points per pixel column. Every drawn
    // vertex is a recorded (tick, value), each column keeps its true minimum and maximum, and a
    // gap in the record is never bridged.
    let points = walk(100_000);
    let s = Series::from_points(&points);
    let spikes = points
        .iter()
        .filter(|p| p.1 > 300.0 || p.1 < -300.0)
        .count();
    assert!(spikes > 50, "the walk has one-tick spikes: {spikes}");
    for (lo, hi, columns) in [
        (0, 100_000, 800),
        (0, 100_000, 3),
        (1_234, 56_789, 1_600),
        (99_990, 100_000, 5),
        (0, 100_000, 200_000),
    ] {
        let segments = decimate(&s, lo, hi, columns);
        let drawn: Vec<(u64, f64)> = segments.concat();
        // Recorded points only, in increasing tick order, inside the range.
        for &(t, v) in &drawn {
            assert_eq!(s.at(t), Some(v), "tick {t} is not recorded as drawn");
            assert!((lo..hi).contains(&t));
        }
        assert!(drawn.windows(2).all(|w| w[0].0 < w[1].0));
        // No segment bridges a gap: every tick between a segment's ends is recorded.
        for seg in &segments {
            let (a, b) = (seg[0].0, seg[seg.len() - 1].0);
            let (i, j) = s.range(a, b + 1);
            assert_eq!((j - i) as u64, b - a + 1, "segment {a}..={b} bridges a gap");
        }
        // Each column's true extremes, per unbroken stretch of the record, are drawn.
        let span = hi - lo;
        let width = span.div_ceil(columns as u64);
        let (from, to) = s.range(lo, hi);
        let recorded: Vec<(u64, f64)> = s.ticks()[from..to]
            .iter()
            .copied()
            .zip(s.values()[from..to].iter().copied())
            .collect();
        let mut groups: Vec<Vec<(u64, f64)>> = Vec::new();
        for (k, &(t, v)) in recorded.iter().enumerate() {
            let same = k > 0
                && recorded[k - 1].0 + 1 == t
                && (recorded[k - 1].0 - lo) / width == (t - lo) / width;
            if same {
                groups.last_mut().unwrap().push((t, v));
            } else {
                groups.push(vec![(t, v)]);
            }
        }
        for g in &groups {
            let min = g.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
            let max = g.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
            let inside = |v: f64| drawn.iter().any(|p| p.1 == v && g.contains(p));
            assert!(
                inside(min) && inside(max),
                "a column starting {} lost an extreme",
                g[0].0
            );
        }
        // About two points a column: never more than two per column and stretch.
        assert!(drawn.len() <= 2 * groups.len());
        // The global extremes survive any thinning.
        let gmin = recorded.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let gmax = recorded
            .iter()
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(drawn.iter().any(|p| p.1 == gmin) && drawn.iter().any(|p| p.1 == gmax));
        if width == 1 {
            assert_eq!(drawn, recorded, "one tick a column draws every point");
        }
    }
}

#[test]
fn decimation_extends_as_ticks_arrive() {
    // The plot cache extends as ticks arrive: a decimator fed a growing series, a few points at
    // a time, draws what one pass over the whole series draws.
    let points = walk(20_000);
    let whole = Series::from_points(&points);
    let mut once = Decimator::new(0, 37);
    once.extend(&whole);
    let mut grown = Decimator::new(0, 37);
    let mut n = 0;
    let mut step = 1;
    while n < points.len() {
        n = (n + step).min(points.len());
        step = step * 3 % 101 + 1;
        grown.extend(&Series::from_points(&points[..n]));
    }
    assert_eq!(grown.segments(), once.segments());
    assert_eq!(
        once.segments(),
        decimate(&whole, 0, 20_000, 20_000_usize.div_ceil(37))
    );
    assert_eq!(
        once.segments().len(),
        2,
        "one gap in the first 20,000 ticks"
    );
}

fn key(s: &str) -> Key {
    Key::new(s).expect("a key")
}

/// A Runner's observations of the gate's first `ticks` ticks, in slices of `slice`.
fn gate_obs(ticks: u64, slice: u32) -> Vec<Obs> {
    let (mut r, seen) = collecting();
    r.handle(Cmd::Load {
        tape: Box::new(tape_of(GATE)),
        from: None,
    });
    r.handle(Cmd::Step(ticks));
    while Runner::advance(&mut r, slice).busy {}
    drain(&seen)
}

#[test]
fn nonfinite_ingest_stops_with_the_series_named() {
    // U10: a non-finite value at ingest stops ingestion and names the series and tick. The row
    // that holds it is dropped whole, so every series ends at the last good tick, and every
    // later observation is refused with the same error.
    let mut obs = gate_obs(12, 5);
    // The price of bread in the village: a series well inside the catalogue, so that naming
    // any other one fails.
    let bread = SeriesKey {
        measure: Measure::Price,
        at: At::Market {
            node: key("village"),
            good: key("bread"),
        },
    };
    let mut catalogue = Vec::new();
    let mut poisoned = false;
    for o in &mut obs {
        if let Obs::Batch(b) = o {
            catalogue.extend(b.new_series.iter().cloned());
            let i = catalogue
                .iter()
                .position(|k| *k == bread)
                .expect("the price") as u32;
            for row in b.rows.iter_mut().filter(|r| r.tick == 7) {
                for cell in row.cells.iter_mut().filter(|c| c.0 == i) {
                    cell.1 = f64::NAN;
                    poisoned = true;
                }
            }
        }
    }
    assert!(poisoned, "tick 7 has a price of bread in town");
    let later = obs.len()
        - obs
            .iter()
            .position(|o| matches!(o, Obs::Batch(b) if b.rows.iter().any(|r| r.tick == 7)))
            .unwrap()
        - 1;
    let mut store = Store::default();
    let mut errors = Vec::new();
    for o in obs {
        if let Err(e) = store.ingest(o) {
            errors.push(e);
        }
    }
    assert_eq!(
        errors.len(),
        1 + later,
        "every later observation is refused"
    );
    assert!(errors
        .windows(2)
        .all(|w| w[0].to_string() == w[1].to_string()));
    match &errors[0] {
        IngestError::NonFinite {
            series,
            tick,
            value,
        } => {
            assert_eq!(series, &bread);
            assert_eq!(*tick, 7);
            assert!(value.is_nan());
        }
        other => panic!("{other:?}"),
    }
    let text = errors[0].to_string();
    assert!(
        text.contains("price at village/bread") && text.contains("tick 7"),
        "{text}"
    );
    // Ticks 0 to 6 are recorded, whole; nothing of tick 7 or after.
    assert_eq!(store.hashes().len(), 7);
    assert_eq!(store.tick(), 7);
    assert_eq!(store.status(), RunStatus::Stopped);
    assert!(!store.can_run());
    for k in store.catalogue() {
        let s = store.series(k).expect("a catalogued series");
        assert!(s.last().is_none_or(|(t, _)| t <= 6), "{k} runs past tick 6");
    }
    assert_eq!(store.series(&bread).unwrap().len(), 7);
    assert!(store.catalogue().iter().position(|k| *k == bread) > Some(0));
    // The health chip says so.
    let health = vm::toolbar::build(&store, Origin::Run).health;
    assert_eq!(health.status, vm::toolbar::Status::Stopped);
    assert_eq!(health.stopped.as_deref(), Some(text.as_str()));
    // A clean run of the same ticks records all twelve.
    let mut clean = Store::default();
    for o in gate_obs(12, 5) {
        clean.ingest(o).expect("finite values ingest");
    }
    assert_eq!(clean.hashes().len(), 12);
    assert_eq!(clean.series(&bread).unwrap().len(), 12);
}

#[test]
fn the_store_keeps_the_runs_ticks_contiguous() {
    // A batch that does not continue the run's ticks, or its catalogue, stops ingestion.
    let obs = gate_obs(10, 4);
    let mut store = Store::default();
    let mut batches = Vec::new();
    for o in obs {
        match o {
            Obs::Batch(b) => batches.push(b),
            other => store.ingest(other).expect("a load and its state"),
        }
    }
    store
        .ingest(Obs::Batch(batches[0].clone()))
        .expect("the first batch");
    let skipped = store.ingest(Obs::Batch(batches[2].clone()));
    assert_eq!(
        skipped,
        Err(IngestError::OutOfOrder {
            expected: 4,
            got: 8
        })
    );
    assert_eq!(store.status(), RunStatus::Stopped);
    let mut fresh = Store::default();
    assert_eq!(
        fresh.ingest(Obs::Batch(batches[0].clone())),
        Err(IngestError::NotLoaded)
    );
    // Recorded values are the report's: the price of bread in town at tick 3.
    let mut store = Store::default();
    for o in gate_obs(10, 4) {
        store.ingest(o).unwrap();
    }
    let mut sim = Sim::new(&tape_of(GATE)).unwrap();
    let town: NodeId = sim.world().id_of("town").unwrap();
    let bread_id: GoodId = sim.world().id_of("bread").unwrap();
    let mut price = None;
    sim.run_until(4, &mut |r| {
        if r.tick == 3 {
            price = r
                .markets
                .iter()
                .find(|m| m.node == town && m.good == bread_id)
                .map(|m| m.price);
        }
    })
    .unwrap();
    assert!(price.is_some());
    let bread = SeriesKey {
        measure: Measure::Price,
        at: At::Market {
            node: key("town"),
            good: key("bread"),
        },
    };
    assert_eq!(store.series(&bread).unwrap().at(3), price);
}
