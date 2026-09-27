//! m1: units 1a and 1b as the one-type case of unit 1c, exactly (PLAN Phase 1's gate, R1;
//! docs/unit-1c.md §2.6 and §8). "Exactly" is bit equality: the one-type form, with no
//! operating recipe, θ = 1 and no intermediate inputs, repeats 1b's floating-point operations
//! in 1b's order (§5.1), and so 1a's.

use std::collections::BTreeMap;

use oracle::{
    CategoryEconomy, CategoryParams, Economy, Eq1a, Eq1b, Eq1c, MachineEconomy, MachineParams,
    Output, Output1b, ParamError, Params, PowerSchedule, Recipe, Regime, Schedule, SolveError,
    UniformWorkCost, BRACKET_LO, CURVATURE_CEIL, SCALE_CEIL,
};

use crate::g5_random_economies::{draw, Set, SplitMix64, MAX_DRAWS, SEED, WANTED};
use crate::g8_regimes::{saturated, steep, Jump};
use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;
use crate::support_1c::*;

/// Unit 1b's outputs, by printed key, as bits.
fn bits_1b(eq: &Eq1b) -> BTreeMap<String, Option<u64>> {
    eq.outputs()
        .into_iter()
        .map(|(key, output)| {
            let bits = match output {
                Output1b::Float(v) | Output1b::Optional(Some(v)) => Some(v.to_bits()),
                Output1b::Optional(None) => None,
                Output1b::Flag(b) => Some(u64::from(b)),
                Output1b::Count(n) => Some(u64::from(n)),
            };
            (key.to_string(), bits)
        })
        .collect()
}

/// docs/unit-1c.md §4.8's name map from 1b's keys to 1c's: 1b's machine row is type 0.
fn key_from_1b(key: &str) -> String {
    match key {
        "p_m" => "type0.price",
        "v_m" => "type0.build_cost",
        "k" => "type0.services",
        "lambda_tilde_machine" => "type0.lambda_tilde",
        "b_tilde_machine" => "type0.b_tilde",
        other => other,
    }
    .to_string()
}

/// The map from 1a's keys to 1c's, through 1b's (docs/unit-1b.md §6, docs/unit-1c.md §4.8).
fn key_from_1a(key: &str) -> String {
    match key {
        "j_star" => "m_s".to_string(),
        "p" => "cat0.price".to_string(),
        "lambda_tilde_good" => "cat0.lambda_tilde".to_string(),
        "lambda_tilde_space" => "cat1.lambda_tilde".to_string(),
        "b_tilde_good" => "cat0.b_tilde".to_string(),
        "b_tilde_space" => "cat1.b_tilde".to_string(),
        "lambda_tilde_machine" => "type0.lambda_tilde".to_string(),
        "b_tilde_machine" => "type0.b_tilde".to_string(),
        other => key_from_1b(other),
    }
}

/// Every output of 1b's equilibrium equals its 1c counterpart bit for bit, and 1c has no
/// extra output that 1b's form would not have.
fn assert_equal_bits_1b(what: &str, b: &Eq1b, c: &Eq1c) {
    let c_bits = bits_1c(c);
    for (key, bits) in bits_1b(b) {
        let key_c = key_from_1b(&key);
        let other = c_bits
            .get(&key_c)
            .unwrap_or_else(|| panic!("{what}: 1c has no {key_c}"));
        assert_eq!(
            bits,
            *other,
            "{what}: {key} is {:?} in 1b and {:?} in 1c",
            bits.map(f64::from_bits),
            other.map(f64::from_bits)
        );
    }
    assert_eq!(c.bisection_steps, b.bisection_steps, "{what}");
    assert_eq!(
        (c.technique, c.tie, c.switches.len()),
        (0, None, 0),
        "{what}"
    );
}

/// Every output of 1a's equilibrium equals its 1c counterpart bit for bit. 1a reports its
/// cost system only at u = 1, so an absent 1a value is not compared, except φ, which both
/// report only at u = 1.
fn assert_equal_bits_1a(what: &str, a: &Eq1a, c: &Eq1c) {
    let c_bits = bits_1c(c);
    let mut compared = 0;
    for (key, output) in a.outputs() {
        let bits = match output {
            Output::Float(v) | Output::FlowOnly(Some(v)) => Some(v.to_bits()),
            Output::FlowOnly(None) => None,
            Output::Flag(b) => Some(u64::from(b)),
            Output::Count(n) => Some(u64::from(n)),
        };
        if bits.is_none() && !key.starts_with("phi_") {
            continue;
        }
        let key_c = key_from_1a(key);
        let other = c_bits
            .get(&key_c)
            .unwrap_or_else(|| panic!("{what}: 1c has no {key_c}"));
        assert_eq!(bits, *other, "{what}: {key} differs from 1c's {key_c}");
        compared += 1;
    }
    assert!(compared >= 32, "{what}: only {compared} outputs compared");
}

fn outcome_1b(result: &Result<Regime<Eq1b>, SolveError>) -> Outcome1c {
    match result {
        Ok(Regime::Interior(_)) => Outcome1c::Interior,
        Ok(Regime::BoundaryNoMargin { f_at_1 }) => Outcome1c::Boundary("f_at_1", f_at_1.to_bits()),
        Ok(Regime::NotViable { d_at_1 }) => Outcome1c::Boundary("d_at_1", d_at_1.to_bits()),
        Ok(Regime::NoInteriorAtZero { f_at_0 }) => Outcome1c::Boundary("f_at_0", f_at_0.to_bits()),
        Err(e) => Outcome1c::Error(format!("{e:?}")),
    }
}

/// Solves `params` as 1b and in one-type form, asserts the same outcome bit for bit, and
/// returns it. When interior, every 1b output, `d` (1b's D(x*)) and the identities are checked.
fn assert_nests_1b<S: Schedule + Clone + std::fmt::Debug>(
    what: &str,
    params: CategoryParams<S>,
) -> Outcome1c {
    let many = CategoryEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{what}: {e}"));
    let typed = MachineEconomy::new(MachineParams::from_categories(params))
        .unwrap_or_else(|e| panic!("{what}: {e}"));
    assert_eq!(
        many.user_cost().to_bits(),
        typed.block().user_costs()[0].to_bits(),
        "{what}: u"
    );
    let (b, c) = (many.solve(), typed.solve());
    let outcome = outcome_1b(&b);
    assert_eq!(outcome, outcome_1c(&c), "{what}");
    if let (Ok(Regime::Interior(b)), Ok(Regime::Interior(c))) = (&b, &c) {
        assert_equal_bits_1b(what, b, c);
        assert_eq!(
            c.d.to_bits(),
            many.at(c.x_star).d.to_bits(),
            "{what}: d = D(x*)"
        );
    }
    outcome
}

/// As [`assert_nests_1b`] for a 1a economy in category form, and every 1a output too.
fn assert_nests_1a(what: &str, params: Params) -> Outcome1c {
    let outcome = assert_nests_1b(what, CategoryParams::from_one_category(params.clone()));
    if outcome == Outcome1c::Interior {
        let a = interior(params.clone());
        let c = interior_1c(one_type(CategoryParams::from_one_category(params.clone())));
        assert_equal_bits_1a(what, &a, &c);
        assert_eq!(c.d.to_bits(), economy(params).at(c.x_star).d.to_bits());
    }
    outcome
}

#[test]
fn appendix_b_is_bit_identical() {
    // docs/unit-1c.md §3.3's M1: one type, θ 1, no operating recipe, the build recipe
    // (a, λ, b), 1b's δ and J_b, and no intermediate inputs.
    let params = one_type(CategoryParams::from_one_category(appendix_b()));
    assert_eq!(params.machine_types.len(), 1);
    let t = &params.machine_types[0];
    assert_eq!(t.task_efficiency, 1.0);
    assert_eq!(t.operating, Recipe::zero(1));
    assert_eq!(t.build, recipe(&[0.3], 0.05, 0.4));
    assert_eq!((t.delta, t.build_lag), (1.0, 1));
    assert_eq!(params.intermediate, no_inputs(2));
    assert_eq!(assert_nests_1a("G1", appendix_b()), Outcome1c::Interior);
    let a = interior(appendix_b());
    let c = interior_1c(params.clone());
    assert_eq!(a.bisection_steps, c.bisection_steps);
    assert_eq!(a.x_star.to_bits(), c.x_star.to_bits());
    assert_eq!(a.v_m.to_bits(), c.types[0].build_cost.to_bits());
    assert_eq!(c.types[0].operating_cost, 0.0);
    // d is 1a's D at x*.
    assert_eq!(
        c.d.to_bits(),
        economy(appendix_b()).at(a.x_star).d.to_bits()
    );
    // The published figures hold through the one-type form (SSRN p.30).
    near("x*", c.x_star, 0.86315, PUBLISHED);
    near("v", c.v, 0.54344, PUBLISHED);
    let e = economy_1c(params);
    check_identities_1c(&e, &c);
    check_fork_and_bounds_1c(&e, &c);
}

#[test]
fn every_1b_golden_instance_is_bit_identical() {
    // 1b's c1 set: 1a's 27 golden instances, and 1c's identities on each. G4's instances at
    // u = 1 with δ < 1 and ρ > 0 check each type's φ_w on the price side, where λ̃ ≠ λ̃^q.
    let instances = crate::c1_nesting::golden_instances();
    assert_eq!(instances.len(), 27);
    let mut price_side_phi = 0;
    for (what, params) in instances {
        assert_eq!(
            assert_nests_1a(&what, params.clone()),
            Outcome1c::Interior,
            "{what}"
        );
        let typed = one_type(CategoryParams::from_one_category(params));
        let e = economy_1c(typed.clone());
        let eq = interior_1c(typed);
        check_identities_1c(&e, &eq);
        check_fork_and_bounds_1c(&e, &eq);
        let t = &eq.types[0];
        if t.phi_w.is_some() && t.lambda_tilde != t.lambda_tilde_q {
            price_side_phi += 1;
        }
    }
    assert!(price_side_phi > 0);
    // Near full automation: x* rounds to 1.0 and 1 - x* = 4.6e-21 is carried.
    let deepest = CategoryParams::from_one_category(Params {
        lam: 0.0,
        schedule: linear(1.0, 1e-20, 1e-20),
        ..appendix_b()
    });
    assert_eq!(
        assert_nests_1b("G3 eta 1e-20", deepest.clone()),
        Outcome1c::Interior
    );
    let eq = interior_1c(one_type(deepest.clone()));
    assert_eq!(eq.x_star, 1.0);
    assert!(eq.one_minus_x_star > 4e-21 && eq.one_minus_x_star < 5e-21);
    assert_eq!(eq.categories[0].human, eq.one_minus_x_star);
    assert_eq!(eq.categories[0].lambda_tilde_q, eq.one_minus_x_star);
    let e = economy_1c(one_type(deepest));
    check_identities_1c(&e, &eq);
    check_fork_and_bounds_1c(&e, &eq);
    // 1b's own instances: C3 and its durable and rho = 0 forms.
    let fork = fork_economy();
    let mut list: Vec<(String, CategoryParams)> = vec![
        ("C3".to_string(), fork.clone()),
        (
            "C3d".to_string(),
            CategoryParams {
                rho: 0.04,
                delta: 0.35,
                build_lag: 2,
                ..fork.clone()
            },
        ),
        (
            "C3z".to_string(),
            CategoryParams {
                delta: 0.1,
                ..fork.clone()
            },
        ),
    ];
    // The seven C4 points: task automation, and recursive automation.
    for eta in [1.0, 0.5, 0.25, 0.1, 0.03] {
        list.push((format!("C4 eta {eta}"), crate::c4_paths::with_eta(eta)));
    }
    for lam in [0.025, 0.0] {
        list.push((
            format!("C4 lambda {lam}"),
            CategoryParams {
                lam,
                ..fork.clone()
            },
        ));
    }
    // The gap economy, its near-edge roots, and the roots on its edges.
    for workers in [
        4.5,
        5.0,
        C7_EDGE_BELOW_N,
        C7_EDGE_ABOVE_N,
        4.219565357117047,
        5.39015188299268,
    ] {
        list.push((format!("gap N {workers}"), gap_economy(workers)));
    }
    list.push((
        "sliver".to_string(),
        crate::c7_gaps_and_regimes::sliver_economy(C7_SLIVER_N),
    ));
    assert_eq!(list.len(), 17);
    for (what, params) in list {
        assert_eq!(
            assert_nests_1b(&what, params.clone()),
            Outcome1c::Interior,
            "{what}"
        );
        let e = economy_1c(one_type(params.clone()));
        let eq = interior_1c(one_type(params));
        check_identities_1c(&e, &eq);
        check_fork_and_bounds_1c(&e, &eq);
    }
}

#[test]
fn random_economies_are_bit_identical() {
    // 1a's G5 draws and 1b's C5 draws, from the same generator and seeds as their gates:
    // every draw, interior or not, until each set has WANTED interior economies.
    let mut skipped = 0;
    for (set, seed) in [
        (Set::Flow, SEED),
        (Set::Durable, SEED + 1),
        (Set::BuildLag, SEED + 2),
    ] {
        let mut rng = SplitMix64(seed);
        let (mut found, mut draws) = (0, 0);
        while found < WANTED {
            assert!(draws < MAX_DRAWS, "{set:?}");
            draws += 1;
            let params = draw(&mut rng, set);
            match assert_nests_1a(&format!("G5 {set:?} draw {draws}"), params) {
                Outcome1c::Interior => found += 1,
                _ => skipped += 1,
            }
        }
    }
    use crate::c5_random_categories::{draw as draw_1b, Set as Set1b, SEED_1B};
    for (set, seed) in [
        (Set1b::Flow, SEED_1B),
        (Set1b::Durable, SEED_1B + 1),
        (Set1b::BuildLag, SEED_1B + 2),
    ] {
        let mut rng = SplitMix64(seed);
        let (mut found, mut draws) = (0, 0);
        while found < WANTED {
            assert!(draws < MAX_DRAWS, "{set:?}");
            draws += 1;
            let params = draw_1b(&mut rng, set);
            let Ok(_) = CategoryEconomy::new(params.clone()) else {
                assert!(
                    MachineEconomy::new(one_type(params)).is_err(),
                    "1c accepted a draw 1b rejects"
                );
                skipped += 1;
                continue;
            };
            match assert_nests_1b(&format!("C5 {set:?} draw {draws}"), params) {
                Outcome1c::Interior => found += 1,
                _ => skipped += 1,
            }
        }
    }
    assert!(skipped > 0, "no skipped draw was compared");
}

/// What a regime row must give.
#[derive(Debug)]
enum Want {
    /// This diagnostic, whatever its value.
    Kind(&'static str),
    /// This diagnostic with exactly this value.
    Exact(&'static str, f64),
    /// This error.
    Error(SolveError),
}

fn check_row(what: &str, got: Outcome1c, want: Want) {
    match (&got, want) {
        (Outcome1c::Boundary(kind, _), Want::Kind(want_kind)) => {
            assert_eq!(*kind, want_kind, "{what}")
        }
        (Outcome1c::Boundary(kind, bits), Want::Exact(want_kind, value)) => {
            assert_eq!((*kind, *bits), (want_kind, value.to_bits()), "{what}");
        }
        (Outcome1c::Error(text), Want::Error(error)) => {
            assert_eq!(*text, format!("{error:?}"), "{what}");
        }
        (got, want) => panic!("{what}: got {got:?}, want {want:?}"),
    }
}

#[test]
fn regime_rows_nest() {
    // 1a's G8 rows (docs/unit-1a.md §6), exact ties included: d_at_1 is the least pivot, which
    // is D(1).
    let base = appendix_b();
    let schedule = |g0: f64, g1: f64| PowerSchedule {
        g0,
        g1,
        ..base.schedule
    };
    let tie_1 = economy(saturated(1.0)).at(1.0).n_d;
    let tie_lo = economy(saturated(1.0)).at(BRACKET_LO).n_d;
    let huge_u = |b: f64, land: f64| Params {
        a: 0.0,
        lam: 0.0,
        b,
        land,
        rho: SCALE_CEIL,
        build_lag: 10,
        ..base.clone()
    };
    let rows: Vec<(&str, Params, Want)> = vec![
        (
            "N 0.25",
            Params {
                workers: 0.25,
                ..base.clone()
            },
            Want::Kind("f_at_1"),
        ),
        (
            "lambda 0.6",
            Params {
                lam: 0.6,
                ..base.clone()
            },
            Want::Kind("f_at_1"),
        ),
        (
            "lambda 0.8",
            Params {
                lam: 0.8,
                ..base.clone()
            },
            Want::Kind("d_at_1"),
        ),
        (
            "rho 0.1",
            Params {
                a: 0.6,
                lam: 0.35,
                rho: 0.1,
                ..base.clone()
            },
            Want::Kind("d_at_1"),
        ),
        (
            "N 20",
            Params {
                workers: 20.0,
                work_cost: UniformWorkCost { chi_max: 0.05 },
                ..base.clone()
            },
            Want::Kind("f_at_0"),
        ),
        (
            "D(1) = 0 exactly",
            Params {
                lam: 0.7,
                ..base.clone()
            },
            Want::Exact("d_at_1", 0.0),
        ),
        (
            "D(1) = 0 exactly, a 0.5",
            Params {
                a: 0.5,
                lam: 0.1,
                schedule: schedule(1.0, 4.0),
                ..base.clone()
            },
            Want::Exact("d_at_1", 0.0),
        ),
        (
            "f(1) = 0 exactly",
            saturated(tie_1),
            Want::Exact("f_at_1", 0.0),
        ),
        (
            "f(lo) = 0 exactly",
            saturated(tie_lo),
            Want::Exact("f_at_0", 0.0),
        ),
        (
            "T/h - N = 5e-12",
            Params {
                workers: 10.0,
                land: 10.000000000005,
                work_cost: UniformWorkCost { chi_max: 1e-3 },
                ..base.clone()
            },
            Want::Kind("f_at_0"),
        ),
        (
            "a = 1 - 2^-53",
            Params {
                a: 1.0 - f64::EPSILON / 2.0,
                lam: 0.0,
                ..base.clone()
            },
            Want::Kind("f_at_0"),
        ),
        (
            "D(1) = -inf",
            Params {
                lam: 1e10,
                rho: SCALE_CEIL,
                build_lag: 10,
                ..base.clone()
            },
            Want::Exact("d_at_1", f64::NEG_INFINITY),
        ),
        (
            "p_m overflows",
            huge_u(1e10, 10.0),
            Want::Error(SolveError::NonFinite {
                what: "n_D(1) - n_S(1)",
            }),
        ),
        (
            "income overflows",
            huge_u(0.4, 1e10),
            Want::Error(SolveError::NonFinite { what: "income" }),
        ),
    ];
    for (what, params, want) in rows {
        let got = assert_nests_1a(what, params);
        check_row(what, got, want);
    }
    // 1b's c7 rows on the fork economy.
    let fork = fork_economy();
    let mut no_top = gap_economy(5.0);
    for c in &mut no_top.categories {
        c.density[2] = 0.0;
    }
    no_top.lam = 0.7;
    let rows: Vec<(&str, CategoryParams, Want)> = vec![
        (
            "fork N 0.2",
            CategoryParams {
                workers: 0.2,
                ..fork.clone()
            },
            Want::Kind("f_at_1"),
        ),
        (
            "fork lambda 0.6",
            CategoryParams {
                lam: 0.6,
                ..fork.clone()
            },
            Want::Kind("f_at_1"),
        ),
        (
            "fork lambda 0.8",
            CategoryParams {
                lam: 0.8,
                ..fork.clone()
            },
            Want::Kind("d_at_1"),
        ),
        (
            "fork N 20",
            CategoryParams {
                workers: 20.0,
                work_cost: UniformWorkCost { chi_max: 0.05 },
                ..fork.clone()
            },
            Want::Kind("f_at_0"),
        ),
        (
            "gap without its top tasks, lambda 0.7",
            no_top,
            Want::Exact("d_at_1", 0.0),
        ),
    ];
    for (what, params, want) in rows {
        let got = assert_nests_1b(what, params);
        check_row(what, got, want);
    }
    assert_eq!(
        assert_nests_1b(
            "fork N 6",
            CategoryParams {
                workers: 6.0,
                ..fork
            }
        ),
        Outcome1c::Interior
    );
    // The jump schedule breaks continuity; 1b and 1c refuse it with the same numbers.
    let jump = |_: ()| Params {
        workers: 4.0,
        land: 10.0,
        space: 1.0,
        a: 0.3,
        lam: 0.05,
        b: 0.4,
        schedule: Jump {
            x0: 0.875,
            below: 0.2,
            above: 1.0,
            slope: 0.1,
        },
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    };
    let many = CategoryEconomy::new(CategoryParams::from_one_category(jump(())))
        .unwrap()
        .solve();
    let typed = MachineEconomy::new(MachineParams::from_categories(
        CategoryParams::from_one_category(jump(())),
    ))
    .unwrap()
    .solve();
    assert!(
        matches!(many, Err(SolveError::LaborNotCleared { .. })),
        "{many:?}"
    );
    assert_eq!(outcome_1b(&many), outcome_1c(&typed));
}

#[test]
fn rejected_rows_nest() {
    // The rows 1a and 1b reject are rejected, each error named (docs/unit-1c.md §3.2). The
    // machine row's own checks are the type's, and its a < 1 and b > 0 are productivity and
    // the chain to land.
    let base = appendix_b();
    let rejected = |params: Params| {
        let one = Economy::new(params.clone()).expect_err("1a accepted it");
        let typed =
            MachineEconomy::new(one_type(CategoryParams::from_one_category(params.clone())))
                .expect_err("1c accepted it");
        (one, typed)
    };
    let in_type = |error: &ParamError, name: &str| match error {
        ParamError::Item {
            kind: "machine type",
            index: 0,
            error,
        } => {
            assert_eq!(error.name(), name, "{error}")
        }
        other => panic!("expected machine type 0's {name}, got {other:?}"),
    };
    let (one, typed) = rejected(Params {
        workers: 1e300,
        land: 1e-30,
        space: 1e300,
        b: 1e-30,
        ..base.clone()
    });
    assert_eq!(one, typed);
    for k in [1e20, CURVATURE_CEIL.next_up()] {
        let (one, typed) = rejected(steep(k));
        assert_eq!(one, typed);
        assert_eq!(typed.name(), "k");
    }
    let (_, typed) = rejected(Params {
        lam: 1e300,
        ..base.clone()
    });
    in_type(&typed, "build.labor");
    let (_, typed) = rejected(Params {
        rho: 1.0,
        build_lag: 2000,
        ..base.clone()
    });
    in_type(&typed, "u");
    let (_, typed) = rejected(Params {
        b: 0.0,
        ..base.clone()
    });
    in_type(&typed, "land");
    assert!(typed.to_string().contains("use no land"), "{typed}");
    let (_, typed) = rejected(Params {
        b: SCALE_FLOOR_HALF,
        ..base.clone()
    });
    in_type(&typed, "build.land");
    for a in [1.0, 1.5] {
        let (one, typed) = rejected(Params { a, ..base.clone() });
        assert_eq!(one.name(), "a");
        assert!(
            matches!(typed, ParamError::Invalid { name: "machine types", reason } if reason.contains("not productive")),
            "{typed:?}"
        );
    }
    let (one, typed) = rejected(Params {
        space: 1e300,
        ..base.clone()
    });
    assert_eq!(one.name(), "space");
    assert!(
        matches!(
            typed,
            ParamError::Item {
                kind: "category",
                index: 1,
                ..
            }
        ),
        "{typed:?}"
    );
    let (_, typed) = rejected(Params { space: 0.0, ..base });
    assert_eq!(typed.name(), "basket");
}

/// Half of SCALE_FLOOR: a land below the floor.
const SCALE_FLOOR_HALF: f64 = oracle::SCALE_FLOOR / 2.0;

#[test]
fn points_nest_on_a_grid() {
    // at(x) equals 1b's at(x) bit for bit on 1b's grid and at the bracket's ends.
    let base = appendix_b();
    let instances = [
        CategoryParams::from_one_category(base.clone()),
        CategoryParams::from_one_category(crate::g5_general_instances::durable()),
        CategoryParams::from_one_category(steep(CURVATURE_CEIL)),
        fork_economy(),
        CategoryParams {
            rho: 0.04,
            delta: 0.35,
            build_lag: 2,
            ..fork_economy()
        },
        gap_economy(5.0),
    ];
    for params in instances {
        let many = economy_1b(params.clone());
        let typed = economy_1c(one_type(params.clone()));
        for x in std::iter::once(BRACKET_LO)
            .chain((1..=100).map(|i| i as f64 / 100.0))
            .chain([0.0, 0.863_150_418_162_437, 1.0f64.next_down()])
        {
            let (b, c) = (many.at(x), typed.at(x));
            assert_eq!(c.technique, 0);
            for (name, p, q) in [
                ("gamma", b.gamma, c.gamma),
                ("J", b.j, c.j),
                ("D", b.d, c.d),
                ("V_m", b.v_m, c.build[0]),
                ("p_m", b.p_m, c.type_prices[0]),
                ("v", b.v, c.v),
                ("P_s", b.p_s, c.p_s),
                ("H_s", b.h_s, c.h_s),
                ("M_s", b.m_s, c.m_s),
                ("B_d", b.b_d, c.b_d),
                ("Y", b.y, c.y),
                ("K", b.k, c.services[0]),
                ("final hours", b.final_hours, c.final_hours),
                ("machine hours", b.machine_hours, c.machine_hours),
                ("n_D", b.n_d, c.n_d),
                ("n_S", b.n_s, c.n_s),
            ] {
                assert_eq!(p.to_bits(), q.to_bits(), "{name} at x = {x:e}, {params:?}");
            }
            assert_eq!(c.operating[0], 0.0);
            for j in 0..params.categories.len() {
                assert_eq!(b.prices[j].to_bits(), c.prices[j].to_bits(), "p_j at {x:e}");
                assert_eq!(b.human[j].to_bits(), c.human[j].to_bits(), "H_j at {x:e}");
                assert_eq!(
                    b.machine[j].to_bits(),
                    c.machine[j].to_bits(),
                    "M_j at {x:e}"
                );
            }
        }
    }
}

#[test]
fn flow_recipe_is_the_flow_economy() {
    // A zero build recipe with operating (a, λ, b) = G1's prices a machine service as SSRN's
    // flow benchmark at every u (check_dynamics R4; SSRN App B): the solve is 1a's G1 bit for
    // bit at any (ρ, δ, J), except u, φ (reported only at u = 1) and V_m, which is the flow
    // type's operating cost (its build cost is 0.0). It earns no interest.
    let g1 = interior(appendix_b());
    for (rho, delta, build_lag) in [(0.0, 1.0, 1), (0.05, 0.1, 3), (0.3, 0.5, 4)] {
        let flow = machine_type(
            1.0,
            recipe(&[0.3], 0.05, 0.4),
            Recipe::zero(1),
            delta,
            build_lag,
        );
        let params = MachineParams {
            rho,
            ..appendix_b_household(linear(1.0, 0.2, 0.8), rho, 1.0, vec![flow])
        };
        let what = format!("flow at (rho, delta, J) = ({rho}, {delta}, {build_lag})");
        let eq = interior_1c(params.clone());
        let bits = bits_1c(&eq);
        let mut compared = 0;
        for (key, output) in g1.outputs() {
            let want = match output {
                Output::Float(v) | Output::FlowOnly(Some(v)) => v.to_bits(),
                Output::Flag(b) => u64::from(b),
                Output::Count(n) => u64::from(n),
                Output::FlowOnly(None) => unreachable!("G1 is at u = 1"),
            };
            let key_c = match key {
                "u" => continue,
                "phi_w" | "phi_r" if rho > 0.0 => {
                    assert_eq!(bits[key], None, "{what}: {key}");
                    continue;
                }
                "v_m" => "type0.operating_cost".to_string(),
                other => key_from_1a(other),
            };
            assert_eq!(Some(want), bits[&key_c], "{what}: {key} vs {key_c}");
            compared += 1;
        }
        assert!(compared >= 39, "{what}: {compared}");
        assert_eq!(eq.types[0].build_cost, 0.0, "{what}");
        assert_eq!((eq.interest, eq.types[0].wealth), (0.0, 0.0), "{what}");
        let e = economy_1c(params);
        check_identities_1c(&e, &eq);
        check_fork_and_bounds_1c(&e, &eq);
    }
}
