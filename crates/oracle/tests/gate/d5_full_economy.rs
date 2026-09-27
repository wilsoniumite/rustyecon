//! d5: the full economy (docs/unit-1d.md §3.3's F; §8): 1c's M4 with the common
//! human-required hours in care and two worker types, the trained's reserved hours in food and
//! care, through the chain of intermediate inputs; a wall past a machine switch; a tie with a
//! walled type.

use oracle::Margin;

use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1b::FORK_NAMES;
use crate::support_1d::*;

#[test]
fn full_goldens() {
    // F1: contestable with the engine, the trained at its wall.
    let (e, eq) = checked_1d(full(4.0, 3.0, 1.0));
    assert_eq!(
        (eq.margin, eq.technique, eq.tie),
        (Margin::Contestable, 1, None)
    );
    let trained = &eq.workers[1];
    assert!(!trained.pooled);
    for (name, got, want) in [
        ("x*", eq.x_star, F1_X_STAR),
        ("v", eq.v, F1_V),
        ("P_s", eq.p_s, F1_P_S),
        ("Y", eq.y, F1_Y),
        ("N_a", eq.n_a, F1_N_A),
        ("n_pool", eq.n_pool, F1_N_POOL),
        ("I", eq.income, F1_INCOME),
        ("interest", eq.interest, F1_INTEREST),
        ("trained wage", trained.wage, F1_TRAINED_WAGE),
        (
            "trained premium",
            trained.premium.unwrap(),
            F1_TRAINED_PREMIUM,
        ),
        ("trained hours", trained.hours, F1_TRAINED_HOURS),
    ] {
        close(name, got, want);
    }
    let prices = [F1_MANUFACTURES_P, F1_FOOD_P, F1_CARE_P, F1_SHELTER_P];
    let reserved = [
        F1_MANUFACTURES_RESERVED_COST,
        F1_FOOD_RESERVED_COST,
        F1_CARE_RESERVED_COST,
        F1_SHELTER_RESERVED_COST,
    ];
    for (j, c) in eq.categories.iter().enumerate() {
        close(&format!("p {}", FORK_NAMES[j]), c.price, prices[j]);
        if reserved[j] == 0.0 {
            assert_eq!(c.reserved_cost, 0.0);
        } else {
            close(
                &format!("R {}", FORK_NAMES[j]),
                c.reserved_cost,
                reserved[j],
            );
        }
    }
    // Care buys 0.2 of food, so its reserved cost carries food's: v_T·(0.3 + 0.2·0.05), not the
    // direct v_T·0.3 (0.12352).
    close(
        "care's reserved cost through the chain",
        trained.wage * (0.3 + 0.2 * 0.05),
        eq.categories[2].reserved_cost,
    );
    assert!(eq.categories[2].reserved_cost - trained.wage * 0.3 > 4e-3);
    close(
        "food's",
        trained.wage * 0.05,
        eq.categories[1].reserved_cost,
    );
    // Care's common human-required hours: 0.2 directly, none through food.
    assert_eq!(eq.categories[2].human_required, 0.2);
    check_path_1d(&e);
}

#[test]
fn cost_system_with_reserved_costs() {
    // At F1 to F3 the full cost system holds over every row, with the reserved costs on the
    // category rows (SSRN eq 3, A.1, App. C): check_identities_1d runs p = Ap + λv + b + R,
    // f = (I − A^qᵀ)y, n_pool = λ^qᵀy, T = b^qᵀy and pᵀf = v·n_pool + Σ v_i·D_i + T + interest.
    for (entrants, trained, eta) in [(4.0, 3.0, 1.0), (4.0, 3.0, 0.25), (16.0, 1.5, 0.5)] {
        let (_, eq) = checked_1d(full(entrants, trained, eta));
        assert!(eq.interest > 0.0);
        assert!(eq.residuals.leontief_price > 0.0 || eq.residuals.leontief_quantity > 0.0);
        let reserved: f64 = eq.workers.iter().map(|w| w.wage * w.reserved_hours).sum();
        close("wage bill", eq.v * eq.n_pool + reserved, eq.wage_bill);
    }
}

#[test]
fn wall_past_a_switch() {
    // F2, η 0.25 (1c's M4 is BoundaryNoMargin there): the loom holds the whole line; the
    // switch to the engine lies on the wall, at v_s below the equilibrium, which is at the wall
    // with the engine. A unit that kept the line's technique on the wall would give v 0.3109.
    let (e, eq) = checked_1d(full(4.0, 3.0, 0.25));
    assert_eq!(eq.margin, Margin::Wall);
    assert_eq!(e.machines().envelope().switches.len(), 0);
    assert_eq!(e.machines().envelope().first, 0);
    assert_eq!(eq.technique, 1);
    let s = e.wall_switches()[0];
    assert_eq!((s.below, s.above), (0, 1));
    assert!(s.gamma > 0.25 && s.wage < eq.v);
    for (name, got, want) in [
        ("v", eq.v, F2_V),
        ("g", eq.g, F2_G),
        ("P_s", eq.p_s, F2_P_S),
        ("Y", eq.y, F2_Y),
        ("N_a", eq.n_a, F2_N_A),
        ("gamma_s", s.gamma, F2_WALL_SWITCH_GAMMA),
        ("v_s", s.wage, F2_WALL_SWITCH_V),
        (
            "trained premium",
            eq.workers[1].premium.unwrap(),
            F2_TRAINED_PREMIUM,
        ),
    ] {
        close(name, got, want);
    }
    // v_s is the loom's closure wage at γ_s, and there the two delivered costs are equal.
    let block = e.machines().block();
    close("v_s", block.closure_wage(s.gamma, 0).unwrap(), s.wage);
    let q = e.at_wage(1.0, s.wage, 0);
    close("delivered costs", q.type_prices[0], q.type_prices[1] / 2.0);
    assert_eq!(e.wall_technique_at(eq.v), 1);
    assert_eq!(e.wall_technique_at(s.wage.next_down()), 0);
    // The wall switch is 1c's M4t switch wage: the same γ as at η 0.5 on the line.
    close("gamma_s = M4t's", s.gamma, crate::goldens_1c::M4T_GAMMA);
    // With the loom kept on the wall the pool would not clear at this wage.
    assert!(e.at_wage(1.0, eq.v, 0).excess_demand().abs() > 1e-3);
    check_path_1d(&e);
}

#[test]
fn tie_with_a_walled_type() {
    // F3 (N_E 16, N_T 1.5, η 0.5): a tie at the line's switch with the trained at its wall.
    // The walled wage moves with the split, so σ is found by bisection, not by 1c's closed
    // form, which would be off by 4.3e-4 relative.
    let (e, eq) = checked_1d(full(16.0, 1.5, 0.5));
    assert_eq!(eq.margin, Margin::Contestable);
    let tie = eq.tie.expect("a tie");
    assert_eq!((eq.technique, tie.above), (0, 1));
    assert!(!eq.workers[1].pooled);
    assert_eq!(eq.bisection_steps, 0);
    for (name, got, want) in [
        ("x*", eq.x_star, F3_X_STAR),
        ("v", eq.v, F3_V),
        ("sigma", tie.share, F3_SHARE),
        ("P_s", eq.p_s, F3_P_S),
        ("Y", eq.y, F3_Y),
        ("N_a", eq.n_a, F3_N_A),
        (
            "trained premium",
            eq.workers[1].premium.unwrap(),
            F3_TRAINED_PREMIUM,
        ),
    ] {
        close(name, got, want);
    }
    assert!((F3_SHARE_CLOSED_FORM - tie.share).abs() > 1e-4 * tie.share);
    // f(σ) is monotone on a grid of σ, and changes sign at the reported σ to adjacent doubles.
    let q = e.at_with(eq.x_star, eq.technique);
    let f = |s: f64| e.excess_at_share(&q, tie.above, s);
    let grid: Vec<f64> = (0..=50).map(|k| f(k as f64 / 50.0)).collect();
    assert!(grid.windows(2).all(|w| w[1] < w[0]), "{grid:?}");
    assert!(grid[0] > 0.0 && grid[50] < 0.0);
    let (below, above) = (f(tie.share.next_down()), f(tie.share.next_up()));
    assert!(below >= 0.0 || above <= 0.0);
    assert!(f(tie.share).abs() <= 1e-12 * eq.n_pool);
    assert_eq!(f(0.0).to_bits(), q.excess_demand().to_bits());
    // The tie's σ-mix moves the walled wage: P_s is not the pure technique's.
    assert_ne!(eq.p_s, q.p_s);
    check_path_1d(&e);
}
