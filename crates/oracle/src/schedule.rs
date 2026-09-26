//! The capability schedule γ(x) and its integral J(x) (spec §2; SSRN eq 21, p.28).

use rustyecon_core::num;

use crate::params::{self, ParamError, Requirement};

/// The default [`Schedule::validate`] samples γ at x = i/`VALIDATION_SAMPLES` for
/// i = 0..=`VALIDATION_SAMPLES`.
pub const VALIDATION_SAMPLES: u32 = 16;

/// The largest curvature k that [`PowerSchedule::validate`] accepts: 1024 = 2^10.
///
/// The solve returns x* as a double, one of the two adjacent doubles around the root,
/// and evaluates γ and every price there. Across one spacing of doubles at x, x^k changes
/// by at most 1 − (1 − spacing/x)^k ≤ max(k, 1)·spacing/x relative (Bernoulli for k ≥ 1;
/// for k < 1, (1 − t)^k ≥ 1 − t), and spacing/x ≤ 2^-52 for every normal double, so
/// γ = η(g0 + g1·x^k) changes by less than max(k, 1)·2^-52 relative however g0 and g1
/// compare (the g1 term is at most all of γ). At k = 1024 that is 2^-42 ≈ 2.3e-13, so
/// away from the viability edge (where prices scale as 1/D) the outputs at the double x*
/// stay within the gate's 1e-12 of the exact equilibrium, with a factor of four to spare
/// for the sensitivities of order 1 that carry γ's error into the prices. Above the
/// ceiling that margin goes. At k = 1e20, x^k is 0 at the double
/// below 1 and 1 at 1, so γ jumps from g0 to g0 + g1 across one double and bisection
/// closes on that jump rather than on a root (review of 2026-09-25: x* = 1 − 2^-53 and
/// v = 0.1159, against 1 − x* = 2.55e-20 and v = 0.1527 from a 60-digit solve in
/// s = 1 − x).
///
/// The paper's schedules have k = 1 (SSRN p.30); G5 draws k up to 6 and pins k = 4.5 and
/// 2.5. The ceiling is 170 times the largest of these. G8 solves at the ceiling against a
/// 70-digit golden (γ(x*) and v within 1.3e-14), and 2000 random interior economies with
/// k up to the ceiling matched generate.py's solve within 5.1e-14 on every output.
pub const CURVATURE_CEIL: f64 = 1024.0;

/// Relative human productivity γ(x) = γ_L(x)/γ_M(x) on the task line [0, 1].
///
/// With γ_L = 1 and γ_M = 1/γ, machines perform the tasks below the threshold x and
/// people those above it (SSRN p.28).
///
/// **Contract.** γ is finite, positive, continuous and strictly increasing on [0, 1], and
/// [`integral`](Schedule::integral) returns J(x) = ∫₀ˣ γ(t) dt. The paper assumes
/// exactly this (γ' > 0, SSRN p.28). The solver relies on it: viability at x = 1 implies
/// viability on all of [0, 1], and the equilibrium threshold is unique (SSRN Lemma B.1,
/// p.29). A γ that jumps can make the excess demand change sign without a root; the
/// solve then returns [`SolveError::LaborNotCleared`](crate::SolveError::LaborNotCleared)
/// (see [`LABOR_RESIDUAL_NET`](crate::LABOR_RESIDUAL_NET)).
///
/// J(x) is the capability integral. It is not the build lag J_b.
pub trait Schedule {
    /// γ(x).
    fn gamma(&self, x: f64) -> f64;

    /// J(x) = ∫₀ˣ γ(t) dt.
    fn integral(&self, x: f64) -> f64;

    /// Checks the contract as far as the schedule's form allows.
    ///
    /// The default can only sample. It checks that γ(0) is in
    /// [[`SCALE_FLOOR`](crate::SCALE_FLOOR), [`SCALE_CEIL`](crate::SCALE_CEIL)], that γ is
    /// finite and strictly increasing on the points x = i/[`VALIDATION_SAMPLES`], that
    /// J(0) = 0, and that γ(0) < J(1) < γ(1), which every strictly increasing γ
    /// satisfies. A γ that dips or jumps between samples passes; a schedule with a closed
    /// form should override this with the exact condition on its coefficients, as
    /// [`PowerSchedule`] does.
    fn validate(&self) -> Result<(), ParamError> {
        let g0 = params::scale("gamma(0)", self.gamma(0.0))?;
        let mut last = g0;
        for i in 1..=VALIDATION_SAMPLES {
            let x = f64::from(i) / f64::from(VALIDATION_SAMPLES);
            let g = params::finite("gamma", self.gamma(x))?;
            if g <= last {
                return Err(ParamError::OutOfRange {
                    name: "gamma",
                    value: g,
                    requirement: Requirement::SampledIncrease,
                });
            }
            last = g;
        }
        let g1 = last;
        let (j0, j1) = (self.integral(0.0), self.integral(1.0));
        if j0 != 0.0 {
            return Err(ParamError::OutOfRange {
                name: "J(0)",
                value: j0,
                requirement: Requirement::Schedule("J(0) = 0: J is the integral of gamma from 0"),
            });
        }
        params::finite("J(1)", j1)?;
        if j1 <= g0 || j1 >= g1 {
            return Err(ParamError::OutOfRange {
                name: "J(1)",
                value: j1,
                requirement: Requirement::Schedule(
                    "gamma(0) < J(1) < gamma(1) for an increasing gamma",
                ),
            });
        }
        Ok(())
    }
}

/// γ(x) = η(g0 + g1·x^k), the family of laborformal `paths/code/macro.py:56-68` at
/// 31b3482.
///
/// The paper's instance is η = 1, g0 = 0.2, g1 = 0.8, k = 1 (SSRN p.30). With
/// k = 1 and g0 = g1 = η it is the automation path γ = η(1 + x) (SSRN p.30).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PowerSchedule {
    /// η, a level that scales the whole schedule (task automation lowers it).
    pub eta: f64,
    /// g0, the schedule's value at x = 0 before scaling.
    pub g0: f64,
    /// g1, the rise from x = 0 to x = 1 before scaling.
    pub g1: f64,
    /// k, the curvature: k > 1 concentrates human advantage near x = 1.
    pub k: f64,
}

impl Schedule for PowerSchedule {
    fn gamma(&self, x: f64) -> f64 {
        self.eta * (self.g0 + self.g1 * num::pow(x, self.k))
    }

    fn integral(&self, x: f64) -> f64 {
        // Same operation order as macro.py:68.
        self.eta * (self.g0 * x + self.g1 * num::pow(x, self.k + 1.0) / (self.k + 1.0))
    }

    /// η, g0 and g1 finite and in [[`SCALE_FLOOR`](crate::SCALE_FLOOR),
    /// [`SCALE_CEIL`](crate::SCALE_CEIL)], so positive, and k in
    /// [[`SCALE_FLOOR`](crate::SCALE_FLOOR), [`CURVATURE_CEIL`]]. Together these make γ
    /// positive, continuous and strictly increasing on [0, 1], and CURVATURE_CEIL keeps it
    /// resolved by the doubles of x.
    fn validate(&self) -> Result<(), ParamError> {
        params::scale("eta", self.eta)?;
        params::scale("g0", self.g0)?;
        params::scale("g1", self.g1)?;
        params::curvature("k", self.k)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pow2;

    const PAPER: PowerSchedule = PowerSchedule {
        eta: 1.0,
        g0: 0.2,
        g1: 0.8,
        k: 1.0,
    };

    #[test]
    fn paper_schedule_values() {
        // γ(x) = 0.2 + 0.8x and J(x) = 0.2x + 0.4x² (SSRN p.30).
        assert_eq!(PAPER.gamma(0.0), 0.2);
        assert_eq!(PAPER.gamma(1.0), 1.0);
        assert_eq!(PAPER.integral(0.0), 0.0);
        assert_eq!(PAPER.integral(1.0), 0.6000000000000001); // 0.2 + 0.8/2 in f64
        assert!((PAPER.integral(0.5) - 0.2).abs() < 1e-16);
    }

    #[test]
    fn values_away_from_k_one() {
        // Point values with no root-finding between the formula and the check, at powers
        // of x = 1/4 that are exact in binary: 0.25^2.5 = 2^-5, 0.25^0.5 = 2^-1 and
        // 0.25^6.5 = 2^-13, and J's powers 0.25^(k+1) likewise. The review's mutants that
        // round k, or cap it at 4, pass every golden at k = 1 and fail here.
        let close = |got: f64, want: f64, what: &str| {
            assert!(
                (got - want).abs() <= 4.0 * f64::EPSILON * want,
                "{what}: got {got:e}, want {want:e}"
            );
        };
        for (k, x_k, x_k1) in [
            (2.5, pow2(-5), pow2(-7)),
            (0.5, 0.5, pow2(-3)),
            (6.5, pow2(-13), pow2(-15)),
        ] {
            let s = PowerSchedule {
                eta: 1.25,
                g0: 0.2,
                g1: 0.8,
                k,
            };
            // gamma(1/4) = eta (g0 + g1/4^k); J(1/4) = eta (g0/4 + g1/(4^(k+1) (k + 1))).
            close(
                s.gamma(0.25),
                1.25 * (0.2 + 0.8 * x_k),
                &format!("gamma(1/4), k = {k}"),
            );
            close(
                s.integral(0.25),
                1.25 * (0.2 * 0.25 + 0.8 * x_k1 / (k + 1.0)),
                &format!("J(1/4), k = {k}"),
            );
        }
        // The review's value: eta 1, g0 0.2, g1 0.8, k 2.5 gives gamma(1/4) = 0.225, and
        // J(1/4) = 0.05 + 0.8/(128 * 3.5) = 29/560.
        let s = PowerSchedule { k: 2.5, ..PAPER };
        close(s.gamma(0.25), 0.225, "gamma(1/4) = 0.225");
        close(s.integral(0.25), 29.0 / 560.0, "J(1/4) = 29/560");
        // At x = 1 the power is 1 for every k: gamma(1) = eta (g0 + g1) and
        // J(1) = eta (g0 + g1/(k + 1)).
        let s = PowerSchedule {
            eta: 2.3,
            g0: 0.15,
            g1: 0.9,
            k: 4.5,
        };
        close(s.gamma(1.0), 2.3 * (0.15 + 0.9), "gamma(1)");
        close(s.integral(1.0), 2.3 * (0.15 + 0.9 / 5.5), "J(1)");
    }

    #[test]
    fn integral_matches_quadrature() {
        // Simpson's rule with 2000 panels: exact (to rounding) for the paper's quadratic
        // J, and within about 1e-11 for the smooth k = 4.83 schedule. k < 1 has an
        // infinite slope at 0, which Simpson's rule resolves poorly; the derivative test
        // below covers it.
        let schedules = [
            PAPER,
            PowerSchedule {
                eta: 0.5,
                g0: 0.1,
                g1: 1.3,
                k: 4.83,
            },
        ];
        let panels = 2000;
        for s in schedules {
            for &x in &[0.1, 0.5, 0.86, 1.0] {
                let h = x / panels as f64;
                let mut sum = s.gamma(0.0) + s.gamma(x);
                for i in 1..panels {
                    let w = if i % 2 == 1 { 4.0 } else { 2.0 };
                    sum += w * s.gamma(i as f64 * h);
                }
                let simpson = sum * h / 3.0;
                let rel = (simpson - s.integral(x)).abs() / s.integral(x);
                assert!(rel < 1e-9, "{s:?} at x = {x}: rel err {rel:e}");
            }
        }
    }

    #[test]
    fn integral_differentiates_to_gamma() {
        // Central differences with step 1e-5: truncation near 1e-10 and rounding near
        // 1e-11 relative, far inside the 1e-7 bound.
        let schedules = [
            PAPER,
            PowerSchedule {
                eta: 0.5,
                g0: 0.1,
                g1: 1.3,
                k: 4.83,
            },
            PowerSchedule {
                eta: 2.0,
                g0: 0.01,
                g1: 0.2,
                k: 0.5,
            },
        ];
        let step = 1e-5;
        for s in schedules {
            for &x in &[0.1, 0.5, 0.86, 0.99] {
                let slope = (s.integral(x + step) - s.integral(x - step)) / (2.0 * step);
                let rel = (slope - s.gamma(x)).abs() / s.gamma(x);
                assert!(rel < 1e-7, "{s:?} at x = {x}: rel err {rel:e}");
            }
        }
    }

    #[test]
    fn power_schedule_validation() {
        assert!(PAPER.validate().is_ok());
        for (bad, name) in [
            (PowerSchedule { eta: 0.0, ..PAPER }, "eta"),
            (PowerSchedule { g0: 0.0, ..PAPER }, "g0"),
            (PowerSchedule { g1: 0.0, ..PAPER }, "g1"),
            (PowerSchedule { k: -1.0, ..PAPER }, "k"),
            (
                PowerSchedule {
                    k: f64::NAN,
                    ..PAPER
                },
                "k",
            ),
            (
                PowerSchedule {
                    eta: f64::INFINITY,
                    ..PAPER
                },
                "eta",
            ),
        ] {
            let err = bad.validate().expect_err("invalid schedule accepted");
            assert_eq!(err.name(), name, "{bad:?}");
        }
    }

    #[test]
    fn curvature_has_its_own_ceiling() {
        // k = CURVATURE_CEIL is accepted and the next double is not, with a message that
        // states the bound from the constant. k = 1e30 was accepted under SCALE_CEIL.
        assert!(PowerSchedule {
            k: CURVATURE_CEIL,
            ..PAPER
        }
        .validate()
        .is_ok());
        for k in [CURVATURE_CEIL.next_up(), 1e20, 1e30] {
            let err = PowerSchedule { k, ..PAPER }
                .validate()
                .expect_err("above the curvature ceiling accepted");
            assert_eq!(err.name(), "k");
            let text = err.to_string();
            assert!(
                text.contains("CURVATURE_CEIL") && text.contains(&format!("{CURVATURE_CEIL}")),
                "{text}"
            );
        }
        // The justification: across one double just below 1, x^k at the ceiling moves by
        // about k·2^-53 relative, far inside the bound k·2^-52 < 1e-12.
        let below = 1.0f64.next_down();
        let step = 1.0 - num::pow(below, CURVATURE_CEIL);
        assert!(step > 0.9 * CURVATURE_CEIL * pow2(-53), "{step:e}");
        assert!(CURVATURE_CEIL * pow2(-52) < 1e-12);
        // At k = 1e20 the same double gives x^k = 0: gamma jumps from g0 to g0 + g1.
        assert_eq!(num::pow(below, 1e20), 0.0);
    }

    /// A schedule defined only through the trait, to exercise the default check.
    struct Table {
        gamma: fn(f64) -> f64,
        integral: fn(f64) -> f64,
    }

    impl Schedule for Table {
        fn gamma(&self, x: f64) -> f64 {
            (self.gamma)(x)
        }
        fn integral(&self, x: f64) -> f64 {
            (self.integral)(x)
        }
    }

    #[test]
    fn default_validation_samples_the_contract() {
        let good = Table {
            gamma: |x| 1.0 + x * x,
            integral: |x| x + x * x * x / 3.0,
        };
        assert!(good.validate().is_ok());
        let falling = Table {
            gamma: |x| 2.0 - x,
            integral: |x| 2.0 * x - x * x / 2.0,
        };
        assert_eq!(falling.validate().unwrap_err().name(), "gamma");
        let flat = Table {
            gamma: |_| 1.0,
            integral: |x| x,
        };
        assert_eq!(flat.validate().unwrap_err().name(), "gamma");
        // Rises from gamma(0) = 0.2 to gamma(1) = 1 but dips inside: gamma(1/8) = 0.3 is below
        // gamma(1/16) = 0.55. It passed the check before sampling. The review's case was
        // 0.2 + 0.8x + 0.3 sin(20x), whose dip falls between the same two samples; the
        // workspace denies the platform's sin (A5) and core::num has none, so the bump is a
        // tent of height 0.3 on [0, 1/8], peaking at 1/16, with area 1/16.
        let wavy = Table {
            gamma: |x| 0.2 + 0.8 * x + 0.3 * (1.0 - (16.0 * x - 1.0).abs()).max(0.0),
            integral: |x| {
                let t = (16.0 * x).min(2.0);
                let tent = if t <= 1.0 {
                    t * t / 2.0
                } else {
                    1.0 - (2.0 - t) * (2.0 - t) / 2.0
                };
                0.2 * x + 0.4 * x * x + 0.3 * tent / 16.0
            },
        };
        let err = wavy.validate().unwrap_err();
        assert_eq!(err.name(), "gamma");
        let text = err.to_string();
        assert!(text.contains("strictly increase"), "{text}");
        // The message states the sampling from the constant.
        assert!(
            text.contains(&format!("x = i/{VALIDATION_SAMPLES}")),
            "{text}"
        );
        let negative = Table {
            gamma: |x| x - 0.5,
            integral: |x| x * x / 2.0 - 0.5 * x,
        };
        assert_eq!(negative.validate().unwrap_err().name(), "gamma(0)");
        let offset = Table {
            gamma: |x| 1.0 + x,
            integral: |x| 1.0 + x + x * x / 2.0,
        };
        assert_eq!(offset.validate().unwrap_err().name(), "J(0)");
        let wrong_scale = Table {
            gamma: |x| 1.0 + x,
            integral: |x| 3.0 * (x + x * x / 2.0),
        };
        assert_eq!(wrong_scale.validate().unwrap_err().name(), "J(1)");
    }
}
