//! h4: A0, SSRN Appendix B's flow machine as a durable good at 52 ticks a year
//! (docs/unit-1g.md §3.3 and §8; AGENTS-GOODS §1). Its build embodies a/δ = 148 of its own hours,
//! which unit 1c's rule refused; per period it is productive, and at ρ = 0 its equilibrium is
//! Appendix B's.

use oracle::{MachineParams, UniformWorkCost};

use crate::goldens::*;
use crate::goldens_1b::*;
use crate::goldens_1g::*;
use crate::support::*;
use crate::support_1b::*;
use crate::support_1c::*;
use crate::support_1g::*;

#[test]
fn a0_is_appendix_b() {
    let (_, chain) = solved_chain(a0(0.0));
    let one = interior_1c(a0_one_type(0.0));
    for (what, eq) in [("A0's chain", &*chain.eq), ("A0 as one type", &*one)] {
        close(&format!("{what}: x*"), eq.x_star, G1_X_STAR);
        close(&format!("{what}: v"), eq.v, G1_V);
        close(&format!("{what}: P_s"), eq.p_s, G1_P_S);
        close(&format!("{what}: Y"), eq.y, G1_Y);
        close(&format!("{what}: N_a"), eq.n_a, G1_N_A);
        close(&format!("{what}: income"), eq.income, G1_INCOME);
        close(
            &format!("{what}: final hours"),
            eq.final_hours,
            G1_FINAL_HOURS,
        );
        close(
            &format!("{what}: machine hours"),
            eq.machine_hours,
            G1_MACHINE_HOURS,
        );
        assert_eq!(eq.interest, 0.0, "{what}");
    }
    // The horse-day is Appendix B's machine service: its price p_m and its quantity K.
    good_is("A0", &chain, "HORSE_DAYS", G1_P_M, G1_K);
    good_is(
        "A0",
        &chain,
        "HORSE_DAYS",
        A0_HORSE_DAYS_PRICE,
        A0_HORSE_DAYS_HOURS,
    );
    good_is("A0", &chain, "FODDER", A0_FODDER_PRICE, A0_FODDER_OUTPUT);
    good_is("A0", &chain, "HORSE", A0_HORSE_PRICE, A0_HORSE_OUTPUT);
    let h = chain.machine("HORSE").unwrap();
    close("A0: horses", h.stock, A0_HORSE_STOCK);
    close("A0: O", h.operating_cost, A0_HORSE_DAYS_OPERATING);
    close("A0: V", h.build_cost, A0_HORSE_DAYS_BUILD);
    // As one type: V = (p_m − ω·b)/δ, W = V·X at J 1.
    let t = &one.types[0];
    close("A0 one type: V", t.build_cost, A0_ONE_BUILD);
    close(
        "A0 one type: V from p_m",
        t.build_cost,
        (G1_P_M - A0_OMEGA * 0.4) / per_week(0.1),
    );
    close("A0 one type: W", t.wealth, A0_ONE_WEALTH);
    close("A0 one type: X", t.services, G1_K);
}

#[test]
fn a0_with_interest() {
    // A0R: ρ 5% a year, a tick's 1.05^(1/52) − 1: unit 1a's durable point, which 1a's a < 1
    // refuses (a/δ = 148), and the chain and the one type agree.
    let rho = rho_per_week(0.05);
    let (_, chain) = solved_chain(a0(rho));
    let one = interior_1c(a0_one_type(rho));
    for (what, eq) in [("A0R's chain", &*chain.eq), ("A0R as one type", &*one)] {
        aggregates(
            what,
            eq,
            [
                A0R_X_STAR,
                A0R_ONE_MINUS_X_STAR,
                A0R_V,
                A0R_P_S,
                A0R_Y,
                A0R_N_A,
                A0R_INCOME,
            ],
        );
        close(&format!("{what}: interest"), eq.interest, A0R_INTEREST);
    }
    good_is("A0R", &chain, "FODDER", A0R_FODDER_PRICE, A0R_FODDER_OUTPUT);
    good_is("A0R", &chain, "HORSE", A0R_HORSE_PRICE, A0R_HORSE_OUTPUT);
    good_is(
        "A0R",
        &chain,
        "HORSE_DAYS",
        A0R_HORSE_DAYS_PRICE,
        A0R_HORSE_DAYS_HOURS,
    );
    let h = chain.machine("HORSE").unwrap();
    close("A0R: horses", h.stock, A0R_HORSE_STOCK);
    close("A0R: O", h.operating_cost, A0R_HORSE_DAYS_OPERATING);
    close("A0R: V", h.build_cost, A0R_HORSE_DAYS_BUILD);
    // 1a's own form refuses it.
    let params = oracle::Params {
        a: 0.3 / per_week(0.1),
        rho,
        delta: per_week(0.1),
        build_lag: 1,
        ..appendix_b()
    };
    assert_eq!(
        oracle::Economy::new(params).expect_err("a >= 1").name(),
        "a"
    );
}

#[test]
fn a0_in_the_fork_economy_is_c3() {
    // A0's machine as one type in 1b's fork economy C3, at ρ 0: 1b's C3 exactly (the goldens to
    // 1e-12).
    let fork = fork_economy();
    let params = MachineParams {
        workers: 4.0,
        land: 10.0,
        schedule: linear(1.0, 0.2, 0.8),
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.0,
        machine_types: vec![a0_type()],
        edges: fork.edges,
        categories: fork.categories,
        intermediate: no_inputs(4),
    };
    let eq = interior_1c(params);
    close("A0F: x*", eq.x_star, C3_X_STAR);
    close("A0F: v", eq.v, C3_V);
    close("A0F: P_s", eq.p_s, C3_P_S);
    close("A0F: Y", eq.y, C3_Y);
    close("A0F: N_a", eq.n_a, C3_N_A);
    close("A0F: p_m", eq.types[0].price, C3_P_M);
    close("A0F: K", eq.types[0].services, C3_K);
    for (j, want) in [C3_MANUFACTURES_P, C3_FOOD_P, C3_CARE_P, C3_SHELTER_P]
        .into_iter()
        .enumerate()
    {
        close(&format!("A0F: p_{j}"), eq.categories[j].price, want);
    }
    close("A0F: V", eq.types[0].build_cost, A0F_BUILD);
}

#[test]
fn the_clock_gives_the_tick() {
    // The tape builder converts yearly values with core's Clock (docs/unit-1g.md §2.3); the
    // goldens are the 70-digit values of the same formulas.
    close_to("delta a week", per_week(0.1), A0_DELTA, 1e-15);
    close_to("delta a week, 8%", per_week(0.08), HORSE_DELTA, 1e-15);
    close_to("rho a week", rho_per_week(0.05), A0_RHO_TICK, 1e-15);
    close_to("the plant's delta", per_week(0.1), PLANT_DELTA, 1e-15);
    assert_eq!(ticks(3.0), 156);
}
