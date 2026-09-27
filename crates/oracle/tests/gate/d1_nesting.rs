//! d1: units 1a to 1c as the one-type case of unit 1d, exactly (PLAN Phase 1's gate, R1;
//! docs/unit-1d.md §2.6 and §8). With one worker type of efficiency 1 and support 1 and no
//! human-required or reserved hours, every new operation of §5.1 is an exact no-op, so wherever
//! 1c has an equilibrium 1d has the same one bit for bit; where 1c returns a boundary regime,
//! 1d solves it, and its line values are 1c's diagnostics bit for bit.

use oracle::{
    CategoryParams, Eq1c, Eq1d, MachineEconomy, MachineParams, Margin, ParamError, Params, Regime,
    SolveError, UniformWorkCost, WorkerEconomy, WorkerParams, BRACKET_LO, CURVATURE_CEIL,
    SCALE_CEIL,
};

use crate::g5_random_economies::{draw, Set, SplitMix64, MAX_DRAWS, SEED, WANTED};
use crate::g8_regimes::{saturated, steep};
use crate::goldens_1b::*;
use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1b::*;
use crate::support_1c::*;
use crate::support_1d::*;

/// What a unit-1c economy gave, and what its one-type form gave in 1d.
#[derive(Debug, PartialEq)]
enum Nesting {
    /// 1c's equilibrium, 1d's bit for bit.
    Same,
    /// 1c's boundary regime, solved by 1d (a wall, an all-human corner or a root below lo),
    /// with 1d's line value at the boundary equal to 1c's diagnostic bit for bit.
    Solved(Margin),
    /// 1c's boundary regime, `LaborShort` in 1d, with the same line value.
    Short,
    /// 1c's equilibrium, but the wall holds further equilibria (an upward jump on it).
    MoreOnTheWall(usize),
    /// `NotViable` in both, with the same d(1).
    NotViable,
    /// `MultipleEquilibria` in both: (1c's count, 1d's).
    Multiple(usize, usize),
    /// The same error in both.
    Error(String),
}

/// Every 1c output of 1c's equilibrium equals 1d's under the same key, bit for bit; the one
/// worker's wage, hours and supply are v, N_a and n_S; the pool's hours are N_a; the new
/// residuals are 0.
fn assert_same(what: &str, economy: &MachineEconomy, c: &Eq1c, d: &Eq1d) {
    let d_bits = bits_1d(d);
    let mut compared = 0;
    for (key, bits) in bits_1c(c) {
        let other = d_bits
            .get(&key)
            .unwrap_or_else(|| panic!("{what}: 1d has no {key}"));
        assert_eq!(
            bits,
            *other,
            "{what}: {key} is {:?} in 1c and {:?} in 1d",
            bits.map(f64::from_bits),
            other.map(f64::from_bits)
        );
        compared += 1;
    }
    assert!(compared >= 52, "{what}: {compared}");
    assert_eq!(d.margin, Margin::Contestable, "{what}");
    assert_eq!(c.bisection_steps, d.bisection_steps, "{what}");
    let w = &d.workers[0];
    assert_eq!(w.wage.to_bits(), c.v.to_bits(), "{what}: wage");
    assert_eq!(w.hours.to_bits(), c.n_a.to_bits(), "{what}: hours");
    assert_eq!(d.n_pool.to_bits(), c.n_a.to_bits(), "{what}: n_pool");
    let n_s = economy.at_with(c.x_star, c.technique).n_s;
    assert_eq!(w.supply.to_bits(), n_s.to_bits(), "{what}: supply");
    assert!(w.pooled && w.premium == Some(1.0) && w.reserved_hours == 0.0);
    let r = d.residuals;
    assert_eq!((r.corner, r.reserved, r.basket), (0.0, 0.0, 0.0), "{what}");
    assert_eq!((d.required_hours, d.reserved_hours), (0.0, 0.0), "{what}");
    assert!(d.wall_switches.is_empty() || c.switches.len() < c.types.len());
}

/// Solves a unit-1c economy in 1c and in one-type 1d form and compares them (§8, d1).
fn nest(what: &str, params: MachineParams) -> Nesting {
    let economy = MachineEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{what}: {e}"));
    let workers = economy_1d(WorkerParams::from_machines(params));
    let (c, d) = (economy.solve(), workers.solve());
    let env = economy.envelope();
    let line_1 = workers.at_with(1.0, env.last()).excess_demand();
    let line_lo = workers.at_with(BRACKET_LO, env.first).excess_demand();
    match (&c, &d) {
        (Ok(Regime::Interior(c)), Ok(Regime::Interior(d))) => {
            assert_same(what, &economy, c, d);
            check_identities_1d(&workers, d);
            Nesting::Same
        }
        (Ok(Regime::Interior(_)), Err(SolveError::MultipleEquilibria { sign_changes, .. })) => {
            // The wall must hold the further equilibria: the line's part of the sequence has
            // one change, as in 1c.
            assert!(!workers.wall_switches().is_empty(), "{what}: {d:?}");
            Nesting::MoreOnTheWall(*sign_changes)
        }
        (Ok(Regime::BoundaryNoMargin { f_at_1 }), _) => {
            assert_eq!(f_at_1.to_bits(), line_1.to_bits(), "{what}: f_line_1");
            boundary(what, &d, line_1, line_lo)
        }
        (Ok(Regime::NoInteriorAtZero { f_at_0 }), _) => {
            assert_eq!(f_at_0.to_bits(), line_lo.to_bits(), "{what}: f_line_lo");
            boundary(what, &d, line_1, line_lo)
        }
        (Ok(Regime::NotViable { d_at_1 }), Ok(Regime::NotViable { d_at_1: e })) => {
            assert_eq!(d_at_1.to_bits(), e.to_bits(), "{what}: d_at_1");
            Nesting::NotViable
        }
        (
            Err(SolveError::MultipleEquilibria {
                sign_changes: a,
                switches: s,
            }),
            Err(SolveError::MultipleEquilibria {
                sign_changes: b,
                switches: t,
            }),
        ) => {
            assert_eq!(s, t, "{what}: the line's switches");
            Nesting::Multiple(*a, *b)
        }
        (Err(a), Err(b)) if a == b => Nesting::Error(format!("{a:?}")),
        (c, d) => panic!("{what}: 1c gave {c:?} and 1d {d:?}"),
    }
}

/// A 1c boundary regime in 1d: solved, or `LaborShort`; its line values are reported.
fn boundary(
    what: &str,
    d: &Result<Regime<Eq1d>, SolveError>,
    line_1: f64,
    line_lo: f64,
) -> Nesting {
    match d {
        Ok(Regime::Interior(eq)) => {
            let bits = |f: Option<f64>| f.map(f64::to_bits);
            assert_eq!(
                bits(eq.f_line_1),
                bits(Some(line_1).filter(|f| f.is_finite()))
            );
            assert_eq!(
                bits(eq.f_line_lo),
                bits(Some(line_lo).filter(|f| f.is_finite()))
            );
            if eq.margin == Margin::Contestable {
                assert!(
                    eq.x_star <= BRACKET_LO,
                    "{what}: a root in [0, lo], {}",
                    eq.x_star
                );
            }
            Nesting::Solved(eq.margin)
        }
        Err(SolveError::LaborShort { .. }) => Nesting::Short,
        other => panic!("{what}: 1c's boundary regime gave {other:?} in 1d"),
    }
}

/// 1a's economy through 1b and 1c.
fn nest_1a(what: &str, params: Params) -> Nesting {
    nest(
        what,
        MachineParams::from_categories(CategoryParams::from_one_category(params)),
    )
}

#[test]
fn every_1c_golden_instance_is_bit_identical() {
    // 1a's 27 golden instances.
    let instances = crate::c1_nesting::golden_instances();
    assert_eq!(instances.len(), 27);
    for (what, params) in instances {
        assert_eq!(nest_1a(&what, params), Nesting::Same, "{what}");
    }
    // 1b's C3, C3d, C3z, C4, the gap economy and its near-edge and on-edge roots, the sliver.
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
    for eta in [1.0, 0.5, 0.25, 0.1, 0.03] {
        list.push((format!("C4 eta {eta}"), crate::c4_paths::with_eta(eta)));
    }
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
    for (what, params) in list {
        assert_eq!(
            nest(&what, MachineParams::from_categories(params)),
            Nesting::Same,
            "{what}"
        );
    }
    // 1c's M3, M3z, the M4 path, M4t and the M5 path.
    let mut machines: Vec<(String, MachineParams)> = vec![
        ("M3".to_string(), m3(0.05)),
        ("M3z".to_string(), m3(0.0)),
        (
            "M4t".to_string(),
            MachineParams {
                workers: 8.0,
                ..m4(0.5)
            },
        ),
    ];
    for eta in [2.0, 1.0, 0.5] {
        machines.push((format!("M4 eta {eta}"), m4(eta)));
    }
    for rho in [0.0, 0.05, 0.1, 0.15, 0.3] {
        machines.push((format!("M5 rho {rho}"), m5(rho, 4.0, 1.0)));
    }
    let mut ties = 0;
    for (what, params) in machines {
        assert_eq!(nest(&what, params.clone()), Nesting::Same, "{what}");
        ties += usize::from(solved_1d(WorkerParams::from_machines(params)).tie.is_some());
    }
    assert_eq!(ties, 1, "M4t is a tie");
}

#[test]
fn random_economies_are_bit_identical() {
    // 1a's G5 draws, 1b's C5 draws and 1c's m6 draws, every draw, interior or not.
    let mut tally = std::collections::BTreeMap::<String, usize>::new();
    let mut count = |n: &Nesting| {
        let key = match n {
            Nesting::Solved(m) => format!("Solved({m:?})"),
            Nesting::MoreOnTheWall(_) => "MoreOnTheWall".to_string(),
            Nesting::Multiple(..) => "Multiple".to_string(),
            Nesting::Error(_) => "Error".to_string(),
            other => format!("{other:?}"),
        };
        *tally.entry(key).or_default() += 1;
    };
    for (set, seed) in [
        (Set::Flow, SEED),
        (Set::Durable, SEED + 1),
        (Set::BuildLag, SEED + 2),
    ] {
        let mut rng = SplitMix64(seed);
        let (mut found, mut draws) = (0, 0);
        while found < WANTED {
            assert!(draws < MAX_DRAWS);
            draws += 1;
            let n = nest_1a(&format!("G5 {set:?} draw {draws}"), draw(&mut rng, set));
            found += usize::from(n == Nesting::Same);
            count(&n);
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
            assert!(draws < MAX_DRAWS);
            draws += 1;
            let params = MachineParams::from_categories(draw_1b(&mut rng, set));
            if MachineEconomy::new(params.clone()).is_err() {
                assert!(WorkerEconomy::new(WorkerParams::from_machines(params)).is_err());
                continue;
            }
            let n = nest(&format!("C5 {set:?} draw {draws}"), params);
            found += usize::from(n == Nesting::Same);
            count(&n);
        }
    }
    for (samples, _) in crate::m6_random_leontief::all_sets() {
        for (i, s) in samples.iter().enumerate() {
            let n = nest(&format!("m6 {:?} {i}", s.set), s.economy.params().clone());
            count(&n);
        }
    }
    // Every kind the 1c draws hold is exercised; the tallies of 2026-09-27 are recorded in
    // docs/unit-1d.md §12.
    for key in ["Same", "Solved(Wall)", "Short", "NotViable", "Multiple"] {
        assert!(tally.contains_key(key), "no {key}: {tally:?}");
    }
    println!("d1 random tallies: {tally:?}");
}

#[test]
fn boundary_rows_are_solved() {
    // Every regime row where 1a, 1b or 1c returns BoundaryNoMargin or NoInteriorAtZero: 1d
    // solves it or refuses it as LaborShort, and its line values are 1c's diagnostics.
    let base = appendix_b();
    let tie_1 = economy(saturated(1.0)).at(1.0).n_d;
    let tie_lo = economy(saturated(1.0)).at(BRACKET_LO).n_d;
    let rows: Vec<(&str, Params, Nesting)> = vec![
        (
            "N 0.25",
            Params {
                workers: 0.25,
                ..base.clone()
            },
            Nesting::Short,
        ),
        (
            "lambda 0.6",
            Params {
                lam: 0.6,
                ..base.clone()
            },
            Nesting::Solved(Margin::Wall),
        ),
        (
            "N 20",
            Params {
                workers: 20.0,
                work_cost: UniformWorkCost { chi_max: 0.05 },
                ..base.clone()
            },
            Nesting::Solved(Margin::AllHuman),
        ),
        // Supply saturated at N = n_D(1): f is 0 on the whole wall and at its end; the piece
        // starts at an exact zero, so the junction is the equilibrium (docs/unit-1d.md §12
        // item 17).
        (
            "f(1) = 0 exactly",
            saturated(tie_1),
            Nesting::Solved(Margin::Wall),
        ),
        (
            "f(lo) = 0 exactly",
            saturated(tie_lo),
            Nesting::Solved(Margin::Contestable),
        ),
        (
            "T/h - N = 5e-12",
            Params {
                workers: 10.0,
                land: 10.000000000005,
                work_cost: UniformWorkCost { chi_max: 1e-3 },
                ..base.clone()
            },
            Nesting::Solved(Margin::Contestable),
        ),
        (
            "a = 1 - 2^-53",
            Params {
                a: 1.0 - f64::EPSILON / 2.0,
                lam: 0.0,
                ..base.clone()
            },
            Nesting::Solved(Margin::Contestable),
        ),
    ];
    for (what, params, want) in rows {
        assert_eq!(nest_1a(what, params), want, "{what}");
    }
    let fork = fork_economy();
    for (what, params, want) in [
        (
            "fork N 0.2",
            CategoryParams {
                workers: 0.2,
                ..fork.clone()
            },
            Nesting::Short,
        ),
        (
            "fork lambda 0.6",
            CategoryParams {
                lam: 0.6,
                ..fork.clone()
            },
            Nesting::Solved(Margin::Wall),
        ),
        (
            "fork N 20",
            CategoryParams {
                workers: 20.0,
                work_cost: UniformWorkCost { chi_max: 0.05 },
                ..fork.clone()
            },
            Nesting::Solved(Margin::AllHuman),
        ),
    ] {
        assert_eq!(
            nest(what, MachineParams::from_categories(params)),
            want,
            "{what}"
        );
    }
    // 1c's M4 rows: the wall at eta 0.25, the all-human corner at N 200.
    assert_eq!(nest("M4 eta 0.25", m4(0.25)), Nesting::Solved(Margin::Wall));
    let eq = solved_1d(WorkerParams::from_machines(m4(0.25)));
    close("M4 eta 0.25 v", eq.v, D0_M4_ETA025_V);
    close(
        "M4 eta 0.25 f_line_1",
        eq.f_line_1.unwrap(),
        D0_M4_ETA025_F_LINE_1,
    );
    close(
        "M4 eta 0.25 f_line_1 (1c)",
        eq.f_line_1.unwrap(),
        crate::goldens_1c::M4_REG_ETA025_F_AT_1,
    );
    let crowded = MachineParams {
        workers: 200.0,
        work_cost: UniformWorkCost { chi_max: 0.01 },
        ..m4(1.0)
    };
    assert_eq!(
        nest("M4 N 200", crowded.clone()),
        Nesting::Solved(Margin::AllHuman)
    );
    let (e, eq) = checked_1d(WorkerParams::from_machines(crowded));
    close("M4 N 200 v", eq.v, D0_M4_N200_V);
    close(
        "M4 N 200 f_line_lo",
        eq.f_line_lo.unwrap(),
        D0_M4_N200_F_LINE_LO,
    );
    close(
        "M4 N 200 f_line_lo (1c)",
        eq.f_line_lo.unwrap(),
        crate::goldens_1c::M4_REG_N200_F_AT_LO,
    );
    check_path_1d(&e);
}

#[test]
fn refusals_nest() {
    // NotViable rows with d(1) bit for bit, 1c's multiple equilibria, and 1c's rejected rows.
    let base = appendix_b();
    for (what, params) in [
        (
            "lambda 0.8",
            Params {
                lam: 0.8,
                ..base.clone()
            },
        ),
        (
            "rho 0.1",
            Params {
                a: 0.6,
                lam: 0.35,
                rho: 0.1,
                ..base.clone()
            },
        ),
        (
            "D(1) = 0 exactly",
            Params {
                lam: 0.7,
                ..base.clone()
            },
        ),
        (
            "D(1) = -inf",
            Params {
                lam: 1e10,
                rho: SCALE_CEIL,
                build_lag: 10,
                ..base.clone()
            },
        ),
    ] {
        assert_eq!(nest_1a(what, params), Nesting::NotViable, "{what}");
    }
    assert_eq!(nest("M4 eta 50", m4(50.0)), Nesting::NotViable);
    // The overflow rows fail with 1c's errors.
    let huge_u = |b: f64, land: f64| Params {
        a: 0.0,
        lam: 0.0,
        b,
        land,
        rho: SCALE_CEIL,
        build_lag: 10,
        ..base.clone()
    };
    for (what, params) in [
        ("p_m overflows", huge_u(1e10, 10.0)),
        ("income overflows", huge_u(0.4, 1e10)),
    ] {
        assert!(matches!(nest_1a(what, params), Nesting::Error(_)), "{what}");
    }
    // M5m: three equilibria in both.
    assert_eq!(nest("M5m", m5(0.1, 60.0, 0.2)), Nesting::Multiple(3, 3));
    // M5b (unit-1c.md §3.3, H4 and H1): 1c counts three, the boundary one of them; in 1d the
    // wall is labour-short to the end (f_∞ = 6.23 at H4), so the boundary is not an
    // equilibrium and the count is two. Both refuse.
    for chi_max in [1.0, 0.01] {
        let params = crate::m5_interest::hidden(chi_max);
        assert_eq!(nest("M5b", params.clone()), Nesting::Multiple(3, 2));
        let e = economy_1d(WorkerParams::from_machines(params));
        if chi_max == 1.0 {
            close("M5b f_end", e.wall_end().excess, D0_M5B_F_END);
        }
        assert!(e.wall_end().excess > 6.0);
    }
    // 1c's rejected rows are rejected, N and chi_max named as worker type 0's.
    let rejected = |params: Params| {
        let typed = MachineParams::from_categories(CategoryParams::from_one_category(params));
        let c = MachineEconomy::new(typed.clone()).expect_err("1c accepted it");
        let d = WorkerEconomy::new(WorkerParams::from_machines(typed)).expect_err("1d accepted it");
        (c, d)
    };
    let in_worker = |error: &ParamError, name: &str| match error {
        ParamError::Item {
            kind: "worker type",
            index: 0,
            error,
        } => assert_eq!(error.name(), name),
        other => panic!("expected worker type 0's {name}, got {other:?}"),
    };
    for (params, name) in [
        (
            Params {
                workers: 0.0,
                ..base.clone()
            },
            "workers",
        ),
        (
            Params {
                workers: 1e31,
                ..base.clone()
            },
            "workers",
        ),
        (
            Params {
                work_cost: UniformWorkCost { chi_max: f64::NAN },
                ..base.clone()
            },
            "chi_max",
        ),
    ] {
        let (c, d) = rejected(params);
        assert_eq!(c.name(), name);
        in_worker(&d, name);
    }
    for params in [
        steep(1e20),
        steep(CURVATURE_CEIL.next_up()),
        Params {
            lam: 1e300,
            ..base.clone()
        },
        Params {
            rho: 1.0,
            build_lag: 2000,
            ..base.clone()
        },
        Params {
            b: 0.0,
            ..base.clone()
        },
        Params {
            a: 1.0,
            ..base.clone()
        },
        Params {
            space: 1e300,
            ..base.clone()
        },
        Params {
            space: 0.0,
            ..base.clone()
        },
        Params {
            land: 0.0,
            ..base.clone()
        },
    ] {
        let (c, d) = rejected(params);
        assert_eq!(c, d);
    }
}

#[test]
fn points_nest_on_a_grid() {
    // at(x) equals 1c's at(x) bit for bit on 1b's grid, at 0 and at the bracket's ends.
    let base = appendix_b();
    let instances = [
        MachineParams::from_categories(CategoryParams::from_one_category(base)),
        MachineParams::from_categories(CategoryParams::from_one_category(steep(CURVATURE_CEIL))),
        MachineParams::from_categories(fork_economy()),
        MachineParams::from_categories(gap_economy(5.0)),
        m3(0.05),
        m4(1.0),
        m5(0.1, 4.0, 1.0),
    ];
    for params in instances {
        let c = economy_1c(params.clone());
        let d = economy_1d(WorkerParams::from_machines(params.clone()));
        for x in std::iter::once(BRACKET_LO)
            .chain((1..=100).map(|i| i as f64 / 100.0))
            .chain([0.0, 0.863_150_418_162_437, 1.0f64.next_down()])
        {
            let (a, b) = (c.at(x), d.at(x));
            assert_eq!(a.technique, b.technique);
            for (name, p, q) in [
                ("gamma", a.gamma, b.gamma),
                ("J", a.j, b.j),
                ("d", a.d, b.d),
                ("v", a.v, b.v),
                ("pi", a.task_price, b.task_price),
                ("P_s", a.p_s, b.p_s),
                ("H_s", a.h_s, b.h_s),
                ("M_s", a.m_s, b.m_s),
                ("B_d", a.b_d, b.b_d),
                ("Y", a.y, b.y),
                ("final hours", a.final_hours, b.final_hours),
                ("machine hours", a.machine_hours, b.machine_hours),
                ("n_D", a.n_d, b.n_d),
                ("n_S", a.n_s, b.n_s),
                ("f", a.excess_demand(), b.excess_demand()),
            ] {
                assert_eq!(p.to_bits(), q.to_bits(), "{name} at x = {x:e}, {params:?}");
            }
            for (name, p, q) in [
                ("type prices", &a.type_prices, &b.type_prices),
                ("operating", &a.operating, &b.operating),
                ("build", &a.build, &b.build),
                ("services", &a.services, &b.services),
                ("prices", &a.prices, &b.prices),
                ("human", &a.human, &b.human),
                ("machine", &a.machine, &b.machine),
            ] {
                let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                assert_eq!(bits(p), bits(q), "{name} at x = {x:e}");
            }
            assert_eq!(b.wages[0].to_bits(), a.v.to_bits());
            assert_eq!(b.p_s.to_bits(), b.base_p_s.to_bits());
        }
    }
}
