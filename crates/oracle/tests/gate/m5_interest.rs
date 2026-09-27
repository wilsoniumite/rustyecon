//! m5: interest selects the technique (docs/unit-1c.md §3.3, M5): a flow type that does not
//! see ρ against a durable type whose user cost rises steeply with it; uniqueness at ρ = 0;
//! and the refusal of multiple equilibria (M5m).

use oracle::{MachineParams, Recipe, SolveError, UniformWorkCost};

use crate::goldens_1c::*;
use crate::support::*;
use crate::support_1c::*;

#[test]
fn interest_selects_the_technique() {
    // docs/unit-1c.md §7's ρ table: below the switch rate the durable type is at the margin
    // and the wage rises with ρ, because the machine the wage is pinned to costs more; above
    // it the flow type takes the margin and ρ drops out.
    let rows = [
        (
            0.0,
            1,
            M5_RHO0_X_STAR,
            M5_RHO0_V,
            M5_RHO0_N_A,
            0.0,
            M5_RHO0_INCOME,
        ),
        (
            0.05,
            1,
            M5_RHO0_05_X_STAR,
            M5_RHO0_05_V,
            M5_RHO0_05_N_A,
            M5_RHO0_05_INTEREST,
            M5_RHO0_05_INCOME,
        ),
        (
            0.1,
            1,
            M5_RHO0_1_X_STAR,
            M5_RHO0_1_V,
            M5_RHO0_1_N_A,
            M5_RHO0_1_INTEREST,
            M5_RHO0_1_INCOME,
        ),
        (
            0.15,
            0,
            M5_RHO0_15_X_STAR,
            M5_RHO0_15_V,
            M5_RHO0_15_N_A,
            0.0,
            M5_RHO0_15_INCOME,
        ),
        (
            0.3,
            0,
            M5_RHO0_3_X_STAR,
            M5_RHO0_3_V,
            M5_RHO0_3_N_A,
            0.0,
            M5_RHO0_3_INCOME,
        ),
    ];
    let mut last_v = 0.0;
    for (rho, technique, x_star, v, n_a, interest, income) in rows {
        let params = m5(rho, 4.0, 1.0);
        let eq = interior_1c(params.clone());
        assert_eq!(eq.technique, technique, "rho {rho}");
        close("x*", eq.x_star, x_star);
        close("v", eq.v, v);
        close("N_a", eq.n_a, n_a);
        close("I", eq.income, income);
        if interest == 0.0 {
            assert_eq!(eq.interest, 0.0, "rho {rho}");
        } else {
            close("interest", eq.interest, interest);
        }
        if technique == 1 {
            assert!(
                eq.v > last_v,
                "the wage rises with rho while the durable type binds"
            );
            last_v = eq.v;
        }
        let e = economy_1c(params);
        check_identities_1c(&e, &eq);
        check_fork_and_bounds_1c(&e, &eq);
    }
    // At ρ = 0.1 the flow type does the tasks below the switch.
    let eq = interior_1c(m5(0.1, 4.0, 1.0));
    assert_eq!(eq.switches.len(), 1);
    close("the switch", eq.switches[0].x, M5_RHO0_1_SWITCH_X);
    assert!(eq.switches[0].x < eq.x_star);
    assert_eq!((eq.switches[0].below, eq.switches[0].above), (0, 1));
    // Above the switch rate the flow economy does not see ρ: every output is bit-equal at 0.15
    // and 0.3 except u (the flow type's is 1 + ρ, though it prices nothing), the unused
    // durable type's own outputs, and the residuals that are maxima over types.
    let (a, b) = (
        interior_1c(m5(0.15, 4.0, 1.0)),
        interior_1c(m5(0.3, 4.0, 1.0)),
    );
    let (bits_a, bits_b) = (bits_1c(&a), bits_1c(&b));
    let mut compared = 0;
    for (key, value) in &bits_a {
        let excepted = key == "u"
            || key == "type0.user_cost"
            || key.starts_with("type1.")
            || key == "res_user_cost"
            || key == "res_leontief_price";
        if excepted {
            continue;
        }
        assert_eq!(value, &bits_b[key], "{key}");
        compared += 1;
    }
    assert!(compared > 100, "{compared}");
    assert_eq!(a.u, 1.15);
    assert_eq!(b.u, 1.3);
}

#[test]
fn rho_zero_is_unique() {
    // docs/unit-1c.md §5.5's Proposition: at ρ = 0, u = δ, and at a switch from a to b the
    // incoming type uses less labour and more land per task, so labour demand falls.
    let mut switches = 0;
    let cases: Vec<MachineParams> = vec![
        MachineParams {
            rho: 0.0,
            ..m4(1.0)
        },
        MachineParams {
            rho: 0.0,
            ..m4(0.5)
        },
        MachineParams {
            rho: 0.0,
            ..m4(2.0)
        },
        MachineParams {
            rho: 0.0,
            workers: 8.0,
            ..m4(0.5)
        },
        m5(0.0, 4.0, 1.0),
        m5(0.0, 60.0, 0.2),
    ];
    for params in cases {
        let e = economy_1c(params.clone());
        for (s, x) in e.envelope().switches.iter().zip(e.switch_points().unwrap()) {
            let (below, above) = (e.at_with(x, s.below), e.at_with(x, s.above));
            assert!(above.n_d < below.n_d, "{params:?}");
            switches += 1;
        }
        // One equilibrium: a root, a tie or a boundary regime's corner.
        let changes = check_regions(&e);
        match e.solve() {
            Ok(_) => assert_eq!(changes, 1, "{params:?}"),
            Err(error) => panic!("{params:?}: {error}"),
        }
    }
    assert!(switches >= 2, "{switches}");
    // The random switch set at ρ = 0 is m6's (m6::ties_and_multiplicity).
}

#[test]
fn multiple_equilibria_are_refused() {
    // docs/unit-1c.md §2.5 and §5.3, M5m: with interest the switch to the durable type raises
    // labour demand, and there are three equilibria. The solve counts the sign changes and
    // refuses.
    let params = m5(0.1, 60.0, 0.2);
    let e = economy_1c(params);
    let x_sw = e.switch_points().unwrap()[0];
    close("the switch", x_sw, M5M_SWITCH_X);
    match e.solve() {
        Err(SolveError::MultipleEquilibria {
            sign_changes,
            switches,
        }) => {
            assert_eq!(sign_changes, 3);
            assert_eq!(switches, vec![x_sw]);
        }
        other => panic!("expected MultipleEquilibria, got {other:?}"),
    }
    // The four values of the sign sequence.
    close(
        "f(1e-12)",
        e.at_with(oracle::BRACKET_LO, 0).excess_demand(),
        M5M_F_AT_LO,
    );
    close(
        "f flow at the switch",
        e.at_with(x_sw, 0).excess_demand(),
        M5M_F_FLOW_AT_SWITCH,
    );
    close(
        "f durable at the switch",
        e.at_with(x_sw, 1).excess_demand(),
        M5M_F_DURABLE_AT_SWITCH,
    );
    close("f(1)", e.at_with(1.0, 1).excess_demand(), M5M_F_AT_1);
    assert_eq!(check_regions(&e), 3);
    // The three equilibria the generator records exist: a root of each technique in its
    // region, and a tie's split in (0, 1).
    let f_flow = |x: f64| e.at_with(x, 0).excess_demand();
    let f_durable = |x: f64| e.at_with(x, 1).excess_demand();
    assert!(
        f_flow(M5M_FLOW_ROOT * (1.0 - 1e-9)) > 0.0 && f_flow(M5M_FLOW_ROOT * (1.0 + 1e-9)) < 0.0
    );
    assert!(M5M_FLOW_ROOT < x_sw && x_sw < M5M_DURABLE_ROOT);
    assert!(
        f_durable(M5M_DURABLE_ROOT * (1.0 - 1e-9)) > 0.0
            && f_durable(M5M_DURABLE_ROOT * (1.0 + 1e-9)) < 0.0
    );
    // The tie's share, from both techniques' quantities at the switch and the flow type's
    // prices (docs/unit-1c.md §4.7): σ = B_a·f_a/(B_a·f_a − B_b·f_b).
    let (flow, durable) = (e.at_with(x_sw, 0), e.at_with(x_sw, 1));
    let land = e.params().land;
    let (b_a, b_b) = (land / flow.y, land / durable.y);
    let (f_a, f_b) = (flow.n_d - flow.n_s, durable.n_d - flow.n_s);
    let share = b_a * f_a / (b_a * f_a - b_b * f_b);
    close("the tie's share", share, M5M_TIE_SHARE);
    assert!(share > 0.0 && share < 1.0);
    let text = SolveError::MultipleEquilibria {
        sign_changes: 3,
        switches: vec![x_sw],
    }
    .to_string();
    assert!(
        text.contains("3 times") && text.contains("more than one equilibrium"),
        "{text}"
    );
    assert!(!text.contains("  "), "{text}");
}

/// H1 and H4 of the second verification (2026-09-27; docs/unit-1c.md §12 item 14): the good and
/// space (h 0.05) on γ = 3(0.2 + 0.8x), N 15, ρ 0.1, and two task types: a flow type, operating
/// (λ 0.3, land 1), δ 1, J 1; and a type run on labour (operating λ 0.25) and built from land
/// (9), δ 0.01, J 3, whose u is far above its δ. Cheap above the switch on the price side, it
/// uses little land per period, so labour demand jumps up there.
pub(crate) fn hidden(chi_max: f64) -> MachineParams {
    let flow = machine_type(1.0, recipe(&[0.0, 0.0], 0.3, 1.0), Recipe::zero(2), 1.0, 1);
    let durable = machine_type(
        1.0,
        recipe(&[0.0, 0.0], 0.25, 0.0),
        recipe(&[0.0, 0.0], 0.0, 9.0),
        0.01,
        3,
    );
    MachineParams {
        workers: 15.0,
        work_cost: UniformWorkCost { chi_max },
        ..appendix_b_household(linear(3.0, 0.2, 0.8), 0.1, 0.05, vec![flow, durable])
    }
}

#[test]
fn an_equilibrium_behind_the_boundary_is_refused() {
    // docs/unit-1c.md §5.3: f(1) ≥ 0 is BoundaryNoMargin only when the sign sequence has no
    // change of side inside the bracket. Here f is +, −, +, + (f(lo), the flow type's f at the
    // switch, the durable type's there, f(1)): a root below the switch, a tie on the upward
    // jump, and the boundary. Three equilibria, refused; before 2026-09-27 the solve returned
    // BoundaryNoMargin from f(1) alone.
    for chi_max in [1.0, 0.01] {
        let e = economy_1c(hidden(chi_max));
        let env = e.envelope();
        assert_eq!((env.first, env.switches.len(), env.last()), (0, 1, 1));
        let x_sw = e.switch_points().unwrap()[0];
        let f = |x: f64, t: usize| e.at_with(x, t).excess_demand();
        let (lo, flow_at, durable_at, one) =
            (f(oracle::BRACKET_LO, 0), f(x_sw, 0), f(x_sw, 1), f(1.0, 1));
        assert!(
            lo > 0.0 && flow_at < 0.0 && durable_at > 0.0 && one > 0.0,
            "chi_max {chi_max}: {lo} {flow_at} {durable_at} {one}"
        );
        match e.solve() {
            Err(SolveError::MultipleEquilibria {
                sign_changes,
                switches,
            }) => {
                assert_eq!(sign_changes, 3, "chi_max {chi_max}");
                assert_eq!(switches, vec![x_sw]);
            }
            other => panic!("chi_max {chi_max}: expected MultipleEquilibria, got {other:?}"),
        }
        assert_eq!(check_regions(&e), 3);
        // The root below the switch is an equilibrium: labour clears there to rounding.
        let (mut a, mut b) = (oracle::BRACKET_LO, x_sw);
        while a.next_up() < b {
            let mid = 0.5 * (a + b);
            if f(mid, 0) > 0.0 {
                a = mid;
            } else {
                b = mid;
            }
        }
        let q = e.at_with(a, 0);
        assert!((q.n_d - q.n_s).abs() <= 1e-12 * q.n_d, "chi_max {chi_max}");
        assert!(a > 0.3 && a < x_sw, "chi_max {chi_max}: {a}");
    }
    // At chi_max 1 the verification's 50-digit solve put the root at 0.471287190181555.
    let e = economy_1c(hidden(1.0));
    assert!(e.at_with(0.47128719018155, 0).excess_demand() > 0.0);
    assert!(e.at_with(0.47128719018156, 0).excess_demand() < 0.0);
}
