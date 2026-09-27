//! C1: unit 1a as the one-category case of unit 1b, exactly (PLAN Phase 1's gate, R1;
//! docs/unit-1b.md §5.1 and §8). "Exactly" is bit equality: the category form repeats 1a's
//! floating-point operations in 1a's order.

use std::collections::BTreeMap;

use oracle::{
    CategoryEconomy, CategoryParams, Economy, Eq1a, Eq1b, Output, Output1b, ParamError, Params,
    PowerSchedule, Regime, Schedule, SolveError, UniformWorkCost, BRACKET_LO, CURVATURE_CEIL,
    SCALE_CEIL,
};

use crate::g5_random_economies::{draw, Set, SplitMix64, MAX_DRAWS, SEED, WANTED};
use crate::g8_regimes::{saturated, steep, Jump};
use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;

/// Unit 1a's outputs, by key, as bits (flags and counts as integers); `None` for a flow-only
/// output that is absent.
fn bits_1a(eq: &Eq1a) -> Vec<(&'static str, Option<u64>)> {
    eq.outputs()
        .into_iter()
        .map(|(key, output)| {
            let bits = match output {
                Output::Float(v) | Output::FlowOnly(Some(v)) => Some(v.to_bits()),
                Output::FlowOnly(None) => None,
                Output::Flag(b) => Some(u64::from(b)),
                Output::Count(n) => Some(u64::from(n)),
            };
            (key, bits)
        })
        .collect()
}

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

/// docs/unit-1b.md §6's name map from 1a's output keys to 1b's: J(x*) is M_s, the good is
/// category 0 and space category 1; every other key is the same.
fn key_1b(key: &str) -> &str {
    match key {
        "j_star" => "m_s",
        "p" => "cat0.price",
        "lambda_tilde_good" => "cat0.lambda_tilde",
        "lambda_tilde_space" => "cat1.lambda_tilde",
        "b_tilde_good" => "cat0.b_tilde",
        "b_tilde_space" => "cat1.b_tilde",
        other => other,
    }
}

/// Every output of 1a's equilibrium equals its 1b counterpart bit for bit. 1a reports its
/// cost system only at u = 1; 1b reports the price-side totals at every u, so an absent 1a
/// value is not compared, except φ, which both report only at u = 1.
fn assert_equal_bits(what: &str, a: &Eq1a, b: &Eq1b) {
    let b_bits = bits_1b(b);
    let mut compared = 0;
    for (key, bits) in bits_1a(a) {
        let other = b_bits
            .get(key_1b(key))
            .unwrap_or_else(|| panic!("{what}: 1b has no {}", key_1b(key)));
        if bits.is_none() && !key.starts_with("phi_") {
            continue;
        }
        assert_eq!(
            bits,
            *other,
            "{what}: {key} is {:?} in 1a and {:?} in 1b",
            bits.map(f64::from_bits),
            other.map(f64::from_bits)
        );
        compared += 1;
    }
    assert!(compared >= 32, "{what}: only {compared} outputs compared");
}

/// The outcome of a solve as comparable data: the regime's name and its diagnostic's bits,
/// every output's bits when interior, or the error.
#[derive(Debug, PartialEq)]
enum Outcome {
    Interior,
    Boundary(&'static str, u64),
    Error(String),
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

fn outcome_1a(result: &Result<Regime, SolveError>) -> Outcome {
    match result {
        Ok(Regime::Interior(_)) => Outcome::Interior,
        Ok(Regime::BoundaryNoMargin { f_at_1 }) => Outcome::Boundary("f_at_1", f_at_1.to_bits()),
        Ok(Regime::NotViable { d_at_1 }) => Outcome::Boundary("d_at_1", d_at_1.to_bits()),
        Ok(Regime::NoInteriorAtZero { f_at_0 }) => Outcome::Boundary("f_at_0", f_at_0.to_bits()),
        Err(e) => Outcome::Error(format!("{e:?}")),
    }
}

fn outcome_1b(result: &Result<Regime<Eq1b>, SolveError>) -> Outcome {
    match result {
        Ok(Regime::Interior(_)) => Outcome::Interior,
        Ok(Regime::BoundaryNoMargin { f_at_1 }) => Outcome::Boundary("f_at_1", f_at_1.to_bits()),
        Ok(Regime::NotViable { d_at_1 }) => Outcome::Boundary("d_at_1", d_at_1.to_bits()),
        Ok(Regime::NoInteriorAtZero { f_at_0 }) => Outcome::Boundary("f_at_0", f_at_0.to_bits()),
        Err(e) => Outcome::Error(format!("{e:?}")),
    }
}

/// Solves `params` as 1a and in category form, and asserts the same outcome bit for bit.
/// Returns the outcome.
fn assert_nests<S: Schedule + Clone>(what: &str, params: Params<S>) -> Outcome {
    let one = Economy::new(params.clone()).unwrap_or_else(|e| panic!("{what}: {e}"));
    let many = CategoryEconomy::new(CategoryParams::from_one_category(params))
        .unwrap_or_else(|e| panic!("{what}: {e}"));
    assert_eq!(
        one.user_cost().to_bits(),
        many.user_cost().to_bits(),
        "{what}: u"
    );
    let (a, b) = (one.solve(), many.solve());
    let outcome = outcome_1a(&a);
    assert_eq!(outcome, outcome_1b(&b), "{what}");
    if let (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) = (&a, &b) {
        assert_equal_bits(what, a, b);
    }
    outcome
}

#[test]
fn appendix_b_is_bit_identical() {
    let params = CategoryParams::from_one_category(appendix_b());
    // docs/unit-1b.md §3.3's C1: one segment, the good and space.
    assert_eq!(params.edges, [0.0, 1.0]);
    assert_eq!(
        params.categories,
        [category(1.0, 0.0, &[1.0]), category(1.0, 1.0, &[0.0])]
    );
    assert_eq!(assert_nests("G1", appendix_b()), Outcome::Interior);
    let a = interior(appendix_b());
    let b = interior_1b(params);
    assert_eq!(a.bisection_steps, b.bisection_steps);
    assert_eq!(a.p.to_bits(), b.categories[0].price.to_bits());
    let cs = a.cost_system.expect("u = 1");
    assert_eq!(cs.l_s.to_bits(), b.l_s.to_bits());
    assert_eq!(cs.b_s.to_bits(), b.b_s.to_bits());
    assert_eq!(
        cs.lambda_tilde[1].to_bits(),
        b.lambda_tilde_machine.to_bits()
    );
    assert_eq!(cs.b_tilde[1].to_bits(), b.b_tilde_machine.to_bits());
}

/// Every interior instance of 1a's gate (docs/unit-1a.md §6), as 1a's tests build it.
pub(crate) fn golden_instances() -> Vec<(String, Params)> {
    let base = appendix_b();
    let with_schedule = |eta: f64, g0: f64, g1: f64, lam: f64| Params {
        lam,
        schedule: PowerSchedule {
            eta,
            g0,
            g1,
            k: 1.0,
        },
        ..base.clone()
    };
    let durable = |rho: f64, delta: f64, build_lag: u32| Params {
        rho,
        delta,
        build_lag,
        ..base.clone()
    };
    let mut list = vec![
        ("G1".to_string(), base.clone()),
        ("G2 lambda 0".to_string(), with_schedule(1.0, 0.2, 0.8, 0.0)),
        ("G2 eta 0.5".to_string(), with_schedule(0.5, 0.2, 0.8, 0.05)),
    ];
    for eta in [1.0, 0.3, 0.1, 0.03, 0.01, 1e-6, 1e-20] {
        list.push((format!("G3 eta {eta:e}"), with_schedule(1.0, eta, eta, 0.0)));
    }
    for (tag, rho, delta, lag) in [
        ("A", 0.05, 1.0, 1),
        ("B", 0.05, 0.1, 1),
        ("C", 0.0, 0.1, 1),
        ("D", 0.05, 0.1, 3),
        ("F", 0.5, 0.5, 1),
        ("interest 1", 0.08, 0.3, 6),
        ("interest 2", 1e-9, 0.5, 4),
    ] {
        list.push((format!("G4 {tag}"), durable(rho, delta, lag)));
    }
    list.push((
        "G4 E".to_string(),
        Params {
            workers: 7.3,
            ..durable(0.05, 1.0, 1)
        },
    ));
    list.push((
        "G4 G, the viability edge".to_string(),
        Params {
            a: 1.0 - pow2(-19),
            lam: pow2(-20),
            b: 0.375,
            schedule: PowerSchedule {
                g0: 0.5,
                g1: 1.0,
                ..base.schedule
            },
            rho: 0.5,
            delta: 0.5,
            ..base.clone()
        },
    ));
    list.push(("G5 flow".to_string(), crate::g5_general_instances::flow()));
    list.push((
        "G5 durable".to_string(),
        crate::g5_general_instances::durable(),
    ));
    let ps_at_1 = economy(base.clone()).at(1.0).p_s;
    for (tag, params) in [
        (
            "G8 N 8",
            Params {
                workers: 8.0,
                ..base.clone()
            },
        ),
        (
            "G8 N 7.35",
            Params {
                workers: 7.35,
                ..base.clone()
            },
        ),
        (
            "G8 funded tie",
            Params {
                workers: 7.41560525573509,
                ..base.clone()
            },
        ),
        (
            "G8 lemma tie",
            Params {
                land: 4.0 * ps_at_1,
                ..base.clone()
            },
        ),
        ("G8 curvature ceiling", steep(CURVATURE_CEIL)),
        (
            "G5 saturated supply (R321)",
            Params {
                workers: 2.2310703064623203,
                land: 12.534605676139824,
                space: 0.3247530084073593,
                a: 0.06616273179304444,
                lam: 0.09729595503340215,
                b: 0.8904454223397135,
                schedule: PowerSchedule {
                    eta: 1.0,
                    g0: 0.015811448855388807,
                    g1: 1.3506072330297898,
                    k: 4.9847528799139935,
                },
                work_cost: UniformWorkCost {
                    chi_max: 0.5018953974986752,
                },
                rho: 0.08927722845190324,
                delta: 0.9759438021953466,
                build_lag: 5,
            },
        ),
    ] {
        list.push((tag.to_string(), params));
    }
    list
}

#[test]
fn every_1a_golden_instance_is_bit_identical() {
    let instances = golden_instances();
    assert_eq!(instances.len(), 27);
    for (what, params) in instances {
        assert_eq!(assert_nests(&what, params), Outcome::Interior, "{what}");
    }
    // The deepest point of the automation path: x* rounds to 1.0, and 1 - x* = 4.6e-21 is
    // carried in the top segment (docs/unit-1b.md §5.1 step 5).
    let base = appendix_b();
    let params = CategoryParams::from_one_category(Params {
        lam: 0.0,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 1e-20,
            g1: 1e-20,
            k: 1.0,
        },
        ..base
    });
    let eq = interior_1b(params.clone());
    assert_eq!(eq.x_star, 1.0);
    assert!(eq.one_minus_x_star > 4e-21 && eq.one_minus_x_star < 5e-21);
    let good = &eq.categories[0];
    assert_eq!(good.human, eq.one_minus_x_star);
    assert_eq!(eq.h_s, eq.one_minus_x_star);
    // Every hours-type output carries it (§5.1 step 5), not only H and H_s: the good's
    // final hours, and at lambda = 0 its clearing-side row and L_s^q, which are its hours.
    // From the double x* = 1.0 each of them would be 0.
    assert_eq!(
        good.final_hours.to_bits(),
        (good.output * eq.one_minus_x_star).to_bits()
    );
    assert_eq!(good.lambda_tilde_q, eq.one_minus_x_star);
    assert_eq!(eq.l_s_q, eq.one_minus_x_star);
    let economy = economy_1b(params);
    check_identities_1b(&economy, &eq);
    check_fork_and_bounds(&economy, &eq);
}

#[test]
fn random_1a_economies_are_bit_identical() {
    // G5's three sets of draws, from the same generator and seeds as 1a's gate: every draw,
    // interior or not, until each set has WANTED interior economies.
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
            match assert_nests(&format!("{set:?} draw {draws}"), params) {
                Outcome::Interior => found += 1,
                _ => skipped += 1,
            }
        }
    }
    assert!(skipped > 0, "no skipped draw was compared");
}

#[test]
fn regime_rows_nest() {
    // G8's rows (docs/unit-1a.md §6), exact ties included.
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
            Want::Exact("d_at_1", 0.0f64),
        ),
        (
            "D(1) = 0 exactly, a 0.5",
            Params {
                a: 0.5,
                lam: 0.1,
                schedule: schedule(1.0, 4.0),
                ..base.clone()
            },
            Want::Exact("d_at_1", 0.0f64),
        ),
        (
            "f(1) = 0 exactly",
            saturated(tie_1),
            Want::Exact("f_at_1", 0.0f64),
        ),
        (
            "f(lo) = 0 exactly",
            saturated(tie_lo),
            Want::Exact("f_at_0", 0.0f64),
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
        // assert_nests compares 1a and 1b bit for bit; the row's own value pins the kind,
        // and the diagnostic where it is exact.
        let got = assert_nests(what, params);
        match (&got, want) {
            (Outcome::Boundary(kind, _), Want::Kind(want_kind)) => assert_eq!(*kind, want_kind),
            (Outcome::Boundary(kind, bits), Want::Exact(want_kind, value)) => {
                assert_eq!((*kind, *bits), (want_kind, value.to_bits()), "{what}");
            }
            (Outcome::Error(text), Want::Error(error)) => {
                assert_eq!(*text, format!("{error:?}"), "{what}");
            }
            (got, want) => panic!("{what}: got {got:?}, want {want:?}"),
        }
    }
    // The jump schedule breaks continuity; both units refuse it with the same numbers.
    let jump = Params {
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
    let one = Economy::new(clone_jump(&jump)).unwrap().solve();
    let many = CategoryEconomy::new(CategoryParams::from_one_category(clone_jump(&jump)))
        .unwrap()
        .solve();
    assert!(
        matches!(one, Err(SolveError::LaborNotCleared { .. })),
        "{one:?}"
    );
    assert_eq!(outcome_1a(&one), outcome_1b(&many));
}

/// `Jump` is not `Clone`; build a copy field by field.
fn clone_jump(p: &Params<Jump>) -> Params<Jump> {
    Params {
        schedule: Jump {
            x0: p.schedule.x0,
            below: p.schedule.below,
            above: p.schedule.above,
            slope: p.schedule.slope,
        },
        work_cost: p.work_cost,
        ..*p
    }
}

#[test]
fn rejected_rows_nest() {
    // The same rows are rejected, naming the same parameter, except h, which is space's
    // weight in category form.
    let base = appendix_b();
    for (params, name_1b) in [
        (
            Params {
                workers: 1e300,
                land: 1e-30,
                space: 1e300,
                b: 1e-30,
                ..base.clone()
            },
            "workers",
        ),
        (steep(1e20), "k"),
        (steep(CURVATURE_CEIL.next_up()), "k"),
        (
            Params {
                lam: 1e300,
                ..base.clone()
            },
            "lam",
        ),
        (
            Params {
                rho: 1.0,
                build_lag: 2000,
                ..base.clone()
            },
            "u",
        ),
        (
            Params {
                space: 1e300,
                ..base.clone()
            },
            "weight",
        ),
        (
            Params {
                space: 0.0,
                ..base.clone()
            },
            "basket",
        ),
    ] {
        let one = Economy::new(params.clone()).expect_err("1a accepted it");
        let many = CategoryEconomy::new(CategoryParams::from_one_category(params.clone()))
            .expect_err("1b accepted it");
        assert_eq!(many.name(), name_1b, "{params:?}: {many}");
        if name_1b != "weight" && name_1b != "basket" {
            assert_eq!(one, many, "{params:?}");
        } else {
            assert_eq!(one.name(), "space");
        }
    }
    // h = 1e300 is category 1's weight.
    let err = CategoryEconomy::new(CategoryParams::from_one_category(Params {
        space: 1e300,
        ..base
    }))
    .unwrap_err();
    assert!(
        matches!(
            err,
            ParamError::Item {
                kind: "category",
                index: 1,
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn points_nest_on_a_grid() {
    // at(x)'s aggregates equal 1a's Point bit for bit, on the grid and at the bracket's ends.
    let base = appendix_b();
    let instances = [
        base.clone(),
        crate::g5_general_instances::flow(),
        crate::g5_general_instances::durable(),
        steep(CURVATURE_CEIL),
        Params {
            lam: 0.0,
            schedule: PowerSchedule {
                eta: 1.0,
                g0: 1e-20,
                g1: 1e-20,
                k: 1.0,
            },
            ..base
        },
    ];
    for params in instances {
        let one = economy(params.clone());
        let many = economy_1b(CategoryParams::from_one_category(params.clone()));
        for x in std::iter::once(BRACKET_LO)
            .chain((1..=100).map(|i| i as f64 / 100.0))
            .chain([0.0, 0.863_150_418_162_437, 1.0f64.next_down()])
        {
            let (a, b) = (one.at(x), many.at(x));
            for (name, p, q) in [
                ("x", a.x, b.x),
                ("gamma", a.gamma, b.gamma),
                ("J", a.j, b.m_s),
                ("D", a.d, b.d),
                ("V_m", a.v_m, b.v_m),
                ("p_m", a.p_m, b.p_m),
                ("v", a.v, b.v),
                ("p", a.p, b.prices[0]),
                ("P_s", a.p_s, b.p_s),
                ("Y", a.y, b.y),
                ("K", a.k, b.k),
                ("final hours", a.final_hours, b.final_hours),
                ("machine hours", a.machine_hours, b.machine_hours),
                ("n_D", a.n_d, b.n_d),
                ("n_S", a.n_s, b.n_s),
            ] {
                assert_eq!(p.to_bits(), q.to_bits(), "{name} at x = {x:e}, {params:?}");
            }
            assert_eq!(b.prices[1], 1.0, "space is priced at r = 1");
            assert_eq!(b.b_d, params.space);
        }
    }
}

#[test]
fn fork_at_g1() {
    // docs/unit-1b.md §7 C1; SSRN eq 12-13, main.tex:459 and :469.
    let eq = interior_1b(CategoryParams::from_one_category(appendix_b()));
    let (good, space) = (&eq.categories[0], &eq.categories[1]);
    for (name, got, want) in [
        ("L* of the good", good.l_star, C1_GOOD_L_STAR),
        (
            "lambda-tilde of the good",
            good.lambda_tilde,
            C1_GOOD_LAMBDA_TILDE,
        ),
        ("b-tilde of the good", good.b_tilde, C1_GOOD_B_TILDE),
        ("p of the good", good.price, C1_GOOD_P),
        ("v/p of the good", good.real_wage, C1_GOOD_REAL_WAGE),
        (
            "v/b-tilde of the good",
            good.wage_ceiling.expect("b-tilde > 0"),
            C1_GOOD_WAGE_CEILING,
        ),
        ("v/B_s", eq.rent_ceiling, C1_RENT_CEILING),
        ("v/P_s", eq.real_wage, C1_REAL_WAGE),
        ("share of the good", good.share, C1_GOOD_SHARE),
    ] {
        close(name, got, want);
    }
    // The good has L-bar = 1 and no direct land: the floor is 1/L-bar = 1, and v/p is
    // strictly between it and the ceiling.
    assert_eq!(good.l_bar, 1.0);
    assert_eq!(good.wage_floor, 1.0);
    assert!(1.0 < good.real_wage && good.real_wage < C1_GOOD_WAGE_CEILING);
    assert!(eq.real_wage < eq.rent_ceiling);
    // Space is the pass-through row: p = r = 1, no tasks, and v/p = v bit for bit, both
    // ends of the pair attained.
    assert_eq!(space.price, 1.0);
    assert_eq!(space.real_wage.to_bits(), eq.v.to_bits());
    assert_eq!(
        (space.l_star, space.l_bar, space.human, space.machine),
        (0.0, 0.0, 0.0, 0.0)
    );
    assert_eq!(space.wage_floor, eq.v);
    assert_eq!(space.wage_ceiling, Some(eq.v));
    check_fork_and_bounds(
        &economy_1b(CategoryParams::from_one_category(appendix_b())),
        &eq,
    );
}

/// (η, v/p of the good, v, q = r/p of the good) along G3's path (docs/unit-1b.md §7 C2).
pub const G3_PATH: [(f64, f64, f64, f64); 5] = [
    (1.0, C2_ETA_1_GOOD_REAL_WAGE, C2_ETA_1_V, C2_ETA_1_Q),
    (0.3, C2_ETA_0_3_GOOD_REAL_WAGE, C2_ETA_0_3_V, C2_ETA_0_3_Q),
    (0.1, C2_ETA_0_1_GOOD_REAL_WAGE, C2_ETA_0_1_V, C2_ETA_0_1_Q),
    (
        0.03,
        C2_ETA_0_03_GOOD_REAL_WAGE,
        C2_ETA_0_03_V,
        C2_ETA_0_03_Q,
    ),
    (
        0.01,
        C2_ETA_0_01_GOOD_REAL_WAGE,
        C2_ETA_0_01_V,
        C2_ETA_0_01_Q,
    ),
];

/// G3's economy at η in category form: λ = 0, γ = η(1 + x).
pub fn g3_path(eta: f64) -> CategoryParams {
    CategoryParams::from_one_category(Params {
        lam: 0.0,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: eta,
            g1: eta,
            k: 1.0,
        },
        ..appendix_b()
    })
}

#[test]
fn fork_along_g3() {
    // SSRN p.14's corollary and App. C p.31: along task automation v falls toward 0, while
    // the wage in the good (no direct land) rises toward gamma(1)/J(1) = 4/3 and stays above
    // 1/L-bar = 1; r/p >= 1/(v L-bar), so q = r/p grows without bound.
    let (mut last_v, mut last_wage) = (f64::INFINITY, 0.0);
    for (eta, wage, v, q) in G3_PATH {
        let e = economy_1b(g3_path(eta));
        let eq = interior_1b(g3_path(eta));
        let good = &eq.categories[0];
        let tag = |what: &str| format!("{what} at eta = {eta}");
        close(&tag("v/p of the good"), good.real_wage, wage);
        close(&tag("v"), eq.v, v);
        close(&tag("q = r/p"), 1.0 / good.price, q);
        assert!(good.real_wage >= 1.0 / good.l_bar, "{}", tag("floor"));
        assert!(
            1.0 / good.price >= 1.0 / (eq.v * good.l_bar),
            "{}",
            tag("r/p")
        );
        assert!(good.real_wage < C2_LIMIT_GOOD_REAL_WAGE, "{}", tag("limit"));
        assert!(
            eq.v < last_v && good.real_wage > last_wage,
            "{}",
            tag("monotone")
        );
        (last_v, last_wage) = (eq.v, good.real_wage);
        check_fork_and_bounds(&e, &eq);
    }
    close("the limit 4/3", C2_LIMIT_GOOD_REAL_WAGE, 4.0 / 3.0);
}
