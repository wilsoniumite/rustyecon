//! m2: the machine block alone (docs/unit-1c.md §4.0, §4.2-4.3, §4.9 and §8): check_dynamics'
//! steady-state targets where they are in 1c's closure (§0.2), its U, R, S and L checks, the
//! closure per task type, the Leontief totals and the technique envelope.

use oracle::{
    closure, user_cost, ClosureError, Economy, MachineBlock, MachineType, Params, Recipe, Schedule,
};
use rustyecon_core::num;

use crate::g5_random_economies::SplitMix64;
use crate::goldens_1c::*;
use crate::support::*;
use crate::support_1c::*;

/// dynamics/checks/dynamics_ss_targets.json at laborformal 31b3482, as written: the doubles
/// check_dynamics.py (:480-500) printed. The sloped economy's (x*, c, w, r, Y, X, p_K).
const JSON_SLOPED: [f64; 7] = [
    0.5962490482385818,
    0.37397572659090933,
    1.2659064107675564,
    0.0607097333057802,
    1.381406447778144,
    3.685465266168454,
    0.2917930494787178,
];

/// The same file's flat economy (c, w, r, X, Y, m).
const JSON_FLAT: [f64; 6] = [
    0.3333333333333333,
    1.0,
    0.1381118092872455,
    2.4150834730304904,
    1.1046536171646546,
    0.3570926015168396,
];

/// The JSON's doubles against the 70-digit values: 2e-15 relative (docs/unit-1c.md §7; the
/// largest miss, 1.4e-15 on r, is the JSON's own rounding amplified by Den = 0.033).
const JSON_TOL: f64 = 2e-15;

/// check_dynamics' machine alone, at ρ = 0.05 (docs/unit-1c.md §3.3, M2).
fn dynamics_block() -> MachineBlock {
    MachineBlock::new(vec![dynamics_machine()], 0.05).expect("valid")
}

#[test]
fn dynamics_sloped_target() {
    // docs/unit-1c.md §0.2: the sloped target's price block at its own x* (an input), and its
    // quantities per unit of the good with N_a = N = 1 (check_dynamics EJ, :311-341).
    let block = dynamics_block();
    let totals = block.totals();
    close("u", totals.user_cost[0], M2_U);
    close("omega", totals.wealth_factor[0], M2_OMEGA);
    close(
        "lt",
        totals.lambda_tilde.as_ref().unwrap()[0],
        M2_LAMBDA_TILDE,
    );
    close("bt", totals.b_tilde.as_ref().unwrap()[0], M2_B_TILDE);
    close("lt^q", totals.lambda_tilde_q[0], M2_LAMBDA_TILDE_Q);
    close("bt^q", totals.b_tilde_q[0], M2_B_TILDE_Q);
    let x = JSON_SLOPED[0];
    assert_eq!(x, M2_SLOPED_X_STAR);
    // γ = 1 + 4x with γ_L = 1.
    let schedule = linear(1.0, 1.0, 4.0);
    let (gamma, j) = (schedule.gamma(x), schedule.integral(x));
    close("gamma*", gamma, M2_SLOPED_GAMMA);
    close("J(x*)", j, M2_SLOPED_J);
    let prices = block.closure(gamma, 0).expect("viable at x*");
    close("p_m = c/r", prices.prices[0], M2_SLOPED_P_M);
    close("v = w/r", prices.v, M2_SLOPED_V);
    close("V = p_K/r", prices.build[0], M2_SLOPED_BUILD);
    close("O", prices.operating[0], M2_SLOPED_OPERATING);
    // R6: the (O, V) system's determinant, the product of its pivots, is Den.
    close("Den", prices.pivots[0] * prices.pivots[1], M2_SLOPED_DEN);
    let p_good = prices.v * (1.0 - x) + prices.prices[0] * j;
    close("p_good = 1/r", p_good, M2_SLOPED_P_GOOD);
    let labour = (1.0 - x) + totals.lambda_tilde_q[0] * j;
    close("labour per good", labour, M2_SLOPED_LABOR_PER_GOOD);
    let per_good = block.gross_services(&[j])[0];
    close("X/Y", per_good, M2_SLOPED_SERVICES_PER_GOOD);
    let y = 1.0 / labour;
    close("Y", y, M2_SLOPED_Y);
    close("X", y * per_good, M2_SLOPED_SERVICES);
    // In the target's units, the good as numeraire: c = p_m r, w = v r, p_K = V r, r = 1/p.
    let r = 1.0 / p_good;
    for (name, got, want) in [
        ("c", prices.prices[0] * r, M2_SLOPED_TARGET_C),
        ("w", prices.v * r, M2_SLOPED_TARGET_W),
        ("r", r, M2_SLOPED_TARGET_R),
        ("Y", y, M2_SLOPED_TARGET_Y),
        ("X", y * per_good, M2_SLOPED_TARGET_X),
        ("pK", prices.build[0] * r, M2_SLOPED_TARGET_PK),
    ] {
        close(name, got, want);
    }
    // The JSON's doubles are the 70-digit values to 2e-15 (the gate's check of the targets).
    for (name, json, want) in [
        ("c", JSON_SLOPED[1], M2_SLOPED_TARGET_C),
        ("w", JSON_SLOPED[2], M2_SLOPED_TARGET_W),
        ("r", JSON_SLOPED[3], M2_SLOPED_TARGET_R),
        ("Y", JSON_SLOPED[4], M2_SLOPED_TARGET_Y),
        ("X", JSON_SLOPED[5], M2_SLOPED_TARGET_X),
        ("pK", JSON_SLOPED[6], M2_SLOPED_TARGET_PK),
    ] {
        close_to(&format!("JSON {name}"), json, want, JSON_TOL);
    }
    // x* is not 1c's to compute (the land-share closure, §0.2): the JSON's double is one
    // spacing below the 70-digit root, whose nearest double is the next one up.
    assert_eq!(M2_SLOPED_LAND_SHARE_ROOT, x.next_up());
    close_to("JSON x*", x, M2_SLOPED_LAND_SHARE_ROOT, JSON_TOL);
    // The sloped schedule is not viable at x = 1 with this machine (EJ1): Den(5) < 0. The
    // operating row alone is at its edge there, 1 − a − λγ = 0, and that first pivot is d.
    let (den, _) = den_and_theta_c(&dynamics_machine(), totals.user_cost[0], 5.0);
    assert!(den < 0.0);
    match block.closure(schedule.gamma(1.0), 0) {
        Err(ClosureError::NotViable { d }) => assert_eq!(d, 0.0),
        other => panic!("expected NotViable, got {other:?}"),
    }
}

#[test]
fn dynamics_flat_target() {
    // docs/unit-1c.md §0.2: the flat target at γ* = 3 (check_dynamics E1-E3, :212-231), with
    // its machine-task share m (an input); N = 1.
    let block = dynamics_block();
    let totals = block.totals();
    let prices = block.closure(3.0, 0).expect("viable");
    close("p_m", prices.prices[0], M2_FLAT_P_M);
    close("v", prices.v, M2_FLAT_V);
    close("V", prices.build[0], M2_FLAT_BUILD);
    close("O", prices.operating[0], M2_FLAT_OPERATING);
    close("Den", prices.pivots[0] * prices.pivots[1], M2_FLAT_DEN);
    let m = JSON_FLAT[5];
    assert_eq!(m, M2_FLAT_M);
    let per_good = block.gross_services(&[3.0 * m])[0];
    let y = 1.0 / ((1.0 - m) + totals.lambda_tilde_q[0] * (3.0 * m));
    close("Y", y, M2_FLAT_Y);
    close("X", y * per_good, M2_FLAT_SERVICES);
    // The target's units have w = 1: c = p_m/v, r = 1/v (E1: c = 1/γ-bar, so c = 1/3).
    close("c", prices.prices[0] / prices.v, M2_FLAT_TARGET_C);
    close("r", 1.0 / prices.v, M2_FLAT_TARGET_R);
    close("c = 1/3", M2_FLAT_TARGET_C, 1.0 / 3.0);
    for (name, json, want) in [
        ("c", JSON_FLAT[0], M2_FLAT_TARGET_C),
        ("w", JSON_FLAT[1], 1.0),
        ("r", JSON_FLAT[2], M2_FLAT_TARGET_R),
        ("X", JSON_FLAT[3], M2_FLAT_SERVICES),
        ("Y", JSON_FLAT[4], M2_FLAT_Y),
        ("m", JSON_FLAT[5], M2_FLAT_LAND_SHARE_M),
    ] {
        close_to(&format!("JSON {name}"), json, want, JSON_TOL);
    }
}

/// A random one-type block over check_dynamics' two recipes, and γ*.
fn draw_block(rng: &mut SplitMix64) -> (MachineType, f64, f64) {
    let t = machine_type(
        1.0,
        recipe(
            &[rng.uniform(0.0, 0.5)],
            rng.uniform(0.0, 0.2),
            rng.uniform(0.05, 0.5),
        ),
        recipe(
            &[rng.uniform(0.0, 0.3)],
            rng.uniform(0.0, 0.3),
            rng.uniform(0.0, 0.3),
        ),
        rng.uniform(0.05, 1.0),
        1 + (rng.next_u64() % 5) as u32,
    );
    (t, rng.uniform(0.0, 0.1), rng.uniform(0.1, 5.0))
}

/// check_dynamics R1: Den = 1 − a − λγ* − u(a_I + λ_Iγ*) and θ_c = (b + u·b_I)/Den.
fn den_and_theta_c(t: &MachineType, u: f64, gamma: f64) -> (f64, f64) {
    let (op, build) = (&t.operating, &t.build);
    let den =
        1.0 - op.machines[0] - op.labor * gamma - u * (build.machines[0] + build.labor * gamma);
    (den, (op.land + u * build.land) / den)
}

#[test]
fn two_recipe_closed_forms() {
    // check_dynamics R1-R3 and R6 on 200 random viable one-type blocks, and viability by the
    // least pivot exactly when Den > 0 (away from 0).
    let mut rng = SplitMix64(9261);
    let (mut viable, mut not_viable) = (0, 0);
    while viable < 200 {
        let (t, rho, gamma) = draw_block(&mut rng);
        let block = MachineBlock::new(vec![t.clone()], rho).expect("valid");
        let u = block.user_costs()[0];
        let (den, theta_c) = den_and_theta_c(&t, u, gamma);
        let result = block.closure(gamma, 0);
        if den.abs() < 1e-9 {
            continue;
        }
        match result {
            Ok(p) => {
                assert!(den > 0.0, "d > 0 but Den = {den:e}");
                assert!(p.d > 0.0);
                close("p_m = theta_c (R1)", p.prices[0], theta_c);
                close("v = gamma theta_c (R2)", p.v, gamma * theta_c);
                close(
                    "V = a_I theta_c + lambda_I theta_w + b_I (R3)",
                    p.build[0],
                    t.build.machines[0] * theta_c + t.build.labor * p.v + t.build.land,
                );
                close("det = Den (R6)", p.pivots[0] * p.pivots[1], den);
                viable += 1;
            }
            Err(ClosureError::NotViable { d }) => {
                assert!(den < 0.0 && d <= 0.0, "Den = {den:e}, d = {d:e}");
                not_viable += 1;
            }
            Err(e) => panic!("{e}"),
        }
    }
    assert!(not_viable > 0, "no draw was not viable");
}

#[test]
fn recipe_corners() {
    // R4: a zero build recipe prices the service as the flow benchmark at every u,
    // p_m = b/(1 − a − λγ*), bit for bit, with V = 0.
    let gamma = 1.75;
    for (rho, delta, lag) in [
        (0.0, 1.0, 1),
        (0.05, 0.1, 3),
        (0.2, 0.3, 2),
        (0.01, 0.9, 5),
        (0.5, 0.05, 4),
    ] {
        let t = machine_type(1.0, recipe(&[0.5], 0.1, 0.2), Recipe::zero(1), delta, lag);
        let block = MachineBlock::new(vec![t], rho).unwrap();
        let p = block.closure(gamma, 0).unwrap();
        assert_eq!(
            p.prices[0].to_bits(),
            (0.2 / ((1.0 - 0.5) - 0.1 * gamma)).to_bits(),
            "u {}",
            block.user_costs()[0]
        );
        assert_eq!(p.operating[0], p.prices[0]);
        assert_eq!(p.build[0], 0.0);
    }
    // R5: a zero operating recipe is 1a's carrying-factor form, u·b'/(1 − u(a' + λ'γ*)): 1a's
    // closure to 2 ulps (it associates u·b·r/D), and 1a's own solve's p_m bit for bit.
    let t = machine_type(1.0, Recipe::zero(1), recipe(&[0.1], 0.2, 0.02), 0.1, 3);
    let block = MachineBlock::new(vec![t], 0.05).unwrap();
    let u = block.user_costs()[0];
    let one = Economy::new(Params {
        a: 0.1,
        lam: 0.2,
        b: 0.02,
        schedule: linear(1.0, 1.0, 2.0),
        rho: 0.05,
        delta: 0.1,
        build_lag: 3,
        ..appendix_b()
    })
    .unwrap();
    for x in [0.0, 0.1, 0.3, 0.7] {
        let point = one.at(x);
        let p = block.closure(point.gamma, 0).unwrap();
        assert_eq!(p.prices[0].to_bits(), point.p_m.to_bits(), "p_m at x = {x}");
        assert_eq!(p.build[0].to_bits(), point.v_m.to_bits(), "V_m at x = {x}");
        assert_eq!(p.d.to_bits(), point.d.to_bits(), "D at x = {x}");
        assert_eq!(p.v.to_bits(), point.v.to_bits(), "v at x = {x}");
        let c = closure(0.1, 0.2, point.gamma, 0.02, 1.0, u).unwrap();
        let ulps = (p.prices[0] - c.p_m).abs() / (c.p_m.next_up() - c.p_m);
        assert!(ulps <= 2.0, "{ulps} ulps from 1a's closure at x = {x}");
    }
}

#[test]
fn user_cost_per_type() {
    // check_dynamics U1-U6: u_k = (ρ + δ_k)(1 + ρ)^(J_k − 1) per type, bit for bit.
    let types: Vec<MachineType> = [(0.1, 3), (0.5, 1), (0.03, 7), (1.0, 2)]
        .iter()
        .map(|&(delta, lag)| {
            machine_type(
                1.0,
                Recipe::zero(4),
                recipe(&[0.0; 4], 0.1, 0.2),
                delta,
                lag,
            )
        })
        .collect();
    for rho in [0.0, 0.05, 0.12] {
        let block = MachineBlock::new(types.clone(), rho).unwrap();
        for (k, t) in types.iter().enumerate() {
            assert_eq!(
                block.user_costs()[k].to_bits(),
                user_cost(rho, t.delta, t.build_lag).to_bits()
            );
        }
    }
    // U6: the free-entry PV sum Σ_{s≥J} (1 + ρ)^(−s)(1 − δ)^(s−J), summed to convergence,
    // is (1 + ρ)^(1−J)/(ρ + δ), so u = 1/PV of a unit rental.
    for (rho, delta, lag) in [(0.05, 0.10, 3u32), (0.12, 0.04, 7)] {
        let mut sum = 0.0;
        let mut s = lag;
        loop {
            let term =
                num::pow(1.0 + rho, -f64::from(s)) * num::pow(1.0 - delta, f64::from(s - lag));
            sum += term;
            if term < 1e-18 * sum {
                break;
            }
            s += 1;
        }
        close(
            "PV",
            sum,
            num::pow(1.0 + rho, 1.0 - f64::from(lag)) / (rho + delta),
        );
        close("u = 1/PV", user_cost(rho, delta, lag), 1.0 / sum);
    }
    // U4 and U4b's corners; ρ = 0 gives δ at every lag.
    for rho in [0.0, 0.05, 0.3] {
        assert_eq!(user_cost(rho, 0.2, 1), rho + 0.2);
        assert_eq!(user_cost(rho, 1.0, 1), 1.0 + rho);
        assert_eq!(user_cost(rho, 1.0, 0), 1.0);
    }
    assert_eq!(user_cost(0.0, 0.1, 6), 0.1);
    // U5: the misread (ρ + δ)(1 + ρ)^J is u·(1 + ρ), and differs whenever ρ > 0; S4: u rises
    // with J.
    for (rho, delta, lag) in [(0.05, 0.1, 3), (0.12, 0.04, 7)] {
        let u = user_cost(rho, delta, lag);
        let misread = (rho + delta) * num::pow(1.0 + rho, f64::from(lag));
        close("misread", misread, u * (1.0 + rho));
        assert!(misread > u);
        assert!(user_cost(rho, delta, lag + 1) > u);
    }
}

#[test]
fn comparative_statics() {
    // check_dynamics S1-S6 on 200 random viable points: each sign by a forward step in the
    // parameter, and S1's and S3's closed-form derivatives against central differences.
    const STEP: f64 = 1e-6;
    let mut rng = SplitMix64(9262);
    let mut done = 0;
    let theta_c = |t: &MachineType, rho: f64, gamma: f64| -> Option<f64> {
        MachineBlock::new(vec![t.clone()], rho)
            .ok()?
            .closure(gamma, 0)
            .ok()
            .map(|p| p.prices[0])
    };
    let theta_w = |t: &MachineType, rho: f64, gamma: f64| -> Option<f64> {
        MachineBlock::new(vec![t.clone()], rho)
            .ok()?
            .closure(gamma, 0)
            .ok()
            .map(|p| p.v)
    };
    while done < 200 {
        let (mut t, rho, gamma) = draw_block(&mut rng);
        t.build.labor = t.build.labor.max(0.01);
        let rho = rho.max(0.001);
        let Some(c) = theta_c(&t, rho, gamma) else {
            continue;
        };
        let u = user_cost(rho, t.delta, t.build_lag);
        let (den, _) = den_and_theta_c(&t, u, gamma);
        if den < 0.05 {
            continue;
        }
        let h = STEP * gamma;
        // S1, S2: automation (γ* down) cheapens the service and the wage's land claim.
        let up = theta_c(&t, rho, gamma + h).unwrap();
        assert!(up > c, "S1");
        assert!(
            theta_w(&t, rho, gamma + h).unwrap() > theta_w(&t, rho, gamma).unwrap(),
            "S2"
        );
        let down = theta_c(&t, rho, gamma - h).unwrap();
        let lam_hat = t.operating.labor + u * t.build.labor;
        close_to(
            "S1 derivative",
            (up - down) / (2.0 * h),
            c * lam_hat / den,
            1e-6,
        );
        // S3: recursive automation through either recipe.
        let mut more = t.clone();
        more.operating.labor += STEP;
        let d_lam = (theta_c(&more, rho, gamma).unwrap() - c) / STEP;
        assert!(d_lam > 0.0, "S3 lambda");
        close_to("S3 d/d lambda", d_lam, gamma * c / den, 1e-4);
        let mut more = t.clone();
        more.build.labor += STEP;
        let d_lam_i = (theta_c(&more, rho, gamma).unwrap() - c) / STEP;
        assert!(d_lam_i > 0.0, "S3 lambda_I");
        close_to("S3 d/d lambda_I", d_lam_i, u * gamma * c / den, 1e-4);
        // S4, S5: a longer build lag raises u, and u raises θ_c.
        let mut longer = t.clone();
        longer.build_lag += 1;
        assert!(user_cost(rho, t.delta, longer.build_lag) > u, "S4");
        if t.build.land + t.operating.land > 0.0 {
            if let Some(slower) = theta_c(&longer, rho, gamma) {
                assert!(slower > c, "S5");
            }
        }
        // S6: at λ = 0 the cross-effect survives through the build recipe alone.
        let mut no_lambda = t.clone();
        no_lambda.operating.labor = 0.0;
        let c0 = theta_c(&no_lambda, rho, gamma).unwrap();
        assert!(theta_c(&no_lambda, rho, gamma + h).unwrap() > c0, "S6");
        done += 1;
    }
}

#[test]
fn ledger() {
    // check_dynamics L1-L4 per type: u − δ = ρω (so (u − δ)VX = ρW), ω = 1 exactly at J = 1
    // and above 1 for J > 1 when ρ > 0, and zero NPV, u(1 + ρ)/(ρ + δ) = (1 + ρ)^J.
    let mut rng = SplitMix64(9263);
    for _ in 0..200 {
        let rho = rng.uniform(1e-3, 0.2);
        let delta = rng.uniform(0.02, 1.0);
        let lag = 1 + (rng.next_u64() % 12) as u32;
        let t = machine_type(1.0, Recipe::zero(1), recipe(&[0.0], 0.1, 0.3), delta, lag);
        let block = MachineBlock::new(vec![t], rho).unwrap();
        let (u, omega) = (block.user_costs()[0], block.wealth_factors()[0]);
        close("u - delta = rho omega (L1-L2)", u - delta, rho * omega);
        if lag == 1 {
            assert_eq!(omega, 1.0, "L4 at J = 1");
        } else {
            assert!(omega > 1.0, "L4 at J = {lag}");
        }
        close(
            "zero NPV (L3)",
            u * (1.0 + rho) / (rho + delta),
            num::pow(1.0 + rho, f64::from(lag)),
        );
    }
}

#[test]
fn closure_per_type() {
    // SSRN A.1's closure for every task type: v_t = γb̃_t/(θ_t − γλ̃_t) from the totals is
    // the block's v with τ = t; the envelope's technique has the least; SSRN eq 6's limit.
    let block = MachineBlock::new(m4_types(), 0.04).unwrap();
    let totals = block.totals();
    let (lt, bt) = (totals.lambda_tilde.unwrap(), totals.b_tilde.unwrap());
    let envelope = block.envelope(0.05, 3.0);
    for gamma in [0.05, 0.2, 0.4, 0.43, 0.9, 1.5, 2.1, 3.0] {
        let mut least = f64::INFINITY;
        for t in 0..3 {
            let theta = block.types()[t].task_efficiency;
            match block.closure(gamma, t) {
                Ok(p) => {
                    let wage = gamma * bt[t] / (theta - gamma * lt[t]);
                    close(&format!("v_{t} at {gamma}"), p.v, wage);
                    close("closure_wage", block.closure_wage(gamma, t).unwrap(), wage);
                    least = least.min(p.v);
                    for k in 0..3 {
                        close("p = v lt + bt", p.prices[k], p.v * lt[k] + bt[k]);
                    }
                }
                Err(ClosureError::NotViable { .. }) => {
                    assert!(theta - gamma * lt[t] <= 0.0);
                    assert!(block.closure_wage(gamma, t).is_none());
                }
                Err(ClosureError::Invalid(_)) => assert_eq!(theta, 0.0, "power does no tasks"),
                Err(e) => panic!("{e}"),
            }
        }
        let tau = envelope.technique_at(gamma);
        close(
            "the technique has the least v",
            block.closure(gamma, tau).unwrap().v,
            least,
        );
    }
    // SSRN eq 6: as γλ̃_τ/θ_τ → 0, p_τ/b̃_τ → 1.
    let block = dynamics_block();
    let bt = block.totals().b_tilde.unwrap()[0];
    let lt = block.totals().lambda_tilde.unwrap()[0];
    let mut last = f64::INFINITY;
    for gamma in [1.0, 1e-2, 1e-4, 1e-8] {
        let gap = block.closure(gamma, 0).unwrap().prices[0] / bt - 1.0;
        assert!(gap > 0.0 && gap < last, "{gamma}");
        assert!(
            gap <= 2.0 * gamma * lt / (1.0 - gamma * lt),
            "{gamma}: {gap}"
        );
        last = gap;
    }
    assert!(last < 1e-8);
}

/// Power iteration's Collatz-Wielandt bounds on the spectral radius of a positive matrix.
fn spectral_bounds(n: usize, a: &[f64]) -> (f64, f64) {
    let mut x = vec![1.0; n];
    let (mut lo, mut hi) = (0.0, f64::INFINITY);
    for _ in 0..2000 {
        let y: Vec<f64> = (0..n)
            .map(|i| (0..n).map(|j| a[i * n + j] * x[j]).sum())
            .collect();
        lo = (0..n).map(|i| y[i] / x[i]).fold(f64::INFINITY, f64::min);
        hi = (0..n).map(|i| y[i] / x[i]).fold(0.0, f64::max);
        let norm = y.iter().copied().fold(0.0, f64::max);
        x = y.iter().map(|v| v / norm).collect();
    }
    (lo, hi)
}

#[test]
fn leontief_totals() {
    // (I − Â)λ̃ = λ̂ and (I − Â)b̃ = b̂ by multiplication, on M4's block.
    let block = MachineBlock::new(m4_types(), 0.04).unwrap();
    let totals = block.totals();
    let (lt, bt) = (totals.lambda_tilde.unwrap(), totals.b_tilde.unwrap());
    for k in 0..3 {
        let (mut l, mut b) = (lt[k], bt[k]);
        for j in 0..3 {
            l -= block.a_hat(k, j) * lt[j];
            b -= block.a_hat(k, j) * bt[j];
        }
        close("(I - A-hat) lt", l, block.lambda_hat()[k]);
        close("(I - A-hat) bt", b, block.land_hat()[k]);
        let (mut l, mut b) = (totals.lambda_tilde_q[k], totals.b_tilde_q[k]);
        for j in 0..3 {
            l -= block.a_q(k, j) * totals.lambda_tilde_q[j];
            b -= block.a_q(k, j) * totals.b_tilde_q[j];
        }
        close("(I - A^q) lt^q", l, block.lambda_q()[k]);
        close("(I - A^q) bt^q", b, block.land_q()[k]);
    }
    // On 200 random positive matrices A (as build recipes at u = 1), the recipes are accepted
    // as productive exactly when power iteration puts the spectral radius below 1, away from
    // 1 by 1e-6; then every pivot of I − A is positive.
    let mut rng = SplitMix64(9264);
    let (mut below, mut above, mut done) = (0, 0, 0);
    while done < 200 {
        let n = 1 + (rng.next_u64() % 4) as usize;
        let scale = rng.uniform(0.1, 0.7);
        let a: Vec<f64> = (0..n * n).map(|_| rng.uniform(0.001, scale)).collect();
        let (lo, hi) = spectral_bounds(n, &a);
        if (lo - 1.0).abs() < 1e-6 || (hi - 1.0).abs() < 1e-6 || (lo < 1.0) != (hi < 1.0) {
            continue;
        }
        let types: Vec<MachineType> = (0..n)
            .map(|k| {
                machine_type(
                    1.0,
                    Recipe::zero(n),
                    recipe(&a[k * n..(k + 1) * n], 0.0, 0.5),
                    1.0,
                    1,
                )
            })
            .collect();
        match MachineBlock::new(types, 0.0) {
            Ok(block) => {
                assert!(hi < 1.0, "accepted with spectral radius {lo}..{hi}");
                assert!(block.totals().price_pivots.iter().all(|&p| p > 0.0));
                below += 1;
            }
            Err(e) => {
                assert!(lo > 1.0, "rejected with spectral radius {lo}..{hi}: {e}");
                assert!(e.to_string().contains("not productive"), "{e}");
                above += 1;
            }
        }
        done += 1;
    }
    assert!(below > 20 && above > 20, "{below} below, {above} above");
    // A = 0 and a diagonal A: the solve is exact, λ̃_k = λ_k/(1 − a_kk) with one rounding.
    let diag = [0.0, 0.1, 0.25, 0.7];
    let types: Vec<MachineType> = (0..4)
        .map(|k| {
            let mut row = [0.0; 4];
            row[k] = diag[k];
            machine_type(
                1.0,
                Recipe::zero(4),
                recipe(&row, 0.3 + k as f64, 0.2),
                1.0,
                1,
            )
        })
        .collect();
    let block = MachineBlock::new(types, 0.0).unwrap();
    let totals = block.totals();
    for (k, a) in diag.iter().enumerate() {
        let want = (0.3 + k as f64) / (1.0 - a);
        assert_eq!(
            totals.lambda_tilde.as_ref().unwrap()[k].to_bits(),
            want.to_bits()
        );
        assert_eq!(
            totals.b_tilde.as_ref().unwrap()[k].to_bits(),
            (0.2 / (1.0 - a)).to_bits()
        );
    }
}

/// A flow task type (operating recipe only) with λ and b, θ 1, over K types: its totals are
/// λ and b exactly.
fn flow(k: usize, labor: f64, land: f64) -> MachineType {
    machine_type(
        1.0,
        recipe(&vec![0.0; k], labor, land),
        Recipe::zero(k),
        1.0,
        1,
    )
}

#[test]
fn envelope() {
    // M4's switch equals the closed form γ_τl = (b̃_l θ_τ − b̃_τ θ_l)/Δ_τl and the golden.
    let block = MachineBlock::new(m4_types(), 0.04).unwrap();
    let totals = block.totals();
    let (lt, bt) = (totals.lambda_tilde.unwrap(), totals.b_tilde.unwrap());
    let schedule = linear(1.0, 0.2, 0.8);
    let env = block.envelope(schedule.gamma(oracle::BRACKET_LO), schedule.gamma(1.0));
    assert_eq!(env.first, 0);
    assert_eq!(env.switches.len(), 1);
    let s = env.switches[0];
    assert_eq!((s.below, s.above), (0, 1));
    let closed = (bt[1] * 1.0 - bt[0] * 2.0) / (bt[1] * lt[0] - bt[0] * lt[1]);
    assert_eq!(s.gamma.to_bits(), closed.to_bits());
    close("switch gamma", s.gamma, M4_SWITCH_GAMMA);
    // An exact switch point belongs to the piece above; no type repeats.
    assert_eq!(env.technique_at(s.gamma), 1);
    assert_eq!(env.technique_at(s.gamma.next_down()), 0);
    assert_eq!(env.last(), 1);
    // Tie-breaks at γ_lo (§5.2): equal closure wages go to the type cheaper just above (the
    // smaller λ̃/θ), whatever its index. Flow types have λ̃ = λ and b̃ = b exactly; at γ = 1
    // both wages are 1, exactly.
    let steep = flow(2, 0.5, 0.5);
    let flat = flow(2, 0.25, 0.75);
    let block = MachineBlock::new(vec![steep.clone(), flat.clone()], 0.0).unwrap();
    assert_eq!(block.closure_wage(1.0, 0), block.closure_wage(1.0, 1));
    let env = block.envelope(1.0, 1.5);
    assert_eq!((env.first, env.switches.len()), (1, 0));
    // An exact duplicate ties at every γ: the lower index.
    let block = MachineBlock::new(vec![flat.clone(), flat.clone()], 0.0).unwrap();
    let env = block.envelope(0.5, 1.5);
    assert_eq!((env.first, env.switches.len()), (0, 0));
    // A double of a type (its unit is two of the other's: θ, recipe and so totals doubled)
    // ties too, and λ̃θ' = λ̃'θ exactly: the lower index.
    let double = machine_type(2.0, recipe(&[0.0, 0.0], 0.5, 1.5), Recipe::zero(2), 1.0, 1);
    for (types, want) in [
        (vec![flat.clone(), double.clone()], 0),
        (vec![double, flat], 0),
    ] {
        let block = MachineBlock::new(types, 0.0).unwrap();
        assert_eq!(block.closure_wage(0.7, 0), block.closure_wage(0.7, 1));
        assert_eq!(block.envelope(0.5, 1.5).first, want);
    }
}
