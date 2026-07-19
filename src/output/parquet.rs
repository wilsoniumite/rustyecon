//! Streaming Parquet writer for run telemetry (v2 Phase 2, spec architecture/engine.md).
//!
//! One tidy long table per run — `(tick, region, entity_kind, id, good, metric,
//! value)` — written incrementally as row groups. Nothing accumulates for the
//! length of the run: the buffer is bounded by `rows_per_group` and flushed, so
//! telemetry volume is a disk question rather than a memory one. That matters at
//! the 11,700-tick horizon Phase 3 exists to probe.
//!
//! Deviation from the spec, recorded rather than silently taken: engine.md gives
//! six columns and omits `good`. Prices, supplies and inventories are all keyed
//! by *(entity, good)*, so six columns cannot express them without folding the
//! good into `id` or into the metric name — both of which push structure into a
//! string and make the obvious groupby wrong. `good` is therefore a seventh,
//! nullable column: null means the observation is not per-good (a region's
//! velocity), not "good zero".
//!
//! `region` is likewise nullable: a market node can serve several regions, so a
//! price belongs to a node and not to any one region. The manifest carries the
//! region→node map for analysis that wants to join them.

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use parquet::basic::{Compression, ZstdLevel};
use parquet::data_type::{ByteArray, ByteArrayType, DoubleType, Int32Type, Int64Type};
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// The tidy long schema. Column order here is the order [`Buffer::flush`] writes
/// them in; the two must agree, and the round-trip test is what holds them to it.
const SCHEMA: &str = "
message telemetry {
  REQUIRED INT64 tick;
  OPTIONAL INT32 region;
  REQUIRED BYTE_ARRAY entity_kind (STRING);
  REQUIRED INT32 id;
  OPTIONAL INT32 good;
  REQUIRED BYTE_ARRAY metric (STRING);
  REQUIRED DOUBLE value;
}
";

/// Rows buffered before a row group is flushed. Chosen so a row group is a few
/// MB — large enough that per-group metadata is negligible, small enough that
/// peak memory stays flat regardless of run length.
pub const DEFAULT_ROWS_PER_GROUP: usize = 250_000;

/// Struct-of-arrays staging for one row group.
///
/// Nullable columns keep values and definition levels separately: `*_vals` holds
/// only the present values, `*_def` holds one level per row (1 present, 0 null).
/// That is Parquet's own representation, so no sentinel value is ever invented.
#[derive(Default)]
struct Buffer {
    tick: Vec<i64>,
    region_vals: Vec<i32>,
    region_def: Vec<i16>,
    entity_kind: Vec<ByteArray>,
    id: Vec<i32>,
    good_vals: Vec<i32>,
    good_def: Vec<i16>,
    metric: Vec<ByteArray>,
    value: Vec<f64>,
}

impl Buffer {
    fn len(&self) -> usize {
        self.tick.len()
    }

    fn clear(&mut self) {
        self.tick.clear();
        self.region_vals.clear();
        self.region_def.clear();
        self.entity_kind.clear();
        self.id.clear();
        self.good_vals.clear();
        self.good_def.clear();
        self.metric.clear();
        self.value.clear();
    }
}

/// Writes tidy long rows to a Parquet file, flushing row groups as it goes.
pub struct TelemetryWriter {
    writer: SerializedFileWriter<File>,
    buf: Buffer,
    rows_per_group: usize,
    total_rows: u64,
    /// `entity_kind` and `metric` repeat for millions of rows. `ByteArray` is
    /// refcounted, so interning turns each push into a cheap clone instead of an
    /// allocation; Parquet's dictionary encoding then makes them near-free on
    /// disk as well.
    interned: HashMap<&'static str, ByteArray>,
}

fn intern(cache: &mut HashMap<&'static str, ByteArray>, s: &'static str) -> ByteArray {
    cache
        .entry(s)
        .or_insert_with(|| ByteArray::from(s.as_bytes().to_vec()))
        .clone()
}

impl TelemetryWriter {
    pub fn create(path: &Path, rows_per_group: usize) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let schema = Arc::new(parse_message_type(SCHEMA)?);
        // Zstd over Snappy: telemetry is written once and read many times, the
        // long format is extremely repetitive, and Phase 3 gates on total volume.
        let props = Arc::new(
            WriterProperties::builder()
                .set_compression(Compression::ZSTD(ZstdLevel::try_new(3)?))
                .build(),
        );
        let file = File::create(path)?;
        Ok(Self {
            writer: SerializedFileWriter::new(file, schema, props)?,
            buf: Buffer::default(),
            rows_per_group: rows_per_group.max(1),
            total_rows: 0,
            interned: HashMap::new(),
        })
    }

    /// Append one observation. Flushes a row group once the buffer is full.
    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &mut self,
        tick: u64,
        region: Option<u32>,
        entity_kind: &'static str,
        id: u32,
        good: Option<u32>,
        metric: &'static str,
        value: f64,
    ) -> Result<()> {
        self.buf.tick.push(tick as i64);
        match region {
            Some(r) => {
                self.buf.region_vals.push(r as i32);
                self.buf.region_def.push(1);
            }
            None => self.buf.region_def.push(0),
        }
        let kind = intern(&mut self.interned, entity_kind);
        self.buf.entity_kind.push(kind);
        self.buf.id.push(id as i32);
        match good {
            Some(g) => {
                self.buf.good_vals.push(g as i32);
                self.buf.good_def.push(1);
            }
            None => self.buf.good_def.push(0),
        }
        let m = intern(&mut self.interned, metric);
        self.buf.metric.push(m);
        self.buf.value.push(value);

        if self.buf.len() >= self.rows_per_group {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.buf.len() == 0 {
            return Ok(());
        }
        let mut rg = self.writer.next_row_group()?;

        macro_rules! column {
            ($ty:ty, $values:expr, $def:expr) => {{
                let mut col = rg
                    .next_column()?
                    .ok_or("telemetry schema has fewer columns than the writer emits")?;
                col.typed::<$ty>().write_batch($values, $def, None)?;
                col.close()?;
            }};
        }

        column!(Int64Type, &self.buf.tick, None);
        column!(Int32Type, &self.buf.region_vals, Some(&self.buf.region_def));
        column!(ByteArrayType, &self.buf.entity_kind, None);
        column!(Int32Type, &self.buf.id, None);
        column!(Int32Type, &self.buf.good_vals, Some(&self.buf.good_def));
        column!(ByteArrayType, &self.buf.metric, None);
        column!(DoubleType, &self.buf.value, None);

        if rg.next_column()?.is_some() {
            return Err("telemetry schema has more columns than the writer emits".into());
        }
        rg.close()?;

        self.total_rows += self.buf.len() as u64;
        self.buf.clear();
        Ok(())
    }

    /// Flush the tail and finalise the file. Returns the total rows written.
    ///
    /// Must be called: Parquet keeps its schema and row-group index in a footer,
    /// so a file that is never closed is not a short file, it is an unreadable
    /// one. The caller reporting the error is the point — a silently truncated
    /// telemetry file would be an analysis that quietly studies a prefix.
    pub fn close(mut self) -> Result<u64> {
        self.flush()?;
        let total = self.total_rows;
        self.writer.close()?;
        Ok(total)
    }
}
