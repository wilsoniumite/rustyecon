//! Fixtures and checks shared by unit 1d's tests (docs/unit-1d.md §8).

// The identities are sums over categories, machine types and worker types, and the checks
// index several vectors at once, as docs/unit-1d.md §4.7 writes them.
#![allow(clippy::needless_range_loop)]

use std::collections::BTreeMap;

use oracle::{
    CategoryParams, Eq1d, MachineParams, MachineType, Margin, Output1b, Params, PowerSchedule,
    Recipe, Regime, Residuals1d, Schedule, SolveError, UniformWorkCost, WorkerEconomy,
    WorkerParams, WorkerPoint, WorkerType, BRACKET_LO,
};

use crate::support::*;
use crate::support_1b::*;
use crate::support_1c::*;

/// A worker type (N, χ_max, ε, ν), as docs/unit-1d.md §3.3 writes them.
pub fn worker(workers: f64, chi_max: f64, efficiency: f64, support: f64) -> WorkerType {
    WorkerType {
        workers,
        work_cost: UniformWorkCost { chi_max },
        efficiency,
        support,
    }
}

/// D0: a unit-1a economy in worker form, through 1b's and 1c's forms.
pub fn from_1a(params: Params) -> WorkerParams {
    WorkerParams::from_machines(MachineParams::from_categories(
        CategoryParams::from_one_category(params),
    ))
}

/// Validates unit-1d parameters the test knows to be valid.
pub fn economy_1d(params: WorkerParams) -> WorkerEconomy {
    WorkerEconomy::new(params.clone()).unwrap_or_else(|e| panic!("{params:?}: {e}"))
}

/// Solves a unit-1d economy the test knows to have one equilibrium.
pub fn solved_1d(params: WorkerParams) -> Box<Eq1d> {
    match economy_1d(params.clone()).solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{params:?} has no equilibrium: {other:?}"),
    }
}

/// Solves, checks every identity of §4.7, and returns the equilibrium and the economy.
pub fn checked_1d(params: WorkerParams) -> (WorkerEconomy, Box<Eq1d>) {
    let e = economy_1d(params.clone());
    let eq = solved_1d(params);
    check_identities_1d(&e, &eq);
    (e, eq)
}

/// Every output of a unit-1d equilibrium, by printed key, as bits.
pub fn bits_1d(eq: &Eq1d) -> BTreeMap<String, Option<u64>> {
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

/// W (docs/unit-1d.md §3.3): 1a's G1 with N, λ and χ_max changed, in one-type form.
pub fn goodspace(workers: f64, lam: f64, chi_max: f64) -> WorkerParams {
    from_1a(Params {
        workers,
        lam,
        work_cost: UniformWorkCost { chi_max },
        ..appendix_b()
    })
}

/// Appendix B's machine as a unit-1c type: build (0.3; 0.05; 0.4), δ 1, J 1.
pub fn appendix_b_machine() -> MachineType {
    machine_type(1.0, Recipe::zero(1), recipe(&[0.3], 0.05, 0.4), 1.0, 1)
}

/// B (docs/unit-1d.md §3.3): services (z 1, μ 0.75, L^H `required`), goods (z 1, μ 1) and space
/// (z 1, b 1) on one segment, Appendix B's machine, T 10, γ = η(0.2 + 0.8x), ρ 0, with the
/// worker types and reserved hours (one row per category) given.
pub fn baumol(
    eta: f64,
    worker_types: Vec<WorkerType>,
    required: f64,
    reserved: Vec<Vec<f64>>,
) -> WorkerParams {
    WorkerParams {
        land: 10.0,
        schedule: linear(eta, 0.2, 0.8),
        rho: 0.0,
        machine_types: vec![appendix_b_machine()],
        edges: vec![0.0, 1.0],
        categories: vec![
            category(1.0, 0.0, &[0.75]),
            category(1.0, 0.0, &[1.0]),
            category(1.0, 1.0, &[0.0]),
        ],
        intermediate: no_inputs(3),
        worker_types,
        human_required: vec![required, 0.0, 0.0],
        reserved,
    }
}

/// B's economy with one type of N workers (χ_max 1, ε 1, ν 1) and the tail L^H.
pub fn baumol_one(eta: f64, workers: f64, required: f64) -> WorkerParams {
    baumol(
        eta,
        vec![worker(workers, 1.0, 1.0, 1.0)],
        required,
        vec![vec![0.0]; 3],
    )
}

/// E (docs/unit-1d.md §3.3): the entrant (N_E, χ_E, 1, 1) and the trained (N_T, 0.8, 1.5, 1.2)
/// with reserved hours 0.1 per unit of services and 0.02 per unit of goods, on B's economy.
pub fn entrant_trained(entrants: f64, trained: f64, eta: f64, chi_entrant: f64) -> WorkerParams {
    baumol(
        eta,
        vec![
            worker(entrants, chi_entrant, 1.0, 1.0),
            worker(trained, 0.8, 1.5, 1.2),
        ],
        0.25,
        vec![vec![0.0, 0.1], vec![0.0, 0.02], vec![0.0, 0.0]],
    )
}

/// E7 and E8: E's economy with the master (N_M, 0.6, 1.8, 1.5), reserved 0.05 per unit of goods,
/// and the trained's reserved hours 0.1 per unit of services only; N_E 8.
pub fn three_types(trained: f64, masters: f64) -> WorkerParams {
    baumol(
        1.0,
        vec![
            worker(8.0, 1.0, 1.0, 1.0),
            worker(trained, 0.8, 1.5, 1.2),
            worker(masters, 0.6, 1.8, 1.5),
        ],
        0.25,
        vec![
            vec![0.0, 0.1, 0.0],
            vec![0.0, 0.0, 0.05],
            vec![0.0, 0.0, 0.0],
        ],
    )
}

/// F (docs/unit-1d.md §3.3): 1c's M4 at η with L^H_care 0.2, the entrant (N_E, 1, 1, 1) and the
/// trained (N_T, 0.8, 1.4, 1.2) with reserved hours 0.05 per unit of food and 0.3 per unit of
/// care.
pub fn full(entrants: f64, trained: f64, eta: f64) -> WorkerParams {
    WorkerParams {
        worker_types: vec![
            worker(entrants, 1.0, 1.0, 1.0),
            worker(trained, 0.8, 1.4, 1.2),
        ],
        human_required: vec![0.0, 0.0, 0.2, 0.0],
        reserved: vec![
            vec![0.0, 0.0],
            vec![0.0, 0.05],
            vec![0.0, 0.3],
            vec![0.0, 0.0],
        ],
        ..WorkerParams::from_machines(m4(eta))
    }
}

/// X (docs/unit-1d.md §3.3): 1c's M5 at ρ 0.15 and χ_max 3, at N, in one-type form.
pub fn wall_switch_economy(workers: f64) -> WorkerParams {
    WorkerParams::from_machines(MachineParams {
        work_cost: UniformWorkCost { chi_max: 3.0 },
        ..m5(0.15, workers, 1.0)
    })
}

/// The evaluation that priced an equilibrium: the line's at x* on the line, the wage-given one
/// at a corner, under the reported technique.
pub fn point_of(economy: &WorkerEconomy, eq: &Eq1d) -> WorkerPoint {
    match eq.margin {
        Margin::Contestable => economy.at_with(eq.x_star, eq.technique),
        _ => economy.at_wage(eq.x_star, eq.v, eq.technique),
    }
}

/// The share of the machine tasks type k does at the equilibrium.
pub fn task_share_1d(eq: &Eq1d, k: usize) -> f64 {
    match eq.tie {
        None if k == eq.technique => 1.0,
        Some(t) if k == eq.technique => 1.0 - t.share,
        Some(t) if k == t.above => t.share,
        _ => 0.0,
    }
}

/// The residuals of docs/unit-1d.md §6, recomputed from the reported fields and the parameters
/// in the solve's own form: the same operations on the same doubles, so they must agree bit
/// for bit.
pub fn recomputed_1d(economy: &WorkerEconomy, eq: &Eq1d) -> Residuals1d {
    let p = economy.params();
    let types = &p.machine_types;
    let (k_count, count) = (types.len(), p.categories.len());
    let q = point_of(economy, eq);
    let worse = |a: f64, b: f64| if b.is_nan() || b > a { b } else { a };
    let a_q = |k: usize, l: usize| {
        types[k].operating.machines[l] + types[k].delta * types[k].build.machines[l]
    };
    let (mut fork, mut totals, mut spending) = (0.0f64, 0.0f64, 0.0);
    for c in &eq.categories {
        fork = worse(
            fork,
            (c.price - ((eq.v * c.l_star + c.chain_land) + c.reserved_cost)).abs() / c.price,
        );
        totals = worse(
            totals,
            (c.price - ((eq.v * c.lambda_tilde + c.b_tilde) + c.reserved_cost)).abs() / c.price,
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
            let share = task_share_1d(eq, k);
            if share > 0.0 {
                inputs += (share * (q.machine[j] / t.task_efficiency)) * eq.types[k].price;
            }
        }
        let mut reserved = 0.0;
        for (i, w) in eq.workers.iter().enumerate() {
            reserved += w.wage * p.reserved[j][i];
        }
        let cost = (inputs + q.human[j] * eq.v + p.categories[j].direct_land) + reserved;
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
    let yhat = economy.machines().basket_outputs();
    let gross: Vec<f64> = yhat.iter().map(|b| b * eq.y).collect();
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
            let share = task_share_1d(eq, k);
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
    let closure = if eq.margin == Margin::Contestable {
        let tau = &eq.types[eq.technique];
        let theta = types[eq.technique].task_efficiency;
        let wage = (eq.gamma_star * tau.b_tilde) / (theta - eq.gamma_star * tau.lambda_tilde);
        (eq.v - wage).abs() / eq.v
    } else {
        0.0
    };
    let task_price = eq.types[eq.technique].price / types[eq.technique].task_efficiency;
    let mut cheapest = 0.0f64;
    for (k, t) in types.iter().enumerate() {
        if t.task_efficiency > 0.0 {
            let undercut = (task_price - eq.types[k].price / t.task_efficiency) / task_price;
            cheapest = worse(cheapest, undercut.max(0.0));
        }
    }
    let corner = match eq.margin {
        Margin::Contestable => 0.0,
        Margin::Wall => (eq.replacement_top - eq.v).max(0.0) / eq.v,
        Margin::AllHuman => (eq.v - eq.replacement_bottom).max(0.0) / eq.v,
    };
    let mut reserved = 0.0f64;
    let mut pool = 0.0;
    for w in &eq.workers {
        if w.pooled {
            pool += w.efficiency * (w.supply - w.reserved_hours);
        } else {
            reserved = worse(
                reserved,
                (w.supply - w.reserved_hours).abs() / w.reserved_hours,
            );
        }
    }
    let mut spent = 0.0;
    for (c, cat) in eq.categories.iter().zip(&p.categories) {
        spent += cat.weight * c.price;
    }
    Residuals1d {
        labor: (eq.n_pool - pool).abs(),
        income: (eq.y * eq.p_s - eq.income).abs() / eq.income,
        land: (eq.b_d * eq.y + machine_land - p.land).abs() / p.land,
        services,
        user_cost,
        fork,
        totals,
        expenditure: (spending - eq.income).abs() / eq.income,
        leontief_price,
        leontief_quantity,
        closure,
        cheapest,
        corner,
        reserved,
        basket: (eq.p_s - spent).abs() / eq.p_s,
    }
}

/// Every identity and bound of docs/unit-1d.md §4.7 at an equilibrium, recomputed here by
/// multiplication from the reported fields and the parameters, and the prices pinned to the
/// evaluation that set them (`at_with` on the line, `at_wage` at a corner) bit for bit (§8).
pub fn check_identities_1d(economy: &WorkerEconomy, eq: &Eq1d) {
    let p = economy.params();
    let types = &p.machine_types;
    let (k_count, count) = (types.len(), p.categories.len());
    let at = |what: &str| format!("{what} at {p:?}");
    let q = point_of(economy, eq);
    // The prices, bit for bit.
    assert_eq!(eq.v.to_bits(), q.v.to_bits(), "{}", at("v"));
    assert_eq!(eq.d.to_bits(), q.d.to_bits(), "{}", at("d"));
    assert!(eq.d > 0.0, "{}", at("viable"));
    for k in 0..k_count {
        assert_eq!(
            eq.types[k].price.to_bits(),
            q.type_prices[k].to_bits(),
            "{}",
            at("p_k")
        );
    }
    for j in 0..count {
        let c = &eq.categories[j];
        assert_eq!(
            c.price.to_bits(),
            (q.base_prices[j] + c.reserved_cost).to_bits(),
            "{}",
            at("p_j = p0_j + R_j")
        );
        assert_eq!(c.machine.to_bits(), q.machine[j].to_bits(), "{}", at("M_j"));
    }
    if eq.tie.is_none() {
        assert_eq!(eq.p_s.to_bits(), q.p_s.to_bits(), "{}", at("P_s"));
        for j in 0..count {
            assert_eq!(
                eq.categories[j].price.to_bits(),
                q.prices[j].to_bits(),
                "{}",
                at("p_j")
            );
        }
    }
    let task_price = eq.types[eq.technique].price / types[eq.technique].task_efficiency;
    close(&at("g = v/pi"), eq.v / task_price, eq.g);
    close(
        &at("replacement top"),
        p.schedule.gamma(1.0) * task_price,
        eq.replacement_top,
    );
    close(
        &at("replacement bottom"),
        p.schedule.gamma(0.0) * task_price,
        eq.replacement_bottom,
    );
    // The margin, or the corner's inequality (§4.7).
    match eq.margin {
        Margin::Contestable => {
            let tau = &eq.types[eq.technique];
            let theta = types[eq.technique].task_efficiency;
            close(
                &at("closure"),
                eq.gamma_star * tau.b_tilde / (theta - eq.gamma_star * tau.lambda_tilde),
                eq.v,
            );
            close(&at("v = gamma pi"), eq.gamma_star * task_price, eq.v);
            assert!(eq.x_star <= 1.0 && eq.x_star >= 0.0);
        }
        Margin::Wall => {
            assert_eq!(
                (eq.x_star, eq.one_minus_x_star),
                (1.0, 0.0),
                "{}",
                at("wall")
            );
            at_most(&at("v >= gamma(1) pi"), eq.replacement_top, eq.v);
            assert!(!eq.margin_active);
            // No contestable task is human: the final hours are the tail alone.
            assert_eq!(eq.final_hours.to_bits(), eq.required_hours.to_bits());
        }
        Margin::AllHuman => {
            assert_eq!(
                (eq.x_star, eq.one_minus_x_star),
                (0.0, 1.0),
                "{}",
                at("all-human")
            );
            at_most(&at("v <= gamma(0) pi"), eq.v, eq.replacement_bottom);
            assert!(!eq.margin_active);
            assert_eq!(eq.m_s, 0.0);
            assert_eq!(eq.interest, 0.0);
            assert_eq!(eq.machine_hours, 0.0);
            for t in &eq.types {
                assert_eq!(t.services, 0.0, "{}", at("no machine services"));
            }
        }
    }
    // The cheapest task type (SSRN A.1).
    for k in 0..k_count {
        if types[k].task_efficiency > 0.0 {
            at_most(
                &at("no task type is cheaper"),
                task_price,
                eq.types[k].price / types[k].task_efficiency,
            );
        }
    }
    // The machine rows (check_dynamics R1-R3).
    let u = |k: usize| eq.types[k].user_cost;
    for k in 0..k_count {
        let (op, build) = (&types[k].operating, &types[k].build);
        let t = &eq.types[k];
        let (mut o, mut v_k) = (0.0, 0.0);
        for l in 0..k_count {
            o += op.machines[l] * eq.types[l].price;
            v_k += build.machines[l] * eq.types[l].price;
        }
        near(
            &at("O_k"),
            t.operating_cost,
            o + op.labor * eq.v + op.land,
            FULL * t.price,
        );
        near(
            &at("V_k"),
            t.build_cost,
            v_k + build.labor * eq.v + build.land,
            FULL * t.price.max(t.build_cost),
        );
        close(
            &at("p_k = O + u V"),
            t.operating_cost + u(k) * t.build_cost,
            t.price,
        );
        close(
            &at("p_k = v lt + bt"),
            eq.v * t.lambda_tilde + t.b_tilde,
            t.price,
        );
    }
    // The worker types (§4.3, §4.7; SSRN eq 8-10 per type).
    let mut pool_net = 0.0;
    let mut n_a = 0.0;
    let mut wage_bill = eq.v * eq.n_pool;
    let mut support = 0.0;
    let mut reserved_hours = 0.0;
    for (i, w) in eq.workers.iter().enumerate() {
        let t = &p.worker_types[i];
        let d_i = eq.y * economy.reserved_per_basket()[i];
        close(
            &at("D_i = Y R_i"),
            d_i.max(1e-300),
            w.reserved_hours.max(1e-300),
        );
        close(&at("real wage"), w.wage / eq.p_s, w.real_wage);
        assert_eq!(
            w.marginal_work_cost.to_bits(),
            rustyecon_core::num::ln1p(w.wage / (t.support * eq.p_s)).to_bits()
        );
        near(
            &at("supply"),
            w.supply,
            t.workers * t.work_cost.cdf(w.marginal_work_cost),
            FULL * t.workers,
        );
        // The exit value at the marginal work cost is the wage (SSRN eq 9 per type).
        close(
            &at("exit value"),
            rustyecon_core::num::expm1(w.marginal_work_cost) * t.support * eq.p_s,
            w.wage,
        );
        assert_eq!(w.efficiency, t.efficiency);
        if w.pooled {
            assert_eq!(
                w.wage.to_bits(),
                (t.efficiency * eq.v).to_bits(),
                "{}",
                at("pooled wage")
            );
            assert_eq!(w.premium, Some(1.0), "{}", at("premium"));
            at_most(&at("pooled covers reserved"), w.reserved_hours, w.supply);
            let clearing = if t.efficiency > 0.0 {
                t.efficiency * eq.real_wage / t.support
            } else {
                0.0
            };
            at_most(
                &at("pooled above its threshold"),
                w.clearing_real_wage,
                clearing,
            );
            close(
                &at("hours = D + pool"),
                w.reserved_hours + w.pool_hours,
                w.hours,
            );
            pool_net += t.efficiency * (w.supply - w.reserved_hours);
        } else {
            if t.efficiency > 0.0 {
                assert!(
                    w.wage > t.efficiency * eq.v,
                    "{}",
                    at("walled wage above pooled")
                );
                assert!(w.premium.unwrap() > 1.0);
            } else {
                assert_eq!(w.premium, None);
            }
            assert_eq!(w.hours, w.reserved_hours, "{}", at("walled hours"));
            assert_eq!(w.pool_hours, 0.0);
            close(&at("reserved clears"), w.supply, w.reserved_hours);
            close(
                &at("walled wage = zeta nu P_s"),
                w.clearing_real_wage * t.support * eq.p_s,
                w.wage,
            );
            assert!(w.at_wall);
        }
        assert_eq!(w.at_wall, !w.pooled || eq.margin == Margin::Wall);
        near(
            &at("participation"),
            w.participation,
            (w.hours / t.workers).min(1.0),
            4.0 * f64::EPSILON,
        );
        n_a += w.hours;
        wage_bill += w.wage * w.reserved_hours;
        support += t.support * t.workers;
        reserved_hours += w.reserved_hours;
    }
    let pooled_hours: f64 = eq.workers.iter().map(|w| w.efficiency * w.pool_hours).sum();
    near(
        &at("pool hours sum to n_pool"),
        pooled_hours,
        eq.n_pool,
        FULL * eq.n_pool.max(1.0),
    );
    near(
        &at("pool clears"),
        pool_net,
        eq.n_pool,
        FULL * eq.n_pool.max(1.0),
    );
    close(&at("n_a"), n_a, eq.n_a);
    close(&at("wage bill"), wage_bill, eq.wage_bill);
    near(
        &at("reserved hours"),
        reserved_hours,
        eq.reserved_hours,
        FULL * eq.n_a,
    );
    assert_eq!(support.to_bits(), economy.support().to_bits());
    assert_eq!(
        eq.required_hours.to_bits(),
        (eq.y * economy.human_required_per_basket()).to_bits()
    );
    close(&at("n_pool"), eq.final_hours + eq.machine_hours, eq.n_pool);
    near(
        &at("final hours"),
        eq.y * eq.h_s,
        eq.final_hours,
        FULL * eq.n_pool.max(1e-300),
    );
    // The categories: the price rows with reserved costs, the chain, both forks, the bounds.
    let gross: Vec<f64> = eq.categories.iter().map(|c| c.gross_output).collect();
    for j in 0..count {
        let c = &eq.categories[j];
        let tag = |what: &str| format!("category {j}: {what} at {p:?}");
        let mut cost = q.human[j] * eq.v + p.categories[j].direct_land;
        for l in 0..count {
            cost += p.intermediate[j][l] * eq.categories[l].price;
        }
        for (k, t) in types.iter().enumerate() {
            let share = task_share_1d(eq, k);
            if share > 0.0 {
                cost += share * q.machine[j] / t.task_efficiency * eq.types[k].price;
            }
        }
        let mut direct_reserved = 0.0;
        for (i, w) in eq.workers.iter().enumerate() {
            direct_reserved += w.wage * p.reserved[j][i];
        }
        close(&tag("p = Ap + lv + b + R"), cost + direct_reserved, c.price);
        // The chain of reserved costs and of the common hours, by multiplication.
        let mut chain_r = c.reserved_cost;
        let mut chain_h = c.human_required;
        for l in 0..count {
            chain_r -= p.intermediate[j][l] * eq.categories[l].reserved_cost;
            chain_h -= p.intermediate[j][l] * eq.categories[l].human_required;
        }
        near(
            &tag("(I - A) R = direct reserved"),
            chain_r,
            direct_reserved,
            FULL * c.price,
        );
        near(
            &tag("(I - A) L^H = L^H"),
            chain_h,
            p.human_required[j],
            FULL * c.l_bar.max(1e-300),
        );
        close(
            &tag("l_bar"),
            economy.machines().all_human_hours()[j] + c.human_required,
            c.l_bar,
        );
        close(
            &tag("fork direct"),
            eq.v * c.l_star + c.chain_land + c.reserved_cost,
            c.price,
        );
        close(
            &tag("fork totals"),
            eq.v * c.lambda_tilde + c.b_tilde + c.reserved_cost,
            c.price,
        );
        at_most(&tag("b-bar <= bt^q"), c.chain_land, c.b_tilde_q);
        at_most(&tag("bt^q <= bt"), c.b_tilde_q, c.b_tilde);
        at_most(&tag("bt + R <= p"), c.b_tilde + c.reserved_cost, c.price);
        at_most(
            &tag("p <= v (Lbar + L^H) + b-bar + R"),
            c.price,
            eq.v * c.l_bar + c.chain_land + c.reserved_cost,
        );
        at_most(&tag("L* <= Lbar + L^H"), c.l_star, c.l_bar);
        at_most(&tag("lt^q <= lt"), c.lambda_tilde_q, c.lambda_tilde);
        if eq.margin == Margin::AllHuman {
            close(
                &tag("the upper bound is attained"),
                eq.v * c.l_bar + c.chain_land + c.reserved_cost,
                c.price,
            );
        }
        close(&tag("v/p"), eq.v / c.price, c.real_wage);
        close(
            &tag("floor"),
            1.0 / (c.l_bar + (c.chain_land + c.reserved_cost) / eq.v),
            c.wage_floor,
        );
        at_most(&tag("floor <= v/p"), c.wage_floor, c.real_wage);
        match c.wage_ceiling {
            Some(ceiling) => {
                close(
                    &tag("ceiling"),
                    eq.v / (c.b_tilde + c.reserved_cost),
                    ceiling,
                );
                at_most(&tag("v/p <= ceiling"), c.real_wage, ceiling);
            }
            None => assert_eq!(c.b_tilde + c.reserved_cost, 0.0),
        }
        if let (Some(w), Some(r)) = (c.phi_w, c.phi_r) {
            close(
                &tag("phi_w"),
                (eq.v * c.lambda_tilde + c.reserved_cost) / c.price,
                w,
            );
            close(&tag("phi_w + phi_r"), w + r, 1.0);
        }
        close(
            &tag("gross output"),
            economy.machines().basket_outputs()[j] * eq.y,
            c.gross_output,
        );
        assert_eq!(
            c.output.to_bits(),
            (p.categories[j].weight * eq.y).to_bits()
        );
        near(
            &tag("final hours"),
            c.final_hours,
            c.gross_output * c.human,
            FULL * eq.n_pool.max(1.0),
        );
    }
    // The clearing side: f = (I − A^qᵀ)y, n_pool = λ^qᵀy, T = b^qᵀy, and pᵀf.
    let services: Vec<f64> = eq.types.iter().map(|t| t.services).collect();
    let biggest = gross
        .iter()
        .chain(&services)
        .copied()
        .fold(0.0f64, f64::max);
    let a_q = |k: usize, l: usize| {
        types[k].operating.machines[l] + types[k].delta * types[k].build.machines[l]
    };
    let (mut hours, mut land, mut pf) = (0.0, 0.0, 0.0);
    for j in 0..count {
        let mut used = 0.0;
        for l in 0..count {
            used += p.intermediate[l][j] * gross[l];
        }
        let f_j = gross[j] - used;
        near(
            &at("f_j = Y z_j"),
            f_j,
            p.categories[j].weight * eq.y,
            FULL * biggest,
        );
        hours += eq.categories[j].human * gross[j];
        land += p.categories[j].direct_land * gross[j];
        pf += eq.categories[j].price * f_j;
    }
    for k in 0..k_count {
        let mut used = 0.0;
        let share = task_share_1d(eq, k);
        if share > 0.0 {
            for j in 0..count {
                used += gross[j] * share * eq.categories[j].machine / types[k].task_efficiency;
            }
        }
        near(
            &at("task services"),
            eq.types[k].task_services,
            used,
            FULL * biggest.max(1e-300),
        );
        for l in 0..k_count {
            used += a_q(l, k) * services[l];
        }
        let f_k = services[k] - used;
        near(&at("f_machine = 0"), f_k, 0.0, FULL * biggest.max(1e-300));
        let (op, build) = (&types[k].operating, &types[k].build);
        hours += (op.labor + types[k].delta * build.labor) * services[k];
        land += (op.land + types[k].delta * build.land) * services[k];
        pf += eq.types[k].price * f_k;
    }
    near(
        &at("n_pool = l^q' y"),
        hours,
        eq.n_pool,
        FULL * eq.n_pool.max(1.0),
    );
    near(&at("T = b^q' y"), land, p.land, FULL * p.land);
    near(
        &at("p'f = v n_pool + sum v_i D_i + T + interest"),
        pf,
        eq.wage_bill + p.land + eq.interest,
        FULL * eq.income,
    );
    // Income four ways (SSRN App. C with reserved labour), shares and the baskets.
    let supply_side: f64 = eq.workers.iter().map(|w| w.wage * w.hours).sum();
    near(
        &at("I = sum v_i hours_i + T + interest"),
        supply_side + p.land + eq.interest,
        eq.income,
        FULL * eq.income,
    );
    near(
        &at("I = wage bill + T + interest"),
        eq.wage_bill + p.land + eq.interest,
        eq.income,
        FULL * eq.income,
    );
    near(&at("I = Y P_s"), eq.y * eq.p_s, eq.income, FULL * eq.income);
    let spending: f64 = eq.categories.iter().map(|c| c.price * c.output).sum();
    near(&at("I = sum p z Y"), spending, eq.income, FULL * eq.income);
    close(
        &at("labour share"),
        eq.wage_bill / eq.income,
        eq.labor_share,
    );
    close(
        &at("shares"),
        eq.labor_share + eq.capital_share + p.land / eq.income,
        1.0,
    );
    close(
        &at("support cost"),
        economy.support() * eq.p_s,
        eq.support_cost,
    );
    close(
        &at("worker baskets"),
        economy.support() + eq.wage_bill / eq.p_s,
        eq.worker_baskets,
    );
    close(
        &at("baskets = Y"),
        eq.worker_baskets + eq.provider_baskets,
        eq.y,
    );
    assert_eq!(eq.funded, eq.provider_baskets > 0.0);
    let total: f64 = p.worker_types.iter().map(|t| t.workers).sum();
    near(
        &at("participation"),
        eq.participation,
        (eq.n_a / total).min(1.0),
        4.0 * f64::EPSILON,
    );
    // The basket (SSRN eq 7): P_s = z'p = v L_s + B_s + sum v_i R_i; eq 11 for the pool.
    let dot = |f: fn(&oracle::CategoryEq1d) -> f64| -> f64 {
        eq.categories
            .iter()
            .zip(&p.categories)
            .map(|(c, cat)| cat.weight * f(c))
            .sum()
    };
    close(&at("P_s = z p"), dot(|c| c.price), eq.p_s);
    let mut reserved_basket = 0.0;
    for (i, w) in eq.workers.iter().enumerate() {
        reserved_basket += w.wage * economy.reserved_per_basket()[i];
    }
    close(
        &at("P_s = v L_s + B_s + sum v_i R_i"),
        eq.v * eq.l_s + eq.b_s + reserved_basket,
        eq.p_s,
    );
    close(&at("L_s"), dot(|c| c.lambda_tilde), eq.l_s);
    close(&at("B_s"), dot(|c| c.b_tilde), eq.b_s);
    close(&at("L_s^q"), dot(|c| c.lambda_tilde_q), eq.l_s_q);
    close(&at("B_s^q"), dot(|c| c.b_tilde_q), eq.b_s_q);
    close(&at("Y = T/B_s^q"), p.land / eq.b_s_q, eq.y);
    close(
        &at("n_pool = T L_s^q/B_s^q"),
        p.land * eq.l_s_q / eq.b_s_q,
        eq.n_pool,
    );
    close(&at("real wage"), eq.v / eq.p_s, eq.real_wage);
    close(&at("rent ceiling"), eq.v / eq.b_s, eq.rent_ceiling);
    assert_eq!(eq.u, eq.types[eq.technique].user_cost);
    // The residuals: each small, and each its recomputation bit for bit.
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
        ("corner", r.corner),
        ("reserved", r.reserved),
        ("basket", r.basket),
        ("labour / n_pool", r.labor / eq.n_pool.max(1.0)),
    ] {
        assert!(value <= FULL, "{name} residual {value:e} {}", at(""));
    }
    assert_eq!(recomputed_1d(economy, eq), r, "{}", at("residuals"));
}

/// The sequence of docs/unit-1d.md §5.3 step 2 from the public evaluations, and its number of
/// changes of side: f(0), f(lo), each line switch (below, above), f(1), each wall switch
/// (below, above), f_∞, with a positive side first, f(1) and f_∞ positive when ≥ 0.
pub fn sequence_1d(economy: &WorkerEconomy) -> (Vec<f64>, usize) {
    let env = economy.machines().envelope();
    let f = |q: WorkerPoint| q.excess_demand();
    let mut values = vec![
        f(economy.at_with(0.0, env.first)),
        f(economy.at_with(BRACKET_LO, env.first)),
    ];
    let techniques: Vec<usize> = std::iter::once(env.first)
        .chain(env.switches.iter().map(|s| s.above))
        .collect();
    for (i, &x) in economy.switch_points().unwrap().iter().enumerate() {
        values.push(f(economy.at_with(x, techniques[i])));
        values.push(f(economy.at_with(x, techniques[i + 1])));
    }
    let one = values.len();
    values.push(f(economy.at_with(1.0, env.last())));
    for s in economy.wall_switches() {
        values.push(f(economy.at_wage(1.0, s.wage, s.below)));
        values.push(f(economy.at_wage(1.0, s.wage, s.above)));
    }
    let end = economy.wall_end().excess;
    values.push(end);
    let last = values.len() - 1;
    let mut sides = vec![true];
    for (i, &v) in values.iter().enumerate() {
        sides.push(if i == one || i == last {
            v >= 0.0
        } else {
            v > 0.0
        });
    }
    let changes = sides.windows(2).filter(|w| w[0] != w[1]).count();
    (values, changes)
}

/// f nonincreasing on a grid of every stretch within each technique (Lemmas 2' and 3'): the
/// all-human corner in v up to v(0), the line from 0 in each technique's region, and each piece
/// of the wall in v (the last up to 20 times its start). A short point (+∞) may only come first.
pub fn check_path_1d(economy: &WorkerEconomy) {
    const GRID: usize = 24;
    let p = economy.params();
    let env = economy.machines().envelope();
    let monotone = |what: &str, values: &[WorkerPoint]| {
        for w in values.windows(2) {
            let (a, b) = (w[0].excess_demand(), w[1].excess_demand());
            if a == f64::INFINITY {
                continue;
            }
            assert!(
                b != f64::INFINITY,
                "{what}: a short point after a finite one at {p:?}"
            );
            let slack = 8.0 * f64::EPSILON * w[0].n_d.max(w[0].n_s.abs()).max(1.0);
            assert!(
                b <= a + slack,
                "{what}: f rises from {a:e} to {b:e} at {p:?}"
            );
        }
    };
    let zero = economy.at_with(0.0, env.first);
    if zero.short.is_none() {
        let values: Vec<WorkerPoint> = (1..=GRID)
            .map(|i| economy.at_wage(0.0, zero.v * i as f64 / GRID as f64, env.first))
            .collect();
        monotone("all-human corner", &values);
    }
    let points = economy.switch_points().unwrap();
    let techniques: Vec<usize> = std::iter::once(env.first)
        .chain(env.switches.iter().map(|s| s.above))
        .collect();
    let bounds: Vec<f64> = std::iter::once(0.0)
        .chain(points.iter().copied())
        .chain(std::iter::once(1.0))
        .collect();
    for (r, &t) in techniques.iter().enumerate() {
        let (lo, hi) = (bounds[r], bounds[r + 1]);
        let values: Vec<WorkerPoint> = (0..=GRID)
            .map(|i| {
                let x = if i == GRID {
                    hi
                } else {
                    lo + (hi - lo) * i as f64 / GRID as f64
                };
                economy.at_with(x, t)
            })
            .collect();
        monotone("line", &values);
    }
    let one = economy.at_with(1.0, env.last());
    if one.v.is_finite() {
        let mut starts = vec![(one.v, env.last())];
        for s in economy.wall_switches() {
            starts.push((s.wage, s.above));
        }
        for (i, &(v0, t)) in starts.iter().enumerate() {
            let v1 = starts.get(i + 1).map_or(20.0 * v0 + 20.0, |s| s.0);
            let values: Vec<WorkerPoint> = (0..=GRID)
                .map(|k| economy.at_wage(1.0, v0 + (v1 - v0) * k as f64 / GRID as f64, t))
                .collect();
            monotone("wall", &values);
        }
    }
}

/// The count of the sequence against the solve's result: one equilibrium, none
/// (`LaborShort`; one when its change of side is a jump to a short point, excess +∞), or
/// `MultipleEquilibria` with the count.
pub fn check_count_1d(economy: &WorkerEconomy, result: &Result<Regime<Eq1d>, SolveError>) {
    let (_, changes) = sequence_1d(economy);
    match result {
        Ok(Regime::Interior(_)) => assert_eq!(changes, 1),
        Ok(Regime::NotViable { .. }) => {}
        Err(SolveError::LaborShort { excess, .. }) if changes == 1 => {
            assert_eq!(*excess, f64::INFINITY, "a jump")
        }
        Err(SolveError::LaborShort { .. }) => assert_eq!(changes, 0),
        Err(SolveError::MultipleEquilibria { sign_changes, .. }) => {
            assert_eq!(*sign_changes, changes)
        }
        other => panic!("unexpected {other:?}"),
    }
}

/// γ = η(0.2 + 0.8x).
pub fn appendix_b_schedule(eta: f64) -> PowerSchedule {
    linear(eta, 0.2, 0.8)
}
