//! h6: plants as machine types (docs/unit-1g.md §2.6, §4.5, §5.3 and §8; CAPACITY.md): a plant
//! on a flow type rewritten to its long run, the bundle plant in closed form (at s1 the flow
//! economy exactly), any other recipe a fixed point.

use oracle::{
    s1_size, MachineEconomy, ParamError, Plant, PlantEconomy, PlantEq, PlantError, PlantRecipe,
    Recipe, Regime, MAX_PLANT_STEPS, PLANT_TOL,
};
use rustyecon_core::num;

use crate::goldens_1g::*;
use crate::support::*;
use crate::support_1c::*;
use crate::support_1g::*;

/// The plants' long run, known interior.
fn long_run(economy: &PlantEconomy) -> Box<PlantEq> {
    match economy.solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{other:?}"),
    }
}

/// The identities of §4.5 at a long run: O = θ·p, u·V = (1 − θ)·p, ζ^θ·κ^(1−θ) = 1, V = κ·P_K,
/// the plant K* = κ·X and the bundles ζ·X, and 1c's identities.
fn check_plants(economy: &PlantEconomy, eq: &PlantEq) {
    let e = economy_1c(economy.long_run(&eq.ratios));
    check_identities_1c(&e, &eq.eq);
    for (r, (k, plant)) in eq.plants.iter().zip(economy.plants()) {
        let theta = plant.bundle_share;
        let t = &eq.eq.types[*k];
        assert_eq!(r.machine_type, *k);
        close_to(
            "zeta^theta kappa^(1-theta)",
            num::pow(r.zeta, theta) * num::pow(r.kappa, 1.0 - theta),
            1.0,
            1e-14,
        );
        close_to("kappa/zeta", r.kappa / r.zeta, r.ratio, 1e-14);
        close_to("O = theta p", t.operating_cost, theta * t.price, 1e-12);
        close_to(
            "uV = (1 - theta) p",
            t.user_cost * t.build_cost,
            (1.0 - theta) * t.price,
            1e-12,
        );
        close_to("the capital share", r.capital_share, 1.0 - theta, 1e-12);
        close_to(
            "V = kappa P_K",
            t.build_cost,
            r.kappa * r.plant_price,
            1e-13,
        );
        close_to(
            "O = zeta c_f",
            t.operating_cost,
            r.zeta * r.bundle_cost,
            1e-13,
        );
        assert_eq!(r.stock, r.kappa * t.services);
        assert_eq!(r.bundles, r.zeta * t.services);
        // Marginal cost c_f·ζ/θ is the price.
        close_to("MC = p", r.bundle_cost * r.zeta / theta, t.price, 1e-12);
    }
}

#[test]
fn a_bundle_plant_at_s1_is_the_flow_economy() {
    // P1: L2 with a plant on engine and power, θ 0.8, δ 10% a year a week, J 1, ρ 0, built from
    // s1 bundles of the desk's own recipe. Its long run is L2's flow economy (CAPACITY's "at s1
    // exactly the registered instance").
    let flow = interior_1c(l2(0.0));
    close("L2: v", flow.v, L2_V);
    close("L2: x*", flow.x_star, L2_X_STAR);
    close("s1", s1_size(PLANT_THETA, per_week(0.1)), P1_S1);
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, bundle_plant(None)), (1, bundle_plant(None))],
    )
    .unwrap();
    let eq = long_run(&economy);
    check_plants(&economy, &eq);
    assert_eq!((eq.steps, eq.gap), (0, 0.0));
    for (what, got, want) in [
        ("x*", eq.eq.x_star, flow.x_star),
        ("v", eq.eq.v, flow.v),
        ("P_s", eq.eq.p_s, flow.p_s),
        ("Y", eq.eq.y, flow.y),
        ("N_a", eq.eq.n_a, flow.n_a),
    ] {
        close_to(&format!("P1 against L2: {what}"), got, want, 1e-13);
    }
    assert_eq!(eq.eq.interest, 0.0);
    for k in 0..2 {
        close_to("p", eq.eq.types[k].price, flow.types[k].price, 1e-13);
        close_to("X", eq.eq.types[k].services, flow.types[k].services, 1e-13);
        let r = &eq.plants[k];
        close_to("zeta = theta", r.zeta, PLANT_THETA, 1e-15);
        close_to(
            "kappa = theta^(-theta/(1-theta))",
            r.kappa,
            num::pow(0.8, -4.0),
            1e-14,
        );
    }
    close("P1: engine kappa", eq.plants[0].kappa, P1_ENGINE_KAPPA);
    close("P1: engine plant", eq.plants[0].stock, P1_ENGINE_STOCK);
    close(
        "P1: engine P_K",
        eq.plants[0].plant_price,
        P1_ENGINE_PLANT_PRICE,
    );
    close("P1: engine V", eq.eq.types[0].build_cost, P1_ENGINE_BUILD);
    close("P1: power kappa", eq.plants[1].kappa, P1_POWER_KAPPA);
    close("P1: power plant", eq.plants[1].stock, P1_POWER_STOCK);
    close(
        "P1: power P_K",
        eq.plants[1].plant_price,
        P1_POWER_PLANT_PRICE,
    );
    close("P1: power V", eq.eq.types[1].build_cost, P1_POWER_BUILD);
    // P_K = s1·c_f for the bundle.
    close_to(
        "P_K = s1 c_f",
        eq.plants[0].plant_price,
        P1_S1 * eq.plants[0].bundle_cost,
        1e-13,
    );
}

#[test]
fn plants_need_d_g10() {
    // The bundle plant's build is (1 − θ)/δ bundles of its own recipe, 99 at θ 0.8 and 10% a
    // year a week: 1c's rule refuses the long run, D-G10 accepts it.
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, bundle_plant(None)), (1, bundle_plant(None))],
    )
    .unwrap();
    let Ok(Regime::Interior(eq)) = economy.solve() else {
        panic!()
    };
    let p = economy.long_run(&eq.ratios);
    close_to(
        "kappa s1 = (1 - theta)/delta",
        eq.plants[0].kappa * P1_S1,
        0.2 / per_week(0.1),
        1e-13,
    );
    assert!(!accepted_by_1c(&p.machine_types));
    assert!(MachineEconomy::new(p).is_ok());
}

#[test]
fn a_bundle_plant_at_another_size() {
    // P1S: s = 1. The long run is the flow economy with every planted recipe times
    // m = θ^(−θ)(1 − θ)^(−(1−θ))(δs)^(1−θ).
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, bundle_plant(Some(1.0))), (1, bundle_plant(Some(1.0)))],
    )
    .unwrap();
    let eq = long_run(&economy);
    check_plants(&economy, &eq);
    aggregates(
        "P1S",
        &eq.eq,
        [
            P1S_X_STAR,
            P1S_ONE_MINUS_X_STAR,
            P1S_V,
            P1S_P_S,
            P1S_Y,
            P1S_N_A,
            P1S_INCOME,
        ],
    );
    let m = num::pow(0.8, -0.8) * num::pow(0.2, -0.2) * num::pow(per_week(0.1), 0.2);
    close("m", m, P1S_M);
    let mut scaled = l2(0.0);
    for t in &mut scaled.machine_types {
        t.operating.machines.iter_mut().for_each(|a| *a *= m);
        t.operating.labor *= m;
        t.operating.land *= m;
    }
    let q = interior_1c(scaled);
    close_to(
        "P1S against the scaled flow economy: v",
        eq.eq.v,
        q.v,
        1e-13,
    );
    close_to("x*", eq.eq.x_star, q.x_star, 1e-13);
}

#[test]
fn a_plant_with_interest() {
    // P1R: P1 at ρ 5% a year a week. The bundle's ratio is still free of prices, but u > δ and
    // the long run is not L2's.
    let economy = PlantEconomy::new(
        l2(rho_per_week(0.05)),
        vec![(0, bundle_plant(None)), (1, bundle_plant(None))],
    )
    .unwrap();
    let eq = long_run(&economy);
    check_plants(&economy, &eq);
    aggregates(
        "P1R",
        &eq.eq,
        [
            P1R_X_STAR,
            P1R_ONE_MINUS_X_STAR,
            P1R_V,
            P1R_P_S,
            P1R_Y,
            P1R_N_A,
            P1R_INCOME,
        ],
    );
    close("P1R: interest", eq.eq.interest, P1R_INTEREST);
    close("P1R: engine p", eq.eq.types[0].price, P1R_ENGINE_PRICE);
    close("P1R: engine plant", eq.plants[0].stock, P1R_ENGINE_STOCK);
    close("P1R: power p", eq.eq.types[1].price, P1R_POWER_PRICE);
    close("P1R: power plant", eq.plants[1].stock, P1R_POWER_STOCK);
    assert!((eq.eq.v - L2_V).abs() > 0.05);
}

#[test]
fn a_fixed_recipe_is_a_fixed_point() {
    // P2: a plant of labour 0.5 and land 0.5 a unit on both desks. κ/ζ moves with prices; the
    // long run is the fixed point (CAPACITY check 2c: v 0.13869).
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, labour_land_plant()), (1, labour_land_plant())],
    )
    .unwrap();
    let eq = long_run(&economy);
    check_plants(&economy, &eq);
    // The record (docs/unit-1g.md §12 item 5): 42 steps from the unplanted economy's ratios.
    assert!(eq.steps < MAX_PLANT_STEPS);
    assert_eq!(eq.steps, 42, "P2's steps");
    assert!(eq.gap <= PLANT_TOL);
    eprintln!("P2: {} steps, gap {:e}", eq.steps, eq.gap);
    aggregates(
        "P2",
        &eq.eq,
        [
            P2_X_STAR,
            P2_ONE_MINUS_X_STAR,
            P2_V,
            P2_P_S,
            P2_Y,
            P2_N_A,
            P2_INCOME,
        ],
    );
    for (k, [ratio, zeta, kappa, price, stock, plant_price]) in [
        [
            P2_ENGINE_RATIO,
            P2_ENGINE_ZETA,
            P2_ENGINE_KAPPA,
            P2_ENGINE_PRICE,
            P2_ENGINE_STOCK,
            P2_ENGINE_PLANT_PRICE,
        ],
        [
            P2_POWER_RATIO,
            P2_POWER_ZETA,
            P2_POWER_KAPPA,
            P2_POWER_PRICE,
            P2_POWER_STOCK,
            P2_POWER_PLANT_PRICE,
        ],
    ]
    .into_iter()
    .enumerate()
    {
        let r = &eq.plants[k];
        close("P2: ratio", r.ratio, ratio);
        close("P2: zeta", r.zeta, zeta);
        close("P2: kappa", r.kappa, kappa);
        close("P2: price", eq.eq.types[k].price, price);
        close("P2: stock", r.stock, stock);
        close("P2: P_K", r.plant_price, plant_price);
        close_to(
            "P_K = 0.5 v + 0.5",
            r.plant_price,
            0.5 * eq.eq.v + 0.5,
            1e-15,
        );
    }
    // The ratios the equilibrium's prices give are the ratios it was solved at.
    for (a, b) in economy.ratios_at(&eq.eq).iter().zip(&eq.ratios) {
        assert!((num::ln(*a) - num::ln(*b)).abs() <= PLANT_TOL);
    }
}

#[test]
fn a_fixed_recipe_beside_a_bundle() {
    // One plant of each kind: the bundle's ratio stays in closed form while the other moves.
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, bundle_plant(None)), (1, labour_land_plant())],
    )
    .unwrap();
    let eq = long_run(&economy);
    check_plants(&economy, &eq);
    assert!(eq.steps > 1);
    let bundle = (1.0 - PLANT_THETA) / (PLANT_THETA * per_week(0.1) * P1_S1);
    close_to("the bundle's ratio", eq.ratios[0], bundle, 1e-15);
}

#[test]
fn no_plant_is_the_type_bit_for_bit() {
    // θ = 1: no plant. The long run is the unplanted economy, bit for bit, at any ratio.
    let none = Plant {
        bundle_share: 1.0,
        delta: per_week(0.1),
        build_lag: 1,
        recipe: PlantRecipe::Fixed(recipe(&[0.0, 0.0], 0.5, 0.5)),
    };
    let economy = PlantEconomy::new(l2(0.0), vec![(0, none.clone())]).unwrap();
    assert_eq!(economy.long_run(&[7.0]), l2(0.0));
    let eq = long_run(&economy);
    assert_eq!(bits_1c(&eq.eq), bits_1c(&interior_1c(l2(0.0))));
    assert_eq!((eq.plants[0].zeta, eq.plants[0].kappa), (1.0, 0.0));
    assert_eq!(eq.plants[0].stock, 0.0);
    // And beside a fixed plant it changes nothing of the other's fixed point.
    let with = PlantEconomy::new(l2(0.0), vec![(0, none), (1, labour_land_plant())]).unwrap();
    let alone_on_power = PlantEconomy::new(l2(0.0), vec![(1, labour_land_plant())]).unwrap();
    let (a, b) = (long_run(&with), long_run(&alone_on_power));
    assert_eq!(bits_1c(&a.eq), bits_1c(&b.eq));
    assert_eq!(a.steps, b.steps);
}

#[test]
fn every_step_must_be_interior() {
    // An unplanted economy that is a boundary regime cannot start the fixed point.
    let mut p = l2(0.0);
    p.workers = 0.05;
    assert!(matches!(
        economy_1c(p.clone()).solve(),
        Ok(Regime::BoundaryNoMargin { .. })
    ));
    let economy = PlantEconomy::new(p.clone(), vec![(0, labour_land_plant())]).unwrap();
    assert_eq!(
        economy.solve(),
        Err(PlantError::NotInterior {
            step: 0,
            regime: "BoundaryNoMargin"
        })
    );
    // A bundle plant needs no fixed point: its long run's regime is returned.
    let economy = PlantEconomy::new(p, vec![(0, bundle_plant(None))]).unwrap();
    assert!(matches!(
        economy.solve(),
        Ok(Regime::BoundaryNoMargin { .. })
    ));
}

#[test]
fn the_cap_refuses() {
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, labour_land_plant()), (1, labour_land_plant())],
    )
    .unwrap();
    let gap = |steps: u32| match economy.solve_within(steps) {
        Err(PlantError::NoFixedPoint { steps: s, gap }) => {
            assert_eq!(s, steps);
            assert!(gap > PLANT_TOL);
            gap
        }
        other => panic!("{other:?}"),
    };
    // Each step moves ln(kappa/zeta) half way (docs/unit-1g.md §2.6), and the map is nearly flat
    // here, so the move halves from one step to the next.
    for steps in [3, 10, 20] {
        let ratio = gap(steps + 1) / gap(steps);
        assert!((0.45..0.55).contains(&ratio), "{steps}: {ratio}");
    }
    for e in [
        PlantError::NoFixedPoint { steps: 3, gap: 0.1 },
        PlantError::NotInterior {
            step: 0,
            regime: "NotViable",
        },
        PlantError::Param(ParamError::BuildLag { value: 0 }),
    ] {
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn validation() {
    let rejected =
        |plants: Vec<(usize, Plant)>, name: &str| match PlantEconomy::new(l2(0.0), plants) {
            Err(ParamError::Item {
                kind: "plant",
                index,
                error,
            }) => {
                assert_eq!(error.name(), name, "{error}");
                index
            }
            other => panic!("{name}: {other:?}"),
        };
    let ok = labour_land_plant();
    assert_eq!(rejected(vec![(2, ok.clone())], "machine_type"), 0);
    assert_eq!(
        rejected(vec![(0, ok.clone()), (0, ok.clone())], "machine_type"),
        1
    );
    let with = |f: &dyn Fn(&mut Plant)| {
        let mut p = ok.clone();
        f(&mut p);
        vec![(0, p)]
    };
    rejected(with(&|p| p.bundle_share = 0.0), "bundle_share");
    rejected(with(&|p| p.bundle_share = 1.5), "bundle_share");
    rejected(with(&|p| p.delta = 0.0), "delta");
    rejected(with(&|p| p.build_lag = 0), "build_lag");
    rejected(
        with(&|p| p.recipe = PlantRecipe::Bundle { size: 0.0 }),
        "size",
    );
    rejected(
        with(&|p| p.recipe = PlantRecipe::Fixed(recipe(&[0.0], 1.0, 1.0))),
        "recipe.machines",
    );
    rejected(
        with(&|p| p.recipe = PlantRecipe::Fixed(recipe(&[-1.0, 0.0], 1.0, 1.0))),
        "recipe.machines",
    );
    rejected(
        with(&|p| p.recipe = PlantRecipe::Fixed(recipe(&[0.0, 0.0], f64::NAN, 1.0))),
        "recipe.labor",
    );
    rejected(
        with(&|p| p.recipe = PlantRecipe::Fixed(recipe(&[0.0, 0.0], 1.0, 1e-40))),
        "recipe.land",
    );
    rejected(
        with(&|p| p.recipe = PlantRecipe::Fixed(Recipe::zero(2))),
        "recipe",
    );
    let mut huge = l2(0.0);
    huge.rho = 1.0;
    match PlantEconomy::new(huge, with(&|p| p.build_lag = 5000)) {
        Err(ParamError::Item { error, .. }) => assert_eq!(error.name(), "u"),
        other => panic!("{other:?}"),
    }
    // A type with a build recipe holds a stock already.
    let mut p = l2(0.0);
    p.machine_types[0].build = recipe(&[0.0, 0.0], 0.1, 0.1);
    p.machine_types[0].delta = 0.1;
    match PlantEconomy::new(p, vec![(0, ok.clone())]) {
        Err(ParamError::Item { error, .. }) => assert_eq!(error.name(), "machine_type"),
        other => panic!("{other:?}"),
    }
    // The unplanted economy must be valid.
    let mut p = l2(0.0);
    p.workers = -1.0;
    assert_eq!(
        PlantEconomy::new(p, vec![(0, ok.clone())])
            .unwrap_err()
            .name(),
        "workers"
    );
    // −0.0 is stored as +0.0, and a bundle's size is kept.
    let mut z = ok.clone();
    z.recipe = PlantRecipe::Fixed(recipe(&[-0.0, 0.0], 0.5, 0.5));
    let economy = PlantEconomy::new(l2(0.0), vec![(1, z)]).unwrap();
    match &economy.plants()[0].1.recipe {
        PlantRecipe::Fixed(r) => assert!(r.machines[0].is_sign_positive()),
        other => panic!("{other:?}"),
    }
    assert_eq!(economy.base(), &l2(0.0));
}

#[test]
fn the_fixed_point_starts_at_the_unplanted_prices() {
    // docs/unit-1g.md §5.3: the unplanted economy's interior equilibrium gives the first ratios.
    // Step 1 replayed here from them, as `solve_within` takes it (ln r, back by exp, the long
    // run solved, the ratios its prices give): the gap `solve_within(1)` reports is this one,
    // bit for bit.
    let economy = PlantEconomy::new(
        l2(0.0),
        vec![(0, labour_land_plant()), (1, labour_land_plant())],
    )
    .unwrap();
    let step_one_gap = |logs: &[f64]| {
        let ratios: Vec<f64> = logs.iter().map(|&s| num::exp(s)).collect();
        let eq = interior_1c(economy.long_run(&ratios));
        economy
            .ratios_at(&eq)
            .iter()
            .zip(logs)
            .fold(0.0_f64, |g, (r, s)| g.max((num::ln(*r) - s).abs()))
    };
    let base = interior_1c(l2(0.0));
    let logs: Vec<f64> = economy.ratios_at(&base).into_iter().map(num::ln).collect();
    let gap = step_one_gap(&logs);
    match economy.solve_within(1) {
        Err(PlantError::NoFixedPoint { steps: 1, gap: g }) => {
            assert_eq!(g.to_bits(), gap.to_bits(), "{g:e} against {gap:e}")
        }
        other => panic!("{other:?}"),
    }
    // Started from ratios of 1 instead, step 1 would move by another amount.
    let from_one = step_one_gap(&[0.0, 0.0]);
    assert!((from_one - gap).abs() > 1e-3 * gap, "{from_one:e} {gap:e}");
}

#[test]
fn a_plant_with_a_build_lag() {
    // P1R's plants with J 2 (CAPACITY registers M3's timing, J 2): ρ 5% a year a week, so
    // u = (ρ + δ)(1 + ρ) and the plant's wealth factor ω = (1 + ρ) + δ. The long run takes the
    // plant's J, and at it O = θp and uV = (1 − θ)p (§4.5); with J 1 in the long run and J 2 in
    // the ratio, uV would be 0.19985 of p.
    let rho = rho_per_week(0.05);
    let d = per_week(0.1);
    let lagged = || Plant {
        build_lag: 2,
        ..bundle_plant(None)
    };
    let economy = PlantEconomy::new(l2(rho), vec![(0, lagged()), (1, lagged())]).unwrap();
    let eq = long_run(&economy);
    check_plants(&economy, &eq);
    let p = economy.long_run(&eq.ratios);
    let u = (rho + d) * (1.0 + rho);
    let omega = (1.0 + rho) + d;
    for k in 0..2 {
        let (planted, t) = (&p.machine_types[k], &eq.eq.types[k]);
        assert_eq!((planted.delta, planted.build_lag), (d, 2));
        close_to("u at J 2", t.user_cost, u, 1e-15);
        close_to(
            "the ratio at J 2",
            eq.ratios[k],
            (1.0 - PLANT_THETA) / ((PLANT_THETA * u) * P1_S1),
            1e-14,
        );
        close_to(
            "W = omega V X",
            t.wealth,
            omega * t.build_cost * t.services,
            1e-14,
        );
        close_to("the capital share", eq.plants[k].capital_share, 0.2, 1e-13);
    }
    // The lag moves the long run off P1R's.
    assert!((eq.eq.x_star - P1R_X_STAR).abs() > 1e-6);
    assert!((eq.eq.v - P1R_V).abs() > 1e-4);
}
