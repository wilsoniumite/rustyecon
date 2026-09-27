//! Fixtures and checks shared by unit 1c's tests (docs/unit-1c.md §8).

// The identities are sums over categories and types, and the checks index several vectors at
// once, as docs/unit-1c.md §4.5 writes them; index loops keep each check beside its equation.
#![allow(clippy::needless_range_loop)]

use std::collections::BTreeMap;

use oracle::{
    Category, CategoryParams, Eq1c, MachineEconomy, MachineParams, MachineType, Output1b,
    PowerSchedule, Recipe, Regime, Residuals1c, SolveError, UniformWorkCost, BRACKET_LO,
};

use crate::support::*;
use crate::support_1b::*;

/// A recipe from (machines, λ, b).
pub fn recipe(machines: &[f64], labor: f64, land: f64) -> Recipe {
    Recipe {
        machines: machines.to_vec(),
        labor,
        land,
    }
}

/// A machine type from θ, the operating and build recipes, δ and J.
pub fn machine_type(
    task_efficiency: f64,
    operating: Recipe,
    build: Recipe,
    delta: f64,
    build_lag: u32,
) -> MachineType {
    MachineType {
        task_efficiency,
        operating,
        build,
        delta,
        build_lag,
    }
}

/// A C×C matrix of zeros.
pub fn no_inputs(count: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; count]; count]
}

/// check_dynamics' machine (sloped :311-321, flat :225-231; docs/unit-1c.md §3.3 M2): operating
/// (a, λ, b) = (0.5, 0.1, 0.2), build (a_I, λ_I, b_I) = (0.1, 0.2, 0.02), δ 0.1, J 3.
pub fn dynamics_machine() -> MachineType {
    machine_type(
        1.0,
        recipe(&[0.5], 0.1, 0.2),
        recipe(&[0.1], 0.2, 0.02),
        0.1,
        3,
    )
}

/// G1's household (N 4, T 10, χ ~ U[0, 1]) with the good and space (h 1) on one segment, on the
/// given schedule, ρ and machine types.
pub fn appendix_b_household(
    schedule: PowerSchedule,
    rho: f64,
    space: f64,
    machine_types: Vec<MachineType>,
) -> MachineParams {
    MachineParams {
        workers: 4.0,
        land: 10.0,
        schedule,
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho,
        machine_types,
        edges: vec![0.0, 1.0],
        categories: vec![category(1.0, 0.0, &[1.0]), category(space, 1.0, &[0.0])],
        intermediate: no_inputs(2),
    }
}

/// γ = η(g0 + g1·x) with k = 1.
pub fn linear(eta: f64, g0: f64, g1: f64) -> PowerSchedule {
    PowerSchedule {
        eta,
        g0,
        g1,
        k: 1.0,
    }
}

/// M3 (constructed 2026-09-27; docs/unit-1c.md §3.3): M2's machine in Appendix B's closure, G1's
/// household on γ = 1 + 2x, at ρ.
pub fn m3(rho: f64) -> MachineParams {
    appendix_b_household(linear(1.0, 1.0, 2.0), rho, 1.0, vec![dynamics_machine()])
}

/// M4's machine types (constructed 2026-09-27; docs/unit-1c.md §3.3): the loom (θ 1), the
/// engine (θ 2) and power (θ 0), over (loom, engine, power).
pub fn m4_types() -> Vec<MachineType> {
    vec![
        machine_type(
            1.0,
            recipe(&[0.0, 0.0, 0.05], 0.3, 0.05),
            recipe(&[0.1, 0.0, 0.0], 1.0, 0.3),
            0.1,
            2,
        ),
        machine_type(
            2.0,
            recipe(&[0.0, 0.0, 0.5], 0.02, 0.0),
            recipe(&[0.0, 0.1, 0.0], 0.1, 0.4),
            0.05,
            3,
        ),
        machine_type(
            0.0,
            recipe(&[0.0, 0.0, 0.0], 0.02, 0.5),
            recipe(&[0.0, 0.1, 0.0], 0.1, 0.1),
            0.05,
            3,
        ),
    ]
}

/// The type names of M4, in order.
pub const M4_NAMES: [&str; 3] = ["loom", "engine", "power"];

/// M4's intermediate inputs: food uses 0.1 of manufactures, care 0.2 of food, shelter 0.15 of
/// manufactures (row = the user).
pub fn m4_intermediate() -> Vec<Vec<f64>> {
    vec![
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.1, 0.0, 0.0, 0.0],
        vec![0.0, 0.2, 0.0, 0.0],
        vec![0.15, 0.0, 0.0, 0.0],
    ]
}

/// M4, the three-type fork economy (constructed 2026-09-27; docs/unit-1c.md §3.3): 1b's fork
/// economy (N 4, T 10, χ ~ U[0, 1], edges (0, 0.4, 0.75, 1), γ = η(0.2 + 0.8x)) with M4's
/// intermediate inputs and types, ρ 0.04, at η.
pub fn m4(eta: f64) -> MachineParams {
    let fork = fork_economy();
    MachineParams {
        workers: 4.0,
        land: 10.0,
        schedule: linear(eta, 0.2, 0.8),
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.04,
        machine_types: m4_types(),
        edges: fork.edges,
        categories: fork.categories,
        intermediate: m4_intermediate(),
    }
}

/// M5's flow type: operating (0, 0; 0.1; 0.5), no build recipe, δ 1, J 1.
pub fn flow_type() -> MachineType {
    machine_type(1.0, recipe(&[0.0, 0.0], 0.1, 0.5), Recipe::zero(2), 1.0, 1)
}

/// M5's durable type: no operating recipe, build (0, 0; 0.02; 1.85), δ 0.02, J 10.
pub fn durable_type() -> MachineType {
    machine_type(
        1.0,
        Recipe::zero(2),
        recipe(&[0.0, 0.0], 0.02, 1.85),
        0.02,
        10,
    )
}

/// M5 (constructed 2026-09-27; docs/unit-1c.md §3.3): G1's household on γ = 0.2 + 0.8x with
/// the flow and the durable type, at ρ, N and h.
pub fn m5(rho: f64, workers: f64, space: f64) -> MachineParams {
    MachineParams {
        workers,
        ..appendix_b_household(
            linear(1.0, 0.2, 0.8),
            rho,
            space,
            vec![flow_type(), durable_type()],
        )
    }
}

/// M1: a unit-1b economy in one-type form.
pub fn one_type(params: CategoryParams) -> MachineParams {
    MachineParams::from_categories(params)
}

/// Validates unit-1c parameters the test knows to be valid.
pub fn economy_1c(params: MachineParams) -> MachineEconomy {
    MachineEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{params:?}: {e}"))
}

/// Solves a unit-1c economy the test knows to be interior.
pub fn interior_1c(params: MachineParams) -> Box<Eq1c> {
    match economy_1c(params.clone()).solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{params:?} is not interior: {other:?}"),
    }
}

/// Every output of a unit-1c equilibrium, by printed key, as bits (flags and counts as
/// integers); `None` for an absent optional output.
pub fn bits_1c(eq: &Eq1c) -> BTreeMap<String, Option<u64>> {
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

/// The outcome of a unit-1c solve as comparable data.
#[derive(Debug, PartialEq)]
pub enum Outcome1c {
    /// Interior.
    Interior,
    /// A boundary regime's diagnostic, by name, as bits.
    Boundary(&'static str, u64),
    /// An error, as its Debug text.
    Error(String),
}

/// The outcome of a unit-1c solve.
pub fn outcome_1c(result: &Result<Regime<Eq1c>, SolveError>) -> Outcome1c {
    match result {
        Ok(Regime::Interior(_)) => Outcome1c::Interior,
        Ok(Regime::BoundaryNoMargin { f_at_1 }) => Outcome1c::Boundary("f_at_1", f_at_1.to_bits()),
        Ok(Regime::NotViable { d_at_1 }) => Outcome1c::Boundary("d_at_1", d_at_1.to_bits()),
        Ok(Regime::NoInteriorAtZero { f_at_0 }) => Outcome1c::Boundary("f_at_0", f_at_0.to_bits()),
        Err(e) => Outcome1c::Error(format!("{e:?}")),
    }
}

/// The share of the machine tasks type k does at the equilibrium: 1 for the technique, the
/// tie's split, 0 otherwise.
pub fn task_share(eq: &Eq1c, k: usize) -> f64 {
    match eq.tie {
        None if k == eq.technique => 1.0,
        Some(t) if k == eq.technique => 1.0 - t.share,
        Some(t) if k == t.above => t.share,
        _ => 0.0,
    }
}

/// The residuals of docs/unit-1c.md §6, recomputed from the reported fields and the parameters
/// in the solve's own form: the same operations on the same doubles, so they must agree bit
/// for bit.
pub fn recomputed_1c(economy: &MachineEconomy, eq: &Eq1c) -> Residuals1c {
    let p = economy.params();
    let types = &p.machine_types;
    let k_count = types.len();
    let count = p.categories.len();
    let q = economy.at_with(eq.x_star, eq.technique);
    let worse = |a: f64, b: f64| if b.is_nan() || b > a { b } else { a };
    let a_q = |k: usize, l: usize| {
        types[k].operating.machines[l] + types[k].delta * types[k].build.machines[l]
    };
    let (mut fork, mut totals, mut spending) = (0.0f64, 0.0f64, 0.0);
    for (j, c) in eq.categories.iter().enumerate() {
        fork = worse(
            fork,
            (c.price - (eq.v * c.l_star + economy.chain_land()[j])).abs() / c.price,
        );
        totals = worse(
            totals,
            (c.price - (eq.v * c.lambda_tilde + c.b_tilde)).abs() / c.price,
        );
        spending += c.price * c.output;
    }
    let mut machine_land = 0.0;
    for (k, t) in eq.types.iter().enumerate() {
        machine_land +=
            (types[k].operating.land + types[k].delta * types[k].build.land) * t.services;
    }
    let mut services = 0.0f64;
    for (k, t) in eq.types.iter().enumerate() {
        let r = if t.services > 0.0 {
            let mut used = 0.0;
            for l in 0..k_count {
                used += a_q(l, k) * eq.types[l].services;
            }
            ((t.services - t.task_services) - used).abs() / t.services
        } else {
            0.0
        };
        services = worse(services, r);
    }
    let mut user_cost = 0.0f64;
    for k in 0..k_count {
        let (op, build) = (&types[k].operating, &types[k].build);
        let (mut o, mut v_k) = (0.0, 0.0);
        for l in 0..k_count {
            o += op.machines[l] * eq.types[l].price;
            v_k += build.machines[l] * eq.types[l].price;
        }
        let o = o + op.labor * eq.v + op.land;
        let v_k = v_k + build.labor * eq.v + build.land;
        let price = eq.types[k].price;
        user_cost = worse(
            user_cost,
            (price - (o + eq.types[k].user_cost * v_k)).abs() / price,
        );
    }
    let mut leontief_price = 0.0f64;
    for j in 0..count {
        let mut inputs = 0.0;
        for l in 0..count {
            inputs += p.intermediate[j][l] * eq.categories[l].price;
        }
        for (k, t) in types.iter().enumerate() {
            let share = task_share(eq, k);
            if share > 0.0 {
                inputs += (share * (q.machine[j] / t.task_efficiency)) * eq.types[k].price;
            }
        }
        let cost = inputs + q.human[j] * eq.v + p.categories[j].direct_land;
        leontief_price = worse(
            leontief_price,
            (eq.categories[j].price - cost).abs() / eq.categories[j].price,
        );
    }
    let u = |k: usize| eq.types[k].user_cost;
    for k in 0..k_count {
        let (op, build) = (&types[k].operating, &types[k].build);
        let mut inputs = 0.0;
        for l in 0..k_count {
            inputs += (op.machines[l] + u(k) * build.machines[l]) * eq.types[l].price;
        }
        let cost = inputs + (op.labor + u(k) * build.labor) * eq.v + (op.land + u(k) * build.land);
        let price = eq.types[k].price;
        leontief_price = worse(leontief_price, (price - cost).abs() / price);
    }
    let gross: Vec<f64> = economy.basket_outputs().iter().map(|b| b * eq.y).collect();
    let mut leontief_quantity = 0.0f64;
    for j in 0..count {
        if gross[j] > 0.0 {
            let mut used = 0.0;
            for l in 0..count {
                used += p.intermediate[l][j] * gross[l];
            }
            let demand = used + p.categories[j].weight * eq.y;
            leontief_quantity = worse(leontief_quantity, (gross[j] - demand).abs() / gross[j]);
        }
    }
    for k in 0..k_count {
        let x_k = eq.types[k].services;
        if x_k > 0.0 {
            let mut used = 0.0;
            let share = task_share(eq, k);
            if share > 0.0 {
                for j in 0..count {
                    used += gross[j] * (share * (q.machine[j] / types[k].task_efficiency));
                }
            }
            for l in 0..k_count {
                used += a_q(l, k) * eq.types[l].services;
            }
            leontief_quantity = worse(leontief_quantity, (x_k - used).abs() / x_k);
        }
    }
    let tau = &eq.types[eq.technique];
    let theta = types[eq.technique].task_efficiency;
    let wage = (eq.gamma_star * tau.b_tilde) / (theta - eq.gamma_star * tau.lambda_tilde);
    let task_price = tau.price / theta;
    let mut cheapest = 0.0f64;
    for (k, t) in types.iter().enumerate() {
        if t.task_efficiency > 0.0 {
            let undercut = (task_price - eq.types[k].price / t.task_efficiency) / task_price;
            cheapest = worse(cheapest, undercut.max(0.0));
        }
    }
    Residuals1c {
        labor: (eq.n_a - q.n_s).abs(),
        income: (eq.y * eq.p_s - eq.income).abs() / eq.income,
        land: (eq.b_d * eq.y + machine_land - p.land).abs() / p.land,
        services,
        user_cost,
        fork,
        totals,
        expenditure: (spending - eq.income).abs() / eq.income,
        leontief_price,
        leontief_quantity,
        closure: (eq.v - wage).abs() / eq.v,
        cheapest,
    }
}

/// Every identity of docs/unit-1c.md §4.4-4.7 at an interior equilibrium, recomputed here by
/// multiplication from the reported fields and the parameters (§8, `m6::identities` and
/// `m6::leontief_identities`), and the prices pinned to `at_with(x*, technique)` bit for bit.
pub fn check_identities_1c(economy: &MachineEconomy, eq: &Eq1c) {
    let p = economy.params();
    let types = &p.machine_types;
    let (k_count, count) = (types.len(), p.categories.len());
    let at = |what: &str| format!("{what} at {p:?}");
    let rho = p.rho;
    let u = |k: usize| eq.types[k].user_cost;
    // The prices, bit for bit at the double x* with the technique (§8).
    let q = economy.at_with(eq.x_star, eq.technique);
    assert_eq!(eq.v.to_bits(), q.v.to_bits(), "{}", at("v"));
    assert_eq!(eq.p_s.to_bits(), q.p_s.to_bits(), "{}", at("P_s"));
    assert_eq!(eq.d.to_bits(), q.d.to_bits(), "{}", at("d"));
    for k in 0..k_count {
        assert_eq!(
            eq.types[k].price.to_bits(),
            q.type_prices[k].to_bits(),
            "{}",
            at("p_k")
        );
    }
    for j in 0..count {
        assert_eq!(
            eq.categories[j].price.to_bits(),
            q.prices[j].to_bits(),
            "{}",
            at("p_j")
        );
        assert_eq!(
            eq.categories[j].machine.to_bits(),
            q.machine[j].to_bits(),
            "{}",
            at("M_j")
        );
    }
    assert!(eq.d > 0.0, "{}", at("viable at x*"));
    // The machine block: the two-recipe rows (check_dynamics R1-R3; SSRN A.4).
    for k in 0..k_count {
        let (op, build) = (&types[k].operating, &types[k].build);
        let t = &eq.types[k];
        let (mut o, mut v_k) = (0.0, 0.0);
        for l in 0..k_count {
            o += op.machines[l] * eq.types[l].price;
            v_k += build.machines[l] * eq.types[l].price;
        }
        let o = o + op.labor * eq.v + op.land;
        let v_k = v_k + build.labor * eq.v + build.land;
        near(&at("O_k"), t.operating_cost, o, FULL * t.price);
        near(&at("V_k"), t.build_cost, v_k, FULL * t.price.max(v_k));
        close(
            &at("p_k = O_k + u_k V_k"),
            t.operating_cost + u(k) * t.build_cost,
            t.price,
        );
        // p_k = v lt_k + bt_k (SSRN eq 4 with the machine rows scaled).
        close(
            &at("p_k = v lt + bt"),
            eq.v * t.lambda_tilde + t.b_tilde,
            t.price,
        );
        // (I − Â) lt = λ̂ and (I − Â) bt = b̂ by multiplication.
        let (mut lhs_l, mut lhs_b) = (t.lambda_tilde, t.b_tilde);
        for l in 0..k_count {
            let a_hat = op.machines[l] + u(k) * build.machines[l];
            lhs_l -= a_hat * eq.types[l].lambda_tilde;
            lhs_b -= a_hat * eq.types[l].b_tilde;
        }
        near(
            &at("(I - A-hat) lt = l-hat"),
            lhs_l,
            op.labor + u(k) * build.labor,
            FULL * t.lambda_tilde.max(1e-300),
        );
        near(
            &at("(I - A-hat) bt = b-hat"),
            lhs_b,
            op.land + u(k) * build.land,
            FULL * t.b_tilde,
        );
        // The clearing side: (I − A^q) lt^q = λ^q.
        let (mut lhs_l, mut lhs_b) = (t.lambda_tilde_q, t.b_tilde_q);
        for l in 0..k_count {
            let a_q = op.machines[l] + types[k].delta * build.machines[l];
            lhs_l -= a_q * eq.types[l].lambda_tilde_q;
            lhs_b -= a_q * eq.types[l].b_tilde_q;
        }
        near(
            &at("(I - A^q) lt^q = l^q"),
            lhs_l,
            op.labor + types[k].delta * build.labor,
            FULL * t.lambda_tilde.max(1e-300),
        );
        near(
            &at("(I - A^q) bt^q = b^q"),
            lhs_b,
            op.land + types[k].delta * build.land,
            FULL * t.b_tilde,
        );
        at_most(&at("lt^q <= lt"), t.lambda_tilde_q, t.lambda_tilde);
        at_most(&at("bt^q <= bt"), t.b_tilde_q, t.b_tilde);
        // The ledger per type (check_dynamics L1-L4).
        assert_eq!(
            t.user_cost,
            oracle::user_cost(rho, types[k].delta, types[k].build_lag)
        );
        near(
            &at("wealth"),
            t.wealth,
            t.wealth_factor * t.build_cost * t.services,
            FULL * t.wealth.max(1e-300),
        );
        near(
            &at("interest = rho W"),
            t.interest,
            rho * t.wealth,
            FULL * eq.income,
        );
        if rho >= 1e-3 {
            near(
                &at("cash = rho W"),
                u(k) * t.build_cost * t.services - t.build_cost * types[k].delta * t.services,
                rho * t.wealth,
                FULL * (u(k) * t.build_cost * t.services).max(1e-300),
            );
        }
        at_most(&at("W >= V X"), t.build_cost * t.services, t.wealth);
        near(
            &at("builds"),
            t.builds,
            types[k].delta * t.services,
            FULL * t.services.max(1e-300),
        );
        // Delivered cost and the closure wage (SSRN A.1).
        match (types[k].task_efficiency > 0.0, t.delivered_cost) {
            (true, Some(c)) => {
                close(&at("delivered cost"), c, t.price / types[k].task_efficiency);
                at_most(
                    &at("no task type is cheaper"),
                    eq.types[eq.technique].price / types[eq.technique].task_efficiency,
                    c,
                );
            }
            (false, None) => assert!(
                t.closure_wage.is_none(),
                "{}",
                at("closure wage of a non-task type")
            ),
            other => panic!("{}: {other:?}", at("delivered cost")),
        }
        if let Some(w) = t.closure_wage {
            at_most(&at("closure wage >= v"), eq.v, w);
        }
    }
    let tau = &eq.types[eq.technique];
    let theta = types[eq.technique].task_efficiency;
    close(
        &at("closure"),
        eq.gamma_star * tau.b_tilde / (theta - eq.gamma_star * tau.lambda_tilde),
        eq.v,
    );
    close(
        &at("v = gamma p_tau/theta"),
        eq.gamma_star * tau.price / theta,
        eq.v,
    );
    if let Some(w) = tau.closure_wage {
        close(&at("the technique's closure wage"), w, eq.v);
    }
    // Quantities: the full clearing system by multiplication (SSRN A.1: f = (I − Aᵀ)y).
    let gross: Vec<f64> = eq.categories.iter().map(|c| c.gross_output).collect();
    for (j, c) in eq.categories.iter().enumerate() {
        close(
            &at("gross output = y-hat Y"),
            c.gross_output,
            economy.basket_outputs()[j] * eq.y,
        );
        let mut used = 0.0;
        for l in 0..count {
            used += p.intermediate[l][j] * gross[l];
        }
        near(
            &at("f_j = Y z_j"),
            gross[j] - used,
            p.categories[j].weight * eq.y,
            FULL * eq.y * 4.0,
        );
        close(&at("y_j = z_j Y"), c.output, p.categories[j].weight * eq.y);
    }
    let mut pf = 0.0;
    for (j, c) in eq.categories.iter().enumerate() {
        let mut used = 0.0;
        for l in 0..count {
            used += p.intermediate[l][j] * gross[l];
        }
        pf += c.price * (gross[j] - used);
    }
    let biggest = gross
        .iter()
        .copied()
        .chain(eq.types.iter().map(|t| t.services))
        .fold(0.0f64, f64::max);
    for k in 0..k_count {
        let t = &eq.types[k];
        let mut used = 0.0;
        let share = task_share(eq, k);
        if share > 0.0 {
            for j in 0..count {
                used += gross[j] * share * eq.categories[j].machine / types[k].task_efficiency;
            }
        }
        near(&at("task services"), t.task_services, used, FULL * biggest);
        for l in 0..k_count {
            let a_q = types[l].operating.machines[k] + types[l].delta * types[l].build.machines[k];
            used += a_q * eq.types[l].services;
        }
        let f_k = t.services - used;
        near(&at("f_machine = 0"), f_k, 0.0, FULL * biggest);
        pf += t.price * f_k;
    }
    // N_a = λ^qᵀy and T = b^qᵀy.
    let (mut hours, mut land) = (0.0, 0.0);
    for (j, c) in eq.categories.iter().enumerate() {
        hours += c.human * gross[j];
        land += p.categories[j].direct_land * gross[j];
    }
    for (k, t) in eq.types.iter().enumerate() {
        let (op, build) = (&types[k].operating, &types[k].build);
        hours += (op.labor + types[k].delta * build.labor) * t.services;
        land += (op.land + types[k].delta * build.land) * t.services;
        close(
            &at("type hours"),
            t.hours,
            (op.labor + types[k].delta * build.labor) * t.services,
        );
        close(
            &at("type land"),
            t.land,
            (op.land + types[k].delta * build.land) * t.services,
        );
    }
    near(&at("N_a = l^q' y"), hours, eq.n_a, FULL * eq.n_a);
    near(&at("T = b^q' y"), land, p.land, FULL * p.land);
    // Income (§4.6; SSRN App. C): p'f = v N_a + T + interest = Y P_s = Σ p_j z_j Y.
    let interest: f64 = eq.types.iter().map(|t| t.interest).sum();
    near(&at("interest"), interest, eq.interest, FULL * eq.income);
    near(
        &at("income = v N_a + T + interest"),
        eq.v * eq.n_a + p.land + eq.interest,
        eq.income,
        FULL * eq.income,
    );
    near(&at("p'f"), pf, eq.income, FULL * eq.income);
    near(
        &at("income = Y P_s"),
        eq.y * eq.p_s,
        eq.income,
        FULL * eq.income,
    );
    let spending: f64 = eq.categories.iter().map(|c| c.price * c.output).sum();
    near(
        &at("income = sum p_j z_j Y"),
        spending,
        eq.income,
        FULL * eq.income,
    );
    if rho >= 1e-3 {
        let via_u: f64 = eq
            .types
            .iter()
            .enumerate()
            .map(|(k, t)| (u(k) - types[k].delta) * t.build_cost * t.services)
            .sum();
        near(
            &at("interest = sum (u - delta) V X"),
            via_u,
            eq.interest,
            FULL * eq.income,
        );
    }
    if rho == 0.0 {
        assert_eq!(eq.interest, 0.0, "{}", at("interest at rho 0"));
    }
    // The categories' price side, by multiplication (SSRN eq 2).
    for j in 0..count {
        let c = &eq.categories[j];
        let mut cost = 0.0;
        for l in 0..count {
            cost += p.intermediate[j][l] * eq.categories[l].price;
        }
        for (k, t) in types.iter().enumerate() {
            let share = task_share(eq, k);
            if share > 0.0 {
                cost += share * q.machine[j] / t.task_efficiency * eq.types[k].price;
            }
        }
        cost += q.human[j] * eq.v + p.categories[j].direct_land;
        close(&at("p_j = sum a p + v H + pi M + b"), cost, c.price);
        // The chain totals by multiplication: (I − A_cc) L* = H + M/γ, and λ̃, b̃ likewise.
        let chain = |f: fn(&oracle::CategoryEq1c) -> f64| -> f64 {
            let mut s = f(c);
            for l in 0..count {
                s -= p.intermediate[j][l] * f(&eq.categories[l]);
            }
            s
        };
        near(
            &at("(I - A) L*"),
            chain(|c| c.l_star),
            q.human[j] + q.machine[j] / eq.gamma_star,
            FULL * c.l_bar.max(c.l_star),
        );
        let mut coef_l = 0.0;
        let mut coef_b = 0.0;
        let mut coef_lq = 0.0;
        let mut coef_bq = 0.0;
        for (k, t) in types.iter().enumerate() {
            let share = task_share(eq, k);
            if share > 0.0 {
                coef_l += share * eq.types[k].lambda_tilde / t.task_efficiency;
                coef_b += share * eq.types[k].b_tilde / t.task_efficiency;
                coef_lq += share * eq.types[k].lambda_tilde_q / t.task_efficiency;
                coef_bq += share * eq.types[k].b_tilde_q / t.task_efficiency;
            }
        }
        let scale = c.lambda_tilde + c.b_tilde / eq.v;
        near(
            &at("(I - A) lt"),
            chain(|c| c.lambda_tilde),
            c.human + c.machine * coef_l,
            FULL * scale,
        );
        near(
            &at("(I - A) bt"),
            chain(|c| c.b_tilde),
            p.categories[j].direct_land + c.machine * coef_b,
            FULL * c.price.max(c.b_tilde),
        );
        near(
            &at("(I - A) lt^q"),
            chain(|c| c.lambda_tilde_q),
            c.human + c.machine * coef_lq,
            FULL * scale,
        );
        near(
            &at("(I - A) bt^q"),
            chain(|c| c.b_tilde_q),
            p.categories[j].direct_land + c.machine * coef_bq,
            FULL * c.price.max(c.b_tilde),
        );
        near(
            &at("(I - A) b-bar"),
            chain(|c| c.chain_land),
            p.categories[j].direct_land,
            FULL * c.chain_land.max(p.categories[j].direct_land).max(1e-300),
        );
        assert_eq!(c.l_bar, economy.all_human_hours()[j]);
        assert_eq!(c.chain_land, economy.chain_land()[j]);
        close(
            &at("share"),
            c.share,
            p.categories[j].weight * c.price / eq.p_s,
        );
        near(
            &at("final hours"),
            c.final_hours,
            c.gross_output * c.human,
            FULL * eq.n_a,
        );
        near(
            &at("machine services"),
            c.machine_services,
            c.gross_output * c.machine,
            FULL * biggest,
        );
    }
    // The basket (SSRN eq 7): P_s in both forms, and eq 11 on the clearing side.
    let dot = |f: fn(&oracle::CategoryEq1c) -> f64| -> f64 {
        eq.categories
            .iter()
            .zip(&p.categories)
            .map(|(c, cat)| cat.weight * f(c))
            .sum()
    };
    close(&at("P_s = sum z p"), dot(|c| c.price), eq.p_s);
    close(&at("L_s"), dot(|c| c.lambda_tilde), eq.l_s);
    close(&at("B_s"), dot(|c| c.b_tilde), eq.b_s);
    close(&at("L_s^q"), dot(|c| c.lambda_tilde_q), eq.l_s_q);
    close(&at("B_s^q"), dot(|c| c.b_tilde_q), eq.b_s_q);
    close(&at("L*_s"), dot(|c| c.l_star), eq.l_star_s);
    close(&at("P_s = v L_s + B_s"), eq.v * eq.l_s + eq.b_s, eq.p_s);
    close(
        &at("P_s = v L*_s + z'b-bar"),
        eq.v * eq.l_star_s + dot(|c| c.chain_land),
        eq.p_s,
    );
    close(&at("B_y = z'b-bar"), dot(|c| c.chain_land), eq.b_d);
    assert_eq!(eq.b_d, economy.basket_land());
    close(&at("Y = T/B_s^q"), p.land / eq.b_s_q, eq.y);
    close(
        &at("N_a = T L_s^q/B_s^q"),
        p.land * eq.l_s_q / eq.b_s_q,
        eq.n_a,
    );
    near(
        &at("final hours"),
        eq.y * eq.h_s,
        eq.final_hours,
        FULL * eq.n_a,
    );
    close(&at("N_a"), eq.final_hours + eq.machine_hours, eq.n_a);
    let machine_hours: f64 = eq.types.iter().map(|t| t.hours).sum();
    near(
        &at("machine hours"),
        machine_hours,
        eq.machine_hours,
        FULL * eq.n_a,
    );
    // Shares of income, the real wage and the baskets.
    close(
        &at("labour + capital + land shares"),
        eq.labor_share + eq.capital_share + p.land / eq.income,
        1.0,
    );
    close(&at("v / P_s"), eq.v / eq.p_s, eq.real_wage);
    close(
        &at("worker baskets"),
        p.workers + eq.v * eq.n_a / eq.p_s,
        eq.worker_baskets,
    );
    close(
        &at("baskets = Y"),
        eq.worker_baskets + eq.provider_baskets,
        eq.y,
    );
    assert_eq!(eq.funded, eq.provider_baskets > 0.0, "{}", at("funded"));
    assert_eq!(
        eq.participation,
        (eq.n_a / p.workers).min(1.0),
        "{}",
        at("participation")
    );
    assert_eq!(eq.u, eq.types[eq.technique].user_cost);
    // φ, reported only when every u_k = 1: labour's share of the technique's price,
    // γλ̃_τ/θ_τ = vλ̃_τ/p_τ, and per type vλ̃_k/p_k.
    let all_flow = eq.types.iter().all(|t| t.user_cost == 1.0);
    match (eq.phi_w, eq.phi_r) {
        (Some(w), Some(r)) => {
            assert!(all_flow, "{}", at("phi at u != 1"));
            let t = &eq.types[eq.technique];
            let theta = types[eq.technique].task_efficiency;
            close(
                &at("phi_w = gamma lt/theta"),
                w,
                eq.gamma_star * t.lambda_tilde / theta,
            );
            close(&at("phi_w = v lt/p"), w, eq.v * t.lambda_tilde / t.price);
            assert_eq!(r, 1.0 - w);
        }
        (None, None) => assert!(!all_flow, "{}", at("no phi at u = 1")),
        other => panic!("{}: {other:?}", at("phi")),
    }
    for t in &eq.types {
        match t.phi_w {
            Some(w) => {
                assert_eq!(t.user_cost, 1.0);
                close(&at("type phi_w"), w, eq.v * t.lambda_tilde / t.price);
            }
            None => assert_ne!(t.user_cost, 1.0),
        }
    }
    // 1 − x*: carried from the root, or 1.0 − x_i at a tie.
    let spacing = |v: f64| (v - v.next_down()).max(v.next_up() - v);
    assert!(
        (eq.one_minus_x_star - (1.0 - eq.x_star)).abs()
            <= spacing(eq.x_star) + spacing(1.0 - eq.x_star),
        "{}",
        at("1 - x*")
    );
    if eq.tie.is_some() {
        assert_eq!(eq.one_minus_x_star, 1.0 - eq.x_star);
    }
    // The residuals: each at most FULL, and each its recomputation.
    let r = eq.residuals;
    for (name, value) in [
        ("income", r.income),
        ("land", r.land),
        ("services", r.services),
        ("user cost", r.user_cost),
        ("fork", r.fork),
        ("totals", r.totals),
        ("expenditure", r.expenditure),
        ("leontief price", r.leontief_price),
        ("leontief quantity", r.leontief_quantity),
        ("closure", r.closure),
        ("cheapest", r.cheapest),
        ("labour / N_a", r.labor / eq.n_a),
    ] {
        assert!(value <= FULL, "{name} residual {value:e} {}", at(""));
    }
    assert_eq!(recomputed_1c(economy, eq), r, "{}", at("residuals"));
}

/// Both forms of the fork identity, the bounds and the pair with chain totals, for every
/// category, and v/P_s ≤ v/B_s (docs/unit-1c.md §4.4).
pub fn check_fork_and_bounds_1c(economy: &MachineEconomy, eq: &Eq1c) {
    let p = economy.params();
    for (j, c) in eq.categories.iter().enumerate() {
        let tag = |what: &str| format!("category {j}: {what} at {p:?}");
        let b_bar = c.chain_land;
        close(&tag("p = v L* + b-bar"), eq.v * c.l_star + b_bar, c.price);
        close(
            &tag("p = v lt + bt"),
            eq.v * c.lambda_tilde + c.b_tilde,
            c.price,
        );
        close(&tag("v/p"), eq.v / c.price, c.real_wage);
        at_most(&tag("b-bar <= bt^q"), b_bar, c.b_tilde_q);
        at_most(&tag("bt^q <= bt"), c.b_tilde_q, c.b_tilde);
        at_most(&tag("bt <= p"), c.b_tilde, c.price);
        at_most(&tag("p <= v Lbar + b-bar"), c.price, eq.v * c.l_bar + b_bar);
        at_most(&tag("L* <= Lbar"), c.l_star, c.l_bar);
        at_most(&tag("lt^q <= lt"), c.lambda_tilde_q, c.lambda_tilde);
        close(&tag("floor"), 1.0 / (c.l_bar + b_bar / eq.v), c.wage_floor);
        at_most(&tag("floor <= v/p"), c.wage_floor, c.real_wage);
        match c.wage_ceiling {
            Some(ceiling) => {
                assert!(c.b_tilde > 0.0, "{}", tag("ceiling without land"));
                close(&tag("ceiling"), eq.v / c.b_tilde, ceiling);
                at_most(&tag("v/p <= v/bt"), c.real_wage, ceiling);
            }
            None => assert_eq!(c.b_tilde, 0.0, "{}", tag("ceiling")),
        }
        if b_bar == 0.0 {
            at_most(&tag("v/p >= 1/Lbar"), 1.0 / c.l_bar, c.real_wage);
        }
        let all_flow = eq.types.iter().all(|t| t.user_cost == 1.0);
        match (c.phi_w, c.phi_r) {
            (Some(w), Some(r)) => {
                assert!(all_flow);
                close(&tag("phi_w + phi_r"), w + r, 1.0);
            }
            (None, None) => assert!(!all_flow),
            other => panic!("{}: {other:?}", tag("phi")),
        }
    }
    at_most("v/P_s <= v/B_s", eq.real_wage, eq.rent_ceiling);
    close("rent ceiling", eq.v / eq.b_s, eq.rent_ceiling);
}

/// The technique regions of an economy's bracket: (x_i, x_{i+1}, τ_i) for each region, from
/// the switch points (docs/unit-1c.md §5.3).
pub fn regions(economy: &MachineEconomy) -> Vec<(f64, f64, usize)> {
    let points = economy.switch_points().expect("switch points");
    let env = economy.envelope();
    let techniques: Vec<usize> = std::iter::once(env.first)
        .chain(env.switches.iter().map(|s| s.above))
        .collect();
    let bounds: Vec<f64> = std::iter::once(BRACKET_LO)
        .chain(points)
        .chain(std::iter::once(1.0))
        .collect();
    (0..techniques.len())
        .map(|i| (bounds[i], bounds[i + 1], techniques[i]))
        .collect()
}

/// Within each region f is nonincreasing on a grid, n_D nonincreasing and n_S nondecreasing
/// (Lemma 2); returns the number of equilibria the sign sequence shows (§5.3 step 3): its
/// changes of side between a positive value before f(lo) and a nonpositive one after f(1),
/// with f(1) on the positive side when ≥ 0 (1a's BoundaryNoMargin). A change at the first is
/// the corner at lo (NoInteriorAtZero), at the last the corner at 1 (BoundaryNoMargin).
pub fn check_regions(economy: &MachineEconomy) -> usize {
    const GRID: usize = 40;
    let mut sequence = Vec::new();
    for (lo, hi, t) in regions(economy) {
        let points: Vec<_> = (0..=GRID)
            .map(|i| {
                let x = if i == GRID {
                    hi
                } else {
                    lo + (hi - lo) * i as f64 / GRID as f64
                };
                economy.at_with(x, t)
            })
            .collect();
        for w in points.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            let slack = 4.0 * f64::EPSILON * a.n_d.max(a.n_s);
            assert!(b.n_d <= a.n_d + slack, "n_D rises at x = {} under {t}", b.x);
            assert!(b.n_s + slack >= a.n_s, "n_S falls at x = {} under {t}", b.x);
        }
        sequence.push(points[0].excess_demand());
        sequence.push(points[GRID].excess_demand());
    }
    let last = sequence.len() - 1;
    let mut sides = vec![true];
    for (i, f) in sequence.into_iter().enumerate() {
        sides.push(if i == last { f >= 0.0 } else { f > 0.0 });
    }
    sides.push(false);
    sides.windows(2).filter(|w| w[0] != w[1]).count()
}

/// x is where f under technique t changes sign, to one double, and the better end.
pub fn assert_root_1c(economy: &MachineEconomy, x: f64, t: usize) {
    let f = |x: f64| economy.at_with(x, t).excess_demand();
    let fx = f(x);
    if fx == 0.0 {
        return;
    }
    let other = if fx > 0.0 { x.next_up() } else { x.next_down() };
    let f_other = f(other);
    assert!(
        f_other == 0.0 || f_other.signum() != fx.signum(),
        "no sign change between {x:e} (f = {fx:e}) and {other:e} (f = {f_other:e})"
    );
    assert!(
        fx.abs() <= f_other.abs(),
        "x* = {x:e} is not the better end"
    );
}

/// A category used only by the unit-1c tests' constructions.
pub fn site(weight: f64, segments: usize) -> Category {
    category(weight, 1.0, &vec![0.0; segments])
}

/// The Leontief identities alone, by multiplication over the C + K rows (SSRN eq 3-4, A.1 and
/// App. C; docs/unit-1c.md §4.5): p = Ap + λv + b with the price-side machine rows,
/// (I − Â)λ̃ = λ̂ and (I − Â)b̃ = b̂, f = (I − A^qᵀ)y = (Yz, 0), N_a = λ^qᵀy, T = b^qᵀy and
/// pᵀf = vN_a + T + interest.
pub fn check_leontief_1c(economy: &MachineEconomy, eq: &Eq1c) {
    let p = economy.params();
    let types = &p.machine_types;
    let (k_count, count) = (types.len(), p.categories.len());
    let q = economy.at_with(eq.x_star, eq.technique);
    let at = |what: &str| format!("{what} at {p:?}");
    let u = |k: usize| eq.types[k].user_cost;
    let a_hat =
        |k: usize, l: usize| types[k].operating.machines[l] + u(k) * types[k].build.machines[l];
    let a_q = |k: usize, l: usize| {
        types[k].operating.machines[l] + types[k].delta * types[k].build.machines[l]
    };
    // The price side, row by row.
    for j in 0..count {
        let mut cost = q.human[j] * eq.v + p.categories[j].direct_land;
        for l in 0..count {
            cost += p.intermediate[j][l] * eq.categories[l].price;
        }
        for (k, t) in types.iter().enumerate() {
            let share = task_share(eq, k);
            if share > 0.0 {
                cost += share * q.machine[j] / t.task_efficiency * eq.types[k].price;
            }
        }
        close(
            &at("p = Ap + lv + b, category row"),
            cost,
            eq.categories[j].price,
        );
    }
    for k in 0..k_count {
        let (op, build) = (&types[k].operating, &types[k].build);
        let (mut cost, mut l_row, mut b_row) = (0.0, eq.types[k].lambda_tilde, eq.types[k].b_tilde);
        for l in 0..k_count {
            cost += a_hat(k, l) * eq.types[l].price;
            l_row -= a_hat(k, l) * eq.types[l].lambda_tilde;
            b_row -= a_hat(k, l) * eq.types[l].b_tilde;
        }
        cost += (op.labor + u(k) * build.labor) * eq.v + (op.land + u(k) * build.land);
        close(&at("p = Ap + lv + b, machine row"), cost, eq.types[k].price);
        let scale = eq.types[k].lambda_tilde + eq.types[k].b_tilde;
        near(
            &at("(I - A-hat) lt = l-hat"),
            l_row,
            op.labor + u(k) * build.labor,
            FULL * scale,
        );
        near(
            &at("(I - A-hat) bt = b-hat"),
            b_row,
            op.land + u(k) * build.land,
            FULL * scale,
        );
    }
    // The clearing side: f = (I − A^qᵀ)y.
    let gross: Vec<f64> = eq.categories.iter().map(|c| c.gross_output).collect();
    let services: Vec<f64> = eq.types.iter().map(|t| t.services).collect();
    let biggest = gross
        .iter()
        .chain(&services)
        .copied()
        .fold(0.0f64, f64::max);
    let mut f = Vec::with_capacity(count + k_count);
    for j in 0..count {
        let mut used = 0.0;
        for l in 0..count {
            used += p.intermediate[l][j] * gross[l];
        }
        f.push(gross[j] - used);
        near(
            &at("f_j = Y z_j"),
            f[j],
            p.categories[j].weight * eq.y,
            FULL * biggest,
        );
    }
    for k in 0..k_count {
        let mut used = 0.0;
        let share = task_share(eq, k);
        if share > 0.0 {
            for j in 0..count {
                used += gross[j] * share * q.machine[j] / types[k].task_efficiency;
            }
        }
        for l in 0..k_count {
            used += a_q(l, k) * services[l];
        }
        f.push(services[k] - used);
        near(&at("f_machine = 0"), f[count + k], 0.0, FULL * biggest);
    }
    let (mut hours, mut land, mut pf) = (0.0, 0.0, 0.0);
    for j in 0..count {
        hours += eq.categories[j].human * gross[j];
        land += p.categories[j].direct_land * gross[j];
        pf += eq.categories[j].price * f[j];
    }
    for k in 0..k_count {
        let (op, build) = (&types[k].operating, &types[k].build);
        hours += (op.labor + types[k].delta * build.labor) * services[k];
        land += (op.land + types[k].delta * build.land) * services[k];
        pf += eq.types[k].price * f[count + k];
    }
    near(&at("N_a = l^q' y"), hours, eq.n_a, FULL * eq.n_a);
    near(&at("T = b^q' y"), land, p.land, FULL * p.land);
    near(
        &at("p'f = v N_a + T + interest"),
        pf,
        eq.v * eq.n_a + p.land + eq.interest,
        FULL * eq.income,
    );
}
