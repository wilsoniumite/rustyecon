//! C6: the price block on its own (docs/unit-1b.md §4.5): categories as lists of task cells
//! at given prices, as laborformal's pinning/checks/check_interior.py builds them
//! (cost_and_accounting_checks :10-59 and replacement_and_interior_formula_checks :128-171
//! at 31b3482). The fork identity is a real check here: the price is summed from task costs
//! and L* from capability ratios.

use oracle::{cell_cost, closure, CategoryCost, Cell, ParamError, SCALE_CEIL};
use rustyecon_core::num;

use crate::g5_random_economies::SplitMix64;
use crate::goldens_1b::*;
use crate::support::*;
use crate::support_1b::*;

/// e^U(lo, hi): log-uniform. check_interior draws lognormal productivities; core has no
/// normal generator, so these draw log-uniform over lognormal's ±2σ.
fn log_uniform(rng: &mut SplitMix64, lo: f64, hi: f64) -> f64 {
    num::exp(rng.uniform(lo, hi))
}

/// The fork identity, SSRN eq 12 when the totals are given, the bounds and the pair at
/// prices (w, r) for a category with direct land b (SSRN eq 12, 13, 19; main.tex:459-474).
/// Returns the largest relative error of the two price identities.
fn check_cost(what: &str, cost: &CategoryCost, b: f64, w: f64, r: f64) -> f64 {
    let v = w / r;
    let mut worst: f64 = 0.0;
    let mut identity = |name: &str, got: f64, want: f64| {
        let err = (got - want).abs() / want;
        assert!(
            err <= FULL,
            "{what}: {name}: {got:e} vs {want:e}, rel {err:e}"
        );
        worst = worst.max(err);
    };
    identity(
        "p = w L* + r b",
        w * cost.effective_hours + r * b,
        cost.price,
    );
    if let (Some(lt), Some(bt)) = (cost.lambda_tilde, cost.b_tilde) {
        identity("p = w lt + r bt", w * lt + r * bt, cost.price);
        at_most(&format!("{what}: b <= bt"), b, bt);
        at_most(&format!("{what}: r bt <= p"), r * bt, cost.price);
        at_most(&format!("{what}: w/p <= (w/r)/bt"), w / cost.price, v / bt);
    }
    at_most(&format!("{what}: r b <= p"), r * b, cost.price);
    at_most(
        &format!("{what}: p <= r b + w Lbar"),
        cost.price,
        r * b + w * cost.all_human_hours,
    );
    at_most(
        &format!("{what}: L* <= Lbar"),
        cost.effective_hours,
        cost.all_human_hours,
    );
    close(
        &format!("{what}: w/p = 1/(L* + b r/w)"),
        1.0 / (cost.effective_hours + b / v),
        w / cost.price,
    );
    at_most(
        &format!("{what}: 1/(Lbar + b r/w) <= w/p"),
        1.0 / (cost.all_human_hours + b / v),
        w / cost.price,
    );
    if b > 0.0 {
        at_most(&format!("{what}: w/p <= (w/r)/b"), w / cost.price, v / b);
    } else {
        at_most(
            &format!("{what}: w/p >= 1/Lbar"),
            1.0 / cost.all_human_hours,
            w / cost.price,
        );
    }
    worst
}

#[test]
fn cost_and_accounting() {
    // check_interior.py:10-49: 80 economies of 4 categories x 193 cells, 9% closed to
    // machines, category 0 without direct land; then the income identity for arbitrary
    // outputs and a household rental, I = r T_h + sum p_j y_j = w hours + r land, and the
    // land share r land / I = 1/(1 + v hours / land) (SSRN App. C).
    const CELLS: usize = 193;
    let mut rng = SplitMix64(20_260_905);
    let (mut worst_price, mut worst_income) = (0.0f64, 0.0f64);
    let mut closed = 0;
    for economy in 0..80 {
        let a = rng.uniform(0.05, 0.65);
        let lam = rng.uniform(0.0, 0.3);
        let b = rng.uniform(0.1, 1.0);
        let v = num::exp(rng.uniform(-5.0, 1.0) * num::ln(10.0));
        let r = rng.uniform(0.1, 3.0);
        let (w, p_m) = (r * v, r * (lam * v + b) / (1.0 - a));
        let totals = (lam / (1.0 - a), b / (1.0 - a));
        let (mut hours_final, mut machine_services, mut direct_land, mut bill) =
            (0.0, 0.0, 0.0, 0.0);
        for j in 0..4 {
            let cells: Vec<Cell> = (0..CELLS)
                .map(|_| {
                    let human = log_uniform(&mut rng, -1.4, 1.4);
                    let machine = log_uniform(&mut rng, -2.0, 2.0);
                    let machine = if rng.uniform(0.0, 1.0) < 0.09 {
                        closed += 1;
                        0.0
                    } else {
                        machine
                    };
                    Cell {
                        weight: 1.0 / CELLS as f64,
                        human,
                        machine,
                    }
                })
                .collect();
            let land = if j == 0 { 0.0 } else { rng.uniform(0.1, 2.0) };
            let cost = cell_cost(&cells, land, w, p_m, r, Some(totals)).unwrap();
            let what = format!("economy {economy}, category {j}");
            worst_price = worst_price.max(check_cost(&what, &cost, land, w, r));
            let y = rng.uniform(0.1, 5.0);
            hours_final += y * cost.human_hours;
            machine_services += y * cost.machine_services;
            direct_land += land * y;
            bill += cost.price * y;
        }
        let x = machine_services / (1.0 - a);
        let hours = hours_final + lam * x;
        let housing = rng.uniform(0.2, 3.0);
        let stock = housing + direct_land + b * x;
        let income = bill + r * housing;
        let error = (income - (w * hours + r * stock)).abs() / income;
        assert!(error <= FULL, "economy {economy}: income {error:e}");
        worst_income = worst_income.max(error);
        near(
            &format!("economy {economy}: land share"),
            r * stock / income,
            1.0 / (1.0 + v * hours / stock),
            FULL,
        );
    }
    // Recorded: check_interior asserts 5e-13 and reports 4.15e-16.
    assert!(
        worst_income < 5e-13 && worst_price < 5e-13,
        "{worst_income:e} {worst_price:e}"
    );
    assert!(closed > 0);
}

#[test]
fn exact_interior_prices() {
    // check_interior.py:145-161: 100 categories of 257 cells at a margin g* = w/p_m, 10%
    // closed; the fork identity and the pair to 1e-12, and L* <= Lbar.
    const CELLS: usize = 257;
    let mut rng = SplitMix64(20_260_921);
    let mut machine_cells = 0;
    for instance in 0..100 {
        let g = rng.uniform(0.2, 1.5);
        let cells: Vec<Cell> = (0..CELLS)
            .map(|_| {
                let human = log_uniform(&mut rng, -1.0, 1.0);
                let relative = log_uniform(&mut rng, -1.6, 1.6);
                let machine = if rng.uniform(0.0, 1.0) < 0.1 {
                    0.0
                } else {
                    human / relative
                };
                Cell {
                    weight: 1.0 / CELLS as f64,
                    human,
                    machine,
                }
            })
            .collect();
        let w = rng.uniform(0.3, 3.0);
        let p_m = w / g;
        let r = rng.uniform(0.2, 2.0);
        let land = rng.uniform(0.0, 1.0);
        let cost = cell_cost(&cells, land, w, p_m, r, None).unwrap();
        check_cost(&format!("instance {instance}"), &cost, land, w, r);
        assert_eq!((cost.lambda_tilde, cost.b_tilde), (None, None));
        if cost.machine_services > 0.0 {
            machine_cells += 1;
        }
    }
    assert!(machine_cells > 90);
}

#[test]
fn parity_with_uneven_human_productivity() {
    // check_interior.py:50-58 and SSRN A.3's parity remark: relative capability 0.35 at
    // every cell and w = 0.35 p_m, with human productivity 0.2 + i 0.008125 (i = 0..320):
    // every cell is at parity and the cost is land + v Lbar at every land level.
    let c = closure(0.2, 0.1, 0.35, 0.4, 1.0, 1.0).unwrap();
    close("v", c.w, C6_PARITY_V);
    close("p_m", c.p_m, C6_PARITY_P_M);
    close(
        "p_m = (lambda v + b)/(1 - a)",
        (0.1 * c.w + 0.4) / 0.8,
        c.p_m,
    );
    let cells: Vec<Cell> = (0..321)
        .map(|i| {
            let human = 0.2 + f64::from(i) * 0.008125;
            Cell {
                weight: 1.0 / 321.0,
                human,
                machine: human / 0.35,
            }
        })
        .collect();
    for land in [0.0, 0.1, 0.5, 2.0] {
        let cost = cell_cost(&cells, land, c.w, c.p_m, 1.0, None).unwrap();
        close("Lbar", cost.all_human_hours, C6_PARITY_L_BAR);
        close("L* = Lbar", cost.effective_hours, cost.all_human_hours);
        close(
            &format!("cost at land {land}"),
            cost.price,
            land + C6_PARITY_TASK_COST,
        );
        close("v Lbar", c.w * cost.all_human_hours, C6_PARITY_TASK_COST);
        check_cost(&format!("parity, land {land}"), &cost, land, c.w, 1.0);
    }
}

#[test]
fn flat_zero_land_category_is_invariant() {
    // check_fan F4 and check_dynamics F4 (at 31b3482; main.tex:506-509): with relative
    // capability gbar at every cell and w = gbar p_m, a category without direct land has
    // w/p = 1/Lbar whatever the machine's recipe and user cost. p_m and w come from 1a's
    // closure at u = 1, 1 + rho, rho + delta and u(0.05, 0.1, 3).
    let humans = [0.5, 1.0, 2.0, 4.0];
    for u in [1.0, 1.05, 0.15, 0.165375] {
        for gbar in [0.25, 1.0, 3.0] {
            for lam in [0.0, 0.05] {
                let c = closure(0.3, lam, gbar, 0.4, 1.0, u).unwrap();
                let cells: Vec<Cell> = humans
                    .iter()
                    .map(|&human| Cell {
                        weight: 0.25,
                        human,
                        machine: human / gbar,
                    })
                    .collect();
                let what = format!("u {u}, gbar {gbar}, lambda {lam}");
                let cost = cell_cost(&cells, 0.0, c.w, c.p_m, 1.0, None).unwrap();
                close(&what, c.w / cost.price, 1.0 / cost.all_human_hours);
                // The upper bound is attained with direct land too: p = w Lbar + r b.
                let with_land = cell_cost(&cells, 0.7, c.w, c.p_m, 1.0, None).unwrap();
                close(
                    &what,
                    with_land.price,
                    c.w * with_land.all_human_hours + 0.7,
                );
                if gbar != 3.0 {
                    // gbar a power of two: every quotient is exact, every cell ties, and
                    // L* = Lbar bit for bit.
                    assert_eq!(cost.effective_hours, cost.all_human_hours, "{what}");
                    assert_eq!(cost.all_human_hours, 0.25 * (2.0 + 1.0 + 0.5 + 0.25));
                    assert_eq!(cost.machine_services, 0.0, "{what}: ties go to people");
                }
            }
        }
    }
}

#[test]
fn closed_cells_go_to_people() {
    // A cell closed to machines is done by hand at any prices (SSRN §3.1's H), and a tie
    // goes to people (check_interior.py:27).
    let closed = [Cell {
        weight: 0.5,
        human: 2.0,
        machine: 0.0,
    }];
    for (w, p_m) in [(1.0, 1.0), (1e20, 1e-20), (1e-20, 1e20)] {
        let cost = cell_cost(&closed, 0.0, w, p_m, 1.0, Some((0.1, 0.2))).unwrap();
        assert_eq!(cost.human_hours, 0.25);
        assert_eq!(cost.machine_services, 0.0);
        assert_eq!(cost.effective_hours, cost.all_human_hours);
        assert_eq!(cost.price, 0.5 * (w / 2.0));
        assert_eq!(cost.b_tilde, Some(0.0));
    }
    // w/gamma_L = 2/2 = 1 = p_m/gamma_M = 1/1: a tie, done by hand; one ulp cheaper by
    // machine, done by machine.
    let tie = [Cell {
        weight: 1.0,
        human: 2.0,
        machine: 1.0,
    }];
    let cost = cell_cost(&tie, 0.0, 2.0, 1.0, 1.0, None).unwrap();
    assert_eq!((cost.human_hours, cost.machine_services), (0.5, 0.0));
    assert_eq!(cost.effective_hours, 0.5);
    let cheaper = cell_cost(&tie, 0.0, 2.0, 1.0f64.next_down(), 1.0, None).unwrap();
    assert_eq!((cheaper.human_hours, cheaper.machine_services), (0.0, 1.0));
    assert!(cheaper.effective_hours < 0.5);
}

#[test]
fn cells_approach_the_line() {
    // The fork economy's categories, cut into midpoint cells of width about 2^-12 on each
    // segment, priced at the equilibrium's v and p_m, give L* and p_j within 1e-6 of the
    // line's closed forms (the midpoint rule's error is O(width^2)). A sanity check of the
    // two price blocks against each other, not a gate identity.
    let params = fork_economy();
    let eq = interior_1b(params.clone());
    let gamma = |x: f64| oracle::Schedule::gamma(&params.schedule, x);
    for (j, cat) in params.categories.iter().enumerate() {
        let mut cells = Vec::new();
        for s in 0..params.edges.len() - 1 {
            let (lo, hi) = (params.edges[s], params.edges[s + 1]);
            if cat.density[s] == 0.0 {
                continue;
            }
            let n = ((hi - lo) * 4096.0).ceil();
            let width = (hi - lo) / n;
            for i in 0..n as usize {
                let t = lo + (i as f64 + 0.5) * width;
                cells.push(Cell {
                    weight: cat.density[s] * width,
                    human: 1.0,
                    machine: 1.0 / gamma(t),
                });
            }
        }
        let cost = cell_cost(&cells, cat.direct_land, eq.v, eq.p_m, 1.0, None).unwrap();
        let c = &eq.categories[j];
        close_to(&format!("p of category {j}"), cost.price, c.price, 1e-6);
        if c.l_star > 0.0 {
            close_to(
                &format!("L* of category {j}"),
                cost.effective_hours,
                c.l_star,
                1e-6,
            );
        }
        close_to(
            &format!("Lbar of category {j}"),
            cost.all_human_hours,
            c.l_bar,
            1e-12,
        );
    }
}

#[test]
fn cell_validation() {
    let good = Cell {
        weight: 0.5,
        human: 1.0,
        machine: 2.0,
    };
    let item = |e: ParamError| match e {
        ParamError::Item {
            kind: "cell",
            index,
            error,
        } => (index, error.name()),
        other => panic!("{other:?}"),
    };
    for (cell, name) in [
        (Cell { human: 0.0, ..good }, "human"),
        (
            Cell {
                human: -1.0,
                ..good
            },
            "human",
        ),
        (
            Cell {
                weight: -0.1,
                ..good
            },
            "weight",
        ),
        (
            Cell {
                machine: f64::NAN,
                ..good
            },
            "machine",
        ),
        (
            Cell {
                machine: 1e-31,
                ..good
            },
            "machine",
        ),
    ] {
        let err = cell_cost(&[good, cell], 0.0, 1.0, 1.0, 1.0, None).unwrap_err();
        assert_eq!(item(err), (1, name));
    }
    for (args, name) in [
        ((-0.1, 1.0, 1.0, 1.0), "direct_land"),
        ((0.0, 0.0, 1.0, 1.0), "w"),
        ((0.0, SCALE_CEIL * 2.0, 1.0, 1.0), "w"),
        ((0.0, 1.0, f64::NAN, 1.0), "p_m"),
        ((0.0, 1.0, 1.0, 0.0), "r"),
    ] {
        let (land, w, p_m, r) = args;
        let err = cell_cost(&[good], land, w, p_m, r, None).unwrap_err();
        assert_eq!(err.name(), name, "{args:?}");
    }
    assert_eq!(
        cell_cost(&[good], 0.0, 1.0, 1.0, 1.0, Some((-0.1, 0.5)))
            .unwrap_err()
            .name(),
        "lambda_tilde_machine"
    );
    assert_eq!(
        cell_cost(&[good], 0.0, 1.0, 1.0, 1.0, Some((0.1, 0.0)))
            .unwrap_err()
            .name(),
        "b_tilde_machine"
    );
    // A category with no tasks and no land has no positive price.
    let err = cell_cost(&[], 0.0, 1.0, 1.0, 1.0, None).unwrap_err();
    assert_eq!(err.name(), "category");
    let zero = Cell {
        weight: 0.0,
        ..good
    };
    assert_eq!(
        cell_cost(&[zero], 0.0, 1.0, 1.0, 1.0, None)
            .unwrap_err()
            .name(),
        "category"
    );
    // With land and no tasks it is the pass-through row: p = r b.
    let site = cell_cost(&[], 1.0, 2.0, 3.0, 0.5, None).unwrap();
    assert_eq!((site.price, site.effective_hours), (0.5, 0.0));
    // A negative zero is read as +0.0: the result is the same, bit for bit.
    let neg = Cell {
        weight: -0.0,
        ..good
    };
    assert_eq!(
        cell_cost(&[good, neg], -0.0, 1.0, 1.0, 1.0, None).unwrap(),
        cell_cost(&[good, zero], 0.0, 1.0, 1.0, 1.0, None).unwrap()
    );
    assert!(cell_cost(&[good, neg], -0.0, 1.0, 1.0, 1.0, None)
        .unwrap()
        .price
        .is_sign_positive());
}
