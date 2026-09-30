//! The kick check inside the harness (MARKETS-SPEC §7.5, MG7; CERTIFY §7): a run's CONVERGED
//! stands only if its target's kick set decays.
//!
//! A kick set is certify's: at the end state of a run (the target's mode-A run, or its dated
//! cost-shock run), the base continuation and one run per market and sign with that market's
//! posted price × (1 ± 1e-9) through `ScalePrice`, each resumed from the run's checkpoint under
//! a tape that differs only by the kick (`certify::kick::kick_segment`), for H ticks. It passes
//! when every kick is realised, its `gain_tail` over the last tenth of H is at most 1e-3 and its
//! `gain_peak` at most 1e6 (`criteria/appb-2026-09-26.ron`'s bars), by certify's own verdict
//! (`certify::battery::kick`). A market posting 0 at the kick, a free good (P2.4; FREE-SPEC
//! §6.1), is not kicked, and a free-able market's equal prices read a gap of 0 (certify's
//! `kick_gain_free`).

use super::perturb::Perturbation;
use super::setup::{tape_ron, Setup};
use certify::battery::{kick, kick_gain_free, KickBars, KickSegment};
use certify::fold::ceil_share;
use certify::kick::kick_segment;
use certify::Names;
use rustyecon_engine::prelude::{Sim, Tape};

/// The kick's size, relative (REPORT §5–§6; MARKETS-SPEC §7.5).
pub const KICK_SIZE: f64 = 1e-9;
/// The share of H the tail reads.
pub const KICK_TAIL: f64 = 0.1;
/// The largest gain the tail may show.
pub const KICK_MAX_GAIN: f64 = 1e-3;
/// The largest gain the horizon may show.
pub const KICK_MAX_PEAK: f64 = 1e6;

/// One kick's readings.
#[derive(Debug, Clone, PartialEq)]
pub struct Kick {
    /// The market kicked, by its good's key.
    pub market: String,
    /// `+` or `-`.
    pub sign: &'static str,
    /// g(T), the realised kick.
    pub size: f64,
    /// The largest g(t)/g(T) over the tail.
    pub gain_tail: f64,
    /// The largest g(t)/g(T) over H.
    pub gain_peak: f64,
    /// Why it has no readings, if it has none.
    pub error: Option<String>,
}

/// A kick set: where it was kicked, for how long, each kick, and certify's verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct KickSet {
    /// The run whose end state was kicked.
    pub name: String,
    /// The tick the kicks fire in.
    pub at: u64,
    /// H.
    pub horizon: u64,
    /// Every kick.
    pub kicks: Vec<Kick>,
    /// Certify's verdict on the set.
    pub pass: bool,
    /// Certify's notes, when it fails.
    pub notes: Vec<String>,
}

/// The kick set at the end of the run `name` of `base`, scored for `ticks` ticks (and the ticks
/// before a dated shock), with horizon `horizon`.
pub fn kick_set(base: &Setup, name: &str, ticks: u64, horizon: u64) -> Result<KickSet, String> {
    let pert = Perturbation::parse(name)?;
    let mut setup = base.clone();
    pert.apply(&mut setup, ticks)?;
    let text = tape_ron(&setup)?;
    let tape = Tape::from_ron(&text).map_err(|e| format!("the tape does not load: {e}"))?;
    let mut sim = Sim::new(&tape).map_err(|e| format!("the tape does not load: {e}"))?;
    let until = pert.clock_start(ticks) + ticks;
    sim.run_until(until, &mut |_| {})
        .map_err(|e| format!("the base run failed before its end: {e}"))?;
    let cp = sim
        .checkpoint()
        .map_err(|e| format!("no checkpoint at the end: {e}"))?;
    let bars = KickBars {
        size: KICK_SIZE,
        horizon,
        tail: ceil_share(KICK_TAIL, horizon),
        max_gain: KICK_MAX_GAIN,
        max_peak: KICK_MAX_PEAK,
    };
    let world = sim.world();
    let seg: KickSegment = kick_segment(&tape, world, &cp, 0, &bars, "MARKETS-SPEC §7.5");
    let names = Names::of(world);
    let verdict = kick(std::slice::from_ref(&seg), &bars, &names);
    let mut kicks = Vec::new();
    if let Ok((base_prices, runs)) = &seg.runs {
        for r in runs {
            let market = names.market(r.market);
            let (size, gain_tail, gain_peak, error) = match &r.prices {
                Ok(p) => match kick_gain_free(base_prices, p, bars.tail, &seg.free) {
                    Some(g) => (g.size, g.gain_tail, g.gain_peak, None),
                    None => (f64::NAN, f64::NAN, f64::NAN, Some("no gain".to_string())),
                },
                Err(e) => (f64::NAN, f64::NAN, f64::NAN, Some(e.clone())),
            };
            kicks.push(Kick {
                market,
                sign: r.sign.mark(),
                size,
                gain_tail,
                gain_peak,
                error,
            });
        }
    }
    let mut notes = verdict.notes.clone();
    if let Err(e) = &seg.runs {
        notes.push(e.clone());
    }
    Ok(KickSet {
        name: name.to_string(),
        at: seg.at,
        horizon,
        kicks,
        pass: verdict.pass,
        notes,
    })
}
