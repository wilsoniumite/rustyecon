//! The map's view-model (docs/GUI.md §4 "Lenses", §6; D.3, 2026-09-27): one [`LensVm`] per
//! lens feeds the map's colours, its legend and the ranked table beside it, so the map's values
//! are the table's (G4's gate): the table's rows are indices into the same regions.
//!
//! - **Geography.** A tape has a map when every node is a region of the atlas (D7's keys,
//!   `county.<chapman>`). A tape with a node the atlas lacks is refused, and one with no
//!   region at all has none (§6). A region of the atlas the tape lacks is drawn hatched grey.
//! - **Values.** A lens's value for a county is `rustyecon_worldgen::lens::value` of the
//!   county's recorded numbers at the report tick ([`readings`]): prices, cleared volumes,
//!   whether each market traded, rationing lines, params and the roles' states, as the run
//!   recorded them, and N per tick through the engine's own conversion for its use
//!   (`ClockMethod::per_tick`). Nothing here defines a measure (U6).
//! - **Scales.** Neutral, and fixed for the run: each lens's domain is the one `lenses.csv`
//!   registers, chosen at design from the oracle's range over every county and step with a
//!   margin (docs/demo/WORLD.md §6), never from the run's values. A value outside it takes the
//!   end colour and is marked below or above. A diverging lens centres on its reference. A
//!   position on the scale is a display transform: a ratio, or a log through `core::num`.
//! - **Names.** Every value is named with its run (build, `world_id`, `tape_hash`, origin) and
//!   its report tick and date (U3).

use super::toolbar::IdentityVm;
use super::{date, report_tick};
use crate::run::{At, Entity, Measure, Origin, SeriesKey, StateField, Store};
use rustyecon_engine::num;
use rustyecon_engine::prelude::*;
use rustyecon_worldgen::atlas::{Atlas, Country};
use rustyecon_worldgen::lens::{self as wl, CountyKeys, Readings, MARKETS, NO_TRADE_WINDOW};
use rustyecon_worldgen::tables::{Lens, LensSource, Param, Scale};
use serde::Serialize;

/// Where a value sits against its lens's domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Beyond {
    /// Inside the domain.
    Within,
    /// Below it: drawn in the low end's colour.
    Below,
    /// Above it: drawn in the high end's colour.
    Above,
}

/// One region of the atlas under a lens.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegionVm {
    /// Its key, `county.<chapman>`.
    pub key: String,
    /// Its name.
    pub name: String,
    /// Its Chapman code.
    pub chapman: String,
    /// Its country.
    pub country: String,
    /// Whether the tape has it as a node. One it lacks is drawn hatched grey.
    pub in_tape: bool,
    /// The lens's value, when it has one.
    pub value: Option<f64>,
    /// Where the value sits on the scale, from 0 (the domain's low end) to 1, clamped.
    pub at: Option<f64>,
    /// Whether the value lies outside the domain.
    pub beyond: Beyond,
    /// Why it has no value, when it has none.
    pub why: Option<String>,
    /// Its rank, from 1 for the largest value, when it has a value.
    pub rank: Option<usize>,
}

/// A mark on the legend.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TickVm {
    /// The value.
    pub value: f64,
    /// Where it sits on the scale, from 0 to 1.
    pub at: f64,
}

/// A lens's reference, where it has one.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReferenceVm {
    /// Its value.
    pub value: f64,
    /// What it is, as the table says.
    pub text: String,
    /// Where it sits on the scale, when it lies in the domain.
    pub at: Option<f64>,
}

/// How many regions show what.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub struct CountsVm {
    /// Regions of the atlas.
    pub regions: usize,
    /// Of them, the tape's nodes.
    pub in_tape: usize,
    /// Of those, with a value.
    pub valued: usize,
    /// Of those, below the domain.
    pub below: usize,
    /// Of those, above it.
    pub above: usize,
}

/// One lens at one report tick: the map's colours, its legend and the ranked table.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LensVm {
    /// Its key.
    pub key: String,
    /// Its name.
    pub name: String,
    /// What it measures, as a formula.
    pub measure: String,
    /// The unit of its values.
    pub unit: String,
    /// `sequential`, `sequential, log` or `diverging`.
    pub scale: String,
    /// The fixed domain, for the whole run: `lenses.csv`'s.
    pub domain: (f64, f64),
    /// A diverging scale's centre: its reference.
    pub centre: Option<f64>,
    /// Its reference, where it has one.
    pub reference: Option<ReferenceVm>,
    /// Its inputs, as the table names them.
    pub inputs: String,
    /// The table's note.
    pub note: String,
    /// Why no county has a value, when the lens waits for what is not built.
    pub unavailable: Option<String>,
    /// The legend's marks.
    pub legend: Vec<TickVm>,
    /// The run the values belong to.
    pub run: Option<IdentityVm>,
    /// The report tick the values are read at, once a tick has run.
    pub tick: Option<u64>,
    /// Its date.
    pub date: String,
    /// Every region of the atlas, in key order.
    pub regions: Vec<RegionVm>,
    /// The ranked table: indices into `regions`, largest value first, ties by key. Only regions
    /// with a value.
    pub ranked: Vec<usize>,
    /// The counts.
    pub counts: CountsVm,
}

/// One lens as the selector lists it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LensItemVm {
    /// Its key.
    pub key: String,
    /// Its name.
    pub name: String,
    /// Its unit.
    pub unit: String,
    /// The selector's group.
    pub group: String,
    /// Why it cannot be shown, when it cannot.
    pub unavailable: Option<String>,
}

/// One recorded number of the selected county, with its series so it can be plotted.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InputVm {
    /// What it is.
    pub label: String,
    /// Its value at the report tick.
    pub value: Option<f64>,
    /// Its unit.
    pub unit: String,
    /// Its series.
    pub series: SeriesKey,
}

/// One lens's value for the selected county.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CountyLensVm {
    /// The lens.
    pub key: String,
    /// Its name.
    pub name: String,
    /// The value.
    pub value: Option<f64>,
    /// Its unit.
    pub unit: String,
    /// Why there is none.
    pub why: Option<String>,
}

/// The selected county: every lens's value and its recorded inputs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CountyVm {
    /// Its key.
    pub key: String,
    /// Its name.
    pub name: String,
    /// Its country.
    pub country: String,
    /// Whether the tape has it.
    pub in_tape: bool,
    /// Each lens's value, in the table's order.
    pub lenses: Vec<CountyLensVm>,
    /// Its recorded numbers at the report tick, each with its series.
    pub inputs: Vec<InputVm>,
}

/// The map pane.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapVm {
    /// Why the tape has no map, when it has none.
    pub refused: Option<String>,
    /// The lens shown.
    pub lens: Option<LensVm>,
    /// Every lens, for the selector.
    pub lenses: Vec<LensItemVm>,
    /// The selected county, when a node of the map is selected.
    pub county: Option<CountyVm>,
}

/// The country as a word.
fn country(c: Country) -> &'static str {
    match c {
        Country::England => "England",
        Country::Wales => "Wales",
        Country::Scotland => "Scotland",
        Country::NorthernIreland => "Northern Ireland",
    }
}

/// Whether the run's tape has a map: every node is a region of the atlas, and there is one at
/// least. Otherwise, why not.
pub fn geography(w: &World, atlas: &Atlas) -> Result<(), String> {
    let nodes: Vec<&str> = w.nodes.iter().map(|n| n.key.as_str()).collect();
    let foreign: Vec<&str> = nodes
        .iter()
        .copied()
        .filter(|k| atlas.index(k).is_none())
        .collect();
    if foreign.len() == nodes.len() {
        let some: Vec<&str> = nodes.iter().copied().take(4).collect();
        return Err(format!(
            "no map: this tape's nodes ({}{}) are not regions of the atlas; a map needs \
             nodes keyed county.<chapman> (docs/GUI.md §6, D7)",
            some.join(", "),
            if nodes.len() > 4 { ", …" } else { "" }
        ));
    }
    if !foreign.is_empty() {
        let some: Vec<&str> = foreign.iter().copied().take(6).collect();
        return Err(format!(
            "the map is refused: {} of this tape's nodes are not regions of the atlas ({}{}); \
             an atlas whose keys differ from the tape's node keys is refused (docs/GUI.md §6)",
            foreign.len(),
            some.join(", "),
            if foreign.len() > 6 { ", …" } else { "" }
        ));
    }
    Ok(())
}

/// A lens's scale, as a word.
fn scale_name(s: Scale) -> &'static str {
    match s {
        Scale::Sequential => "sequential",
        Scale::SequentialLog => "sequential, log",
        Scale::Diverging => "diverging",
    }
}

/// Where a value sits on a lens's scale, from 0 to 1, clamped to the domain, and whether it
/// lies outside. A display transform (U6): a ratio, or a ratio of logs through `core::num`.
pub fn position(lens: &Lens, v: f64) -> (f64, Beyond) {
    let (lo, hi) = lens.domain;
    let beyond = if v < lo {
        Beyond::Below
    } else if v > hi {
        Beyond::Above
    } else {
        Beyond::Within
    };
    let x = v.clamp(lo, hi);
    let at = match lens.scale {
        Scale::Sequential => (x - lo) / (hi - lo),
        Scale::SequentialLog => (num::ln(x) - num::ln(lo)) / (num::ln(hi) - num::ln(lo)),
        Scale::Diverging => {
            let c = wl::centre(lens).unwrap_or((lo + hi) / 2.0);
            if x <= c {
                0.5 * (x - lo) / (c - lo)
            } else {
                0.5 + 0.5 * (x - c) / (hi - c)
            }
        }
    };
    (at.clamp(0.0, 1.0), beyond)
}

/// A step of 1, 2 or 5 times a power of ten that puts between 3 and 7 marks across `span`.
fn nice_step(span: f64) -> f64 {
    let mut decade = 1.0;
    while span / decade > 10.0 {
        decade *= 10.0;
    }
    while span / decade < 1.0 {
        decade /= 10.0;
    }
    for m in [0.2, 0.5, 1.0, 2.0] {
        if span / (m * decade) <= 7.0 {
            return m * decade;
        }
    }
    2.0 * decade
}

/// The legend's marks: the domain's ends, the reference, and round values between, each at
/// its place on the scale. Fixed for the run, since the domain is.
pub fn legend(lens: &Lens) -> Vec<TickVm> {
    let (lo, hi) = lens.domain;
    let mut values = vec![lo];
    match lens.scale {
        Scale::Sequential => {
            let step = nice_step(hi - lo);
            let mut k = (lo / step).ceil();
            while k * step < hi {
                let v = k * step;
                if v - lo > 0.3 * step && hi - v > 0.3 * step {
                    values.push(v);
                }
                k += 1.0;
            }
        }
        Scale::SequentialLog => {
            let mut decade = 1.0;
            while decade > lo {
                decade /= 10.0;
            }
            while decade <= hi {
                for m in [1.0, 2.0, 5.0] {
                    let v = m * decade;
                    if v > lo * 1.15 && v < hi / 1.15 {
                        values.push(v);
                    }
                }
                decade *= 10.0;
            }
        }
        Scale::Diverging => {
            let c = wl::centre(lens).unwrap_or((lo + hi) / 2.0);
            values.extend([(lo + c) / 2.0, c, (c + hi) / 2.0]);
        }
    }
    values.push(hi);
    values
        .into_iter()
        .map(|value| TickVm {
            value,
            at: position(lens, value).0,
        })
        .collect()
}

/// The selector's group of a lens.
fn group(key: &str) -> &'static str {
    match key {
        k if k.starts_with("since.") => "Change since the record's first tick",
        k if k.starts_with("param.") => "The history's levers",
        k if k.starts_with("gap.") => "Against the oracle",
        "wage.baskets" | "wage.goods" | "wage.land" => "Wages",
        "rent.goods" | "price.good" | "price.mach" => "Prices",
        "share.land" | "share.labour" => "Income",
        "frontier.x" | "participation" | "output.per.head" => "Production",
        "relief.burden" | "shortfall" => "Relief",
        _ => "Markets",
    }
}

/// Why a lens cannot be shown at all, when it cannot: the oracle lenses wait for
/// `crates/observe` (docs/GUI.md §7.3).
fn unavailable(lens: &Lens) -> Option<String> {
    (lens.source == LensSource::Observe).then(|| {
        "waits for crates/observe's oracle_gap (docs/GUI.md §7.3, docs/demo/WORLD.md §6): the \
         county's oracle point at its params in force, never fed back (R13)"
            .to_string()
    })
}

/// Every lens, for the selector.
pub fn items(lenses: &[Lens]) -> Vec<LensItemVm> {
    lenses
        .iter()
        .map(|l| LensItemVm {
            key: l.key.clone(),
            name: l.name.clone(),
            unit: l.unit.clone(),
            group: group(&l.key).to_string(),
            unavailable: unavailable(l),
        })
        .collect()
}

fn series(measure: Measure, at: At) -> SeriesKey {
    SeriesKey { measure, at }
}

fn key(s: &str) -> Option<Key> {
    Key::new(s).ok()
}

fn market(node: &str, good: &str) -> Option<At> {
    Some(At::Market {
        node: key(node)?,
        good: key(good)?,
    })
}

fn at(store: &Store, k: &SeriesKey, tick: u64) -> Option<f64> {
    store.series(k)?.at(tick)
}

/// N per tick from the workers param's value: the conversion its first use takes, as the
/// registry records it (`ClockMethod::per_tick`, the engine's function).
fn per_tick(w: &World, param: &str, v: f64) -> Option<f64> {
    let d = w.registry.get(w.id_of::<ParamId>(param)?)?;
    d.sites.first()?.method.per_tick(&w.clock, v).ok()
}

/// A county's recorded numbers at report tick `tick` (docs/demo/WORLD.md §6's inputs): each
/// is `None` where the record has none.
pub fn readings(store: &Store, node: &str, tick: u64) -> Readings {
    readings_for(store, node, tick, None)
}

/// As [`readings`], for the one level `only` when it is given: the rationing lines and the
/// trailing window, which cost the most to read, are read only for the lenses that use them.
fn readings_for(store: &Store, node: &str, tick: u64, only: Option<wl::Level>) -> Readings {
    let wants = |l: wl::Level| only.is_none_or(|o| o == l);
    let k = CountyKeys::of(node);
    let mut x = Readings::default();
    let Some(w) = store.world() else {
        return x;
    };
    for (i, good) in MARKETS.iter().enumerate() {
        if let Some(m) = market(node, good) {
            x.prices[i] = at(store, &series(Measure::Price, m.clone()), tick);
            x.cleared[i] = at(store, &series(Measure::Cleared, m), tick);
        }
    }
    for p in Param::ALL {
        if let Some(pk) = key(k.param(p)) {
            x.params[p.index()] = at(store, &series(Measure::Param, At::Param(pk)), tick);
        }
    }
    x.n_tick =
        x.params[Param::Workers.index()].and_then(|v| per_tick(w, k.param(Param::Workers), v));
    let state = |actor: &str, f: StateField| {
        key(actor).and_then(|a| at(store, &series(Measure::State(f), At::Actor(a)), tick))
    };
    x.share = state(&k.desk_good, StateField::Share);
    x.due = state(&k.provider, StateField::Due);
    x.paid = state(&k.provider, StateField::Paid);
    // The tick's rationing lines at the node: every class line the record holds there.
    if let Some(n) = key(node).filter(|_| wants(wl::Level::Rationing)) {
        for good in MARKETS {
            let Some(g) = key(good) else { continue };
            for c in &w.classes {
                for side in [SideTag::Buy, SideTag::Sell] {
                    let line = At::Class {
                        node: n.clone(),
                        good: g.clone(),
                        class: c.clone(),
                        side,
                    };
                    let req = at(store, &series(Measure::Requested, line.clone()), tick);
                    let fill = at(store, &series(Measure::Filled, line), tick);
                    if let (Some(r), Some(f)) = (req, fill) {
                        x.rationing.push((r, f));
                    }
                }
            }
        }
    }
    // Whether all four markets traded, in each report tick of the trailing window the record
    // holds.
    if wants(wl::Level::NoTrade) {
        let lo = tick
            .saturating_add(1)
            .saturating_sub(NO_TRADE_WINDOW)
            .max(store.reports().start);
        let flags: Option<Vec<&crate::run::Series>> = MARKETS
            .iter()
            .map(|g| market(node, g).and_then(|m| store.series(&series(Measure::Trades, m))))
            .collect();
        if let Some(flags) = flags {
            for t in lo..=tick {
                let all: Option<Vec<f64>> = flags.iter().map(|s| s.at(t)).collect();
                if let Some(v) = all {
                    x.traded.push(v.iter().all(|f| *f > 0.0));
                }
            }
        }
    }
    x
}

/// The value of `lens` for a county, from its readings at the report tick and at the record's
/// first report tick, or why there is none.
fn value_of(lens: &Lens, now: &Readings, first: Option<&Readings>) -> Result<f64, String> {
    if let Some(why) = unavailable(lens) {
        return Err(why);
    }
    let m = wl::Measure::of(&lens.key)
        .ok_or_else(|| format!("{} names no measure lens.rs defines", lens.key))?;
    wl::value(m, now, first).map_err(|e| e.to_string())
}

/// The readings at the record's first report tick, for a change lens: from genesis only, so a
/// change is a change since 1750, as the lens's name says.
fn first_readings(store: &Store, node: &str, only: Option<wl::Level>) -> Option<Readings> {
    let r = store.reports();
    (r.start == 0 && !r.is_empty()).then(|| readings_for(store, node, 0, only))
}

fn identity(store: &Store, origin: Origin) -> Option<IdentityVm> {
    store.run().zip(store.world()).map(|(k, w)| IdentityVm {
        name: w.name.clone(),
        commit: k.build.commit.clone(),
        dirty: k.build.dirty,
        world_id: k.world_id.to_string(),
        tape_hash: k.tape_hash.to_string(),
        origin,
    })
}

/// One lens of a run whose tape has a map, at report tick `cursor` (`None`: live).
pub fn lens(
    store: &Store,
    origin: Origin,
    atlas: &Atlas,
    lens: &Lens,
    cursor: Option<u64>,
) -> Option<LensVm> {
    let w = store.world()?;
    let tick = report_tick(store, cursor);
    let measure = wl::Measure::of(&lens.key);
    let needs_first = matches!(measure, Some(wl::Measure::Since(_)));
    let level = measure.map(wl::Measure::level);
    let mut regions = Vec::with_capacity(atlas.regions.len());
    let mut counts = CountsVm {
        regions: atlas.regions.len(),
        ..CountsVm::default()
    };
    for r in &atlas.regions {
        let in_tape = w.id_of::<NodeId>(r.key.as_str()).is_some();
        let result = match (in_tape, tick) {
            (false, _) => Err("not a node of this tape".to_string()),
            (true, None) => Err("no tick has run".to_string()),
            (true, Some(t)) => {
                let now = readings_for(store, &r.key, t, level);
                let first = if needs_first {
                    first_readings(store, &r.key, level)
                } else {
                    None
                };
                value_of(lens, &now, first.as_ref())
            }
        };
        counts.in_tape += usize::from(in_tape);
        let (value, at, beyond, why) = match result {
            Ok(v) => {
                let (at, beyond) = position(lens, v);
                counts.valued += 1;
                counts.below += usize::from(beyond == Beyond::Below);
                counts.above += usize::from(beyond == Beyond::Above);
                (Some(v), Some(at), beyond, None)
            }
            Err(why) => (None, None, Beyond::Within, Some(why)),
        };
        regions.push(RegionVm {
            key: r.key.clone(),
            name: r.name.clone(),
            chapman: r.chapman.clone(),
            country: country(r.country).to_string(),
            in_tape,
            value,
            at,
            beyond,
            why,
            rank: None,
        });
    }
    let mut ranked: Vec<usize> = (0..regions.len())
        .filter(|&i| regions[i].value.is_some())
        .collect();
    ranked.sort_by(|&a, &b| {
        let (va, vb) = (regions[a].value, regions[b].value);
        vb.partial_cmp(&va)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| regions[a].key.cmp(&regions[b].key))
    });
    for (n, &i) in ranked.iter().enumerate() {
        regions[i].rank = Some(n + 1);
    }
    let reference = wl::reference_value(&lens.reference).map(|value| ReferenceVm {
        value,
        text: lens.reference.clone(),
        at: (lens.domain.0..=lens.domain.1)
            .contains(&value)
            .then(|| position(lens, value).0),
    });
    Some(LensVm {
        key: lens.key.clone(),
        name: lens.name.clone(),
        measure: lens.measure.clone(),
        unit: lens.unit.clone(),
        scale: scale_name(lens.scale).to_string(),
        domain: lens.domain,
        centre: wl::centre(lens),
        reference,
        inputs: lens.inputs.clone(),
        note: lens.note.clone(),
        unavailable: unavailable(lens),
        legend: legend(lens),
        run: identity(store, origin),
        tick,
        date: tick.map_or_else(String::new, |t| date(w, t)),
        regions,
        ranked,
        counts,
    })
}

/// The selected county's card: every lens's value, and its recorded numbers with their series.
fn county(
    store: &Store,
    atlas: &Atlas,
    lenses: &[Lens],
    node: &Key,
    cursor: Option<u64>,
) -> Option<CountyVm> {
    let w = store.world()?;
    let r = atlas.region(node.as_str())?;
    let in_tape = w.id_of::<NodeId>(node.as_str()).is_some();
    let tick = report_tick(store, cursor);
    let now = tick.map(|t| readings(store, node.as_str(), t));
    let first = first_readings(store, node.as_str(), None);
    let lenses = lenses
        .iter()
        .map(|l| {
            let v = match (&now, in_tape) {
                (_, false) => Err("not a node of this tape".to_string()),
                (None, true) => Err("no tick has run".to_string()),
                (Some(now), true) => value_of(l, now, first.as_ref()),
            };
            CountyLensVm {
                key: l.key.clone(),
                name: l.name.clone(),
                unit: l.unit.clone(),
                value: v.as_ref().ok().copied(),
                why: v.err(),
            }
        })
        .collect();
    let k = CountyKeys::of(node.as_str());
    let mut inputs = Vec::new();
    let mut add = |label: String, s: SeriesKey| {
        inputs.push(InputVm {
            label,
            value: tick.and_then(|t| at(store, &s, t)),
            unit: super::unit_of(w, &s),
            series: s,
        });
    };
    if in_tape {
        for good in MARKETS {
            if let Some(m) = market(node.as_str(), good) {
                add(
                    format!("price of {good}"),
                    series(Measure::Price, m.clone()),
                );
                add(format!("{good} cleared"), series(Measure::Cleared, m));
            }
        }
        for p in wl::PARAMS_READ {
            if let Some(pk) = key(k.param(p)) {
                add(format!("param {pk}"), series(Measure::Param, At::Param(pk)));
            }
        }
        if let Some(a) = key(&k.desk_good) {
            add(
                format!("{a}'s share (1 − x)"),
                series(Measure::State(StateField::Share), At::Actor(a)),
            );
        }
        for f in [StateField::Due, StateField::Paid] {
            if let Some(a) = key(&k.provider) {
                add(
                    format!("{a}'s {}", f.name()),
                    series(Measure::State(f), At::Actor(a)),
                );
            }
        }
    }
    Some(CountyVm {
        key: r.key.clone(),
        name: r.name.clone(),
        country: country(r.country).to_string(),
        in_tape,
        lenses,
        inputs,
    })
}

/// The map pane of a run: the lens `current` (or the first the table lists), the selector and
/// the selected county, at report tick `cursor` (`None`: live).
pub fn build(
    store: &Store,
    origin: Origin,
    atlas: &Atlas,
    lenses: &[Lens],
    current: &str,
    selection: Option<&Entity>,
    cursor: Option<u64>,
) -> Option<MapVm> {
    let w = store.world()?;
    let items = items(lenses);
    if let Err(why) = geography(w, atlas) {
        return Some(MapVm {
            refused: Some(why),
            lens: None,
            lenses: items,
            county: None,
        });
    }
    let shown = lenses
        .iter()
        .find(|l| l.key == current)
        .or_else(|| lenses.first());
    let county = match selection {
        Some(Entity::Node(n)) => county(store, atlas, lenses, n, cursor),
        _ => None,
    };
    Some(MapVm {
        refused: None,
        lens: shown.and_then(|l| lens(store, origin, atlas, l, cursor)),
        lenses: items,
        county,
    })
}
