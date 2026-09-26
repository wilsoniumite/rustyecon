//! G8: regime recognition. The cases are constructed (2026-09-25); laborformal has no
//! instance.

use oracle::{
    dump, Economy, Params, PowerSchedule, Regime, Schedule, SolveError, UniformWorkCost,
    BRACKET_LO, CURVATURE_CEIL, LABOR_RESIDUAL_NET, SCALE_CEIL,
};

use crate::goldens::*;
use crate::support::*;

fn regime(params: Params) -> Regime {
    economy(params).solve().expect("finite")
}

#[test]
fn base_is_interior() {
    assert!(matches!(regime(appendix_b()), Regime::Interior(_)));
}

#[test]
fn few_workers_hold_no_contestable_task() {
    match regime(Params {
        workers: 0.25,
        ..appendix_b()
    }) {
        Regime::BoundaryNoMargin { f_at_1 } => close("f(1)", f_at_1, G8_N025_F_AT_1),
        other => panic!("expected BoundaryNoMargin, got {other:?}"),
    }
}

#[test]
fn labour_heavy_machines_hold_no_contestable_task() {
    match regime(Params {
        lam: 0.6,
        ..appendix_b()
    }) {
        Regime::BoundaryNoMargin { f_at_1 } => close("f(1)", f_at_1, G8_LAM06_F_AT_1),
        other => panic!("expected BoundaryNoMargin, got {other:?}"),
    }
}

#[test]
fn not_viable() {
    match regime(Params {
        lam: 0.8,
        ..appendix_b()
    }) {
        Regime::NotViable { d_at_1 } => close("D(1)", d_at_1, G8_LAM08_D_AT_1),
        other => panic!("expected NotViable, got {other:?}"),
    }
}

#[test]
fn not_viable_through_the_user_cost() {
    // Viable as a flow economy (D(1) = 0.05), not at u = 1.1.
    let flow = Params {
        a: 0.6,
        lam: 0.35,
        ..appendix_b()
    };
    assert!(economy(flow.clone()).at(1.0).d > 0.0);
    match regime(Params { rho: 0.1, ..flow }) {
        Regime::NotViable { d_at_1 } => close("D(1)", d_at_1, G8_RHO01_D_AT_1),
        other => panic!("expected NotViable, got {other:?}"),
    }
}

#[test]
fn no_interior_at_zero() {
    // T/h = 10 < N = 20, and a low work cost puts everyone to work.
    let params = Params {
        workers: 20.0,
        work_cost: UniformWorkCost { chi_max: 0.05 },
        ..appendix_b()
    };
    match regime(params) {
        Regime::NoInteriorAtZero { f_at_0 } => close("f(1e-12)", f_at_0, G8_N20_F_AT_LO),
        other => panic!("expected NoInteriorAtZero, got {other:?}"),
    }
}

#[test]
fn interior_although_lemma_b1_fails() {
    // N = 8: the equilibrium is interior but T = 10 < N P_s(1), so Lemma B.1's funding
    // condition (SSRN eq 25) fails.
    let params = Params {
        workers: 8.0,
        ..appendix_b()
    };
    let e = economy(params.clone());
    close(
        "N P_s(1)",
        params.workers * e.at(1.0).p_s,
        G8_N8_SUPPORT_COST_AT_1,
    );
    let eq = interior(params);
    close("x*", eq.x_star, G8_N8_X_STAR);
    assert_eq!(eq.lemma_b1, G8_N8_LEMMA_B1);
    assert_eq!(eq.funded, G8_N8_FUNDED);
}

#[test]
fn funded_although_lemma_b1_fails() {
    // N = 7.35: N P_s(x*) < T = 10 < N P_s(1), so support is funded at the equilibrium
    // (macro.py:113) while Lemma B.1's funding condition at x = 1 (SSRN eq 25) fails. The
    // two flags test P_s at different points and must not be confused.
    let params = Params {
        workers: 7.35,
        ..appendix_b()
    };
    let e = economy(params.clone());
    close(
        "N P_s(1)",
        params.workers * e.at(1.0).p_s,
        G8_N7_35_SUPPORT_COST_AT_1,
    );
    let eq = interior(params.clone());
    close("x*", eq.x_star, G8_N7_35_X_STAR);
    close("N P_s(x*)", eq.support_cost, G8_N7_35_SUPPORT_COST);
    assert!(eq.support_cost < params.land);
    assert!(params.land < params.workers * e.at(1.0).p_s);
    assert_eq!(eq.lemma_b1, G8_N7_35_LEMMA_B1);
    assert_eq!(eq.funded, G8_N7_35_FUNDED);
}

#[test]
fn lemma_b1_is_strict_at_its_funding_boundary() {
    // SSRN eq 25 asks T > N P_s(1), strictly. P_s(1) does not depend on T, so
    // T = 4 P_s(1) in f64 makes N P_s(1) equal T exactly (scaling by 4 is exact). The
    // supply condition n_S(1) > n_D(1) holds, so the funding condition alone decides the
    // flag, and it must read false. With >= it would read true.
    let ps_at_1 = economy(appendix_b()).at(1.0).p_s;
    let params = Params {
        land: 4.0 * ps_at_1,
        ..appendix_b()
    };
    let at_one = economy(params.clone()).at(1.0);
    assert_eq!(params.workers * at_one.p_s, params.land);
    assert!(at_one.n_s > at_one.n_d, "n_S(1) > n_D(1) must hold here");
    let eq = interior(params);
    assert!(!eq.lemma_b1);
    // Funding at x* is a different test (P_s(x*) < P_s(1)), and it holds.
    assert!(eq.funded && eq.provider_baskets > 0.0);
}

#[test]
fn funded_is_strict_at_an_exact_tie() {
    // At N = 7.41560525573509, N P_s(x*) is exactly T = 10 in f64, and the provider's
    // baskets (T + interest)/P_s - N are exactly 0: support is not funded (macro.py:113,
    // strictly). With >= it would read true.
    let params = Params {
        workers: 7.41560525573509,
        ..appendix_b()
    };
    let eq = interior(params.clone());
    assert_eq!(eq.support_cost, 10.0);
    assert_eq!(eq.support_cost, params.land + eq.interest);
    assert_eq!(eq.provider_baskets, 0.0);
    assert!(!eq.funded);
    // funded is provider_baskets > 0 on every side of the tie, in both directions.
    let (mut seen_true, mut seen_false) = (false, false);
    let mut workers = params.workers;
    for _ in 0..64 {
        workers = workers.next_down();
    }
    for _ in 0..=128 {
        let eq = interior(Params {
            workers,
            ..params.clone()
        });
        assert_eq!(eq.funded, eq.provider_baskets > 0.0, "N = {workers:?}");
        seen_true |= eq.funded;
        seen_false |= !eq.funded;
        workers = workers.next_up();
    }
    assert!(seen_true && seen_false);
}

// The regime tests at exact equality in f64. The spec's convention is D(1) <= 0 NotViable,
// f(1) >= 0 BoundaryNoMargin and f(lo) <= 0 NoInteriorAtZero; each case below makes the
// f64 value exactly 0. They test the convention on the f64 evaluation: in exact
// arithmetic on the same doubles the sign can differ (docs/unit-1a.md §4).

#[test]
fn d_at_1_exactly_zero_is_not_viable() {
    // 0.3 + 0.7 * 1 rounds to 1 in f64 (and is 1 in decimal), so D(1) = 0 and V_m = b/0
    // would be infinite.
    match regime(Params {
        lam: 0.7,
        ..appendix_b()
    }) {
        Regime::NotViable { d_at_1 } => assert_eq!(d_at_1, G8_LAM07_D_AT_1),
        other => panic!("expected NotViable, got {other:?}"),
    }
    // 0.5 + 0.1 * (1 + 4) is 1 in f64 too.
    let base = appendix_b();
    match regime(Params {
        a: 0.5,
        lam: 0.1,
        schedule: PowerSchedule {
            g0: 1.0,
            g1: 4.0,
            ..base.schedule
        },
        ..base
    }) {
        Regime::NotViable { d_at_1 } => assert_eq!(d_at_1, 0.0),
        other => panic!("expected NotViable, got {other:?}"),
    }
}

/// The Appendix B instance with χ_max = 0.05, low enough that everyone works at x = lo
/// and at x = 1 (z = ln(1 + v/P_s) > 0.05 at both), so n_S = N there exactly.
fn saturated(workers: f64) -> Params {
    Params {
        workers,
        work_cost: UniformWorkCost { chi_max: 0.05 },
        ..appendix_b()
    }
}

#[test]
fn f_at_1_exactly_zero_is_boundary() {
    // n_D(1) does not depend on N, so N = n_D(1) makes f(1) = n_D(1) - N exactly 0.
    let workers = economy(saturated(1.0)).at(1.0).n_d;
    assert_eq!(workers, 0.31914893617021284); // 15/47 rounded
    let at_one = economy(saturated(workers)).at(1.0);
    assert_eq!(at_one.n_s, workers);
    match regime(saturated(workers)) {
        Regime::BoundaryNoMargin { f_at_1 } => assert_eq!(f_at_1, 0.0),
        other => panic!("expected BoundaryNoMargin, got {other:?}"),
    }
}

#[test]
fn f_at_lo_exactly_zero_is_no_interior() {
    // Likewise N = n_D(lo) makes f(lo) exactly 0, with f(1) < 0.
    let workers = economy(saturated(1.0)).at(BRACKET_LO).n_d;
    assert_eq!(economy(saturated(workers)).at(BRACKET_LO).n_s, workers);
    assert!(economy(saturated(workers)).at(1.0).excess_demand() < 0.0);
    match regime(saturated(workers)) {
        Regime::NoInteriorAtZero { f_at_0 } => assert_eq!(f_at_0, 0.0),
        other => panic!("expected NoInteriorAtZero, got {other:?}"),
    }
}

#[test]
fn no_interior_at_lo_although_t_over_h_exceeds_n() {
    // T/h > N rules NoInteriorAtZero out at x = 0 only. Here T/h - N = 5e-12, less than
    // n_D(0) - n_D(lo) = (T/h) lo [1 + delta gamma(0) (b/h - lambda)/(1 - a delta)]
    // = 1.1e-11 to first order, so f(lo) < 0.
    let params = Params {
        workers: 10.0,
        land: 10.000000000005,
        work_cost: UniformWorkCost { chi_max: 1e-3 },
        ..appendix_b()
    };
    assert!(params.land / params.space > params.workers);
    match regime(params) {
        Regime::NoInteriorAtZero { f_at_0 } => assert!(f_at_0 < 0.0, "{f_at_0:e}"),
        other => panic!("expected NoInteriorAtZero, got {other:?}"),
    }
}

#[test]
fn no_interior_at_lo_when_a_delta_is_near_one() {
    // a = 1 - 2^-53, lambda = 0: 1 - a delta = 2^-53, and the first-order gap above is
    // 7e14 lo, not small. n_D falls from T/h = 10 at x = 0 to 0.014 at lo, so f(lo) < 0
    // although T/h - N = 6. A root exists in (0, lo), near 3.6e-15, where the solve does
    // not look (Regime::NoInteriorAtZero).
    let params = Params {
        a: 1.0 - f64::EPSILON / 2.0,
        lam: 0.0,
        ..appendix_b()
    };
    assert_eq!(params.a, 0.9999999999999999);
    let e = economy(params.clone());
    assert!(e.at(0.0).excess_demand() > 0.0 && e.at(1e-15).excess_demand() > 0.0);
    let n_d_lo = e.at(BRACKET_LO).n_d;
    assert!(n_d_lo > 0.013 && n_d_lo < 0.015, "n_D(lo) = {n_d_lo}");
    match regime(params) {
        Regime::NoInteriorAtZero { f_at_0 } => close("f(1e-12)", f_at_0, G8_A_NEAR_1_F_AT_LO),
        other => panic!("expected NoInteriorAtZero, got {other:?}"),
    }
}

#[test]
fn huge_parameters_are_rejected() {
    // N = h = 1e300 with T = b = 1e-30 underflows T/h and v/P_s, so f(1) read 0 and the
    // regime BoundaryNoMargin, although the exact f(1) is -1.5e-30 and the economy is
    // NoInteriorAtZero (review of 2026-09-25). SCALE_CEIL rejects it.
    let err = Economy::new(Params {
        workers: 1e300,
        land: 1e-30,
        space: 1e300,
        b: 1e-30,
        ..appendix_b()
    })
    .unwrap_err();
    assert_eq!(err.name(), "workers");
    assert!(err.to_string().contains("SCALE_CEIL"), "{err}");
    for (name, params) in [
        (
            "space",
            Params {
                space: 1e300,
                ..appendix_b()
            },
        ),
        (
            "lam",
            Params {
                lam: 1e300,
                ..appendix_b()
            },
        ),
        (
            "rho",
            Params {
                rho: 1e300,
                ..appendix_b()
            },
        ),
    ] {
        assert_eq!(Economy::new(params).unwrap_err().name(), name);
    }
}

/// The review's economy of 2026-09-25 (G1 with N = 1) at curvature k.
fn steep(k: f64) -> Params {
    let base = appendix_b();
    Params {
        workers: 1.0,
        schedule: PowerSchedule { k, ..base.schedule },
        ..base
    }
}

#[test]
fn steep_schedules_are_rejected() {
    // At k = 1e20, x^k is 0 at the double below 1 and 1 at 1: gamma jumps from 0.2 to 1
    // across one double, and bisection closed on the jump. The solve returned Interior with
    // x* = 1 - 2^-53, gamma* = 0.2 and v = 0.1159, and a labour residual of 0.029 hours,
    // against 1 - x* = 2.55e-20, gamma(x*) = 0.2622 and v = 0.1527 from a 60-digit solve in
    // s = 1 - x (review of 2026-09-25). CURVATURE_CEIL now rejects it at validation.
    let line = "workers=1 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 g1=0.8 k=1e20 \
                chi_max=1 rho=0 delta=1 build_lag=1";
    assert_eq!(
        dump::line(line),
        "error=invalid parameter: k = 1e20 is out of range: must be in \
         [SCALE_FLOOR, CURVATURE_CEIL] = [1e-30, 1024]"
    );
    assert_eq!(dump::parse(line).unwrap(), steep(1e20));
    for k in [1e20, CURVATURE_CEIL.next_up()] {
        let err = Economy::new(steep(k)).expect_err("a k above CURVATURE_CEIL accepted");
        assert_eq!(err.name(), "k", "{err}");
    }
}

#[test]
fn solve_at_the_curvature_ceiling() {
    // The steepest schedule accepted, against generate.py's 70-digit solve. The root lies
    // in the steep part of x^1024, at 1 - x* = 0.002; across one double there gamma moves
    // by about k·2^-53·(g1·x^k/gamma) = 1024·2^-53·0.34 = 3.8e-14 relative.
    assert_eq!(CURVATURE_CEIL, 1024.0, "the goldens are at k = 1024");
    let e = economy(steep(CURVATURE_CEIL));
    let eq = interior(steep(CURVATURE_CEIL));
    for (name, got, want) in [
        ("x*", eq.x_star, G8_K_CEIL_X_STAR),
        ("1 - x*", eq.one_minus_x_star, G8_K_CEIL_ONE_MINUS_X_STAR),
        ("gamma(x*)", eq.gamma_star, G8_K_CEIL_GAMMA_STAR),
        ("J(x*)", eq.j_star, G8_K_CEIL_J_STAR),
        ("v", eq.v, G8_K_CEIL_V),
        ("p_m", eq.p_m, G8_K_CEIL_P_M),
        ("P_s", eq.p_s, G8_K_CEIL_P_S),
        ("Y", eq.y, G8_K_CEIL_Y),
        ("K", eq.k, G8_K_CEIL_K),
        ("final hours", eq.final_hours, G8_K_CEIL_FINAL_HOURS),
        ("N_a", eq.n_a, G8_K_CEIL_N_A),
        ("I", eq.income, G8_K_CEIL_INCOME),
        ("w / P_s", eq.real_wage, G8_K_CEIL_REAL_WAGE),
        (
            "provider baskets",
            eq.provider_baskets,
            G8_K_CEIL_PROVIDER_BASKETS,
        ),
    ] {
        close(name, got, want);
    }
    assert_root(&e, eq.x_star);
    assert!(
        eq.residuals.labor <= FULL * eq.n_a,
        "{:e}",
        eq.residuals.labor
    );
}

/// γ = c + s·x below x0 and C + s·x from x0 on: positive and strictly increasing, with
/// J its exact integral, but with a jump at x0. It breaks the `Schedule` contract
/// (continuity) and passes the sampled check, which cannot see a jump.
struct Jump {
    x0: f64,
    below: f64,
    above: f64,
    slope: f64,
}

impl Schedule for Jump {
    fn gamma(&self, x: f64) -> f64 {
        let level = if x < self.x0 { self.below } else { self.above };
        level + self.slope * x
    }

    fn integral(&self, x: f64) -> f64 {
        let ramp = self.slope * x * x / 2.0;
        if x < self.x0 {
            self.below * x + ramp
        } else {
            self.below * self.x0 + self.above * (x - self.x0) + ramp
        }
    }
}

#[test]
fn a_jump_in_excess_demand_trips_the_labour_residual_net() {
    // gamma jumps from 0.2875 to 1.0875 at x0 = 7/8, so n_S jumps up and f = n_D - n_S jumps
    // from +0.70 to -0.51 across the double below x0: a sign change with no root. Bisection
    // closes on it; the labour market at the result misses by about 0.4 of N_a, and the
    // solve must refuse it rather than return Interior.
    let x0 = 0.875;
    let e = Economy::new(Params {
        workers: 4.0,
        land: 10.0,
        space: 1.0,
        a: 0.3,
        lam: 0.05,
        b: 0.4,
        schedule: Jump {
            x0,
            below: 0.2,
            above: 1.0,
            slope: 0.1,
        },
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    })
    .expect("the sampled check passes a jump");
    let (before, after) = (e.at(x0.next_down()), e.at(x0));
    assert!(before.excess_demand() > 0.5 && after.excess_demand() < -0.5);
    match e.solve() {
        Err(SolveError::LaborNotCleared { x_star, relative }) => {
            assert!(x_star == x0 || x_star == x0.next_down(), "{x_star:?}");
            assert!(relative > 0.1, "{relative:e}");
            assert!(relative > 1e6 * LABOR_RESIDUAL_NET);
            let text = SolveError::LaborNotCleared { x_star, relative }.to_string();
            assert!(text.contains("LABOR_RESIDUAL_NET = 1e-9"), "{text}");
        }
        other => panic!("expected LaborNotCleared, got {other:?}"),
    }
}

#[test]
fn d_at_1_minus_infinity_is_not_viable() {
    // rho = 1e30 and J_b = 10 give u = 1e300, and lambda = 1e10 makes u lambda gamma(1)
    // overflow, so D(1) = 1 - u a - u lambda gamma(1) is -inf: certainly not viable, not
    // an undecidable value.
    let params = Params {
        lam: 1e10,
        rho: SCALE_CEIL,
        build_lag: 10,
        ..appendix_b()
    };
    match economy(params).solve() {
        Ok(Regime::NotViable { d_at_1 }) => assert_eq!(d_at_1, f64::NEG_INFINITY),
        other => panic!("expected NotViable, got {other:?}"),
    }
    let line = "workers=4 land=10 space=1 a=0.3 lam=1e10 b=0.4 eta=1 g0=0.2 g1=0.8 k=1 \
                chi_max=1 rho=1e30 delta=1 build_lag=10";
    assert_eq!(dump::line(line), "regime=NotViable d_at_1=-inf");
}

#[test]
fn non_finite_values_are_errors() {
    // Every parameter is at most SCALE_CEIL, but u = (rho + delta)(1 + rho)^(J_b - 1) is
    // not bounded: rho = 1e30 with J_b = 10 gives u = 1e300. With a = lambda = 0, D = 1
    // and p_m = u b = 1e300 b.
    let huge_u = |b: f64, land: f64| Params {
        a: 0.0,
        lam: 0.0,
        b,
        land,
        rho: SCALE_CEIL,
        build_lag: 10,
        ..appendix_b()
    };
    // b = 1e10: p_m = 1e310 overflows, so f(1) is NaN and no regime can be decided.
    match economy(huge_u(1e10, 10.0)).solve() {
        Err(SolveError::NonFinite { what }) => assert_eq!(what, "n_D(1) - n_S(1)"),
        other => panic!("expected NonFinite, got {other:?}"),
    }
    // b = 0.4 and T = 1e10: prices near 4e299 are finite and so is the bracket, so the root
    // is found, but interest = rho W_K is about u b K = 4e299 * 5e9 and overflows, and
    // income with it. The solve must not return it as Interior.
    let e = economy(huge_u(0.4, 1e10));
    assert!(e.at(1.0).excess_demand().is_finite());
    match e.solve() {
        Err(SolveError::NonFinite { what }) => {
            assert_eq!(what, "income");
            assert_eq!(
                SolveError::NonFinite { what }.to_string(),
                "income is not finite"
            );
        }
        other => panic!("expected NonFinite, got {other:?}"),
    }
}

#[test]
fn names() {
    assert_eq!(regime(appendix_b()).name(), "Interior");
    assert_eq!(
        regime(Params {
            lam: 0.8,
            ..appendix_b()
        })
        .name(),
        "NotViable"
    );
    assert!(regime(Params {
        lam: 0.8,
        ..appendix_b()
    })
    .interior()
    .is_none());
}
