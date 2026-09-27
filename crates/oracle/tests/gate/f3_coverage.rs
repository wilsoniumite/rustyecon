//! f3: Prop 6, coverage and the corollary (docs/unit-1f.md §4.8; SSRN §7, eq 16, eq 17, D.3,
//! p.31): the receipts identity, eq 16 as the RentRate closure's output, D.3's identity, eq 17
//! at full automation, and the fiscal-capacity corollary along SSRN's automation path.

use oracle::{Government, HouseholdParams};

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1d::goodspace;
use crate::support_1e::*;
use crate::support_1f::*;

#[test]
fn receipts() {
    // SSRN p.16 (iii); main.tex:541-545; check_pinning.py:107-129; check_interior.py:162-169:
    // incomes (1 − τ_R)·R and the dividend N·d sum to R (GC, TX2); the general receipts identity
    // is checked at every equilibrium by check_identities_1f.
    for params in [g1_with(dividend(0.5)), tx(dividend(1.0))] {
        let (_, eq) = checked_1f(params);
        let rent = eq.base.rent * eq.base.land.market;
        let g = &eq.government;
        close(
            "(1 − τ_R)R + N d = R",
            (1.0 - g.rent_tax.unwrap()) * rent + g.transfers,
            rent,
        );
    }
    close("TX2: (1 − 1)·8 + 4·2", 4.0 * TX2_D, TX_R);
}

#[test]
fn eq_16() {
    // SSRN eq 16, p.16: a transfer of one composite per person under RentRate needs the rent
    // tax τ_R = N·P/R = 1/κ, within the rent exactly when κ ≥ 1.
    let (_, ga) = checked_1f(g1_with(replacing(rent_rate(1.0))));
    close("GA τ_R", ga.government.rent_tax.unwrap(), GA_RENT_TAX);
    close(
        "GA τ_R = 1/κ",
        ga.government.rent_tax.unwrap(),
        1.0 / ga.base.coverage,
    );
    close("G1's κ", ga.base.coverage, G1_KAPPA);
    assert_eq!(ga.government.within_rent, Some(true));
    let (_, txd) = checked_1f(tx(rent_rate(1.0)));
    close("TXD τ_R", txd.government.rent_tax.unwrap(), TXD_RENT_TAX);
    assert_eq!(txd.government.within_rent, Some(false));
    for (tag, people, eta, kappa, tau, within) in [
        ("KR4", 80.0, 1.0, KR4_KAPPA, KR4_RENT_TAX, false),
        ("KR5", 80.0, 0.7, KR5_KAPPA, KR5_RENT_TAX, true),
    ] {
        let (_, eq) = checked_1f(household(race(people, eta), replacing(rent_rate(1.0))));
        close(tag, eq.base.coverage, kappa);
        close(tag, eq.government.rent_tax.unwrap(), tau);
        close(tag, eq.government.rent_tax.unwrap(), 1.0 / eq.base.coverage);
        assert_eq!(eq.government.within_rent, Some(within), "{tag}");
        assert_eq!(within, eq.base.coverage >= 1.0);
        // each allocation is 1e's bit for bit (Replace at d̂ = ν)
        assert_eq!(
            bits_1e(&eq.base),
            bits_1e(&solved_1e(race(people, eta))),
            "{tag}"
        );
    }
    // KR1: plots rented, T_m < T, so N·P/(r·T_m) > 1/κ (§2.1, decision 115).
    let (_, kr1) = checked_1f(household(race(50.0, 2.5), replacing(rent_rate(1.0))));
    let tau = kr1.government.rent_tax.unwrap();
    close("KR1 τ_R", tau, KR1_RENT_TAX);
    close("KR1 κ", kr1.base.coverage, KR1_KAPPA);
    close(
        "KR1 τ_R = N P/(r T_m)",
        tau,
        50.0 * kr1.basket.price / kr1.base.land.market,
    );
    assert!(kr1.base.land.rented_plots > 0.0 && tau > 1.0 / kr1.base.coverage);
    // within_rent ⟺ κ ≥ 1 on economies without rented plots, one composite each
    let mut both = (false, false);
    for p in [
        e0(goodspace(4.0, 0.05, 1.0)),
        e0(goodspace(4.0, 0.6, 1.0)),
        e0(goodspace(20.0, 0.05, 0.05)),
        race(80.0, 1.0),
        race(80.0, 0.7),
        k1(),
    ] {
        let (_, eq) = checked_1f(household(p, rent_rate(1.0)));
        if eq.base.land.rented_plots == 0.0 && eq.base.land.market == eq.base.land.enclosed {
            let within = eq.government.within_rent.unwrap();
            assert_eq!(within, eq.base.coverage >= 1.0);
            if within {
                both.0 = true;
            } else {
                both.1 = true;
            }
        }
    }
    assert!(both.0 && both.1);
}

#[test]
fn d3_identity() {
    // SSRN D.3 p.32; check_feasibility F-iii: κ = T/(N·(Lᵖ_s·v + Bᵖ_s)) at r = 1 (checked at
    // every equilibrium by check_identities_1f), and N·Bᵖ_s ≤ T wherever κ ≥ 1.
    let mut n = 0;
    for params in [
        g1_with(Government::none()),
        g1_with(rent_rate(1.0)),
        g1_with(payroll(0.1)),
        tx(Government::none()),
        household(race(80.0, 0.7), Government::none()),
        ces(g1_parcels(4.0, 0.05, 1.0), 0.5),
    ] {
        let (e, eq) = checked_1f(params);
        let b = &eq.base.base;
        let t = e.parcels().enclosed_land();
        let people: f64 = e
            .params()
            .economy
            .worker_types
            .iter()
            .map(|w| w.workers)
            .sum();
        close(
            "κ, D.3",
            eq.base.coverage,
            t / (people * (b.l_s * b.v + b.b_s)),
        );
        if eq.base.coverage >= 1.0 {
            assert!(people * b.b_s <= t);
            n += 1;
        }
    }
    assert!(n >= 3);
}

#[test]
fn eq_17_at_full_automation() {
    // SSRN eq 17, p.17 and p.31: at x = 1 with λ = 0 (TX3's economy and AP's) the basket embodies
    // no labour, so the whole value of output is rent: P·Y = r·T, P = r·B_s and κ = T/(N·B_s).
    for p in [
        tx_parcels(0.0, 12.0),
        ap_parcels(1.0),
        ap_parcels(1.0 / 256.0),
    ] {
        let e = economy_1f(HouseholdParams::from_parcels(p));
        let q = e.at(1.0);
        assert_eq!(q.point.point.n_d, 0.0);
        assert_eq!(q.point.point.h_s, 0.0);
        let t = e.parcels().enclosed_land();
        close("P Y = r T", q.price * q.point.point.y, t);
    }
    // along AP, κ·N·B_s/T → 1
    for (k, kappa, tnb) in [
        (0, AP0_KAPPA, AP0_T_NB),
        (8, AP8_KAPPA, AP8_T_NB),
        (16, AP16_KAPPA, AP16_T_NB),
        (20, AP20_KAPPA, AP20_T_NB),
    ] {
        let ratio = kappa / tnb;
        assert!(ratio <= 1.0 + 1e-15, "{k}");
        if k >= 16 {
            assert!(1.0 - ratio < 1e-9, "{k}: {ratio}");
        }
    }
}

/// AP at η = 2^-k with its goldens.
fn ap_row(k: i32) -> (f64, [f64; 8]) {
    let rows = [
        (
            0,
            [
                AP0_X_STAR,
                AP0_V,
                AP0_LABOR_SHARE,
                AP0_KAPPA,
                AP0_T_NB,
                AP0_GW_NP,
                AP0_BOUND,
                AP0_C_R,
            ],
        ),
        (
            4,
            [
                AP4_X_STAR,
                AP4_V,
                AP4_LABOR_SHARE,
                AP4_KAPPA,
                AP4_T_NB,
                AP4_GW_NP,
                AP4_BOUND,
                AP4_C_R,
            ],
        ),
        (
            8,
            [
                AP8_X_STAR,
                AP8_V,
                AP8_LABOR_SHARE,
                AP8_KAPPA,
                AP8_T_NB,
                AP8_GW_NP,
                AP8_BOUND,
                AP8_C_R,
            ],
        ),
        (
            12,
            [
                AP12_X_STAR,
                AP12_V,
                AP12_LABOR_SHARE,
                AP12_KAPPA,
                AP12_T_NB,
                AP12_GW_NP,
                AP12_BOUND,
                AP12_C_R,
            ],
        ),
        (
            16,
            [
                AP16_X_STAR,
                AP16_V,
                AP16_LABOR_SHARE,
                AP16_KAPPA,
                AP16_T_NB,
                AP16_GW_NP,
                AP16_BOUND,
                AP16_C_R,
            ],
        ),
        (
            20,
            [
                AP20_X_STAR,
                AP20_V,
                AP20_LABOR_SHARE,
                AP20_KAPPA,
                AP20_T_NB,
                AP20_GW_NP,
                AP20_BOUND,
                AP20_C_R,
            ],
        ),
    ];
    let (_, row) = rows.into_iter().find(|(kk, _)| *kk == k).unwrap();
    (1.0 / f64::from(1_u32 << k), row)
}

#[test]
fn the_corollary() {
    // SSRN p.16 corollary, p.31; check_mix M-i: along AP with τ_w 1/2, v/r → 0, the payroll
    // revenue per person over P falls below its bound τ_w·(v/r)/Bᵖ_s toward 0, R/I rises to 1,
    // the labour share falls to 0 and C/R → 1: the consumption base becomes the rent base.
    let mut last: Option<(f64, f64, f64)> = None;
    for k in [0, 4, 8, 12, 16, 20] {
        let (eta, want) = ap_row(k);
        let (e, eq) = checked_1f(household(ap_parcels(eta), payroll(0.5)));
        let b = &eq.base.base;
        let t = e.parcels().enclosed_land();
        let (people, price) = (4.0, eq.basket.price);
        let rent = eq.base.rent * eq.base.land.market;
        let gw = eq.government.revenue_payroll / (people * price);
        let bound = 0.5 * b.v / b.b_s;
        let got = [
            b.x_star,
            b.v,
            b.labor_share,
            eq.base.coverage,
            t / (people * b.b_s),
            gw,
            bound,
            b.y * price / rent,
        ];
        let names = [
            "x*",
            "v/r",
            "labour share",
            "κ",
            "T/(N B_s)",
            "G_w/(N P)",
            "bound",
            "C/R",
        ];
        for ((name, g), w) in names.iter().zip(got).zip(want) {
            // outputs made of 1 − x* keep 2^-53/(1 − x*) relative near x = 1 (1a §4)
            let tol = 1e-12_f64.max(4.0 * f64::EPSILON / (1.0 - want[0]));
            close_to(&format!("AP{k} {name}"), g, w, tol);
        }
        assert!(gw <= bound);
        let ri = rent / b.income;
        if let Some((gw0, ri0, share0)) = last {
            assert!(gw < gw0 && ri > ri0 && b.labor_share < share0, "AP{k}");
        }
        last = Some((gw, ri, b.labor_share));
    }
    let (gw, ri, share) = last.unwrap();
    assert!(gw < 1e-12 && 1.0 - ri < 1e-12 && share < 1e-12);
}
