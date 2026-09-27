//! The history, composed and stepped (docs/demo/WORLD.md §4.3).
//!
//! - **Composition.** A county's value of a param in month m is its path value (the latest
//!   `path` ramp begun, else its genesis value) times the product of every `scale` ramp's
//!   factor `(to/from)^(w·s(m))`, with s(m) = clamp((m − start)/(end − start), 0, 1) in months
//!   and w the county's weight. Ramps on one param multiply, so overlapping waves compose.
//! - **Threshold stepping.** On the union of the param's ramps' grids (the first of a month),
//!   a step is emitted when the composed value has moved by `step_log` or more in log since the
//!   last step, and at each ramp's end when it has moved at all. A step's value is the composed
//!   value to six significant figures, as the tables write their numbers; the threshold is
//!   measured from the composed value, so rounding never accumulates.
//!
//! Every transcendental is core's `num`, libm's, so the steps are the same on every platform
//! (A5).

use crate::tables::{County, Mode, Param, Ramp, Start, World};
use crate::CompileError;
use rustyecon_core::num;
use std::collections::BTreeMap;

/// A month, counted from January of the world's first year.
pub type Month = i64;

/// The first of the month `m` as a tape date, `YYYY-MM-01`.
pub fn date(w: &World, m: Month) -> String {
    let y = i64::from(w.start_year()) + m.div_euclid(12);
    format!("{y:04}-{:02}-01", m.rem_euclid(12) + 1)
}

/// `YYYY-MM`, the suffix of a step's key.
pub fn stamp(w: &World, m: Month) -> String {
    let y = i64::from(w.start_year()) + m.div_euclid(12);
    format!("{y:04}-{:02}", m.rem_euclid(12) + 1)
}

/// One ramp as it moves one county.
#[derive(Debug, Clone, Copy)]
struct Applied<'a> {
    ramp: &'a Ramp,
    /// Index of the ramp in the table.
    index: usize,
    start: Month,
    end: Month,
    w: f64,
}

impl Applied<'_> {
    fn s(&self, m: Month) -> f64 {
        if m <= self.start {
            0.0
        } else if m >= self.end {
            1.0
        } else {
            (m - self.start) as f64 / (self.end - self.start) as f64
        }
    }

    /// Its grid: the first of every month a step may fall on, after its start and up to and
    /// including its end.
    fn grid(&self) -> impl Iterator<Item = Month> + '_ {
        let every = 12 / i64::from(self.ramp.steps_per_year);
        let n = (self.end - self.start) / every;
        (1..=n).map(move |k| self.start + k * every)
    }
}

/// One step of one param of one county: a `SetParam` dated the first of `month`.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// The month it falls on.
    pub month: Month,
    /// The param it sets.
    pub param: Param,
    /// The value it sets, to six significant figures.
    pub value: f64,
    /// The ramps that moved the param since the last step, by index into the table.
    pub ramps: Vec<usize>,
}

/// A value to six significant figures, as the tables write their numbers. Rust formats a
/// double exactly, so this is the same on every platform.
pub fn six(v: f64) -> f64 {
    format!("{v:.5e}").parse().unwrap_or(v)
}

fn months(w: &World, year: i32) -> Month {
    i64::from(year - w.start_year()) * 12
}

/// The ramps that move `c`, per param.
fn applied<'a>(
    w: &World,
    c: &County,
    ramps: &'a [Ramp],
) -> Result<[Vec<Applied<'a>>; 11], CompileError> {
    let mut out: [Vec<Applied<'a>>; 11] = Default::default();
    for (index, r) in ramps.iter().enumerate() {
        if !r.regions.selects(c) {
            continue;
        }
        let wt = r.weight.of(c);
        if wt == 0.0 {
            continue;
        }
        let start = match r.start {
            Start::Year(y) => y,
            Start::TextileFrom => c.textile_from.unwrap_or(r.end),
        };
        if start >= r.end {
            return Err(CompileError::new(
                format!("history.csv:{} {}", r.line, r.key),
                format!(
                    "{} starts its textiles in {start}, not before the ramp's end {}",
                    c.key, r.end
                ),
            ));
        }
        out[r.param.index()].push(Applied {
            ramp: r,
            index,
            start: months(w, start),
            end: months(w, r.end),
            w: wt,
        });
    }
    // Paths: one at a time, each starting where the last ended, the first at genesis.
    for (p, list) in out.iter().enumerate() {
        let mut paths: Vec<&Applied<'_>> =
            list.iter().filter(|a| a.ramp.mode == Mode::Path).collect();
        paths.sort_by_key(|a| a.start);
        let mut level = c.genesis[p];
        let mut free_from = Month::MIN;
        for a in paths {
            let at = format!("history.csv:{} {}", a.ramp.line, a.ramp.key);
            if a.start < free_from {
                return Err(CompileError::new(
                    at,
                    format!(
                        "it overlaps another path of {}'s {}",
                        c.key,
                        a.ramp.param.column()
                    ),
                ));
            }
            if (a.ramp.from / level - 1.0).abs() > 1e-9 {
                return Err(CompileError::new(
                    at,
                    format!(
                        "it starts {}'s {} at {}, where the value is {level}",
                        c.key,
                        a.ramp.param.column(),
                        a.ramp.from
                    ),
                ));
            }
            level = a.ramp.to;
            free_from = a.end;
        }
    }
    Ok(out)
}

/// A param's composed value in month `m`.
fn value(genesis: f64, list: &[Applied<'_>], m: Month) -> f64 {
    let mut best: Option<&Applied<'_>> = None;
    for a in list.iter().filter(|a| a.ramp.mode == Mode::Path) {
        if a.start <= m && best.is_none_or(|b| a.start > b.start) {
            best = Some(a);
        }
    }
    let base = match best {
        None => genesis,
        Some(a) => {
            let s = a.s(m);
            if s <= 0.0 {
                a.ramp.from
            } else if s >= 1.0 {
                a.ramp.to
            } else {
                a.ramp.from * num::pow(a.ramp.to / a.ramp.from, s)
            }
        }
    };
    let mut f = 1.0;
    for a in list.iter().filter(|a| a.ramp.mode == Mode::Scale) {
        let s = a.s(m);
        if s > 0.0 {
            f *= num::pow(a.ramp.to / a.ramp.from, a.w * s);
        }
    }
    base * f
}

/// Every step of a county's history, in (month, param) order.
pub fn steps(w: &World, c: &County, ramps: &[Ramp]) -> Result<Vec<Step>, CompileError> {
    let ap = applied(w, c, ramps)?;
    let mut out = Vec::new();
    for p in Param::ALL {
        let list = &ap[p.index()];
        if list.is_empty() {
            continue;
        }
        let genesis = c.genesis[p.index()];
        // month -> whether some ramp ends there
        let mut grid: BTreeMap<Month, bool> = BTreeMap::new();
        for a in list {
            for m in a.grid() {
                let e = grid.entry(m).or_insert(false);
                *e = *e || m == a.end;
            }
        }
        let mut last = genesis;
        let mut last_m = Month::MIN;
        for (m, end) in grid {
            let v = value(genesis, list, m);
            let moved = num::ln(v / last).abs();
            if moved >= w.step_log || (end && moved > 0.0) {
                let mut moving: Vec<usize> = list
                    .iter()
                    .filter(|a| a.start < m && a.end > last_m)
                    .map(|a| a.index)
                    .collect();
                moving.sort_unstable();
                moving.dedup();
                out.push(Step {
                    month: m,
                    param: p,
                    value: six(v),
                    ramps: moving,
                });
                last = v;
                last_m = m;
            }
        }
    }
    out.sort_by_key(|s| (s.month, s.param));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::six;

    #[test]
    fn six_significant_figures() {
        assert_eq!(six(1_636.681_234_5), 1636.68);
        assert_eq!(six(0.023_184_444), 0.023_184_4);
        assert_eq!(six(110.24), 110.24);
        assert_eq!(six(9.999_996), 10.0);
    }
}
