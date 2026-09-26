//! The folds every verdict and report reads through (docs/CERTIFY.md §6, N12): a maximum, a
//! minimum, a sum and a log range that keep a NaN rather than drop it, and the window arithmetic
//! of §5. Every fold is a left fold in the series' order (ENGINE §9), and every log is
//! `core::num::ln` (A5).
//!
//! July's statistics filtered NaN one sample at a time and folded maxima with `f64::max`, which
//! returns the other operand when one is NaN (`v2p3: certify/verdict.rs:104-106`), so one bad
//! sample vanished instead of failing the verdict. Here a NaN anywhere in a series is the fold's
//! result, and an infinity stays an infinity, so the finite scan of §8 sees either.

use rustyecon_core::num;

/// The larger of two values, or the NaN if either is one.
pub fn max2(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() || b > a {
        b
    } else {
        a
    }
}

/// The smaller of two values, or the NaN if either is one.
pub fn min2(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() || b < a {
        b
    } else {
        a
    }
}

/// The largest value of a series, NaN if any value is NaN; `None` for an empty series.
pub fn max_nan(xs: impl IntoIterator<Item = f64>) -> Option<f64> {
    xs.into_iter().fold(None, |acc, x| {
        Some(match acc {
            None => x,
            Some(m) => max2(m, x),
        })
    })
}

/// The smallest value of a series, NaN if any value is NaN; `None` for an empty series.
pub fn min_nan(xs: impl IntoIterator<Item = f64>) -> Option<f64> {
    xs.into_iter().fold(None, |acc, x| {
        Some(match acc {
            None => x,
            Some(m) => min2(m, x),
        })
    })
}

/// The sum of a series, a left fold from 0: a NaN or an infinity anywhere stays in it.
pub fn sum(xs: impl IntoIterator<Item = f64>) -> f64 {
    xs.into_iter().fold(0.0, |acc, x| acc + x)
}

/// The largest value less the smallest: NaN if any value is NaN, and infinite if any is
/// infinite; `None` for an empty series.
pub fn range(xs: impl IntoIterator<Item = f64> + Clone) -> Option<f64> {
    let hi = max_nan(xs.clone())?;
    let lo = min_nan(xs)?;
    Some(hi - lo)
}

/// The range of `ln x` over a series: the width, in log, of the band the series moved in. A
/// zero value makes it infinite and a negative or NaN one NaN, so neither reads as at rest.
/// `None` for an empty series. The probe's "at rest in F" is this over its observables, and
/// certify's Settles is this over nominal price and cleared volume (§11).
pub fn ln_range(xs: impl IntoIterator<Item = f64>) -> Option<f64> {
    let logs: Vec<f64> = xs.into_iter().map(num::ln).collect();
    range(logs.iter().copied())
}

/// The first minimum of a series of (tick, value) and its tick, NaN (at the NaN's tick) if any
/// value is NaN; `None` for an empty series. The probe passes each market's cleared volume over
/// its oracle volume, and certify's reports the cleared volume itself (§6, §11).
pub fn trough(series: impl IntoIterator<Item = (u64, f64)>) -> Option<(f64, u64)> {
    let mut out: Option<(f64, u64)> = None;
    for (t, x) in series {
        out = match out {
            None => Some((x, t)),
            Some((m, at)) if m.is_nan() => Some((m, at)),
            Some(_) if x.is_nan() => Some((x, t)),
            Some((m, _)) if x < m => Some((x, t)),
            keep => keep,
        };
    }
    out
}

/// The first tick of a window that starts a share `share` of the way through `[from, from +
/// n)`: `from + ⌈share·n⌉`, the product and the ceiling in f64 (§5).
pub fn window_start(from: u64, n: u64, share: f64) -> u64 {
    let offset = (share * n as f64).ceil();
    // A share in [0, 1] of a tick count fits; the cast saturates otherwise.
    from.saturating_add(offset as u64)
}

/// `⌊share·n⌋`, in f64: a resume tick, or the dead ticks a window may hold (§4, §6).
pub fn floor_share(share: f64, n: u64) -> u64 {
    (share * n as f64).floor() as u64
}

/// `⌈share·n⌉`, in f64: the kick's tail (§7).
pub fn ceil_share(share: f64, n: u64) -> u64 {
    (share * n as f64).ceil() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_keep_a_nan() {
        // N12: f64::max drops a NaN, so fold(0.0, f64::max) over [1, NaN, 2] gives 2; these
        // give NaN wherever the NaN sits.
        for xs in [
            vec![f64::NAN, 1.0, 2.0],
            vec![1.0, f64::NAN, 2.0],
            vec![1.0, 2.0, f64::NAN],
        ] {
            assert!(max_nan(xs.clone()).unwrap().is_nan(), "{xs:?}");
            assert!(min_nan(xs.clone()).unwrap().is_nan(), "{xs:?}");
            assert!(sum(xs.clone()).is_nan(), "{xs:?}");
            assert!(range(xs.clone()).unwrap().is_nan(), "{xs:?}");
            assert!(ln_range(xs.clone()).unwrap().is_nan(), "{xs:?}");
            let ticked: Vec<(u64, f64)> =
                xs.iter().copied().zip(0..).map(|(x, t)| (t, x)).collect();
            assert!(trough(ticked).unwrap().0.is_nan(), "{xs:?}");
        }
        assert_eq!(xs_fold(), 2.0, "the std fold this module replaces drops it");
        assert_eq!(max_nan([1.0, f64::INFINITY]), Some(f64::INFINITY));
        assert_eq!(min_nan([1.0, f64::NEG_INFINITY]), Some(f64::NEG_INFINITY));
        assert_eq!(ln_range([1.0, 0.0]), Some(f64::INFINITY));
        assert!(ln_range([1.0, -1.0]).unwrap().is_nan());
        assert_eq!(max_nan(Vec::new()), None);
        assert_eq!(range([2.0, 5.0, 3.0]), Some(3.0));
        assert_eq!(
            trough([(4, 3.0), (5, 1.0), (6, 1.0), (7, 2.0)]),
            Some((1.0, 5))
        );
        assert_eq!(window_start(100, 1000, 0.9), 1000);
        assert_eq!(window_start(0, 7, 0.5), 4);
        assert_eq!(floor_share(0.5, 7), 3);
        assert_eq!(ceil_share(0.1, 21), 3);
    }

    fn xs_fold() -> f64 {
        [1.0, f64::NAN, 2.0].iter().copied().fold(0.0, f64::max)
    }
}
