//! Dates, ticks and the conversion of every time unit to its per-tick value (docs/ENGINE.md §6,
//! ADDENDUM A13).
//!
//! The tick length is registered on the tape as `ticks_per_year`, and this is the only place
//! that divides by it. Dates are proleptic Gregorian and the year is the mean Gregorian year,
//! 365.2425 days, held as the integer 3,652,425 ten-thousandths of a day so that the map from
//! dates to ticks uses no floating point.

use crate::error::CoreError;
use crate::ids::ParamId;
use crate::num;
use crate::registry::Params;
use crate::units::{CompoundPerYear, FlowPerYear, FractionPerYear, RatePerYear, Unit, Years};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Ten-thousandths of a day in the mean Gregorian year.
const YEAR_E4_DAYS: i128 = 3_652_425;
/// The scale of [`YEAR_E4_DAYS`].
const E4: i128 = 10_000;

/// A calendar date, written `"YYYY-MM-DD"` on the tape. Proleptic Gregorian; the year has at
/// least four digits and no sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    /// The year.
    pub y: i32,
    /// The month, 1 to 12.
    pub m: u8,
    /// The day of the month, from 1.
    pub d: u8,
}

impl Date {
    /// A date, if it exists in the proleptic Gregorian calendar and its year is not negative.
    pub fn new(y: i32, m: u8, d: u8) -> Result<Date, DateError> {
        if y < 0 || !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
            return Err(DateError(format!("{y:04}-{m:02}-{d:02}")));
        }
        Ok(Date { y, m, d })
    }

    /// Parse `"YYYY-MM-DD"`.
    pub fn parse(s: &str) -> Result<Date, DateError> {
        let bad = || DateError(s.to_string());
        let mut parts = s.split('-');
        let (Some(y), Some(m), Some(d), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(bad());
        };
        let digits = |p: &str, lo: usize, hi: usize| {
            p.len() >= lo && p.len() <= hi && p.bytes().all(|b| b.is_ascii_digit())
        };
        if !digits(y, 4, 9) || !digits(m, 2, 2) || !digits(d, 2, 2) {
            return Err(bad());
        }
        let y: i32 = y.parse().map_err(|_| bad())?;
        let m: u8 = m.parse().map_err(|_| bad())?;
        let d: u8 = d.parse().map_err(|_| bad())?;
        Date::new(y, m, d).map_err(|_| bad())
    }

    /// Days since 1970-01-01 (negative before it), by Howard Hinnant's `days_from_civil`.
    pub fn days(self) -> i64 {
        let (y, m, d) = (i64::from(self.y), i64::from(self.m), i64::from(self.d));
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01, by Howard Hinnant's `civil_from_days`. `None` if its
    /// year does not fit.
    pub fn from_days(days: i64) -> Option<Date> {
        let z = days.checked_add(719_468)?;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        Some(Date {
            y: i32::try_from(y).ok()?,
            m: u8::try_from(m).ok()?,
            d: u8::try_from(d).ok()?,
        })
    }
}

fn days_in_month(y: i32, m: u8) -> u8 {
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Date::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// A string or triple that is not a valid date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateError(pub String);

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid date {:?}: expected an existing YYYY-MM-DD",
            self.0
        )
    }
}

impl std::error::Error for DateError {}

/// Why a time conversion failed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockError {
    /// The date is before the clock's start.
    BeforeStart {
        /// The date.
        date: Date,
        /// The start.
        start: Date,
    },
    /// A period or shelf life rounds to zero ticks.
    ZeroTicks {
        /// The value in years.
        years: f64,
    },
    /// The value is not finite, is negative, or the result does not fit its integer type.
    OutOfRange {
        /// The value in years, or the tick.
        value: f64,
    },
}

impl fmt::Display for ClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClockError::BeforeStart { date, start } => {
                write!(f, "{date} is before the start, {start}")
            }
            ClockError::ZeroTicks { years } => {
                write!(f, "{years:e} years rounds to 0 ticks")
            }
            ClockError::OutOfRange { value } => write!(f, "{value:e} is out of range"),
        }
    }
}

impl std::error::Error for ClockError {}

/// The run's calendar: its start date and the registered tick length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clock {
    /// The date of tick 0.
    pub start: Date,
    /// Ticks per year, at least 1. Δ = 1/`ticks_per_year`.
    pub ticks_per_year: u32,
}

impl Clock {
    fn tpy(&self) -> f64 {
        f64::from(self.ticks_per_year)
    }

    /// A flow per tick: `v·Δ`, computed as `v/ticks_per_year`.
    pub fn flow(&self, v: FlowPerYear) -> f64 {
        v.0 / self.tpy()
    }

    /// The share of a stock a continuous annual rate draws in one tick: `−expm1(−v·Δ)`.
    pub fn share(&self, v: RatePerYear) -> f64 {
        -num::expm1(-(v.0 / self.tpy()))
    }

    /// The per-tick log step of a price rate under `Imbalance`: `v·Δ`.
    pub fn log_step(&self, v: RatePerYear) -> f64 {
        v.0 / self.tpy()
    }

    /// The per-tick rate equivalent to an effective annual rate ρ: `expm1(Δ·ln1p(ρ))`, that is
    /// `(1+ρ)^Δ − 1`.
    pub fn compound(&self, rho: CompoundPerYear) -> f64 {
        num::expm1(num::ln1p(rho.0) / self.tpy())
    }

    /// The per-tick fraction equivalent to an annual fraction δ: `−expm1(Δ·ln1p(−δ))`, that is
    /// `1 − (1−δ)^Δ`.
    pub fn fraction(&self, delta: FractionPerYear) -> f64 {
        -num::expm1(num::ln1p(-delta.0) / self.tpy())
    }

    /// The per-tick weight of an EMA with time constant τ years: `−expm1(−Δ/τ)`, with `Δ/τ`
    /// computed as `1/(ticks_per_year·τ)`.
    pub fn weight(&self, tau: Years) -> f64 {
        -num::expm1(-(1.0 / (self.tpy() * tau.0)))
    }

    /// A span in years as a whole number of ticks, `round(v·ticks_per_year)`. Zero ticks is an
    /// error: a period or shelf life must last at least one tick.
    pub fn ticks(&self, v: Years) -> Result<u32, ClockError> {
        if !num::is_clean(v.0) {
            return Err(ClockError::OutOfRange { value: v.0 });
        }
        let t = (v.0 * self.tpy()).round();
        if t == 0.0 {
            return Err(ClockError::ZeroTicks { years: v.0 });
        }
        if t > f64::from(u32::MAX) {
            return Err(ClockError::OutOfRange { value: v.0 });
        }
        Ok(t as u32)
    }

    /// The tick a date falls in: `floor(days·ticks_per_year·10⁴ / 3,652,425)`, in integers, with
    /// `days` counted from the start.
    pub fn tick_of(&self, date: Date) -> Result<u64, ClockError> {
        let days = date.days() - self.start.days();
        if days < 0 {
            return Err(ClockError::BeforeStart {
                date,
                start: self.start,
            });
        }
        let t = i128::from(days) * i128::from(self.ticks_per_year) * E4 / YEAR_E4_DAYS;
        u64::try_from(t).map_err(|_| ClockError::OutOfRange { value: t as f64 })
    }

    /// The first date of a tick: `start + ceil(t·3,652,425 / (ticks_per_year·10⁴))` days, in
    /// integers. `tick_of(date_of(t)) == t` whenever `ticks_per_year <= 365`. `None` if the date
    /// does not fit.
    pub fn date_of(&self, tick: u64) -> Option<Date> {
        let days = i64::try_from(self.days_to(tick)).ok()?;
        Date::from_days(self.start.days().checked_add(days)?)
    }

    /// The first day of a tick as [`Date::days`] counts them: the day of
    /// [`Clock::date_of`], saturating at `i64::MAX` where that would not fit. The schedule
    /// dates a recurring occurrence by it.
    pub fn first_day(&self, tick: u64) -> i64 {
        let day = i128::from(self.start.days()) + self.days_to(tick);
        i64::try_from(day).unwrap_or(i64::MAX)
    }

    /// Days from the start to the first day of `tick`: `ceil(t·3,652,425 /
    /// (ticks_per_year·10⁴))`.
    fn days_to(&self, tick: u64) -> i128 {
        let per = i128::from(self.ticks_per_year) * E4;
        (i128::from(tick) * YEAR_E4_DAYS + per - 1) / per
    }
}

/// How one use of a param turns its value into a per-tick value: a row of docs/ENGINE.md §6's
/// table. The use decides, not the unit: a `RatePerYear` is a `Share` of a stock where it draws
/// on one and a `LogStep` where it moves a price, and only the use knows which (amended at S2.2,
/// D10 item 4). Each method implies its unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ClockMethod {
    /// A `Dimensionless` value, as it is.
    Value,
    /// A `FlowPerYear` per tick, [`Clock::flow`].
    Flow,
    /// A `RatePerYear` as the share of a stock it draws in a tick, [`Clock::share`].
    Share,
    /// A `RatePerYear` as a per-tick log step, [`Clock::log_step`].
    LogStep,
    /// A `CompoundPerYear` per tick, [`Clock::compound`].
    Compound,
    /// A `FractionPerYear` per tick, [`Clock::fraction`].
    Fraction,
    /// A `Years` time constant as an EMA weight, [`Clock::weight`].
    Weight,
    /// A `Years` span as whole ticks, [`Clock::ticks`]: a shelf life or a period, turned into
    /// structure at load.
    Ticks,
}

impl ClockMethod {
    /// Every method, in declaration order.
    pub const ALL: [ClockMethod; 8] = [
        ClockMethod::Value,
        ClockMethod::Flow,
        ClockMethod::Share,
        ClockMethod::LogStep,
        ClockMethod::Compound,
        ClockMethod::Fraction,
        ClockMethod::Weight,
        ClockMethod::Ticks,
    ];

    /// The unit a param must be registered with for this use.
    pub fn unit(self) -> Unit {
        match self {
            ClockMethod::Value => Unit::Dimensionless,
            ClockMethod::Flow => Unit::FlowPerYear,
            ClockMethod::Share | ClockMethod::LogStep => Unit::RatePerYear,
            ClockMethod::Compound => Unit::CompoundPerYear,
            ClockMethod::Fraction => Unit::FractionPerYear,
            ClockMethod::Weight | ClockMethod::Ticks => Unit::Years,
        }
    }

    /// The method's name, as the registry listing prints it: `value`, `flow`, `share`,
    /// `log_step`, `compound`, `fraction`, `weight` or `ticks`.
    pub fn name(self) -> &'static str {
        match self {
            ClockMethod::Value => "value",
            ClockMethod::Flow => "flow",
            ClockMethod::Share => "share",
            ClockMethod::LogStep => "log_step",
            ClockMethod::Compound => "compound",
            ClockMethod::Fraction => "fraction",
            ClockMethod::Weight => "weight",
            ClockMethod::Ticks => "ticks",
        }
    }

    /// `v`, in this method's unit, as a per-tick value on `c`: the `Clock` method of §6's table
    /// (for `Ticks`, the whole number of ticks).
    pub fn per_tick(self, c: &Clock, v: f64) -> Result<f64, ClockError> {
        Ok(match self {
            ClockMethod::Value => v,
            ClockMethod::Flow => c.flow(FlowPerYear(v)),
            ClockMethod::Share => c.share(RatePerYear(v)),
            ClockMethod::LogStep => c.log_step(RatePerYear(v)),
            ClockMethod::Compound => c.compound(CompoundPerYear(v)),
            ClockMethod::Fraction => c.fraction(FractionPerYear(v)),
            ClockMethod::Weight => c.weight(Years(v)),
            ClockMethod::Ticks => f64::from(c.ticks(Years(v))?),
        })
    }
}

impl fmt::Display for ClockMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One use of a registered param, as a resolved spec holds it: the param, and the conversion its
/// use takes. The resolver makes it ([`crate::Resolver::param`]) and records the same method in
/// the param's [`ParamSite`]s, and the run reads the param only through it, so the method a site
/// declares is the conversion the run uses (amended at S2.2, D10 item 4). A `Site` inside a spec
/// or the world is part of `world_id`, since its method decides a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Site {
    /// The param.
    pub param: ParamId,
    /// How this use converts it.
    pub method: ClockMethod,
}

impl Site {
    /// The param's current value in its registered unit. A param registered with another unit
    /// than the method's is [`CoreError::UnitMismatch`].
    pub fn value(&self, params: &Params<'_>) -> Result<f64, CoreError> {
        let def = params
            .registry()
            .get(self.param)
            .ok_or(CoreError::UnknownParam(self.param))?;
        let unit = self.method.unit();
        if def.unit != unit {
            return Err(CoreError::UnitMismatch {
                param: self.param,
                registered: def.unit,
                requested: unit,
            });
        }
        params.value(self.param)
    }

    /// `v`, in the param's unit, converted by this site's method. A rule that scales an annual
    /// value before converting it (the cash rule's `share(v·μ^κ)`, docs/probe/RULES.md §2) reads
    /// [`Site::value`] and converts here, so the conversion is still the site's own.
    pub fn convert(&self, clock: &Clock, v: f64) -> Result<f64, CoreError> {
        self.method
            .per_tick(clock, v)
            .map_err(|_| CoreError::BadValue {
                what: "a param's per-tick conversion",
                value: v,
            })
    }

    /// The param's current per-tick value: [`Site::value`] converted by [`Site::convert`]. The
    /// one read of a param at use.
    pub fn per_tick(&self, params: &Params<'_>, clock: &Clock) -> Result<f64, CoreError> {
        let v = self.value(params)?;
        self.convert(clock, v)
    }
}

/// A use of a param as the registry records it: the tape path of the reference and the method
/// it takes. A `SetParam`'s source records its target's methods at the event's `to`; a
/// recurring period records `Ticks` at its `every`. Not part of `world_id`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ParamSite {
    /// The tape path, such as `actors[mill].spec.spend`.
    pub path: String,
    /// How the use converts the param.
    pub method: ClockMethod,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{CompoundPerYear, FlowPerYear, FractionPerYear, RatePerYear, Years};

    /// Relative bar for identities that compose a per-tick value `ticks_per_year` times. Each
    /// composition rounds about `ticks_per_year` times at 1.1e-16, so 52 ticks accumulate a few
    /// parts in 1e-14; 1e-12 leaves two orders of headroom and catches any wrong formula.
    const COMPOSE_BAR: f64 = 1e-12;

    fn clock(tpy: u32) -> Clock {
        Clock {
            start: Date::new(1750, 1, 1).unwrap(),
            ticks_per_year: tpy,
        }
    }

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs()
    }

    #[test]
    fn clock_methods_convert_as_the_clock() {
        // S2.2 (D10 item 4): a site's method is §6's row, bit for bit, with the unit it implies;
        // and a site reads its param only in that unit.
        let values = [0.0, 1e-300, 0.0577, 0.5, 1.0, 5.2, 10.4, 520.0];
        for tpy in [1u32, 12, 52, 365] {
            let c = clock(tpy);
            for v in values {
                let want = |m: ClockMethod| -> Option<f64> {
                    Some(match m {
                        ClockMethod::Value => v,
                        ClockMethod::Flow => c.flow(FlowPerYear(v)),
                        ClockMethod::Share => c.share(RatePerYear(v)),
                        ClockMethod::LogStep => c.log_step(RatePerYear(v)),
                        ClockMethod::Compound => c.compound(CompoundPerYear(v)),
                        ClockMethod::Fraction => c.fraction(FractionPerYear(v)),
                        ClockMethod::Weight => c.weight(Years(v)),
                        ClockMethod::Ticks => f64::from(c.ticks(Years(v)).ok()?),
                    })
                };
                for m in ClockMethod::ALL {
                    let got = m.per_tick(&c, v).ok().map(f64::to_bits);
                    assert_eq!(got, want(m).map(f64::to_bits), "{m} of {v} at {tpy}");
                }
            }
        }
        let units: Vec<Unit> = ClockMethod::ALL.iter().map(|m| m.unit()).collect();
        assert_eq!(
            units,
            [
                Unit::Dimensionless,
                Unit::FlowPerYear,
                Unit::RatePerYear,
                Unit::RatePerYear,
                Unit::CompoundPerYear,
                Unit::FractionPerYear,
                Unit::Years,
                Unit::Years
            ]
        );
        let names: Vec<&str> = ClockMethod::ALL.iter().map(|m| m.name()).collect();
        assert_eq!(
            names,
            ["value", "flow", "share", "log_step", "compound", "fraction", "weight", "ticks"]
        );
        // A site reads its param through its method, and only in the method's unit.
        let (w, s) = crate::testkit::load();
        let params = s.params(&w.registry);
        let rate = w.id_of::<ParamId>("rate.grain").unwrap();
        let site = |method| Site {
            param: rate,
            method,
        };
        let at = |m| site(m).per_tick(&params, &w.clock);
        assert_eq!(
            at(ClockMethod::LogStep),
            Ok(w.clock.log_step(RatePerYear(5.2)))
        );
        assert_eq!(at(ClockMethod::Share), Ok(w.clock.share(RatePerYear(5.2))));
        assert_ne!(at(ClockMethod::Share), at(ClockMethod::LogStep));
        assert_eq!(
            at(ClockMethod::Flow),
            Err(CoreError::UnitMismatch {
                param: rate,
                registered: Unit::RatePerYear,
                requested: Unit::FlowPerYear
            })
        );
        let life = site(ClockMethod::Ticks);
        assert!(matches!(
            life.convert(&w.clock, 0.001),
            Err(CoreError::BadValue { .. })
        ));
    }

    #[test]
    fn clock_conversions() {
        for tpy in [1u32, 4, 12, 52, 365] {
            let c = clock(tpy);
            let t = f64::from(tpy);
            // Flow: annual total over a year of ticks.
            assert_eq!(c.flow(FlowPerYear(520.0)), 520.0 / t);
            // Share: compounding the per-tick draw over a year leaves exp(−v).
            let v = 0.3;
            let s = c.share(RatePerYear(v));
            assert_eq!(s.to_bits(), (-num::expm1(-(v / t))).to_bits());
            assert!(rel(num::pow(1.0 - s, t), num::exp(-v)) < COMPOSE_BAR);
            // Log step: the Imbalance exponent per tick.
            assert_eq!(c.log_step(RatePerYear(5.2)), 5.2 / t);
            // Compound: tpy per-tick rates compound back to ρ.
            let rho = 0.05;
            let r = c.compound(CompoundPerYear(rho));
            assert!(rel(num::pow(1.0 + r, t) - 1.0, rho) < COMPOSE_BAR);
            // Fraction: tpy per-tick fractions compound back to δ.
            let delta = 0.08;
            let f = c.fraction(FractionPerYear(delta));
            assert!(rel(1.0 - num::pow(1.0 - f, t), delta) < COMPOSE_BAR);
            // Weight: after τ years of ticks the EMA's step response has decayed to e^-1.
            let tau = 2.0;
            let w = c.weight(Years(tau));
            assert!(rel(num::pow(1.0 - w, t * tau), num::exp(-1.0)) < COMPOSE_BAR);
            // Zero rates give exactly zero, never -0.0.
            for z in [
                c.share(RatePerYear(0.0)),
                c.log_step(RatePerYear(0.0)),
                c.compound(CompoundPerYear(0.0)),
                c.fraction(FractionPerYear(0.0)),
                c.flow(FlowPerYear(0.0)),
            ] {
                assert_eq!(z.to_bits(), 0, "tpy {tpy}");
            }
        }
        // Ticks: round(v·tpy) at every tick length (A13: a year is 52 weekly ticks, 12 monthly
        // and 365 daily; three weeks is 3, 1 and 21), and zero is an error.
        for (tpy, year, bread) in [(52, 52, 3), (12, 12, 1), (365, 365, 21)] {
            let c = clock(tpy);
            assert_eq!(c.ticks(Years(1.0)), Ok(year), "tpy {tpy}");
            assert_eq!(c.ticks(Years(0.0577)), Ok(bread), "tpy {tpy}");
            assert_eq!(c.ticks(Years(2.0)), Ok(year * 2), "tpy {tpy}");
        }
        let c = clock(52);
        assert!(matches!(
            c.ticks(Years(0.0)),
            Err(ClockError::ZeroTicks { .. })
        ));
        assert!(matches!(
            c.ticks(Years(0.009)),
            Err(ClockError::ZeroTicks { .. })
        ));
        assert!(matches!(
            c.ticks(Years(-1.0)),
            Err(ClockError::OutOfRange { .. })
        ));
        assert!(matches!(
            c.ticks(Years(f64::NAN)),
            Err(ClockError::OutOfRange { .. })
        ));
    }

    #[test]
    fn date_to_tick_is_integer_exact() {
        let c = clock(52);
        let d = |y, m, dd| Date::new(y, m, dd).unwrap();
        assert_eq!(c.tick_of(d(1750, 1, 1)), Ok(0));
        // 365 days · 52 · 10⁴ / 3,652,425 = 51.966…
        assert_eq!(c.tick_of(d(1751, 1, 1)), Ok(51));
        // 366 days: 52.108…
        assert_eq!(c.tick_of(d(1751, 1, 2)), Ok(52));
        assert!(matches!(
            c.tick_of(d(1749, 12, 31)),
            Err(ClockError::BeforeStart { .. })
        ));
        // At one tick per ten-thousandth of a mean year's day count the map is days·10⁴ exactly,
        // which a floating-point day count would not give.
        let fine = clock(3_652_425);
        for days in [1i64, 7, 365, 146_097, 3_000_000] {
            let date = Date::from_days(d(1750, 1, 1).days() + days).unwrap();
            assert_eq!(fine.tick_of(date), Ok(days as u64 * 10_000));
        }
        // Every day of 300 years at 52 ticks a year, against the formula in u128.
        let start = d(1750, 1, 1).days();
        for days in 0..(300 * 366) {
            let date = Date::from_days(start + days).unwrap();
            let want = (days as u128 * 52 * 10_000 / 3_652_425) as u64;
            assert_eq!(c.tick_of(date), Ok(want), "{date}");
        }
    }

    #[test]
    fn date_of_inverts_date_to_tick() {
        for tpy in [1u32, 4, 12, 52, 100, 365] {
            let c = clock(tpy);
            assert_eq!(c.date_of(0), Some(c.start));
            let mut last = c.start;
            for t in 0..20_000u64 {
                let date = c.date_of(t).unwrap();
                assert_eq!(c.tick_of(date), Ok(t), "tpy {tpy}, tick {t}");
                assert!(t == 0 || date > last, "dates increase");
                last = date;
            }
        }
    }

    #[test]
    fn dates_parse_print_and_count_days() {
        let d = Date::parse("1750-01-01").unwrap();
        assert_eq!(d.to_string(), "1750-01-01");
        assert_eq!(Date::parse("2000-02-29").unwrap().d, 29);
        for bad in [
            "1900-02-29",
            "1750-13-01",
            "1750-00-10",
            "1750-1-01",
            "175-01-01",
            "1750-01-01-01",
            "-750-01-01",
            "1750/01/01",
        ] {
            assert!(Date::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(Date::parse("1970-01-01").unwrap().days(), 0);
        for z in [-800_000i64, -1, 0, 1, 59, 60, 11_016, 2_932_896] {
            assert_eq!(Date::from_days(z).unwrap().days(), z);
        }
    }
}
