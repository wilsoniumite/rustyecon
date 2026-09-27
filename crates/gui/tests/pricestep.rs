//! The price-step explainer and the log waterfall (docs/GUI.md §4, "Why is this price 12.3?";
//! §9 G1's gate): the explainer, which calls markets' own `imbalance` and `next_price` on each
//! tick's recorded inputs, equals the engine's next price bit for bit on every tick of the
//! gate world, and of the Appendix B world; and the waterfall's Σ k·x and residual add up to
//! ln(p/p₀).

mod common;

use common::{driver, edit, tape_of, wait_for, wait_paused, APPB, APPB_TICKS, GATE, GATE_TICKS};
use egui_kittest::Harness;
use rustyecon_engine::num;
use rustyecon_engine::prelude::*;
use rustyecon_gui::drive::Driver;
use rustyecon_gui::run::{At, Cmd, Measure, Obs, PauseReason, SeriesKey, Store};
use rustyecon_gui::vm::pricestep::{explain, waterfall, ExplainerVm, WaterfallVm};

/// A store of `t` run to `until` through a `ThreadDriver`, and the engine's own reports of the
/// same run, from `Sim::new` and `run_until`.
fn record(t: &Tape, until: u64) -> (Store, Vec<TickReport>) {
    let mut d = driver();
    let mut log = Vec::new();
    d.send(Cmd::Load {
        tape: Box::new(t.clone()),
        from: None,
    });
    wait_for(&mut d, &mut log, |o| matches!(o, Obs::Loaded { .. }));
    d.send(Cmd::Run {
        until: Some(until),
        max_tps: None,
    });
    assert_eq!(
        wait_paused(&mut d, &mut log),
        (until, PauseReason::Reached(until))
    );
    drop(d);
    let mut store = Store::default();
    for o in log {
        store.ingest(o).expect("every observation ingests");
    }
    let mut sim = Sim::new(t).expect("the tape loads");
    let mut reports = Vec::new();
    sim.run_until(until, &mut |r| reports.push(r.clone()))
        .expect("the run succeeds");
    (store, reports)
}

/// Every tick and market of a run: the explainer's next price is the engine's, bit for bit.
fn every_tick(name: &str, text: &str, until: u64) -> Store {
    let t = tape_of(text);
    let (store, reports) = record(&t, until);
    let w = store.world().expect("loaded").clone();
    let mut checked = 0;
    for r in &reports {
        for m in &r.markets {
            let node = w.key_of(m.node).expect("a node key");
            let good = w.key_of(m.good).expect("a good key");
            let e = explain(&store, node, good, r.tick)
                .unwrap_or_else(|why| panic!("{name} {node}/{good} tick {}: {why}", r.tick));
            assert_eq!(
                e.next.to_bits(),
                m.next_price.to_bits(),
                "{name} {node}/{good} tick {}: {:?} against the engine's {:?}",
                r.tick,
                e.next,
                m.next_price
            );
            assert_eq!(e.recorded.map(f64::to_bits), Some(m.next_price.to_bits()));
            assert_eq!(e.equal, Some(true));
            assert_eq!(e.price.to_bits(), m.price.to_bits());
            checked += 1;
        }
    }
    assert_eq!(checked, reports.len() * w.markets().count(), "{name}");
    println!("{name}: the explainer equals next_price at {checked} (tick, market) pairs");
    store
}

#[test]
fn the_explainer_equals_next_price_on_every_gate_tick() {
    let store = every_tick("gate", GATE, GATE_TICKS);
    // The gate's rate is 5.2 a year, read as a log step: k = 5.2/52 = 0.1 per tick.
    let town = Key::new("town").unwrap();
    let bread = Key::new("bread").unwrap();
    let e = explain(&store, &town, &bread, 0).unwrap();
    assert_eq!(e.rate.as_str(), "rate.bread");
    assert_eq!(e.method, "log_step");
    assert_eq!(e.k, 5.2 / 52.0);
    assert_eq!(e.kx.to_bits(), (e.k * e.imbalance).to_bits());
    // A tick past the record has no inputs, and says so.
    assert!(explain(&store, &town, &bread, GATE_TICKS + 5)
        .unwrap_err()
        .contains("not recorded"));
}

#[test]
fn the_explainer_equals_next_price_on_every_appb_tick() {
    every_tick("appb", APPB, APPB_TICKS);
}

#[test]
fn the_waterfall_adds_up_to_the_log_price() {
    // ln(p_t/p₀) = Σ k·x + residual at the cursor, the bins' Σ k·x is the whole's, each bin's
    // level is ln(p/p₀) at its end, and the bins tile the span a year at a time. The gate holds
    // a one-sided market's price (`Hold`) and no event of it moves a price, so its residual is
    // what the holds did not step, −Σ k·x over the one-sided ticks, and rounding.
    let t = tape_of(GATE);
    let (store, reports) = record(&t, GATE_TICKS);
    let town = Key::new("town").unwrap();
    for good in ["bread", "fuel", "grain"] {
        let good = Key::new(good).unwrap();
        for cursor in [Some(1), Some(51), Some(800), None] {
            let w = waterfall(&store, &town, &good, cursor).unwrap();
            let tick = cursor.unwrap_or(GATE_TICKS - 1);
            assert_eq!(w.tick, tick);
            let p = |t: u64| {
                let r = &reports[t as usize];
                r.markets
                    .iter()
                    .find(|m| {
                        store.world().unwrap().key_of(m.node) == Some(&town)
                            && store.world().unwrap().key_of(m.good) == Some(&good)
                    })
                    .unwrap()
                    .price
            };
            assert_eq!(w.p0.to_bits(), p(0).to_bits());
            assert_eq!(w.p.to_bits(), p(tick).to_bits());
            assert_eq!(w.level.to_bits(), num::ln(w.p / w.p0).to_bits());
            assert_eq!(w.residual.to_bits(), (w.level - w.explained).to_bits());
            let sum: f64 = w.bins.iter().map(|b| b.kx).sum();
            assert!((sum - w.explained).abs() <= 1e-12, "{sum} {}", w.explained);
            let mut held = 0.0;
            let mut sides = 0;
            for t in 0..tick {
                let e = explain(&store, &town, &good, t).unwrap();
                if e.one_side {
                    held += e.kx;
                    sides += 1;
                }
            }
            assert_eq!(w.one_sided, sides);
            assert!(
                (w.residual + held).abs() < 1e-9,
                "{good} at {cursor:?}: residual {} against the holds' {}",
                w.residual,
                -held
            );
            assert_eq!(w.missing, 0);
            if let Some(last) = w.bins.last() {
                assert_eq!(last.to, tick);
                assert_eq!(last.level.to_bits(), w.level.to_bits());
            }
            let mut from = 0;
            for b in &w.bins {
                assert_eq!(b.from, from);
                from = b.to;
            }
            if tick >= 104 {
                assert!(w.bins.len() <= 42, "{} bins", w.bins.len());
            } else {
                assert_eq!(w.bins.len() as u64, tick);
            }
        }
    }
    // The events fired in the span are marked; none moves a gate price.
    let w = waterfall(&store, &town, &Key::new("bread").unwrap(), None).unwrap();
    assert!(w.events.iter().any(|e| e.key.as_str() == "mine.cut"));
    assert!(w.events.iter().all(|e| !e.moves));
}

/// The gate with the rate of bread doubled on 1755-01-01, from 5.2 to 10.4 a year, by the event
/// `rate.up`.
fn rate_change() -> String {
    let t = edit(
        GATE,
        "        (key: \"rate.bread\", value: 5.2,",
        "        (key: \"rate.bread.fast\", value: 10.4, unit: RatePerYear, basis: Assumed(\"test\")),\n        (key: \"rate.bread\", value: 5.2,",
    );
    edit(
        &t,
        "    events: [\n",
        "    events: [\n        (key: \"rate.up\", at: \"1755-01-01\", basis: Assumed(\"test\"), act: SetParam(param: \"rate.bread\", to: \"rate.bread.fast\")),\n",
    )
}

/// The gate with the price of bread at the town scaled by 1.5 on 1756-01-01, by the event
/// `bread.shock`.
fn shock() -> String {
    let t = edit(
        GATE,
        "        (key: \"rate.bread\", value: 5.2,",
        "        (key: \"shock.bread\", value: 1.5, unit: Dimensionless, basis: Assumed(\"test\")),\n        (key: \"rate.bread\", value: 5.2,",
    );
    edit(
        &t,
        "    events: [\n",
        "    events: [\n        (key: \"bread.shock\", at: \"1756-01-01\", basis: Assumed(\"test\"), act: ScalePrice(node: \"town\", good: \"bread\", by: \"shock.bread\")),\n",
    )
}

/// The gate under another price rule or one-sided rule.
fn market_rule(rule: &str) -> String {
    edit(GATE, "rule: Imbalance, one_sided: Hold", rule)
}

#[test]
fn the_explainer_equals_next_price_under_a_rate_change_a_shock_ratio_and_saturate() {
    // G1's verification (O37): the gate and appb have a constant rate, no event that moves a
    // price, and one rule, so an explainer that read the rate at tick 0 passed. Four variants of
    // the gate, each run 2,080 ticks: the explainer's next price is the engine's at every
    // (tick, market) pair.
    for (name, text) in [
        ("rate change", rate_change()),
        ("shock", shock()),
        ("Ratio", market_rule("rule: Ratio, one_sided: Hold")),
        (
            "Saturate",
            market_rule("rule: Imbalance, one_sided: Saturate"),
        ),
    ] {
        every_tick(name, &text, GATE_TICKS);
    }
}

/// The waterfall of town/`good` at the end of a store, its bins checked: they tile the span,
/// each bin's residual is its change of ln(p/p₀) less its steps, and the bins' steps and
/// residuals add up to the whole's.
fn checked_waterfall(store: &Store, good: &str) -> WaterfallVm {
    let town = Key::new("town").unwrap();
    let w = waterfall(store, &town, &Key::new(good).unwrap(), None).unwrap();
    let mut before = 0.0;
    let mut from = w.start;
    for b in &w.bins {
        assert_eq!(b.from, from);
        assert_eq!(
            b.residual.to_bits(),
            ((b.level - before) - b.kx).to_bits(),
            "{good} {}",
            b.label
        );
        before = b.level;
        from = b.to;
    }
    let kx: f64 = w.bins.iter().map(|b| b.kx).sum();
    let residual: f64 = w.bins.iter().map(|b| b.residual).sum();
    assert!(
        (kx - w.explained).abs() < 1e-12,
        "{good}: {kx} {}",
        w.explained
    );
    assert!(
        (residual - w.residual).abs() < 1e-12,
        "{good}: the bins' residuals {residual} against {}",
        w.residual
    );
    w
}

/// The one-sided ticks' steps, which `Hold` did not take, over town/`good`'s record.
fn held(store: &Store, good: &str, until: u64) -> f64 {
    let town = Key::new("town").unwrap();
    let good = Key::new(good).unwrap();
    (0..until)
        .map(|t| explain(store, &town, &good, t).unwrap())
        .filter(|e| e.one_side)
        .map(|e| e.kx)
        .sum()
}

#[test]
fn the_waterfall_flags_what_moves_a_price_and_its_bins_add_up() {
    // G1's verification (O37): the waterfall's steps read the rate at each tick, an event that
    // sets this price's rate or scales this price is flagged, and the bins' residuals add up.
    // Under a rate change the steps take the new rate, so bread's residual is still the holds;
    // under a shock of 1.5 it is ln 1.5 more.
    let last = GATE_TICKS - 1;
    let (store, _) = record(&tape_of(&rate_change()), GATE_TICKS);
    let w = checked_waterfall(&store, "bread");
    let moving: Vec<&str> = w
        .events
        .iter()
        .filter(|e| e.moves)
        .map(|e| e.key.as_str())
        .collect();
    assert_eq!(moving, ["rate.up"]);
    let residual = w.residual + held(&store, "bread", last);
    assert!(
        residual.abs() < 1e-9,
        "bread under a rate change: {residual}"
    );
    assert!(checked_waterfall(&store, "fuel")
        .events
        .iter()
        .all(|e| !e.moves));
    let (store, _) = record(&tape_of(&shock()), GATE_TICKS);
    let w = checked_waterfall(&store, "bread");
    let moving: Vec<&str> = w
        .events
        .iter()
        .filter(|e| e.moves)
        .map(|e| e.key.as_str())
        .collect();
    assert_eq!(moving, ["bread.shock"]);
    let residual = w.residual + held(&store, "bread", last) - num::ln(1.5);
    assert!(residual.abs() < 1e-9, "bread under a shock: {residual}");
    assert!(checked_waterfall(&store, "grain")
        .events
        .iter()
        .all(|e| !e.moves));
    assert_eq!(w.term, "k·x");
}

#[test]
fn under_ratio_the_waterfall_sums_the_rules_own_steps() {
    // G1's verification: `Ratio` ignores k, so Σ k·x explained nothing of a price under it (for
    // town/bread, 0.196 of 2.907). Under `Ratio` the waterfall sums the rule's own step, ln(D/S)
    // where both sides posted; a one-sided tick holds by the rule, so the residual is rounding,
    // and a rate set would move nothing.
    let (store, _) = record(
        &tape_of(&market_rule("rule: Ratio, one_sided: Hold")),
        GATE_TICKS,
    );
    for good in ["bread", "fuel", "grain"] {
        let w = checked_waterfall(&store, good);
        assert_eq!(w.term, "ln(D/S)");
        assert!(w.level.abs() > 1.0, "{good}: the price moved, {}", w.level);
        assert!(
            w.residual.abs() < 1e-9,
            "{good}: ln(p/p₀) {} against Σ ln(D/S) {}",
            w.level,
            w.explained
        );
        if good == "fuel" {
            assert!(w.one_sided > 0, "fuel has one-sided ticks");
        }
    }
}

/// The gate run to `until`, with one bit of town/bread's recorded next price flipped at `tick`
/// before the store ingests it.
fn record_flipped(until: u64, tick: u64) -> Store {
    let t = tape_of(GATE);
    let mut d = driver();
    let mut log = Vec::new();
    d.send(Cmd::Load {
        tape: Box::new(t),
        from: None,
    });
    wait_for(&mut d, &mut log, |o| matches!(o, Obs::Loaded { .. }));
    d.send(Cmd::Run {
        until: Some(until),
        max_tps: None,
    });
    wait_paused(&mut d, &mut log);
    drop(d);
    let next = SeriesKey {
        measure: Measure::NextPrice,
        at: At::Market {
            node: Key::new("town").unwrap(),
            good: Key::new("bread").unwrap(),
        },
    };
    let mut catalogue: Vec<SeriesKey> = Vec::new();
    let mut flipped = 0;
    let mut store = Store::default();
    for mut o in log {
        if let Obs::Batch(b) = &mut o {
            assert_eq!(b.known as usize, catalogue.len());
            catalogue.extend(b.new_series.iter().cloned());
            let i = catalogue.iter().position(|k| *k == next);
            for row in b.rows.iter_mut().filter(|r| r.tick == tick) {
                for cell in row.cells.iter_mut().filter(|c| Some(c.0 as usize) == i) {
                    cell.1 = f64::from_bits(cell.1.to_bits() ^ 1);
                    flipped += 1;
                }
            }
        }
        store.ingest(o).expect("every observation ingests");
    }
    assert_eq!(flipped, 1, "one next price is flipped");
    store
}

/// The rows the explainer's grid paints, key and value, in order: each key's text is followed
/// by its value's.
fn painted_rows(texts: &[common::paint::Text]) -> Vec<(String, String)> {
    let keys = [
        "p, posted",
        "S",
        "D",
        "x = imbalance(S, D)",
        "k·x",
        "the run's next price",
    ];
    let mut rows = Vec::new();
    for (i, t) in texts.iter().enumerate() {
        let k = keys.contains(&t.text.as_str()) || t.text.starts_with("k = ");
        if k {
            if let Some(v) = texts.get(i + 1) {
                rows.push((t.text.clone(), v.text.clone()));
            }
        }
    }
    rows
}

/// The number a painted value starts with, read back as a double.
fn number(text: &str) -> f64 {
    let end = text.find([' ', ':']).unwrap_or(text.len());
    text[..end]
        .parse()
        .unwrap_or_else(|_| panic!("{text:?} starts with a number"))
}

/// The explainer's grid, alone, as the inspector paints it.
fn paint_explainer(e: &ExplainerVm) -> Vec<common::paint::Text> {
    let shown = e.clone();
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(move |ui| rustyecon_gui::ui::inspector::explainer(ui, &shown));
    h.run();
    common::paint::texts(&h.output().shapes)
}

#[test]
fn a_next_price_that_differs_is_said_to() {
    // G1's verification: the explainer's "DIFFERS" was never shown. A store whose recorded next
    // price of town/bread has one bit flipped at one tick: the explainer there still gives the
    // engine's next price, says the record differs, and the panel paints it so; the ticks
    // either side are equal. And the panel paints every input as the view has it, bit for bit.
    let tick = 400;
    let store = record_flipped(500, tick);
    let (town, bread) = (Key::new("town").unwrap(), Key::new("bread").unwrap());
    let e = explain(&store, &town, &bread, tick).unwrap();
    let recorded = e.recorded.expect("recorded");
    assert_eq!(recorded.to_bits(), e.next.to_bits() ^ 1);
    assert_eq!(e.equal, Some(false));
    for t in [tick - 1, tick + 1] {
        assert_eq!(explain(&store, &town, &bread, t).unwrap().equal, Some(true));
    }
    let texts = paint_explainer(&e);
    let rows = painted_rows(&texts);
    let said = |k: &str| {
        rows.iter()
            .find(|(key, _)| key == k || (k == "k" && key.starts_with("k = ")))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("no row {k}: {rows:?}"))
    };
    assert_eq!(
        said("the run's next price"),
        format!("{recorded:?}: DIFFERS from the recomputed step")
    );
    // Each input, read back from the screen, is the view's double: p, S, D, x, k and k·x.
    assert!(e.supply != e.demand, "S and D differ at this tick");
    for (k, v) in [
        ("p, posted", e.price),
        ("S", e.supply),
        ("D", e.demand),
        ("x = imbalance(S, D)", e.imbalance),
        ("k", e.k),
        ("k·x", e.kx),
    ] {
        assert_eq!(number(&said(k)).to_bits(), v.to_bits(), "{k}: {}", said(k));
    }
    // The next tick's equal record is painted equal.
    let e = explain(&store, &town, &bread, tick + 1).unwrap();
    let rows = painted_rows(&paint_explainer(&e));
    let r = rows
        .iter()
        .find(|(k, _)| k == "the run's next price")
        .expect("the row");
    assert_eq!(r.1, format!("{:?}: equal bit for bit", e.next));
}
