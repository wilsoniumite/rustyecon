//! C7: gaps, reductions, regimes and validation (docs/unit-1b.md §8). A gap is a stretch
//! of the task line that no basket category uses; the reductions are changes of
//! representation that must leave the economy as it is.

use oracle::{
    Category, CategoryEconomy, CategoryParams, Eq1b, Output1b, ParamError, Regime, Schedule,
    SCALE_CEIL,
};

use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;

#[test]
fn gap_goldens() {
    // docs/unit-1b.md §7's gap economy at N 5: x* lies in [0.4, 0.6), which no category
    // uses. No produced task is at parity; labour clearing sets w/p_m = gamma(x*).
    let e = economy_1b(gap_economy(5.0));
    let eq = interior_1b(gap_economy(5.0));
    assert!(0.4 < eq.x_star && eq.x_star < 0.6);
    assert!(!eq.margin_active);
    for (name, got, want) in [
        ("x*", eq.x_star, C7_GAP_X_STAR),
        ("gamma(x*)", eq.gamma_star, C7_GAP_GAMMA_STAR),
        ("v", eq.v, C7_GAP_V),
        ("P_s", eq.p_s, C7_GAP_P_S),
        ("Y", eq.y, C7_GAP_Y),
        ("K", eq.k, C7_GAP_K),
        ("N_a", eq.n_a, C7_GAP_N_A),
        ("I", eq.income, C7_GAP_INCOME),
        ("H_s", eq.h_s, C7_GAP_H_S),
        ("M_s", eq.m_s, C7_GAP_M_S),
        ("L_s^q", eq.l_s_q, C7_GAP_L_S_Q),
        ("B_s^q", eq.b_s_q, C7_GAP_B_S_Q),
    ] {
        close(name, got, want);
    }
    close("L_s^q = 1.0312/7", C7_GAP_L_S_Q, 1.0312 / 7.0);
    close("B_s^q = 10.5736/7", C7_GAP_B_S_Q, 10.5736 / 7.0);
    let per_category = [
        (C7_GAP_MANUFACTURES_P, C7_GAP_MANUFACTURES_REAL_WAGE),
        (C7_GAP_FOOD_P, C7_GAP_FOOD_REAL_WAGE),
        (C7_GAP_CARE_P, C7_GAP_CARE_REAL_WAGE),
        (C7_GAP_SHELTER_P, C7_GAP_SHELTER_REAL_WAGE),
    ];
    for ((name, c), (p, wage)) in FORK_NAMES.iter().zip(&eq.categories).zip(per_category) {
        close(&format!("p of {name}"), c.price, p);
        close(&format!("v/p of {name}"), c.real_wage, wage);
    }
    // The fork identity and the bounds hold at boundary task assignments too (SSRN p.13).
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
    check_single_crossing_1b(&e);
    // Every task below the gap is machine work and every task above it hand work.
    for c in &eq.categories {
        assert!(c.l_star < c.l_bar || c.machine == 0.0);
    }
}

#[test]
fn only_bought_tasks_make_a_margin() {
    // A category nobody buys, with tasks inside the gap, does not close it: the margin is
    // the basket's (docs/unit-1b.md §2.3), and the equilibrium is unchanged bit for bit.
    let reference = interior_1b(gap_economy(5.0));
    let mut params = gap_economy(5.0);
    params.categories.push(category(0.0, 0.3, &[0.0, 2.0, 0.0]));
    let eq = interior_1b(params.clone());
    assert!(!eq.margin_active);
    assert!(!margin_is_active(&params, eq.x_star));
    assert_eq!(eq.x_star.to_bits(), reference.x_star.to_bits());
    // Bought, the same category puts tasks at parity in the segment and moves the root.
    params.categories[4].weight = 0.5;
    let eq = interior_1b(params.clone());
    assert!(0.4 < eq.x_star && eq.x_star < 0.6 && eq.margin_active);
}

#[test]
fn a_root_below_every_bought_task_uses_no_machines() {
    // One service with tasks only on [0.5, 1] and a site; with N 30 the root lies in
    // [0, 0.5), below every bought task. No machine service is used: K = 0, M_s = 0, and the
    // services residual is 0 by definition (0/0 otherwise; docs/unit-1b.md §5.1).
    let params = CategoryParams {
        workers: 30.0,
        ..g1_scalars(
            &[0.0, 0.5, 1.0],
            vec![
                category(1.0, 0.2, &[0.0, 1.0]),
                category(1.0, 1.0, &[0.0, 0.0]),
            ],
        )
    };
    let e = economy_1b(params.clone());
    let eq = interior_1b(params);
    assert!(eq.x_star < 0.5 && !eq.margin_active);
    assert_eq!((eq.k, eq.m_s, eq.machine_hours), (0.0, 0.0, 0.0));
    assert_eq!(eq.residuals.services, 0.0);
    assert_eq!(eq.categories[0].machine, 0.0);
    // Everything else holds as usual: n_D = Y H_s with H_s = 1/2.
    assert_eq!(eq.h_s, 0.5);
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
}

#[test]
fn gap_quantities_do_not_move() {
    // In the gap n_D does not depend on x (no task changes hands), so the root moves with
    // N through supply alone: N 4.5 and N 5 give different x* and v, and bit-equal Y, K,
    // N_a, H_s and M_s (docs/unit-1b.md §5.4).
    let five = interior_1b(gap_economy(5.0));
    let four_and_half = interior_1b(gap_economy(4.5));
    close("x* at N 4.5", four_and_half.x_star, C7_GAP_N4_5_X_STAR);
    assert!(four_and_half.x_star > five.x_star + 0.05);
    assert!(four_and_half.v > five.v);
    assert!(!four_and_half.margin_active);
    for (name, a, b) in [
        ("Y", five.y, four_and_half.y),
        ("K", five.k, four_and_half.k),
        ("N_a", five.n_a, four_and_half.n_a),
        ("H_s", five.h_s, four_and_half.h_s),
        ("M_s", five.m_s, four_and_half.m_s),
        ("L_s^q", five.l_s_q, four_and_half.l_s_q),
        ("B_s^q", five.b_s_q, four_and_half.b_s_q),
    ] {
        assert_eq!(a.to_bits(), b.to_bits(), "{name}");
    }
    // At N 5.4 the root has left the gap for the segment below, where the margin is active.
    let left = interior_1b(gap_economy(5.4));
    assert!(left.x_star < 0.4 && left.margin_active);
}

#[test]
fn margin_on_an_edge() {
    // margin_active reads x* on an edge as in the segment above it, e_s <= x* < e_{s+1}
    // (Eq1b::margin_active; docs/unit-1b.md §2.3). These N put the gap economy's root
    // exactly on its edges; they were found on 2026-09-27 by stepping N a double at a time
    // near n_D(e)/F(e). At 0.6 the segment above is the top one, which the basket uses; at
    // 0.4 it is the gap.
    for (workers, edge, active) in [
        (4.219565357117047, 0.6f64, true),
        (5.39015188299268, 0.4, false),
    ] {
        let params = gap_economy(workers);
        let eq = interior_1b(params.clone());
        assert_eq!(eq.x_star.to_bits(), edge.to_bits(), "N {workers}: x* moved");
        assert_eq!(eq.margin_active, active, "x* = {edge}");
        assert_eq!(eq.margin_active, margin_is_active(&params, eq.x_star));
        let e = economy_1b(params);
        check_identities_1b(&e, &eq);
        check_fork_and_bounds(&e, &eq);
    }
}

/// The sliver economy (constructed 2026-09-27; docs/unit-1b.md §5.4): a service with tasks
/// on [0.5, 1] only (z 1, b 0.5) and a site (z 1, b 1), durable (ρ 0.05, δ 0.2, J_b 2).
/// With the root just above 0.5, all machine use is in the sliver [0.5, x*].
fn sliver_economy(workers: f64) -> CategoryParams {
    CategoryParams {
        workers,
        rho: 0.05,
        delta: 0.2,
        build_lag: 2,
        ..g1_scalars(
            &[0.0, 0.5, 1.0],
            vec![
                category(1.0, 0.5, &[0.0, 1.0]),
                category(1.0, 1.0, &[0.0, 0.0]),
            ],
        )
    }
}

/// An output read from an equilibrium.
type Field = fn(&Eq1b) -> f64;

/// One root near an interior edge: the outputs that keep full precision against their
/// goldens at the gate's 1e-12, and those made of the sliver (with whether they are machine
/// quantities) against the bound of docs/unit-1b.md §5.4. Returns the worst sliver error.
fn near_edge(
    what: &str,
    params: CategoryParams,
    edge: f64,
    full: &[(&str, Field, f64)],
    sliver: &[(&str, Field, f64, bool)],
) -> f64 {
    // 2^-53, the unit roundoff; the bound's factor 2 is slack for the few roundings that
    // add to the root's.
    const UNIT: f64 = f64::EPSILON / 2.0;
    const SLACK: f64 = 2.0;
    let e = economy_1b(params.clone());
    let eq = interior_1b(params.clone());
    assert!(eq.margin_active, "{what}");
    for (name, field, want) in full {
        close(&format!("{name}, {what}"), field(&eq), *want);
    }
    let d = (eq.x_star - edge).abs();
    let j_over_gamma = params.schedule.integral(eq.x_star) / eq.gamma_star;
    let mut worst: f64 = 0.0;
    for (name, field, want, machine) in sliver {
        let error = (field(&eq) - want).abs() / want;
        let reach = if *machine {
            eq.x_star + 2.0 * j_over_gamma
        } else {
            eq.x_star
        };
        let bound = SLACK * UNIT * reach / d;
        assert!(
            error <= bound,
            "{name}, {what}: {error:e} beyond the bound {bound:e}"
        );
        worst = worst.max(error);
    }
    check_identities_1b(&e, &eq);
    check_fork_and_bounds(&e, &eq);
    worst
}

#[test]
fn precision_near_an_interior_edge() {
    // docs/unit-1b.md §5.4: x* is a double, and only the top segment carries its offset, so
    // an output made of the sliver between x* and an interior edge loses relative precision
    // as the sliver narrows; the prices, v, P_s, Y and N_a keep it. The goldens take every
    // input as the double the oracle reads (goldens/generate_1b.py).
    let below = near_edge(
        "x* 1e-9 below 0.4",
        gap_economy(C7_EDGE_BELOW_N),
        0.4,
        &[
            ("x*", |q| q.x_star, C7_EDGE_BELOW_X_STAR),
            ("v", |q| q.v, C7_EDGE_BELOW_V),
            ("P_s", |q| q.p_s, C7_EDGE_BELOW_P_S),
            ("Y", |q| q.y, C7_EDGE_BELOW_Y),
            ("N_a", |q| q.n_a, C7_EDGE_BELOW_N_A),
        ],
        &[(
            "H of manufactures",
            |q| q.categories[0].human,
            C7_EDGE_BELOW_MANUFACTURES_H,
            false,
        )],
    );
    let above = near_edge(
        "x* 1e-9 above 0.6",
        gap_economy(C7_EDGE_ABOVE_N),
        0.6,
        &[
            ("x*", |q| q.x_star, C7_EDGE_ABOVE_X_STAR),
            ("v", |q| q.v, C7_EDGE_ABOVE_V),
            ("P_s", |q| q.p_s, C7_EDGE_ABOVE_P_S),
            ("Y", |q| q.y, C7_EDGE_ABOVE_Y),
            ("N_a", |q| q.n_a, C7_EDGE_ABOVE_N_A),
        ],
        &[
            (
                "M of care",
                |q| q.categories[2].machine,
                C7_EDGE_ABOVE_CARE_M,
                true,
            ),
            (
                "M of shelter",
                |q| q.categories[3].machine,
                C7_EDGE_ABOVE_SHELTER_M,
                true,
            ),
        ],
    );
    let sliver = near_edge(
        "x* 1e-6 above 0.5",
        sliver_economy(C7_SLIVER_N),
        0.5,
        &[
            ("x*", |q| q.x_star, C7_SLIVER_X_STAR),
            ("v", |q| q.v, C7_SLIVER_V),
            ("P_s", |q| q.p_s, C7_SLIVER_P_S),
            ("Y", |q| q.y, C7_SLIVER_Y),
            ("N_a", |q| q.n_a, C7_SLIVER_N_A),
        ],
        &[
            ("K", |q| q.k, C7_SLIVER_K, true),
            ("M_s", |q| q.m_s, C7_SLIVER_M_S, true),
            ("interest", |q| q.interest, C7_SLIVER_INTEREST, true),
        ],
    );
    // The loss is real, not only allowed: on 2026-09-27 the errors were 2.1e-8, 7.5e-8 and
    // 1.9e-11, all beyond the gate's 1e-12, so these outputs cannot be held to it here.
    for (what, worst) in [
        ("below 0.4", below),
        ("above 0.6", above),
        ("above 0.5", sliver),
    ] {
        assert!(worst > FULL, "{what}: {worst:e}");
    }
}

/// Every economy-level number of an equilibrium, by key, as bits, without the two residuals
/// that are maxima over categories.
fn aggregates(eq: &Eq1b) -> Vec<(&'static str, u64)> {
    eq.outputs()
        .into_iter()
        .filter(|(key, _)| key.category.is_none())
        .filter(|(key, _)| key.name != "res_fork" && key.name != "res_totals")
        .map(|(key, output)| {
            let bits = match output {
                Output1b::Float(v) | Output1b::Optional(Some(v)) => v.to_bits(),
                Output1b::Optional(None) => u64::MAX,
                Output1b::Flag(b) => u64::from(b),
                Output1b::Count(n) => u64::from(n),
            };
            (key.name, bits)
        })
        .collect()
}

/// The aggregates that must agree within FULL after a change of representation.
fn close_aggregates(what: &str, a: &Eq1b, b: &Eq1b) {
    for (name, x, y) in [
        ("x*", a.x_star, b.x_star),
        ("gamma(x*)", a.gamma_star, b.gamma_star),
        ("v", a.v, b.v),
        ("p_m", a.p_m, b.p_m),
        ("P_s", a.p_s, b.p_s),
        ("Y", a.y, b.y),
        ("K", a.k, b.k),
        ("M_s", a.m_s, b.m_s),
        ("H_s", a.h_s, b.h_s),
        ("B_d", a.b_d, b.b_d),
        ("final hours", a.final_hours, b.final_hours),
        ("machine hours", a.machine_hours, b.machine_hours),
        ("N_a", a.n_a, b.n_a),
        ("I", a.income, b.income),
        ("v/P_s", a.real_wage, b.real_wage),
        ("provider baskets", a.provider_baskets, b.provider_baskets),
        ("L_s*", a.l_star_s, b.l_star_s),
        ("L_s", a.l_s, b.l_s),
        ("B_s", a.b_s, b.b_s),
        ("L_s^q", a.l_s_q, b.l_s_q),
        ("B_s^q", a.b_s_q, b.b_s_q),
    ] {
        close(&format!("{what}: {name}"), x, y);
    }
    assert_eq!(a.margin_active, b.margin_active, "{what}");
    assert_eq!((a.funded, a.lemma_b1), (b.funded, b.lemma_b1), "{what}");
}

/// A category nobody buys.
fn unbought() -> Category {
    category(0.0, 0.5, &[1.0, 0.7, 2.0])
}

#[test]
fn unused_category_changes_nothing() {
    // A category with z = 0 adds exact zeros to every sum over the basket, so every aggregate
    // is unchanged bit for bit, wherever the category is placed (R1). It is priced all the
    // same.
    for durable in [false, true] {
        let base = CategoryParams {
            rho: if durable { 0.04 } else { 0.0 },
            delta: if durable { 0.35 } else { 1.0 },
            build_lag: if durable { 2 } else { 1 },
            ..fork_economy()
        };
        let reference = interior_1b(base.clone());
        let mut last = base.clone();
        last.categories.push(unbought());
        let mut first = base.clone();
        first.categories.insert(0, unbought());
        for (offset, params) in [(0, last), (1, first)] {
            let e = economy_1b(params.clone());
            let eq = interior_1b(params);
            assert_eq!(aggregates(&eq), aggregates(&reference), "durable {durable}");
            for (j, c) in reference.categories.iter().enumerate() {
                assert_eq!(
                    *c,
                    eq.categories[j + offset],
                    "category {j}, durable {durable}"
                );
            }
            let extra = &eq.categories[if offset == 0 { 4 } else { 0 }];
            assert_eq!((extra.output, extra.share), (0.0, 0.0));
            check_fork_and_bounds(&e, &eq);
        }
    }
}

#[test]
fn split_category() {
    // Food replaced by two copies with weights 0.3 and 0.7 of its own: the same economy.
    // The copies have the same data, so their prices are bit-equal.
    let reference = interior_1b(fork_economy());
    let mut params = fork_economy();
    let food = params.categories[1].clone();
    params.categories[1].weight = 0.3 * food.weight;
    params.categories.insert(
        2,
        Category {
            weight: 0.7 * food.weight,
            ..food
        },
    );
    let e = economy_1b(params.clone());
    let eq = interior_1b(params);
    close_aggregates("split food", &reference, &eq);
    assert_eq!(
        eq.categories[1].price.to_bits(),
        eq.categories[2].price.to_bits()
    );
    close(
        "food's price",
        eq.categories[1].price,
        reference.categories[1].price,
    );
    close(
        "the copies' shares add up",
        eq.categories[1].share + eq.categories[2].share,
        reference.categories[1].share,
    );
    check_identities_1b(&e, &eq);
}

#[test]
fn split_segment() {
    // The middle segment [0.4, 0.75) cut at 0.6 with the same densities on both halves.
    let reference = interior_1b(fork_economy());
    let mut params = fork_economy();
    params.edges = vec![0.0, 0.4, 0.6, 0.75, 1.0];
    for c in &mut params.categories {
        let middle = c.density[1];
        c.density.insert(1, middle);
    }
    let e = economy_1b(params.clone());
    let eq = interior_1b(params);
    close_aggregates("split segment", &reference, &eq);
    for (j, (a, b)) in reference.categories.iter().zip(&eq.categories).enumerate() {
        close(&format!("p_{j}"), b.price, a.price);
        close(&format!("L*_{j}"), b.l_star, a.l_star);
        close(&format!("L-bar_{j}"), b.l_bar, a.l_bar);
    }
    check_identities_1b(&e, &eq);
}

#[test]
fn permutation() {
    // The categories in reverse order: the same aggregates, within FULL (the sums round in
    // another order), and at any fixed x the same category prices, permuted, bit for bit.
    let reference_e = economy_1b(fork_economy());
    let reference = interior_1b(fork_economy());
    let mut params = fork_economy();
    params.categories.reverse();
    let e = economy_1b(params.clone());
    let eq = interior_1b(params);
    close_aggregates("reversed", &reference, &eq);
    for (j, a) in reference.categories.iter().enumerate() {
        let b = &eq.categories[3 - j];
        close(&format!("p_{j}"), b.price, a.price);
        close(&format!("v/p_{j}"), b.real_wage, a.real_wage);
        close(&format!("lambda-tilde_{j}"), b.lambda_tilde, a.lambda_tilde);
    }
    for x in [0.1, 0.5, 0.71, 0.9] {
        let (a, b) = (reference_e.at(x), e.at(x));
        for j in 0..4 {
            assert_eq!(a.prices[j].to_bits(), b.prices[3 - j].to_bits(), "x = {x}");
            assert_eq!(a.human[j].to_bits(), b.human[3 - j].to_bits(), "x = {x}");
            assert_eq!(
                a.machine[j].to_bits(),
                b.machine[3 - j].to_bits(),
                "x = {x}"
            );
        }
    }
}

#[test]
fn unit_rescaling() {
    // Measuring food in units c times as large: (mu, b, z) -> (c mu, c b, z/c). The economy
    // is the same; food's price is c times as large and its v/p 1/c times. With c = 4 every
    // scaling is exact in binary and the whole solve is the same bit for bit; with c = 3 it
    // agrees within FULL.
    let reference = interior_1b(fork_economy());
    for c in [4.0, 3.0] {
        let mut params = fork_economy();
        let food = &mut params.categories[1];
        food.weight /= c;
        food.direct_land *= c;
        for mu in &mut food.density {
            *mu *= c;
        }
        let e = economy_1b(params.clone());
        let eq = interior_1b(params);
        let (a, b) = (&reference.categories[1], &eq.categories[1]);
        if c == 4.0 {
            assert_eq!(aggregates(&eq), aggregates(&reference));
            assert_eq!(b.price.to_bits(), (4.0 * a.price).to_bits());
            assert_eq!(b.real_wage.to_bits(), (a.real_wage / 4.0).to_bits());
            assert_eq!(b.l_star.to_bits(), (4.0 * a.l_star).to_bits());
        } else {
            close_aggregates("c = 3", &reference, &eq);
            close("p", b.price, c * a.price);
            close("v/p", b.real_wage, a.real_wage / c);
            close("L*", b.l_star, c * a.l_star);
        }
        close("share", b.share, a.share);
        check_fork_and_bounds(&e, &eq);
    }
}

#[test]
fn regimes() {
    // docs/unit-1b.md §7's regime rows on the fork economy.
    let regime = |params: CategoryParams| economy_1b(params).solve().expect("finite");
    match regime(CategoryParams {
        workers: 0.2,
        ..fork_economy()
    }) {
        Regime::BoundaryNoMargin { f_at_1 } => close("f(1)", f_at_1, C7_N0_2_F_AT_1),
        other => panic!("expected BoundaryNoMargin, got {other:?}"),
    }
    match regime(CategoryParams {
        lam: 0.6,
        ..fork_economy()
    }) {
        Regime::BoundaryNoMargin { f_at_1 } => close("f(1)", f_at_1, C7_LAM0_6_F_AT_1),
        other => panic!("expected BoundaryNoMargin, got {other:?}"),
    }
    match regime(CategoryParams {
        lam: 0.8,
        ..fork_economy()
    }) {
        Regime::NotViable { d_at_1 } => close("D(1)", d_at_1, C7_LAM0_8_D_AT_1),
        other => panic!("expected NotViable, got {other:?}"),
    }
    match regime(CategoryParams {
        workers: 20.0,
        work_cost: oracle::UniformWorkCost { chi_max: 0.05 },
        ..fork_economy()
    }) {
        Regime::NoInteriorAtZero { f_at_0 } => close("f(1e-12)", f_at_0, C7_N20_F_AT_LO),
        other => panic!("expected NoInteriorAtZero, got {other:?}"),
    }
    let six = CategoryParams {
        workers: 6.0,
        ..fork_economy()
    };
    let eq = interior_1b(six.clone());
    close("x* at N 6", eq.x_star, C7_N6_X_STAR);
    assert_eq!((eq.funded, eq.lemma_b1), (C7_N6_FUNDED, C7_N6_LEMMA_B1));
    assert!(!eq.funded && !eq.lemma_b1);
    check_identities_1b(&economy_1b(six.clone()), &eq);
    // Viability is decided at the top of the line even when no basket category uses the top
    // segment (docs/unit-1b.md §3.2, 1a's convention): the gap economy without its top
    // segment's tasks, at lambda 0.7, where D(1) computes to 0.0 and is NotViable by the
    // convention at zero. It is exactly 0 for the decimal inputs; for the doubles it is
    // +1.7e-17, as goldens/generate_1b.py asserts (docs/unit-1b.md §8).
    let mut no_top = gap_economy(5.0);
    for c in &mut no_top.categories {
        c.density[2] = 0.0;
    }
    no_top.lam = 0.7;
    match regime(no_top) {
        Regime::NotViable { d_at_1 } => assert_eq!(d_at_1, 0.0),
        other => panic!("expected NotViable, got {other:?}"),
    }
}

/// γ = 1 + x with J(x) = 1 + x + x²/2: J(0) is 1, not 0, and `validate` is overridden to
/// pass, so only unit 1b's own check can catch it.
#[derive(Debug)]
struct Offset;

impl Schedule for Offset {
    fn gamma(&self, x: f64) -> f64 {
        1.0 + x
    }
    fn integral(&self, x: f64) -> f64 {
        1.0 + x + x * x / 2.0
    }
    fn validate(&self) -> Result<(), ParamError> {
        Ok(())
    }
}

#[test]
fn validation() {
    let base = fork_economy();
    let invalid = |params: CategoryParams| {
        CategoryEconomy::new(params.clone()).expect_err(&format!("accepted {params:?}"))
    };
    let with_edges = |edges: &[f64]| CategoryParams {
        edges: edges.to_vec(),
        ..base.clone()
    };
    // The task line.
    for (edges, name) in [
        (&[0.1, 0.4, 0.75, 1.0][..], "edges"),
        (&[0.0, 0.4, 0.75, 0.99][..], "edges"),
        (&[0.0, 0.75, 0.4, 1.0][..], "edges"),
        (&[0.0, 0.4, 0.4, 1.0][..], "edges"),
        (&[0.0, f64::NAN, 0.75, 1.0][..], "edges"),
    ] {
        assert_eq!(invalid(with_edges(edges)).name(), name, "{edges:?}");
    }
    let err = invalid(CategoryParams {
        edges: vec![0.0],
        categories: vec![category(1.0, 1.0, &[])],
        ..base.clone()
    });
    assert_eq!(err.name(), "edges");
    // A density of the wrong length is category j's error.
    let mut short = base.clone();
    short.categories[2].density.pop();
    let err = invalid(short);
    assert!(
        matches!(&err, ParamError::Item { kind: "category", index: 2, error } if error.name() == "density"),
        "{err:?}"
    );
    assert_eq!(
        err.to_string(),
        "category 2: density: a category needs one density per segment of the task line"
    );
    // No categories.
    assert_eq!(
        invalid(CategoryParams {
            categories: vec![],
            ..base.clone()
        })
        .name(),
        "categories"
    );
    // Out-of-range weights, densities and land: negative, below the floor, above the
    // ceiling, NaN.
    for bad in [-0.5, 1e-31, SCALE_CEIL * 2.0, f64::NAN] {
        for (field, name) in [(0, "weight"), (1, "direct_land"), (2, "density")] {
            let mut params = base.clone();
            let c = &mut params.categories[1];
            match field {
                0 => c.weight = bad,
                1 => c.direct_land = bad,
                _ => c.density[1] = bad,
            }
            let err = invalid(params);
            assert_eq!(err.name(), name, "{bad:e}");
            assert!(matches!(err, ParamError::Item { index: 1, .. }), "{err:?}");
        }
    }
    // A category with neither tasks nor land.
    let mut empty = base.clone();
    empty.categories[3] = category(0.8, 0.0, &[0.0, 0.0, 0.0]);
    let err = invalid(empty);
    assert!(matches!(err, ParamError::Item { index: 3, .. }), "{err:?}");
    assert_eq!(err.name(), "category");
    // A basket with no direct land: only manufactures (b = 0) is bought.
    let mut landless = base.clone();
    for c in &mut landless.categories[1..] {
        c.weight = 0.0;
    }
    assert_eq!(invalid(landless).name(), "basket");
    // A basket with no work: only a site is bought.
    let mut idle = base.clone();
    idle.categories = vec![
        category(1.0, 1.0, &[0.0, 0.0, 0.0]),
        category(0.0, 0.0, &[1.0, 1.0, 1.0]),
    ];
    let err = invalid(idle);
    assert_eq!(err.name(), "basket");
    assert!(err.to_string().contains("need work"), "{err}");
    // A schedule whose J(0) is not 0.
    let offset = CategoryParams {
        workers: base.workers,
        land: base.land,
        a: base.a,
        lam: base.lam,
        b: base.b,
        schedule: Offset,
        work_cost: base.work_cost,
        rho: base.rho,
        delta: base.delta,
        build_lag: base.build_lag,
        edges: base.edges.clone(),
        categories: base.categories.clone(),
    };
    assert_eq!(CategoryEconomy::new(offset).unwrap_err().name(), "J(0)");
    // 1a's scalars are checked with 1a's names.
    assert_eq!(
        invalid(CategoryParams {
            a: 1.0,
            ..base.clone()
        })
        .name(),
        "a"
    );
    assert_eq!(
        invalid(CategoryParams {
            build_lag: 0,
            ..base.clone()
        }),
        ParamError::BuildLag { value: 0 }
    );
    // -0.0 is stored as +0.0, and every other value passes unchanged.
    let mut signed = base.clone();
    signed.edges[0] = -0.0;
    signed.categories[0].direct_land = -0.0;
    signed.categories[0].density[1] = -0.0;
    signed.categories[2].density[0] = -0.0;
    let e = economy_1b(signed);
    assert_eq!(e.params().edges[0].to_bits(), 0.0f64.to_bits());
    assert_eq!(
        e.params().categories[0].direct_land.to_bits(),
        0.0f64.to_bits()
    );
    assert_eq!(
        e.params().categories[0].density[1].to_bits(),
        0.0f64.to_bits()
    );
    assert_eq!(
        e.params().categories[2].density[0].to_bits(),
        0.0f64.to_bits()
    );
    assert_eq!(economy_1b(base.clone()).params(), &base);
}
