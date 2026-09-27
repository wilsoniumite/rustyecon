//! Decimation for drawing (docs/GUI.md §3.4). egui_plot draws every vertex it is given, so a
//! long series is thinned once, to about two points per pixel column, and extended as ticks
//! arrive. Thinning keeps what the eye must see: every drawn vertex is a recorded `(tick,
//! value)`, and each column keeps its true minimum and maximum. A gap in the record (a tick with
//! no value) splits the line into segments and is never bridged.

use super::Series;

/// One column being filled: its index and its extreme points so far.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Column {
    index: u64,
    min: (u64, f64),
    max: (u64, f64),
}

impl Column {
    /// The column's points in tick order: its minimum and its maximum, or one point when they
    /// are the same point.
    fn points(&self) -> impl Iterator<Item = (u64, f64)> {
        let (a, b) = if self.min.0 <= self.max.0 {
            (self.min, self.max)
        } else {
            (self.max, self.min)
        };
        std::iter::once(a).chain((b.0 != a.0).then_some(b))
    }
}

/// Thins a series column by column: columns of `width` ticks, counted from `origin`. It takes
/// points in tick order, closes a column when a point falls in a later one, and keeps the closed
/// columns' points, so extending it costs only the new points.
#[derive(Debug, Clone, PartialEq)]
pub struct Decimator {
    origin: u64,
    width: u64,
    /// Points of the series already taken.
    taken: usize,
    /// The closed columns' points, in tick order.
    done: Vec<(u64, f64)>,
    /// Where each segment after the first starts in `done` (or at its end, for the open column).
    breaks: Vec<usize>,
    open: Option<Column>,
    last: Option<u64>,
}

impl Decimator {
    /// A decimator of columns `width` ticks wide (at least 1) from tick `origin`.
    pub fn new(origin: u64, width: u64) -> Decimator {
        Decimator {
            origin,
            width: width.max(1),
            taken: 0,
            done: Vec::new(),
            breaks: Vec::new(),
            open: None,
            last: None,
        }
    }

    /// The width of a column, in ticks.
    pub fn width(&self) -> u64 {
        self.width
    }

    /// Take one recorded point. Points come in increasing tick order; one before `origin` is
    /// ignored.
    pub fn push(&mut self, tick: u64, value: f64) {
        if tick < self.origin || self.last.is_some_and(|l| tick <= l) {
            return;
        }
        let gap = self.last.is_some_and(|l| tick != l + 1);
        let index = (tick - self.origin) / self.width;
        if gap || self.open.is_some_and(|c| c.index != index) {
            self.close();
        }
        if gap {
            self.breaks.push(self.done.len());
        }
        match &mut self.open {
            Some(c) => {
                if value < c.min.1 {
                    c.min = (tick, value);
                }
                if value > c.max.1 {
                    c.max = (tick, value);
                }
            }
            None => {
                self.open = Some(Column {
                    index,
                    min: (tick, value),
                    max: (tick, value),
                });
            }
        }
        self.last = Some(tick);
    }

    /// Take the points `series` gained since the last call. The series must be the same one,
    /// grown only at its end.
    pub fn extend(&mut self, series: &Series) {
        let n = series.len();
        let from = self.taken.min(n);
        for (t, v) in series.points(from, n) {
            self.push(t, v);
        }
        self.taken = n;
    }

    fn close(&mut self) {
        if let Some(c) = self.open.take() {
            self.done.extend(c.points());
        }
    }

    /// The thinned line: one list of recorded points per unbroken stretch of the record, in
    /// tick order.
    pub fn segments(&self) -> Vec<Vec<(u64, f64)>> {
        let mut all = self.done.clone();
        if let Some(c) = &self.open {
            all.extend(c.points());
        }
        let mut out = Vec::with_capacity(self.breaks.len() + 1);
        let mut from = 0;
        for &b in &self.breaks {
            out.push(all[from..b].to_vec());
            from = b;
        }
        out.push(all[from..].to_vec());
        out.retain(|s| !s.is_empty());
        out
    }
}

/// The points of `series` with ticks in `[lo, hi)`, thinned to at most two per column over
/// `columns` columns of equal width, split at every gap in the record.
pub fn decimate(series: &Series, lo: u64, hi: u64, columns: usize) -> Vec<Vec<(u64, f64)>> {
    let span = hi.saturating_sub(lo);
    let columns = u64::try_from(columns.max(1)).unwrap_or(u64::MAX);
    let mut d = Decimator::new(lo, span.div_ceil(columns));
    let (from, to) = series.range(lo, hi);
    for (t, v) in series.points(from, to) {
        d.push(t, v);
    }
    d.segments()
}
