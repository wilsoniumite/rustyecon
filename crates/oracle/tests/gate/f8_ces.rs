//! f8: the CES household (docs/unit-1f.md §2.10-2.13, §4.2, §5.4; SSRN eq 26 p.31;
//! main.tex:807-810): the composite's content at the equilibrium's prices, space's share eq 26's
//! α(q) along the path, the land-share household of check_pinning's A-joint, a free category that
//! empties the idle stretch, and Lemma F1's composition effect.

use oracle::{ces_share, Basket, HouseholdParams, LandMarket, Margin, Regime};
use rustyecon_core::num;

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1f::*;

/// P = Z·M(p) of a CES over the categories with weights z, written here from §4.2.
fn ces_price(z: &[f64], sigma: f64, p: &[f64]) -> f64 {
    let total: f64 = z.iter().sum();
    let mut sum = 0.0;
    for (zj, pj) in z.iter().zip(p) {
        if *zj > 0.0 {
            sum += (zj / total) * num::pow(*pj, 1.0 - sigma);
        }
    }
    if sigma == 1.0 {
        let mut ln = 0.0;
        for (zj, pj) in z.iter().zip(p) {
            if *zj > 0.0 {
                ln += (zj / total) * num::ln(*pj);
            }
        }
        return total * num::exp(ln);
    }
    total * num::pow(sum, 1.0 / (1.0 - sigma))
}

#[test]
fn eq_26_at_the_equilibrium() {
    // C1-C3 on G1's economy: the goldens; space's expenditure share is eq 26's α(q) (1b's
    // ces_share) at the equilibrium's q = r/p_g; Euler's P = Σ_j c_j·p_j; Shephard's c_j =
    // ∂P/∂p_j against a central difference.
    for (tag, sigma, want) in [
        (
            "C1",
            0.5,
            [C1_X_STAR, C1_V, C1_P, C1_N_A, C1_SPACE_SHARE, C1_Q],
        ),
        (
            "C2",
            1.0,
            [C2_X_STAR, C2_V, C2_P, C2_N_A, C2_SPACE_SHARE, C2_Q],
        ),
        (
            "C3",
            2.0,
            [C3_X_STAR, C3_V, C3_P, C3_N_A, C3_SPACE_SHARE, C3_Q],
        ),
    ] {
        let (_, eq) = checked_1f(ces(g1_parcels(4.0, 0.05, 1.0), sigma));
        let b = &eq.base.base;
        let got = [
            b.x_star,
            b.v,
            eq.basket.price,
            b.n_a,
            eq.basket.shares[1],
            eq.base.q,
        ];
        for ((name, g), w) in ["x*", "v", "P", "N_a", "space's share", "q"]
            .iter()
            .zip(got)
            .zip(want)
        {
            close(&format!("{tag} {name}"), g, w);
        }
        close(
            &format!("{tag}: eq 26"),
            eq.basket.shares[1],
            ces_share(0.3, sigma, eq.base.q).unwrap(),
        );
        let prices: Vec<f64> = b.categories.iter().map(|c| c.price).collect();
        let z = [num::pow(0.7, sigma), num::pow(0.3, sigma)];
        close(
            &format!("{tag}: P"),
            ces_price(&z, sigma, &prices),
            eq.basket.price,
        );
        for j in 0..2 {
            let h = prices[j] * 1e-6;
            let (mut up, mut down) = (prices.clone(), prices.clone());
            up[j] += h;
            down[j] -= h;
            let shephard = (ces_price(&z, sigma, &up) - ces_price(&z, sigma, &down)) / (2.0 * h);
            close_to(
                &format!("{tag}: Shephard {j}"),
                shephard,
                eq.basket.content[j],
                1e-8,
            );
        }
    }
    // C2's space share is α itself at every q
    assert_eq!(C2_SPACE_SHARE, 0.3);
}

#[test]
fn eq_26_along_the_path() {
    // App. C p.31: along AP's path (λ 0, γ = η(1 + x)) q = r/p_g grows without bound, and
    // space's share tends to 1, α and 0 for σ below, at and above 1.
    let rows = [
        (
            0.5,
            [
                (0, CP05_0_SHARE, CP05_0_Q),
                (8, CP05_8_SHARE, CP05_8_Q),
                (16, CP05_16_SHARE, CP05_16_Q),
            ],
        ),
        (
            1.0,
            [
                (0, CP1_0_SHARE, CP1_0_Q),
                (8, CP1_8_SHARE, CP1_8_Q),
                (16, CP1_16_SHARE, CP1_16_Q),
            ],
        ),
        (
            2.0,
            [
                (0, CP2_0_SHARE, CP2_0_Q),
                (8, CP2_8_SHARE, CP2_8_Q),
                (16, CP2_16_SHARE, CP2_16_Q),
            ],
        ),
    ];
    for (sigma, points) in rows {
        let mut shares = Vec::new();
        for (k, share, q) in points {
            let eta = 1.0 / f64::from(1_u32 << k);
            let (_, eq) = checked_1f(ces(ap_parcels(eta), sigma));
            // the share and q are made of prices that carry 1 − x* near x = 1 (1a §4)
            let tol = 1e-12_f64.max(8.0 * f64::EPSILON / eq.base.base.one_minus_x_star);
            close_to(
                &format!("share σ {sigma} η 2^-{k}"),
                eq.basket.shares[1],
                share,
                tol,
            );
            close_to(&format!("q σ {sigma} η 2^-{k}"), eq.base.q, q, tol);
            shares.push(eq.basket.shares[1]);
        }
        if sigma < 1.0 {
            assert!(shares[0] < shares[1] && shares[1] < shares[2] && 1.0 - shares[2] < 0.01);
        } else if sigma > 1.0 {
            assert!(shares[0] > shares[1] && shares[1] > shares[2] && shares[2] < 1e-5);
        } else {
            assert!(shares.iter().all(|&s| (s - 0.3).abs() < 1e-12));
        }
    }
}

#[test]
fn the_land_share_household() {
    // check_pinning's A-joint household, Ces { sigma: 1 } over (good, space) with z = (0.7, 0.3)
    // and saturated supply: AJ1 (γ = 1 + 3.9x) on the line, where P = p^0.7·r^0.3
    // (check_dynamics :405), and AJW (λ 0.08) at its wall in closed form.
    let (_, aj1) = checked_1f(a_joint(3.9, 0.1));
    let b = &aj1.base.base;
    let p = b.categories[0].price;
    for (name, got, want) in [
        ("x*", b.x_star, AJ1_X_STAR),
        ("v", b.v, AJ1_V),
        ("P", aj1.basket.price, AJ1_P),
        ("w/p", b.v / p, AJ1_W_P),
        ("r/p", 1.0 / p, AJ1_R_P),
        ("goods", b.categories[0].output, AJ1_GOODS),
    ] {
        close(&format!("AJ1 {name}"), got, want);
    }
    close("P = p^0.7 r^0.3", aj1.basket.price, num::pow(p, 0.7));
    close("full participation", b.n_a, 1.0);
    let (_, ajw) = checked_1f(a_joint(4.0, 0.08));
    let b = &ajw.base.base;
    assert_eq!(b.margin, Margin::Wall);
    let p = b.categories[0].price;
    for (name, got, want) in [
        ("goods", b.categories[0].output, AJW_GOODS),
        ("housing", b.categories[1].output, AJW_HOUSING),
        ("r/p", 1.0 / p, AJW_R_P),
        ("p", p, AJW_P_GOOD),
        ("v", b.v, AJW_V),
        ("w/p", b.v / p, AJW_W_P),
    ] {
        close(&format!("AJW {name}"), got, want);
    }
}

#[test]
fn a_joint_is_not_viable() {
    // Decision 64, §2.13: A-joint itself has 1 − a − λ·γ(1) = 1 − 0.5 − 0.1·5 = 0 exactly, in
    // decimals and in f64, so the oracle refuses it with D(1) = 0.
    assert_eq!(1.0 - 0.5 - 0.1 * 5.0, 0.0);
    assert_eq!(
        economy_1f(a_joint(4.0, 0.1)).solve(),
        Ok(Regime::NotViable { d_at_1: 0.0 })
    );
}

#[test]
fn no_idle_stretch_with_a_free_category() {
    // §2.12: 1d's W3 (λ 0.6, χ_max 3), LaborShort in 1d and on idle land in 1e, under C1's and
    // C3's baskets: space embodies no labour at the wall's end, so it becomes free relative to
    // the good and n_D → 0 there; f_∞ = −S_∞, land never idles, and the equilibrium is on the
    // wall with land scarce.
    for (tag, sigma, v, p, n, f_end) in [
        ("CW 0.5", 0.5, CW05_V, CW05_P, CW05_N_A, CW05_F_END),
        ("CW 2", 2.0, CW2_V, CW2_P, CW2_N_A, CW2_F_END),
    ] {
        let (_, eq) = checked_1f(ces(g1_parcels(4.0, 0.6, 3.0), sigma));
        let b = &eq.base.base;
        assert_eq!(eq.base.land_market, LandMarket::Scarce, "{tag}");
        assert_eq!(b.margin, Margin::Wall, "{tag}");
        close(tag, b.v, v);
        close(tag, eq.basket.price, p);
        close(tag, b.n_a, n);
        close(tag, eq.base.f_end.unwrap(), f_end);
    }
    // under the fixed basket the same economy idles land
    let fixed = solved_1f(HouseholdParams::from_parcels(g1_parcels(4.0, 0.6, 3.0)));
    assert_eq!(fixed.base.land_market, LandMarket::Idle);
}

#[test]
fn an_idle_stretch_without_one() {
    // §2.12: W3 with space given one human-required hour per unit, so every weighted category
    // embodies labour, under C1's basket: 1e's idle stretch, the basket at the idle prices.
    let mut p = g1_parcels(4.0, 0.6, 3.0);
    p.human_required = vec![0.0, 1.0];
    let (_, eq) = checked_1f(ces(p, 0.5));
    assert_eq!(eq.base.land_market, LandMarket::Idle);
    for (name, got, want) in [
        ("T_m", eq.base.land.market, CI_T_M),
        ("Y", eq.base.base.y, CI_Y),
        ("N_a", eq.base.base.n_a, CI_N_A),
        ("P", eq.basket.price, CI_P),
        ("T_idle", eq.base.land.idle, CI_IDLE),
        ("space's share", eq.basket.shares[1], CI_SPACE_SHARE),
    ] {
        close(&format!("CI {name}"), got, want);
    }
}

#[test]
fn composition_lowers_labour_demand() {
    // Lemma F1 (§5.4): at a fixed technique and r, as v rises the composition moves L_s/B_s by
    // −σ·d ln v·Σ_j c_j·b̃_j·(ℓ_j − ℓ_s)·(φ(ℓ_j) − φ(ℓ_s))/B_s ≤ 0. On the wall (x = 1, the
    // technique fixed) n_D = T·L_s/B_s at ρ = 0, so d ln n_D/d ln v is the lemma's, against a
    // central difference; under the fixed basket it is 0.
    let mut checked = 0;
    for params in [
        ces(g1_parcels(4.0, 0.6, 3.0), 0.5),
        ces(g1_parcels(4.0, 0.6, 3.0), 2.0),
        a_joint(4.0, 0.08),
    ] {
        let sigma = match params.basket {
            Basket::Ces { sigma } => sigma,
            Basket::Fixed => unreachable!(),
        };
        let (e, eq) = checked_1f(params);
        let b = &eq.base.base;
        assert_eq!(b.margin, Margin::Wall);
        let t = b.technique;
        let price = eq.basket.price;
        let (c, v) = (&eq.basket.content, b.v);
        let (mut l, mut bb) = (0.0, 0.0);
        for (j, cat) in b.categories.iter().enumerate() {
            l += c[j] * cat.lambda_tilde;
            bb += c[j] * cat.b_tilde;
        }
        let phi_s = v * l / price;
        let mut closed = 0.0;
        for (j, cat) in b.categories.iter().enumerate() {
            let phi_j = v * cat.lambda_tilde / cat.price;
            closed +=
                c[j] * (cat.lambda_tilde * bb - cat.b_tilde * l) / (bb * bb) * (phi_j - phi_s);
        }
        let dlog = -sigma * closed / (l / bb);
        let h = 1e-6;
        let (up, down) = (
            e.at_wage(1.0, v * (1.0 + h), t),
            e.at_wage(1.0, v * (1.0 - h), t),
        );
        let fd = (num::ln(up.point.point.n_d) - num::ln(down.point.point.n_d))
            / (num::ln(1.0 + h) - num::ln(1.0 - h));
        close_to("Lemma F1", fd, dlog, 1e-6);
        assert!(dlog < 0.0);
        checked += 1;
    }
    assert_eq!(checked, 3);
    // the fixed basket: n_D does not move on the wall
    let e = economy_1f(HouseholdParams::from_parcels(g1_parcels(4.0, 0.6, 1.0)));
    let (a, b) = (e.at_wage(1.0, 20.0, 0), e.at_wage(1.0, 30.0, 0));
    assert_eq!(a.point.point.n_d, b.point.point.n_d);
}

#[test]
fn the_frozen_composite_is_another_economy() {
    // §3.3's wrong unit: a composite frozen at its reference content per good (one good and
    // (3/7)^σ space, a fixed basket) is a different economy from C1's and C3's CES households,
    // by far more than the gate's tolerance.
    for (sigma, x, share, ces_x, ces_share) in [
        (0.5, X1_X_STAR, X1_SPACE_SHARE, C1_X_STAR, C1_SPACE_SHARE),
        (2.0, X3_X_STAR, X3_SPACE_SHARE, C3_X_STAR, C3_SPACE_SHARE),
    ] {
        let mut p = g1_parcels(4.0, 0.05, 1.0);
        p.categories[1].weight = num::pow(3.0 / 7.0, sigma);
        let (_, frozen) = checked_1f(HouseholdParams::from_parcels(p));
        close("the frozen composite's x*", frozen.base.base.x_star, x);
        close("its space share", frozen.basket.shares[1], share);
        assert!((x - ces_x).abs() > 1e-3 && (share - ces_share).abs() > 1e-2);
    }
}
