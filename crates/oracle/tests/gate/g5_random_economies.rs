//! G5: identities and single crossing at random interior economies, as in laborformal's
//! check_macro.py I1-I2 (:45-60) and A1-A3 (:108-133) at 31b3482.

use oracle::{
    closure, Economy, Eq1a, Params, Point, PowerSchedule, Regime, Residuals, Schedule,
    UniformWorkCost, BRACKET_LO,
};
use rustyecon_core::num;

use crate::support::*;

/// SplitMix64 (Steele, Lea and Flood, OOPSLA 2014): a fixed, seedable generator
/// written out here, so the draws are the same on every platform and need no crate.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform on [lo, hi), from the top 53 bits.
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        let unit = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + (hi - lo) * unit
    }
}

/// Seed of the first set; the others follow it. The number is check_macro.py's
/// (:46, `default_rng(923)`), though the generator differs.
const SEED: u64 = 923;

/// Interior economies per set: at least 60, as the spec and check_macro.py:48,111 ask.
const WANTED: usize = 60;

/// Draws allowed per set before the test fails. Most draws from these ranges are
/// interior, so a set needs far fewer.
const MAX_DRAWS: usize = 10_000;

/// Intervals in the grid on which single crossing is checked.
const GRID: usize = 100;

#[derive(Clone, Copy, Debug)]
enum Set {
    /// (ρ, δ, J_b) = (0, 1, 1), check_macro.py I (:49-51).
    Flow,
    /// ρ ~ U(0, 0.12), δ ~ U(0.03, 1), J_b = 1, check_macro.py A (:112-114).
    Durable,
    /// As `Durable`, with J_b drawn from 1..=5.
    BuildLag,
}

/// Build lags drawn in the `BuildLag` set: 1..=MAX_LAG.
const MAX_LAG: u64 = 5;

fn draw(rng: &mut SplitMix64, set: Set) -> Params {
    // Ranges: docs/unit-1a.md §6 G5. Fields are evaluated in the order written.
    let base = Params {
        workers: rng.uniform(1.0, 5.0),
        land: rng.uniform(2.0, 20.0),
        space: rng.uniform(0.1, 2.0),
        a: rng.uniform(0.05, 0.5),
        b: rng.uniform(0.1, 1.0),
        lam: rng.uniform(0.0, 0.1),
        schedule: PowerSchedule {
            eta: 1.0,
            g0: rng.uniform(0.01, 0.5),
            g1: rng.uniform(0.2, 1.5),
            k: rng.uniform(0.5, 6.0),
        },
        work_cost: UniformWorkCost {
            chi_max: rng.uniform(0.5, 3.0),
        },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    };
    match set {
        Set::Flow => base,
        Set::Durable => Params {
            rho: rng.uniform(0.0, 0.12),
            delta: rng.uniform(0.03, 1.0),
            ..base
        },
        Set::BuildLag => Params {
            rho: rng.uniform(0.0, 0.12),
            delta: rng.uniform(0.03, 1.0),
            build_lag: 1 + (rng.next_u64() % MAX_LAG) as u32,
            ..base
        },
    }
}

struct Sample {
    params: Params,
    economy: Economy,
    eq: Box<Eq1a>,
}

/// The first WANTED interior economies of a set, skipping the other regimes.
fn sample(set: Set, seed: u64) -> Vec<Sample> {
    let mut rng = SplitMix64(seed);
    let mut out = Vec::with_capacity(WANTED);
    let mut draws = 0;
    while out.len() < WANTED {
        assert!(
            draws < MAX_DRAWS,
            "{set:?}: only {} interior in {draws} draws",
            out.len()
        );
        draws += 1;
        let params = draw(&mut rng, set);
        let economy = Economy::new(params.clone()).expect("the ranges are valid");
        match economy.solve() {
            Ok(Regime::Interior(eq)) => out.push(Sample {
                params,
                economy,
                eq,
            }),
            Ok(_) => {}
            Err(e) => panic!("{params:?}: {e}"),
        }
    }
    out
}

/// Every identity of spec §3.2-3.4, recomputed here from the reported fields.
fn check_identities(s: &Sample) {
    let (p, eq) = (&s.params, &s.eq);
    let at = |what: &str| format!("{what} at {p:?}");
    // Income: Y P_s = v N_a + T + interest (SSRN App. C, p.30; check_macro I1 :57, A2 :122).
    near(
        &at("income"),
        eq.y * eq.p_s,
        eq.v * eq.n_a + p.land + eq.interest,
        FULL * eq.income,
    );
    // Land: h Y + b delta K = T (I2 :58, A2 :123).
    near(
        &at("land"),
        p.space * eq.y + p.b * p.delta * eq.k,
        p.land,
        FULL * p.land,
    );
    // Machine services: K = Y J + a delta K.
    near(
        &at("services"),
        eq.y * eq.j_star + p.a * p.delta * eq.k,
        eq.k,
        FULL * eq.k,
    );
    // The user-cost price: p_m = u (a p_m + lambda v + b) (A1 :120-121), V_m the build cost.
    let build_cost = p.a * eq.p_m + p.lam * eq.v + p.b;
    close(&at("user cost"), eq.u * build_cost, eq.p_m);
    close(&at("V_m"), build_cost, eq.v_m);
    // The task margin and zero profit (SSRN eq 21-22).
    close(&at("v = gamma p_m"), eq.gamma_star * eq.p_m, eq.v);
    close(
        &at("p"),
        eq.v * (1.0 - eq.x_star) + eq.p_m * eq.j_star,
        eq.p,
    );
    // Hours clear: N_a = Y (1 - x) + lambda delta K, with 1 - x* as the solve carries it.
    close(
        &at("hours"),
        eq.y * eq.one_minus_x_star + p.lam * p.delta * eq.k,
        eq.n_a,
    );
    close(
        &at("final hours"),
        eq.y * eq.one_minus_x_star,
        eq.final_hours,
    );
    // 1 - x* lies between 1 - hi and 1 - lo for the adjacent doubles lo, hi around x*,
    // and 1.0 - x_star is itself rounded to the doubles near 1 - x*: the two differ by at
    // most the spacing of doubles at x* plus the spacing at 1 - x*.
    let spacing = |v: f64| (v - v.next_down()).max(v.next_up() - v);
    assert!(
        (eq.one_minus_x_star - (1.0 - eq.x_star)).abs()
            <= spacing(eq.x_star) + spacing(1.0 - eq.x_star),
        "{}",
        at("1 - x*")
    );
    // The income shares: labour, capital and land exhaust income.
    close(
        &at("labour share"),
        eq.v * eq.n_a / eq.income,
        eq.labor_share,
    );
    close(
        &at("capital share"),
        eq.interest / eq.income,
        eq.capital_share,
    );
    close(
        &at("shares"),
        eq.labor_share + eq.capital_share + p.land / eq.income,
        1.0,
    );
    // The real wage and the cost of support, at this economy's h.
    close(&at("w / P_s"), eq.v / eq.p_s, eq.real_wage);
    close(&at("P_s = p + h"), eq.p + p.space, eq.p_s);
    close(&at("N P_s"), p.workers * eq.p_s, eq.support_cost);
    close(
        &at("worker baskets"),
        p.workers + eq.v * eq.n_a / eq.p_s,
        eq.worker_baskets,
    );
    // Interest is rho times machine wealth (check_dynamics.py L2, :164-169).
    if p.rho > 0.0 {
        let carry = num::pow(1.0 + p.rho, f64::from(p.build_lag - 1));
        let wealth = eq.v_m * eq.k * (carry + p.delta * (carry - 1.0) / p.rho);
        near(
            &at("interest"),
            eq.interest,
            p.rho * wealth,
            FULL * eq.income,
        );
    } else {
        assert_eq!(eq.interest, 0.0, "{}", at("interest"));
    }
    // Goods clear by Walras' law: worker and provider baskets sum to Y (SSRN p.30).
    close(
        &at("baskets"),
        eq.worker_baskets + eq.provider_baskets,
        eq.y,
    );
    // Funding: the provider has baskets of its own exactly when T + interest covers N P_s.
    assert_eq!(eq.funded, eq.provider_baskets > 0.0, "{}", at("funded"));
    // Participation is N_a / N, a share: never above 1 (Eq1a::participation).
    assert!(eq.participation <= 1.0, "{}", at("participation"));
    assert_eq!(
        eq.participation,
        (eq.n_a / p.workers).min(1.0),
        "{}",
        at("participation")
    );
    // The solver's own residuals, and the root.
    let r = eq.residuals;
    for (name, value) in [
        ("income", r.income),
        ("land", r.land),
        ("services", r.services),
        ("user cost", r.user_cost),
        ("labour / N_a", r.labor / eq.n_a),
    ] {
        assert!(value <= FULL, "{name} residual {value:e} {}", at(""));
    }
    assert_eq!(recomputed(s), r, "{}", at("residuals"));
    assert_root(&s.economy, eq.x_star);
    // The price block alone (spec §3.6) reproduces the equilibrium's prices.
    let c = closure(p.a, p.lam, eq.gamma_star, p.b, 1.0, eq.u).expect("viable at x*");
    close(&at("closure p_m"), c.p_m, eq.p_m);
    close(&at("closure w"), c.w, eq.v);
}

/// The residuals of spec §4 step 5, recomputed from the reported fields in the spec's
/// form. The same operations on the same doubles, so they must agree bit for bit.
fn recomputed(s: &Sample) -> Residuals {
    let (p, eq) = (&s.params, &s.eq);
    Residuals {
        // Demand at the reported equilibrium against supply at the reported prices.
        labor: (eq.n_a - s.economy.at(eq.x_star).n_s).abs(),
        income: (eq.y * eq.p_s - eq.income).abs() / eq.income,
        land: (p.space * eq.y + p.b * p.delta * eq.k - p.land).abs() / p.land,
        services: (eq.k - eq.y * eq.j_star - p.a * p.delta * eq.k).abs() / eq.k,
        user_cost: (eq.p_m - eq.u * (p.a * eq.p_m + p.lam * eq.v + p.b)).abs() / eq.p_m,
    }
}

/// Spec §3.5 at the flow benchmark, and check_macro.py A3 (:124-126).
fn check_flow(s: &Sample) {
    let (p, eq) = (&s.params, &s.eq);
    let cs = eq.cost_system.expect("u = 1");
    close("P_s = w L_s + r B_s", eq.v * cs.l_s + cs.b_s, eq.p_s);
    close("Y = T/B_s", p.land / cs.b_s, eq.y);
    close("n_D = T L_s/B_s", p.land * cs.l_s / cs.b_s, eq.n_a);
    close(
        "phi_w = w lt_m / p_m",
        eq.v * cs.lambda_tilde[1] / eq.p_m,
        eq.phi_w.expect("u = 1"),
    );
    close(
        "phi_r = r bt_m / p_m",
        cs.b_tilde[1] / eq.p_m,
        eq.phi_r.expect("u = 1"),
    );
    let gamma = p.schedule.gamma(0.6);
    close(
        "A3 nesting",
        s.economy.at(0.6).p_m,
        p.b / (1.0 - p.a - p.lam * gamma),
    );
}

/// n_D strictly decreases, v/P_s strictly increases, n_S never falls, and f = n_D − n_S
/// changes sign exactly once, on the grid {BRACKET_LO, 1/GRID, 2/GRID, ..., 1}
/// (SSRN Lemma B.1's proof, p.29; spec §4 step 4).
fn check_single_crossing(s: &Sample) {
    let grid: Vec<Point> = std::iter::once(BRACKET_LO)
        .chain((1..=GRID).map(|i| i as f64 / GRID as f64))
        .map(|x| s.economy.at(x))
        .collect();
    for w in grid.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        assert!(
            b.n_d < a.n_d,
            "n_D rises from x = {} to {} at {:?}",
            a.x,
            b.x,
            s.params
        );
        assert!(
            b.v / b.p_s > a.v / a.p_s,
            "v/P_s falls at x = {} at {:?}",
            b.x,
            s.params
        );
        assert!(b.n_s >= a.n_s, "n_S falls at x = {} at {:?}", b.x, s.params);
    }
    let positive: Vec<bool> = grid.iter().map(|q| q.excess_demand() > 0.0).collect();
    assert!(positive[0] && !positive[GRID], "{:?}", s.params);
    let crossings = positive.windows(2).filter(|w| w[0] != w[1]).count();
    assert_eq!(crossings, 1, "{:?}", s.params);
}

#[test]
fn flow_benchmark() {
    for s in sample(Set::Flow, SEED) {
        assert_eq!(s.eq.u, 1.0);
        check_identities(&s);
        check_flow(&s);
        check_single_crossing(&s);
    }
}

#[test]
fn durability_and_interest() {
    for s in sample(Set::Durable, SEED + 1) {
        check_identities(&s);
        check_single_crossing(&s);
    }
}

#[test]
fn build_lags() {
    let samples = sample(Set::BuildLag, SEED + 2);
    for lag in 1..=MAX_LAG as u32 {
        assert!(
            samples.iter().any(|s| s.params.build_lag == lag),
            "no draw with J_b = {lag}"
        );
    }
    for s in &samples {
        check_identities(s);
        check_single_crossing(s);
    }
}

#[test]
fn residuals_are_computed() {
    // check_identities compares every residual with its recomputation. That has teeth
    // only where a residual is not exactly 0, so require each to be nonzero somewhere in
    // the three sets; a residual that silently read 0 would fail here.
    let samples: Vec<Sample> = [
        (Set::Flow, SEED),
        (Set::Durable, SEED + 1),
        (Set::BuildLag, SEED + 2),
    ]
    .into_iter()
    .flat_map(|(set, seed)| sample(set, seed))
    .collect();
    let nonzero = |pick: fn(&Residuals) -> f64| {
        samples
            .iter()
            .filter(|s| pick(&s.eq.residuals) > 0.0)
            .count()
    };
    for (name, count) in [
        ("labour", nonzero(|r| r.labor)),
        ("income", nonzero(|r| r.income)),
        ("land", nonzero(|r| r.land)),
        ("services", nonzero(|r| r.services)),
        ("user cost", nonzero(|r| r.user_cost)),
    ] {
        assert!(
            count > 0,
            "the {name} residual is 0 in every one of {} economies",
            samples.len()
        );
    }
}

#[test]
fn participation_is_capped_where_supply_saturates() {
    // A draw from the 2026-09-25 review (R321, J_b = 5). Supply is saturated at the root
    // (F = 1, n_S = N), and n_D(x*) can land a few ulps above N; participation stays at 1.
    let params = Params {
        workers: 2.2310703064623203,
        land: 12.534605676139824,
        space: 0.3247530084073593,
        a: 0.06616273179304444,
        lam: 0.09729595503340215,
        b: 0.8904454223397135,
        schedule: PowerSchedule {
            eta: 1.0,
            g0: 0.015811448855388807,
            g1: 1.3506072330297898,
            k: 4.9847528799139935,
        },
        work_cost: UniformWorkCost {
            chi_max: 0.5018953974986752,
        },
        rho: 0.08927722845190324,
        delta: 0.9759438021953466,
        build_lag: 5,
    };
    let e = economy(params.clone());
    let eq = interior(params.clone());
    assert_eq!(e.at(eq.x_star).n_s, params.workers, "supply saturated");
    close("N_a", eq.n_a, params.workers);
    assert!(eq.participation <= 1.0, "{}", eq.participation);
    close("participation", eq.participation, 1.0);
}
