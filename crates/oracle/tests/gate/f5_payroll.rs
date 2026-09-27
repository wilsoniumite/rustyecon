//! f5: payroll incidence (docs/unit-1f.md §4.10 (e); SSRN D.2 p.32; main.tex:829-833): workers
//! bear ε_D/(ε_D + ε_S) of a payroll tax on the line, none of it at the wall where labour
//! demand does not move, all of it on idle land and where participation saturates.

use oracle::Government;
use rustyecon_core::num;

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1f::*;

#[test]
fn incidence_on_the_line() {
    // At G1: −d ln ω_net/dτ_w at τ_w = 0, by the one-sided three-point difference over the
    // oracle's equilibria at τ_w = 0, 2^-20 and 2^-19 (§5.5), is ε_D/(ε_D + ε_S), with ε_D and ε_S
    // from the path at x* ± 2^-20.
    let h = 1.0 / 1_048_576.0;
    let g = |tw: f64| {
        let eq = solved_1f(g1_with(payroll(tw)));
        num::ln(omega_net(&payroll(tw), &eq))
    };
    let share = (3.0 * g(0.0) - 4.0 * g(h) + g(2.0 * h)) / (2.0 * h);
    close_to("the incidence share", share, INC_SHARE, 1e-8);
    let e = economy_1f(g1_with(Government::none()));
    let x = solved_1f(g1_with(Government::none())).base.base.x_star;
    let (up, down) = (e.at(x + h), e.at(x - h));
    let ln_omega = |q: &oracle::HouseholdPoint| num::ln(q.point.point.v / q.price);
    let d_omega = ln_omega(&up) - ln_omega(&down);
    let eps_d = -(num::ln(up.point.point.n_d) - num::ln(down.point.point.n_d)) / d_omega;
    let eps_s = (num::ln(up.point.point.n_s) - num::ln(down.point.point.n_s)) / d_omega;
    close_to("ε_D", eps_d, INC_EPS_D, 1e-8);
    close_to("ε_S", eps_s, INC_EPS_S, 1e-8);
    close_to("ε_D/(ε_D + ε_S)", eps_d / (eps_d + eps_s), INC_SHARE, 1e-8);
}

#[test]
fn payroll_goldens() {
    // GP: τ_w 0.1 on the line; supply reads the net wage, κ_w·v = 0.9·v.
    let (e, eq) = checked_1f(g1_with(payroll(0.1)));
    assert_eq!(e.net_factor(), 0.9);
    let b = &eq.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, GP_X_STAR),
        ("v", b.v, GP_V),
        ("P", eq.basket.price, GP_P),
        ("N_a", b.n_a, GP_N_A),
    ] {
        close(name, got, want);
    }
    close("G_w", eq.government.revenue_payroll, 0.1 * b.wage_bill);
}

#[test]
fn borne_by_employers_at_the_wall() {
    // W1 (λ 0.6) at its wall: labour demand does not move with the wage (ε_D = 0), so the net
    // real wage is the same at every τ_w and the gross wage rises by 1/(1 − τ_w) in real terms.
    let (mut net, mut gross) = (Vec::new(), Vec::new());
    for (tw, want) in [(0.0, W1_V_0), (0.05, W1_V_005), (0.1, W1_V_01)] {
        let (_, eq) = checked_1f(household(g1_parcels(4.0, 0.6, 1.0), payroll(tw)));
        let b = &eq.base.base;
        assert_eq!(b.margin, oracle::Margin::Wall);
        close(&format!("W1 v at {tw}"), b.v, want);
        net.push(omega_net(&payroll(tw), &eq));
        gross.push(b.v);
        close(
            "Y as at τ_w 0",
            b.y,
            solved_1f(household(g1_parcels(4.0, 0.6, 1.0), Government::none()))
                .base
                .base
                .y,
        );
    }
    for w in &net {
        close_to("the net real wage", *w, W1_OMEGA_NET, 1e-14);
    }
    assert!(gross[0] < gross[1] && gross[1] < gross[2]);
}

#[test]
fn onto_idle_land() {
    // WI: W1 at τ_w 0.2. The wall's real-wage ceiling in net terms, (1 − τ_w)/L_s, is below the
    // net wage the wall needs, so the economy moves onto idle land (1e §4.6), ω_net = 14/9.
    let (_, eq) = checked_1f(household(g1_parcels(4.0, 0.6, 1.0), payroll(0.2)));
    assert_eq!(eq.base.land_market, oracle::LandMarket::Idle);
    assert!(eq.base.land.idle > 0.0);
    close("ω_net", omega_net(&payroll(0.2), &eq), WI_OMEGA_NET);
    close("T_m", eq.base.land.market, WI_T_M);
    close("N_a", eq.base.base.n_a, WI_N_A);
    close("Y", eq.base.base.y, WI_Y);
    assert_eq!(eq.government.rent_tax, None);
}

#[test]
fn borne_by_workers_when_participation_saturates() {
    // TX: supply saturates (ε_S = 0), so a payroll tax moves no allocation bit for bit and the net
    // real wage falls by the whole tax, (1 − τ_w)·0.8.
    let base = solved_1f(tx(Government::none()));
    for tw in [0.125, 0.25, 0.5] {
        let (_, eq) = checked_1f(tx(payroll(tw)));
        same_allocation_bits(&format!("TX at τ_w {tw}"), &base, &eq);
        close(
            "ω_net",
            omega_net(&payroll(tw), &eq),
            (1.0 - tw) * TX_OMEGA_NET,
        );
    }
}
