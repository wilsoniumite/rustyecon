//! A worker's pace: how many ticks the next slice may run. A slice aims at about 8 ms, from the
//! time the last slice took per tick, so the worker reads its commands that often. Under a speed
//! cap of `c` ticks per second, the ticks run since the cap began never exceed `c` times the
//! seconds elapsed. The pace decides when the Runner steps, never what a step computes.

use std::time::{Duration, Instant};

/// The time a slice aims at.
const SLICE: Duration = Duration::from_millis(8);
/// The most ticks one slice may run.
const MAX_SLICE: u32 = 100_000;
/// Nanoseconds in a second.
const NANOS: u128 = 1_000_000_000;

#[derive(Debug, Default)]
pub(super) struct Pace {
    cap: Option<u32>,
    since: Option<Instant>,
    done: u64,
    per_tick: Option<Duration>,
}

impl Pace {
    /// A run or a step begins: the cap, in ticks per second, counts from `now`.
    pub(super) fn start(&mut self, cap: Option<u32>, now: Instant) {
        self.cap = cap.filter(|&c| c > 0);
        self.since = Some(now);
        self.done = 0;
    }

    /// The ticks the next slice may run: about 8 ms of them, and no more than the cap allows by
    /// `now`. Zero means the cap is ahead; wait for [`Pace::wait`].
    pub(super) fn budget(&self, now: Instant) -> u32 {
        let slice = match self.per_tick {
            None => 1,
            Some(d) if d.is_zero() => MAX_SLICE,
            Some(d) => u32::try_from(SLICE.as_nanos() / d.as_nanos())
                .unwrap_or(MAX_SLICE)
                .clamp(1, MAX_SLICE),
        };
        match (self.cap, self.since) {
            (Some(cap), Some(since)) => {
                let allowed = now.duration_since(since).as_nanos() * u128::from(cap) / NANOS;
                let left = allowed.saturating_sub(u128::from(self.done));
                slice.min(u32::try_from(left).unwrap_or(u32::MAX))
            }
            _ => slice,
        }
    }

    /// How long to wait, from `now`, before the cap allows the next tick; a slice at most.
    pub(super) fn wait(&self, now: Instant) -> Duration {
        let (Some(cap), Some(since)) = (self.cap, self.since) else {
            return Duration::ZERO;
        };
        let due_nanos = u128::from(self.done + 1) * NANOS / u128::from(cap);
        let due = since + Duration::from_nanos(u64::try_from(due_nanos).unwrap_or(u64::MAX));
        due.saturating_duration_since(now).min(SLICE)
    }

    /// A slice ran `ran` ticks in `took`.
    pub(super) fn record(&mut self, ran: u32, took: Duration) {
        if ran > 0 {
            self.per_tick = Some(took / ran);
            self.done += u64::from(ran);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cap_holds_the_rate_and_slices_aim_at_eight_ms() {
        let t0 = Instant::now();
        let mut p = Pace::default();
        p.start(None, t0);
        assert_eq!(p.budget(t0), 1, "the first slice runs one tick");
        p.record(1, Duration::from_micros(80));
        assert_eq!(p.budget(t0), 100, "8 ms of 80 µs ticks");
        // 1,000 ticks a second: none at once, one after a millisecond, ten after ten.
        p.start(Some(1_000), t0);
        assert_eq!(p.budget(t0), 0);
        assert_eq!(p.wait(t0), Duration::from_millis(1));
        assert_eq!(p.budget(t0 + Duration::from_millis(1)), 1);
        assert_eq!(p.budget(t0 + Duration::from_millis(10)), 10);
        p.record(10, Duration::from_micros(800));
        assert_eq!(p.budget(t0 + Duration::from_millis(10)), 0);
        assert_eq!(
            p.budget(t0 + Duration::from_secs(1)),
            100,
            "the slice bounds it"
        );
        // No cap, no wait.
        p.start(None, t0);
        assert_eq!(p.wait(t0), Duration::ZERO);
    }
}
