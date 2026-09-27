//! C4: the fork economy along the paths (docs/unit-1b.md §7): task automation, which
//! lowers η, and recursive automation, which lowers λ. Along task automation the wage in
//! manufactures (no direct land) rises and the wage in shelter falls with v: the fork
//! (main.tex:491-504).

use oracle::{CategoryParams, PowerSchedule};

use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;

/// (η, x*, v, v/p per category, v/P_s) along task automation.
const TASK: [(f64, f64, f64, [f64; 4], f64); 5] = [
    (
        1.0,
        C4_ETA_1_X_STAR,
        C4_ETA_1_V,
        [
            C4_ETA_1_MANUFACTURES_REAL_WAGE,
            C4_ETA_1_FOOD_REAL_WAGE,
            C4_ETA_1_CARE_REAL_WAGE,
            C4_ETA_1_SHELTER_REAL_WAGE,
        ],
        C4_ETA_1_REAL_WAGE,
    ),
    (
        0.5,
        C4_ETA_0_5_X_STAR,
        C4_ETA_0_5_V,
        [
            C4_ETA_0_5_MANUFACTURES_REAL_WAGE,
            C4_ETA_0_5_FOOD_REAL_WAGE,
            C4_ETA_0_5_CARE_REAL_WAGE,
            C4_ETA_0_5_SHELTER_REAL_WAGE,
        ],
        C4_ETA_0_5_REAL_WAGE,
    ),
    (
        0.25,
        C4_ETA_0_25_X_STAR,
        C4_ETA_0_25_V,
        [
            C4_ETA_0_25_MANUFACTURES_REAL_WAGE,
            C4_ETA_0_25_FOOD_REAL_WAGE,
            C4_ETA_0_25_CARE_REAL_WAGE,
            C4_ETA_0_25_SHELTER_REAL_WAGE,
        ],
        C4_ETA_0_25_REAL_WAGE,
    ),
    (
        0.1,
        C4_ETA_0_1_X_STAR,
        C4_ETA_0_1_V,
        [
            C4_ETA_0_1_MANUFACTURES_REAL_WAGE,
            C4_ETA_0_1_FOOD_REAL_WAGE,
            C4_ETA_0_1_CARE_REAL_WAGE,
            C4_ETA_0_1_SHELTER_REAL_WAGE,
        ],
        C4_ETA_0_1_REAL_WAGE,
    ),
    (
        0.03,
        C4_ETA_0_03_X_STAR,
        C4_ETA_0_03_V,
        [
            C4_ETA_0_03_MANUFACTURES_REAL_WAGE,
            C4_ETA_0_03_FOOD_REAL_WAGE,
            C4_ETA_0_03_CARE_REAL_WAGE,
            C4_ETA_0_03_SHELTER_REAL_WAGE,
        ],
        C4_ETA_0_03_REAL_WAGE,
    ),
];

/// (λ, x*, v, v/p per category, v/P_s) along recursive automation, after λ = 0.05 (C3).
const RECURSIVE: [(f64, f64, f64, [f64; 4], f64); 2] = [
    (
        0.025,
        C4_LAM_0_025_X_STAR,
        C4_LAM_0_025_V,
        [
            C4_LAM_0_025_MANUFACTURES_REAL_WAGE,
            C4_LAM_0_025_FOOD_REAL_WAGE,
            C4_LAM_0_025_CARE_REAL_WAGE,
            C4_LAM_0_025_SHELTER_REAL_WAGE,
        ],
        C4_LAM_0_025_REAL_WAGE,
    ),
    (
        0.0,
        C4_LAM_0_X_STAR,
        C4_LAM_0_V,
        [
            C4_LAM_0_MANUFACTURES_REAL_WAGE,
            C4_LAM_0_FOOD_REAL_WAGE,
            C4_LAM_0_CARE_REAL_WAGE,
            C4_LAM_0_SHELTER_REAL_WAGE,
        ],
        C4_LAM_0_REAL_WAGE,
    ),
];

fn with_eta(eta: f64) -> CategoryParams {
    let base = fork_economy();
    CategoryParams {
        schedule: PowerSchedule {
            eta,
            ..base.schedule
        },
        ..base
    }
}

fn check_row(
    what: &str,
    params: CategoryParams,
    (x_star, v, wages, real_wage): (f64, f64, [f64; 4], f64),
) -> Box<oracle::Eq1b> {
    let e = economy_1b(params.clone());
    let eq = interior_1b(params);
    close(&format!("x* at {what}"), eq.x_star, x_star);
    close(&format!("v at {what}"), eq.v, v);
    close(&format!("v/P_s at {what}"), eq.real_wage, real_wage);
    for ((name, c), want) in FORK_NAMES.iter().zip(&eq.categories).zip(wages) {
        close(&format!("v/p of {name} at {what}"), c.real_wage, want);
    }
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
    eq
}

#[test]
fn task_automation() {
    let mut path = Vec::new();
    for (eta, x_star, v, wages, real_wage) in TASK {
        path.push(check_row(
            &format!("eta = {eta}"),
            with_eta(eta),
            (x_star, v, wages, real_wage),
        ));
    }
    for w in path.windows(2) {
        let (earlier, later) = (&w[0], &w[1]);
        // v and the wage in shelter fall; the wage in manufactures rises (strictly, with
        // room at every step).
        assert!(later.v < earlier.v * (1.0 - 1e-3));
        assert!(later.categories[3].real_wage < earlier.categories[3].real_wage * (1.0 - 1e-3));
        assert!(later.categories[0].real_wage > earlier.categories[0].real_wage * (1.0 + 1e-3));
        // Automation moves the margin up the line.
        assert!(later.x_star > earlier.x_star);
    }
    for eq in &path {
        // Manufactures has no direct land: its wage stays above 1/L-bar = 1.25.
        let m = &eq.categories[0];
        assert!(m.real_wage >= 1.25 && m.wage_floor == 1.0 / 0.8);
        // Shelter's wage is bounded by v/b = v/1 (SSRN eq 13's ceiling with its direct
        // land).
        assert!(eq.categories[3].real_wage < eq.v);
    }
    // The fork's size: manufactures up 27%, shelter down 96% (docs/unit-1b.md §7 C4).
    let (first, last) = (&path[0], &path[path.len() - 1]);
    let rise = last.categories[0].real_wage / first.categories[0].real_wage - 1.0;
    let fall = 1.0 - last.categories[3].real_wage / first.categories[3].real_wage;
    assert!(0.26 < rise && rise < 0.28, "{rise}");
    assert!(0.95 < fall && fall < 0.97, "{fall}");
}

#[test]
fn recursive_automation() {
    // SSRN §3.4: lowering lambda lowers x* and v here, and every v/p_j with them.
    let mut path = vec![interior_1b(fork_economy())];
    close("x* at lambda 0.05", path[0].x_star, C3_X_STAR);
    for (lam, x_star, v, wages, real_wage) in RECURSIVE {
        path.push(check_row(
            &format!("lambda = {lam}"),
            CategoryParams {
                lam,
                ..fork_economy()
            },
            (x_star, v, wages, real_wage),
        ));
    }
    for w in path.windows(2) {
        let (earlier, later) = (&w[0], &w[1]);
        assert!(later.x_star < earlier.x_star && later.v < earlier.v);
        for (a, b) in earlier.categories.iter().zip(&later.categories) {
            assert!(b.real_wage < a.real_wage);
        }
    }
    // At lambda = 0 machines use no labour: the machine sector's hours and totals vanish.
    let last = &path[path.len() - 1];
    assert_eq!(last.machine_hours, 0.0);
    assert_eq!(last.lambda_tilde_machine, 0.0);
    assert_eq!(last.phi_w, Some(0.0));
}
