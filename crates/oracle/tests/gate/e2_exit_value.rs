//! e2: the exit value alone (docs/unit-1e.md §4.2): check_pinning P3, check_enclosure N-ii,
//! N-iii and N-vi, SSRN D.3's coverage, and the identical-workers limit of §2.3.

use oracle::{
    coverage, coverage_threshold, crowding_limit, ParamError, PricedExit, UniformWorkCost,
    BRACKET_HI,
};

use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1e::*;

fn form(gross: f64, floor: f64, plot: f64) -> PricedExit {
    PricedExit { gross, floor, plot }
}

#[test]
fn p3_floor() {
    // check_pinning.py:131-145: (s₀, s̲, h) = (10, 4, 6) meets the floor exactly at q_enc = 1,
    // is weakly decreasing on (1/3, 2/3, 1, 2) and 4 above q_enc.
    let e = form(10.0, 4.0, 6.0);
    assert_eq!(e.threshold().unwrap(), Some(P3_Q_ENC));
    assert_eq!(e.value(P3_Q_ENC).unwrap(), 4.0);
    let grid = [1.0 / 3.0, 2.0 / 3.0, 1.0, 2.0];
    let values: Vec<f64> = grid.iter().map(|&q| e.value(q).unwrap()).collect();
    for (v, want) in values.iter().zip([8.0, 6.0, 4.0, 4.0]) {
        close("s(q)", *v, want);
    }
    assert!(values.windows(2).all(|w| w[1] <= w[0]));
    assert_eq!(e.value(5.0).unwrap(), 4.0);
    assert_eq!(e.value(0.0).unwrap(), 10.0);
}

#[test]
fn slope_and_cap() {
    // N-ii: ds/dq = −h below q_enc, exactly on dyadic q; 0 above. N-vi: the take s₀ − s(q) is
    // q·h below q_enc and h·q_enc above it.
    let e = form(1.5, 0.0, 1.0);
    let q_enc = e.threshold().unwrap().unwrap();
    assert_eq!(q_enc, P_Q_ENC);
    let step = pow2(-10);
    for i in 0..1400 {
        let q = f64::from(i) * step;
        let (a, b) = (e.value(q).unwrap(), e.value(q + step).unwrap());
        if q + step <= q_enc {
            assert_eq!(a - b, step, "slope at {q}");
            assert_eq!(e.take(q).unwrap(), q);
        } else if q >= q_enc {
            assert_eq!(a, b, "flat at {q}");
            assert_eq!(e.take(q).unwrap(), q_enc);
        }
    }
    let n_vi = form(1.5, 0.1, 1.0);
    assert_eq!(n_vi.take(1.0).unwrap(), 1.0);
    for q in [1.4, 2.0, 10.0, 1e6] {
        close("the cap h q_enc", n_vi.take(q).unwrap(), P_TAKE_CAP);
    }
}

#[test]
fn race_closed_forms() {
    // check_enclosure N-iii (:56-80), main.tex:841-862 and SSRN eq 28 at T 100, g_s = h_s = 1.
    let q_enc = form(1.5, 0.0, 1.0).threshold().unwrap().unwrap();
    assert_eq!(crowding_limit(q_enc, 100.0, 1.0, 1.0).unwrap(), P_N_CRIT);
    for (people, want) in [
        (50.0, P_Q_STAR_50),
        (60.0, P_Q_STAR_60),
        (80.0, P_Q_STAR_80),
    ] {
        assert_eq!(
            coverage_threshold(100.0, people, 1.0, 1.0).unwrap(),
            Some(want),
            "q* at N {people}"
        );
        // κ(q*) = 1, and κ < 1 below it
        close(
            "κ(q*)",
            coverage(want, 100.0, people, 1.0, 1.0).unwrap(),
            1.0,
        );
        assert!(coverage(want * 0.99, 100.0, people, 1.0, 1.0).unwrap() < 1.0);
    }
    // the race: q_enc ≥ q* exactly when N ≤ N_crit
    for people in [50.0, 60.0, 80.0] {
        let q_star = coverage_threshold(100.0, people, 1.0, 1.0)
            .unwrap()
            .unwrap();
        assert_eq!(q_enc >= q_star, people <= P_N_CRIT, "N {people}");
    }
    close(
        "SSRN D.3",
        coverage(10.0 / 9.0, 100.0, 50.0, 1.0, 1.0).unwrap(),
        P_KAPPA_D3,
    );
    // 120 are never covered (T ≤ N·h_s)
    assert_eq!(coverage_threshold(100.0, 120.0, 1.0, 1.0).unwrap(), None);
    assert_eq!(coverage_threshold(100.0, 100.0, 1.0, 1.0).unwrap(), None);
    // κ rises with q toward T/(N·h_s)
    let mut last = 0.0;
    for i in 1..=60 {
        let q = f64::from(i) * 0.5;
        let k = coverage(q, 100.0, 120.0, 1.0, 1.0).unwrap();
        assert!(k > last && k < 100.0 / 120.0);
        last = k;
    }
    close_to(
        "κ → T/(N h_s)",
        coverage(1e12, 100.0, 120.0, 1.0, 1.0).unwrap(),
        100.0 / 120.0,
        1e-11,
    );
}

#[test]
fn identical_workers_ignore_support() {
    // As χ_max → 0 a worker works exactly when v ≥ e (main.tex:396; N-iv's transfer cancels):
    // at K3's wall, with χ_max 1e-12, the type supplies N or 0 on either side of its exit
    // value, whatever its support ν.
    for support in [0.1, 1.0, 10.0] {
        let mut p = k3();
        p.worker_types[0].work_cost = UniformWorkCost { chi_max: 1e-12 };
        p.worker_types[0].support = support;
        let e = economy_1e(p);
        let technique = e.workers().machines().envelope().last();
        let mut seen = (false, false);
        let mut v = 0.01;
        for _ in 0..400 {
            v *= 1.02;
            let q = e.at_wage(BRACKET_HI, v, technique);
            let (e_i, n) = (q.exit_values[0], q.point.supply[0]);
            let gap = (v - e_i) / (support * q.point.p_s + e_i);
            if gap > 1e-9 {
                assert_eq!(n, 4.0, "ν {support} v {v}: works");
                seen.0 = true;
            } else if gap < 0.0 {
                assert_eq!(n, 0.0, "ν {support} v {v}: exits");
                seen.1 = true;
            }
        }
        assert_eq!(seen, (true, true), "ν {support}");
    }
}

#[test]
fn bad_arguments() {
    let e = form(1.5, 0.0, 1.0);
    for q in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(e.value(q).unwrap_err().name(), "q");
        assert_eq!(e.take(q).unwrap_err().name(), "q");
    }
    for (bad, name) in [
        (form(-1.0, 0.0, 1.0), "gross"),
        (form(1.5, f64::NAN, 1.0), "floor"),
        (form(1.5, 0.0, 1e31), "plot"),
        (form(1e-31, 0.0, 1.0), "gross"),
    ] {
        assert_eq!(bad.value(1.0).unwrap_err().name(), name);
        assert_eq!(bad.threshold().unwrap_err().name(), name);
    }
    let checks: [(Result<f64, ParamError>, &str); 6] = [
        (coverage(-0.5, 100.0, 50.0, 1.0, 1.0), "q"),
        (coverage(1.0, 0.0, 50.0, 1.0, 1.0), "land"),
        (coverage(1.0, 100.0, f64::NAN, 1.0, 1.0), "people"),
        (coverage(1.0, 100.0, 50.0, -1.0, 1.0), "goods"),
        (coverage(1.0, 100.0, 50.0, 1.0, f64::INFINITY), "space"),
        (crowding_limit(1.5, 100.0, 0.0, 0.0), "basket"),
    ];
    for (result, name) in checks {
        assert_eq!(result.unwrap_err().name(), name);
    }
    assert_eq!(
        coverage_threshold(1e31, 50.0, 1.0, 1.0).unwrap_err().name(),
        "land"
    );
    assert_eq!(
        crowding_limit(f64::NAN, 100.0, 1.0, 1.0)
            .unwrap_err()
            .name(),
        "q_enc"
    );
}
