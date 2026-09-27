//! e1: units 1a to 1d in parcel form, exactly (PLAN Phase 1's gate, R1; docs/unit-1e.md §2.12
//! and §8). With one enclosed parcel of quality 1 and every type in the dependence form, every
//! new operation of §5.1 is an exact no-op, so wherever 1d has an equilibrium 1e has the same one
//! bit for bit; 1d's `LaborShort` economies are idle-land equilibria whose f_∞ is 1d's excess.
//! The exit option switched off (s₀ = s̲ = 0) is the dependence form, and an exit priced out of
//! use is G1.

use oracle::{
    Branch, CategoryParams, Eq1d, Eq1e, ExitForm, ExitLand, LandMarket, MachineEconomy,
    MachineParams, Margin, ParamError, Params, ParcelEconomy, Regime, SolveError, WorkerEconomy,
    WorkerParams,
};

use crate::g5_random_economies::{draw, Set, SplitMix64, MAX_DRAWS, SEED, WANTED};
use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1b::*;
use crate::support_1c::*;
use crate::support_1d::*;
use crate::support_1e::*;

/// What a unit-1d economy gave, and what its parcel form gave in 1e.
#[derive(Debug, PartialEq)]
enum Nesting {
    /// 1d's equilibrium, 1e's bit for bit.
    Same,
    /// 1d's `LaborShort`, an idle-land equilibrium in 1e with f_∞ 1d's excess.
    Idle,
    /// `NotViable` in both, with the same d(1).
    NotViable,
    /// `MultipleEquilibria` in both: (1d's count, 1e's).
    Multiple(usize, usize),
    /// Rejected by both.
    Rejected,
    /// 1d's `LaborShort` whose idle stretch closes on the walk's ceiling, a jump to +∞ that is
    /// not a reserved shortage: refused, as 1d refuses such a change of side on the line
    /// (docs/unit-1e.md §12).
    Ceiling,
    /// The same error in both.
    Error(String),
}

/// Every 1d output of 1d's equilibrium equals 1e's under the same key, bit for bit, and 1e's
/// land and exit are the scarce, unused ones.
fn assert_same(what: &str, d: &Eq1d, e: &Eq1e) {
    let e_bits = bits_1e(e);
    let mut compared = 0;
    for (key, bits) in bits_1d(d) {
        let other = e_bits
            .get(&key)
            .unwrap_or_else(|| panic!("{what}: 1e has no {key}"));
        assert_eq!(
            bits,
            *other,
            "{what}: {key} is {:?} in 1d and {:?} in 1e",
            bits.map(f64::from_bits),
            other.map(f64::from_bits)
        );
        compared += 1;
    }
    assert!(compared >= 60, "{what}: {compared}");
    assert_eq!(d.bisection_steps, e.base.bisection_steps, "{what}");
    assert_eq!(e.land_market, LandMarket::Scarce, "{what}");
    assert_eq!(e.exit_land, ExitLand::Unused, "{what}");
    assert_eq!(
        (e.rent, e.plot_rent, e.scan_points),
        (1.0, 0.0, 0),
        "{what}"
    );
    assert_eq!(
        (e.residuals.partition, e.residuals.commons),
        (0.0, 0.0),
        "{what}"
    );
    assert_eq!(
        (e.land.rented_plots, e.land.idle, e.land.commons_occupied),
        (0.0, 0.0, 0.0)
    );
    assert!(e.workers.iter().all(|w| w.branch == Branch::Dependence));
}

/// Solves a unit-1d economy in 1d and in parcel form and compares them (§8, e1).
fn nest(what: &str, params: WorkerParams) -> Nesting {
    let Ok(workers) = WorkerEconomy::new(params.clone()) else {
        let e = ParcelEconomy::new(e0(params.clone()));
        assert!(e.is_err(), "{what}: 1d rejects, 1e accepts");
        return Nesting::Rejected;
    };
    let parcels = economy_1e(e0(params));
    let (d, e) = (workers.solve(), parcels.solve());
    match (&d, &e) {
        (Ok(Regime::Interior(d)), Ok(Regime::Interior(e))) => {
            assert_same(what, d, e);
            check_identities_1e(&parcels, e);
            Nesting::Same
        }
        (Err(SolveError::LaborShort { excess, .. }), Ok(Regime::Interior(e))) => {
            assert_eq!(e.land_market, LandMarket::Idle, "{what}");
            assert_eq!(
                e.f_end.map(f64::to_bits),
                Some(*excess).filter(|f| f.is_finite()).map(f64::to_bits),
                "{what}: f_end is 1d's excess"
            );
            check_identities_1e(&parcels, e);
            Nesting::Idle
        }
        (
            Err(SolveError::LaborShort { .. }),
            Err(SolveError::NonFinite {
                what: "n_D - n_S beside a change of side",
            }),
        ) => Nesting::Ceiling,
        (Ok(Regime::NotViable { d_at_1 }), Ok(Regime::NotViable { d_at_1: other })) => {
            assert_eq!(d_at_1.to_bits(), other.to_bits(), "{what}: d_at_1");
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
            // one more change of side where f_∞ > 0 (§2.12)
            let extra = usize::from(workers.wall_end().excess > 0.0);
            assert_eq!(*b, a + extra, "{what}");
            Nesting::Multiple(*a, *b)
        }
        (Err(a), Err(b)) if a == b => Nesting::Error(format!("{a:?}")),
        (d, e) => panic!("{what}: 1d gave {d:?} and 1e {e:?}"),
    }
}

/// A unit-1c economy through 1d's form.
fn nest_1c(what: &str, params: MachineParams) -> Nesting {
    nest(what, WorkerParams::from_machines(params))
}

/// 1a's economy through 1b, 1c and 1d.
fn nest_1a(what: &str, params: Params) -> Nesting {
    nest_1c(
        what,
        MachineParams::from_categories(CategoryParams::from_one_category(params)),
    )
}

/// Unit 1d's golden instances (docs/unit-1d.md §3.3) and what each is in 1d.
fn instances_1d() -> Vec<(&'static str, WorkerParams)> {
    vec![
        ("W1", goodspace(4.0, 0.6, 1.0)),
        ("W2", goodspace(0.25, 0.05, 1.0)),
        ("W3", goodspace(4.0, 0.6, 3.0)),
        ("W4", goodspace(20.0, 0.05, 0.05)),
        ("B1", baumol_one(1.0, 8.0, 0.25)),
        ("B2", baumol_one(1.0, 4.0, 0.25)),
        ("B2 no tail", baumol_one(1.0, 4.0, 0.0)),
        ("BP 0.1", baumol_one(0.1, 8.0, 0.25)),
        ("BP 1e-6", baumol_one(1e-6, 8.0, 0.25)),
        ("E1", entrant_trained(8.0, 3.0, 1.0, 1.0)),
        ("E2", entrant_trained(8.0, 1.0, 1.0, 1.0)),
        ("E3", entrant_trained(8.0, 2.0, 0.3, 1.0)),
        ("E4", entrant_trained(8.0, 3.0, 0.3, 1.0)),
        ("E5", entrant_trained(40.0, 2.0, 1.0, 0.05)),
        ("E6", entrant_trained(8.0, 0.5, 1.0, 1.0)),
        ("E7", three_types(3.0, 0.6)),
        ("E8", three_types(1.53, 0.5)),
        ("F1", full(4.0, 3.0, 1.0)),
        ("F2", full(4.0, 3.0, 0.25)),
        ("F3", full(16.0, 1.5, 0.5)),
        ("X1", wall_switch_economy(0.5)),
        ("X2", wall_switch_economy(1.0)),
        ("X3", wall_switch_economy(0.003)),
        ("J1", entrant_trained(40.0, 1.0, 1.0, 0.05)),
    ]
}

#[test]
fn every_1d_golden_instance_is_bit_identical() {
    // 1a's 27 golden instances.
    let instances = crate::c1_nesting::golden_instances();
    assert_eq!(instances.len(), 27);
    for (what, params) in instances {
        assert_eq!(nest_1a(&what, params), Nesting::Same, "{what}");
    }
    // 1b's fork, path and gap economies; 1c's M3, M4 and M5.
    for (what, params) in [
        ("C3", fork_economy()),
        ("C4 eta 0.5", crate::c4_paths::with_eta(0.5)),
        ("C4 eta 0.03", crate::c4_paths::with_eta(0.03)),
        ("gap", gap_economy(5.0)),
    ] {
        let n = nest_1c(what, MachineParams::from_categories(params));
        assert!(matches!(n, Nesting::Same | Nesting::Idle), "{what}: {n:?}");
    }
    for (what, params) in [
        ("M3", m3(0.05)),
        ("M3z", m3(0.0)),
        ("M4", m4(1.0)),
        ("M4 eta 2", m4(2.0)),
        ("M4 eta 0.5", m4(0.5)),
        ("M5 rho 0.1", m5(0.1, 4.0, 1.0)),
        ("M5 rho 0.3", m5(0.3, 4.0, 1.0)),
    ] {
        assert_eq!(nest_1c(what, params), Nesting::Same, "{what}");
    }
    // 1d's instances: each equilibrium bit for bit, W2, W3 and E6 on idle land.
    for (what, params) in instances_1d() {
        let n = nest(what, params);
        let want = if matches!(what, "W2" | "W3" | "E6") {
            Nesting::Idle
        } else {
            Nesting::Same
        };
        assert_eq!(n, want, "{what}");
    }
}

#[test]
fn random_economies_are_bit_identical() {
    // 1a's G5, 1b's C5, 1c's m6 and 1d's d7 draws, every one, through 1d and parcel form.
    let mut tally = std::collections::BTreeMap::<String, usize>::new();
    let mut count = |n: &Nesting| {
        let key = match n {
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
                continue;
            }
            let n = nest_1c(&format!("C5 {set:?} draw {draws}"), params);
            found += usize::from(n == Nesting::Same);
            count(&n);
        }
    }
    for (samples, _) in crate::m6_random_leontief::all_sets() {
        for (i, s) in samples.iter().enumerate() {
            let n = nest_1c(&format!("m6 {:?} {i}", s.set), s.economy.params().clone());
            count(&n);
        }
    }
    for (samples, _) in crate::d7_random_workers::all_sets() {
        for (i, s) in samples.iter().enumerate() {
            let n = nest(&format!("d7 {:?} {i}", s.set), s.economy.params().clone());
            if matches!(s.result, Err(SolveError::LaborShort { .. })) {
                assert!(
                    matches!(n, Nesting::Idle | Nesting::Ceiling),
                    "d7 {:?} {i}",
                    s.set
                );
            }
            count(&n);
        }
    }
    // The kinds the draws hold; the tallies of 2026-09-27 are in docs/unit-1e.md §12.
    for key in ["Same", "Idle", "NotViable", "Multiple"] {
        assert!(tally.contains_key(key), "no {key}: {tally:?}");
    }
    assert!(tally.get("Ceiling").copied().unwrap_or(0) <= 1, "{tally:?}");
    println!("e1 random tallies: {tally:?}");
}

#[test]
fn labor_short_rows_are_idle() {
    // 1d's W2, W3 and E6: LaborShort in 1d, idle-land equilibria here with f_end 1d's excess,
    // bit for bit (+∞ at E6, reported as absent).
    for (what, params) in [
        ("W2", goodspace(0.25, 0.05, 1.0)),
        ("W3", goodspace(4.0, 0.6, 3.0)),
        ("E6", entrant_trained(8.0, 0.5, 1.0, 1.0)),
    ] {
        let Err(SolveError::LaborShort { excess, .. }) = economy_1d(params.clone()).solve() else {
            panic!("{what} is LaborShort in 1d");
        };
        let (_, eq) = checked_1e(e0(params));
        assert_eq!(eq.land_market, LandMarket::Idle, "{what}");
        assert_eq!(
            eq.f_end.map(f64::to_bits),
            Some(excess).filter(|f| f.is_finite()).map(f64::to_bits),
            "{what}"
        );
        assert!(eq.land.idle > 0.0, "{what}");
    }
    close(
        "W2's f_∞",
        solved_1e(e0(goodspace(0.25, 0.05, 1.0))).f_end.unwrap(),
        I1_F_END,
    );
}

#[test]
fn refusals_nest() {
    // NotViable with d_at_1 bit for bit.
    assert_eq!(nest_1c("M4 eta 50", m4(50.0)), Nesting::NotViable);
    // Rejected rows are rejected, land's rules named for parcel 0.
    let base = e0(goodspace(4.0, 0.05, 1.0));
    for (land, name) in [
        (0.0, "acreage"),
        (f64::NAN, "acreage"),
        (1e31, "acreage"),
        (-1.0, "acreage"),
    ] {
        let mut p = base.clone();
        p.parcels[0].acreage = land;
        match ParcelEconomy::new(p) {
            Err(ParamError::Item { kind, index, error }) => {
                assert_eq!((kind, index, error.name()), ("parcel", 0, name));
            }
            other => panic!("land {land}: {other:?}"),
        }
        let mut w = goodspace(4.0, 0.05, 1.0);
        w.land = land;
        assert_eq!(WorkerEconomy::new(w).unwrap_err().name(), "land");
    }
    // 1d's other rules keep 1d's errors.
    let mut w = goodspace(4.0, 0.05, 1.0);
    w.worker_types[0].support = 0.0;
    assert_eq!(
        ParcelEconomy::new(e0(w.clone())).unwrap_err(),
        WorkerEconomy::new(w).unwrap_err()
    );
    let mut w = goodspace(4.0, 0.05, 1.0);
    w.human_required = vec![0.0];
    assert_eq!(
        ParcelEconomy::new(e0(w.clone())).unwrap_err(),
        WorkerEconomy::new(w).unwrap_err()
    );
    let mut w = goodspace(4.0, 0.05, 1.0);
    w.rho = -1.0;
    assert_eq!(nest("rho -1", w), Nesting::Rejected);
}

#[test]
fn points_nest_on_a_grid() {
    // at(x) is 1d's at(x) bit for bit on a grid, on G1, M4 and E1.
    for (what, params) in [
        ("G1", goodspace(4.0, 0.05, 1.0)),
        ("M4", WorkerParams::from_machines(m4(1.0))),
        ("E1", entrant_trained(8.0, 3.0, 1.0, 1.0)),
    ] {
        let d = economy_1d(params.clone());
        let e = economy_1e(e0(params));
        for i in 0..=100 {
            let x = f64::from(i) / 100.0;
            let (a, b) = (d.at(x), e.at(x));
            assert_eq!(format!("{a:?}"), format!("{:?}", b.point), "{what} at {x}");
            assert_eq!(b.market_land, economy_1e_land(&e), "{what}");
        }
    }
}

/// T of an economy, the land a point in 1d form uses.
fn economy_1e_land(e: &ParcelEconomy) -> f64 {
    e.enclosed_land()
}

#[test]
fn exit_option_off() {
    // Every E0 instance without reserved hours with every type Priced(0, 0, h), h 0 and 1: the
    // dependence result bit for bit, but the branch, which is the floor's; no scan.
    let mut list: Vec<(String, WorkerParams)> = crate::c1_nesting::golden_instances()
        .into_iter()
        .map(|(what, p)| {
            (
                what,
                WorkerParams::from_machines(MachineParams::from_categories(
                    CategoryParams::from_one_category(p),
                )),
            )
        })
        .collect();
    for (what, p) in instances_1d() {
        if p.reserved.iter().flatten().all(|&r| r == 0.0) {
            list.push((what.to_string(), p));
        }
    }
    let mut n = 0;
    for (what, params) in list {
        let dependence = economy_1e(e0(params.clone())).solve();
        for h in [0.0, 1.0] {
            let mut p = e0(params.clone());
            p.exits = vec![priced(0.0, 0.0, h); p.worker_types.len()];
            let off = economy_1e(p).solve();
            match (&dependence, &off) {
                (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) => {
                    assert_eq!(b.scan_points, 0, "{what}");
                    let (a_bits, b_bits) = (bits_1e(a), bits_1e(b));
                    for (key, bits) in &a_bits {
                        if key.ends_with(".branch") {
                            continue;
                        }
                        assert_eq!(Some(bits), b_bits.get(key), "{what} h {h}: {key}");
                    }
                    assert!(b.workers.iter().all(|w| w.branch == Branch::Floor));
                    n += 1;
                }
                (a, b) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{what}"),
            }
        }
    }
    assert_eq!(n, 2 * 39);
}

#[test]
fn a_dead_exit_is_dependence() {
    // D: exit (1, 0, 1) with q_enc = 1 below G1's q = 2.77: no plot pays, every exiter stands
    // on the floor s̲ = 0, which is the dependence form: G1 bit for bit (N-iv, N-vi).
    let g1 = solved_1e(e0(goodspace(4.0, 0.05, 1.0)));
    let (_, d) = checked_1e(dead_exit());
    let d_bits = bits_1e(&d);
    for (key, bits) in bits_1d(&g1.base) {
        assert_eq!(Some(&bits), d_bits.get(&key), "{key}");
    }
    assert_eq!(d.exit_land, ExitLand::Enclosed);
    assert_eq!(d.plot_rent, 1.0);
    assert_eq!(d.workers[0].branch, Branch::Floor);
    assert_eq!(d.workers[0].exit_value, 0.0);
    assert_eq!(d.land.rented_plots, 0.0);
    assert_eq!(d.base.margin, Margin::Contestable);
    close("q", d.q, D_Q);
    assert!(d.q > d.workers[0].threshold.unwrap());
    assert!(matches!(dead_exit().exits[0], ExitForm::Priced(_)));
    // the same economy with a floor s̲ = 0.1: a priced exit that is not dependence
    let mut p = dead_exit();
    p.exits = vec![priced(1.0, 0.1, 1.0)];
    let other = solved_1e(p);
    assert!(other.base.n_a < d.base.n_a);
}
