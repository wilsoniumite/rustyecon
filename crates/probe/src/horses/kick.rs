//! The kick check inside the harness (HORSES-SPEC §7.5; MARKETS-SPEC §7.5, MG7): a run's
//! CONVERGED stands only if its target's kick set decays; and the engine's slowest local mode,
//! from the base kick set, for the run length's third term (§7.4).
//!
//! A kick set is certify's, as the markets probe runs it ([`crate::markets::kick`]): at the end
//! state of a run, the base continuation and one run per market and sign with that market's
//! posted price × (1 ± 1e-9) through `ScalePrice`, each for H ticks, judged by certify's own
//! verdict with `criteria/appb-2026-09-26.ron`'s bars (`gain_tail` ≤ 1e-3 over the last tenth,
//! `gain_peak` ≤ 1e6).

use super::perturb::Perturbation;
use super::setup::{tape_ron, Setup};
use crate::markets::kick::{Kick, KickSet, KICK_MAX_GAIN, KICK_MAX_PEAK, KICK_SIZE, KICK_TAIL};
use certify::battery::{kick, kick_gain, KickBars, KickSegment};
use certify::fold::ceil_share;
use certify::kick::kick_segment;
use certify::Names;
use rustyecon_core::num;
use rustyecon_engine::prelude::{Sim, Tape};

/// The kick set at the end of the run `name` of `base`, scored for `ticks` ticks (and the ticks
/// before a dated shock), with horizon `horizon`; and the kicks' largest gain g(t)/g(T) at each
/// tick of the horizon, the envelope the slowest mode is read from.
pub fn kick_set(
    base: &Setup,
    name: &str,
    ticks: u64,
    horizon: u64,
) -> Result<(KickSet, Vec<f64>), String> {
    let pert = Perturbation::parse(name)?;
    let mut setup = base.clone();
    pert.apply(&mut setup, ticks)?;
    let text = tape_ron(&setup)?;
    kick_set_of(&text, pert.clock_start(ticks) + ticks, horizon, name)
}

/// The kick set at tick `until` of the tape `text`, with horizon `horizon`, named `name`; and
/// the kicks' envelope, as [`kick_set`] reads them. The loop step's harness shares it.
pub fn kick_set_of(
    text: &str,
    until: u64,
    horizon: u64,
    name: &str,
) -> Result<(KickSet, Vec<f64>), String> {
    let tape = Tape::from_ron(text).map_err(|e| format!("the tape does not load: {e}"))?;
    let mut sim = Sim::new(&tape).map_err(|e| format!("the tape does not load: {e}"))?;
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
    let seg: KickSegment = kick_segment(&tape, world, &cp, 0, &bars, "HORSES-SPEC §7.5");
    let names = Names::of(world);
    let verdict = kick(std::slice::from_ref(&seg), &bars, &names);
    let mut kicks = Vec::new();
    let mut envelope: Vec<f64> = Vec::new();
    if let Ok((base_prices, runs)) = &seg.runs {
        envelope = vec![0.0; base_prices.len()];
        for r in runs {
            let market = names.market(r.market);
            let (size, gain_tail, gain_peak, error) = match &r.prices {
                Ok(p) => {
                    let g = gains(base_prices, p);
                    if let Some(g) = g {
                        for (e, x) in envelope.iter_mut().zip(g) {
                            *e = if x.is_nan() || x > *e { x } else { *e };
                        }
                    }
                    match kick_gain(base_prices, p, bars.tail) {
                        Some(g) => (g.size, g.gain_tail, g.gain_peak, None),
                        None => (f64::NAN, f64::NAN, f64::NAN, Some("no gain".to_string())),
                    }
                }
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
    Ok((
        KickSet {
            name: name.to_string(),
            at: seg.at,
            horizon,
            kicks,
            pass: verdict.pass,
            notes,
        },
        envelope,
    ))
}

/// g(t)/g(T) of one kick: the largest |ln(p_kick/p_base)| over markets at each tick over its
/// value on the kicked tick, as certify's `kick_gain` reads it.
fn gains(base: &[Vec<f64>], kicked: &[Vec<f64>]) -> Option<Vec<f64>> {
    if base.is_empty() || base.len() != kicked.len() {
        return None;
    }
    let g: Vec<f64> = base
        .iter()
        .zip(kicked)
        .map(|(b, k)| {
            b.iter()
                .zip(k)
                .map(|(pb, pk)| num::ln(pk / pb).abs())
                .fold(0.0, |a: f64, x| if x.is_nan() || x > a { x } else { a })
        })
        .collect();
    let size = g[0];
    Some(g.iter().map(|x| x / size).collect())
}

/// The kick envelope's decay (HORSES-SPEC §7.4; its reading registered in
/// docs/probe/HORSES-RULES.md §6): g, the per-tick factor of the least-squares line of ln G(t)
/// over the ticks from the envelope's peak in the first half of the horizon to the first tick
/// at which it falls below [`FLOOR_MARGIN`] times its rounding floor, the median of its last
/// tenth. A 1e-9 kick at a point whose slowest mode decays by 1 − ε a tick freezes about 1e-16/ε
/// from the base run, some 1e-5 of the kick at δ 10%, so the fit reads the whole descent the kick
/// can show, and its oscillations average out; the frame's "second half" of a 3·T6 horizon lies
/// on that floor. `None` when the window holds fewer than 20 ticks.
pub fn slowest_mode(envelope: &[f64]) -> Option<f64> {
    let n = envelope.len();
    if n < 40 {
        return None;
    }
    let mut tail: Vec<f64> = envelope[n - n / 10..].to_vec();
    tail.sort_by(f64::total_cmp);
    let floor = tail[tail.len() / 2];
    let peak = (0..n / 2).max_by(|&a, &b| envelope[a].total_cmp(&envelope[b]))?;
    let window: Vec<(f64, f64)> = envelope[peak..]
        .iter()
        .enumerate()
        .take_while(|(_, &g)| g >= FLOOR_MARGIN * floor)
        .map(|(t, &g)| (t as f64, num::ln(g)))
        .collect();
    if window.len() < 20 {
        return None;
    }
    let m = window.len() as f64;
    let (sx, sy) = window
        .iter()
        .fold((0.0, 0.0), |(a, b), (x, y)| (a + x, b + y));
    let (mx, my) = (sx / m, sy / m);
    let (sxy, sxx) = window.iter().fold((0.0, 0.0), |(a, b), (x, y)| {
        (a + (x - mx) * (y - my), b + (x - mx) * (x - mx))
    });
    Some(num::exp(sxy / sxx))
}

/// How far above its rounding floor the envelope must lie to be fitted.
pub const FLOOR_MARGIN: f64 = 10.0;

/// HORSES-SPEC §7.4's third term: 3·T6 rounded up to a thousand ticks, T6 = ln(10⁶)/(−ln g)
/// the ticks in which the slowest mode falls by 10⁶.
pub fn three_t6(g: f64) -> u64 {
    let t6 = num::ln(1e6) / -num::ln(g);
    let thousands = (3.0 * t6 / 1000.0).ceil();
    if thousands.is_finite() && thousands > 0.0 {
        (thousands as u64) * 1000
    } else {
        u64::MAX
    }
}
