//! m7: regimes, validation and reductions (docs/unit-1c.md §3.2, §5.2-5.3 and §8): M4's regime
//! rows, every validation rule, an unused type, a duplicate task type, permutation and unit
//! rescaling of the types, and the envelope's edge cases.

use oracle::{
    Eq1c, MachineBlock, MachineEconomy, MachineParams, MachineType, ParamError, Recipe, Regime,
    SCALE_CEIL, SCALE_FLOOR,
};

use crate::goldens_1c::*;
use crate::support::*;
use crate::support_1b::category;
use crate::support_1c::*;

#[test]
fn regimes() {
    // M4's regime rows (docs/unit-1c.md §3.3 and §7).
    let regime = |params: MachineParams| economy_1c(params).solve().expect("finite");
    match regime(m4(0.25)) {
        Regime::BoundaryNoMargin { f_at_1 } => {
            close("f(1) at eta 0.25", f_at_1, M4_REG_ETA025_F_AT_1)
        }
        other => panic!("expected BoundaryNoMargin, got {other:?}"),
    }
    // η = 50: no task type is viable at γ(1) = 50. The envelope's last technique is the engine
    // (the loom is not viable even at the bottom of the line), and d(1) is its least pivot.
    let e = economy_1c(m4(50.0));
    assert_eq!((e.envelope().first, e.envelope().switches.len()), (1, 0));
    match e.solve().unwrap() {
        Regime::NotViable { d_at_1 } => {
            close("d(1) at eta 50", d_at_1, M4_REG_ETA50_D_AT_1);
            assert_eq!(d_at_1.to_bits(), e.at_with(1.0, 1).d.to_bits());
        }
        other => panic!("expected NotViable, got {other:?}"),
    }
    match regime(MachineParams {
        workers: 200.0,
        work_cost: oracle::UniformWorkCost { chi_max: 0.01 },
        ..m4(1.0)
    }) {
        Regime::NoInteriorAtZero { f_at_0 } => {
            close("f(1e-12) at N 200", f_at_0, M4_REG_N200_F_AT_LO)
        }
        other => panic!("expected NoInteriorAtZero, got {other:?}"),
    }
    // A price-side block that is not a nonsingular M-matrix (u·a ≥ 1) has no totals and prices
    // no technique: NotViable, as 1a's u·a ≥ 1 is.
    let mut heavy = m3(0.05);
    heavy.machine_types[0].operating.machines[0] = 0.0;
    heavy.machine_types[0].build.machines[0] = 0.9;
    heavy.rho = 3.0;
    let e = economy_1c(heavy);
    assert!(e.block().totals().lambda_tilde.is_none());
    assert!(matches!(e.solve(), Ok(Regime::NotViable { d_at_1 }) if d_at_1 <= 0.0));
}

#[test]
fn exact_zeros_at_the_ends_of_a_region() {
    // docs/unit-1c.md §5.3 step 3 with a switch, at exact f64 zeros. M4 with χ_max 0.01, so
    // that F = 1 wherever the real wage is above 1%: n_S = N·1.0 = N exactly, and N = n_D at a
    // point makes f exactly 0.0 there. The switch lowers labour demand at both ρ.
    for rho in [0.0, 0.04] {
        let base = MachineParams {
            rho,
            work_cost: oracle::UniformWorkCost { chi_max: 0.01 },
            ..m4(1.0)
        };
        let e = economy_1c(base.clone());
        let x1 = e.switch_points().unwrap()[0];
        let s = e.envelope().switches[0];
        assert_eq!((s.below, s.above), (0, 1));
        let (lo, loom, engine, one) = (
            e.at_with(oracle::BRACKET_LO, 0),
            e.at_with(x1, 0),
            e.at_with(x1, 1),
            e.at_with(1.0, 1),
        );
        for q in [&lo, &loom, &engine, &one] {
            assert_eq!(q.n_s, base.workers, "F = 1 at x = {}", q.x);
        }
        assert!(lo.n_d > loom.n_d && loom.n_d > engine.n_d && engine.n_d > one.n_d);
        let with = |workers: f64| {
            economy_1c(MachineParams {
                workers,
                ..base.clone()
            })
        };
        // f_0(x_1) = 0: the loom's root is the switch point itself (1a's Root::exact), not a
        // tie: a value is on the positive side when > 0.
        let e = with(loom.n_d);
        match e.solve() {
            Ok(Regime::Interior(eq)) => {
                assert_eq!(
                    (eq.x_star, eq.technique, eq.tie),
                    (x1, 0, None),
                    "rho {rho}"
                );
                assert_eq!(eq.bisection_steps, 0);
                assert_eq!(eq.one_minus_x_star, 1.0 - x1);
                check_identities_1c(&e, &eq);
            }
            other => panic!("rho {rho}: expected a root at x_1, got {other:?}"),
        }
        // f_1(x_1) = 0: the change of side is across the switch, a tie, and the engine clears
        // labour alone there: σ = 1.
        let e = with(engine.n_d);
        match e.solve() {
            Ok(Regime::Interior(eq)) => {
                assert_eq!((eq.x_star, eq.technique), (x1, 0), "rho {rho}");
                let tie = eq.tie.expect("a tie");
                assert_eq!((tie.above, tie.share, tie.gamma), (1, 1.0, s.gamma));
                check_identities_1c(&e, &eq);
            }
            other => panic!("rho {rho}: expected a tie at x_1, got {other:?}"),
        }
        // f(1) = 0 with a switch below: BoundaryNoMargin, as 1a's f(1) = 0 is; every other
        // value of the sequence is positive.
        let e = with(one.n_d);
        assert_eq!(
            e.solve(),
            Ok(Regime::BoundaryNoMargin { f_at_1: 0.0 }),
            "rho {rho}"
        );
        assert_eq!(check_regions(&e), 1);
        // f(lo) = 0 with a switch above: NoInteriorAtZero; every other value is negative.
        let e = with(lo.n_d);
        assert_eq!(
            e.solve(),
            Ok(Regime::NoInteriorAtZero { f_at_0: 0.0 }),
            "rho {rho}"
        );
        assert_eq!(check_regions(&e), 1);
        // A boundary economy with a switch and no change of side inside: BoundaryNoMargin.
        let e = with(0.5 * one.n_d);
        assert!(matches!(e.solve(), Ok(Regime::BoundaryNoMargin { f_at_1 }) if f_at_1 > 0.0));
    }
}

/// Asserts that `params` is rejected with an error whose innermost name is `name`, inside the
/// item `item` (kind and index) when given.
fn rejected(what: &str, params: MachineParams, item: Option<(&str, usize)>, name: &str) {
    let err = MachineEconomy::new(params).expect_err(what);
    assert_eq!(err.name(), name, "{what}: {err}");
    match (item, &err) {
        (
            Some((kind, index)),
            ParamError::Item {
                kind: k, index: i, ..
            },
        ) => {
            assert_eq!((*k, *i), (kind, index), "{what}: {err}")
        }
        (None, ParamError::Item { .. }) => panic!("{what}: {err:?} is an item's"),
        (Some(_), other) => panic!("{what}: {other:?} is not an item's"),
        (None, _) => {}
    }
    assert!(!err.to_string().is_empty());
}

#[test]
fn validation() {
    // Every rule of docs/unit-1c.md §3.2, as docs/unit-1g.md §2.1 amends it, is an error.
    let base = m4(1.0);
    let with_type = |k: usize, f: &dyn Fn(&mut MachineType)| {
        let mut p = base.clone();
        f(&mut p.machine_types[k]);
        p
    };
    let ty = Some(("machine type", 1));
    rejected(
        "theta",
        with_type(1, &|t| t.task_efficiency = -1.0),
        ty,
        "task_efficiency",
    );
    rejected(
        "theta below the floor",
        with_type(1, &|t| t.task_efficiency = SCALE_FLOOR / 2.0),
        ty,
        "task_efficiency",
    );
    rejected(
        "machines length",
        with_type(1, &|t| t.operating.machines.pop().map(|_| ()).unwrap()),
        ty,
        "operating.machines",
    );
    rejected(
        "machines negative",
        with_type(1, &|t| t.build.machines[0] = -0.1),
        ty,
        "build.machines",
    );
    rejected(
        "machines NaN",
        with_type(1, &|t| t.operating.machines[2] = f64::NAN),
        ty,
        "operating.machines",
    );
    rejected(
        "machines above the ceiling",
        with_type(1, &|t| t.build.machines[1] = 2.0 * SCALE_CEIL),
        ty,
        "build.machines",
    );
    rejected(
        "labour",
        with_type(1, &|t| t.operating.labor = -1e-3),
        ty,
        "operating.labor",
    );
    rejected(
        "labour ceiling",
        with_type(1, &|t| t.build.labor = 1e31),
        ty,
        "build.labor",
    );
    rejected(
        "land below the floor",
        with_type(1, &|t| t.build.land = 1e-31),
        ty,
        "build.land",
    );
    rejected(
        "land",
        with_type(1, &|t| t.operating.land = f64::INFINITY),
        ty,
        "operating.land",
    );
    rejected("delta", with_type(1, &|t| t.delta = 0.0), ty, "delta");
    rejected(
        "delta above 1",
        with_type(1, &|t| t.delta = 1.5),
        ty,
        "delta",
    );
    rejected(
        "build lag",
        with_type(1, &|t| t.build_lag = 0),
        ty,
        "build_lag",
    );
    let mut huge = base.clone();
    huge.rho = 1.0;
    huge.machine_types[1].build_lag = 2000;
    rejected("u overflows", huge, ty, "u");
    // At least one type, and one that does tasks.
    rejected(
        "no types",
        MachineParams {
            machine_types: vec![],
            ..base.clone()
        },
        None,
        "machine types",
    );
    let mut no_tasks = base.clone();
    for t in &mut no_tasks.machine_types {
        t.task_efficiency = 0.0;
    }
    rejected("no task type", no_tasks, None, "machine types");
    // Productive recipes, per period (docs/unit-1g.md §2.1, D-G10, amending unit-1c.md §3.2):
    // I − A^q with A^q = A^op + Δ·A^I. The loom (δ 0.1) built from 12 of its own service uses 1.2
    // of it a period, and is not productive. Built from one of itself, which 1c refused, it uses
    // 0.1 a period and is valid.
    rejected(
        "not productive",
        with_type(0, &|t| t.build.machines[0] = 12.0),
        None,
        "machine types",
    );
    assert!(MachineEconomy::new(with_type(0, &|t| t.build.machines[0] = 1.0)).is_ok());
    // The rule is on the sum A^op + Δ·A^I: 0.6 of the loom's own service to operate and 4.1 to
    // build is 0.6 + 0.1·4.1 = 1.01 a period, though each recipe alone is productive per period.
    // 1c's rule on A^op + A^I refused 0.6 and 0.6 (1.2); per period that is 0.66, and valid.
    rejected(
        "not productive as a sum",
        with_type(0, &|t| {
            t.operating.machines[0] = 0.6;
            t.build.machines[0] = 4.1;
        }),
        None,
        "machine types",
    );
    assert!(MachineEconomy::new(with_type(0, &|t| {
        t.operating.machines[0] = 0.6;
        t.build.machines[0] = 0.6;
    }))
    .is_ok());
    // The chain to land: a type with no land that uses only another landless type.
    let landless = |uses: usize| {
        let mut row = vec![0.0; 3];
        row[uses] = 0.1;
        machine_type(
            1.0,
            recipe(&row, 0.1, 0.0),
            recipe(&[0.0; 3], 0.2, 0.0),
            0.1,
            1,
        )
    };
    let mut p = base.clone();
    p.machine_types[0] = landless(0);
    rejected("no land", p, Some(("machine type", 0)), "land");
    // Land through another type's services is enough.
    let mut p = base.clone();
    p.machine_types[0] = landless(2);
    assert!(MachineEconomy::new(p).is_ok());
    // Intermediate inputs: shape, range and productivity.
    let with_inputs = |a: Vec<Vec<f64>>| MachineParams {
        intermediate: a,
        ..base.clone()
    };
    rejected(
        "rows",
        with_inputs(vec![vec![0.0; 4]; 3]),
        None,
        "intermediate",
    );
    rejected(
        "columns",
        with_inputs(vec![vec![0.0; 3]; 4]),
        None,
        "intermediate",
    );
    let mut a = m4_intermediate();
    a[2][1] = -0.1;
    rejected("negative input", with_inputs(a), None, "intermediate");
    let mut a = m4_intermediate();
    a[0][3] = f64::NAN;
    rejected("NaN input", with_inputs(a), None, "intermediate");
    let mut a = m4_intermediate();
    a[0][1] = 3.0;
    a[1][0] = 0.5;
    rejected(
        "inputs not productive",
        with_inputs(a),
        None,
        "intermediate",
    );
    // A category priced only through its inputs is valid; one with nothing is not.
    let mut p = base.clone();
    p.categories[0] = category(0.3, 0.0, &[0.0, 0.0, 0.0]);
    p.intermediate[0][1] = 0.5;
    assert!(MachineEconomy::new(p.clone()).is_ok());
    p.intermediate[0][1] = 0.0;
    rejected("unpriced category", p, Some(("category", 0)), "category");
    // The basket's chain land and chain hours.
    let mut p = base.clone();
    for c in &mut p.categories {
        c.direct_land = 0.0;
    }
    rejected("basket land", p, None, "basket");
    let mut p = appendix_b_household(linear(1.0, 0.2, 0.8), 0.0, 1.0, vec![dynamics_machine()]);
    p.categories[0].weight = 0.0;
    rejected("basket hours", p, None, "basket");
    // −0.0 is stored as +0.0.
    let mut p = base.clone();
    p.rho = -0.0;
    p.machine_types[0].operating.labor = -0.0;
    p.machine_types[2].task_efficiency = -0.0;
    p.machine_types[1].build.machines[0] = -0.0;
    p.intermediate[0][0] = -0.0;
    let e = MachineEconomy::new(p).unwrap();
    let q = e.params();
    for value in [
        q.rho,
        q.machine_types[0].operating.labor,
        q.machine_types[2].task_efficiency,
        q.machine_types[1].build.machines[0],
        q.intermediate[0][0],
    ] {
        assert_eq!(value.to_bits(), 0.0f64.to_bits());
    }
    assert_eq!(e.block().types(), q.machine_types.as_slice());
}

/// Every economy-level output of an equilibrium, by key, as bits, without the residuals that
/// are maxima over types (docs/unit-1c.md §8).
fn aggregates(eq: &Eq1c) -> Vec<(String, Option<u64>)> {
    bits_1c(eq)
        .into_iter()
        .filter(|(key, _)| !key.contains('.'))
        .filter(|(key, _)| {
            key != "res_user_cost" && key != "res_leontief_price" && key != "res_services"
        })
        .collect()
}

/// A type appended to M4's (with a zero column for it in every recipe).
fn append(mut params: MachineParams, t: MachineType) -> MachineParams {
    for other in &mut params.machine_types {
        other.operating.machines.push(0.0);
        other.build.machines.push(0.0);
    }
    params.machine_types.push(t);
    params
}

#[test]
fn unused_type_changes_nothing() {
    // A θ = 0 type that nobody uses, added last, leaves every other output bit for bit, when
    // the addition keeps I − Â a nonsingular M-matrix; one whose price recursion diverges makes
    // the economy NotViable (an_unused_type_that_cannot_be_priced_is_not_viable).
    for (what, params) in [
        ("M4", m4(1.0)),
        (
            "M4t",
            MachineParams {
                workers: 8.0,
                ..m4(0.5)
            },
        ),
        ("M3", m3(0.05)),
    ] {
        let k = params.machine_types.len();
        let spare = machine_type(
            0.0,
            recipe(&vec![0.0; k + 1], 0.3, 0.7),
            recipe(&vec![0.0; k + 1], 1.0, 0.2),
            0.2,
            2,
        );
        let (a, b) = (
            interior_1c(params.clone()),
            interior_1c(append(params, spare)),
        );
        assert_eq!(aggregates(&a), aggregates(&b), "{what}");
        let (bits_a, bits_b) = (bits_1c(&a), bits_1c(&b));
        for (key, value) in &bits_a {
            if key.starts_with("type") || key.starts_with("cat") || key.starts_with("switch") {
                assert_eq!(value, &bits_b[key], "{what}: {key}");
            }
        }
        assert_eq!(b.types[k].services, 0.0);
    }
}

#[test]
fn phi_only_when_every_type_is_flow() {
    // φ is the flow price system's (1a's u = 1): with every u_k = 1 it is reported, and one
    // unused type with u ≠ 1 removes it, economy-wide and per category; that type's own φ is
    // absent and the flow type's stays. Everything else is unchanged.
    let flow = machine_type(1.0, recipe(&[0.3], 0.05, 0.4), Recipe::zero(1), 1.0, 1);
    let params = appendix_b_household(linear(1.0, 0.2, 0.8), 0.0, 1.0, vec![flow]);
    let spare = machine_type(
        0.0,
        recipe(&[0.0, 0.0], 0.1, 0.3),
        recipe(&[0.0, 0.0], 0.2, 0.2),
        0.2,
        2,
    );
    let with_spare = append(params.clone(), spare);
    let (a, b) = (interior_1c(params.clone()), interior_1c(with_spare.clone()));
    assert!(a.phi_w.is_some() && a.categories.iter().all(|c| c.phi_w.is_some()));
    assert_eq!(b.types[1].user_cost, 0.2);
    assert!(b.phi_w.is_none() && b.phi_r.is_none());
    assert!(b
        .categories
        .iter()
        .all(|c| c.phi_w.is_none() && c.phi_r.is_none()));
    assert_eq!(b.types[0].phi_w, a.types[0].phi_w);
    assert_eq!(b.types[1].phi_w, None);
    for (x, y) in [(a.x_star, b.x_star), (a.v, b.v), (a.y, b.y), (a.n_a, b.n_a)] {
        assert_eq!(x.to_bits(), y.to_bits());
    }
    check_identities_1c(&economy_1c(params), &a);
    let e = economy_1c(with_spare);
    check_identities_1c(&e, &b);
    check_fork_and_bounds_1c(&e, &b);
}

#[test]
fn a_category_bought_only_as_an_input_makes_a_margin() {
    // margin_active reads the basket's gross outputs ŷ, not its weights z (docs/unit-1c.md
    // §4.8): a category outside the basket that a bought one uses has tasks that are produced.
    // Here only the unbought B has tasks on the upper segment, where the root lies.
    let mut params = appendix_b_household(
        linear(1.0, 0.2, 0.8),
        0.0,
        1.0,
        vec![machine_type(
            1.0,
            Recipe::zero(1),
            recipe(&[0.3], 0.05, 0.4),
            1.0,
            1,
        )],
    );
    params.edges = vec![0.0, 0.5, 1.0];
    params.categories = vec![
        category(1.0, 0.5, &[1.0, 0.0]),
        category(0.0, 0.2, &[0.0, 1.0]),
    ];
    params.intermediate = vec![vec![0.0, 0.5], vec![0.0, 0.0]];
    let e = economy_1c(params.clone());
    assert_eq!(e.basket_outputs(), [1.0, 0.5]);
    let eq = interior_1c(params);
    assert!(eq.x_star > 0.5, "{}", eq.x_star);
    assert!(eq.margin_active);
    assert!(eq.categories[1].gross_output > 0.0 && eq.categories[1].output == 0.0);
    check_identities_1c(&e, &eq);
    check_fork_and_bounds_1c(&e, &eq);
}

#[test]
fn duplicate_task_type() {
    // A copy of the technique's type at a higher index ties with it everywhere and is never
    // chosen (the lower index, §5.2); every aggregate is bit-equal. The copied types have no
    // machine inputs, so the copy's totals are the original's exactly.
    for (what, rho, technique) in [("M5 durable", 0.05, 1), ("M5 flow", 0.15, 0)] {
        let params = m5(rho, 4.0, 1.0);
        let copy = {
            let mut t = params.machine_types[technique].clone();
            t.operating.machines.push(0.0);
            t.build.machines.push(0.0);
            t
        };
        let doubled = append(params.clone(), copy);
        let e = economy_1c(doubled.clone());
        let totals = e.block().totals();
        let (lt, bt) = (totals.lambda_tilde.unwrap(), totals.b_tilde.unwrap());
        assert_eq!((lt[2], bt[2]), (lt[technique], bt[technique]), "{what}");
        assert_ne!(e.envelope().first, 2);
        assert!(e.envelope().switches.iter().all(|s| s.above != 2));
        let (a, b) = (interior_1c(params), interior_1c(doubled));
        assert_eq!(b.technique, technique);
        assert_eq!(b.types[2].services, 0.0);
        assert_eq!(
            b.types[2].price.to_bits(),
            b.types[technique].price.to_bits()
        );
        assert_eq!(aggregates(&a), aggregates(&b), "{what}");
    }
}

/// The types in reverse order, with every recipe's machine columns reversed.
fn reversed(mut params: MachineParams) -> MachineParams {
    params.machine_types.reverse();
    for t in &mut params.machine_types {
        t.operating.machines.reverse();
        t.build.machines.reverse();
    }
    params
}

#[test]
fn permutation() {
    // Reversing the types gives the same equilibrium within 1e-12, permuted.
    for params in [
        m4(1.0),
        m4(2.0),
        MachineParams {
            workers: 8.0,
            ..m4(0.5)
        },
    ] {
        let a = interior_1c(params.clone());
        let b = interior_1c(reversed(params.clone()));
        let k = a.types.len();
        assert_eq!(b.technique, k - 1 - a.technique);
        for (name, x, y) in [
            ("x*", a.x_star, b.x_star),
            ("v", a.v, b.v),
            ("P_s", a.p_s, b.p_s),
            ("Y", a.y, b.y),
            ("N_a", a.n_a, b.n_a),
            ("I", a.income, b.income),
            ("interest", a.interest, b.interest),
        ] {
            close(name, y, x);
        }
        for i in 0..k {
            let (s, t) = (&a.types[i], &b.types[k - 1 - i]);
            close("p_k", t.price, s.price);
            close("V_k", t.build_cost, s.build_cost);
            if s.services == 0.0 {
                assert!(t.services.abs() <= FULL * a.y);
            } else {
                close("X_k", t.services, s.services);
            }
        }
        if let (Some(s), Some(t)) = (a.tie, b.tie) {
            // The switch is still from the loom to the engine: the same share.
            close("sigma", t.share, s.share);
        }
        check_identities_1c(&economy_1c(reversed(params.clone())), &b);
    }
}

/// M4 with the engine's service measured in units of c old units: θ, its recipe rows and the
/// columns that use it rescaled (docs/unit-1c.md §8).
fn engine_in_units_of(c: f64) -> MachineParams {
    let mut p = m4(1.0);
    let e = 1;
    for (k, t) in p.machine_types.iter_mut().enumerate() {
        for recipe in [&mut t.operating, &mut t.build] {
            if k == e {
                // Per new unit: c old units' inputs, with the engine's own column in new units.
                for (l, a) in recipe.machines.iter_mut().enumerate() {
                    if l != e {
                        *a *= c;
                    }
                }
                recipe.labor *= c;
                recipe.land *= c;
            } else {
                recipe.machines[e] /= c;
            }
        }
        if k == e {
            t.task_efficiency *= c;
        }
    }
    p
}

#[test]
fn unit_rescaling() {
    let base = interior_1c(m4(1.0));
    // c = 4: every scaling is by a power of two, and the solve is bit-equal, with p_k = 4·p_k.
    let four = interior_1c(engine_in_units_of(4.0));
    assert_eq!(aggregates(&base), aggregates(&four));
    let (a, b) = (&base.types[1], &four.types[1]);
    for (name, x, y) in [
        ("p", 4.0 * a.price, b.price),
        ("O", 4.0 * a.operating_cost, b.operating_cost),
        ("V", 4.0 * a.build_cost, b.build_cost),
        ("X", a.services / 4.0, b.services),
        ("lt", 4.0 * a.lambda_tilde, b.lambda_tilde),
        ("bt", 4.0 * a.b_tilde, b.b_tilde),
    ] {
        assert_eq!(x.to_bits(), y.to_bits(), "{name}");
    }
    assert_eq!(
        a.delivered_cost.unwrap().to_bits(),
        (b.price / 8.0).to_bits()
    );
    // With every u = 1 (flow types at ρ = 0), φ is reported, and as a share it does not move
    // when the technique's unit does: θ = 4 gives φ_w = γλ̃/θ bit-equal to θ = 1's.
    let flow_economy = |c: f64| {
        let t = machine_type(
            c,
            recipe(&[0.3], c * 0.05, c * 0.4),
            Recipe::zero(1),
            1.0,
            1,
        );
        appendix_b_household(linear(1.0, 0.2, 0.8), 0.0, 1.0, vec![t])
    };
    let (one, four) = (
        interior_1c(flow_economy(1.0)),
        interior_1c(flow_economy(4.0)),
    );
    assert!(one.phi_w.is_some());
    assert_eq!(one.phi_w.map(f64::to_bits), four.phi_w.map(f64::to_bits));
    assert_eq!(
        four.types[0].price.to_bits(),
        (4.0 * one.types[0].price).to_bits()
    );
    check_identities_1c(&economy_1c(flow_economy(4.0)), &four);
    // c = 3: within 1e-12.
    let three = interior_1c(engine_in_units_of(3.0));
    for (name, x, y) in [
        ("x*", base.x_star, three.x_star),
        ("v", base.v, three.v),
        ("Y", base.y, three.y),
        ("N_a", base.n_a, three.n_a),
        ("interest", base.interest, three.interest),
        ("p", 3.0 * a.price, three.types[1].price),
        ("X", a.services / 3.0, three.types[1].services),
    ] {
        close(name, y, x);
    }
}

/// A flow task type over K types, θ 1: its totals are λ and b exactly.
fn flow(k: usize, labor: f64, land: f64) -> MachineType {
    machine_type(
        1.0,
        recipe(&vec![0.0; k], labor, land),
        Recipe::zero(k),
        1.0,
        1,
    )
}

#[test]
fn envelope_edge_cases() {
    // A crossing beyond both types' viability does not count: X (λ 0.25, b 0.1, viable below
    // γ = 4) and the steeper L (λ 0.5, b 1, viable below 2) meet at γ = 4.5 only where both
    // closure wages are negative. The envelope stays X, and past X's edge the economy is
    // NotViable on X's own pivot.
    let (x, l) = (flow(2, 0.25, 0.1), flow(2, 0.5, 1.0));
    let block = MachineBlock::new(vec![x.clone(), l.clone()], 0.0).unwrap();
    let totals = block.totals();
    let (lt, bt) = (totals.lambda_tilde.unwrap(), totals.b_tilde.unwrap());
    let crossing = (bt[1] - bt[0]) / (bt[1] * lt[0] - bt[0] * lt[1]);
    assert!(crossing > 4.0 && crossing < 5.0 && bt[1] * lt[0] > bt[0] * lt[1]);
    let env = block.envelope(1.0, 5.0);
    assert_eq!((env.first, env.switches.len()), (0, 0));
    let params = appendix_b_household(linear(1.0, 1.0, 4.0), 0.0, 1.0, vec![x, l]);
    let e = economy_1c(params);
    assert_eq!(e.envelope().switches.len(), 0);
    match e.solve() {
        Ok(Regime::NotViable { d_at_1 }) => {
            assert_eq!(d_at_1.to_bits(), e.at_with(1.0, 0).d.to_bits())
        }
        other => panic!("expected NotViable, got {other:?}"),
    }
    // Two crossings at one γ: X is cheapest below γ = 2, where A and B both meet it exactly
    // (dyadic recipes); B, the cheaper just above, takes over, and A never enters.
    let (x, a, b) = (
        flow(3, 0.375, 0.125),
        flow(3, 0.25, 0.25),
        flow(3, 0.125, 0.375),
    );
    let block = MachineBlock::new(vec![x.clone(), a, b.clone()], 0.0).unwrap();
    for t in 0..3 {
        assert_eq!(block.closure_wage(2.0, t), Some(1.0));
    }
    let env = block.envelope(1.0, 3.0);
    assert_eq!(env.first, 0);
    assert_eq!(env.switches.len(), 1);
    assert_eq!((env.switches[0].gamma, env.switches[0].above), (2.0, 2));
    // A switch exactly at γ(1): γ = 1 + x reaches 2 at x = 1, which counts (γ_i ≤ γ(1)), so the
    // technique at x = 1 is B, and x_i is the double below 1.
    let params = MachineParams {
        workers: 1.0,
        ..appendix_b_household(
            linear(1.0, 1.0, 1.0),
            0.0,
            1.0,
            vec![flow(2, 0.375, 0.125), flow(2, 0.125, 0.375)],
        )
    };
    let e = economy_1c(params.clone());
    assert_eq!(e.envelope().switches.len(), 1);
    assert_eq!(e.envelope().switches[0].gamma, 2.0);
    assert_eq!(e.technique_at(1.0), 1);
    // x_i is the largest double with γ(x) < 2: 1 − 2^-52, since 1 + (1 − 2^-53) rounds to 2.
    let x_i = e.switch_points().unwrap()[0];
    assert_eq!(x_i, 1.0 - 2.0 * f64::EPSILON / 2.0);
    assert!(oracle::Schedule::gamma(&params.schedule, x_i) < 2.0);
    assert!(oracle::Schedule::gamma(&params.schedule, x_i.next_up()) >= 2.0);
    assert_eq!(block.envelope(1.0, 2.0f64.next_down()).switches.len(), 0);
    let result = e.solve();
    assert!(result.is_ok(), "{result:?}");
    if let Ok(Regime::Interior(eq)) = result {
        check_identities_1c(&e, &eq);
    }
    // A type viable only low on the line: X is not viable above γ = 2.5, and Y takes the
    // margin before; on γ = 1 + 2x (γ(1) = 3) the economy solves with Y at x = 1. X alone is
    // NotViable there.
    let (x, y) = (flow(2, 0.4, 0.1), flow(2, 0.05, 0.5));
    let params = appendix_b_household(linear(1.0, 1.0, 2.0), 0.0, 1.0, vec![x, y]);
    let e = economy_1c(params.clone());
    assert_eq!((e.envelope().first, e.envelope().last()), (0, 1));
    assert!(e.envelope().switches[0].gamma < 2.5);
    assert!(e.solve().is_ok());
    let alone = flow(1, 0.4, 0.1);
    let single = appendix_b_household(linear(1.0, 1.0, 2.0), 0.0, 1.0, vec![alone]);
    assert!(matches!(
        economy_1c(single).solve(),
        Ok(Regime::NotViable { .. })
    ));
}

#[test]
fn an_unused_type_that_cannot_be_priced_is_not_viable() {
    // A recorded departure from SSRN A.1 (docs/unit-1c.md §11 question 9, §12 item 15). M3 at
    // ρ 0.05 is interior. Add a type built from 0.3 of its own service with δ 1 and J 30: its u
    // is 1.05^30 = 4.32, so u·a^I = 1.30 and its price recursion diverges, though the
    // physical recipes are productive (ρ(A^op + A^I) = 0.5). Nobody needs it: SSRN A.1 would
    // leave it unused, its unit cost above any price, and M3's equilibrium would stand. 1c
    // requires every type priced, I − Â a nonsingular M-matrix, and returns NotViable, as a
    // power type and as a task type.
    let base = m3(0.05);
    let eq = interior_1c(base.clone());
    let divergent = |theta: f64| {
        machine_type(
            theta,
            Recipe::zero(2),
            recipe(&[0.0, 0.3], 0.1, 0.5),
            1.0,
            30,
        )
    };
    for theta in [0.0, 1.0] {
        let params = append(base.clone(), divergent(theta));
        let e = economy_1c(params);
        let u = e.block().user_costs()[1];
        assert!(u * 0.3 > 1.0, "{u}");
        assert!(e.block().totals().lambda_tilde.is_none());
        // M3's type is viable on its own at x = 1 and at M3's x*.
        let alone = economy_1c(base.clone());
        assert!(alone.at_with(1.0, 0).d > 0.0 && alone.at_with(eq.x_star, 0).d > 0.0);
        match e.solve() {
            Ok(Regime::NotViable { d_at_1 }) => {
                assert!(d_at_1 < 0.0, "theta {theta}: {d_at_1}");
                assert_eq!(d_at_1.to_bits(), e.at_with(1.0, 0).d.to_bits());
            }
            other => panic!("theta {theta}: expected NotViable, got {other:?}"),
        }
    }
}

/// A flow task type over K types, θ 1, with machine inputs to operate.
fn flow_using(machines: &[f64], labor: f64, land: f64) -> MachineType {
    machine_type(
        1.0,
        recipe(machines, labor, land),
        Recipe::zero(machines.len()),
        1.0,
        1,
    )
}

#[test]
fn two_switches_and_a_tie_at_the_second() {
    // Three flow task types at ρ = 0 (λ̃ = λ and b̃ = b exactly) whose closure wages cross in
    // sequence on γ = 1 + 2x: X (λ 0.6, b 0.05) to A (0.3, 0.2) at γ = 0.15/0.105 = 1.43, then A
    // to B (0.05, 0.6) at 0.4/0.17 = 2.35 (X and B cross at 1.54, after X has left). The
    // envelope, the switch points, a tie at the second switch and a root above it
    // (docs/unit-1c.md §4.3, §5.2 step 3 repeated, §5.3 step 2 on [x_1, 1], §4.7 at switch 2).
    let types = vec![flow(3, 0.6, 0.05), flow(3, 0.3, 0.2), flow(3, 0.05, 0.6)];
    let base = appendix_b_household(linear(1.0, 1.0, 2.0), 0.0, 1.0, types);
    let e = economy_1c(base.clone());
    let env = e.envelope().clone();
    assert_eq!(env.first, 0);
    assert_eq!(env.switches.len(), 2, "{env:?}");
    assert_eq!((env.switches[0].below, env.switches[0].above), (0, 1));
    assert_eq!((env.switches[1].below, env.switches[1].above), (1, 2));
    let crossing = |a: (f64, f64), b: (f64, f64)| (b.1 - a.1) / (b.1 * a.0 - a.1 * b.0);
    let (x, a, b) = ((0.6, 0.05), (0.3, 0.2), (0.05, 0.6));
    assert_eq!(env.switches[0].gamma.to_bits(), crossing(x, a).to_bits());
    assert_eq!(env.switches[1].gamma.to_bits(), crossing(a, b).to_bits());
    assert!(crossing(x, b) > env.switches[0].gamma && crossing(x, b) < env.switches[1].gamma);
    // Each switch point is the largest double below its γ_i, and near the closed form's x.
    let points = e.switch_points().unwrap();
    for (i, s) in env.switches.iter().enumerate() {
        let gamma = |x: f64| oracle::Schedule::gamma(&base.schedule, x);
        assert!(gamma(points[i]) < s.gamma && gamma(points[i].next_up()) >= s.gamma);
        near("x_i", points[i], (s.gamma - 1.0) / 2.0, 4.0 * f64::EPSILON);
    }
    for w in points.windows(2) {
        assert!(w[0] < w[1]);
    }
    let x2 = points[1];
    assert_eq!(e.technique_at(x2), 1);
    assert_eq!(e.technique_at(x2.next_up()), 2);
    let (under_a, under_b) = (e.at_with(x2, 1), e.at_with(x2, 2));
    assert!(
        under_b.n_d < under_a.n_d,
        "the switch lowers labour demand at ρ = 0"
    );
    let per_worker = under_a.n_s / base.workers;
    // The tie: n_S at x_2 midway between the two one-sided n_D.
    let tie_params = MachineParams {
        workers: 0.5 * (under_a.n_d + under_b.n_d) / per_worker,
        ..base.clone()
    };
    let et = economy_1c(tie_params.clone());
    let eq = interior_1c(tie_params);
    let tie = eq.tie.expect("a tie at the second switch");
    assert_eq!((eq.technique, tie.above), (1, 2));
    assert_eq!(tie.gamma, env.switches[1].gamma);
    assert!(tie.share > 0.0 && tie.share < 1.0);
    assert_eq!(eq.x_star, x2);
    assert_eq!(eq.switches.len(), 2);
    for (i, w) in eq.switches.iter().enumerate() {
        let s = env.switches[i];
        assert_eq!(
            (w.gamma, w.x, w.below, w.above),
            (s.gamma, points[i], s.below, s.above)
        );
    }
    assert_eq!(eq.types[0].services, 0.0);
    assert!(eq.types[1].task_services > 0.0 && eq.types[2].task_services > 0.0);
    check_identities_1c(&et, &eq);
    check_fork_and_bounds_1c(&et, &eq);
    // A root in region 2, with fewer workers.
    let root_params = MachineParams {
        workers: 0.9 * under_b.n_d / per_worker,
        ..base
    };
    let er = economy_1c(root_params.clone());
    let eq = interior_1c(root_params);
    assert_eq!((eq.technique, eq.tie), (2, None));
    assert!(eq.x_star > x2);
    assert_root_1c(&er, eq.x_star, 2);
    assert_eq!(check_regions(&er), 1);
    check_identities_1c(&er, &eq);
}

#[test]
fn phi_when_the_technique_is_not_type_0() {
    // docs/unit-1c.md §4.8: φ_w = γλ̃_τ/θ_τ, from the numerator and pivot of λ̃_τ's back
    // substitution. Two flow types at ρ = 0 (every u = 1); type 0 uses 0.2 of its own service,
    // so its pivot is 0.8 and type 1's is 1: the technique at x* is type 1.
    let params = MachineParams {
        workers: PHI_TECHNIQUE_1_WORKERS,
        ..appendix_b_household(
            linear(1.0, 1.0, 2.0),
            0.0,
            1.0,
            vec![
                flow_using(&[0.2, 0.0], 0.4, 0.1),
                flow_using(&[0.0, 0.0], 0.05, 0.5),
            ],
        )
    };
    let e = economy_1c(params.clone());
    let eq = interior_1c(params);
    assert_eq!((eq.technique, eq.tie), (1, None));
    let phi = eq.phi_w.expect("every u = 1");
    close("phi_w", phi, eq.gamma_star * eq.types[1].lambda_tilde);
    assert!((phi - eq.gamma_star * eq.types[0].lambda_tilde).abs() > 1e-3);
    check_identities_1c(&e, &eq);
}

/// N for `phi_when_the_technique_is_not_type_0`: its equilibrium lies above the switch.
const PHI_TECHNIQUE_1_WORKERS: f64 = 2.0;

#[test]
fn type_phi_with_interest() {
    // Each type's φ_w = vλ̃_k/p_k, on the price side, where u_k = 1. With ρ = δ = 0.5 and J = 1,
    // u = (ρ + δ)(1 + ρ)^0 = 1 exactly while δ = 0.5: the price and clearing totals differ.
    // M2's recipes on γ = 0.2 + 0.8x.
    let m = machine_type(
        1.0,
        recipe(&[0.5], 0.1, 0.2),
        recipe(&[0.1], 0.2, 0.02),
        0.5,
        1,
    );
    let params = appendix_b_household(linear(1.0, 0.2, 0.8), 0.5, 1.0, vec![m]);
    let e = economy_1c(params.clone());
    assert_eq!(e.block().user_costs()[0], 1.0);
    let eq = interior_1c(params);
    let t = &eq.types[0];
    assert!(t.lambda_tilde > 1.01 * t.lambda_tilde_q);
    close(
        "type phi_w",
        t.phi_w.expect("u = 1"),
        eq.v * t.lambda_tilde / t.price,
    );
    assert!(eq.interest > 0.0);
    check_identities_1c(&e, &eq);
    check_fork_and_bounds_1c(&e, &eq);
}
