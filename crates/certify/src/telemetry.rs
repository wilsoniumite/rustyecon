//! Telemetry: a run as tidy long Parquet (docs/CERTIFY.md §12; PLAN §3.8; C7). The input is the
//! engine's `TickReport`, one per tick, and nothing else; every row is one number the report
//! holds, bit for bit, named by tape keys so that branches compare by key (E8) with no id table.
//!
//! Behind the feature `parquet`. This is the one module that names `parquet` or `std::io`, so the
//! manifest and the verdicts compile without it, for the GUI's web build (GUI.md §7.2 c).
//!
//! The codecs are pure Rust: LZ4_RAW through lz4_flex, no zstd, snappy, brotli or flate2, and no
//! arrow, so nothing builds C. The cost is size: GUI §3.4 measured one chunk at 46.0 MB as LZ4_RAW
//! with byte-stream-split, against 42.1 MB as zstd-3, about 9% more.
//!
//! **Columns** (the schema below): `tick` (INT64, the report's tick), `kind` (`market`,
//! `ration`, `settle`, `ledger`, `run`, `event`), the nullable keys `node`, `good`, `class`, `key`
//! (the actor of a settlement, the event of a firing), `side` (`buy`, `sell`) and `tag` (a ledger
//! line's provenance, a firing's source param), then `metric` and `value` (DOUBLE, never null,
//! NaN kept). A null is a key the row does not have, never a sentinel.
//!
//! **Rows of one report, in this order:**
//! 1. `market`, per `MarketLine` in the report's order: `price`, `next_price`, `ema`, `supply`,
//!    `demand`, `cleared`, `buyer_fill`, `seller_fill`, at its node and good;
//! 2. `ration`, per `RationLine`: `requested`, `feasible`, `filled`, at its node, good, class and
//!    side;
//! 3. `settle`, per `SettleLine`: `qty`, `value`, at its node, good, class, actor (`key`) and side;
//! 4. `ledger`: per tick-audit line, `declared` at its good with its provenance as `tag`; then
//!    `max_margin` and `max_drift`, with no good;
//! 5. `run`: per good, the run's `drift`; then the run's `max_margin`;
//! 6. `event`, per `FiredEvent`: `fired`, valued at the occurrence, at the event's key (`key`),
//!    with its source param as `tag`.
//!
//! **Encoding.** Strings are dictionary-encoded, `value` is BYTE_STREAM_SPLIT, `tick` is
//! DELTA_BINARY_PACKED, and every column is LZ4_RAW. A row group holds [`ROW_GROUP`] rows, the
//! last one fewer: I/O batching, not behaviour. The footer's key-value metadata names the run:
//! `rustyecon.telemetry` ([`TELEMETRY_FORMAT`]), `rustyecon.tape`, `rustyecon.tape_hash`,
//! `rustyecon.world_id`, and the build as `rustyecon.build.commit`, `.dirty`, `.target` and
//! `.rustc`.
//!
//! **Reproducible bytes.** parquet interns strings with a runtime-seeded hash, but a dictionary
//! is in first-seen order and the file holds no time, so the same reports give the same bytes in
//! every process (`telemetry_is_reproducible`, and `gate.sh`'s two-process `cmp`).

use crate::manifest::RunKey;
use parquet::basic::{Compression, Encoding};
use parquet::data_type::{ByteArray, ByteArrayType, DoubleType, Int64Type};
use parquet::errors::ParquetError;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use parquet::schema::types::ColumnPath;
use rustyecon_core::Fnv;
use rustyecon_engine::prelude::{
    ActorId, ClassId, GoodId, NodeId, Provenance, SideTag, TickReport, World,
};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::sync::Arc;

/// The telemetry format, in the footer's `rustyecon.telemetry`.
pub const TELEMETRY_FORMAT: u32 = 1;

/// Rows a row group holds before it is written: I/O batching, not behaviour.
pub const ROW_GROUP: usize = 1 << 18;

/// The schema, in column order; [`Columns::flush`] writes the columns in this order.
const SCHEMA: &str = "
message rustyecon_telemetry {
  REQUIRED INT64 tick;
  REQUIRED BYTE_ARRAY kind (STRING);
  OPTIONAL BYTE_ARRAY node (STRING);
  OPTIONAL BYTE_ARRAY good (STRING);
  OPTIONAL BYTE_ARRAY class (STRING);
  OPTIONAL BYTE_ARRAY key (STRING);
  OPTIONAL BYTE_ARRAY side (STRING);
  OPTIONAL BYTE_ARRAY tag (STRING);
  REQUIRED BYTE_ARRAY metric (STRING);
  REQUIRED DOUBLE value;
}
";

/// The column names, in schema order.
pub const COLUMNS: [&str; 10] = [
    "tick", "kind", "node", "good", "class", "key", "side", "tag", "metric", "value",
];

/// The `kind`s, in the order a report's rows come.
pub const KINDS: [&str; 6] = ["market", "ration", "settle", "ledger", "run", "event"];

/// A `market` row's metrics, in `MarketLine`'s field order.
pub const MARKET_METRICS: [&str; 8] = [
    "price",
    "next_price",
    "ema",
    "supply",
    "demand",
    "cleared",
    "buyer_fill",
    "seller_fill",
];

/// A ledger line's provenance as its `tag`.
pub fn provenance_tag(p: Provenance) -> &'static str {
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
}

/// A side as its `side`.
pub fn side_tag(s: SideTag) -> &'static str {
    match s {
        SideTag::Buy => "buy",
        SideTag::Sell => "sell",
    }
}

/// Why telemetry was not written.
#[derive(Debug)]
pub enum TelemetryError {
    /// The Parquet writer failed.
    Parquet(String),
    /// The sink failed.
    Io(std::io::Error),
    /// A report names an id the world lacks, or a tick past `i64`: a report of another world.
    Report(String),
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryError::Parquet(m) => write!(f, "the Parquet writer failed: {m}"),
            TelemetryError::Io(e) => write!(f, "the telemetry sink failed: {e}"),
            TelemetryError::Report(m) => write!(f, "a report the telemetry cannot name: {m}"),
        }
    }
}

impl std::error::Error for TelemetryError {}

impl From<ParquetError> for TelemetryError {
    fn from(e: ParquetError) -> TelemetryError {
        TelemetryError::Parquet(e.to_string())
    }
}

impl From<std::io::Error> for TelemetryError {
    fn from(e: std::io::Error) -> TelemetryError {
        TelemetryError::Io(e)
    }
}

/// The sink, with FNV-1a 64 over every byte that reached it, so the manifest pins the file by
/// digest without reading it back.
struct Digesting<W: Write> {
    inner: W,
    fnv: Fnv,
    bytes: u64,
}

impl<W: Write> Write for Digesting<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        let done = buf.get(..n).unwrap_or(buf);
        self.fnv.write(done);
        self.bytes += u64::try_from(done.len()).unwrap_or(u64::MAX);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// One row, before it joins its column.
struct Row {
    kind: ByteArray,
    keys: [Option<ByteArray>; 6],
    metric: ByteArray,
    value: f64,
}

/// A nullable string column: the present values, and a definition level per row.
#[derive(Default)]
struct Opt {
    values: Vec<ByteArray>,
    def: Vec<i16>,
}

/// One row group's rows, column by column.
#[derive(Default)]
struct Columns {
    tick: Vec<i64>,
    kind: Vec<ByteArray>,
    /// `node`, `good`, `class`, `key`, `side`, `tag`.
    keys: [Opt; 6],
    metric: Vec<ByteArray>,
    value: Vec<f64>,
}

impl Columns {
    fn len(&self) -> usize {
        self.tick.len()
    }

    fn push(&mut self, tick: i64, row: Row) {
        self.tick.push(tick);
        self.kind.push(row.kind);
        for (col, k) in self.keys.iter_mut().zip(row.keys) {
            match k {
                Some(v) => {
                    col.values.push(v);
                    col.def.push(1);
                }
                None => col.def.push(0),
            }
        }
        self.metric.push(row.metric);
        self.value.push(row.value);
    }

    /// Write the rows as one row group, in schema order, and empty the buffer.
    fn flush<W: Write + Send>(
        &mut self,
        w: &mut SerializedFileWriter<Digesting<W>>,
    ) -> Result<(), TelemetryError> {
        if self.len() == 0 {
            return Ok(());
        }
        let mut rg = w.next_row_group()?;
        let missing = || TelemetryError::Parquet("the schema has fewer columns".to_string());
        let mut c = rg.next_column()?.ok_or_else(missing)?;
        c.typed::<Int64Type>().write_batch(&self.tick, None, None)?;
        c.close()?;
        let mut c = rg.next_column()?.ok_or_else(missing)?;
        c.typed::<ByteArrayType>()
            .write_batch(&self.kind, None, None)?;
        c.close()?;
        for o in &self.keys {
            let mut c = rg.next_column()?.ok_or_else(missing)?;
            c.typed::<ByteArrayType>()
                .write_batch(&o.values, Some(&o.def), None)?;
            c.close()?;
        }
        let mut c = rg.next_column()?.ok_or_else(missing)?;
        c.typed::<ByteArrayType>()
            .write_batch(&self.metric, None, None)?;
        c.close()?;
        let mut c = rg.next_column()?.ok_or_else(missing)?;
        c.typed::<DoubleType>()
            .write_batch(&self.value, None, None)?;
        c.close()?;
        if rg.next_column()?.is_some() {
            return Err(TelemetryError::Parquet(
                "the schema has more columns than the writer writes".to_string(),
            ));
        }
        rg.close()?;
        *self = Columns::default();
        Ok(())
    }
}

fn ba(s: &str) -> ByteArray {
    ByteArray::from(s)
}

/// The strings a row repeats, made once.
struct Names {
    nodes: Vec<ByteArray>,
    goods: Vec<ByteArray>,
    classes: Vec<ByteArray>,
    actors: BTreeMap<ActorId, ByteArray>,
    kinds: [ByteArray; 6],
    market: [ByteArray; 8],
    ration: [ByteArray; 3],
    settle: [ByteArray; 2],
    declared: ByteArray,
    max_margin: ByteArray,
    max_drift: ByteArray,
    drift: ByteArray,
    fired: ByteArray,
}

const MARKET: usize = 0;
const RATION: usize = 1;
const SETTLE: usize = 2;
const LEDGER: usize = 3;
const RUN: usize = 4;
const EVENT: usize = 5;

fn unknown(what: impl fmt::Debug) -> TelemetryError {
    TelemetryError::Report(format!("{what:?}, which the world lacks"))
}

impl Names {
    fn of(w: &World) -> Names {
        Names {
            nodes: w.nodes.iter().map(|n| ba(n.key.as_str())).collect(),
            goods: w.goods.iter().map(|g| ba(g.key.as_str())).collect(),
            classes: w.classes.iter().map(|c| ba(c.as_str())).collect(),
            actors: w
                .actors
                .iter()
                .map(|a| (a.id, ba(a.key.as_str())))
                .collect(),
            kinds: KINDS.map(ba),
            market: MARKET_METRICS.map(ba),
            ration: ["requested", "feasible", "filled"].map(ba),
            settle: ["qty", "value"].map(ba),
            declared: ba("declared"),
            max_margin: ba("max_margin"),
            max_drift: ba("max_drift"),
            drift: ba("drift"),
            fired: ba("fired"),
        }
    }

    fn node(&self, n: NodeId) -> Result<Option<ByteArray>, TelemetryError> {
        self.nodes
            .get(n.idx())
            .cloned()
            .map(Some)
            .ok_or_else(|| unknown(n))
    }

    fn good(&self, g: GoodId) -> Result<Option<ByteArray>, TelemetryError> {
        self.goods
            .get(g.idx())
            .cloned()
            .map(Some)
            .ok_or_else(|| unknown(g))
    }

    fn class(&self, c: ClassId) -> Result<Option<ByteArray>, TelemetryError> {
        self.classes
            .get(c.idx())
            .cloned()
            .map(Some)
            .ok_or_else(|| unknown(c))
    }

    fn actor(&self, a: ActorId) -> Result<Option<ByteArray>, TelemetryError> {
        self.actors
            .get(&a)
            .cloned()
            .map(Some)
            .ok_or_else(|| unknown(a))
    }

    /// A report's rows, in the module's order. An id the world lacks refuses the whole report.
    fn rows(&self, r: &TickReport) -> Result<Vec<Row>, TelemetryError> {
        let mut out = Vec::new();
        let mut row = |kind: usize, keys: [Option<ByteArray>; 6], metric: &ByteArray, value| {
            out.push(Row {
                kind: self.kinds[kind].clone(),
                keys,
                metric: metric.clone(),
                value,
            });
        };
        for l in &r.markets {
            let (node, good) = (self.node(l.node)?, self.good(l.good)?);
            let values = [
                l.price,
                l.next_price,
                l.ema,
                l.supply,
                l.demand,
                l.cleared,
                l.buyer_fill,
                l.seller_fill,
            ];
            for (m, v) in self.market.iter().zip(values) {
                let keys = [node.clone(), good.clone(), None, None, None, None];
                row(MARKET, keys, m, v);
            }
        }
        for l in &r.rationing {
            let (node, good, class) =
                (self.node(l.node)?, self.good(l.good)?, self.class(l.class)?);
            let side = Some(ba(side_tag(l.side)));
            for (m, v) in self.ration.iter().zip([l.requested, l.feasible, l.filled]) {
                let keys = [
                    node.clone(),
                    good.clone(),
                    class.clone(),
                    None,
                    side.clone(),
                    None,
                ];
                row(RATION, keys, m, v);
            }
        }
        for l in &r.settlements {
            let (node, good, class, actor) = (
                self.node(l.node)?,
                self.good(l.good)?,
                self.class(l.class)?,
                self.actor(l.actor)?,
            );
            let side = Some(ba(side_tag(l.side)));
            for (m, v) in self.settle.iter().zip([l.qty, l.value]) {
                let keys = [
                    node.clone(),
                    good.clone(),
                    class.clone(),
                    actor.clone(),
                    side.clone(),
                    None,
                ];
                row(SETTLE, keys, m, v);
            }
        }
        for (g, p, q) in &r.audit.lines {
            let tag = Some(ba(provenance_tag(*p)));
            let keys = [None, self.good(*g)?, None, None, None, tag];
            row(LEDGER, keys, &self.declared, *q);
        }
        row(
            LEDGER,
            Default::default(),
            &self.max_margin,
            r.audit.max_margin,
        );
        row(
            LEDGER,
            Default::default(),
            &self.max_drift,
            r.audit.max_drift,
        );
        for (g, d) in &r.run.drift {
            let keys = [None, self.good(*g)?, None, None, None, None];
            row(RUN, keys, &self.drift, *d);
        }
        row(RUN, Default::default(), &self.max_margin, r.run.max_margin);
        for e in &r.events {
            let key = Some(ba(e.key.as_str()));
            let tag = e.source.as_ref().map(|s| ba(s.as_str()));
            let keys = [None, None, None, key, None, tag];
            row(EVENT, keys, &self.fired, f64::from(e.occurrence));
        }
        Ok(out)
    }
}

/// What [`TelemetryWriter::finish`] hands back: the sink, and what the manifest records.
#[derive(Debug)]
pub struct Finished<W> {
    /// The sink, flushed.
    pub sink: W,
    /// The rows written.
    pub rows: u64,
    /// The bytes written.
    pub bytes: u64,
    /// FNV-1a 64 over the bytes written: the manifest's `telemetry.digest`.
    pub digest: u64,
}

/// A run's telemetry, written to `sink` as the reports arrive (§12).
pub struct TelemetryWriter<W: Write + Send> {
    writer: SerializedFileWriter<Digesting<W>>,
    names: Names,
    rows: Columns,
    written: u64,
}

impl<W: Write + Send> TelemetryWriter<W> {
    /// A writer for the reports of a run of `world`, named by `run` in the footer.
    pub fn new(sink: W, world: &World, run: &RunKey) -> Result<Self, TelemetryError> {
        let schema = Arc::new(parse_message_type(SCHEMA)?);
        let b = &run.build;
        let footer = [
            ("rustyecon.telemetry", TELEMETRY_FORMAT.to_string()),
            ("rustyecon.tape", world.name.clone()),
            ("rustyecon.tape_hash", run.tape_hash.to_string()),
            ("rustyecon.world_id", run.world_id.to_string()),
            ("rustyecon.build.commit", b.commit.clone()),
            ("rustyecon.build.dirty", b.dirty.to_string()),
            ("rustyecon.build.target", b.target.clone()),
            ("rustyecon.build.rustc", b.rustc.clone()),
        ]
        .into_iter()
        .map(|(k, v)| KeyValue::new(k.to_string(), v))
        .collect();
        let col = |c: &str| ColumnPath::new(vec![c.to_string()]);
        let props = WriterProperties::builder()
            .set_compression(Compression::LZ4_RAW)
            .set_dictionary_enabled(true)
            .set_column_dictionary_enabled(col("value"), false)
            .set_column_encoding(col("value"), Encoding::BYTE_STREAM_SPLIT)
            .set_column_dictionary_enabled(col("tick"), false)
            .set_column_encoding(col("tick"), Encoding::DELTA_BINARY_PACKED)
            .set_max_row_group_row_count(Some(ROW_GROUP))
            .set_key_value_metadata(Some(footer))
            .build();
        let sink = Digesting {
            inner: sink,
            fnv: Fnv::new(),
            bytes: 0,
        };
        Ok(TelemetryWriter {
            writer: SerializedFileWriter::new(sink, schema, Arc::new(props))?,
            names: Names::of(world),
            rows: Columns::default(),
            written: 0,
        })
    }

    /// Add one report's rows, writing each row group as it fills. A report that names an id the
    /// world lacks is refused whole, before any of its rows is kept.
    pub fn push(&mut self, r: &TickReport) -> Result<(), TelemetryError> {
        let tick = i64::try_from(r.tick)
            .map_err(|_| TelemetryError::Report(format!("tick {} is past i64", r.tick)))?;
        for row in self.names.rows(r)? {
            self.rows.push(tick, row);
            self.written += 1;
            if self.rows.len() >= ROW_GROUP {
                self.rows.flush(&mut self.writer)?;
            }
        }
        Ok(())
    }

    /// The rows pushed so far.
    pub fn rows(&self) -> u64 {
        self.written
    }

    /// Write the last row group and the footer, flush the sink, and hand it back with the rows,
    /// bytes and digest the manifest records.
    pub fn finish(mut self) -> Result<Finished<W>, TelemetryError> {
        self.rows.flush(&mut self.writer)?;
        let mut sink = self.writer.into_inner()?;
        sink.flush()?;
        Ok(Finished {
            sink: sink.inner,
            rows: self.written,
            bytes: sink.bytes,
            digest: sink.fnv.finish(),
        })
    }
}
