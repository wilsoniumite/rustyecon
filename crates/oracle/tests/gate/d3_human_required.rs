//! d3: the human-required economy (docs/unit-1d.md §3.3's B; §8): the paper's H in the
//! equilibrium ("including the human-required tail", SSRN p.26), the tail deciding the regime,
//! and check_pinning D1's automation path to the wall with SSRN Prop E.1's limits.

use oracle::{ces_share, Margin};

use crate::goldens_1d::*;
use crate::support::*;
use crate::support_1d::*;

#[test]
fn interior_with_the_tail() {
    // B1, N 8, η 1: a contestable margin with |H| = 0.25 in services. Without the tail in labour
    // demand the root would be x* 0.8442.
    let (e, eq) = checked_1d(baumol_one(1.0, 8.0, 0.25));
    assert_eq!(eq.margin, Margin::Contestable);
    for (name, got, want) in [
        ("x*", eq.x_star, B1_X_STAR),
        ("1 - x*", eq.one_minus_x_star, B1_ONE_MINUS_X_STAR),
        ("v", eq.v, B1_V),
        ("p_m", eq.types[0].price, B1_P_M),
        ("P_s", eq.p_s, B1_P_S),
        ("Y", eq.y, B1_Y),
        ("N_a", eq.n_a, B1_N_A),
        ("final hours", eq.final_hours, B1_FINAL_HOURS),
        ("required hours", eq.required_hours, B1_REQUIRED_HOURS),
        ("I", eq.income, B1_INCOME),
        ("p services", eq.categories[0].price, B1_P_SERVICES),
        ("p goods", eq.categories[1].price, B1_P_GOODS),
        ("f_line_1", eq.f_line_1.unwrap(), B1_F_LINE_1),
    ] {
        close(name, got, want);
    }
    assert_eq!(eq.categories[2].price, 1.0);
    // The final hours are Y·(H_ŷ(x*) + L^H_ŷ); the tail is Y·L^H_ŷ = Y/4, bit for bit.
    assert_eq!(e.human_required_per_basket(), 0.25);
    assert_eq!(eq.required_hours.to_bits(), (eq.y * 0.25).to_bits());
    let h_line = 0.75 * eq.one_minus_x_star + eq.one_minus_x_star;
    close("final hours", eq.y * (h_line + 0.25), eq.final_hours);
    // Services' human hours carry the tail: h = 0.75(1 − x*) + 0.25.
    close(
        "h services",
        0.75 * eq.one_minus_x_star + 0.25,
        eq.categories[0].human,
    );
    close("L-bar services", 1.0, eq.categories[0].l_bar);
    assert_eq!(eq.categories[0].human_required, 0.25);
    let without = solved_1d(baumol_one(1.0, 8.0, 0.0));
    assert!(
        eq.x_star - without.x_star > 0.1,
        "{} {}",
        eq.x_star,
        without.x_star
    );
    check_path_1d(&e);
}

#[test]
fn the_tail_decides_the_regime() {
    // B2, N 4: contestable without the tail, at the wall with it; both funded.
    let (_, no_tail) = checked_1d(baumol_one(1.0, 4.0, 0.0));
    assert_eq!(no_tail.margin, Margin::Contestable);
    for (name, got, want) in [
        ("x*", no_tail.x_star, B2_NO_TAIL_X_STAR),
        ("v", no_tail.v, B2_NO_TAIL_V),
        ("P_s", no_tail.p_s, B2_NO_TAIL_P_S),
        ("Y", no_tail.y, B2_NO_TAIL_Y),
        ("N_a", no_tail.n_a, B2_NO_TAIL_N_A),
        (
            "provider baskets",
            no_tail.provider_baskets,
            B2_NO_TAIL_PROVIDER_BASKETS,
        ),
    ] {
        close(name, got, want);
    }
    let (e, tail) = checked_1d(baumol_one(1.0, 4.0, 0.25));
    assert_eq!(tail.margin, Margin::Wall);
    for (name, got, want) in [
        ("v", tail.v, B2_TAIL_V),
        ("g", tail.g, B2_TAIL_G),
        ("P_s", tail.p_s, B2_TAIL_P_S),
        ("Y", tail.y, B2_TAIL_Y),
        ("N_a", tail.n_a, B2_TAIL_N_A),
        (
            "provider baskets",
            tail.provider_baskets,
            B2_TAIL_PROVIDER_BASKETS,
        ),
        ("f_line_1", tail.f_line_1.unwrap(), B2_TAIL_F_LINE_1),
    ] {
        close(name, got, want);
    }
    assert!(no_tail.funded && tail.funded);
    assert!(tail.g > 2.0);
    check_path_1d(&e);
}

#[test]
fn baumol_path() {
    // check_pinning.py D1 (:147-178) as an equilibrium path, η → 0 with N 8, every point at
    // the wall: the machine price stays bounded at (λv + b)/(1 − a); services' unit cost tends
    // to |H|·v (P9-i's C → k·w_K/γ_L) and labour's share of it to 1; goods' price tends to 0 and
    // the relative price of H-content diverges; the CES share of H-content (α 0.3, σ 0.5) tends
    // to 1 (SSRN Prop E.1(i)-(ii)); the wall's wage converges to v_∞ while v/(γ(1)·π) grows as
    // 1/η: the machine no longer prices the hour.
    let table = [
        (
            0.3,
            [
                BP_ETA0_3_V,
                BP_ETA0_3_P_M,
                BP_ETA0_3_P_SERVICES_OVER_V,
                BP_ETA0_3_PHI_W_SERVICES,
                BP_ETA0_3_P_GOODS,
                BP_ETA0_3_RELATIVE_PRICE,
                BP_ETA0_3_CES_SHARE,
                BP_ETA0_3_V_OVER_REPLACEMENT,
            ],
        ),
        (
            0.1,
            [
                BP_ETA0_1_V,
                BP_ETA0_1_P_M,
                BP_ETA0_1_P_SERVICES_OVER_V,
                BP_ETA0_1_PHI_W_SERVICES,
                BP_ETA0_1_P_GOODS,
                BP_ETA0_1_RELATIVE_PRICE,
                BP_ETA0_1_CES_SHARE,
                BP_ETA0_1_V_OVER_REPLACEMENT,
            ],
        ),
        (
            0.01,
            [
                BP_ETA0_01_V,
                BP_ETA0_01_P_M,
                BP_ETA0_01_P_SERVICES_OVER_V,
                BP_ETA0_01_PHI_W_SERVICES,
                BP_ETA0_01_P_GOODS,
                BP_ETA0_01_RELATIVE_PRICE,
                BP_ETA0_01_CES_SHARE,
                BP_ETA0_01_V_OVER_REPLACEMENT,
            ],
        ),
        (
            0.001,
            [
                BP_ETA0_001_V,
                BP_ETA0_001_P_M,
                BP_ETA0_001_P_SERVICES_OVER_V,
                BP_ETA0_001_PHI_W_SERVICES,
                BP_ETA0_001_P_GOODS,
                BP_ETA0_001_RELATIVE_PRICE,
                BP_ETA0_001_CES_SHARE,
                BP_ETA0_001_V_OVER_REPLACEMENT,
            ],
        ),
        (
            1e-6,
            [
                BP_ETA1EM6_V,
                BP_ETA1EM6_P_M,
                BP_ETA1EM6_P_SERVICES_OVER_V,
                BP_ETA1EM6_PHI_W_SERVICES,
                BP_ETA1EM6_P_GOODS,
                BP_ETA1EM6_RELATIVE_PRICE,
                BP_ETA1EM6_CES_SHARE,
                BP_ETA1EM6_V_OVER_REPLACEMENT,
            ],
        ),
    ];
    let names = [
        "v",
        "p_m",
        "p_services/v",
        "phi_w services",
        "p_goods",
        "p_services/p_goods",
        "CES share",
        "v/(gamma(1) pi)",
    ];
    let mut last: Option<[f64; 8]> = None;
    for (eta, want) in table {
        let (e, eq) = checked_1d(baumol_one(eta, 8.0, 0.25));
        assert_eq!(eq.margin, Margin::Wall, "eta {eta}");
        let (services, goods) = (&eq.categories[0], &eq.categories[1]);
        let ratio = services.price / goods.price;
        let got = [
            eq.v,
            eq.types[0].price,
            services.price / eq.v,
            services.phi_w.unwrap(),
            goods.price,
            ratio,
            ces_share(0.3, 0.5, ratio).unwrap(),
            eq.v / eq.replacement_top,
        ];
        for ((name, g), w) in names.iter().zip(got).zip(want) {
            close(&format!("{name} at eta {eta}"), g, w);
        }
        close(
            &format!("p_m = (lambda v + b)/(1 - a) at eta {eta}"),
            (0.05 * eq.v + 0.4) / 0.7,
            eq.types[0].price,
        );
        if let Some(prev) = last {
            assert!(
                got[2] < prev[2] && got[2] > 0.25,
                "p_services/v falls to 1/4"
            );
            assert!(got[3] > prev[3] && got[3] < 1.0, "phi_w rises to 1");
            assert!(got[6] > prev[6] && got[6] < 1.0, "the CES share rises to 1");
            assert!(got[4] < prev[4], "p_goods falls");
            assert!(got[5] > prev[5], "the relative price rises");
            assert!(got[7] > prev[7], "v/(gamma(1) pi) grows");
            assert!(
                got[0] < prev[0] && got[0] > BLIM_V_INF,
                "v converges from above"
            );
        }
        last = Some(got);
        if eta == 0.1 {
            check_path_1d(&e);
        }
    }
    let at_limit = last.unwrap();
    near("p_services/v at 1e-6", at_limit[2], 0.25, 1e-6);
    near("phi_w at 1e-6", at_limit[3], 1.0, 1e-5);
    near("v at 1e-6", at_limit[0], BLIM_V_INF, 1e-6);
    // v_∞ = ζ/(1 − ζ/4), ζ = expm1(χ_max·2.5/N) = expm1(5/16) (§4.8).
    let zeta = rustyecon_core::num::expm1(5.0 / 16.0);
    close("v_inf", zeta / (1.0 - zeta / 4.0), BLIM_V_INF);
}
