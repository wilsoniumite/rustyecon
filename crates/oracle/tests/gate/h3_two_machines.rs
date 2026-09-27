//! h3: two machines, horse and engine (docs/unit-1g.md §3.3 S2 and S2H, and §8): the technique
//! envelope over the hours types, the switch in unit 1c's closed form, and the unused machine
//! priced.

use oracle::{Regime, Row};

use crate::goldens_1g::*;
use crate::support::*;
use crate::support_1g::*;

/// The type indices of S2's chain: coal, iron, fodder, the horse good, the engine good, horse
/// hours, engine-hours.
const HORSE_HOURS: usize = 5;
const ENGINE_HOURS: usize = 6;

#[test]
fn the_engine_at_the_margin() {
    let (e, eq) = solved_chain(s2(false, 0.05));
    assert_eq!(e.row("HORSE_HOURS"), Some(Row::Hours(HORSE_HOURS)));
    assert_eq!(e.row("ENGINE_HOURS"), Some(Row::Hours(ENGINE_HOURS)));
    // Only the hours do tasks; the envelope goes horse → engine, at 1c's closed form.
    let env = e.economy().envelope();
    assert_eq!(env.first, HORSE_HOURS);
    assert_eq!(env.switches.len(), 1);
    let s = env.switches[0];
    assert_eq!((s.below, s.above), (HORSE_HOURS, ENGINE_HOURS));
    close("S2: the switch's gamma", s.gamma, S2_SWITCH_GAMMA);
    close_to(
        "S2: the switch's x",
        eq.eq.switches[0].x,
        S2_SWITCH_X,
        1e-12,
    );
    assert_eq!(eq.eq.technique, ENGINE_HOURS);
    assert!(eq.eq.tie.is_none());
    aggregates(
        "S2",
        &eq.eq,
        [
            S2_X_STAR,
            S2_ONE_MINUS_X_STAR,
            S2_V,
            S2_P_S,
            S2_Y,
            S2_N_A,
            S2_INCOME,
        ],
    );
    close("S2: interest", eq.eq.interest, S2_INTEREST);
    good_is("S2", &eq, "COAL", S2_COAL_PRICE, S2_COAL_OUTPUT);
    good_is("S2", &eq, "IRON", S2_IRON_PRICE, S2_IRON_OUTPUT);
    good_is("S2", &eq, "FODDER", S2_FODDER_PRICE, S2_FODDER_OUTPUT);
    good_is("S2", &eq, "HORSE", S2_HORSE_PRICE, S2_HORSE_OUTPUT);
    good_is("S2", &eq, "ENGINE", S2_ENGINE_PRICE, S2_ENGINE_OUTPUT);
    good_is(
        "S2",
        &eq,
        "HORSE_HOURS",
        S2_HORSE_HOURS_PRICE,
        S2_HORSE_HOURS_HOURS,
    );
    good_is(
        "S2",
        &eq,
        "ENGINE_HOURS",
        S2_ENGINE_HOURS_PRICE,
        S2_ENGINE_HOURS_HOURS,
    );
    let (h, g) = (eq.machine("HORSE").unwrap(), eq.machine("ENGINE").unwrap());
    assert_eq!((h.stock, h.made, h.hours), (S2_HORSE_STOCK, 0.0, 0.0));
    close("S2: engines", g.stock, S2_ENGINE_STOCK);
    close("S2: horse O", h.operating_cost, S2_HORSE_HOURS_OPERATING);
    close("S2: horse V", h.build_cost, S2_HORSE_HOURS_BUILD);
    close("S2: engine O", g.operating_cost, S2_ENGINE_HOURS_OPERATING);
    close("S2: engine V", g.build_cost, S2_ENGINE_HOURS_BUILD);
    // The unused horse is priced, its closure wage above v (SSRN A.1's unused methods).
    let t = &eq.eq.types[HORSE_HOURS];
    assert!(t.closure_wage.unwrap() > eq.eq.v);
    // Materials and goods do no tasks.
    for k in 0..5 {
        assert_eq!(eq.eq.types[k].task_services, 0.0);
        assert!(eq.eq.types[k].closure_wage.is_none());
    }
}

#[test]
fn the_horse_at_the_margin() {
    let (e, eq) = solved_chain(s2(true, 0.05));
    let env = e.economy().envelope();
    close(
        "S2H: the switch's gamma",
        env.switches[0].gamma,
        S2H_SWITCH_GAMMA,
    );
    close_to(
        "S2H: the switch's x",
        eq.eq.switches[0].x,
        S2H_SWITCH_X,
        1e-12,
    );
    assert_eq!(eq.eq.technique, HORSE_HOURS);
    assert!(eq.eq.x_star < eq.eq.switches[0].x);
    aggregates(
        "S2H",
        &eq.eq,
        [
            S2H_X_STAR,
            S2H_ONE_MINUS_X_STAR,
            S2H_V,
            S2H_P_S,
            S2H_Y,
            S2H_N_A,
            S2H_INCOME,
        ],
    );
    close("S2H: interest", eq.eq.interest, S2H_INTEREST);
    good_is("S2H", &eq, "COAL", S2H_COAL_PRICE, S2H_COAL_OUTPUT);
    good_is("S2H", &eq, "IRON", S2H_IRON_PRICE, S2H_IRON_OUTPUT);
    good_is("S2H", &eq, "FODDER", S2H_FODDER_PRICE, S2H_FODDER_OUTPUT);
    good_is("S2H", &eq, "HORSE", S2H_HORSE_PRICE, S2H_HORSE_OUTPUT);
    good_is("S2H", &eq, "ENGINE", S2H_ENGINE_PRICE, S2H_ENGINE_OUTPUT);
    good_is(
        "S2H",
        &eq,
        "HORSE_HOURS",
        S2H_HORSE_HOURS_PRICE,
        S2H_HORSE_HOURS_HOURS,
    );
    good_is(
        "S2H",
        &eq,
        "ENGINE_HOURS",
        S2H_ENGINE_HOURS_PRICE,
        S2H_ENGINE_HOURS_HOURS,
    );
    let (h, g) = (eq.machine("HORSE").unwrap(), eq.machine("ENGINE").unwrap());
    close("S2H: horses", h.stock, S2H_HORSE_STOCK);
    close("S2H: horse O", h.operating_cost, S2H_HORSE_HOURS_OPERATING);
    close("S2H: horse V", h.build_cost, S2H_HORSE_HOURS_BUILD);
    close(
        "S2H: engine O",
        g.operating_cost,
        S2H_ENGINE_HOURS_OPERATING,
    );
    close("S2H: engine V", g.build_cost, S2H_ENGINE_HOURS_BUILD);
    assert_eq!((g.stock, g.hours), (S2H_ENGINE_STOCK, 0.0));
}

#[test]
fn at_rho_zero_the_horse_is_used_everywhere() {
    // The horse carries more capital per hour than the engine; without interest the switch
    // leaves the line (ORACLE-GOODS §1.6).
    for horse_first in [false, true] {
        let e = chain_economy(s2(horse_first, 0.0));
        let env = e.economy().envelope();
        assert_eq!(env.first, HORSE_HOURS);
        assert!(env.switches.is_empty(), "{horse_first}");
        match e.solve() {
            Ok(Regime::Interior(eq)) => assert_eq!(eq.eq.technique, HORSE_HOURS),
            other => panic!("{other:?}"),
        }
    }
}
