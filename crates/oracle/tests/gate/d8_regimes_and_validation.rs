//! d8: regimes, validation and reductions (docs/unit-1d.md §8): exact zeros at the junctions,
//! viability unchanged by the worker types, every validation rule, the worker types permuted,
//! efficiency measured in other units, and the walk's ceiling.

use oracle::{
    MachineEconomy, Margin, ParamError, Regime, Shortage, SolveError, WorkerEconomy, WorkerParams,
    BRACKET_LO,
};

use crate::g8_regimes::saturated;
use crate::support::*;
use crate::support_1b::category;
use crate::support_1c::*;
use crate::support_1d::*;

/// G1 (χ_max 1) with N chosen so that f on the line at x is exactly 0 in f64: supply there is
/// N·c with c = F(ln(1 + v/P_s)) independent of N, so among the doubles next to n_D(x)/c one
/// has N·c = n_D(x) exactly.
fn exact_zero_at(x: f64) -> WorkerParams {
    let probe = economy_1d(goodspace(1.0, 0.05, 1.0));
    let q = probe.at_with(x, 0);
    let c = q.n_s;
    let mut n = q.n_d / c;
    for _ in 0..64 {
        if n * c == q.n_d {
            return goodspace(n, 0.05, 1.0);
        }
        n = if n * c < q.n_d {
            n.next_up()
        } else {
            n.next_down()
        };
    }
    panic!("no N gives f({x}) = 0 exactly");
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
    // The knife edge (§12): with supply saturated at N = n_D(1), f is 0 on the whole wall and
    // at its end, which counts on the positive side (no equilibrium at v = ∞): no change of
    // side, LaborShort, although every wage on the wall clears labour.
    let n_1 = economy_1d(from_1a(saturated(1.0))).at_with(1.0, 0).n_d;
    let e = economy_1d(from_1a(saturated(n_1)));
    assert_eq!(e.at_with(1.0, 0).excess_demand(), 0.0);
    assert_eq!(e.wall_end().excess, 0.0);
    assert!(matches!(e.solve(), Err(SolveError::LaborShort { excess, .. }) if excess == 0.0));
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
fn a_jump_to_a_shortage_is_labor_short() {
    // E's economy with abundant entrants (N_E 40, χ_max 0.05) and N_T 1: low on the line Y is
    // large and the trained's reserved demand D_T = 0.12·Y exceeds N_T (short, +∞); where it
    // falls below N_T the pool is in excess supply. The one change of side is a jump, not a
    // root: no point clears both markets, and the solve says LaborShort, naming the trained,
    // rather than bisecting onto the jump (docs/unit-1d.md §12).
    let e = economy_1d(entrant_trained(40.0, 1.0, 1.0, 0.05));
    let (values, changes) = sequence_1d(&e);
    assert_eq!(changes, 1);
    assert_eq!(values[0], f64::INFINITY);
    match e.solve() {
        Err(SolveError::LaborShort { excess, reserved }) => {
            assert_eq!((excess, reserved), (f64::INFINITY, Some(1)));
        }
        other => panic!("{other:?}"),
    }
    // The jump: adjacent doubles, short below and far in excess supply above.
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    while lo.next_up() < hi {
        let mid = 0.5 * (lo + hi);
        if e.at_with(mid, 0).short.is_some() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    assert_eq!(e.at_with(lo, 0).short, Some(Shortage::Reserved(1)));
    assert!(e.at_with(hi, 0).excess_demand() < -10.0);
    check_path_1d(&e);
    // The same jump inside [0, 1e-12], where the solve bisects on bit patterns: N_T between
    // the trained's reserved demand at x = 0 and at lo (Y falls by about 1e-13 relative
    // across the stretch, and D_T with it), so f(0) is +∞ and f(lo) far below 0.
    let d_0 = e.at_with(0.0, 0).reserved_demand[1];
    let d_lo = e.at_with(BRACKET_LO, 0).reserved_demand[1];
    assert!(d_lo < d_0);
    let n_t = 0.5 * (d_0 + d_lo);
    assert!(d_lo < n_t && n_t < d_0);
    let e = economy_1d(entrant_trained(40.0, n_t, 1.0, 0.05));
    let (values, changes) = sequence_1d(&e);
    assert_eq!((values[0], changes), (f64::INFINITY, 1));
    assert!(values[1] < -10.0);
    match e.solve() {
        Err(SolveError::LaborShort { excess, reserved }) => {
            assert_eq!((excess, reserved), (f64::INFINITY, Some(1)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_tie_across_a_shortage() {
    // F's economy at η 0.5 with N_T between the trained's reserved demand at the line's switch
    // under the loom and under the engine: the loom's side is short (+∞), the engine's clears.
    // The tie's σ moves Y and D_T with it, so f(σ) is +∞ up to some σ and finite after. With
    // N_E 30 it is already below 0 there: a jump, LaborShort. With N_E 16 it falls through 0
    // after the jump: a tie with the trained at its wall, every identity holding.
    let probe = economy_1d(full(16.0, 1.5, 0.5));
    let x = probe.switch_points().unwrap()[0];
    let s = probe.machines().envelope().switches[0];
    let (d_below, d_above) = (
        probe.at_with(x, s.below).reserved_demand[1],
        probe.at_with(x, s.above).reserved_demand[1],
    );
    assert!(d_above < d_below);
    let n_t = 0.5 * (d_below + d_above);
    let e = economy_1d(full(30.0, n_t, 0.5));
    assert_eq!(e.at_with(x, s.below).excess_demand(), f64::INFINITY);
    assert!(e.at_with(x, s.above).excess_demand() < 0.0);
    match e.solve() {
        Err(SolveError::LaborShort { excess, reserved }) => {
            assert_eq!((excess, reserved), (f64::INFINITY, Some(1)));
        }
        other => panic!("{other:?}"),
    }
    let (e, eq) = checked_1d(full(16.0, n_t, 0.5));
    assert_eq!(e.at_with(x, s.below).excess_demand(), f64::INFINITY);
    let tie = eq.tie.expect("a tie");
    assert!(tie.share > 0.0 && tie.share < 1.0);
    assert!(!eq.workers[1].pooled);
    assert!(eq.workers[1].reserved_hours < n_t);
}
