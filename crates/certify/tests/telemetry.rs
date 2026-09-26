//! Telemetry (docs/CERTIFY.md §12; PLAN §3.8, C7): the gate world's reports as tidy long
//! Parquet, read back through parquet's own reader and compared with an expansion of the same
//! reports written here, independently of the writer: every row, in order, with each value bit
//! for bit, NaN, infinities, −0 and a subnormal included; the encodings §12 names; the footer
//! that names the run; and bytes that do not depend on the process.
//!
//! The gate run to tick 2,080 gives more than one row group (2¹⁸ rows each), so a group boundary
//! is read back too.

#![cfg(feature = "parquet")]

mod common;

use certify::telemetry::{TelemetryError, TelemetryWriter, COLUMNS, ROW_GROUP};
use certify::{tape_hash, Build, Hex, RunKey};
use common::*;
use parquet::basic::{Compression, Encoding, Type as Physical};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use rustyecon_core::fnv1a_64;
use rustyecon_engine::prelude::{NodeId, Provenance, SideTag, Tape, TickReport, World};
use std::io::Write;

/// The gate's ticks: 40 years.
const TICKS: u64 = 2080;

/// One row as the test expects it: tick, kind, the six keys, metric, and the value's bits.
type Row = (i64, String, [Option<String>; 6], String, u64);

/// An edit that puts a number into a report.
type Edit = fn(&mut TickReport, f64);

fn gate_reports(until: u64) -> (Tape, World, Vec<TickReport>) {
    let t = tape(GATE);
    let mut s = sim(&t);
    let mut out = Vec::new();
    s.run_until(until, &mut |r| out.push(r.clone())).unwrap();
    (t, s.world().clone(), out)
}

fn key_of(t: &Tape, w: &World) -> RunKey {
    RunKey {
        build: build(),
        tape_hash: Hex(tape_hash(t)),
        world_id: Hex(w.world_id),
    }
}

/// The reports written to bytes: the file, its rows and digest as the writer reported them.
fn write(w: &World, key: &RunKey, reports: &[TickReport]) -> (Vec<u8>, u64, u64) {
    let mut tw = TelemetryWriter::new(Vec::new(), w, key).expect("a writer");
    for r in reports {
        tw.push(r).expect("the report is written");
    }
    let fin = tw.finish().expect("the file is finished");
    assert_eq!(fin.bytes, fin.sink.len() as u64);
    (fin.sink, fin.rows, fin.digest)
}

fn reader(bytes: &[u8]) -> (tempfile::TempDir, SerializedFileReader<std::fs::File>) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.parquet");
    std::fs::File::create(&p)
        .and_then(|mut f| f.write_all(bytes))
        .unwrap();
    let r = SerializedFileReader::new(std::fs::File::open(&p).unwrap()).expect("it reads");
    (dir, r)
}

/// Every row of the file, through parquet's row reader.
fn read_rows(bytes: &[u8]) -> Vec<Row> {
    let (_dir, r) = reader(bytes);
    let mut out = Vec::new();
    for row in r.get_row_iter(None).unwrap() {
        let row = row.unwrap();
        let cols: Vec<(&String, &Field)> = row.get_column_iter().collect();
        assert_eq!(
            cols.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            COLUMNS
        );
        let text = |f: &Field| match f {
            Field::Str(s) => Some(s.clone()),
            Field::Null => None,
            other => panic!("a string column holds {other:?}"),
        };
        let tick = match cols[0].1 {
            Field::Long(v) => *v,
            other => panic!("tick is {other:?}"),
        };
        let value = match cols[9].1 {
            Field::Double(v) => v.to_bits(),
            other => panic!("value is {other:?}"),
        };
        let keys = [2, 3, 4, 5, 6, 7].map(|i| text(cols[i].1));
        out.push((
            tick,
            text(cols[1].1).expect("kind"),
            keys,
            text(cols[8].1).expect("metric"),
            value,
        ));
    }
    out
}

/// The rows §12 says a report gives, expanded here from the report and the world's keys.
fn expected(w: &World, reports: &[TickReport]) -> Vec<Row> {
    let key = |k: Option<&rustyecon_engine::prelude::Key>| k.map(|k| k.to_string());
    let side = |s: SideTag| {
        Some(
            match s {
                SideTag::Buy => "buy",
                SideTag::Sell => "sell",
            }
            .to_string(),
        )
    };
    let prov = |p: Provenance| {
        Some(
            match p {
                Provenance::Production => "production",
                Provenance::Consumption => "consumption",
                Provenance::Spoilage => "spoilage",
                Provenance::Endowment => "endowment",
                Provenance::Depreciation => "depreciation",
                Provenance::Construction => "construction",
                Provenance::Event => "event",
                Provenance::Rounding => "rounding",
            }
            .to_string(),
        )
    };
    let mut out = Vec::new();
    for r in reports {
        let t = r.tick as i64;
        let mut row = |kind: &str, keys: [Option<String>; 6], metric: &str, v: f64| {
            out.push((t, kind.to_string(), keys, metric.to_string(), v.to_bits()));
        };
        for l in &r.markets {
            let (n, g) = (key(w.key_of(l.node)), key(w.key_of(l.good)));
            for (m, v) in [
                ("price", l.price),
                ("next_price", l.next_price),
                ("ema", l.ema),
                ("supply", l.supply),
                ("demand", l.demand),
                ("cleared", l.cleared),
                ("buyer_fill", l.buyer_fill),
                ("seller_fill", l.seller_fill),
            ] {
                row(
                    "market",
                    [n.clone(), g.clone(), None, None, None, None],
                    m,
                    v,
                );
            }
        }
        for l in &r.rationing {
            let at = [
                key(w.key_of(l.node)),
                key(w.key_of(l.good)),
                key(w.key_of(l.class)),
                None,
                side(l.side),
                None,
            ];
            for (m, v) in [
                ("requested", l.requested),
                ("feasible", l.feasible),
                ("filled", l.filled),
            ] {
                row("ration", at.clone(), m, v);
            }
        }
        for l in &r.settlements {
            let at = [
                key(w.key_of(l.node)),
                key(w.key_of(l.good)),
                key(w.key_of(l.class)),
                key(w.key_of(l.actor)),
                side(l.side),
                None,
            ];
            row("settle", at.clone(), "qty", l.qty);
            row("settle", at, "value", l.value);
        }
        for (g, p, q) in &r.audit.lines {
            row(
                "ledger",
                [None, key(w.key_of(*g)), None, None, None, prov(*p)],
                "declared",
                *q,
            );
        }
        row(
            "ledger",
            Default::default(),
            "max_margin",
            r.audit.max_margin,
        );
        row("ledger", Default::default(), "max_drift", r.audit.max_drift);
        for (g, d) in &r.run.drift {
            row(
                "run",
                [None, key(w.key_of(*g)), None, None, None, None],
                "drift",
                *d,
            );
        }
        row("run", Default::default(), "max_margin", r.run.max_margin);
        for e in &r.events {
            row(
                "event",
                [
                    None,
                    None,
                    None,
                    Some(e.key.to_string()),
                    None,
                    e.source.as_ref().map(|s| s.to_string()),
                ],
                "fired",
                f64::from(e.occurrence),
            );
        }
    }
    out
}

#[test]
fn telemetry_round_trips_bit_for_bit() {
    // §12: the value column holds each f64 as the report held it, bit for bit: a NaN with a
    // payload stays that NaN, −0 stays −0, and ±inf and a subnormal come back. Every row of the
    // gate's 2,080 reports comes back in the order §12 gives, named by keys, across more than one
    // row group; and the file uses the codecs and encodings §12 names.
    let (t, w, mut reports) = gate_reports(TICKS);
    let payload_nan = f64::from_bits(0x7ff8_0000_dead_beef);
    let edits: [(usize, Edit, f64); 5] = [
        (10, |r, v| r.markets[0].price = v, payload_nan),
        (11, |r, v| r.markets[1].supply = v, f64::INFINITY),
        (12, |r, v| r.markets[2].buyer_fill = v, f64::NEG_INFINITY),
        (13, |r, v| r.audit.max_drift = v, -0.0),
        (14, |r, v| r.run.max_margin = v, f64::MIN_POSITIVE / 4.0),
    ];
    for (i, edit, v) in edits {
        edit(&mut reports[i], v);
    }
    assert!(!reports[12].rationing.is_empty() && !reports[12].settlements.is_empty());
    let fired: Vec<_> = reports.iter().flat_map(|r| &r.events).collect();
    assert!(fired.iter().any(|e| e.source.is_some()) && fired.iter().any(|e| e.source.is_none()));
    let (bytes, rows, _) = write(&w, &key_of(&t, &w), &reports);
    let want = expected(&w, &reports);
    assert_eq!(rows, want.len() as u64);
    assert!(want.len() > ROW_GROUP, "{} rows fit one group", want.len());
    let got = read_rows(&bytes);
    assert_eq!(got.len(), want.len());
    for (k, (g, e)) in got.iter().zip(&want).enumerate() {
        assert_eq!(g, e, "row {k}");
    }
    for bits in [
        payload_nan.to_bits(),
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
        (-0.0f64).to_bits(),
        (f64::MIN_POSITIVE / 4.0).to_bits(),
    ] {
        assert!(got.iter().any(|r| r.4 == bits), "{bits:#x}");
    }

    let (_dir, r) = reader(&bytes);
    let meta = r.metadata();
    assert!(meta.num_row_groups() >= 2);
    let schema = meta.file_metadata().schema_descr();
    for (i, name) in COLUMNS.iter().enumerate() {
        let c = schema.column(i);
        assert_eq!(c.name(), *name);
        let physical = match *name {
            "tick" => Physical::INT64,
            "value" => Physical::DOUBLE,
            _ => Physical::BYTE_ARRAY,
        };
        assert_eq!(c.physical_type(), physical, "{name}");
        let nullable = !matches!(*name, "tick" | "kind" | "metric" | "value");
        assert_eq!(c.self_type().is_optional(), nullable, "{name}");
    }
    for g in meta.row_groups() {
        assert!(g.num_rows() as usize <= ROW_GROUP);
        for (i, c) in g.columns().iter().enumerate() {
            assert_eq!(c.compression(), Compression::LZ4_RAW, "{}", COLUMNS[i]);
            let enc = c.encodings().collect::<Vec<_>>();
            match COLUMNS[i] {
                "value" => assert!(enc.contains(&Encoding::BYTE_STREAM_SPLIT), "{enc:?}"),
                "tick" => assert!(enc.contains(&Encoding::DELTA_BINARY_PACKED), "{enc:?}"),
                _ => assert!(
                    enc.contains(&Encoding::RLE_DICTIONARY)
                        || enc.contains(&Encoding::PLAIN_DICTIONARY),
                    "{}: {enc:?}",
                    COLUMNS[i]
                ),
            }
        }
    }
}

#[test]
fn telemetry_is_reproducible() {
    // §12: parquet hashes with a runtime-seeded ahash, so the bytes are shown not to depend on
    // it: the same reports written twice give the same bytes (gate.sh writes them from two
    // processes too). The digest and byte count the writer reports are the file's own.
    let (t, w, reports) = gate_reports(300);
    let key = key_of(&t, &w);
    let (a, rows_a, digest_a) = write(&w, &key, &reports);
    let (b, rows_b, digest_b) = write(&w, &key, &reports);
    assert_eq!(a, b);
    assert_eq!((rows_a, digest_a), (rows_b, digest_b));
    assert_eq!(digest_a, fnv1a_64(&a));
    // Another report changes the bytes and the digest.
    let (c, _, digest_c) = write(&w, &key, &reports[..299]);
    assert_ne!(a, c);
    assert_ne!(digest_a, digest_c);
}

#[test]
fn telemetry_names_its_run() {
    // §12 and R16: the footer names the telemetry format, the tape, its tape_hash, the world
    // and the build, each field of the run key.
    let (t, w, reports) = gate_reports(5);
    let mut key = key_of(&t, &w);
    key.build = Build {
        commit: "89abcdef0123456789abcdef0123456789abcdef".to_string(),
        dirty: true,
        target: "x86_64-test".to_string(),
        rustc: "rustc 9.9.9".to_string(),
    };
    let (bytes, _, _) = write(&w, &key, &reports);
    let (_dir, r) = reader(&bytes);
    let kv: Vec<(String, Option<String>)> = r
        .metadata()
        .file_metadata()
        .key_value_metadata()
        .expect("a footer")
        .iter()
        .map(|k| (k.key.clone(), k.value.clone()))
        .collect();
    let want = [
        ("rustyecon.telemetry", "1".to_string()),
        ("rustyecon.tape", "gate".to_string()),
        ("rustyecon.tape_hash", format!("0x{:016x}", tape_hash(&t))),
        ("rustyecon.world_id", format!("0x{:016x}", w.world_id)),
        (
            "rustyecon.build.commit",
            "89abcdef0123456789abcdef0123456789abcdef".to_string(),
        ),
        ("rustyecon.build.dirty", "true".to_string()),
        ("rustyecon.build.target", "x86_64-test".to_string()),
        ("rustyecon.build.rustc", "rustc 9.9.9".to_string()),
    ];
    let want: Vec<(String, Option<String>)> = want
        .into_iter()
        .map(|(k, v)| (k.to_string(), Some(v)))
        .collect();
    assert_eq!(kv, want);
}

#[test]
fn telemetry_refuses_a_report_it_cannot_name() {
    // Keys replace ids (§12), so a report whose ids the world lacks (another world's) is refused
    // whole: none of its rows is kept, and the file holds the rows before it.
    let (t, w, mut reports) = gate_reports(3);
    let mut tw = TelemetryWriter::new(Vec::new(), &w, &key_of(&t, &w)).unwrap();
    tw.push(&reports[0]).unwrap();
    let before = tw.rows();
    reports[1].markets[3].node = NodeId(99);
    let err = tw.push(&reports[1]).expect_err("an unknown node");
    assert!(matches!(err, TelemetryError::Report(_)), "{err}");
    assert_eq!(tw.rows(), before);
    let fin = tw.finish().unwrap();
    assert_eq!(read_rows(&fin.sink), expected(&w, &reports[..1]));
}
