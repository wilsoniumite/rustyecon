//! C2: the CES expenditure share of SSRN eq 26 (p.31), price block only (docs/unit-1b.md
//! §4.5). It is evaluated on prices and feeds nothing back. The parameters α = 0.3 and
//! σ ∈ {0.5, 2} are constructed; the paper gives none.

use oracle::{ces_share, SCALE_CEIL};

use crate::c1_nesting::{g3_path, G3_PATH};
use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;

/// The weight on the rented service.
const ALPHA: f64 = 0.3;

#[test]
fn values_on_the_g3_path() {
    // q = r/p of the good at each equilibrium of G3's path, from the solve.
    let goldens = [
        (C2_ETA_1_CES_HALF, C2_ETA_1_CES_TWO),
        (C2_ETA_0_3_CES_HALF, C2_ETA_0_3_CES_TWO),
        (C2_ETA_0_1_CES_HALF, C2_ETA_0_1_CES_TWO),
        (C2_ETA_0_03_CES_HALF, C2_ETA_0_03_CES_TWO),
        (C2_ETA_0_01_CES_HALF, C2_ETA_0_01_CES_TWO),
    ];
    let (mut last_half, mut last_two) = (0.0, f64::INFINITY);
    for ((eta, _, _, _), (half, two)) in G3_PATH.into_iter().zip(goldens) {
        let eq = interior_1b(g3_path(eta));
        let q = 1.0 / eq.categories[0].price;
        let got_half = ces_share(ALPHA, 0.5, q).unwrap();
        let got_two = ces_share(ALPHA, 2.0, q).unwrap();
        close(&format!("sigma 0.5 at eta = {eta}"), got_half, half);
        close(&format!("sigma 2 at eta = {eta}"), got_two, two);
        // As q grows the share rises toward 1 for sigma < 1 and falls toward 0 for
        // sigma > 1 (SSRN p.31).
        assert!(got_half > last_half && got_two < last_two, "eta = {eta}");
        (last_half, last_two) = (got_half, got_two);
    }
}

#[test]
fn values_at_q_one() {
    // At q = 1 the share is 1/(1 + ((1 - alpha)/alpha)^sigma): 1/2 at sigma 0, alpha at
    // sigma 1, 9/58 at sigma 2.
    for (sigma, want) in [
        (0.0, C2_Q1_CES_ZERO),
        (0.5, C2_Q1_CES_HALF),
        (1.0, C2_Q1_CES_ONE),
        (2.0, C2_Q1_CES_TWO),
    ] {
        close(
            &format!("sigma {sigma}"),
            ces_share(ALPHA, sigma, 1.0).unwrap(),
            want,
        );
    }
    assert_eq!(ces_share(ALPHA, 0.0, 1.0).unwrap(), 0.5);
    close("9/58", C2_Q1_CES_TWO, 9.0 / 58.0);
    close("alpha", C2_Q1_CES_ONE, ALPHA);
    // At sigma 0 the service and the good are consumed in fixed proportions, and the
    // share is 1/(1 + 1/q) = q/(1 + q) whatever alpha is.
    for q in [1e-3, 0.5, 7.0, 1e6] {
        close(
            &format!("sigma 0 at q = {q}"),
            ces_share(0.9, 0.0, q).unwrap(),
            q / (1.0 + q),
        );
    }
}

#[test]
fn unit_elasticity_gives_alpha() {
    // Cobb-Douglas: the share is alpha at every price.
    for alpha in [0.3, 0.05, 0.9] {
        for q in [1e-6, 1.0, 1e6] {
            let share = ces_share(alpha, 1.0, q).unwrap();
            assert!(
                (share - alpha).abs() <= 1e-15,
                "alpha {alpha}, q {q}: {share:e}"
            );
        }
    }
}

#[test]
fn limits_and_monotonicity() {
    // SSRN p.31: as q -> infinity the share tends to 1 for sigma < 1 and to 0 for
    // sigma > 1.
    assert!(ces_share(ALPHA, 0.5, 1e12).unwrap() > 1.0 - 1e-5);
    assert!(ces_share(ALPHA, 2.0, 1e12).unwrap() < 1e-5);
    // Monotone in q on a grid: rising for sigma < 1, falling for sigma > 1, flat at 1. The
    // grid is ln q from -20 to 20 in steps of 0.5, narrower at sigma 7.5, where the share
    // rounds to 1 for q below e^-5.
    for (sigma, half_width) in [(0.0, 40), (0.5, 40), (1.0, 40), (2.0, 40), (7.5, 8)] {
        let mut last = None;
        for i in -half_width..=half_width {
            let q = rustyecon_core::num::exp(f64::from(i) * 0.5);
            let share = ces_share(ALPHA, sigma, q).unwrap();
            assert!((0.0..=1.0).contains(&share), "sigma {sigma}, q {q}");
            if let Some(prev) = last {
                match sigma {
                    s if s < 1.0 => assert!(share > prev, "sigma {sigma} at q = {q}"),
                    s if s > 1.0 => assert!(share < prev, "sigma {sigma} at q = {q}"),
                    _ => assert_eq!(share, prev, "sigma 1 at q = {q}"),
                }
            }
            last = Some(share);
        }
    }
    // Extreme prices give a share in [0, 1], never NaN: the form cannot overflow.
    for q in [1e300, f64::MAX, 5e-324, 1e-300] {
        for sigma in [0.0, 0.5, 2.0, 1e3, SCALE_CEIL] {
            let share = ces_share(ALPHA, sigma, q).unwrap();
            assert!(
                (0.0..=1.0).contains(&share),
                "sigma {sigma}, q {q:e}: {share}"
            );
        }
    }
    assert_eq!(ces_share(ALPHA, 2.0, f64::MAX).unwrap(), 0.0);
    assert_eq!(ces_share(ALPHA, 0.5, f64::MAX).unwrap(), 1.0);
}

#[test]
fn rejects_bad_arguments() {
    for (alpha, sigma, q, name) in [
        (0.0, 0.5, 1.0, "alpha"),
        (1.0, 0.5, 1.0, "alpha"),
        (-0.2, 0.5, 1.0, "alpha"),
        (f64::NAN, 0.5, 1.0, "alpha"),
        (0.3, -0.1, 1.0, "sigma"),
        (0.3, f64::NAN, 1.0, "sigma"),
        (0.3, SCALE_CEIL * 2.0, 1.0, "sigma"),
        (0.3, 0.5, 0.0, "q"),
        (0.3, 0.5, -1.0, "q"),
        (0.3, 0.5, f64::INFINITY, "q"),
        (0.3, 0.5, f64::NAN, "q"),
    ] {
        match ces_share(alpha, sigma, q) {
            Err(e) => {
                assert_eq!(e.name(), name, "({alpha}, {sigma}, {q})");
                assert!(e.to_string().starts_with(name), "{e}");
            }
            other => panic!("({alpha}, {sigma}, {q}) gave {other:?}"),
        }
    }
    let err = ces_share(1.0, 0.5, 1.0).unwrap_err();
    assert_eq!(
        err.to_string(),
        "alpha = 1.0 is out of range: must satisfy 0 < value < 1"
    );
}
