//! G2: SSRN Figure 3 (p.12). The caption publishes each equilibrium's w/P_s and N_a/N to
//! two decimals; laborformal has no script for the figure.

use oracle::{Params, PowerSchedule};

use crate::goldens::*;
use crate::support::*;

/// Recursive automation: no labour in machine services.
fn lambda_zero() -> Params {
    Params {
        lam: 0.0,
        ..appendix_b()
    }
}

/// Task automation: γ halved at every task.
fn gamma_halved() -> Params {
    let base = appendix_b();
    Params {
        schedule: PowerSchedule {
            eta: 0.5,
            ..base.schedule
        },
        ..base
    }
}

#[test]
fn baseline() {
    let eq = interior(appendix_b());
    near(
        "w / P_s caption",
        eq.real_wage,
        PUB_G2_BASE_REAL_WAGE,
        CAPTION,
    );
    near(
        "N_a / N caption",
        eq.participation,
        PUB_G2_BASE_PARTICIPATION,
        CAPTION,
    );
}

#[test]
fn recursive_automation() {
    let eq = interior(lambda_zero());
    close("x*", eq.x_star, G2_LAM0_X_STAR);
    close("v", eq.v, G2_LAM0_V);
    close("Y", eq.y, G2_LAM0_Y);
    close("N_a", eq.n_a, G2_LAM0_N_A);
    close("w / P_s", eq.real_wage, G2_LAM0_REAL_WAGE);
    close("N_a / N", eq.participation, G2_LAM0_PARTICIPATION);
    near(
        "w / P_s caption",
        eq.real_wage,
        PUB_G2_LAM0_REAL_WAGE,
        CAPTION,
    );
    near(
        "N_a / N caption",
        eq.participation,
        PUB_G2_LAM0_PARTICIPATION,
        CAPTION,
    );
    assert_eq!(eq.machine_hours, 0.0);
}

#[test]
fn task_automation() {
    let eq = interior(gamma_halved());
    close("x*", eq.x_star, G2_HALF_X_STAR);
    close("v", eq.v, G2_HALF_V);
    close("Y", eq.y, G2_HALF_Y);
    close("N_a", eq.n_a, G2_HALF_N_A);
    close("w / P_s", eq.real_wage, G2_HALF_REAL_WAGE);
    close("N_a / N", eq.participation, G2_HALF_PARTICIPATION);
    near(
        "w / P_s caption",
        eq.real_wage,
        PUB_G2_HALF_REAL_WAGE,
        CAPTION,
    );
    near(
        "N_a / N caption",
        eq.participation,
        PUB_G2_HALF_PARTICIPATION,
        CAPTION,
    );
}

#[test]
fn both_channels_lower_the_real_wage_and_participation() {
    // SSRN p.12: in this example each automation channel lowers w/P_s and N_a/N.
    let base = interior(appendix_b());
    for moved in [interior(lambda_zero()), interior(gamma_halved())] {
        assert!(moved.real_wage < base.real_wage);
        assert!(moved.participation < base.participation);
    }
}
