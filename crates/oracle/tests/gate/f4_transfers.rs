//! f4: transfers and the rent tax (docs/unit-1f.md §2.4-2.5, §4.10 (b)-(d); SSRN p.16, Prop 6,
//! A.1): a supplementing transfer raises the reservation wage to (e^χ − 1)(P + d), the Dividend
//! closure returns the rent tax as d = τ_R·R/N in both states, and a transfer above the support
//! in Replace mode supplements by the excess.

use oracle::Government;
use rustyecon_core::num;

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1f::*;

#[test]
fn a_supplementing_transfer() {
    // SSRN p.16; main.tex:552: GB, one composite per person on top of the support. The marginal
    // worker's reservation wage (e^χ* − 1)·(P + d) is v (χ* = N_a/N·χ_max), and on the line
    // x* and v rise and N_a falls with d̂ (SSRN p.12: "a lower willingness to work weakly raises
    // x and w/r").
    let (_, gb) = checked_1f(g1_with(rent_rate(1.0)));
    let b = &gb.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, GB_X_STAR),
        ("v", b.v, GB_V),
        ("P", gb.basket.price, GB_P),
        ("N_a", b.n_a, GB_N_A),
        ("τ_R", gb.government.rent_tax.unwrap(), GB_RENT_TAX),
        ("κ", gb.base.coverage, GB_KAPPA),
    ] {
        close(name, got, want);
    }
    let chi = b.n_a / 4.0;
    close("χ*", b.workers[0].marginal_work_cost, chi);
    close(
        "the reservation wage (e^χ − 1)(P + d)",
        num::expm1(chi) * (gb.basket.price + gb.government.dividend),
        b.v,
    );
    let mut last: Option<(f64, f64, f64)> = None;
    for d in [0.0, 0.25, 0.5, 1.0] {
        let (_, eq) = checked_1f(g1_with(rent_rate(d)));
        let b = &eq.base.base;
        if let Some((x, v, n)) = last {
            assert!(b.x_star > x && b.v > v && b.n_a < n, "d̂ {d}");
        }
        last = Some((b.x_star, b.v, b.n_a));
    }
}

#[test]
fn the_dividend_closure() {
    // A.1; Prop 6: GC, τ_R 0.5 returned as d = τ_R·R/N = 5/4, paid in both states (Prop 6 (ii)):
    // a worker's resources exceed an exiter's by the net wage alone.
    let (_, gc) = checked_1f(g1_with(dividend(0.5)));
    let b = &gc.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, GC_X_STAR),
        ("v", b.v, GC_V),
        ("P", gc.basket.price, GC_P),
        ("N_a", b.n_a, GC_N_A),
        ("d", gc.government.dividend, GC_D),
    ] {
        close(name, got, want);
    }
    let rent = gc.base.rent * gc.base.land.market;
    close("d = τ_R R/N", gc.government.dividend, 0.5 * rent / 4.0);
    let a = &gc.accounts.workers[0];
    let worker = a.unearned + a.net_wage;
    let exiter = a.unearned;
    close("a worker's minus an exiter's", worker - exiter, a.net_wage);
    // and the transfer supplements the support: participation falls, as with GB
    let g1 = solved_1f(g1_with(Government::none()));
    assert!(b.n_a < g1.base.base.n_a && b.x_star > g1.base.base.x_star);
}

#[test]
fn replace_above_the_support() {
    // §2.4: in Replace mode a transfer above the support supplements by the excess: G1 with
    // RentRate d̂ 2 in Replace mode is d̂ 1 in Supplement mode, A_i = 2·P^c either way, bit for
    // bit in the allocation.
    let (_, replace) = checked_1f(g1_with(replacing(rent_rate(2.0))));
    let (_, supplement) = checked_1f(g1_with(rent_rate(1.0)));
    same_allocation_bits("Replace 2 against Supplement 1", &replace, &supplement);
    assert_eq!(
        replace.accounts.workers[0].unearned.to_bits(),
        supplement.accounts.workers[0].unearned.to_bits()
    );
    assert_eq!(replace.accounts.workers[0].support, 0.0);
    // the owners pay 2 composites per person in one, 1 in the other: the same total support
    close(
        "total support",
        replace.government.transfers + 4.0 * replace.accounts.workers[0].support,
        supplement.government.transfers + 4.0 * supplement.accounts.workers[0].support,
    );
}
