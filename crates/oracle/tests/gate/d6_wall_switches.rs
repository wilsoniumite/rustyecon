//! d6: switches at the wall (docs/unit-1d.md §3.3's X; §8): 1c's M5 at ρ 0.15 and χ_max 3,
//! where the flow type holds the whole line and the durable type, labour-light, is cheaper
//! above g_s = 5.794 on the wall. The envelope continued above γ(1) gives the wall its own
//! techniques and ties.

use oracle::Margin;

use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1d::*;

#[test]
fn tie_at_a_wall_switch() {
    // X1, N 0.5: the pool's supply at v_s lies between the two techniques' demands, so the
    // equilibrium is a tie at the wall's switch, σ by 1c's closed form (no type is walled).
    let (e, eq) = checked_1d(wall_switch_economy(0.5));
    assert_eq!(eq.margin, Margin::Wall);
    assert!(e.machines().envelope().switches.is_empty());
    let s = e.wall_switches()[0];
    assert_eq!((s.below, s.above), (0, 1));
    assert!(s.gamma > 1.0);
    close("gamma_s", s.gamma, X_WALL_SWITCH_GAMMA);
    close("v_s", s.wage, X_WALL_SWITCH_V);
    let tie = eq.tie.expect("a tie on the wall");
    assert_eq!((eq.technique, tie.above), (0, 1));
    assert_eq!(tie.gamma, s.gamma);
    assert_eq!(eq.v.to_bits(), s.wage.to_bits());
    assert_eq!(eq.bisection_steps, 0);
    for (name, got, want) in [
        ("sigma", tie.share, X1_SHARE),
        ("P_s", eq.p_s, X1_P_S),
        ("Y", eq.y, X1_Y),
        ("N_a", eq.n_a, X1_N_A),
        ("interest", eq.interest, X1_INTEREST),
    ] {
        close(name, got, want);
    }
    // The two delivered costs are equal at v_s.
    let q = e.at_wage(1.0, s.wage, 0);
    close("p_flow = p_durable", q.type_prices[0], q.type_prices[1]);
    // The two pure techniques bracket the pool's supply.
    let (below, above) = (e.at_wage(1.0, s.wage, 0), e.at_wage(1.0, s.wage, 1));
    assert!(below.excess_demand() > 0.0 && above.excess_demand() < 0.0);
    // Stopping the envelope at γ(1) would leave the flow type on the whole wall.
    assert_eq!(e.wall_technique_at(s.wage), 1);
    check_path_1d(&e);
}

#[test]
fn roots_either_side() {
    // X2, N 1: at the wall below the switch (flow); X3, N 0.003: above it (durable). The
    // technique at the wall is the cheapest delivered cost at the wage.
    for (what, workers, technique, want) in [
        ("X2", 1.0, 0, [X2_V, X2_P_S, X2_Y, X2_N_A]),
        ("X3", 0.003, 1, [X3_V, X3_P_S, X3_Y, X3_N_A]),
    ] {
        let (e, eq) = checked_1d(wall_switch_economy(workers));
        assert_eq!(
            (eq.margin, eq.technique, eq.tie),
            (Margin::Wall, technique, None),
            "{what}"
        );
        for (name, got, w) in [
            ("v", eq.v, want[0]),
            ("P_s", eq.p_s, want[1]),
            ("Y", eq.y, want[2]),
            ("N_a", eq.n_a, want[3]),
        ] {
            close(&format!("{what} {name}"), got, w);
        }
        assert_eq!(e.wall_technique_at(eq.v), technique);
        // X2's wage is below v_s, X3's above it.
        assert_eq!(eq.v > e.wall_switches()[0].wage, technique == 1, "{what}");
        let other = 1 - technique;
        assert!(
            eq.types[other].delivered_cost.unwrap() > eq.types[technique].delivered_cost.unwrap()
        );
        check_path_1d(&e);
    }
    let x3 = solved_1d(wall_switch_economy(0.003));
    close("X3 interest", x3.interest, X3_INTEREST);
    close("X2 Y = 100/13", X2_Y, 100.0 / 13.0);
    close("X2 N_a = 6/13", X2_N_A, 6.0 / 13.0);
}
