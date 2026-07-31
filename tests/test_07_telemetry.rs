//! Parquet telemetry tests (v2 Phase 2).
//!
//! The gate Phase 2 is judged against is that the analysis layer runs off
//! Parquet with no RON in its path. That makes two things testable here: the
//! telemetry has to actually *contain* what the analysis layer needs, and it has
//! to contain it losslessly. The second is the reason the format changed at all
//! — the CSV it replaces wrote `{:.6}`, which turns a collapsed market's
//! quantities into zeros.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::{Path, PathBuf};

use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;

use rustyecon::output::manifest::{Manifest, SCHEMA_VERSION};
use rustyecon::output::parquet::TelemetryWriter;
use rustyecon::output::telemetry::{MANIFEST_FILE, TELEMETRY_FILE};
use rustyecon::runner::{RunConfig, SimRunner};
use rustyecon::scenario::loader;

/// One decoded telemetry row. `region` and `good` are `None` when null.
#[derive(Debug, Clone, PartialEq)]
struct Row {
    tick: i64,
    region: Option<i32>,
    entity_kind: String,
    id: i32,
    good: Option<i32>,
    metric: String,
    value: f64,
}

fn read_rows(path: &Path) -> Vec<Row> {
    let reader = SerializedFileReader::new(File::open(path).expect("telemetry exists"))
        .expect("telemetry parses as parquet");
    let mut out = Vec::new();
    for row in reader.get_row_iter(None).expect("row iterator") {
        let row = row.expect("row decodes");
        let mut tick = 0i64;
        let mut region = None;
        let mut entity_kind = String::new();
        let mut id = 0i32;
        let mut good = None;
        let mut metric = String::new();
        let mut value = 0.0;
        for (name, field) in row.get_column_iter() {
            match (name.as_str(), field) {
                ("tick", Field::Long(v)) => tick = *v,
                ("region", Field::Int(v)) => region = Some(*v),
                ("region", Field::Null) => region = None,
                ("entity_kind", Field::Str(v)) => entity_kind = v.clone(),
                ("id", Field::Int(v)) => id = *v,
                ("good", Field::Int(v)) => good = Some(*v),
                ("good", Field::Null) => good = None,
                ("metric", Field::Str(v)) => metric = v.clone(),
                ("value", Field::Double(v)) => value = *v,
                (n, f) => panic!("unexpected column {n} = {f:?}"),
            }
        }
        out.push(Row {
            tick,
            region,
            entity_kind,
            id,
            good,
            metric,
            value,
        });
    }
    out
}

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/scenarios")
        .join(name)
}

/// Run a scenario with telemetry on, into a temp dir the caller keeps alive.
fn run_with_telemetry(name: &str, ticks: u64, dir: &Path) {
    let sdir = scenario_dir(name);
    let s = loader::load(&sdir).expect("scenario loads");
    let config = RunConfig {
        agents: rustyecon::kernel::AgentArm::Legacy,
        price_rule: rustyecon::systems::clearing::PriceRule::Imbalance,
        ticks,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: dir.to_path_buf(),
        record: true,
        telemetry_every: 1,
        certify: true,
        results_dir: dir.join("results"),
    };
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&sdir)
        .with_criteria(s.criteria);
    runner.run();
    assert_eq!(
        runner.telemetry_error(),
        None,
        "telemetry must not fail silently"
    );
}

#[test]
fn values_survive_the_round_trip_exactly() {
    // The whole reason for leaving CSV. Every one of these is a value the old
    // `{:.6}` writer destroyed: the tiny quantity became 0.0 (a market that
    // *did* trade reported as one that did not), and the large price lost its
    // low bits. Bit-equality is the assertion, not approximate equality.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.parquet");
    let hostile = [
        5.7e-24_f64,
        2.24e18,
        f64::MIN_POSITIVE,
        1.0 / 3.0,
        -0.0,
        1e-300,
        f64::MAX,
    ];

    let mut w = TelemetryWriter::create(&path, 4).unwrap();
    for (i, v) in hostile.iter().enumerate() {
        w.push(i as u64, Some(0), "market", 0, Some(0), "price", *v).unwrap();
    }
    let rows = w.close().unwrap();
    assert_eq!(rows, hostile.len() as u64);

    let back = read_rows(&path);
    assert_eq!(back.len(), hostile.len());
    for (row, expected) in back.iter().zip(hostile.iter()) {
        assert_eq!(
            row.value.to_bits(),
            expected.to_bits(),
            "telemetry must round-trip {expected:e} bit-for-bit, got {}",
            row.value
        );
    }
}

#[test]
fn nan_survives_rather_than_becoming_a_number() {
    // NaN means "could not be computed", and under the fail-closed policy that
    // is a FAIL. If the telemetry silently turned it into 0.0 or dropped the
    // row, an uncomputable metric would read as a computed one.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.parquet");
    let mut w = TelemetryWriter::create(&path, 8).unwrap();
    w.push(0, Some(0), "region", 0, None, "real_wage", f64::NAN).unwrap();
    w.push(1, Some(0), "region", 0, None, "real_wage", f64::INFINITY).unwrap();
    w.close().unwrap();

    let back = read_rows(&path);
    assert!(back[0].value.is_nan(), "NaN must not be coerced");
    assert!(back[1].value.is_infinite() && back[1].value > 0.0);
}

#[test]
fn nulls_are_null_not_sentinels() {
    // A market price belongs to a node, not a region; a region's velocity is not
    // per-good. Both absences must read back as null, so that no analysis can
    // mistake them for region 0 or good 0.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.parquet");
    let mut w = TelemetryWriter::create(&path, 2).unwrap();
    w.push(0, None, "market", 7, Some(3), "price", 1.0).unwrap();
    w.push(0, Some(2), "region", 2, None, "velocity", 2.0).unwrap();
    w.push(0, None, "market", 7, None, "odd", 3.0).unwrap();
    w.close().unwrap();

    let back = read_rows(&path);
    assert_eq!((back[0].region, back[0].good), (None, Some(3)));
    assert_eq!((back[1].region, back[1].good), (Some(2), None));
    assert_eq!((back[2].region, back[2].good), (None, None));
    // Row-group boundaries fall mid-batch here (2 rows per group, 3 rows), so
    // this also proves definition levels are flushed per group, not per file.
}

#[test]
fn telemetry_carries_what_the_analysis_layer_needs() {
    // The Phase 2 gate is that analysis runs off Parquet alone. That fails if
    // any column the reader reconstructs is missing, so the presence of each is
    // asserted here rather than discovered later as an empty DataFrame.
    let dir = tempfile::tempdir().unwrap();
    run_with_telemetry("lr_00", 12, dir.path());
    let rows = read_rows(&dir.path().join(TELEMETRY_FILE));
    assert!(!rows.is_empty());

    let present: BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| (r.entity_kind.clone(), r.metric.clone()))
        .collect();
    let required = [
        ("market", "price"),
        ("market", "supply"),
        ("market", "demand"),
        ("building", "recipe_id"),
        ("building", "recipe_size"),
        ("building", "chosen_size"),
        ("building", "efficiency"),
        ("building", "balance"),
        ("building", "last_margin"),
        ("building", "last_throughput"),
        ("building", "channel_id"),
        ("building", "inventory"),
        ("pop", "size"),
        ("pop", "wealth"),
        ("pop", "savings_target"),
        ("pop", "is_employed"),
        ("region", "velocity"),
        ("region", "employment"),
    ];
    for (kind, metric) in required {
        assert!(
            present.contains(&(kind.to_string(), metric.to_string())),
            "telemetry is missing {kind}/{metric}; the analysis layer reconstructs it"
        );
    }

    // Tick 0 must be present: it is the baseline every price index is relative
    // to, and it is written before the loop rather than inside it.
    assert!(rows.iter().any(|r| r.tick == 0 && r.metric == "price"));
    assert_eq!(rows.iter().map(|r| r.tick).max(), Some(12));

    // Market rows carry no region (a node may serve several); every building and
    // pop row does, because the analysis layer groups by it.
    for r in &rows {
        match r.entity_kind.as_str() {
            "market" => assert_eq!(r.region, None, "market rows are node-scoped"),
            "building" | "pop" | "region" => {
                assert!(r.region.is_some(), "{} rows must carry a region", r.entity_kind)
            }
            other => panic!("unexpected entity_kind {other}"),
        }
    }
}

#[test]
fn manifest_resolves_ids_without_reading_any_ron() {
    // If the manifest did not carry the dimension tables, the analysis layer
    // would still need game_data.ron to name a good — and the gate ("no RON
    // parsing in the analysis path") would be met only on a technicality.
    let dir = tempfile::tempdir().unwrap();
    run_with_telemetry("lr_00", 5, dir.path());

    let text = std::fs::read_to_string(dir.path().join(MANIFEST_FILE)).unwrap();
    let m: Manifest = serde_json::from_str(&text).expect("manifest parses");
    assert_eq!(m.schema_version, SCHEMA_VERSION);
    assert_eq!(m.run.scenario, "lr_00");
    assert!(m.run.criteria_sha.is_some(), "lr_00 registers criteria");
    assert_eq!(m.telemetry.first_tick, 0);
    assert_eq!(m.telemetry.last_tick, 5);
    assert_eq!(m.telemetry.every, 1);

    // Every id the telemetry actually used must be resolvable from the manifest.
    let rows = read_rows(&dir.path().join(TELEMETRY_FILE));
    let goods: BTreeMap<u32, &str> = m.goods.iter().map(|g| (g.id, g.name.as_str())).collect();
    let regions: BTreeSet<u32> = m.regions.iter().map(|r| r.id).collect();
    let nodes: BTreeSet<u32> = m.market_nodes.iter().map(|n| n.id).collect();
    for r in &rows {
        if let Some(g) = r.good {
            assert!(goods.contains_key(&(g as u32)), "good {g} not in manifest");
        }
        if let Some(reg) = r.region {
            assert!(regions.contains(&(reg as u32)), "region {reg} not in manifest");
        }
        if r.entity_kind == "market" {
            assert!(nodes.contains(&(r.id as u32)), "node {} not in manifest", r.id);
        }
    }
    assert_eq!(goods.get(&0).copied(), Some("wheat"));
    // The node→region join the market rows depend on must be expressible.
    assert!(m.market_nodes.iter().any(|n| n.region.is_some()));
    assert!(m.telemetry.rows > 0 && m.telemetry.rows == rows.len() as u64);
}

#[test]
fn subsampling_keeps_tick_zero_and_says_so() {
    // A subsampled run must be distinguishable from a truncated one, and must
    // not silently rebase the price index onto whatever tick landed first.
    let dir = tempfile::tempdir().unwrap();
    let sdir = scenario_dir("lr_00");
    let s = loader::load(&sdir).expect("scenario loads");
    let config = RunConfig {
        agents: rustyecon::kernel::AgentArm::Legacy,
        price_rule: rustyecon::systems::clearing::PriceRule::Imbalance,
        ticks: 20,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: dir.path().to_path_buf(),
        record: true,
        telemetry_every: 5,
        certify: false,
        results_dir: dir.path().join("results"),
    };
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&sdir)
        .with_criteria(None);
    runner.run();
    assert_eq!(runner.telemetry_error(), None);

    let rows = read_rows(&dir.path().join(TELEMETRY_FILE));
    let ticks: BTreeSet<i64> = rows.iter().map(|r| r.tick).collect();
    assert_eq!(
        ticks.iter().copied().collect::<Vec<_>>(),
        vec![0, 5, 10, 15, 20]
    );

    let text = std::fs::read_to_string(dir.path().join(MANIFEST_FILE)).unwrap();
    let m: Manifest = serde_json::from_str(&text).unwrap();
    assert_eq!(m.telemetry.every, 5, "the manifest must disclose the sampling");
    assert_eq!(m.run.ticks, 20, "and the run length it was sampled from");
}
