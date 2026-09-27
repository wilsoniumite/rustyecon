//! Fixtures and checks shared by unit 1f's tests (docs/unit-1f.md §8).

// The identities are sums over categories and worker types, as docs/unit-1f.md §4.8 writes them.
#![allow(clippy::needless_range_loop)]

use std::collections::BTreeMap;

use oracle::{
    Basket, Budget, Eq1f, Government, HouseholdEconomy, HouseholdParams, HouseholdPoint,
    LandMarket, Margin, Output1b, ParcelParams, PowerSchedule, Program, Regime, TransferMode,
    WorkerType,
};
use rustyecon_core::num;

use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;

// ------------------------------------------------------------------------------ governments

/// A government built from [`Government::none`] by `f`.
pub fn government(f: impl FnOnce(&mut Government)) -> Government {
    let mut g = Government::none();
    f(&mut g);
    g
}

/// τ_w alone.
pub fn payroll(rate: f64) -> Government {
    government(|g| g.payroll = rate)
}

/// t_c alone.
pub fn consumption(rate: f64) -> Government {
    government(|g| g.consumption = rate)
}

/// RentRate with d̂ composites per person.
pub fn rent_rate(dividend: f64) -> Government {
    government(|g| g.budget = Budget::RentRate { dividend })
}

/// Dividend with τ_R.
pub fn dividend(rent_tax: f64) -> Government {
    government(|g| g.budget = Budget::Dividend { rent_tax })
}

/// The program (μ_w, μ_e) alone.
pub fn program(work: f64, exit: f64) -> Government {
    government(|g| g.program = Program { work, exit })
}

/// `g` in Replace mode.
pub fn replacing(mut g: Government) -> Government {
    g.mode = TransferMode::Replace;
    g
}

// ------------------------------------------------------------------------------ economies

/// A unit-1e economy with a fixed basket and a government.
pub fn household(economy: ParcelParams, government: Government) -> HouseholdParams {
    HouseholdParams {
        economy,
        basket: Basket::Fixed,
        government,
    }
}

/// G1 (Appendix B) in parcel form with N, λ and χ_max changed.
pub fn g1_parcels(workers: f64, lam: f64, chi_max: f64) -> ParcelParams {
    goodspace_1e(workers, 10.0, 1.0, lam, chi_max)
}

/// G1 with a government (docs/unit-1f.md §3.3, G).
pub fn g1_with(government: Government) -> HouseholdParams {
    household(g1_parcels(4.0, 0.05, 1.0), government)
}

/// TX (docs/unit-1f.md §3.3): N 4, T `land`, h 1, a 0.5, λ, b 0.2, γ = 2 + 2x, χ_max 1/8, ν 1/4.
pub fn tx_parcels(lam: f64, land: f64) -> ParcelParams {
    let mut p = from_1a_1e(oracle::Params {
        workers: 4.0,
        land,
        a: 0.5,
        lam,
        b: 0.2,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 2.0,
            g1: 2.0,
            k: 1.0,
        },
        work_cost: oracle::UniformWorkCost { chi_max: 0.125 },
        ..appendix_b()
    });
    p.worker_types[0].support = 0.25;
    p
}

/// TX with a government.
pub fn tx(government: Government) -> HouseholdParams {
    household(tx_parcels(0.1, 8.0), government)
}

/// AP (docs/unit-1f.md §3.3): G1 with λ 0 and γ = η(1 + x).
pub fn ap_parcels(eta: f64) -> ParcelParams {
    from_1a_1e(oracle::Params {
        lam: 0.0,
        schedule: PowerSchedule {
            eta,
            g0: 1.0,
            g1: 1.0,
            k: 1.0,
        },
        ..appendix_b()
    })
}

/// z = ((1 − α)^σ, α^σ) over (good, space): space's share is SSRN eq 26's α(q).
pub fn ces_weights(mut p: ParcelParams, alpha: f64, sigma: f64) -> ParcelParams {
    p.categories[0].weight = num::pow(1.0 - alpha, sigma);
    p.categories[1].weight = num::pow(alpha, sigma);
    p
}

/// A CES household on an economy of G1's form, α 0.3 (C1-C3's baskets).
pub fn ces(economy: ParcelParams, sigma: f64) -> HouseholdParams {
    HouseholdParams {
        economy: ces_weights(economy, 0.3, sigma),
        basket: Basket::Ces { sigma },
        government: Government::none(),
    }
}

/// check_pinning's A-joint household (docs/unit-1f.md §2.13): N 1, T 10, a 0.5, λ, b 0.2,
/// γ = 1 + g1·x, z = (0.7, 0.3) over (good, space) under `Ces { sigma: 1 }`, saturated supply
/// (χ_max 1/64).
pub fn a_joint(g1: f64, lam: f64) -> HouseholdParams {
    let mut p = from_1a_1e(oracle::Params {
        workers: 1.0,
        a: 0.5,
        lam,
        b: 0.2,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 1.0,
            g1,
            k: 1.0,
        },
        work_cost: oracle::UniformWorkCost { chi_max: 0.015625 },
        ..appendix_b()
    });
    p.categories[0].weight = 0.7;
    p.categories[1].weight = 0.3;
    HouseholdParams {
        economy: p,
        basket: Basket::Ces { sigma: 1.0 },
        government: Government::none(),
    }
}

/// 1d's E2 (the trained at its wall) in parcel form.
pub fn e2_parcels() -> ParcelParams {
    e0(entrant_trained(8.0, 1.0, 1.0, 1.0))
}

/// ER's government (docs/unit-1f.md §3.3): τ_w 0.1, t_c 0.1, d̂ 0.5, μ_e 0.1.
pub fn er_government() -> Government {
    government(|g| {
        g.payroll = 0.1;
        g.consumption = 0.1;
        g.budget = Budget::RentRate { dividend: 0.5 };
        g.program.exit = 0.1;
    })
}

// ------------------------------------------------------------------------------ solving

/// Validates unit-1f parameters the test knows to be valid.
pub fn economy_1f(params: HouseholdParams) -> HouseholdEconomy {
    HouseholdEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{params:?}: {e}"))
}

/// Solves a unit-1f economy the test knows to have one equilibrium.
pub fn solved_1f(params: HouseholdParams) -> Box<Eq1f> {
    match economy_1f(params.clone()).solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{params:?} has no equilibrium: {other:?}"),
    }
}

/// Solves, checks every identity of §4.8-4.9, and returns the equilibrium and the economy.
pub fn checked_1f(params: HouseholdParams) -> (HouseholdEconomy, Box<Eq1f>) {
    let e = economy_1f(params.clone());
    let eq = solved_1f(params);
    check_identities_1f(&e, &eq);
    (e, eq)
}

/// Every output of a unit-1f equilibrium, by printed key, as bits.
pub fn bits_1f(eq: &Eq1f) -> BTreeMap<String, Option<u64>> {
    eq.outputs()
        .into_iter()
        .map(|(key, output)| {
            let bits = match output {
                Output1b::Float(v) | Output1b::Optional(Some(v)) => Some(v.to_bits()),
                Output1b::Optional(None) => None,
                Output1b::Flag(b) => Some(u64::from(b)),
                Output1b::Count(n) => Some(u64::from(n)),
            };
            (key.to_string(), bits)
        })
        .collect()
}

/// The allocation of an equilibrium (docs/unit-1f.md §4.10 (f)): x*, v, P, every n_S,i, Y, T_m
/// and T_p.
pub fn allocation(eq: &Eq1f) -> Vec<(String, f64)> {
    let b = &eq.base.base;
    let mut out = vec![
        ("x*".to_string(), b.x_star),
        ("v".to_string(), b.v),
        ("P".to_string(), b.p_s),
        ("Y".to_string(), b.y),
        ("T_m".to_string(), eq.base.land.market),
        ("T_p".to_string(), eq.base.land.rented_plots),
    ];
    for (i, w) in b.workers.iter().enumerate() {
        out.push((format!("n_S,{i}"), w.supply));
    }
    out
}

/// Two allocations bit for bit.
pub fn same_allocation_bits(what: &str, a: &Eq1f, b: &Eq1f) {
    for ((k, x), (_, y)) in allocation(a).into_iter().zip(allocation(b)) {
        assert_eq!(x.to_bits(), y.to_bits(), "{what}: {k} {x} vs {y}");
    }
}

/// Two allocations within `tol` relative (absolute below 1e-300).
pub fn same_allocation(what: &str, a: &Eq1f, b: &Eq1f, tol: f64) {
    for ((k, x), (_, y)) in allocation(a).into_iter().zip(allocation(b)) {
        assert!(
            (x - y).abs() <= tol * x.abs().max(y.abs()) + 1e-300,
            "{what}: {k} {x} vs {y}"
        );
    }
}

/// n_S,i by §4.4's point form, written here from the equations: N_i·F_i(ln1p(num/den)), num =
/// (m_w − m_e) + ((1 − τ_w)·v_i − (1 + t_c)·e_i), den = (A_i + m_e) + (1 + t_c)·e_i.
pub fn rule_supply(
    g: &Government,
    t: &WorkerType,
    wage: f64,
    price: f64,
    exit: f64,
    transfer: f64,
) -> f64 {
    let p_c = (1.0 + g.consumption) * price;
    let a = unearned(g, t.support, p_c, transfer);
    let (m_w, m_e) = (g.program.work * p_c, g.program.exit * p_c);
    let home = (1.0 + g.consumption) * exit;
    let num = (m_w - m_e) + ((1.0 - g.payroll) * wage - home);
    let den = (a + m_e) + home;
    t.workers * t.work_cost.cdf(num::ln1p(num / den))
}

/// A_i (§2.4): ν_i·P^c + d, or max(ν_i·P^c, d) in Replace mode.
pub fn unearned(g: &Government, support: f64, p_c: f64, transfer: f64) -> f64 {
    match g.mode {
        TransferMode::Supplement => support * p_c + transfer,
        TransferMode::Replace => (support * p_c).max(transfer),
    }
}

/// The evaluation that priced an equilibrium, through the household economy (§5.1).
pub fn point_of_1f(economy: &HouseholdEconomy, eq: &Eq1f) -> HouseholdPoint {
    let b = &eq.base.base;
    if eq.base.land_market == LandMarket::Idle {
        return match edge_of(b) {
            Some(edge) => economy.at_idle_edge(eq.base.land.market, b.technique, edge),
            None => economy.at_idle(eq.base.land.market, b.technique),
        };
    }
    if let Some(point) = enclosure_point(&eq.base) {
        let share = eq.base.enclosure.map_or(0.0, |t| t.share);
        return economy.at_enclosure(&point, oracle::EnclosureSide::Share(share));
    }
    match b.margin {
        Margin::Contestable => economy.at_with(b.x_star, b.technique),
        _ => economy.at_wage(b.x_star, b.v, b.technique),
    }
}

/// Every identity and bound of docs/unit-1f.md §4.8-4.9 at an equilibrium, recomputed from the
/// reported fields and the parameters (§8): unit 1e's under the participation rule (with 1d's
/// own where there is no government and the basket is fixed); the prices, P, P^c, d, each A_i,
/// each supply and the basket's content pinned to the evaluation that set them bit for bit;
/// the budget, the accounts and the ledger recomputed bit for bit; the income identity with
/// government (I1)-(I4), the receipts, the rent share, coverage, D.3's identity and the
/// corollary's bound to 1e-12; each walled type's wage c_i·P; the CES's Euler identity, its
/// shares and its content.
pub fn check_identities_1f(economy: &HouseholdEconomy, eq: &Eq1f) {
    let params = economy.params();
    let g = &params.government;
    let p = &params.economy;
    let b = &eq.base.base;
    let at = |what: &str| format!("{what} at {:?} {:?}", g, params.basket);
    let fixed = params.basket == Basket::Fixed;
    let kinds = p.worker_types.len();
    let people: f64 = p.worker_types.iter().map(|t| t.workers).sum();
    // Unit 1e's identities, each supply by the rule.
    let rule = |i: usize, wage: f64, exit: f64| {
        rule_supply(
            g,
            &p.worker_types[i],
            wage,
            b.p_s,
            exit,
            eq.government.dividend,
        )
    };
    if fixed && *g == Government::none() {
        check_identities_1e(economy.parcels(), &eq.base);
    } else {
        check_identities_1e_under(economy.parcels(), &eq.base, Some(&rule));
    }
    // The point, bit for bit.
    let q = point_of_1f(economy, eq);
    let edge_on_scarce_land = eq.base.land_market == LandMarket::Scarce && edge_of(b).is_some();
    let no_reserved = p.reserved.iter().flatten().all(|&r| r == 0.0);
    let pinned = !edge_on_scarce_land && (b.tie.is_none() || no_reserved);
    let price = eq.basket.price;
    assert_eq!(price.to_bits(), b.p_s.to_bits(), "{}", at("P"));
    let p_c = (1.0 + g.consumption) * price;
    assert_eq!(
        eq.basket.consumer_price.to_bits(),
        p_c.to_bits(),
        "{}",
        at("P^c")
    );
    assert_eq!(
        q.content.iter().map(|c| c.to_bits()).collect::<Vec<_>>(),
        eq.basket
            .content
            .iter()
            .map(|c| c.to_bits())
            .collect::<Vec<_>>(),
        "{}",
        at("content")
    );
    if pinned {
        assert_eq!(
            q.price.to_bits(),
            price.to_bits(),
            "{}",
            at("the point's P")
        );
        assert_eq!(
            q.consumer_price.to_bits(),
            p_c.to_bits(),
            "{}",
            at("the point's P^c")
        );
        assert_eq!(
            q.transfer.to_bits(),
            eq.government.dividend.to_bits(),
            "{}",
            at("d")
        );
        for i in 0..kinds {
            assert_eq!(
                q.unearned[i].to_bits(),
                eq.accounts.workers[i].unearned.to_bits(),
                "{}",
                at("A_i")
            );
        }
    }
    // The transfer (§4.3): RentRate's d̂·P^c exactly.
    let d = eq.government.dividend;
    if let Budget::RentRate { dividend } = g.budget {
        assert_eq!(
            d.to_bits(),
            (dividend * p_c).to_bits(),
            "{}",
            at("d = d̂ P^c")
        );
    }
    assert_eq!(
        eq.government.dividend_composites.to_bits(),
        (d / p_c).to_bits()
    );
    // The budget (§4.6), recomputed in the report's order.
    let gv = &eq.government;
    let wages = b.wage_bill;
    let rent = eq.base.rent * eq.base.land.market;
    let consumption_value = b.y * price;
    let (m_w, m_e) = (g.program.work * p_c, g.program.exit * p_c);
    assert_eq!(
        (gv.program_work.to_bits(), gv.program_exit.to_bits()),
        (m_w.to_bits(), m_e.to_bits())
    );
    let (mut working, mut exiting) = (0.0, 0.0);
    for i in 0..kinds {
        working += b.workers[i].supply;
        exiting += eq.base.workers[i].exiters;
    }
    let cost = m_w * working + m_e * exiting;
    assert_eq!(gv.program_cost.to_bits(), cost.to_bits(), "{}", at("M"));
    assert_eq!(gv.revenue_payroll.to_bits(), (g.payroll * wages).to_bits());
    assert_eq!(
        gv.revenue_consumption.to_bits(),
        (g.consumption * consumption_value).to_bits()
    );
    assert_eq!(gv.transfers.to_bits(), (people * d).to_bits());
    match g.budget {
        Budget::RentRate { .. } => {
            let levy = ((gv.transfers + cost) - gv.revenue_payroll) - gv.revenue_consumption;
            assert_eq!(
                gv.levy.map(f64::to_bits),
                Some(levy.to_bits()),
                "{}",
                at("L_R")
            );
            assert_eq!(gv.revenue_rent.to_bits(), levy.to_bits());
            assert_eq!(
                gv.rent_tax.map(f64::to_bits),
                (rent > 0.0).then(|| (levy / rent).to_bits()),
                "{}",
                at("tau_R = L_R/R")
            );
            assert_eq!(gv.within_rent, Some((0.0..=rent).contains(&levy)));
        }
        Budget::Dividend { rent_tax } => {
            assert_eq!((gv.levy, gv.within_rent), (None, None));
            assert_eq!(gv.rent_tax, Some(rent_tax));
            assert_eq!(gv.revenue_rent.to_bits(), (rent_tax * rent).to_bits());
        }
    }
    assert_eq!(
        (
            gv.consumption_wage_leg.to_bits(),
            gv.consumption_rent_leg.to_bits(),
            gv.consumption_interest_leg.to_bits()
        ),
        (
            (g.consumption * wages).to_bits(),
            (g.consumption * rent).to_bits(),
            (g.consumption * b.interest).to_bits()
        )
    );
    // (I2) the budget balances.
    let revenue = (gv.revenue_payroll + gv.revenue_rent) + gv.revenue_consumption;
    near(
        &at("(I2) the budget"),
        revenue / consumption_value,
        (gv.transfers + cost) / consumption_value,
        1e-12,
    );
    // The accounts (§4.7).
    let mut spending = 0.0;
    let mut support = 0.0;
    for (i, t) in p.worker_types.iter().enumerate() {
        let a = &eq.accounts.workers[i];
        let want_a = unearned(g, t.support, p_c, d);
        assert_eq!(a.unearned.to_bits(), want_a.to_bits(), "{}", at("A_i"));
        let own = match g.mode {
            TransferMode::Supplement => t.support * p_c,
            TransferMode::Replace => (t.support * p_c - d).max(0.0),
        };
        assert_eq!(a.support.to_bits(), own.to_bits(), "{}", at("s_i"));
        assert_eq!(
            a.net_wage.to_bits(),
            ((1.0 - g.payroll) * b.workers[i].wage).to_bits()
        );
        assert_eq!(a.working.to_bits(), b.workers[i].supply.to_bits());
        assert_eq!(a.exiting.to_bits(), eq.base.workers[i].exiters.to_bits());
        // A_i − s_i = d in both modes (§2.4).
        near(
            &at("A - s = d"),
            a.unearned - a.support,
            d,
            1e-12 * a.unearned,
        );
        let spent = a.working * ((a.unearned + m_w) + a.net_wage) + a.exiting * (a.unearned + m_e);
        assert_eq!(a.spending.to_bits(), spent.to_bits());
        assert_eq!(a.composites.to_bits(), (spent / p_c).to_bits());
        spending += spent;
        support += t.workers * own;
        // Each pooled type's supply at the equilibrium, by the rule.
        if b.workers[i].pooled {
            close_to(
                &at("n_S,i by the rule"),
                b.workers[i].supply,
                rule_supply(
                    g,
                    t,
                    b.workers[i].wage,
                    price,
                    eq.base.workers[i].exit_value,
                    d,
                ),
                1e-12,
            );
        }
    }
    let provider = &eq.accounts.provider;
    let receipts = (rent - gv.revenue_rent) + b.interest;
    assert_eq!(provider.receipts.to_bits(), receipts.to_bits());
    assert_eq!(provider.support.to_bits(), support.to_bits());
    assert_eq!(provider.spending.to_bits(), (receipts - support).to_bits());
    assert_eq!(provider.funded, provider.composites > 0.0);
    spending += provider.spending;
    assert_eq!(eq.accounts.spending.to_bits(), spending.to_bits());
    // (I1) income, (I3) spending, (I4) composites, and the receipts (Prop 6 (iii)).
    close(&at("(I1) I = Y P"), b.income, consumption_value);
    close(
        &at("(I1) I = W + R + interest"),
        b.income,
        wages + rent + b.interest,
    );
    close(
        &at("(I3) spending = (1 + t_c) Y P"),
        spending,
        (1.0 + g.consumption) * consumption_value,
    );
    close(&at("(I4) composites = Y"), eq.accounts.composites, b.y);
    close(
        &at("receipts"),
        (1.0 - g.payroll) * wages + (rent - gv.revenue_rent) + b.interest + gv.transfers + cost,
        wages + rent + b.interest + gv.revenue_consumption,
    );
    // The ledger (§4.9) and the rent share: R = φ^q_r·C (T5).
    let l = &eq.ledger;
    let mut reserved_bill = 0.0;
    for (i, r) in economy
        .parcels()
        .workers()
        .reserved_per_basket()
        .iter()
        .enumerate()
    {
        reserved_bill += b.workers[i].wage * r;
    }
    assert_eq!(
        l.basket_wage_share.to_bits(),
        ((b.v * b.l_s + reserved_bill) / price).to_bits()
    );
    let flow = b.types.iter().all(|t| t.user_cost == 1.0);
    assert_eq!(
        l.basket_rent_share.map(f64::to_bits),
        flow.then(|| ((eq.base.rent * b.b_s) / price).to_bits())
    );
    if let Some(r) = l.basket_rent_share {
        close(
            &at("phi_w + phi_r of the basket"),
            l.basket_wage_share + r,
            1.0,
        );
    }
    let rent_share = (eq.base.rent * b.b_s_q) / price;
    assert_eq!(l.rent_share.to_bits(), rent_share.to_bits());
    if rent > 0.0 {
        close(&at("R = phi^q_r C"), rent, rent_share * consumption_value);
        let tau = gv.revenue_rent / rent;
        assert_eq!(
            l.multiplier.map(f64::to_bits),
            Some((1.0 / (1.0 - rent_share * tau)).to_bits())
        );
        if (0.0..=1.0).contains(&tau) {
            close(
                &at("R = R_0 multiplier"),
                l.recirculated_base * l.multiplier.unwrap(),
                rent,
            );
        }
    } else {
        assert_eq!(l.multiplier, None);
    }
    // Coverage (SSRN eq 16), D.3's identity and the corollary's bound (SSRN p.31).
    let t = economy.parcels().enclosed_land();
    assert_eq!(
        gv.rate_for_one_composite.map(f64::to_bits),
        (eq.base.coverage > 0.0).then(|| (1.0 / eq.base.coverage).to_bits())
    );
    if eq.base.rent == 1.0 && no_reserved {
        close(
            &at("D.3: kappa = T/(N (L_s v + B_s))"),
            eq.base.coverage,
            t / (people * (b.l_s * b.v + b.b_s)),
        );
        at_most_rel(
            &at("the corollary's bound"),
            wages / (people * price),
            b.v * (b.n_pool / people) / b.b_s,
        );
    }
    // The basket (§4.2): Euler, and a CES basket's shares and content.
    let mut euler = 0.0;
    for (c, cat) in eq.basket.content.iter().zip(&b.categories) {
        euler += c * cat.price;
    }
    close(&at("Euler"), euler, price);
    for (j, cat) in b.categories.iter().enumerate() {
        assert_eq!(eq.basket.shares[j].to_bits(), cat.share.to_bits());
    }
    if let Basket::Ces { sigma } = params.basket {
        let z: Vec<f64> = p.categories.iter().map(|c| c.weight).collect();
        let mut den = 0.0;
        for (j, cat) in b.categories.iter().enumerate() {
            if z[j] > 0.0 {
                den += z[j] * num::pow(cat.price, 1.0 - sigma);
            }
        }
        let total: f64 = z.iter().sum();
        let m = price / total;
        for (j, cat) in b.categories.iter().enumerate() {
            if z[j] > 0.0 {
                close(
                    &at("the CES share"),
                    cat.share,
                    z[j] * num::pow(cat.price, 1.0 - sigma) / den,
                );
                close(
                    &at("the CES content"),
                    eq.basket.content[j],
                    z[j] * num::pow(cat.price / m, -sigma),
                );
            } else {
                assert_eq!(eq.basket.content[j], 0.0);
            }
        }
        assert_eq!(eq.basket.sigma, Some(sigma));
    } else {
        for (j, c) in p.categories.iter().enumerate() {
            assert_eq!(eq.basket.content[j].to_bits(), c.weight.to_bits());
        }
        assert_eq!(eq.basket.sigma, None);
    }
    // Walled types: v_i = c_i·P (§4.5), c_i = ((â_i + μ_e)·ζ_i − (μ_w − μ_e))·(1 + t_c)/(1 − τ_w).
    for (i, w) in b.workers.iter().enumerate() {
        if !w.pooled && !w.edge {
            let t = &p.worker_types[i];
            let hat = match (g.mode, g.budget) {
                (TransferMode::Supplement, Budget::RentRate { dividend }) => t.support + dividend,
                (TransferMode::Replace, Budget::RentRate { dividend }) => t.support.max(dividend),
                _ => panic!("a walled type under the Dividend closure"),
            };
            let rate = ((hat + g.program.exit) * w.clearing_real_wage
                - (g.program.work - g.program.exit))
                * (1.0 + g.consumption)
                / (1.0 - g.payroll);
            close(&at("the walled wage c_i P"), w.wage, rate * price);
        }
    }
    // The new residuals, recomputed, each within 1e-12.
    let r = &eq.residuals;
    assert_eq!(
        r.budget.to_bits(),
        ((revenue - (gv.transfers + cost)).abs() / consumption_value).to_bits()
    );
    let gross = (1.0 + g.consumption) * consumption_value;
    assert_eq!(
        r.spending.to_bits(),
        ((spending - gross).abs() / gross).to_bits()
    );
    assert_eq!(
        r.composites.to_bits(),
        ((eq.accounts.composites - b.y).abs() / b.y).to_bits()
    );
    assert_eq!(r.euler.to_bits(), ((euler - price).abs() / price).to_bits());
    for (name, value) in [
        ("budget", r.budget),
        ("spending", r.spending),
        ("composites", r.composites),
        ("rent share", r.rent_share),
        ("euler", r.euler),
    ] {
        assert!(value <= FULL, "{name} residual {value:e} {}", at(""));
    }
    // A type on the Plot branch has every exiter on a plot, the trial's supply the point's
    // (§5.1 step 5: the exit sub-problem reads the point's transfer).
    if eq.base.enclosure.is_none() {
        for (i, w) in eq.base.workers.iter().enumerate() {
            if w.branch == oracle::Branch::Plot {
                near(
                    &at("plot households = exiters"),
                    w.plot_households,
                    w.exiters,
                    1e-12 * p.worker_types[i].workers,
                );
            }
        }
    }
    // f_0 (§2.9).
    let start = economy.start().unwrap();
    assert_eq!(eq.f_start, start.is_finite().then_some(start));
    assert!(start > 0.0, "{}", at("an equilibrium's start is positive"));
}

/// got ≤ want·(1 + 1e-12).
pub fn at_most_rel(what: &str, got: f64, want: f64) {
    assert!(got <= want * (1.0 + 1e-12), "{what}: {got:e} > {want:e}");
}

/// The net real wage κ_w·v/P of an equilibrium (docs/unit-1f.md §3.1).
pub fn omega_net(g: &Government, eq: &Eq1f) -> f64 {
    ((1.0 - g.payroll) / (1.0 + g.consumption)) * (eq.base.base.v / eq.base.base.p_s)
}
