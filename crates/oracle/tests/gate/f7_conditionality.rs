//! f7: conditionality and walled types (docs/unit-1f.md §2.8-2.9, §4.5, §4.10 (h); SSRN D.1
//! eq 27; main.tex:819-825; check_conditionality.py D-i): an in-work benefit lowers the
//! reservation wage and, on the line, the gross wage; a uniform program is a supplementing
//! transfer; a benefit that alone overfills the economy has no equilibrium; a walled type is
//! paid c_i·P under a government.

use oracle::{Government, Regime, SolveError};
use rustyecon_core::num;

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1f::*;

#[test]
fn eq_27() {
    // SSRN D.1 eq 27 p.32: the marginal worker's reservation wage is (e^χ* − 1)·(P + m_e) −
    // (m_w − m_e) at ν = 1, χ* = N_a/N·χ_max.
    for (work, exit) in [(0.2, 0.0), (0.0, 0.2)] {
        let (_, eq) = checked_1f(g1_with(program(work, exit)));
        let b = &eq.base.base;
        let (m_w, m_e) = (eq.government.program_work, eq.government.program_exit);
        let chi = b.n_a / 4.0;
        close("χ*", b.workers[0].marginal_work_cost, chi);
        close(
            "eq 27",
            num::expm1(chi) * (eq.basket.price + m_e) - (m_w - m_e),
            b.v,
        );
    }
}

#[test]
fn in_work_benefits_lower_the_wage() {
    // main.tex:819-825: an in-work benefit (GW, μ_w 0.2) raises participation and lowers x* and
    // the gross wage; an out-of-work benefit (GE, μ_e 0.2) does the reverse.
    let g1 = solved_1f(g1_with(Government::none()));
    let (_, gw) = checked_1f(g1_with(program(0.2, 0.0)));
    let (_, ge) = checked_1f(g1_with(program(0.0, 0.2)));
    for (tag, eq, x, v, p, n) in [
        ("GW", &gw, GW_X_STAR, GW_V, GW_P, GW_N_A),
        ("GE", &ge, GE_X_STAR, GE_V, GE_P, GE_N_A),
    ] {
        let b = &eq.base.base;
        close(tag, b.x_star, x);
        close(tag, b.v, v);
        close(tag, eq.basket.price, p);
        close(tag, b.n_a, n);
    }
    let (b0, bw, be) = (&g1.base.base, &gw.base.base, &ge.base.base);
    assert!(bw.n_a > b0.n_a && b0.n_a > be.n_a);
    assert!(bw.v < b0.v && b0.v < be.v);
    assert!(bw.x_star < b0.x_star && b0.x_star < be.x_star);
}

#[test]
fn a_uniform_program_is_a_supplementing_transfer() {
    // D.1; §5.1: μ_w = μ_e = 0.2 (GU) and d̂ = 0.2 (GS) have the same allocation bit for bit,
    // and the program's cost equals the transfers'.
    let (_, gu) = checked_1f(g1_with(program(0.2, 0.2)));
    let (_, gs) = checked_1f(g1_with(rent_rate(0.2)));
    same_allocation_bits("GU = GS", &gu, &gs);
    let b = &gu.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, GU_X_STAR),
        ("v", b.v, GU_V),
        ("P", gu.basket.price, GU_P),
        ("N_a", b.n_a, GU_N_A),
    ] {
        close(name, got, want);
    }
    close(
        "M = N d",
        gu.government.program_cost,
        gs.government.transfers,
    );
    assert_eq!(gu.government.transfers, 0.0);
    assert_eq!(gs.government.program_cost, 0.0);
}

#[test]
fn surplus_labour() {
    // §2.9: 1d's W4 (N 20, χ_max 0.05) with μ_w 0.1: at a zero wage the benefit draws all 20
    // into work, against n_D(0) = 10, and f never changes side: SurplusLabour with f_0 = −10.
    let e = economy_1f(household(g1_parcels(20.0, 0.05, 0.05), program(0.1, 0.0)));
    assert_eq!(
        e.solve(),
        Err(SolveError::SurplusLabour {
            f_start: SL_F_START
        })
    );
    assert_eq!(e.start().unwrap(), SL_F_START);
    // without the benefit the start is n_D(0) = 10 and W4 has its all-human corner
    let w4 = economy_1f(household(g1_parcels(20.0, 0.05, 0.05), Government::none()));
    assert_eq!(w4.start().unwrap(), -SL_F_START);
    assert!(matches!(w4.solve(), Ok(Regime::Interior(_))));
}

#[test]
fn walled_types_under_a_government() {
    // §4.5: ER, 1d's E2 with τ_w 0.1, t_c 0.1, d̂ 0.5, μ_e 0.1: the trained at its wall, its
    // supply its reserved demand D_T, its wage c_T·P with c_T = ((ν_T + d̂ + μ_e)·ζ_T − (μ_w −
    // μ_e))·(1 + t_c)/(1 − τ_w).
    let (_, er) = checked_1f(household(e2_parcels(), er_government()));
    let b = &er.base.base;
    let trained = &b.workers[1];
    assert!(!trained.pooled && b.workers[0].pooled);
    close("D_T = n_S,T", trained.supply, trained.reserved_hours);
    let rate = ((1.2 + 0.5 + 0.1) * trained.clearing_real_wage - (0.0 - 0.1)) * 1.1 / 0.9;
    close("c_T", rate, ER_TRAINED_RATE);
    close("v_T = c_T P", trained.wage, ER_TRAINED_WAGE);
    close(
        "v_T = c_T P at the equilibrium's P",
        trained.wage,
        rate * er.basket.price,
    );
    for (name, got, want) in [
        ("v", b.v, ER_V),
        ("P", er.basket.price, ER_P),
        ("N_a", b.n_a, ER_N_A),
    ] {
        close(name, got, want);
    }
    assert_eq!(b.margin, oracle::Margin::Wall);
    // without the government the trained's wage is 1d's ζ_T·ν_T·P
    let (_, e2) = checked_1f(household(e2_parcels(), Government::none()));
    let t = &e2.base.base.workers[1];
    assert!(!t.pooled);
    close(
        "1d's walled wage",
        t.wage,
        t.clearing_real_wage * 1.2 * e2.basket.price,
    );
}
