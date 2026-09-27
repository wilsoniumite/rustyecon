//! f2: three-taxes inside the full closure (docs/unit-1f.md §3.3 TX, §4.9; PLAN Phase 1's
//! gate: "three-taxes' resolution ledger, (φ_w, φ_r) = (0.6, 0.4) on its worked instance"). 1a
//! has the ledger as a price block (g7); here the worked instance is an equilibrium, x* = 1/2 in
//! exact arithmetic, whose machine service and whose basket both split (0.6, 0.4), with T6's
//! legs and T5's circular flow at the equilibrium.

use oracle::{closure, Government};

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1f::*;

#[test]
fn the_worked_instance_inside_the_closure() {
    // check_three_taxes.py:27-61; main.tex:325-336: TX's margin sits at γ(x*) = 3 exactly, so
    // its price block is the worked instance (a, λ, γ*, b, r) = (0.5, 0.1, 3, 0.2, 1).
    let (_, eq) = checked_1f(tx(Government::none()));
    let b = &eq.base.base;
    assert!(
        b.x_star.to_bits().abs_diff(0.5_f64.to_bits()) <= 4,
        "x* {} is not 1/2 to 4 ulps",
        b.x_star
    );
    for (name, got, want) in [
        ("x*", b.x_star, TX_X_STAR),
        ("γ(x*)", b.gamma_star, TX_GAMMA_STAR),
        ("p_m", b.types[0].price, TX_P_M),
        ("v", b.v, TX_V),
        ("the good's price", b.categories[0].price, TX_P_GOOD),
        ("P", eq.basket.price, TX_P),
        ("L_s", b.l_s, TX_L_S),
        ("B_s", b.b_s, TX_B_S),
        ("Y", b.y, TX_Y),
        ("N_a", b.n_a, TX_N_A),
        ("W", b.wage_bill, TX_W),
        ("R", eq.base.rent * eq.base.land.market, TX_R),
        ("I", b.income, TX_I),
        ("λ̃_m", b.types[0].lambda_tilde, TX_LAMBDA_M),
        ("b̃_m", b.types[0].b_tilde, TX_B_M),
        ("κ", eq.base.coverage, TX_KAPPA),
        ("ω_net", omega_net(&Government::none(), &eq), TX_OMEGA_NET),
    ] {
        close(name, got, want);
    }
    // (φ_w, φ_r) = (0.6, 0.4) for the machine service and for the basket.
    close("φ_w, machine", b.phi_w.unwrap(), TX_PHI_W);
    close("φ_r, machine", b.phi_r.unwrap(), TX_PHI_R);
    close("φ_w, basket", eq.ledger.basket_wage_share, TX_PHI_W);
    close(
        "φ_r, basket",
        eq.ledger.basket_rent_share.unwrap(),
        TX_PHI_R,
    );
    // the ledger closes: W_u + R_u = p_m, W_u = λw/(1 − a), R_u = b r/(1 − a)
    let (w_u, r_u) = (0.1 * b.v / 0.5, 0.2 / 0.5);
    close("W_u + R_u = p_m", w_u + r_u, b.types[0].price);
    close("W_u/p_m", w_u / b.types[0].price, TX_PHI_W);
    // and 1a's closure agrees
    let c = closure(0.5, 0.1, b.gamma_star, 0.2, 1.0, 1.0).unwrap();
    close("closure p_m", c.p_m, b.types[0].price);
    close("closure w", c.w, b.v);
    close("closure φ_w", c.phi_w.unwrap(), b.phi_w.unwrap());
}

#[test]
fn every_tax_leaves_the_worked_instance() {
    // §4.10 (e) with ε_S = 0: supply saturates on the grid, so no tax moves the allocation, bit
    // for bit; the net real wage falls by the whole tax and the ledger stays (0.6, 0.4).
    let base = solved_1f(tx(Government::none()));
    let mut n = 0;
    for tw in [0.0, 0.125, 0.25, 0.5] {
        for tc in [0.0, 0.25] {
            for dh in [0.0, 1.0] {
                let g = government(|g| {
                    g.payroll = tw;
                    g.consumption = tc;
                    g.budget = oracle::Budget::RentRate { dividend: dh };
                });
                let (_, eq) = checked_1f(tx(g));
                same_allocation_bits(&format!("TX at {tw} {tc} {dh}"), &base, &eq);
                close(
                    "ω_net",
                    omega_net(&g, &eq),
                    (1.0 - tw) / (1.0 + tc) * TX_OMEGA_NET,
                );
                close("φ_w", eq.ledger.basket_wage_share, TX_PHI_W);
                close("φ_r", eq.ledger.basket_rent_share.unwrap(), TX_PHI_R);
                n += 1;
            }
        }
    }
    assert_eq!(n, 16);
}

#[test]
fn the_legs_of_a_consumption_tax() {
    // check_three_taxes.py:63-76: a 25% consumption tax splits into a wage leg and a rent leg in
    // the ledger's proportions, per unit of machine service, per composite and in aggregate;
    // returned as a uniform transfer (TX1, Dividend τ_R 0) it is 5/4 per person.
    let g = government(|g| {
        g.consumption = 0.25;
        g.budget = oracle::Budget::Dividend { rent_tax: 0.0 };
    });
    let (_, eq) = checked_1f(tx(g));
    let b = &eq.base.base;
    let t = 0.25;
    let m = &b.types[0];
    close(
        "wage leg per machine service",
        t * b.v * m.lambda_tilde,
        TX1_LEG_WAGE_MACHINE,
    );
    close(
        "rent leg per machine service",
        t * m.b_tilde,
        TX1_LEG_RENT_MACHINE,
    );
    close(
        "the legs sum",
        t * b.v * m.lambda_tilde + t * m.b_tilde,
        t * m.price,
    );
    close(
        "wage leg per composite",
        t * b.v * b.l_s,
        TX1_LEG_WAGE_BASKET,
    );
    close("rent leg per composite", t * b.b_s, TX1_LEG_RENT_BASKET);
    let gv = &eq.government;
    close("G_c", gv.revenue_consumption, TX1_G_C);
    close("the wage leg", gv.consumption_wage_leg, TX1_WAGE_LEG);
    close("the rent leg", gv.consumption_rent_leg, TX1_RENT_LEG);
    close(
        "G_c = the legs",
        gv.revenue_consumption,
        gv.consumption_wage_leg + gv.consumption_rent_leg + gv.consumption_interest_leg,
    );
    close("d", gv.dividend, TX1_D);
}

#[test]
fn the_corner() {
    // check_three_taxes.py:77-85: at λ = 0 (TX3, T 12, x* = 1/2 again) the machine service is
    // all rent content, p_m = b/(1 − a) = 0.4 and v = 1.2 (1a's G6 at λ = 0, inside the
    // closure), and a 30% consumption tax and a 30% rent tax take the same 0.12 per unit.
    let (_, eq) = checked_1f(household(tx_parcels(0.0, 12.0), Government::none()));
    let b = &eq.base.base;
    for (name, got, want) in [
        ("x*", b.x_star, TX3_X_STAR),
        ("p_m", b.types[0].price, TX3_P_M),
        ("v", b.v, TX3_V),
        ("P", eq.basket.price, TX3_P),
        ("Y", b.y, TX3_Y),
        ("N_a", b.n_a, TX3_N_A),
    ] {
        close(name, got, want);
    }
    assert_eq!(b.phi_w, Some(0.0));
    assert_eq!(b.phi_r, Some(1.0));
    let m = &b.types[0];
    close(
        "30% consumption tax per unit",
        0.3 * m.price,
        TX3_TAX_PER_UNIT,
    );
    close("30% rent tax per unit", 0.3 * m.b_tilde, TX3_TAX_PER_UNIT);
}

#[test]
fn the_circular_flow() {
    // check_three_taxes.py:105-117: with τ_R = 1 (TX2) the rent revenue recycled into spending
    // generates rent at the basket's rate: R = R₀/(1 − φ^q_r·τ_R), R₀ 4.8, multiplier 5/3, and
    // d = 2 (Prop 6 (iii)).
    let (_, eq) = checked_1f(tx(dividend(1.0)));
    let l = &eq.ledger;
    close("φ^q_r", l.rent_share, TX2_RENT_SHARE);
    close("R₀", l.recirculated_base, TX2_R0);
    close("multiplier", l.multiplier.unwrap(), TX2_MULTIPLIER);
    let rent = eq.base.rent * eq.base.land.market;
    close(
        "R = R₀ multiplier",
        l.recirculated_base * l.multiplier.unwrap(),
        rent,
    );
    close("d", eq.government.dividend, TX2_D);
    // R = φ^q_r·C at every equilibrium with interest too (check_identities_1f), e.g. 1c's M4
    let (_, m4) = checked_1f(household(
        crate::support_1e::e0(oracle::WorkerParams::from_machines(crate::support_1c::m4(
            1.0,
        ))),
        dividend(1.0),
    ));
    assert!(m4.base.base.interest > 0.0);
    close(
        "R = φ C with interest",
        m4.base.rent * m4.base.land.market,
        m4.ledger.rent_share * m4.base.base.y * m4.basket.price,
    );
}
