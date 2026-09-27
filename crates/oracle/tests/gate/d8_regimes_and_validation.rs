//! d8: regimes, validation and reductions (docs/unit-1d.md §8): exact zeros at the junctions,
//! viability unchanged by the worker types, every validation rule, the worker types permuted,
//! efficiency measured in other units, the walk's ceiling, the edge of a reserved shortage on
//! the line and in a tie's split, and the rules at exact equality.

use oracle::{
    MachineEconomy, Margin, ParamError, Regime, Shortage, SolveError, WorkerEconomy, WorkerParams,
    WorkerPoint, BRACKET_LO,
};
use rustyecon_core::num;

use crate::g8_regimes::saturated;
use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1b::category;
use crate::support_1c::*;
use crate::support_1d::*;

/// The one-type economy `build(N)` with N chosen so that f at the point `at` evaluates is
/// exactly 0 in f64: supply there is N·c with c = F(ln(1 + v/P_s)) independent of N (and n_D
/// too), so among the doubles next to n_D/c one has N·c = n_D exactly.
fn exact_zero(
    what: &str,
    build: impl Fn(f64) -> WorkerParams,
    at: impl Fn(&WorkerEconomy) -> WorkerPoint,
) -> WorkerParams {
    let q = at(&economy_1d(build(1.0)));
    let c = q.n_s;
    let mut n = q.n_d / c;
    for _ in 0..64 {
        if n * c == q.n_d {
            return build(n);
        }
        n = if n * c < q.n_d {
            n.next_up()
        } else {
            n.next_down()
        };
    }
    panic!("no N gives f = 0 exactly at {what}");
}

/// G1 (χ_max 1) with f on the line at x exactly 0 in f64.
fn exact_zero_at(x: f64) -> WorkerParams {
    exact_zero(
        &format!("x = {x}"),
        |n| goodspace(n, 0.05, 1.0),
        |e| e.at_with(x, 0),
    )
}

#[test]
fn exact_zeros_at_the_junctions() {
    // f_line(1) = 0 exactly is the wall at v(1), f_line(0) = 0 the all-human corner at v(0),
    // and f_line(lo) = 0 a root at lo (§5.3 step 3), each without a bisection step.
    let params = exact_zero_at(1.0);
    let e = economy_1d(params.clone());
    let line = e.at_with(1.0, 0);
    assert_eq!(line.excess_demand(), 0.0);
    let eq = solved_1d(params);
    assert_eq!(eq.margin, Margin::Wall);
    assert_eq!(eq.v.to_bits(), line.v.to_bits());
    assert_eq!((eq.bisection_steps, eq.f_line_1), (0, Some(0.0)));
    assert!(eq.f_end < 0.0);
    check_identities_1d(&e, &eq);
    let params = exact_zero_at(0.0);
    let e = economy_1d(params.clone());
    let line = e.at_with(0.0, 0);
    assert_eq!(line.excess_demand(), 0.0);
    let eq = solved_1d(params);
    assert_eq!(eq.margin, Margin::AllHuman);
    assert_eq!(eq.v.to_bits(), line.v.to_bits());
    assert_eq!((eq.bisection_steps, eq.f_line_0), (0, Some(0.0)));
    check_identities_1d(&e, &eq);
    // With χ_max 0.05 supply saturates at N at lo, where n_D does not depend on N: N = n_D(lo)
    // makes f_line(lo) exactly 0, and n_D falls from x = 0 to lo, so f_line(0) > 0.
    let n_lo = economy_1d(from_1a(saturated(1.0)))
        .at_with(BRACKET_LO, 0)
        .n_d;
    let e = economy_1d(from_1a(saturated(n_lo)));
    assert_eq!(e.at_with(BRACKET_LO, 0).excess_demand(), 0.0);
    let eq = solved_1d(from_1a(saturated(n_lo)));
    assert_eq!((eq.margin, eq.x_star), (Margin::Contestable, BRACKET_LO));
    assert_eq!(eq.one_minus_x_star, 1.0 - BRACKET_LO);
    assert_eq!(eq.bisection_steps, 0);
    assert!(eq.f_line_0.unwrap() > 0.0);
    check_identities_1d(&e, &eq);
    // The saturated knife edge (§12 items 4 and 17): with supply saturated at N = n_D(1), f is
    // 0 on the whole wall and at its end. The wall's piece starts at an exact zero, so f_∞ = 0
    // counts on the negative side and the junction is the equilibrium: x* = 1 and v = v(1),
    // without a bisection step, as with the unsaturated exact zero above (the exact economy on
    // these doubles has its root within 5e-17 of x = 1: not decidable in f64, §5.5).
    let n_1 = economy_1d(from_1a(saturated(1.0))).at_with(1.0, 0).n_d;
    let e = economy_1d(from_1a(saturated(n_1)));
    let line = e.at_with(1.0, 0);
    assert_eq!(line.excess_demand(), 0.0);
    assert_eq!(e.wall_end().excess, 0.0);
    let (values, changes) = sequence_1d(&e);
    assert_eq!((values[values.len() - 2], changes), (0.0, 1));
    let eq = solved_1d(from_1a(saturated(n_1)));
    assert_eq!(eq.margin, Margin::Wall);
    assert_eq!(eq.v.to_bits(), line.v.to_bits());
    assert_eq!(
        (eq.bisection_steps, eq.f_line_1, eq.f_end),
        (0, Some(0.0), 0.0)
    );
    check_identities_1d(&e, &eq);
}

#[test]
fn not_viable_stands() {
    // 1c's NotViable rows with F's worker types and human-required hours added are still
    // NotViable with the same d(1): the new inputs do not enter the machine block (decision 64).
    let c = MachineEconomy::new(m4(50.0)).unwrap().solve().unwrap();
    let d = economy_1d(full(4.0, 3.0, 50.0)).solve().unwrap();
    match (c, d) {
        (Regime::NotViable { d_at_1: a }, Regime::NotViable { d_at_1: b }) => {
            assert_eq!(a.to_bits(), b.to_bits());
            close("M4 at eta 50", b, crate::goldens_1c::M4_REG_ETA50_D_AT_1);
        }
        other => panic!("{other:?}"),
    }
    // 1a's λ 0.8 row, with a trained type, its reserved hours and a tail added.
    let mut params = goodspace(4.0, 0.8, 1.0);
    params.worker_types.push(worker(1.0, 0.8, 1.5, 1.2));
    params.reserved = vec![vec![0.0, 0.1], vec![0.0, 0.0]];
    params.human_required = vec![0.2, 0.0];
    let one = oracle::Economy::new(oracle::Params {
        lam: 0.8,
        ..appendix_b()
    })
    .unwrap()
    .solve()
    .unwrap();
    match (one, economy_1d(params).solve().unwrap()) {
        (Regime::NotViable { d_at_1: a }, Regime::NotViable { d_at_1: b }) => {
            assert_eq!(a.to_bits(), b.to_bits())
        }
        other => panic!("{other:?}"),
    }
}

/// Asserts a rejection naming `name`, inside `item` when given.
fn rejected(what: &str, params: WorkerParams, item: Option<(&str, usize)>, name: &str) {
    let err = WorkerEconomy::new(params).expect_err(what);
    match item {
        Some((kind, index)) => match &err {
            ParamError::Item {
                kind: k,
                index: i,
                error,
            } => {
                assert_eq!((*k, *i), (kind, index), "{what}: {err}");
                assert_eq!(error.name(), name, "{what}: {err}");
            }
            other => panic!("{what}: {other:?}"),
        },
        None => assert_eq!(err.name(), name, "{what}: {err}"),
    }
    assert!(!err.to_string().is_empty());
}

#[test]
fn validation() {
    let base = entrant_trained(8.0, 3.0, 1.0, 1.0);
    let with = |f: &dyn Fn(&mut WorkerParams)| {
        let mut p = base.clone();
        f(&mut p);
        p
    };
    rejected(
        "no types",
        with(&|p| p.worker_types.clear()),
        None,
        "worker types",
    );
    rejected(
        "N 0",
        with(&|p| p.worker_types[1].workers = 0.0),
        Some(("worker type", 1)),
        "workers",
    );
    rejected(
        "chi_max NaN",
        with(&|p| p.worker_types[0].work_cost.chi_max = f64::NAN),
        Some(("worker type", 0)),
        "chi_max",
    );
    rejected(
        "efficiency -1",
        with(&|p| p.worker_types[1].efficiency = -1.0),
        Some(("worker type", 1)),
        "efficiency",
    );
    rejected(
        "efficiency below the floor",
        with(&|p| p.worker_types[1].efficiency = 1e-31),
        Some(("worker type", 1)),
        "efficiency",
    );
    rejected(
        "support 0",
        with(&|p| p.worker_types[1].support = 0.0),
        Some(("worker type", 1)),
        "support",
    );
    rejected(
        "no pooled type",
        with(&|p| {
            for t in &mut p.worker_types {
                t.efficiency = 0.0;
            }
            p.reserved = vec![vec![0.1, 0.1]; 3];
        }),
        None,
        "worker types",
    );
    rejected(
        "human_required's length",
        with(&|p| {
            p.human_required.pop();
        }),
        None,
        "human_required",
    );
    rejected(
        "human_required too long",
        with(&|p| p.human_required.push(0.0)),
        None,
        "human_required",
    );
    rejected(
        "human_required -1",
        with(&|p| p.human_required[1] = -1.0),
        Some(("category", 1)),
        "human_required",
    );
    rejected(
        "reserved's rows",
        with(&|p| {
            p.reserved.pop();
        }),
        None,
        "reserved",
    );
    rejected(
        "reserved's entries",
        with(&|p| p.reserved[0].push(0.0)),
        None,
        "reserved",
    );
    rejected(
        "reserved NaN",
        with(&|p| p.reserved[2][1] = f64::NAN),
        Some(("category", 2)),
        "reserved",
    );
    rejected(
        "a type with no work",
        with(&|p| {
            p.worker_types[1].efficiency = 0.0;
            p.reserved = vec![vec![0.0, 0.0]; 3];
        }),
        Some(("worker type", 1)),
        "reserved",
    );
    // A type with ε = 0 and reserved work is valid.
    assert!(WorkerEconomy::new(with(&|p| p.worker_types[1].efficiency = 0.0)).is_ok());
    // A category with no tasks and no land is priced by its human-required or reserved hours,
    // which 1c's rule would reject; with neither it is rejected as in 1c.
    let bare = |required: f64, reserved: f64| {
        let mut p = base.clone();
        p.categories.push(category(0.5, 0.0, &[0.0]));
        p.intermediate = no_inputs(4);
        p.human_required.push(required);
        p.reserved.push(vec![0.0, reserved]);
        p
    };
    for (required, reserved) in [(0.3, 0.0), (0.0, 0.3)] {
        assert!(WorkerEconomy::new(bare(required, reserved)).is_ok());
        let mut machine = WorkerEconomy::new(bare(required, reserved))
            .unwrap()
            .machines()
            .params()
            .clone();
        machine.workers = 8.0;
        assert!(MachineEconomy::new(machine).is_err(), "1c would reject it");
    }
    rejected(
        "unpriced",
        bare(0.0, 0.0),
        Some(("category", 3)),
        "category",
    );
    // The basket needs pool work at x = 0: a basket of reserved work alone has none, one of
    // the common human-required hours alone has.
    let reserved_only = WorkerParams {
        categories: vec![category(1.0, 0.0, &[0.0]), category(1.0, 1.0, &[0.0])],
        intermediate: no_inputs(2),
        human_required: vec![0.0, 0.0],
        reserved: vec![vec![0.0, 0.5], vec![0.0, 0.0]],
        ..base.clone()
    };
    rejected("no pool work", reserved_only.clone(), None, "basket");
    let tail_only = WorkerParams {
        human_required: vec![0.5, 0.0],
        ..reserved_only
    };
    let e = WorkerEconomy::new(tail_only).unwrap();
    assert!(matches!(
        e.solve(),
        Ok(Regime::Interior(_)) | Err(SolveError::LaborShort { .. })
    ));
    // −0.0 is stored as +0.0.
    let e = WorkerEconomy::new(with(&|p| {
        p.worker_types[1].efficiency = -0.0;
        p.human_required[1] = -0.0;
        p.reserved[2][0] = -0.0;
    }))
    .unwrap();
    let p = e.params();
    for value in [
        p.worker_types[1].efficiency,
        p.human_required[1],
        p.reserved[2][0],
    ] {
        assert_eq!(value.to_bits(), 0.0f64.to_bits());
    }
    // 1c's own checks come first, in 1c's order, and name what 1c names.
    rejected("T 0", with(&|p| p.land = 0.0), None, "land");
    rejected(
        "density",
        with(&|p| p.categories[0].density = vec![]),
        Some(("category", 0)),
        "density",
    );
    rejected(
        "no basket land",
        with(&|p| p.categories[2].weight = 0.0),
        None,
        "basket",
    );
}

#[test]
fn permutation_of_types() {
    // Reversing the worker types (and the reserved hours' columns) gives the same equilibrium
    // within 1e-12, permuted.
    for params in [
        three_types(3.0, 0.6),
        three_types(1.53, 0.5),
        entrant_trained(8.0, 1.0, 1.0, 1.0),
        full(4.0, 3.0, 1.0),
    ] {
        let a = solved_1d(params.clone());
        let mut reversed = params.clone();
        reversed.worker_types.reverse();
        for row in &mut reversed.reserved {
            row.reverse();
        }
        let (_, b) = checked_1d(reversed);
        for (name, x, y) in [
            ("x*", a.x_star, b.x_star),
            ("v", a.v, b.v),
            ("P_s", a.p_s, b.p_s),
            ("Y", a.y, b.y),
            ("N_a", a.n_a, b.n_a),
            ("I", a.income, b.income),
        ] {
            close(name, y, x);
        }
        let k = a.workers.len();
        for (i, w) in a.workers.iter().enumerate() {
            let u = &b.workers[k - 1 - i];
            assert_eq!(w.pooled, u.pooled);
            close("wage", u.wage, w.wage);
            close("hours", u.hours, w.hours);
        }
        for (c, d) in a.categories.iter().zip(&b.categories) {
            close("price", d.price, c.price);
        }
    }
}

/// Efficiency hours measured in units of c: every ε_i, density, L^H and machine labour
/// coefficient times c, and η over c, so that M_j = μ·J is unchanged (§8, d8).
fn in_units_of(mut params: WorkerParams, c: f64) -> WorkerParams {
    for t in &mut params.worker_types {
        t.efficiency *= c;
    }
    for cat in &mut params.categories {
        for mu in &mut cat.density {
            *mu *= c;
        }
    }
    for h in &mut params.human_required {
        *h *= c;
    }
    for t in &mut params.machine_types {
        t.operating.labor *= c;
        t.build.labor *= c;
    }
    params.schedule.eta /= c;
    params
}

#[test]
fn efficiency_units() {
    // v is per efficiency hour, so it scales as 1/c; the types' wages, x*, the prices and the
    // quantities in hours do not move. At c = 4 every scaling is exact: bit for bit.
    for params in [
        entrant_trained(8.0, 1.0, 1.0, 1.0),
        entrant_trained(8.0, 2.0, 0.3, 1.0),
        full(4.0, 3.0, 1.0),
        full(4.0, 3.0, 0.25),
    ] {
        let a = solved_1d(params.clone());
        let b = solved_1d(in_units_of(params.clone(), 4.0));
        assert_eq!(a.margin, b.margin);
        assert_eq!((b.v * 4.0).to_bits(), a.v.to_bits());
        assert_eq!((b.n_pool / 4.0).to_bits(), a.n_pool.to_bits());
        for (name, x, y) in [
            ("x*", a.x_star, b.x_star),
            ("P_s", a.p_s, b.p_s),
            ("Y", a.y, b.y),
            ("N_a", a.n_a, b.n_a),
            ("I", a.income, b.income),
            ("interest", a.interest, b.interest),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "{name}");
        }
        for (w, u) in a.workers.iter().zip(&b.workers) {
            assert_eq!(w.wage.to_bits(), u.wage.to_bits());
            assert_eq!(w.hours.to_bits(), u.hours.to_bits());
            assert_eq!(w.premium.map(f64::to_bits), u.premium.map(f64::to_bits));
        }
        for (c, d) in a.categories.iter().zip(&b.categories) {
            assert_eq!(c.price.to_bits(), d.price.to_bits());
        }
        let three = solved_1d(in_units_of(params, 3.0));
        close("v at c = 3", three.v * 3.0, a.v);
        close("x* at c = 3", three.x_star, a.x_star);
        close("P_s at c = 3", three.p_s, a.p_s);
        close("N_a at c = 3", three.n_a, a.n_a);
        close(
            "trained wage at c = 3",
            three.workers[1].wage,
            a.workers[1].wage,
        );
    }
}

#[test]
fn walk_ceiling() {
    // A type that does only reserved work, whose walled wage per unit of P_s times its hours
    // per basket reaches 1 (ζ·ν·R ≥ 1): its wage would cost more than the basket it buys, so
    // no finite price system exists. Short everywhere: LaborShort with excess +∞ and no type
    // named.
    let mut params = baumol_one(1.0, 8.0, 0.25);
    params.worker_types.push(worker(10.0, 3.0, 0.0, 2.0));
    params.reserved = vec![vec![0.0, 0.5], vec![0.0, 0.0], vec![0.0, 0.0]];
    let e = economy_1d(params);
    for x in [0.0, BRACKET_LO, 0.5, 1.0] {
        let q = e.at_with(x, 0);
        assert_eq!(q.short, Some(Shortage::Ceiling), "x = {x}");
        assert_eq!(q.excess_demand(), f64::INFINITY);
        let zeta = q.clearing[1];
        assert!(zeta * 2.0 * e.reserved_per_basket()[1] >= 1.0);
        assert!(q.reserved_demand[1] < 10.0);
    }
    match e.solve() {
        Err(SolveError::LaborShort { excess, reserved }) => {
            assert_eq!((excess, reserved), (f64::INFINITY, None));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(e.wall_end().omega, None);
}

#[test]
fn the_edge_of_a_reserved_shortage() {
    // J1 (§12 item 16): E's economy with abundant entrants (N_E 40, χ_max 0.05) and N_T 1. Low
    // on the line Y is large and the trained's reserved demand D_T = 0.12·Y exceeds N_T (short,
    // +∞); from the double where it has fallen to N_T the pool is far in excess supply. At that
    // edge the trained's supply is vertical at N_T for every real wage above expm1(0.8), and the
    // one that clears the pool is the equilibrium: the trained walled at κ·ν·P_s with all its
    // hours reserved, "a scarcity price … set by demand" (main.tex:584). A unit that refuses the
    // jump reports LaborShort; one that bisects onto it at the type's own ζ leaves the pool in
    // excess supply.
    let (e, eq) = checked_1d(entrant_trained(40.0, 1.0, 1.0, 0.05));
    let (values, changes) = sequence_1d(&e);
    assert_eq!((values[0], changes), (f64::INFINITY, 1));
    check_count_1d(&e, &e.solve());
    assert_eq!(eq.margin, Margin::Contestable);
    let trained = &eq.workers[1];
    assert!(trained.edge && !trained.pooled && !eq.workers[0].edge);
    assert!(trained.clearing_real_wage > num::expm1(0.8));
    // x* is the edge's finite side: short one double below, and in excess supply at the
    // trained's own ζ.
    assert_eq!(
        e.at_with(eq.x_star.next_down(), 0).short,
        Some(Shortage::Reserved(1))
    );
    assert!(e.at_with(eq.x_star, 0).excess_demand() < -10.0);
    for (name, got, want) in [
        ("x*", eq.x_star, J1_X_STAR),
        ("v", eq.v, J1_V),
        ("P_s", eq.p_s, J1_P_S),
        ("Y", eq.y, J1_Y),
        ("N_a", eq.n_a, J1_N_A),
        ("n_pool", eq.n_pool, J1_N_POOL),
        ("entrant hours", eq.workers[0].hours, J1_ENTRANT_HOURS),
        ("trained wage", trained.wage, J1_TRAINED_WAGE),
        ("kappa", trained.clearing_real_wage, J1_TRAINED_CLEARING),
        ("premium", trained.premium.unwrap(), J1_TRAINED_PREMIUM),
    ] {
        close(name, got, want);
    }
    close("trained hours = N_T", trained.hours, 1.0);
    // The steps count both bisections: x to adjacent doubles in [lo, 1] (about 53) and κ on
    // bit patterns (up to 64).
    assert!(eq.bisection_steps > 64, "{}", eq.bisection_steps);
    check_path_1d(&e);
    // J1b: the same edge inside [0, 1e-12], where the solve bisects on bit patterns: N_T the
    // double midway between the trained's reserved demand at x = 0 and at lo (Y falls by about
    // 2e-13 relative across the stretch, and D_T with it), so f(0) is +∞ and f(lo) far below 0.
    let e = economy_1d(entrant_trained(40.0, J1B_N_T, 1.0, 0.05));
    let (d_0, d_lo) = (
        e.at_with(0.0, 0).reserved_demand[1],
        e.at_with(BRACKET_LO, 0).reserved_demand[1],
    );
    assert!(d_lo < J1B_N_T && J1B_N_T < d_0);
    let (values, changes) = sequence_1d(&e);
    assert_eq!((values[0], changes), (f64::INFINITY, 1));
    assert!(values[1] < -10.0);
    let (_, eq) = checked_1d(entrant_trained(40.0, J1B_N_T, 1.0, 0.05));
    assert!(eq.workers[1].edge && eq.x_star > 0.0 && eq.x_star < BRACKET_LO);
    assert!(eq.bisection_steps <= 2 * 64);
    // x* is resolved to the rounding of D_T over its slope, absolute (§5.5, as W5's): 8 ulps.
    let slope = (d_0 - d_lo) / BRACKET_LO;
    near(
        "x*",
        eq.x_star,
        J1B_X_STAR,
        8.0 * f64::EPSILON * J1B_N_T / slope,
    );
    for (name, got, want) in [
        ("v", eq.v, J1B_V),
        ("P_s", eq.p_s, J1B_P_S),
        ("trained wage", eq.workers[1].wage, J1B_TRAINED_WAGE),
        (
            "kappa",
            eq.workers[1].clearing_real_wage,
            J1B_TRAINED_CLEARING,
        ),
    ] {
        close(name, got, want);
    }
}

#[test]
fn the_edge_in_a_ties_share() {
    // J2 (§12 item 16): F's economy at η 0.5 with N_T the double midway between the trained's
    // reserved demand at the line's switch under the loom and under the engine: the loom's side
    // is short (+∞), the engine's clears. The tie's σ moves Y and D_T with it; with N_E 30 the
    // pool is already in excess supply where D_T reaches N_T, so the equilibrium is that edge in
    // σ, the trained walled at κ. With N_E 16 f(σ) falls through 0 after the edge: a tie with
    // the trained at its wall at its own ζ.
    let probe = economy_1d(full(16.0, 1.5, 0.5));
    let x = probe.switch_points().unwrap()[0];
    let s = probe.machines().envelope().switches[0];
    let (d_below, d_above) = (
        probe.at_with(x, s.below).reserved_demand[1],
        probe.at_with(x, s.above).reserved_demand[1],
    );
    assert!(d_above < J2_N_T && J2_N_T < d_below);
    let (e, eq) = checked_1d(full(30.0, J2_N_T, 0.5));
    check_count_1d(&e, &e.solve());
    let q = e.at_with(x, s.below);
    assert_eq!(q.excess_demand(), f64::INFINITY);
    assert!(e.at_with(x, s.above).excess_demand() < 0.0);
    let tie = eq.tie.expect("a tie");
    assert_eq!((eq.x_star, eq.technique, tie.above), (x, s.below, s.above));
    let trained = &eq.workers[1];
    assert!(trained.edge && !trained.pooled);
    assert!(trained.clearing_real_wage > num::expm1(0.8));
    // σ is the edge's finite side: the mix one double below is short, and at σ with the
    // trained's own ζ the pool is in excess supply.
    assert_eq!(
        e.excess_at_share(&q, s.above, tie.share.next_down()),
        f64::INFINITY
    );
    assert!(e.excess_at_share(&q, s.above, tie.share) < 0.0);
    for (name, got, want) in [
        ("sigma", tie.share, J2_SHARE),
        ("v", eq.v, J2_V),
        ("P_s", eq.p_s, J2_P_S),
        ("Y", eq.y, J2_Y),
        ("N_a", eq.n_a, J2_N_A),
        ("trained wage", trained.wage, J2_TRAINED_WAGE),
        ("kappa", trained.clearing_real_wage, J2_TRAINED_CLEARING),
    ] {
        close(name, got, want);
    }
    let (e, eq) = checked_1d(full(16.0, J2_N_T, 0.5));
    assert_eq!(e.at_with(x, s.below).excess_demand(), f64::INFINITY);
    let tie = eq.tie.expect("a tie");
    assert!(tie.share > 0.0 && tie.share < 1.0);
    assert!(!eq.workers[1].pooled && !eq.workers[1].edge);
    assert!(eq.workers[1].reserved_hours < J2_N_T);
}

#[test]
fn rules_at_exact_equality() {
    // A reserved market with demand equal to its workers is not short (§4.3: short when
    // D_i > N_i). E3's economy with N_T the double D_T(1) = Y(1)·R_ŷT, which does not depend on
    // N_T: at x = 1 the trained's demand is its workers exactly, and its ζ is expm1(χ_max).
    let d_1 = economy_1d(entrant_trained(8.0, 2.0, 0.3, 1.0))
        .at_with(1.0, 0)
        .reserved_demand[1];
    let e = economy_1d(entrant_trained(8.0, d_1, 0.3, 1.0));
    let one = e.at_with(1.0, 0);
    assert_eq!(one.reserved_demand[1], d_1);
    assert_eq!(one.short, None);
    assert_eq!(one.clearing[1], num::expm1(0.8));
    assert!(one.excess_demand().is_finite());
    let (e, eq) = checked_1d(entrant_trained(8.0, d_1, 0.3, 1.0));
    check_count_1d(&e, &e.solve());
    assert_eq!(eq.x_star, 1.0);
    assert!(!eq.workers[1].pooled);
    // The walk walls a type only when c_i·P > e_i: at equality it stays pooled (§4.3), which
    // the walk's unit test `walk_keeps_a_type_at_its_threshold_pooled` pins.
    // The all-human corner reports the cheapest task type at its wage, ties to the lower index
    // (§5.3): W4 with Appendix B's machine twice, each using only itself.
    let twice = |mut params: WorkerParams| {
        let m = appendix_b_machine();
        let mut first = m.clone();
        first.build.machines = vec![m.build.machines[0], 0.0];
        first.operating.machines = vec![0.0, 0.0];
        let mut second = first.clone();
        second.build.machines = vec![0.0, m.build.machines[0]];
        params.machine_types = vec![first, second];
        params
    };
    let (e, eq) = checked_1d(twice(goodspace(20.0, 0.05, 0.05)));
    assert_eq!(eq.margin, Margin::AllHuman);
    assert_eq!(eq.types[0].price.to_bits(), eq.types[1].price.to_bits());
    assert_eq!(eq.technique, 0);
    check_path_1d(&e);
    // A root at a wall switch's value below, exactly 0: the piece's upper end, v = v_s, without
    // a bisection step (§5.3 step 3). X's economy with N among the doubles next to n_D/c there.
    let at_switch = |e: &WorkerEconomy| {
        let s = e.wall_switches()[0];
        e.at_wage(1.0, s.wage, s.below)
    };
    let params = exact_zero("the wall switch", wall_switch_economy, at_switch);
    let e = economy_1d(params.clone());
    assert_eq!(at_switch(&e).excess_demand(), 0.0);
    let s = e.wall_switches()[0];
    let eq = solved_1d(params);
    assert_eq!(
        (eq.margin, eq.technique, eq.tie),
        (Margin::Wall, s.below, None)
    );
    assert_eq!(eq.v.to_bits(), s.wage.to_bits());
    assert_eq!(eq.bisection_steps, 0);
    assert!(eq.f_line_1.unwrap() > 0.0);
    check_identities_1d(&e, &eq);
}
