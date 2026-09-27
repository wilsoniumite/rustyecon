//! h2: the mapping from a chain of goods to unit 1c (docs/unit-1g.md §2.2-2.5, §4.2-4.4 and §8),
//! on S1, the steam chain calibrated to 1c's M3, and S1Z at ρ 0.

use oracle::{
    Category, ChainCategory, ChainEconomy, ChainError, MachineEconomy, ParamError, Recipe, Regime,
    Row,
};

use crate::goldens_1c::*;
use crate::goldens_1g::*;
use crate::support::*;
use crate::support_1c::*;
use crate::support_1g::*;

#[test]
fn s1_maps_to_its_embedding() {
    let chain = s1(0.05);
    let p = chain.to_machine_params().unwrap();
    // The types: coal and iron (materials), the engine good, engine-hours.
    assert_eq!(p.machine_types.len(), 4);
    let [lab_op, lab_i, seams, site] = S1_SOLVED.map(|(n, d)| n / d);
    // The four coefficients solved as fractions so that S1's totals are M3's.
    for (got, want) in [lab_op, lab_i, seams, site]
        .into_iter()
        .zip([S1_LAB_OP, S1_LAB_I, S1_SEAMS, S1_SITE])
    {
        close("S1's solved coefficient", got, want);
    }
    let flow = |t: &oracle::MachineType| {
        t.task_efficiency == 0.0 && t.build == Recipe::zero(4) && t.delta == 1.0 && t.build_lag == 1
    };
    let (coal, iron, good, hours) = (
        &p.machine_types[0],
        &p.machine_types[1],
        &p.machine_types[2],
        &p.machine_types[3],
    );
    assert!(flow(coal) && flow(iron) && flow(good));
    assert_eq!(coal.operating, recipe(&[0.0, 0.0, 0.0, 0.1], 0.2, seams));
    assert_eq!(iron.operating, recipe(&[0.5, 0.0, 0.0, 0.0], 0.5, 0.1));
    assert_eq!(good.operating, recipe(&[0.0, 0.1, 0.0, 0.0], lab_i, site));
    assert_eq!(hours.task_efficiency, 1.0);
    assert_eq!(hours.operating, recipe(&[0.4, 0.0, 0.0, 0.0], lab_op, 0.0));
    assert_eq!(hours.build, recipe(&[0.0, 0.0, 1.0, 0.0], 0.0, 0.0));
    assert_eq!((hours.delta, hours.build_lag), (0.1, 3));
    assert_eq!(p.intermediate, no_inputs(2));
    assert_eq!((p.workers, p.land, p.rho), (4.0, 10.0, 0.05));
    let e = chain_economy(chain);
    assert_eq!(e.row("GOOD"), Some(Row::Category(0)));
    assert_eq!(e.row("SPACE"), Some(Row::Category(1)));
    assert_eq!(e.row("COAL"), Some(Row::Material(0)));
    assert_eq!(e.row("IRON"), Some(Row::Material(1)));
    assert_eq!(e.row("ENGINE"), Some(Row::MachineGood(2)));
    assert_eq!(e.row("ENGINE_HOURS"), Some(Row::Hours(3)));
    assert_eq!(e.row("STEAM"), None);
    assert_eq!(e.rows().len(), 6);
    assert_eq!(e.economy().params(), &p);
}

#[test]
fn s1_is_m3() {
    // S1's four totals per engine-hour are M3's, so its equilibrium is M3's (ORACLE-GOODS §1.6).
    let (e, eq) = solved_chain(s1(0.05));
    aggregates(
        "S1 against M3",
        &eq.eq,
        [
            M3_X_STAR,
            M3_ONE_MINUS_X_STAR,
            M3_V,
            M3_P_S,
            M3_Y,
            M3_N_A,
            M3_INCOME,
        ],
    );
    close("S1: interest", eq.eq.interest, M3_INTEREST);
    aggregates(
        "S1",
        &eq.eq,
        [
            S1_X_STAR,
            S1_ONE_MINUS_X_STAR,
            S1_V,
            S1_P_S,
            S1_Y,
            S1_N_A,
            S1_INCOME,
        ],
    );
    close("S1: interest", eq.eq.interest, S1_INTEREST);
    // The engine-hour is M3's machine on the price side and in V·X, not in V or X.
    let t = &eq.eq.types[3];
    close("S1: engine-hour price", t.price, M3_P_M);
    close("S1: V X", t.build_cost * t.services, M3_BUILD * M3_SERVICES);
    close("S1: lambda-tilde", t.lambda_tilde, M3_LAMBDA_TILDE);
    close("S1: b-tilde^q", t.b_tilde_q, M3_B_TILDE_Q);
    assert!((t.build_cost - M3_BUILD).abs() > 1.0);
    // Every good.
    good_is("S1", &eq, "COAL", S1_COAL_PRICE, S1_COAL_OUTPUT);
    good_is("S1", &eq, "IRON", S1_IRON_PRICE, S1_IRON_OUTPUT);
    good_is("S1", &eq, "ENGINE", S1_ENGINE_PRICE, S1_ENGINE_OUTPUT);
    good_is(
        "S1",
        &eq,
        "ENGINE_HOURS",
        S1_ENGINE_HOURS_PRICE,
        S1_ENGINE_HOURS_HOURS,
    );
    let m = eq.machine("ENGINE").unwrap();
    close("S1: stock", m.stock, S1_ENGINE_STOCK);
    close("S1: O", m.operating_cost, S1_ENGINE_HOURS_OPERATING);
    close("S1: V", m.build_cost, S1_ENGINE_HOURS_BUILD);
    assert_eq!(e.chain().machines[0].hours_per_period, 1.0);
    // S1Z: at ρ 0 the five numbers are M3z's.
    let (_, z) = solved_chain(s1(0.0));
    aggregates(
        "S1Z against M3z",
        &z.eq,
        [
            M3Z_X_STAR,
            M3Z_ONE_MINUS_X_STAR,
            M3Z_V,
            M3Z_P_S,
            M3Z_Y,
            M3Z_N_A,
            M3Z_INCOME,
        ],
    );
    assert_eq!(z.eq.interest, 0.0);
    good_is("S1Z", &z, "COAL", S1Z_COAL_PRICE, S1Z_COAL_OUTPUT);
    good_is("S1Z", &z, "IRON", S1Z_IRON_PRICE, S1Z_IRON_OUTPUT);
    good_is("S1Z", &z, "ENGINE", S1Z_ENGINE_PRICE, S1Z_ENGINE_OUTPUT);
    good_is(
        "S1Z",
        &z,
        "ENGINE_HOURS",
        S1Z_ENGINE_HOURS_PRICE,
        S1Z_ENGINE_HOURS_HOURS,
    );
    close("S1Z: M3z's p_m", z.eq.types[3].price, M3Z_P_M);
    let m = z.machine("ENGINE").unwrap();
    close("S1Z: stock", m.stock, S1Z_ENGINE_STOCK);
    close("S1Z: O", m.operating_cost, S1Z_ENGINE_HOURS_OPERATING);
    close("S1Z: V", m.build_cost, S1Z_ENGINE_HOURS_BUILD);
}

#[test]
fn readouts() {
    // docs/unit-1g.md §4.4 on S1 and on the horse (κ = 250/52).
    for chain in [s1(0.05), horse()] {
        let (e, eq) = solved_chain(chain);
        for (i, m) in e.chain().machines.iter().enumerate() {
            let r = &eq.machines[i];
            let kappa = m.hours_per_period;
            let good = eq.good(&m.key).unwrap();
            let hours = eq.good(&m.hours).unwrap();
            assert_eq!(r.key, m.key);
            assert_eq!(r.hours_key, m.hours);
            assert_eq!(r.price, good.price);
            assert_eq!(r.made, good.output);
            assert_eq!(r.hour_price, hours.price);
            assert_eq!(r.hours, hours.output);
            close("the good's price is kappa V", r.price, kappa * r.build_cost);
            close(
                "goods made are delta X/kappa",
                r.made,
                m.delta * r.hours / kappa,
            );
            close("the stock is X/kappa", r.stock, r.hours / kappa);
            close(
                "p = O + uV",
                r.hour_price,
                r.operating_cost + r.user_cost * r.build_cost,
            );
            let t = match e.row(&m.hours) {
                Some(Row::Hours(k)) => &eq.eq.types[k],
                other => panic!("{other:?}"),
            };
            assert_eq!(
                (r.wealth, r.interest, r.user_cost),
                (t.wealth, t.interest, t.user_cost)
            );
        }
        // Interest is Σ (u − δ)·V·X over the machines' hours.
        let mut interest = 0.0;
        for (m, r) in e.chain().machines.iter().zip(&eq.machines) {
            interest += (r.user_cost - m.delta) * r.build_cost * r.hours;
        }
        if eq.eq.interest == 0.0 {
            assert_eq!(interest, 0.0);
        } else {
            close("interest", interest, eq.eq.interest);
        }
        // Categories read their price and gross output.
        let good = eq.good("GOOD").unwrap();
        assert_eq!(good.price, eq.eq.categories[0].price);
        assert_eq!(good.output, eq.eq.categories[0].gross_output);
    }
    // The fold (docs/unit-1g.md §4.3): S1 folded to one type, from the goldens, has S1's
    // equilibrium, the five numbers being the same.
    let fold = machine_type(
        1.0,
        recipe(&[S1_FOLD_A], S1_FOLD_LAMBDA, S1_FOLD_B),
        recipe(&[S1_FOLD_A_I], S1_FOLD_LAMBDA_I, S1_FOLD_B_I),
        0.1,
        3,
    );
    let q = interior_1c(appendix_b_household(
        linear(1.0, 1.0, 2.0),
        0.05,
        1.0,
        vec![fold],
    ));
    aggregates(
        "S1's fold",
        &q,
        [
            S1_X_STAR,
            S1_ONE_MINUS_X_STAR,
            S1_V,
            S1_P_S,
            S1_Y,
            S1_N_A,
            S1_INCOME,
        ],
    );
    close(
        "S1's fold: hour price",
        q.types[0].price,
        S1_ENGINE_HOURS_PRICE,
    );
    close("S1's fold: V", q.types[0].build_cost, S1_ENGINE_HOURS_BUILD);
    close("S1's fold: X", q.types[0].services, S1_ENGINE_HOURS_HOURS);
}

#[test]
fn e1_and_e2_are_enforced() {
    // E1 (decision 67 reworded, D-G1): no machine-side recipe uses a category.
    let mut c = s1(0.05);
    c.materials[1].recipe.inputs.push(("GOOD".into(), 0.1));
    assert_eq!(
        c.to_machine_params(),
        Err(ChainError::CategoryInMachineRecipe {
            good: "IRON".into(),
            input: "GOOD".into()
        })
    );
    let mut c = s1(0.05);
    c.machines[0].build.inputs.push(("SPACE".into(), 0.1));
    assert_eq!(
        c.to_machine_params(),
        Err(ChainError::CategoryInMachineRecipe {
            good: "ENGINE".into(),
            input: "SPACE".into()
        })
    );
    let mut c = s1(0.05);
    c.machines[0].operating.inputs.push(("GOOD".into(), 0.1));
    assert_eq!(
        c.to_machine_params(),
        Err(ChainError::CategoryInMachineRecipe {
            good: "ENGINE_HOURS".into(),
            input: "GOOD".into()
        })
    );
    // E2: a category uses materials, goods and hours only through its tasks.
    for input in ["COAL", "ENGINE", "ENGINE_HOURS"] {
        let mut c = s1(0.05);
        c.categories[1].inputs.push((input.into(), 0.3));
        assert_eq!(
            c.to_machine_params(),
            Err(ChainError::DirectUse {
                category: "SPACE".into(),
                input: input.into()
            })
        );
    }
    // A category's inputs are unit 1c's intermediate inputs (row = the user).
    let mut c = s1(0.05);
    c.categories[1].inputs.push(("GOOD".into(), 0.25));
    let p = c.to_machine_params().unwrap();
    assert_eq!(p.intermediate, vec![vec![0.0, 0.0], vec![0.25, 0.0]]);
    assert!(ChainEconomy::new(c).is_ok());
}

#[test]
fn keys_and_errors() {
    let mut c = s1(0.05);
    c.machines[0].hours = "COAL".into();
    assert_eq!(
        c.to_machine_params(),
        Err(ChainError::DuplicateKey { key: "COAL".into() })
    );
    let mut c = s1(0.05);
    c.categories.push(ChainCategory {
        key: "IRON".into(),
        category: Category {
            weight: 0.0,
            direct_land: 1.0,
            density: vec![0.0],
        },
        inputs: vec![],
    });
    assert_eq!(
        c.rows(),
        Err(ChainError::DuplicateKey { key: "IRON".into() })
    );
    let mut c = s1(0.05);
    c.materials[0].recipe.inputs.push(("STEAM".into(), 1.0));
    assert_eq!(
        c.to_machine_params(),
        Err(ChainError::UnknownGood {
            good: "COAL".into(),
            input: "STEAM".into()
        })
    );
    let mut c = s1(0.05);
    c.categories[0].inputs.push(("STEAM".into(), 1.0));
    assert!(matches!(
        c.to_machine_params(),
        Err(ChainError::UnknownGood { good, input }) if good == "GOOD" && input == "STEAM"
    ));
    let mut c = s1(0.05);
    c.materials[1].recipe.inputs.push(("COAL".into(), 0.1));
    assert_eq!(
        c.to_machine_params(),
        Err(ChainError::RepeatedInput {
            good: "IRON".into(),
            input: "COAL".into()
        })
    );
    let mut c = s1(0.05);
    c.categories[1].inputs = vec![("GOOD".into(), 0.1), ("GOOD".into(), 0.1)];
    assert!(matches!(
        c.to_machine_params(),
        Err(ChainError::RepeatedInput { good, .. }) if good == "SPACE"
    ));
    // κ is a scale.
    for kappa in [0.0, -1.0, f64::NAN, 1e31] {
        let mut c = s1(0.05);
        c.machines[0].hours_per_period = kappa;
        match c.to_machine_params() {
            Err(ChainError::Param {
                good: Some(good),
                error,
            }) => {
                assert_eq!(good, "ENGINE");
                assert_eq!(error.name(), "hours_per_period");
            }
            other => panic!("{kappa}: {other:?}"),
        }
    }
    // Unit 1c's errors named by good: a material's, a machine good's, an hours type's, a
    // category's, and the economy's own.
    let named = |c: oracle::GoodsChain| match ChainEconomy::new(c) {
        Err(ChainError::Param { good, error }) => (good, error.name()),
        other => panic!("{other:?}"),
    };
    let mut c = s1(0.05);
    c.materials[1].recipe.labor = -0.5;
    assert_eq!(named(c), (Some("IRON".into()), "operating.labor"));
    let mut c = s1(0.05);
    c.machines[0].build.land = f64::NAN;
    assert_eq!(named(c), (Some("ENGINE".into()), "operating.land"));
    let mut c = s1(0.05);
    c.machines[0].delta = 1.5;
    assert_eq!(named(c), (Some("ENGINE_HOURS".into()), "delta"));
    let mut c = s1(0.05);
    c.categories[0].category.density = vec![-1.0];
    assert_eq!(named(c).0, Some("GOOD".into()));
    let mut c = s1(0.05);
    c.workers = 0.0;
    assert_eq!(named(c), (None, "workers"));
    // Not productive per period: coal pumped by 12 engine-hours a ton, each burning 0.4 t.
    let mut c = s1(0.05);
    c.materials[0].recipe.inputs[0].1 = 12.0;
    match ChainEconomy::new(c) {
        Err(ChainError::Param { good: None, error }) => {
            assert!(matches!(
                error,
                ParamError::Invalid {
                    name: "machine types",
                    ..
                }
            ));
            assert!(error.to_string().contains("not productive"));
        }
        other => panic!("{other:?}"),
    }
    // No task type.
    let mut c = s1(0.05);
    c.machines[0].task_efficiency = 0.0;
    assert!(ChainEconomy::new(c).is_err());
    for e in [
        ChainError::UnknownGood {
            good: "a".into(),
            input: "b".into(),
        },
        ChainError::RepeatedInput {
            good: "a".into(),
            input: "b".into(),
        },
        ChainError::CategoryInMachineRecipe {
            good: "a".into(),
            input: "b".into(),
        },
        ChainError::Param {
            good: None,
            error: ParamError::BuildLag { value: 0 },
        },
    ] {
        assert!(!e.to_string().is_empty());
    }
}

#[test]
fn units_of_stock() {
    // The horse per head (κ 250/52 a week) and per unit of capacity (κ 1, the build divided by
    // κ): the same economy, with the good's price and output κ and 1/κ times (§2.5).
    let head = horse();
    let mut capacity = horse();
    let kappa = 250.0 / 52.0;
    let m = &mut capacity.machines[0];
    m.hours_per_period = 1.0;
    for (_, units) in &mut m.build.inputs {
        *units /= kappa;
    }
    m.build.labor /= kappa;
    m.build.land /= kappa;
    let (_, a) = solved_chain(head);
    let (_, b) = solved_chain(capacity);
    close_to("x*", a.eq.x_star, b.eq.x_star, 1e-13);
    close_to("v", a.eq.v, b.eq.v, 1e-13);
    close_to("Y", a.eq.y, b.eq.y, 1e-13);
    close_to("N_a", a.eq.n_a, b.eq.n_a, 1e-13);
    let (ha, hb) = (a.machine("HORSE").unwrap(), b.machine("HORSE").unwrap());
    close_to("price", ha.price, kappa * hb.price, 1e-13);
    close_to("made", ha.made, hb.made / kappa, 1e-13);
    close_to("stock", ha.stock, hb.stock / kappa, 1e-13);
    close_to("hour price", ha.hour_price, hb.hour_price, 1e-13);
    close_to("V", ha.build_cost, hb.build_cost, 1e-13);
    // The mapped types differ only in the good's row and the hours' build.
    let (pa, pb) = (horse().to_machine_params().unwrap(), {
        let mut c = horse();
        c.machines[0].hours_per_period = 1.0;
        c.to_machine_params().unwrap()
    });
    assert_eq!(pa.machine_types[2].build.machines[1], 1.0 / kappa);
    assert_eq!(pb.machine_types[2].build.machines[1], 1.0);
    assert!(MachineEconomy::new(pb).is_ok());
    // A boundary regime is 1c's.
    let mut c = s1(0.05);
    c.workers = 0.5;
    let e = chain_economy(c);
    assert_eq!(
        format!("{:?}", e.solve().map(|r| r.name())),
        format!("{:?}", e.economy().solve().map(|r| r.name()))
    );
    assert!(!matches!(e.solve(), Ok(Regime::Interior(_))));
}
