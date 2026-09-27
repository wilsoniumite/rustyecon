//! f8: the CES household (docs/unit-1f.md §2.10-2.13, §4.2, §5.4; SSRN eq 26 p.31;
//! main.tex:807-810): the composite's content at the equilibrium's prices, space's share eq 26's
//! α(q) along the path, the land-share household of check_pinning's A-joint, a free category that
//! empties the idle stretch, and Lemma F1's composition effect; and from the verification
//! (§14 items 16-19): the start, the all-human corner as v → 0, the wall far out, a steep
//! basket with a small weight, and S_∞ with transfers and types.

use oracle::{
    ces_share, Basket, Budget, Government, HouseholdParams, LandMarket, Margin, ParcelParams,
    PowerSchedule, Program, Regime, SolveError, UniformWorkCost,
};
use rustyecon_core::num;

use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;
use crate::support_1f::*;

/// CA's economy (docs/unit-1f.md §7): Appendix B's with N 40, T 3, γ = 1.5 + x and χ_max 0.02.
fn ca_parcels() -> ParcelParams {
    from_1a_1e(oracle::Params {
        workers: 40.0,
        land: 3.0,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 1.5,
            g1: 1.0,
            k: 1.0,
        },
        work_cost: UniformWorkCost { chi_max: 0.02 },
        ..appendix_b()
    })
}

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
        // the good has no land at x = 0, so it is free at v = 0 and the start is +∞ (§14 item 5)
        assert_eq!(eq.f_start, None, "{tag}: the start");
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

#[test]
fn the_start_under_ces() {
    // §2.9, §5.3 step 1, §14 item 5: a CES economy's start is its evaluation at v = 0 on the
    // all-human corner, with the basket at the prices there, wherever every weighted category
    // carries land through the chain at x = 0 (here the good is given 0.2 of direct land); not
    // 1d's corner form, which reads z. C1-C3's, where the good is free at v = 0, is +∞ (None).
    let with_land = |workers: f64, chi_max: f64| {
        let mut p = g1_parcels(workers, 0.05, chi_max);
        p.categories[0].direct_land = 0.2;
        p
    };
    let (e, eq) = checked_1f(ces(with_land(4.0, 1.0), 0.5));
    let at_zero = e.at_wage(0.0, 0.0, 0).point.excess_demand();
    assert!(at_zero.is_finite() && at_zero > 0.0);
    assert_eq!(eq.f_start.map(f64::to_bits), Some(at_zero.to_bits()));
    // An in-work benefit that draws all 100 into work at a zero wage, against the labour the
    // basket needs there: a negative start with no change of side after it.
    let mut params = ces(with_land(100.0, 0.05), 0.5);
    params.government = program(0.2, 0.0);
    let e = economy_1f(params);
    let at_zero = e.at_wage(0.0, 0.0, 0).point.excess_demand();
    assert!(at_zero < 0.0);
    assert_eq!(e.start().unwrap().to_bits(), at_zero.to_bits());
    assert_eq!(
        e.solve(),
        Err(SolveError::SurplusLabour { f_start: at_zero })
    );
}

#[test]
fn the_good_free_at_the_all_human_corner() {
    // §5.3 step 2, §14 item 16: CA, C3's basket where the equilibrium is at the all-human corner.
    // The good has no land at x = 0, so as v → 0 space's content vanishes against it (σ = 2) and
    // Y = T/B_s overflows near v = 1e-154, where bisection on bit patterns makes its first
    // midpoints: labour demand there is beyond every double, f = +∞, not a NaN.
    let (e, eq) = checked_1f(ces(ca_parcels(), 2.0));
    let b = &eq.base.base;
    assert_eq!(b.margin, Margin::AllHuman);
    assert_eq!(eq.f_start, None);
    for (name, got, want) in [
        ("v", b.v, CA_V),
        ("P", eq.basket.price, CA_P),
        ("N_a", b.n_a, CA_N_A),
        ("space's share", eq.basket.shares[1], CA_SPACE_SHARE),
    ] {
        close(&format!("CA {name}"), got, want);
    }
    let tiny = e.at_wage(0.0, 1e-154, 0);
    assert_eq!(tiny.point.point.y, f64::INFINITY);
    assert_eq!(tiny.point.excess_demand(), f64::INFINITY);
    // every σ ≥ 2 on CA's economy, with and without a government, has its equilibrium
    for sigma in [2.0, 2.5, 3.0, 4.0, 8.0] {
        for g in [
            Government::none(),
            payroll(0.1),
            rent_rate(0.5),
            dividend(0.5),
        ] {
            let mut params = ces(ca_parcels(), sigma);
            params.government = g;
            checked_1f(params);
        }
    }
    // The verification's draw R7102_41 at σ = 2 under Dividend (t_c 0.15, τ_R 0.64, a uniform
    // program 0.16): at v = 2^-511 Y is finite but n_D = Y·L_s overflows, and the transfer, read
    // from W = v·n_D with τ_w = 0, is 0·∞ = NaN with the supply; that point is +∞ too.
    let e = economy_1f(r7102_41());
    let q = e.at_wage(0.0, f64::from_bits((1023 - 511) << 52), 0);
    assert!(q.point.point.y.is_finite());
    assert_eq!(q.point.point.n_d, f64::INFINITY);
    assert!(q.point.point.n_s.is_nan());
    assert_eq!(q.point.excess_demand(), f64::INFINITY);
    let (_, eq) = checked_1f(r7102_41());
    assert_eq!(eq.base.base.margin, Margin::AllHuman);
}

/// The verification's random draw R7102_41 (docs/unit-1f.md §14 item 16) at σ = 2 in place of its
/// 3.0096: Appendix B's form with the good given human-required hours and a Dividend closure.
fn r7102_41() -> HouseholdParams {
    let mut p = from_1a_1e(oracle::Params {
        workers: 13.835700083128295,
        land: 2.9198283496067368,
        a: 0.04346133664919507,
        lam: 0.07609959466923066,
        b: 0.6892880919296143,
        schedule: PowerSchedule {
            eta: 2.4951548902830303,
            g0: 0.6382528870794157,
            g1: 1.391007360728151,
            k: 1.0,
        },
        work_cost: UniformWorkCost {
            chi_max: 0.017079399059127517,
        },
        ..appendix_b()
    });
    p.categories[0].weight = 1.4244891686524364;
    p.categories[1].weight = 0.6824857875628916;
    p.human_required = vec![0.28103456621585265, 0.0];
    p.worker_types[0].support = 0.8483304539554257;
    HouseholdParams {
        economy: p,
        basket: Basket::Ces { sigma: 2.0 },
        government: government(|g| {
            g.consumption = 0.14981435914188668;
            g.budget = Budget::Dividend {
                rent_tax: 0.6386713829836139,
            };
            g.program = Program {
                work: 0.1602482564766707,
                exit: 0.1602482564766707,
            };
        }),
    }
}

#[test]
fn the_wall_far_out() {
    // §5.3 step 2, §5.5, §14 item 17: C1's basket on W3 with N 0.01 and 0.001 puts the wall's
    // equilibrium at v/r 4.3e6 and 4.3e8. A CES corner is bisected in v itself, which keeps its
    // digits there; in ω_z = v/P_z it would resolve v only to 2^-52·v·L_z/B_z.
    for (n, v, p, n_a, y) in [
        (0.01, CF2_V, CF2_P, CF2_N_A, CF2_Y),
        (0.001, CF3_V, CF3_P, CF3_N_A, CF3_Y),
    ] {
        let (_, eq) = checked_1f(ces(g1_parcels(n, 0.6, 3.0), 0.5));
        let b = &eq.base.base;
        assert_eq!(b.margin, Margin::Wall);
        assert_eq!(eq.base.land_market, LandMarket::Scarce);
        for (name, got, want) in [
            ("v", b.v, v),
            ("P", eq.basket.price, p),
            ("N_a", b.n_a, n_a),
            ("Y", b.y, y),
        ] {
            close(&format!("N {n}: {name}"), got, want);
        }
    }
}

#[test]
fn a_steep_basket_with_a_small_weight() {
    // §5.1 step 2, §5.5, §14 item 18: W1 under Ces { sigma: 20 } with eq 26's weights, space's
    // 0.3^20 = 3.5e-11 against the good's 0.7^20 = 8.0e-4. At the equilibrium's prices 1 + S =
    // Σ_j s̄_j·exp(x_j) is 1.5e-7, which the ln1p form reaches by cancellation; the direct sum
    // keeps the digits of P, the content and the shares.
    let (_, eq) = checked_1f(ces(g1_parcels(4.0, 0.6, 1.0), 20.0));
    let b = &eq.base.base;
    assert_eq!(b.margin, Margin::Contestable);
    for (name, got, want) in [
        ("x*", b.x_star, CS_X_STAR),
        ("v", b.v, CS_V),
        ("P", eq.basket.price, CS_P),
        ("N_a", b.n_a, CS_N_A),
        ("Y", b.y, CS_Y),
        ("space's share", eq.basket.shares[1], CS_SPACE_SHARE),
        ("c_good", eq.basket.content[0], CS_GOOD_CONTENT),
        ("c_space", eq.basket.content[1], CS_SPACE_CONTENT),
    ] {
        close(&format!("CS {name}"), got, want);
    }
}

#[test]
fn the_free_end_with_transfers_and_types() {
    // §2.12, §5.3 step 3, §14 item 5: f_∞ = −S_∞ at a free category's end, S_∞ the supply at
    // ω_∞ = 1/(Z·M(λ̃)) (σ < 1) with the transfer's limit. Under Dividend W, R and C over P vanish
    // there and d/P^c → −μ, so a uniform program cancels against d: CW's f_∞ with and without one.
    for g in [
        dividend(0.5),
        government(|g| {
            g.budget = Budget::Dividend { rent_tax: 0.5 };
            g.program = Program {
                work: 0.2,
                exit: 0.2,
            };
        }),
    ] {
        let mut params = ces(g1_parcels(4.0, 0.6, 3.0), 0.5);
        params.government = g;
        let (_, eq) = checked_1f(params);
        assert_eq!(eq.base.base.margin, Margin::Wall);
        close(
            "CW's f_∞ under Dividend",
            eq.base.f_end.unwrap(),
            CW05_F_END,
        );
    }
    // Two types of efficiency 1 and 2 under RentRate with every instrument: S_∞ = Σ_i ε_i·N_i·
    // F_i(ln1p(ratio_i(ω_∞))), ratio_i = ((μ_w − μ_e) + (1 − τ_w)·ε_i·ω_∞/(1 + t_c))/(ν_i + d̂ +
    // μ_e), with M(λ̃) the power mean of the end technique's labour totals, written here.
    let mut p = g1_parcels(4.0, 0.6, 3.0);
    p.worker_types = vec![worker(2.0, 3.0, 1.0, 1.0), worker(2.0, 3.0, 2.0, 1.0)];
    p.reserved = vec![vec![0.0, 0.0]; 2];
    p.exits = vec![oracle::ExitForm::Dependence; 2];
    let sigma = 0.5;
    let mut params = ces(p, sigma);
    params.government = government(|g| {
        g.payroll = 0.1;
        g.consumption = 0.2;
        g.budget = Budget::RentRate { dividend: 0.3 };
        g.program = Program {
            work: 0.05,
            exit: 0.1,
        };
    });
    let g = params.government;
    let (_, eq) = checked_1f(params.clone());
    let b = &eq.base.base;
    assert_eq!(b.margin, Margin::Wall);
    let z: Vec<f64> = params.economy.categories.iter().map(|c| c.weight).collect();
    let total: f64 = z.iter().sum();
    let mut sum = 0.0;
    for (j, cat) in b.categories.iter().enumerate() {
        if z[j] > 0.0 && cat.lambda_tilde > 0.0 {
            sum += (z[j] / total) * num::pow(cat.lambda_tilde, 1.0 - sigma);
        }
    }
    assert_eq!(
        b.categories[1].lambda_tilde, 0.0,
        "space is free at the end"
    );
    let omega = 1.0 / (total * num::pow(sum, 1.0 / (1.0 - sigma)));
    let dividend = match g.budget {
        Budget::RentRate { dividend } => dividend,
        Budget::Dividend { .. } => unreachable!(),
    };
    let (mut s, mut unweighted) = (0.0, 0.0);
    for t in &params.economy.worker_types {
        let ratio = ((g.program.work - g.program.exit)
            + (1.0 - g.payroll) * t.efficiency * omega / (1.0 + g.consumption))
            / ((t.support + dividend) + g.program.exit);
        let n = t.workers * t.work_cost.cdf(num::ln1p(ratio));
        s += t.efficiency * n;
        unweighted += n;
    }
    close("f_∞ = −S_∞ with two types", eq.base.f_end.unwrap(), -s);
    assert!(
        (s - unweighted).abs() > 1e-3 * s,
        "the efficiency weighs S_∞"
    );
}
