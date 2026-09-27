//! Unit 1e: the priced exit form alone (docs/unit-1e.md §4.2).
//!
//! main.tex's outside option (eq:exit, main.tex:370-372): an exit life yields s₀ units of the
//! exit good and needs h units of land service, so at the rent q per unit of service, in units
//! of the exit good, it is worth s(q) = max(s₀ − q·h, s̲), with s̲ the dependency floor. The
//! plot stops paying at q_enc = (s₀ − s̲)/h (main.tex:858). With a basket of g_s units of the
//! exit good and h_s of space, coverage is κ = qT/(N(g_s + q·h_s)) (SSRN eq 16 and 28;
//! main.tex:841-848), the fiscal replacement covers everyone from q* = N·g_s/(T − N·h_s), and
//! independent exit outlasts it iff N ≤ N_crit = q_enc·T/(g_s + q_enc·h_s) (main.tex:854-864;
//! check_enclosure N-iii).
//!
//! Each helper validates its arguments and returns [`ParamError`] for a non-finite or
//! out-of-range one.

use rustyecon_core::num;

use crate::params::{self, ParamError};

/// A priced exit form, main.tex's s(q) (docs/unit-1e.md §2.3 and §3.1), in units of the exit
/// good. "Zero or scale" means 0 or finite in [[`SCALE_FLOOR`](crate::SCALE_FLOOR),
/// [`SCALE_CEIL`](crate::SCALE_CEIL)].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PricedExit {
    /// s₀, the exit life's gross yield in goods. Zero or scale.
    pub gross: f64,
    /// s̲, the dependency floor in goods, "otherwise-funded" (main.tex:383). Zero or scale.
    pub floor: f64,
    /// h, the land service an exit plot takes. Zero or scale.
    pub plot: f64,
}

impl PricedExit {
    /// The form with each field checked (zero or scale, −0.0 stored as +0.0), as
    /// `ParamError`s named `gross`, `floor` and `plot`.
    pub fn validated(self) -> Result<PricedExit, ParamError> {
        Ok(PricedExit {
            gross: params::zero_or_scale("gross", self.gross)?,
            floor: params::zero_or_scale("floor", self.floor)?,
            plot: params::zero_or_scale("plot", self.plot)?,
        })
    }

    /// s(q) = max(s₀ − q·h, s̲) at the rent q ≥ 0 per unit of land service in units of the
    /// exit good (main.tex:370), s₀ − q·h with one rounding.
    pub fn value(&self, q: f64) -> Result<f64, ParamError> {
        let form = self.validated()?;
        let q = params::nonnegative("q", q)?;
        Ok(num::fma(-q, form.plot, form.gross).max(form.floor))
    }

    /// q_enc = (s₀ − s̲)/h, where a rented plot stops paying (main.tex:858); `None` when the
    /// plot never pays (s₀ ≤ s̲) or takes no land (h = 0).
    pub fn threshold(&self) -> Result<Option<f64>, ParamError> {
        let form = self.validated()?;
        let advantage = form.gross - form.floor;
        Ok((form.plot > 0.0 && advantage > 0.0).then(|| advantage / form.plot))
    }

    /// The take s₀ − s(q): q·h below q_enc, capped at h·q_enc = s₀ − s̲ above it
    /// (check_enclosure N-vi).
    pub fn take(&self, q: f64) -> Result<f64, ParamError> {
        let value = self.value(q)?;
        Ok(self.gross - value)
    }
}

/// The basket's coefficients: g_s of the exit good and h_s of space, each zero or scale, not
/// both zero.
fn basket(goods: f64, space: f64) -> Result<(f64, f64), ParamError> {
    let goods = params::zero_or_scale("goods", goods)?;
    let space = params::zero_or_scale("space", space)?;
    if goods == 0.0 && space == 0.0 {
        return Err(ParamError::Invalid {
            name: "basket",
            reason: "the basket needs goods or space",
        });
    }
    Ok((goods, space))
}

/// κ = qT/(N(g_s + q·h_s)), coverage (SSRN eq 16 and 28; main.tex:841-848): the enclosed rent
/// base over the support bill, with the basket P_s = p·g_s + r·h_s and q = r/p. q ≥ 0; T and
/// N scale.
pub fn coverage(q: f64, land: f64, people: f64, goods: f64, space: f64) -> Result<f64, ParamError> {
    let q = params::nonnegative("q", q)?;
    let land = params::scale("land", land)?;
    let people = params::scale("people", people)?;
    let (goods, space) = basket(goods, space)?;
    Ok((q * land) / (people * (goods + q * space)))
}

/// q* = N·g_s/(T − N·h_s), the rent from which coverage reaches 1 (main.tex:846; SSRN D.3);
/// `None` when T ≤ N·h_s, where κ < 1 at every q.
pub fn coverage_threshold(
    land: f64,
    people: f64,
    goods: f64,
    space: f64,
) -> Result<Option<f64>, ParamError> {
    let land = params::scale("land", land)?;
    let people = params::scale("people", people)?;
    let (goods, space) = basket(goods, space)?;
    let room = land - people * space;
    Ok((room > 0.0).then(|| (people * goods) / room))
}

/// N_crit = q_enc·T/(g_s + q_enc·h_s), the population up to which independent exit outlasts
/// the fiscal replacement, q_enc ≥ q* (main.tex:862; check_enclosure N-iii). q_enc ≥ 0.
pub fn crowding_limit(
    threshold: f64,
    land: f64,
    goods: f64,
    space: f64,
) -> Result<f64, ParamError> {
    let threshold = params::nonnegative("q_enc", threshold)?;
    let land = params::scale("land", land)?;
    let (goods, space) = basket(goods, space)?;
    Ok((threshold * land) / (goods + threshold * space))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_the_max_of_the_plot_and_the_floor() {
        let e = PricedExit {
            gross: 10.0,
            floor: 4.0,
            plot: 6.0,
        };
        assert_eq!(e.threshold().unwrap(), Some(1.0));
        assert_eq!(e.value(0.5).unwrap(), 7.0);
        assert_eq!(e.value(1.0).unwrap(), 4.0);
        assert_eq!(e.value(3.0).unwrap(), 4.0);
        assert_eq!(e.take(3.0).unwrap(), 6.0);
        // A plot that never pays, or takes no land, has no threshold.
        let flat = PricedExit { plot: 0.0, ..e };
        assert_eq!(flat.threshold().unwrap(), None);
        let never = PricedExit { floor: 10.0, ..e };
        assert_eq!(never.threshold().unwrap(), None);
        assert_eq!(never.value(0.0).unwrap(), 10.0);
    }

    #[test]
    fn arguments_are_checked() {
        let e = PricedExit {
            gross: 1.5,
            floor: 0.0,
            plot: 1.0,
        };
        assert_eq!(e.value(-1.0).unwrap_err().name(), "q");
        assert_eq!(e.value(f64::NAN).unwrap_err().name(), "q");
        let bad = PricedExit { gross: -1.0, ..e };
        assert_eq!(bad.value(1.0).unwrap_err().name(), "gross");
        assert_eq!(
            coverage(1.0, 100.0, 50.0, 0.0, 0.0).unwrap_err().name(),
            "basket"
        );
        assert_eq!(
            coverage(1.0, 0.0, 50.0, 1.0, 1.0).unwrap_err().name(),
            "land"
        );
        assert_eq!(
            coverage_threshold(100.0, f64::INFINITY, 1.0, 1.0)
                .unwrap_err()
                .name(),
            "people"
        );
        assert_eq!(
            crowding_limit(-0.5, 100.0, 1.0, 1.0).unwrap_err().name(),
            "q_enc"
        );
    }
}
