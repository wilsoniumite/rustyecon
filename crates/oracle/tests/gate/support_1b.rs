//! Fixtures and checks shared by unit 1b's tests (docs/unit-1b.md §8).

use oracle::{
    Category, CategoryEconomy, CategoryParams, Eq1b, PowerSchedule, Regime, Residuals1b,
    UniformWorkCost, BRACKET_LO,
};
use rustyecon_core::num;

use crate::support::*;

/// The fork economy's categories, in order (docs/unit-1b.md §3.3).
pub const FORK_NAMES: [&str; 4] = ["manufactures", "food", "care", "shelter"];

/// A category from (z, b, μ).
pub fn category(weight: f64, direct_land: f64, density: &[f64]) -> Category {
    Category {
        weight,
        direct_land,
        density: density.to_vec(),
    }
}

/// G1's scalars (N 4, T 10, a 0.3, λ 0.05, b 0.4, γ = 0.2 + 0.8x, χ ~ U[0, 1], flow) on the
/// given task line and categories.
pub fn g1_scalars(edges: &[f64], categories: Vec<Category>) -> CategoryParams {
    CategoryParams {
        workers: 4.0,
        land: 10.0,
        a: 0.3,
        lam: 0.05,
        b: 0.4,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 0.2,
            g1: 0.8,
            k: 1.0,
        },
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
        edges: edges.to_vec(),
        categories,
    }
}

/// The fork economy (constructed 2026-09-27; docs/unit-1b.md §3.3): G1's scalars, edges
/// (0, 0.4, 0.75, 1); manufactures (z 0.3, b 0, μ (2, 0, 0)), food (1, 0.6, (0.5, 1.5, 0)),
/// care (0.2, 0.1, (0, 0, 1)), shelter (0.8, 1, (0, 0.4, 0)).
pub fn fork_economy() -> CategoryParams {
    g1_scalars(
        &[0.0, 0.4, 0.75, 1.0],
        vec![
            category(0.3, 0.0, &[2.0, 0.0, 0.0]),
            category(1.0, 0.6, &[0.5, 1.5, 0.0]),
            category(0.2, 0.1, &[0.0, 0.0, 1.0]),
            category(0.8, 1.0, &[0.0, 0.4, 0.0]),
        ],
    )
}

/// The gap economy (constructed 2026-09-27; docs/unit-1b.md §3.3): the fork economy with
/// N as given and edges (0, 0.4, 0.6, 1), the middle segment used by no category.
pub fn gap_economy(workers: f64) -> CategoryParams {
    CategoryParams {
        workers,
        ..g1_scalars(
            &[0.0, 0.4, 0.6, 1.0],
            vec![
                category(0.3, 0.0, &[2.0, 0.0, 0.0]),
                category(1.0, 0.6, &[0.5, 0.0, 0.2]),
                category(0.2, 0.1, &[0.0, 0.0, 0.3]),
                category(0.8, 1.0, &[0.0, 0.0, 0.1]),
            ],
        )
    }
}

/// Validates unit-1b parameters the test knows to be valid.
pub fn economy_1b(params: CategoryParams) -> CategoryEconomy {
    CategoryEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{params:?}: {e}"))
}

/// Solves a unit-1b economy the test knows to be interior.
pub fn interior_1b(params: CategoryParams) -> Box<Eq1b> {
    match economy_1b(params.clone()).solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{params:?} is not interior: {other:?}"),
    }
}

/// x is where f = n_D − n_S changes sign, to one double, and the better end (1a's
/// `assert_root` on the category economy).
pub fn assert_root_1b(economy: &CategoryEconomy, x: f64) {
    let f = |x: f64| economy.at(x).excess_demand();
    let fx = f(x);
    if fx == 0.0 {
        return;
    }
    let other = if fx > 0.0 { x.next_up() } else { x.next_down() };
    let f_other = f(other);
    assert!(
        f_other == 0.0 || f_other.signum() != fx.signum(),
        "no sign change between {x:e} (f = {fx:e}) and {other:e} (f = {f_other:e})"
    );
    assert!(
        fx.abs() <= f_other.abs(),
        "x* = {x:e} is not the better end"
    );
}

/// a ≤ b with the gate's relative slack: a ≤ b·(1 + FULL).
pub fn at_most(name: &str, a: f64, b: f64) {
    assert!(a <= b + FULL * b.abs(), "{name}: {a:e} > {b:e}");
}

/// The residuals of docs/unit-1b.md §6, recomputed from the reported fields in the solve's
/// own form: the same operations on the same doubles, so they must agree bit for bit.
pub fn recomputed_1b(economy: &CategoryEconomy, eq: &Eq1b) -> Residuals1b {
    let p = economy.params();
    let (mut fork, mut totals, mut spending) = (0.0f64, 0.0f64, 0.0);
    for (c, cat) in eq.categories.iter().zip(&p.categories) {
        fork = fork.max((c.price - (eq.v * c.l_star + cat.direct_land)).abs() / c.price);
        totals = totals.max((c.price - (eq.v * c.lambda_tilde + c.b_tilde)).abs() / c.price);
        spending += c.price * c.output;
    }
    Residuals1b {
        labor: (eq.n_a - economy.at(eq.x_star).n_s).abs(),
        income: (eq.y * eq.p_s - eq.income).abs() / eq.income,
        land: (eq.b_d * eq.y + p.b * p.delta * eq.k - p.land).abs() / p.land,
        services: if eq.k > 0.0 {
            (eq.k - eq.y * eq.m_s - p.a * p.delta * eq.k).abs() / eq.k
        } else {
            0.0
        },
        user_cost: (eq.p_m - eq.u * (p.a * eq.p_m + p.lam * eq.v + p.b)).abs() / eq.p_m,
        fork,
        totals,
        expenditure: (spending - eq.income).abs() / eq.income,
    }
}

/// Every identity of docs/unit-1b.md §4.2-4.4 at an interior equilibrium, recomputed here
/// from the reported fields and the parameters (§8, C5 `identities`).
pub fn check_identities_1b(economy: &CategoryEconomy, eq: &Eq1b) {
    let p = economy.params();
    let at = |what: &str| format!("{what} at {p:?}");
    // Income as Y P_s and as expenditure on the categories (SSRN App. C, p.30).
    near(
        &at("income = Y P_s"),
        eq.y * eq.p_s,
        eq.v * eq.n_a + p.land + eq.interest,
        FULL * eq.income,
    );
    let spending: f64 = eq.categories.iter().map(|c| c.price * c.output).sum();
    near(
        &at("income = sum p_j y_j"),
        spending,
        eq.income,
        FULL * eq.income,
    );
    // Land and machine services clear.
    near(
        &at("land"),
        eq.b_d * eq.y + p.b * p.delta * eq.k,
        p.land,
        FULL * p.land,
    );
    near(
        &at("services"),
        eq.y * eq.m_s + p.a * p.delta * eq.k,
        eq.k,
        FULL * eq.k,
    );
    // The machine row: the user-cost price, V_m, the margin, and the machine totals.
    let build_cost = p.a * eq.p_m + p.lam * eq.v + p.b;
    close(&at("user cost"), eq.u * build_cost, eq.p_m);
    close(&at("V_m"), build_cost, eq.v_m);
    close(&at("v = gamma p_m"), eq.gamma_star * eq.p_m, eq.v);
    close(
        &at("p_m = v lt_m + bt_m"),
        eq.v * eq.lambda_tilde_machine + eq.b_tilde_machine,
        eq.p_m,
    );
    // The basket's sums, recomputed from the categories.
    let dot = |f: fn(&oracle::CategoryEq) -> f64| -> f64 {
        eq.categories
            .iter()
            .zip(&p.categories)
            .map(|(c, cat)| cat.weight * f(c))
            .sum()
    };
    close(&at("P_s = sum z p"), dot(|c| c.price), eq.p_s);
    close(&at("M_s = sum z M"), dot(|c| c.machine), eq.m_s);
    near(
        &at("H_s = sum z H"),
        dot(|c| c.human),
        eq.h_s,
        FULL * eq.h_s,
    );
    let b_d: f64 = p.categories.iter().map(|c| c.weight * c.direct_land).sum();
    close(&at("B_d"), b_d, eq.b_d);
    assert_eq!(eq.b_d, economy.basket_direct_land(), "{}", at("B_d"));
    close(&at("L_s*"), dot(|c| c.l_star), eq.l_star_s);
    close(&at("L_s"), dot(|c| c.lambda_tilde), eq.l_s);
    close(&at("B_s"), dot(|c| c.b_tilde), eq.b_s);
    close(&at("L_s^q"), dot(|c| c.lambda_tilde_q), eq.l_s_q);
    close(&at("B_s^q"), dot(|c| c.b_tilde_q), eq.b_s_q);
    // P_s in both forms (SSRN eq 7 with totals; main.tex's direct form).
    close(&at("P_s = v L_s + B_s"), eq.v * eq.l_s + eq.b_s, eq.p_s);
    close(
        &at("P_s = v L_s* + B_d"),
        eq.v * eq.l_star_s + eq.b_d,
        eq.p_s,
    );
    // SSRN eq 11 on the clearing side.
    close(&at("Y = T / B_s^q"), p.land / eq.b_s_q, eq.y);
    close(
        &at("N_a = T L_s^q / B_s^q"),
        p.land * eq.l_s_q / eq.b_s_q,
        eq.n_a,
    );
    // Hours.
    near(
        &at("final hours"),
        eq.y * eq.h_s,
        eq.final_hours,
        FULL * eq.n_a,
    );
    close(&at("N_a"), eq.final_hours + p.lam * p.delta * eq.k, eq.n_a);
    let spacing = |v: f64| (v - v.next_down()).max(v.next_up() - v);
    assert!(
        (eq.one_minus_x_star - (1.0 - eq.x_star)).abs()
            <= spacing(eq.x_star) + spacing(1.0 - eq.x_star),
        "{}",
        at("1 - x*")
    );
    // Prices and L* are evaluated at the double x* (docs/unit-1b.md §5.1 step 5), bit for
    // bit; the hours-type fields carry 1 - x* in the top segment instead.
    let q = economy.at(eq.x_star);
    for (j, c) in eq.categories.iter().enumerate() {
        assert_eq!(c.price.to_bits(), q.prices[j].to_bits(), "{}", at("p_j"));
        assert_eq!(c.machine.to_bits(), q.machine[j].to_bits(), "{}", at("M_j"));
        assert_eq!(
            c.l_star.to_bits(),
            (q.human[j] + q.machine[j] / q.gamma).to_bits(),
            "{}",
            at("L*_j at the double x*")
        );
    }
    assert_eq!(eq.p_s.to_bits(), q.p_s.to_bits(), "{}", at("P_s"));
    // Per category: outputs clear by the basket, and the hours add up.
    for (j, (c, cat)) in eq.categories.iter().zip(&p.categories).enumerate() {
        let tag = |what: &str| at(&format!("category {j}: {what}"));
        close(&tag("y_j = z_j Y"), cat.weight * eq.y, c.output);
        near(
            &tag("share"),
            cat.weight * c.price / eq.p_s,
            c.share,
            FULL * c.share,
        );
        near(
            &tag("final hours"),
            c.output * c.human,
            c.final_hours,
            FULL * eq.final_hours,
        );
        near(
            &tag("machine services"),
            c.output * c.machine,
            c.machine_services,
            FULL * eq.k,
        );
        assert_eq!(c.l_bar, economy.all_human_hours()[j], "{}", tag("L-bar"));
    }
    let shares: f64 = eq.categories.iter().map(|c| c.share).sum();
    close(&at("shares sum to 1"), shares, 1.0);
    near(
        &at("sum of final hours"),
        eq.categories.iter().map(|c| c.final_hours).sum(),
        eq.final_hours,
        FULL * eq.n_a,
    );
    // Shares of income, the real wage and the baskets.
    close(
        &at("labour + capital + land shares"),
        eq.labor_share + eq.capital_share + p.land / eq.income,
        1.0,
    );
    close(&at("v / P_s"), eq.v / eq.p_s, eq.real_wage);
    close(&at("N P_s"), p.workers * eq.p_s, eq.support_cost);
    close(
        &at("worker baskets"),
        p.workers + eq.v * eq.n_a / eq.p_s,
        eq.worker_baskets,
    );
    close(
        &at("baskets = Y"),
        eq.worker_baskets + eq.provider_baskets,
        eq.y,
    );
    assert_eq!(eq.funded, eq.provider_baskets > 0.0, "{}", at("funded"));
    assert_eq!(
        eq.participation,
        (eq.n_a / p.workers).min(1.0),
        "{}",
        at("participation")
    );
    // Interest is rho times machine wealth (check_dynamics.py L2, :164-169).
    if p.rho > 0.0 {
        let carry = num::pow(1.0 + p.rho, f64::from(p.build_lag - 1));
        let wealth = eq.v_m * eq.k * (carry + p.delta * (carry - 1.0) / p.rho);
        near(
            &at("interest"),
            eq.interest,
            p.rho * wealth,
            FULL * eq.income,
        );
    } else {
        assert_eq!(eq.interest, 0.0, "{}", at("interest"));
    }
    // The residuals, each at most FULL and equal to its recomputation.
    let r = eq.residuals;
    for (name, value) in [
        ("income", r.income),
        ("land", r.land),
        ("services", r.services),
        ("user cost", r.user_cost),
        ("fork", r.fork),
        ("totals", r.totals),
        ("expenditure", r.expenditure),
        ("labour / N_a", r.labor / eq.n_a),
    ] {
        assert!(value <= FULL, "{name} residual {value:e} {}", at(""));
    }
    assert_eq!(recomputed_1b(economy, eq), r, "{}", at("residuals"));
    assert_root_1b(economy, eq.x_star);
}

/// The fork identity in both forms, the category bounds and the purchasing-power pair, for
/// every category, and the basket's rent ceiling (docs/unit-1b.md §4.2; SSRN eq 12, 13,
/// 19; main.tex:459-474).
pub fn check_fork_and_bounds(economy: &CategoryEconomy, eq: &Eq1b) {
    let p = economy.params();
    for (j, (c, cat)) in eq.categories.iter().zip(&p.categories).enumerate() {
        let tag = |what: &str| format!("category {j}: {what} at {p:?}");
        let b_j = cat.direct_land;
        // The fork identity: main.tex's direct form and SSRN eq 12's totals form.
        close(&tag("p = v L* + b"), eq.v * c.l_star + b_j, c.price);
        close(
            &tag("p = v lt + bt"),
            eq.v * c.lambda_tilde + c.b_tilde,
            c.price,
        );
        close(
            &tag("v/p = 1/(L* + b/v)"),
            1.0 / (c.l_star + b_j / eq.v),
            c.real_wage,
        );
        close(&tag("v/p"), eq.v / c.price, c.real_wage);
        // The two forms agree: L* = lt + (bt - b)/v, on the scale of p/v = L* + b/v.
        near(
            &tag("L* = lt + (bt - b)/v"),
            c.lambda_tilde + (c.b_tilde - b_j) / eq.v,
            c.l_star,
            FULL * (c.l_star + b_j / eq.v),
        );
        // The chain b_j <= bt^q <= bt <= p <= v Lbar + b_j, and L* <= Lbar (SSRN eq 19;
        // main.tex eq category-bounds).
        at_most(&tag("b <= bt^q"), b_j, c.b_tilde_q);
        at_most(&tag("bt^q <= bt"), c.b_tilde_q, c.b_tilde);
        at_most(&tag("bt <= p"), c.b_tilde, c.price);
        at_most(&tag("p <= v Lbar + b"), c.price, eq.v * c.l_bar + b_j);
        at_most(&tag("L* <= Lbar"), c.l_star, c.l_bar);
        at_most(&tag("lt^q <= lt"), c.lambda_tilde_q, c.lambda_tilde);
        // The pair (SSRN eq 13; main.tex eq fork-pair and :474).
        close(&tag("floor"), 1.0 / (c.l_bar + b_j / eq.v), c.wage_floor);
        at_most(&tag("floor <= v/p"), c.wage_floor, c.real_wage);
        match c.wage_ceiling {
            Some(ceiling) => {
                assert!(c.b_tilde > 0.0, "{}", tag("ceiling without land"));
                close(&tag("ceiling"), eq.v / c.b_tilde, ceiling);
                at_most(&tag("v/p <= v/bt"), c.real_wage, ceiling);
            }
            None => assert_eq!(c.b_tilde, 0.0, "{}", tag("ceiling")),
        }
        if b_j > 0.0 {
            at_most(&tag("v/p <= v/b"), c.real_wage, eq.v / b_j);
        } else {
            at_most(&tag("v/p >= 1/Lbar"), 1.0 / c.l_bar, c.real_wage);
        }
        // The three-taxes shares of the price, at u = 1.
        match (c.phi_w, c.phi_r) {
            (Some(w), Some(r)) => {
                assert_eq!(eq.u, 1.0);
                close(&tag("phi_w + phi_r"), w + r, 1.0);
                close(&tag("phi_w"), eq.v * c.lambda_tilde / c.price, w);
            }
            (None, None) => assert_ne!(eq.u, 1.0),
            other => panic!("{}: {other:?}", tag("phi")),
        }
    }
    at_most("v/P_s <= v/B_s", eq.real_wage, eq.rent_ceiling);
    close("rent ceiling", eq.v / eq.b_s, eq.rent_ceiling);
}

/// Whether some basket category has tasks on the segment holding x (e_s ≤ x < e_{s+1}, the
/// top segment for x = 1), recomputed from the parameters.
pub fn margin_is_active(params: &CategoryParams, x: f64) -> bool {
    let segments = params.edges.len() - 1;
    let s = (0..segments)
        .rev()
        .find(|&s| params.edges[s] <= x)
        .expect("x >= 0");
    params
        .categories
        .iter()
        .any(|c| c.weight > 0.0 && c.density[s] > 0.0)
}

/// n_D nonincreasing, v/P_s strictly increasing, n_S nondecreasing and f single-crossing on
/// the grid {BRACKET_LO, 1/100, ..., 1} (docs/unit-1b.md §5.3).
pub fn check_single_crossing_1b(economy: &CategoryEconomy) {
    const GRID: usize = 100;
    let grid: Vec<_> = std::iter::once(BRACKET_LO)
        .chain((1..=GRID).map(|i| i as f64 / GRID as f64))
        .map(|x| economy.at(x))
        .collect();
    let p = economy.params();
    for w in grid.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        assert!(b.n_d <= a.n_d, "n_D rises at x = {} at {p:?}", b.x);
        assert!(
            b.v / b.p_s > a.v / a.p_s,
            "v/P_s falls at x = {} at {p:?}",
            b.x
        );
        assert!(b.n_s >= a.n_s, "n_S falls at x = {} at {p:?}", b.x);
    }
    let positive: Vec<bool> = grid.iter().map(|q| q.excess_demand() > 0.0).collect();
    assert!(positive[0] && !positive[GRID], "{p:?}");
    let crossings = positive.windows(2).filter(|w| w[0] != w[1]).count();
    assert_eq!(crossings, 1, "{p:?}");
}
