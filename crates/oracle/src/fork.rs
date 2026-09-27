//! The fork's price block on its own (docs/unit-1b.md §4.5): a category as a list of task
//! cells priced at given w, p_m and r, and the CES expenditure share of SSRN eq 26. There is
//! no equilibrium solve.

use rustyecon_core::num;

use crate::params::{self, ParamError, Requirement};

/// One task cell of a category (SSRN A.3 with a discrete measure; check_interior.py at
/// laborformal 31b3482). "0 or scale" means 0, or finite and in
/// [[`SCALE_FLOOR`](crate::SCALE_FLOOR), [`SCALE_CEIL`](crate::SCALE_CEIL)].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    /// q_c, tasks of this cell per unit of the category's output. 0 or scale.
    pub weight: f64,
    /// γ_Lc, tasks one hour of work performs. Scale, so positive: every task has a
    /// feasible human method (main.tex:445).
    pub human: f64,
    /// γ_Mc, tasks one machine service performs. 0 or scale; 0 closes the cell to machines
    /// (SSRN §3.1's H).
    pub machine: f64,
}

/// A category's cost at given prices (docs/unit-1b.md §4.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CategoryCost {
    /// p_j = r·b_j + Σ_c q_c·min{w/γ_Lc, p_m/γ_Mc}, each task at its cheaper method, summed
    /// cell by cell (main.tex:477-490).
    pub price: f64,
    /// H_j = Σ_human q_c/γ_Lc, hours at the cells people do. A cell is human when it is
    /// closed or w/γ_Lc ≤ p_m/γ_Mc: ties go to people (check_interior.py:27).
    pub human_hours: f64,
    /// M_j = Σ_machine q_c/γ_Mc, machine services at the cells machines do.
    pub machine_services: f64,
    /// L_j* = Σ_c (q_c/γ_Lc)·min{1, γ_c/g}, with γ_c = γ_Lc/γ_Mc (∞ when closed) and
    /// g = w/p_m: main.tex eq effective-hours (:450), summed from capability ratios, not from
    /// the task costs, so that p_j = w·L_j* + r·b_j is a check.
    pub effective_hours: f64,
    /// L̄_j = Σ_c q_c/γ_Lc, the all-human method's hours.
    pub all_human_hours: f64,
    /// λ̃_j = H_j + M_j·λ̃_m, when the machine row's totals are given.
    pub lambda_tilde: Option<f64>,
    /// b̃_j = b_j + M_j·b̃_m, when the machine row's totals are given.
    pub b_tilde: Option<f64>,
}

/// The error for a parameter of cell `index`.
fn in_cell(index: usize) -> impl Fn(ParamError) -> ParamError {
    move |error| ParamError::Item {
        kind: "cell",
        index,
        error: Box::new(error),
    }
}

/// A category's cost, task by task, at the wage w, the machine-service price p_m and the
/// rent r (docs/unit-1b.md §4.5; SSRN A.3 eq 20 and main.tex:477-490).
///
/// `machine_totals`, when given, is (λ̃_m, b̃_m), the machine row's total hours and land with
/// p_m = w·λ̃_m + r·b̃_m (SSRN eq 4; the caller's to make consistent); the result then carries
/// the category's totals, and p_j = w·λ̃_j + r·b̃_j is SSRN eq 12.
///
/// Requires every cell's `weight` and `machine` 0 or scale and `human` scale; `direct_land`
/// 0 or scale; w, p_m and r scale; λ̃_m in [0, [`SCALE_CEIL`](crate::SCALE_CEIL)] and b̃_m
/// scale; and L̄_j > 0 or b_j > 0, so that the price is positive. A negative zero is read as
/// +0.0. Within these bounds no output can overflow: every term of every sum, the totals'
/// M_j·λ̃_m included, is at most SCALE_CEIL²/SCALE_FLOOR = 1e90, so any list of fewer than
/// 1e218 cells gives finite outputs.
pub fn cell_cost(
    cells: &[Cell],
    direct_land: f64,
    w: f64,
    p_m: f64,
    r: f64,
    machine_totals: Option<(f64, f64)>,
) -> Result<CategoryCost, ParamError> {
    let direct_land = params::zero_or_scale("direct_land", direct_land)?;
    let w = params::scale("w", w)?;
    let p_m = params::scale("p_m", p_m)?;
    let r = params::scale("r", r)?;
    let machine_totals = match machine_totals {
        Some((lambda_tilde, b_tilde)) => Some((
            params::nonnegative("lambda_tilde_machine", lambda_tilde)?,
            params::scale("b_tilde_machine", b_tilde)?,
        )),
        None => None,
    };
    let g = w / p_m;
    let (mut task_cost, mut human_hours, mut machine_services) = (0.0, 0.0, 0.0);
    let (mut effective_hours, mut all_human_hours) = (0.0, 0.0);
    for (index, cell) in cells.iter().enumerate() {
        let item = in_cell(index);
        let weight = params::zero_or_scale("weight", cell.weight).map_err(&item)?;
        let gamma_l = params::scale("human", cell.human).map_err(&item)?;
        let gamma_m = params::zero_or_scale("machine", cell.machine).map_err(&item)?;
        let hours = weight / gamma_l;
        let by_hand = w / gamma_l;
        if gamma_m == 0.0 || by_hand <= p_m / gamma_m {
            task_cost += weight * by_hand;
            human_hours += hours;
        } else {
            task_cost += weight * (p_m / gamma_m);
            machine_services += weight / gamma_m;
        }
        all_human_hours += hours;
        let parity = if gamma_m == 0.0 {
            1.0
        } else {
            ((gamma_l / gamma_m) / g).min(1.0)
        };
        effective_hours += hours * parity;
    }
    if !(all_human_hours > 0.0 || direct_land > 0.0) {
        return Err(ParamError::Invalid {
            name: "category",
            reason: "a category needs tasks or direct land, so that its price is positive",
        });
    }
    let price = r * direct_land + task_cost;
    let totals = machine_totals.map(|(lambda_m, b_m)| {
        (
            human_hours + machine_services * lambda_m,
            direct_land + machine_services * b_m,
        )
    });
    Ok(CategoryCost {
        price,
        human_hours,
        machine_services,
        effective_hours,
        all_human_hours,
        lambda_tilde: totals.map(|t| t.0),
        b_tilde: totals.map(|t| t.1),
    })
}

/// The expenditure share of a directly rented service under CES preferences over it and a
/// consumption good (SSRN eq 26, p.31): with weight α on the service, elasticity σ and
/// relative price q = r/p,
///
/// α(q) = α^σ·q^(1−σ) / ((1 − α)^σ + α^σ·q^(1−σ)) = 1/(1 + ((1 − α)/α)^σ·q^(σ−1)).
///
/// The second form is computed. Its t = ((1 − α)/α)^σ·q^(σ−1) is a product of two powers;
/// where either power is not a normal double (it overflowed, underflowed or is subnormal),
/// t is exp(σ·ln((1 − α)/α) + (σ − 1)·ln q) instead, whose exponent is always finite. An
/// infinite t gives the share 0 and a zero t gives 1. The share is conditional on the price
/// path and feeds nothing back (SSRN p.31): it is not a closure (docs/unit-1b.md §2.2).
///
/// Requires 0 < α < 1, σ in [0, [`SCALE_CEIL`](crate::SCALE_CEIL)] and q finite and
/// positive; q is not bounded by the scale bounds, since along an automation path it grows
/// without bound (SSRN p.31).
pub fn ces_share(alpha: f64, sigma: f64, q: f64) -> Result<f64, ParamError> {
    let alpha = params::finite("alpha", alpha)?;
    if !(alpha > 0.0 && alpha < 1.0) {
        return Err(ParamError::OutOfRange {
            name: "alpha",
            value: alpha,
            requirement: Requirement::OpenUnit,
        });
    }
    let sigma = params::nonnegative("sigma", sigma)?;
    let q = params::positive("q", q)?;
    let ratio = (1.0 - alpha) / alpha;
    let (weight, price) = (num::pow(ratio, sigma), num::pow(q, sigma - 1.0));
    let t = if weight.is_normal() && price.is_normal() {
        weight * price
    } else {
        num::exp(sigma * num::ln(ratio) + (sigma - 1.0) * num::ln(q))
    };
    Ok(1.0 / (1.0 + t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_form_takes_over_where_a_power_leaves_the_normal_range() {
        // alpha = 0.3, sigma = 400: ((0.7/0.3))^400 is about 1e147 and q^399 at q = 1e-2 is
        // 1e-798, which underflows to 0; their product is 1e-651, so the share is 1. At
        // q = 1e3, q^399 overflows; the product is huge and the share is 0. At q = 0.1,
        // 1e147 * 1e-399 = 1e-252 is a normal product of normal powers.
        assert_eq!(ces_share(0.3, 400.0, 1e-2).unwrap(), 1.0);
        assert_eq!(ces_share(0.3, 400.0, 1e3).unwrap(), 0.0);
        // Here both powers leave the range in opposite directions, and the plain product
        // would be 0 * inf = NaN: ratio 0.7/0.3 raised to 1e3 overflows, q = 1e-3 raised to
        // 999 underflows. ln t = 1e3 ln(7/3) - 999 ln(1e3) = 847.3 - 6900.9 < 0: share 1.
        let share = ces_share(0.3, 1e3, 1e-3).unwrap();
        assert_eq!(share, 1.0);
        // The same with ln t > 0: q = 0.5, 999 ln 2 = 692.5 < 847.3, so t is e^154.8.
        let share = ces_share(0.3, 1e3, 0.5).unwrap();
        let want = 1.0 / (1.0 + num::exp(1e3 * num::ln(7.0 / 3.0) - 999.0 * num::ln(2.0)));
        assert!(
            (share - want).abs() <= 1e-12 * want,
            "{share:e} vs {want:e}"
        );
        assert!(share > 0.0 && share < 1e-60);
    }
}
