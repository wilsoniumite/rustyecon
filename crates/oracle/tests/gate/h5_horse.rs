//! h5: CHAIN's horse at weekly periods (docs/unit-1g.md §3.3 and §8; CHAIN §1.3-1.4): fodder
//! raised with horse-days, a head reared from fodder, pasture and labour, 250 horse-days a year;
//! δ 8% a year and J three years in ticks. Its build embodies about 3.3 horse-days per unit of
//! weekly capacity, which unit 1c's rule refused.

use oracle::{MachineEconomy, Row};

use crate::goldens_1g::*;
use crate::support::*;
use crate::support_1g::*;

#[test]
fn horse_goldens() {
    let (e, eq) = solved_chain(horse());
    aggregates(
        "HORSE",
        &eq.eq,
        [
            HORSE_X_STAR,
            HORSE_ONE_MINUS_X_STAR,
            HORSE_V,
            HORSE_P_S,
            HORSE_Y,
            HORSE_N_A,
            HORSE_INCOME,
        ],
    );
    assert_eq!(eq.eq.interest, 0.0);
    good_is(
        "HORSE",
        &eq,
        "FODDER",
        HORSE_FODDER_PRICE,
        HORSE_FODDER_OUTPUT,
    );
    good_is("HORSE", &eq, "HORSE", HORSE_HORSE_PRICE, HORSE_HORSE_OUTPUT);
    good_is(
        "HORSE",
        &eq,
        "HORSE_DAYS",
        HORSE_HORSE_DAYS_PRICE,
        HORSE_HORSE_DAYS_HOURS,
    );
    let h = eq.machine("HORSE").unwrap();
    close("HORSE: heads", h.stock, HORSE_HORSE_STOCK);
    close("HORSE: O", h.operating_cost, HORSE_HORSE_DAYS_OPERATING);
    close("HORSE: V", h.build_cost, HORSE_HORSE_DAYS_BUILD);
    // Its totals a horse-day (CHAIN's 0.268 pd and 0.021 acre-yr, with its own rents) and the
    // wealth factor 1 + δ(J − 1) at ρ 0.
    let Some(Row::Hours(k)) = e.row("HORSE_DAYS") else {
        panic!("no hours")
    };
    let t = &eq.eq.types[k];
    close("HORSE: lambda-tilde", t.lambda_tilde, HORSE_LAMBDA_TILDE);
    close("HORSE: b-tilde", t.b_tilde, HORSE_B_TILDE);
    assert_eq!(t.lambda_tilde, t.lambda_tilde_q);
    close("HORSE: omega", t.wealth_factor, HORSE_OMEGA);
    close(
        "HORSE: omega = 1 + delta (J - 1)",
        t.wealth_factor,
        1.0 + per_week(0.08) * 155.0,
    );
    assert_eq!(e.chain().machines[0].build_lag, 156);
}

#[test]
fn the_horse_needs_d_g10() {
    // Per head 8 t of fodder at 2 horse-days a ton, over 250/52 horse-days a week: 3.3
    // horse-days per unit of weekly capacity in its build.
    let p = horse().to_machine_params().unwrap();
    let embodied = p.machine_types[2].build.machines[1] * 8.0 * 2.0;
    close(
        "horse-days in a unit of capacity",
        embodied,
        52.0 / 250.0 * 16.0,
    );
    assert!(embodied > 1.0);
    assert!(!accepted_by_1c(&p.machine_types));
    assert!(MachineEconomy::new(p).is_ok());
}
