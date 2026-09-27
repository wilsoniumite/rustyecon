//! Frame timings for the smoke mode (docs/GUI.md §8.1): the CPU each frame took, as eframe
//! measures it (`IntegrationInfo::cpu_usage`: the app's frame and its painting, without the
//! wait for vsync), summarised as p50, p90 and max. The frame rate is capped by vsync, so CPU
//! per frame is the measure.

use serde::Serialize;

/// Every frame's CPU time, in seconds.
#[derive(Debug, Clone, Default)]
pub struct Frames {
    cpu: Vec<f64>,
}

/// What the frames took.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FrameSummary {
    /// How many frames.
    pub frames: usize,
    /// The median, in milliseconds.
    pub p50_ms: f64,
    /// The 90th percentile, in milliseconds.
    pub p90_ms: f64,
    /// The largest, in milliseconds.
    pub max_ms: f64,
}

impl Frames {
    /// One frame's CPU time, in seconds. A time that is not finite is not a time, and is
    /// dropped.
    pub fn record(&mut self, seconds: f64) {
        if seconds.is_finite() && seconds >= 0.0 {
            self.cpu.push(seconds);
        }
    }

    /// The number of frames recorded.
    pub fn len(&self) -> usize {
        self.cpu.len()
    }

    /// Whether none was recorded.
    pub fn is_empty(&self) -> bool {
        self.cpu.is_empty()
    }

    /// p50, p90 and max by nearest rank; `None` with no frame.
    pub fn summary(&self) -> Option<FrameSummary> {
        let mut s: Vec<f64> = self.cpu.iter().map(|&c| c * 1e3).collect();
        s.sort_by(f64::total_cmp);
        let n = s.len();
        let rank = |p: usize| s[(n * p).div_ceil(100).clamp(1, n) - 1];
        (n > 0).then(|| FrameSummary {
            frames: n,
            p50_ms: rank(50),
            p90_ms: rank(90),
            max_ms: s[n - 1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_are_nearest_rank() {
        let mut f = Frames::default();
        assert!(f.summary().is_none());
        for ms in [5, 1, 4, 2, 3, 10, 9, 8, 7, 6] {
            f.record(f64::from(ms) / 1e3);
        }
        f.record(f64::NAN);
        let s = f.summary().unwrap();
        assert_eq!(s.frames, 10);
        assert!((s.p50_ms - 5.0).abs() < 1e-3, "{s:?}");
        assert!((s.p90_ms - 9.0).abs() < 1e-3, "{s:?}");
        assert!((s.max_ms - 10.0).abs() < 1e-3, "{s:?}");
    }
}
