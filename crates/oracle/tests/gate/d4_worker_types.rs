//! d4: worker types (docs/unit-1d.md §3.3's E; §8): the entrant and the trained on B's
//! economy, pooled and at their walls, on the line, at the wall and at the all-human corner,
//! a reserved shortage, and the walk's order and cascade with a third type.

use oracle::{Margin, Regime, SolveError, WorkerParams};

use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1d::*;

/// The E goldens of one instance: (v, P_s, Y, N_a, n_pool, I, entrant hours, trained wage,
/// premium, hours, reserved hours).
type Row = [f64; 11];

fn check_row(what: &str, eq: &oracle::Eq1d, want: Row) {
    let (entrant, trained) = (&eq.workers[0], &eq.workers[1]);
    let got = [
        eq.v,
        eq.p_s,
        eq.y,
        eq.n_a,
        eq.n_pool,
        eq.income,
        entrant.hours,
        trained.wage,
        trained.premium.unwrap(),
        trained.hours,
        trained.reserved_hours,
    ];
    let names = [
        "v",
        "P_s",
        "Y",
        "N_a",
        "n_pool",
        "I",
        "entrant hours",
        "trained wage",
        "trained premium",
        "trained hours",
        "trained reserved hours",
    ];
    for ((name, g), w) in names.iter().zip(got).zip(want) {
        close(&format!("{what} {name}"), g, w);
    }
}

#[test]
fn both_pooled() {
    // E1, N_T 3: the trained's pooled wage 1.5·v covers its reserved work; it sells its other
    // hours to the pool at 1.5 per efficiency hour, premium 1 exactly.
    let (e, eq) = checked_1d(entrant_trained(8.0, 3.0, 1.0, 1.0));
    assert_eq!(eq.margin, Margin::Contestable);
    close("x*", eq.x_star, E1_X_STAR);
    check_row(
        "E1",
        &eq,
        [
            E1_V,
            E1_P_S,
            E1_Y,
            E1_N_A,
            E1_N_POOL,
            E1_INCOME,
            E1_ENTRANT_HOURS,
            E1_TRAINED_WAGE,
            E1_TRAINED_PREMIUM,
            E1_TRAINED_HOURS,
            E1_TRAINED_RESERVED_HOURS,
        ],
    );
    let trained = &eq.workers[1];
    assert!(trained.pooled && !trained.at_wall);
    assert_eq!(trained.premium, Some(1.0));
    assert_eq!(trained.wage.to_bits(), (1.5 * eq.v).to_bits());
    assert!(trained.pool_hours > 0.0 && trained.hours > trained.reserved_hours);
    // The pool clears: n_pool = Σ ε_i·pool hours, and each pooled type's hours are its supply.
    close(
        "pool",
        eq.workers[0].pool_hours + 1.5 * trained.pool_hours,
        eq.n_pool,
    );
    for w in &eq.workers {
        close("hours = supply", w.hours, w.supply);
    }
    check_path_1d(&e);
}

#[test]
fn trained_at_its_wall() {
    // E2, N_T 1: the trained's reserved work takes all its hours at the pooled wage; its wage
    // rises until its supply meets its reserved demand, a scarcity price (SSRN §5 p.13;
    // main.tex:584). Treating types as efficiency units alone would give premium 1 and x* 0.9289.
    let (e, eq) = checked_1d(entrant_trained(8.0, 1.0, 1.0, 1.0));
    assert_eq!(eq.margin, Margin::Contestable);
    close("x*", eq.x_star, E2_X_STAR);
    check_row(
        "E2",
        &eq,
        [
            E2_V,
            E2_P_S,
            E2_Y,
            E2_N_A,
            E2_N_POOL,
            E2_INCOME,
            E2_ENTRANT_HOURS,
            E2_TRAINED_WAGE,
            E2_TRAINED_PREMIUM,
            E2_TRAINED_HOURS,
            E2_TRAINED_RESERVED_HOURS,
        ],
    );
    let trained = &eq.workers[1];
    assert!(!trained.pooled && trained.at_wall);
    assert!(trained.premium.unwrap() > 2.0);
    assert_eq!(trained.hours, trained.reserved_hours);
    assert!(eq.residuals.reserved > 0.0 && eq.residuals.reserved <= 8.0 * f64::EPSILON);
    close(
        "v_T = zeta nu P_s",
        trained.clearing_real_wage * 1.2 * eq.p_s,
        trained.wage,
    );
    // Low on the line Y is large and the trained's reserved demand exceeds N_T: short.
    assert_eq!(eq.f_line_lo, None);
    assert_eq!(eq.f_line_0, None);
    assert!(e.at_with(oracle::BRACKET_LO, 0).short.is_some());
    // Without reserved hours the trained would be one more efficiency unit in the pool.
    let pure = solved_1d(WorkerParams {
        reserved: vec![vec![0.0; 2]; 3],
        ..entrant_trained(8.0, 1.0, 1.0, 1.0)
    });
    assert_eq!(pure.workers[1].premium, Some(1.0));
    assert!((pure.x_star - eq.x_star).abs() > 0.05);
    check_path_1d(&e);
}

#[test]
fn the_wall_with_types() {
    // E3 (N_T 2, η 0.3): the pool at the wall and the trained at its own wall. E4 (N_T 3):
    // both pooled at the wall.
    for (what, params, want, walled) in [
        (
            "E3",
            entrant_trained(8.0, 2.0, 0.3, 1.0),
            [
                E3_V,
                E3_P_S,
                E3_Y,
                E3_N_A,
                E3_N_POOL,
                E3_INCOME,
                E3_ENTRANT_HOURS,
                E3_TRAINED_WAGE,
                E3_TRAINED_PREMIUM,
                E3_TRAINED_HOURS,
                E3_TRAINED_RESERVED_HOURS,
            ],
            true,
        ),
        (
            "E4",
            entrant_trained(8.0, 3.0, 0.3, 1.0),
            [
                E4_V,
                E4_P_S,
                E4_Y,
                E4_N_A,
                E4_N_POOL,
                E4_INCOME,
                E4_ENTRANT_HOURS,
                E4_TRAINED_WAGE,
                E4_TRAINED_PREMIUM,
                E4_TRAINED_HOURS,
                E4_TRAINED_RESERVED_HOURS,
            ],
            false,
        ),
    ] {
        let (e, eq) = checked_1d(params);
        assert_eq!(eq.margin, Margin::Wall, "{what}");
        check_row(what, &eq, want);
        assert_eq!(eq.workers[1].pooled, !walled, "{what}");
        assert!(eq.workers.iter().all(|w| w.at_wall), "{what}");
        check_path_1d(&e);
    }
}

#[test]
fn all_human_with_a_walled_type() {
    // E5: abundant entrants (N_E 40, χ_max 0.05) hold every contestable task below the bottom
    // task's replacement value; the trained at its wall.
    let (e, eq) = checked_1d(entrant_trained(40.0, 2.0, 1.0, 0.05));
    assert_eq!(eq.margin, Margin::AllHuman);
    check_row(
        "E5",
        &eq,
        [
            E5_V,
            E5_P_S,
            E5_Y,
            E5_N_A,
            E5_N_POOL,
            E5_INCOME,
            E5_ENTRANT_HOURS,
            E5_TRAINED_WAGE,
            E5_TRAINED_PREMIUM,
            E5_TRAINED_HOURS,
            E5_TRAINED_RESERVED_HOURS,
        ],
    );
    close(
        "replacement bottom",
        eq.replacement_bottom,
        E5_REPLACEMENT_BOTTOM,
    );
    assert!(!eq.workers[1].pooled && eq.workers[1].premium.unwrap() > 19.0);
    assert!(eq.v < eq.replacement_bottom);
    check_path_1d(&e);
}

#[test]
fn reserved_shortage() {
    // E6, N_T 0.5: the trained's reserved demand exceeds its workers along the whole path; no
    // wage clears it. A unit that capped the trained's supply at N_T and carried on would
    // report an equilibrium.
    let e = economy_1d(entrant_trained(8.0, 0.5, 1.0, 1.0));
    match e.solve() {
        Err(SolveError::LaborShort { excess, reserved }) => {
            assert_eq!(excess, f64::INFINITY);
            assert_eq!(reserved, Some(1));
        }
        other => panic!("{other:?}"),
    }
    let (values, changes) = sequence_1d(&e);
    assert!(values.iter().all(|&f| f == f64::INFINITY));
    assert_eq!(changes, 0);
    let one = e.at_with(1.0, 0);
    close("D_T(1)", one.reserved_demand[1], E6_TRAINED_DEMAND_AT_1);
    assert_eq!(one.short, Some(oracle::Shortage::Reserved(1)));
    assert!(one.p_s.is_nan() && one.excess_demand() == f64::INFINITY);
    // With the trained and the master both short at the end of the path, `reserved` names the
    // first of them (§5.3 step 3), the trained.
    let e = economy_1d(three_types(0.01, 0.01));
    let end = e.at_with(1.0, 0);
    assert!(end.reserved_demand[1] > 0.01 && end.reserved_demand[2] > 0.01);
    match e.solve() {
        Err(SolveError::LaborShort { excess, reserved }) => {
            assert_eq!((excess, reserved), (f64::INFINITY, Some(1)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_end_of_the_wall_counts_efficiency() {
    // E9 (§12 item 18): E4 with χ_max 3 for both types, so that supply is not saturated at
    // ω_∞ and each type's efficiency enters f_∞ through the walk (§4.5): the trained is walled
    // at the equilibrium and pooled at the end of the wall, where ρ_∞ carries ε_T·R_ŷT.
    let mut params = entrant_trained(8.0, 3.0, 0.3, 3.0);
    params.worker_types[1].work_cost.chi_max = 3.0;
    let (e, eq) = checked_1d(params);
    assert_eq!(eq.margin, Margin::Wall);
    assert!(!eq.workers[1].pooled);
    let end = e.wall_end();
    for (name, got, want) in [
        ("v", eq.v, E9_V),
        ("P_s", eq.p_s, E9_P_S),
        ("f_end", eq.f_end, E9_F_END),
        ("f_end (wall_end)", end.excess, E9_F_END),
        ("omega_end", end.omega.unwrap(), E9_OMEGA_END),
        ("trained wage", eq.workers[1].wage, E9_TRAINED_WAGE),
    ] {
        close(name, got, want);
    }
    check_path_1d(&e);
}

#[test]
fn lemma_b1_counts_the_support() {
    // Lemma B.1's funding condition is T > ν·P_s(1) with ν = Σ ν_i·N_i (§4.7): E1's economy with
    // both supports 0.4 has ν = 4.4 and Σ N_i = 11, and T = 10 lies between ν·P_s(1) and
    // Σ N_i·P_s(1).
    let mut params = entrant_trained(8.0, 3.0, 1.0, 1.0);
    for t in &mut params.worker_types {
        t.support = 0.4;
    }
    let (e, eq) = checked_1d(params.clone());
    let one = e.at_with(1.0, e.machines().envelope().last());
    let workers: f64 = params.worker_types.iter().map(|t| t.workers).sum();
    assert!(one.excess_demand() < 0.0);
    assert!(e.support() * one.p_s < params.land && params.land < workers * one.p_s);
    assert!(eq.lemma_b1);
}

#[test]
fn walk_order() {
    // E7: the master's threshold (the P_s above which it is walled) is below the trained's, so
    // the walk visits it first: master at its wall, trained pooled. A walk in index order that
    // stops at the first pooled type would find the master pooled.
    let (e, eq) = checked_1d(three_types(3.0, 0.6));
    assert_eq!(eq.margin, Margin::Contestable);
    let (trained, master) = (&eq.workers[1], &eq.workers[2]);
    assert!(trained.pooled && !master.pooled);
    for (name, got, want) in [
        ("x*", eq.x_star, E7_X_STAR),
        ("v", eq.v, E7_V),
        ("P_s", eq.p_s, E7_P_S),
        ("trained wage", trained.wage, E7_TRAINED_WAGE),
        ("master wage", master.wage, E7_MASTER_WAGE),
        ("master premium", master.premium.unwrap(), E7_MASTER_PREMIUM),
        ("master hours", master.hours, E7_MASTER_HOURS),
    ] {
        close(name, got, want);
    }
    // The thresholds t_i = ε_i·v/(ζ_i·ν_i): the master's below P_s below the trained's.
    let threshold = |i: usize, w: &oracle::WorkerEq| {
        let t = &e.params().worker_types[i];
        t.efficiency * eq.v / (w.clearing_real_wage * t.support)
    };
    close(
        "trained threshold",
        threshold(1, trained),
        E7_TRAINED_THRESHOLD,
    );
    close(
        "master threshold",
        threshold(2, master),
        E7_MASTER_THRESHOLD,
    );
    assert!(E7_MASTER_THRESHOLD < eq.p_s && eq.p_s < E7_TRAINED_THRESHOLD);
    // In index order the trained comes first; its threshold is above P with every type pooled.
    let q = e.at_with(eq.x_star, eq.technique);
    let mut p_all = q.base_p_s;
    for (i, t) in e.params().worker_types.iter().enumerate() {
        p_all += t.efficiency * eq.v * e.reserved_per_basket()[i];
    }
    assert!(p_all < E7_TRAINED_THRESHOLD && p_all > E7_MASTER_THRESHOLD);
    check_path_1d(&e);
}

#[test]
fn walk_cascade() {
    // E8: walling the master raises P_s past the trained's threshold, so the walk walls the
    // trained too. A walk that did not recompute P_s after walling a type would find the
    // trained pooled, premium 1 not 1.0021.
    let (e, eq) = checked_1d(three_types(1.53, 0.5));
    assert_eq!(eq.margin, Margin::Contestable);
    let (trained, master) = (&eq.workers[1], &eq.workers[2]);
    assert!(!trained.pooled && !master.pooled);
    for (name, got, want) in [
        ("x*", eq.x_star, E8_X_STAR),
        ("v", eq.v, E8_V),
        ("P_s", eq.p_s, E8_P_S),
        ("trained wage", trained.wage, E8_TRAINED_WAGE),
        (
            "trained premium",
            trained.premium.unwrap(),
            E8_TRAINED_PREMIUM,
        ),
        ("master wage", master.wage, E8_MASTER_WAGE),
        ("master premium", master.premium.unwrap(), E8_MASTER_PREMIUM),
    ] {
        close(name, got, want);
    }
    // P with the master alone walled is above the trained's threshold, P with every type pooled
    // below it.
    let q = e.at_with(eq.x_star, eq.technique);
    let types = &e.params().worker_types;
    let r = e.reserved_per_basket();
    let pooled = |i: usize| types[i].efficiency * eq.v * r[i];
    let rate = |i: usize, w: &oracle::WorkerEq| w.clearing_real_wage * types[i].support * r[i];
    let p_all = q.base_p_s + pooled(0) + pooled(1) + pooled(2);
    let p_master = (q.base_p_s + pooled(0) + pooled(1)) / (1.0 - rate(2, master));
    close("P, the master walled", p_master, E8_P_S_MASTER_WALLED);
    let t_trained = types[1].efficiency * eq.v / (trained.clearing_real_wage * types[1].support);
    close("trained threshold", t_trained, E8_TRAINED_THRESHOLD);
    assert!(p_all < t_trained && t_trained < p_master);
    assert!(trained.premium.unwrap() > 1.0 && trained.premium.unwrap() < 1.01);
    check_path_1d(&e);
}

#[test]
fn supply_and_exit_per_type() {
    // At every E equilibrium, each type's supply is N_i·F_i(ln(1 + v_i/(ν_i·P_s))) and its exit
    // value (e^χ − 1)·ν_i·P_s equals v_i at its marginal work cost (SSRN eq 8-10 p.11; A.1 p.25).
    for params in [
        entrant_trained(8.0, 3.0, 1.0, 1.0),
        entrant_trained(8.0, 1.0, 1.0, 1.0),
        entrant_trained(8.0, 2.0, 0.3, 1.0),
        entrant_trained(8.0, 3.0, 0.3, 1.0),
        entrant_trained(40.0, 2.0, 1.0, 0.05),
        three_types(3.0, 0.6),
        three_types(1.53, 0.5),
    ] {
        let eq = solved_1d(params.clone());
        for (i, w) in eq.workers.iter().enumerate() {
            let t = &params.worker_types[i];
            let chi = rustyecon_core::num::ln1p(w.wage / (t.support * eq.p_s));
            assert_eq!(w.marginal_work_cost.to_bits(), chi.to_bits());
            near(
                "supply",
                w.supply,
                t.workers * (chi / t.work_cost.chi_max).clamp(0.0, 1.0),
                FULL * t.workers,
            );
            close(
                "exit value",
                rustyecon_core::num::expm1(chi) * t.support * eq.p_s,
                w.wage,
            );
        }
    }
}

#[test]
fn splitting_a_type_changes_nothing() {
    // E1 with the entrant split into two types of half its workers each: the same equilibrium
    // within 1e-12, and the two halves' hours equal (§2.2: pooled types are perfect
    // substitutes, and the pool's hours are split by their net supplies).
    let whole = solved_1d(entrant_trained(8.0, 3.0, 1.0, 1.0));
    let mut params = entrant_trained(8.0, 3.0, 1.0, 1.0);
    let half = worker(4.0, 1.0, 1.0, 1.0);
    params.worker_types = vec![half, half, params.worker_types[1]];
    params.reserved = params
        .reserved
        .iter()
        .map(|row| vec![row[0], row[0], row[1]])
        .collect();
    let (_, split) = checked_1d(params);
    for (name, a, b) in [
        ("x*", whole.x_star, split.x_star),
        ("v", whole.v, split.v),
        ("P_s", whole.p_s, split.p_s),
        ("Y", whole.y, split.y),
        ("N_a", whole.n_a, split.n_a),
        ("I", whole.income, split.income),
        ("trained wage", whole.workers[1].wage, split.workers[2].wage),
        (
            "trained hours",
            whole.workers[1].hours,
            split.workers[2].hours,
        ),
    ] {
        close(name, b, a);
    }
    close("halves", split.workers[0].hours, split.workers[1].hours);
    close(
        "halves sum",
        split.workers[0].hours + split.workers[1].hours,
        whole.workers[0].hours,
    );
    assert!(matches!(
        economy_1d(entrant_trained(8.0, 3.0, 1.0, 1.0)).solve(),
        Ok(Regime::Interior(_))
    ));
}
