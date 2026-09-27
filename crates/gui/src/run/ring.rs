//! The ring's ticks (docs/GUI.md §3.3, D4): the Runner checkpoints the state at the tick each
//! model year's 1 January falls in. G3 thins them to decades beyond its memory budget.

use rustyecon_engine::prelude::*;

/// The first ring tick at or after `tick`: the tick `clock.tick_of(Y-01-01)` of the first year Y
/// whose 1 January falls in `tick` or later. `None` when no such date fits the calendar.
pub fn ring_tick_at_or_after(clock: &Clock, tick: u64) -> Option<u64> {
    let year = clock.date_of(tick)?.y;
    // The 1 January of the tick's own year falls in `tick` or before it, and the next year's
    // falls after it; a year whose 1 January is before the start is skipped.
    for y in [year, year.checked_add(1)?] {
        let Ok(jan1) = Date::new(y, 1, 1) else {
            continue;
        };
        if let Ok(t) = clock.tick_of(jan1) {
            if t >= tick {
                return Some(t);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(start: &str, tpy: u32) -> Clock {
        Clock {
            start: Date::parse(start).expect("a date"),
            ticks_per_year: tpy,
        }
    }

    #[test]
    fn ring_ticks_are_each_years_first_of_january() {
        // The gate's calendar: 1750-01-01 at 52 ticks a year. 1751-01-01 falls in tick 51, which
        // begins on 1750-12-26 (the clock counts the mean Gregorian year), and 1790-01-01 in
        // tick 2,080.
        let c = clock("1750-01-01", 52);
        assert_eq!(ring_tick_at_or_after(&c, 0), Some(0));
        assert_eq!(ring_tick_at_or_after(&c, 1), Some(51));
        assert_eq!(ring_tick_at_or_after(&c, 51), Some(51));
        assert_eq!(ring_tick_at_or_after(&c, 52), Some(103));
        assert_eq!(ring_tick_at_or_after(&c, 2_070), Some(2_080));
        let mut t = 0;
        for y in 1750..1790 {
            let r = ring_tick_at_or_after(&c, t).expect("a ring tick");
            let jan1 = Date::new(y, 1, 1).unwrap();
            assert_eq!(r, c.tick_of(jan1).unwrap(), "{y}");
            assert!(c.date_of(r).unwrap() <= jan1 && jan1 < c.date_of(r + 1).unwrap());
            t = r + 1;
        }
        // A start after 1 January skips that year's.
        let c = clock("1750-07-01", 12);
        assert_eq!(ring_tick_at_or_after(&c, 0), Some(6));
        let c = clock("1750-01-01", 1);
        assert_eq!(ring_tick_at_or_after(&c, 3), Some(3));
    }
}
