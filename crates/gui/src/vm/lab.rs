//! The oracle lab's view-models (docs/GUI.md §9, G1): an instance solved, with each output
//! beside its golden; any field of the price and quantity block plotted over x in [0, 1], so
//! f(x) = n_D − n_S shows its root and its bracket; and a sweep of one knob.
//!
//! Every number here is the oracle's own double, or a golden as the generator wrote it, or the
//! relative difference of the two (a display transform, U6). Each view-model names its origin,
//! "oracle", its unit and instance, and whether the instance was edited from its preset (U3).

use crate::lab::goldens::{self, Golden, GoldenValue};
use crate::lab::presets::Preset;
use crate::lab::{fields, knobs, Instance, OracleUnit, Solved, Validated, Value};
use oracle::{BRACKET_HI, BRACKET_LO};
use serde::Serialize;

/// A golden beside an output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoldenCellVm {
    /// Its key in the goldens file.
    pub key: String,
    /// Its value as the generator wrote it.
    pub text: String,
    /// The output's difference from it: relative, |out − golden|/|golden| (absolute where the
    /// golden is 0), or absolute for a published value; `None` for a flag.
    pub diff: Option<f64>,
    /// Whether `diff` is absolute: a value the paper published to five decimals.
    pub absolute: bool,
    /// Whether the output agrees: a generator's number within [`AGREES`] relative, a published
    /// one within [`PUBLISHED`] absolute, a flag equal.
    pub agrees: bool,
    /// The generator's note.
    pub note: String,
}

/// The bar below which an output agrees with a generator's golden: 1e-12 relative, the
/// oracle's gate's own (`FULL`).
pub const AGREES: f64 = 1e-12;
/// The bar for a value the paper published to five decimals: 5e-6 absolute, half a unit in the
/// last place, as the oracle's gate holds it (`PUBLISHED`; ADDENDUM A7).
pub const PUBLISHED: f64 = 5e-6;

/// One output's row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OutputRowVm {
    /// The key the oracle prints it under.
    pub key: String,
    /// The value as the oracle's dump prints it: a float's shortest round-trip digits, so the
    /// text reads back as the oracle's double.
    pub value: String,
    /// The number, where it is one.
    pub number: Option<f64>,
    /// The generator's golden of the same name, if there is one.
    pub golden: Option<GoldenCellVm>,
    /// The paper's published value of the same name, if there is one.
    pub published: Option<GoldenCellVm>,
}

/// The lab's instance, solved.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabVm {
    /// The origin of every number here (U3).
    pub origin: String,
    /// The build that solved it.
    pub build: String,
    /// The unit.
    pub unit: String,
    /// What the unit adds.
    pub unit_title: String,
    /// The preset, by its name, and whether the form changed it.
    pub preset: Option<String>,
    /// The preset's line, and the generator's heading of it.
    pub title: String,
    /// The generator's heading, as its goldens file writes it.
    pub heading: Option<String>,
    /// Whether the instance differs from its preset: its outputs are then no longer its
    /// goldens' economy, and no golden is paired.
    pub edited: bool,
    /// The goldens file.
    pub file: Option<String>,
    /// The regime, as the oracle names it.
    pub regime: String,
    /// What else the oracle says of it.
    pub detail: Vec<(String, String)>,
    /// Every output, in the oracle's order.
    pub outputs: Vec<OutputRowVm>,
    /// The instance's goldens that no output of that name pairs with.
    pub unpaired: Vec<GoldenCellVm>,
    /// How many outputs are paired with a golden, and how many of those agree.
    pub paired: (usize, usize),
}

/// A golden beside an output: a generator's, relative to [`AGREES`], or a published one,
/// absolute to [`PUBLISHED`].
fn cell(g: &Golden, out: Option<Value>, published: bool) -> GoldenCellVm {
    let (diff, agrees) = match (g.value, out) {
        (GoldenValue::Number(want), Some(v)) => match v.number() {
            Some(got) if published => {
                let d = (got - want).abs();
                (Some(d), d <= PUBLISHED)
            }
            Some(got) => {
                let r = goldens::relative(got, want);
                (Some(r), r <= AGREES)
            }
            None => (None, false),
        },
        (GoldenValue::Flag(want), Some(Value::Flag(got))) => (None, want == got),
        _ => (None, false),
    };
    GoldenCellVm {
        key: g.key.clone(),
        text: g.text.clone(),
        diff,
        absolute: published,
        agrees,
        note: g.note.clone(),
    }
}

/// The lab's view of `inst`, solved, beside the goldens of `preset` when the instance is the
/// preset's own. `build` names the binary (U3).
pub fn build(preset: Option<&Preset>, inst: &Instance, build: &str) -> LabVm {
    let all = preset.map_or_else(Vec::new, |p| goldens::of(p.file, p.prefix));
    build_beside(preset, inst, build, &all)
}

/// The lab's view of `inst`, solved, beside `goldens`, read as the goldens of `preset`'s prefix
/// when the instance is the preset's own; an edited instance pairs none. [`build`] passes the
/// preset's goldens file; a test passes goldens it doctored, to see a disagreement shown as one.
pub fn build_beside(
    preset: Option<&Preset>,
    inst: &Instance,
    build: &str,
    goldens: &[Golden],
) -> LabVm {
    let solved: Solved = inst.solve();
    let edited = preset.is_some_and(|p| p.instance() != *inst);
    let own = preset.filter(|_| !edited);
    let all: Vec<Golden> = own.map_or_else(Vec::new, |_| goldens.to_vec());
    let mut used = vec![false; all.len()];
    let mut find = |key: &str| -> Option<(usize, &Golden)> {
        let i = all.iter().position(|g| g.key == key)?;
        used[i] = true;
        Some((i, &all[i]))
    };
    let mut paired = (0, 0);
    let outputs: Vec<OutputRowVm> = solved
        .outputs
        .iter()
        .map(|(key, v)| {
            let (golden, published) = match own {
                Some(p) => {
                    let k = goldens::key_of(p.prefix, key);
                    let golden = find(&k).map(|(_, g)| cell(g, Some(*v), false));
                    let published = find(&format!("PUB_{k}")).map(|(_, g)| cell(g, Some(*v), true));
                    (golden, published)
                }
                None => (None, None),
            };
            if let Some(g) = &golden {
                paired.0 += 1;
                paired.1 += usize::from(g.agrees);
            }
            OutputRowVm {
                key: key.clone(),
                value: v.to_string(),
                number: v.number(),
                golden,
                published,
            }
        })
        .collect();
    let unpaired = all
        .iter()
        .zip(&used)
        .filter(|(_, u)| !**u)
        .map(|(g, _)| cell(g, None, g.key.starts_with("PUB_")))
        .collect();
    let unit = inst.unit();
    LabVm {
        origin: "oracle".to_string(),
        build: build.to_string(),
        unit: unit.to_string(),
        unit_title: unit.title().to_string(),
        preset: preset.map(|p| p.id.to_string()),
        title: preset.map_or_else(String::new, |p| p.title.to_string()),
        heading: preset.and_then(|p| goldens::heading(p.file, p.prefix)),
        edited,
        file: preset.map(|p| p.file.to_string()),
        regime: solved.regime,
        detail: solved.detail,
        outputs,
        unpaired,
        paired,
    }
}

/// A field of the price and quantity block over x.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CurveVm {
    /// The unit.
    pub unit: OracleUnit,
    /// The field plotted.
    pub field: String,
    /// Every field a point has, f first.
    pub fields: Vec<String>,
    /// The curve, in unbroken stretches of finite values: `[x, value]`.
    pub segments: Vec<Vec<[f64; 2]>>,
    /// The bracket the solve decides the regime on, [`BRACKET_LO`] to [`BRACKET_HI`].
    pub bracket: (f64, f64),
    /// The field at the bracket's ends, where finite.
    pub at_ends: (Option<f64>, Option<f64>),
    /// x*, at an equilibrium.
    pub root: Option<f64>,
    /// The field at x*, where finite.
    pub at_root: Option<f64>,
    /// The regime.
    pub regime: String,
    /// Why there is no curve, if there is none.
    pub error: Option<String>,
}

/// The field's value at a point, by its path; f is the point's own `excess_demand`.
fn field_at(e: &Validated, x: f64, field: &str) -> Option<f64> {
    let p = e.at(x);
    if field == fields::F {
        return Some(p.excess_demand());
    }
    fields::read(&p.debug())
        .into_iter()
        .find(|(k, _)| k == field)
        .map(|(_, v)| v)
}

/// `field` over x in [0, 1] at `n` + 1 even steps, with the bracket's ends and x* among the
/// points, and the root and the bracket marked.
pub fn curve(inst: &Instance, field: &str, n: usize) -> CurveVm {
    let unit = inst.unit();
    let fail = |why: String, regime: String| CurveVm {
        unit,
        field: field.to_string(),
        fields: Vec::new(),
        segments: Vec::new(),
        bracket: (BRACKET_LO, BRACKET_HI),
        at_ends: (None, None),
        root: None,
        at_root: None,
        regime,
        error: Some(why),
    };
    let e = match inst.validate() {
        Ok(e) => e,
        Err(why) => return fail(why, "invalid".to_string()),
    };
    let solved = e.solve();
    let names: Vec<String> = e.at(0.5).fields().into_iter().map(|(k, _)| k).collect();
    if !names.iter().any(|k| k == field) {
        let mut c = fail(
            format!("a point of unit {unit} has no field {field}"),
            solved.regime,
        );
        c.fields = names;
        return c;
    }
    let n = n.max(1);
    let mut xs: Vec<f64> = (0..=n).map(|i| i as f64 / n as f64).collect();
    xs.push(BRACKET_LO);
    if let Some(r) = solved.x_star {
        xs.push(r);
    }
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|a, b| a.to_bits() == b.to_bits());
    let mut segments: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut open = false;
    for &x in &xs {
        match field_at(&e, x, field).filter(|v| v.is_finite()) {
            Some(v) => {
                if !open {
                    segments.push(Vec::new());
                    open = true;
                }
                if let Some(s) = segments.last_mut() {
                    s.push([x, v]);
                }
            }
            None => open = false,
        }
    }
    let finite = |x: f64| field_at(&e, x, field).filter(|v| v.is_finite());
    CurveVm {
        unit,
        field: field.to_string(),
        fields: names,
        segments,
        bracket: (BRACKET_LO, BRACKET_HI),
        at_ends: (finite(BRACKET_LO), finite(BRACKET_HI)),
        root: solved.x_star,
        at_root: solved.x_star.and_then(finite),
        regime: solved.regime,
        error: None,
    }
}

/// One output along a sweep.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SweepLineVm {
    /// The output's key.
    pub key: String,
    /// Its value at each point, `None` where the point has no equilibrium or the output no
    /// number.
    pub values: Vec<Option<f64>>,
}

/// A sweep of one knob.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SweepVm {
    /// The knob swept.
    pub knob: String,
    /// Its values, `from` to `to` in even steps.
    pub xs: Vec<f64>,
    /// Each output asked for.
    pub lines: Vec<SweepLineVm>,
    /// The regime at each point.
    pub regimes: Vec<String>,
    /// How many points each regime took, in the order first met.
    pub counts: Vec<(String, usize)>,
}

/// Solve `inst` at `points` values of `knob`, `from` to `to` in even steps, and read
/// `outputs` of each. A value the knob does not take, or an instance the oracle refuses, is
/// that point's regime ("invalid", "refused"); a sweep of a knob `inst` lacks is an error.
pub fn sweep(
    inst: &Instance,
    knob: &str,
    from: f64,
    to: f64,
    points: usize,
    outputs: &[String],
) -> Result<SweepVm, String> {
    knobs::get(inst, knob).ok_or_else(|| format!("unit {} has no knob {knob}", inst.unit()))?;
    if !(from.is_finite() && to.is_finite()) {
        return Err(format!("the range {from} to {to} is not finite"));
    }
    let points = points.max(2);
    let step = (to - from) / (points - 1) as f64;
    let xs: Vec<f64> = (0..points)
        .map(|i| {
            if i + 1 == points {
                to
            } else {
                from + step * i as f64
            }
        })
        .collect();
    let mut lines: Vec<SweepLineVm> = outputs
        .iter()
        .map(|k| SweepLineVm {
            key: k.clone(),
            values: Vec::with_capacity(points),
        })
        .collect();
    let mut regimes = Vec::with_capacity(points);
    let mut counts: Vec<(String, usize)> = Vec::new();
    let mut at = inst.clone();
    for &x in &xs {
        let solved = match knobs::set_value(&mut at, knob, x) {
            Ok(()) => at.solve(),
            Err(why) => Solved {
                regime: "invalid".to_string(),
                detail: vec![("why".to_string(), why)],
                x_star: None,
                outputs: Vec::new(),
            },
        };
        for l in &mut lines {
            l.values.push(solved.output(&l.key).and_then(Value::number));
        }
        match counts.iter_mut().find(|(r, _)| *r == solved.regime) {
            Some((_, n)) => *n += 1,
            None => counts.push((solved.regime.clone(), 1)),
        }
        regimes.push(solved.regime);
    }
    Ok(SweepVm {
        knob: knob.to_string(),
        xs,
        lines,
        regimes,
        counts,
    })
}
