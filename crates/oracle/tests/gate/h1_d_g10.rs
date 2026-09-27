//! h1: D-G10, productivity and the chain to land on the per-period recipes (docs/unit-1g.md
//! §2.1, §4.6 and §8). Unit 1c asked that I − (A^op + A^I) be a nonsingular M-matrix and that
//! (I − (A^op + A^I))⁻¹(b^op + b^I) be positive; 1g asks it of A^q = A^op + Δ·A^I, and the chain
//! to land as a pattern. Every economy 1c accepted is accepted, and its results are unchanged
//! (the whole 1a-1f gate, unchanged, shows that); the rest of this group shows what the rule
//! now admits and what it still refuses.

use oracle::{
    MachineEconomy, MachineParams, MachineType, ParamError, Recipe, Regime, UniformWorkCost,
};

use crate::g5_random_economies::SplitMix64;
use crate::goldens_1g::*;
use crate::support::*;
use crate::support_1c::*;
use crate::support_1g::*;

#[test]
fn the_rule_is_on_the_per_period_matrix() {
    // M4's loom (δ 0.1) with 0.6 of its own service to operate and 0.6 to build: 1c's rule
    // refused it (1.2); per period it uses 0.66 and is productive, and the economy solves.
    let with_loom = |op: f64, build: f64, delta: f64| {
        let mut p = m4(1.0);
        p.machine_types[0].operating.machines[0] = op;
        p.machine_types[0].build.machines[0] = build;
        p.machine_types[0].delta = delta;
        p
    };
    let p = with_loom(0.6, 0.6, 0.1);
    assert!(!accepted_by_1c(&p.machine_types));
    let e = economy_1c(p);
    let (_, per_period) = physical_and_per_period(e.block().types());
    close("A^q's loom row", per_period[0], 0.6 + 0.1 * 0.6);
    match e.solve() {
        Ok(Regime::Interior(eq)) => check_identities_1c(&e, &eq),
        other => panic!("{other:?}"),
    }
    // Per period 0.6 + 0.1·4.1 = 1.01: not productive, though each recipe alone is.
    for (op, build) in [(0.6, 4.1), (0.0, 10.5), (1.0, 0.0)] {
        let err = MachineEconomy::new(with_loom(op, build, 0.1)).expect_err("not productive");
        assert!(err.to_string().contains("not productive"), "{err}");
        assert!(err.to_string().contains("per-period"), "{err}");
    }
    for (op, build) in [(0.6, 0.0), (0.0, 4.1)] {
        assert!(MachineEconomy::new(with_loom(op, build, 0.1)).is_ok());
    }
    // At δ = 1 the per-period matrix is 1c's sum, and 0.6 + 0.6 is refused.
    assert!(MachineEconomy::new(with_loom(0.6, 0.6, 1.0)).is_err());
}

#[test]
fn a0_and_the_horse_were_refused_and_are_accepted() {
    // A0 as one type, A0's chain and the horse's chain (docs/unit-1g.md §3.3): 1c's rule refuses
    // each (a nonpositive pivot of I − (A^op + A^I)); 1g accepts each, and the spectral radii are
    // the goldens'.
    let cases: Vec<(&str, Vec<MachineType>, f64, f64)> = vec![
        (
            "A0 one type",
            a0_one_type(0.0).machine_types,
            A0_RADIUS_PHYSICAL,
            A0_RADIUS_PER_PERIOD,
        ),
        (
            "A0's chain",
            a0(0.0).to_machine_params().unwrap().machine_types,
            A0_CHAIN_RADIUS_PHYSICAL,
            A0_CHAIN_RADIUS_PER_PERIOD,
        ),
        (
            "the horse",
            horse().to_machine_params().unwrap().machine_types,
            HORSE_RADIUS_PHYSICAL,
            HORSE_RADIUS_PER_PERIOD,
        ),
        (
            "S1",
            s1(0.05).to_machine_params().unwrap().machine_types,
            S1_RADIUS_PHYSICAL,
            S1_RADIUS_PER_PERIOD,
        ),
    ];
    for (what, types, physical, per_period) in cases {
        let n = types.len();
        let (p, q) = physical_and_per_period(&types);
        close_to(
            &format!("{what}: physical"),
            spectral_radius(n, &p),
            physical,
            1e-9,
        );
        close_to(
            &format!("{what}: per period"),
            spectral_radius(n, &q),
            per_period,
            1e-9,
        );
        assert!(per_period < 1.0);
        if what == "S1" {
            // S1 is yearly: 1c accepted it, and still does.
            assert!(accepted_by_1c(&types));
        } else {
            assert!(physical > 1.0 && !accepted_by_1c(&types), "{what}");
        }
    }
    for chain in [a0(0.0), horse(), s1(0.05)] {
        assert!(oracle::ChainEconomy::new(chain).is_ok());
    }
    assert!(MachineEconomy::new(a0_one_type(0.0)).is_ok());
}

/// A random block of K machine types as 1c's m6 draws them (docs/unit-1c.md §8), with δ drawn
/// down to 1e-3 so that builds embody many periods of services, on Appendix B's household.
fn random_block(rng: &mut SplitMix64) -> MachineParams {
    let k = 1 + (rng.next_u64() % 4) as usize;
    let mut types = Vec::with_capacity(k);
    for i in 0..k {
        let recipe_of = |rng: &mut SplitMix64| {
            let machines = (0..k)
                .map(|_| {
                    if rng.uniform(0.0, 1.0) < 0.6 {
                        0.0
                    } else {
                        rng.uniform(0.02, 3.0)
                    }
                })
                .collect();
            let labor = if rng.uniform(0.0, 1.0) < 0.2 {
                0.0
            } else {
                rng.uniform(0.01, 0.5)
            };
            let land = if rng.uniform(0.0, 1.0) < 0.2 {
                0.0
            } else {
                rng.uniform(0.05, 1.0)
            };
            Recipe {
                machines,
                labor,
                land,
            }
        };
        let operating = recipe_of(rng);
        let build = recipe_of(rng);
        let theta = if i == 0 || rng.uniform(0.0, 1.0) > 0.3 {
            rng.uniform(0.5, 2.0)
        } else {
            0.0
        };
        let delta = num_log_uniform(rng, 1e-3, 1.0);
        let lag = 1 + (rng.next_u64() % 5) as u32;
        types.push(machine_type(theta, operating, build, delta, lag));
    }
    let mut p = appendix_b_household(linear(1.0, 0.2, 0.8), 0.0, 1.0, types);
    p.rho = rng.uniform(0.0, 0.002);
    p.work_cost = UniformWorkCost { chi_max: 1.0 };
    p
}

/// e^U(ln lo, ln hi).
fn num_log_uniform(rng: &mut SplitMix64, lo: f64, hi: f64) -> f64 {
    rustyecon_core::num::exp(rng.uniform(rustyecon_core::num::ln(lo), rustyecon_core::num::ln(hi)))
}

#[test]
fn every_economy_accepted_before_is_accepted() {
    // 1c's rule reimplemented in the test: whatever it accepts, D-G10 accepts. Some draws only
    // D-G10 accepts (builds of many periods of their own services); those that are interior
    // satisfy every identity of unit 1c.
    let mut rng = SplitMix64(1979);
    let (mut both, mut only_now, mut neither, mut interior_now) = (0, 0, 0, 0);
    for _ in 0..3000 {
        let p = random_block(&mut rng);
        let before = accepted_by_1c(&p.machine_types);
        let now = MachineEconomy::new(p.clone());
        match (before, &now) {
            (true, Ok(_)) => both += 1,
            (true, Err(e)) => panic!("1c accepted and 1g refused: {e}: {p:?}"),
            (false, Ok(e)) => {
                only_now += 1;
                if let Ok(Regime::Interior(eq)) = e.solve() {
                    check_identities_1c(e, &eq);
                    interior_now += 1;
                }
            }
            (false, Err(_)) => neither += 1,
        }
    }
    eprintln!(
        "h1 draws: {both} both, {only_now} only 1g ({interior_now} interior), {neither} neither"
    );
    assert!(both > 300 && only_now > 300 && interior_now > 30 && neither > 30);
}

#[test]
fn the_chain_to_land_is_a_pattern() {
    let base = m4(1.0);
    let landless = |op: [f64; 3], build: [f64; 3]| {
        machine_type(1.0, recipe(&op, 0.1, 0.0), recipe(&build, 0.2, 0.0), 0.1, 1)
    };
    // Land through a build recipe's machine input is enough (the engine's service uses no land
    // but is built from power's).
    let mut p = base.clone();
    p.machine_types[0] = landless([0.0; 3], [0.0, 0.0, 0.1]);
    assert!(MachineEconomy::new(p).is_ok());
    // And through an operating one.
    let mut p = base.clone();
    p.machine_types[0] = landless([0.0, 0.0, 0.1], [0.0; 3]);
    assert!(MachineEconomy::new(p).is_ok());
    // A chain found in the second pass over the types: the loom runs on the engine's service,
    // which is landless and runs on power's, which uses land.
    let mut p = base.clone();
    p.machine_types[0] = landless([0.0, 0.1, 0.0], [0.0; 3]);
    p.machine_types[1] = machine_type(
        2.0,
        recipe(&[0.0, 0.0, 0.5], 0.02, 0.0),
        recipe(&[0.0, 0.1, 0.0], 0.1, 0.0),
        0.05,
        3,
    );
    assert!(MachineEconomy::new(p).is_ok());
    // A landless loop, the loom operating on the engine's service and the engine built from the
    // loom's: the lower type, the loom, is named.
    let mut p = base.clone();
    p.machine_types[0] = landless([0.0, 0.1, 0.0], [0.0; 3]);
    p.machine_types[1] = machine_type(
        2.0,
        recipe(&[0.0, 0.0, 0.0], 0.02, 0.0),
        recipe(&[0.1, 0.0, 0.0], 0.1, 0.0),
        0.05,
        3,
    );
    match MachineEconomy::new(p) {
        Err(ParamError::Item {
            kind: "machine type",
            index: 0,
            error,
        }) => assert_eq!(error.name(), "land"),
        other => panic!("{other:?}"),
    }
    // A chain to land through δ·a^I below the least subnormal: a clearing-side solve gives 0
    // there, the pattern does not. The loom reaches land only through a build input of 1e-300
    // of power's service at δ 1e-30, so its per-period input of power is 0.0.
    let mut p = base.clone();
    p.machine_types[0] = machine_type(
        1.0,
        recipe(&[0.0; 3], 0.3, 0.0),
        recipe(&[0.0, 0.0, 1e-300], 1.0, 0.0),
        1e-30,
        2,
    );
    let (_, per_period) = physical_and_per_period(&p.machine_types);
    assert_eq!(per_period[2], 0.0, "the per-period product underflows");
    assert!(MachineEconomy::new(p).is_ok());
}

#[test]
fn the_price_side_is_not_checked() {
    // A0's u·a^I = a(1 + ρ/δ): at ρ a tick of 3δ it is 1.2, and the price recursion diverges.
    // D-G10 validates the per-period matrix, so the economy is valid and NotViable.
    let d = per_week(0.1);
    let e = MachineEconomy::new(a0_one_type(3.0 * d)).expect("valid per period");
    assert!(e.block().totals().lambda_tilde.is_none());
    assert!(matches!(e.solve(), Ok(Regime::NotViable { d_at_1 }) if d_at_1 < 0.0));
    // At 5% a year it is 0.44, and A0R solves.
    let e = MachineEconomy::new(a0_one_type(rho_per_week(0.05))).unwrap();
    let u = e.block().user_costs()[0];
    close("u a^I", u * (0.3 / d), 0.3 * (1.0 + rho_per_week(0.05) / d));
    assert!(matches!(e.solve(), Ok(Regime::Interior(_))));
}
