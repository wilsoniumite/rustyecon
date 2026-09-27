//! h8: tests for mutants the re-checks of units 1d-1f left alive (STATE O28; docs/unit-1g.md
//! §8 and §12). Each fails with its mutant in place, and passes on the oracle as it is: the
//! results the re-checks found right stay right, and now the gate can tell.

use oracle::{
    CategoryParams, ExitLand, LandMarket, MachineParams, Margin, ParamError, Params, Regime,
    Schedule, SolveError, UniformWorkCost, WorkerEconomy, WorkerParams,
};
use rustyecon_core::num;

use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;
use crate::support_1f::*;

/// 1a's Appendix B in worker form with N and χ_max.
fn one_type(workers: f64, chi_max: f64) -> WorkerParams {
    from_1a(Params {
        workers,
        work_cost: UniformWorkCost { chi_max },
        ..appendix_b()
    })
}

#[test]
fn f_at_the_end_is_zero_after_a_positive_start() {
    // 1d, O28 (the re-check's probe): f_∞ = 0 exactly after a last piece that starts above 0.
    // Supply saturates on the wall at a real wage below ω_∞ with N = n_D there, so f is 0 from
    // then on; f_∞ = 0 after a nonzero start is on the positive side, no change of side, and
    // the economy is LaborShort with excess 0. Read as negative, it would be an equilibrium.
    let probe = economy_1d(one_type(1.0, 1.0));
    let q1 = probe.at_with(1.0, 0);
    let omega_1 = q1.v / q1.p_s;
    let end = probe.wall_end();
    let omega_end = end.omega.unwrap();
    assert!(omega_1 < omega_end);
    let chi = num::ln1p(0.5 * (omega_1 + omega_end));
    let n_d = probe.at_wage(1.0, 1e6 * q1.v, end.technique).n_d;
    let e = economy_1d(one_type(n_d, chi));
    let one = e.at_with(1.0, 0);
    assert!(one.excess_demand() > 0.0 && e.wall_end().excess == 0.0);
    match e.solve() {
        Err(SolveError::LaborShort { excess, .. }) => assert_eq!(excess, 0.0),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_last_piece_starts_at_a_wall_switch() {
    // 1d, O28: a wall switch whose value above is exactly 0 with supply saturated on the last
    // piece. The last piece starts at that switch's value, 0, not at f(1); the sequence has one
    // change of side and the economy one equilibrium.
    let base = wall_switch_economy(1.0);
    let with = |n: f64, chi: f64| {
        let mut p = base.clone();
        p.worker_types[0].workers = n;
        p.worker_types[0].work_cost.chi_max = chi;
        p
    };
    let probe = economy_1d(with(1.0, 3.0));
    let s = probe.wall_switches()[0];
    let above = probe.at_wage(1.0, s.wage, s.above);
    let e = economy_1d(with(above.n_d, 1e-6));
    let s = e.wall_switches()[0];
    assert_eq!(e.at_wage(1.0, s.wage, s.above).excess_demand(), 0.0);
    assert_eq!(e.wall_end().excess, 0.0);
    let result = e.solve();
    check_count_1d(&e, &result);
    assert!(result.is_ok(), "{result:?}");
}

/// Closure wages of every task type at a corner: v_k(g) = g·b̃_k/(θ_k − g·λ̃_k) with g = v/π, v
/// for τ and at least v for the others.
fn check_closure_wages(e: &WorkerEconomy, eq: &oracle::Eq1d) -> usize {
    let types = e.machines().block().types();
    let g = eq.g;
    let mut others = 0;
    for (k, t) in eq.types.iter().enumerate() {
        let theta = types[k].task_efficiency;
        let room = theta - g * t.lambda_tilde;
        if theta > 0.0 && room > 0.0 {
            let want = g * t.b_tilde / room;
            let got = t.closure_wage.unwrap();
            assert!(
                (got - want).abs() <= 1e-10 * want.abs(),
                "type {k}: closure wage {got} against {want}"
            );
            assert!(got >= eq.v * (1.0 - 1e-12), "type {k} below v");
            if k != eq.technique {
                others += 1;
            }
        }
    }
    others
}

#[test]
fn closure_wages_of_every_type_at_a_corner() {
    // 1d, O28: at a corner every task type's closure wage is taken at the corner's margin
    // g = v/π, not at γ(x*), the technique's and the others'.
    let (e, eq) = checked_1d(wall_switch_economy(1.0));
    let mut others = check_closure_wages(&e, &eq);
    for n in [0.2, 0.5, 2.0, 5.0] {
        let e = economy_1d(wall_switch_economy(n));
        if let Ok(Regime::Interior(eq)) = e.solve() {
            if eq.margin != Margin::Contestable {
                others += check_closure_wages(&e, &eq);
            }
        }
    }
    for n_e in [2.0, 4.0, 8.0, 16.0] {
        let e = economy_1d(full(n_e, 3.0, 0.5));
        if let Ok(Regime::Interior(eq)) = e.solve() {
            if eq.margin != Margin::Contestable {
                others += check_closure_wages(&e, &eq);
            }
        }
    }
    assert!(others > 0, "no other viable type at a corner was checked");
}

#[test]
fn lemma_b1_uses_the_basket_price_with_reserved_costs() {
    // 1d, O28: Lemma B.1's funding condition is T > ν·P_s(1) with P_s(1) the basket's price with
    // the reserved types' costs, not the base price P⁰_s(1). Supports scaled so that T lies
    // between ν·P⁰_s(1) and ν·P_s(1), pooled types unaffected: the lemma does not hold.
    let base = entrant_trained(8.0, 3.0, 1.0, 1.0);
    let e = economy_1d(base.clone());
    let one = e.at_with(1.0, e.machines().envelope().last());
    let target = base.land / (one.p_s * one.base_p_s).sqrt();
    let scale = target / e.support();
    let mut params = base.clone();
    for t in &mut params.worker_types {
        t.support *= scale;
    }
    let e = economy_1d(params.clone());
    let one = e.at_with(1.0, e.machines().envelope().last());
    assert!(one.excess_demand() < 0.0);
    assert!(e.support() * one.base_p_s < params.land && params.land < e.support() * one.p_s);
    let (_, eq) = checked_1d(params);
    assert!(!eq.lemma_b1);
}

#[test]
fn the_wall_ends_frame_prices_the_exit_good_at_its_land() {
    // 1e, O28: an exit good made of land alone, free at r = 0, is decided at the wall's end at
    // its limit price there, b̃_g (decision 160). Every free exit good of the gate had b̃_g = 1
    // (space); here space's land is 2 and 0.5. At 2 the economy is on the wall with plots
    // rented; at 0.5 it is on idle land with every plot free (the floor). Priced at 1, or at
    // 1/b̃_g, the frame takes the other branch: three equilibria at 2, a broken junction at 0.5.
    for (land, gross, plot, market, branch) in [
        (1.0, 3.2, 1.8, LandMarket::Scarce, ExitLand::Enclosed),
        (2.0, 3.2, 1.8, LandMarket::Scarce, ExitLand::Enclosed),
        (2.0, 0.5, 1.0, LandMarket::Idle, ExitLand::Idle),
        (0.5, 3.2, 1.8, LandMarket::Idle, ExitLand::Idle),
    ] {
        let mut p = land_good(5.0, 1.8);
        p.categories[1].direct_land = land;
        p.exits = vec![priced(gross, 0.0, plot)];
        let (e, eq) = checked_1e(p);
        assert_eq!(eq.base.margin, Margin::Wall, "{land}, {gross}");
        assert_eq!(eq.land_market, market, "{land}, {gross}");
        assert_eq!(eq.exit_land, branch, "{land}, {gross}");
        close_to(&format!("q at {land}"), eq.q, 1.0 / land, 1e-15);
        // f is continuous where the wall meets idle land.
        let tau = e.workers().wall_end().technique;
        let far = e.at_wage(1.0, 1e10, tau).excess_demand();
        let start = e.enclosed_land() - e.at_idle(e.enclosed_land(), tau).rented_plots;
        let idle = e.at_idle(start, tau).excess_demand();
        close_to(&format!("the junction at {land}"), far, idle, 1e-8);
    }
}

#[test]
fn ces_required_hours_come_from_gross_outputs() {
    // 1f, O28: under a CES basket the human-required hours are Σ gross output × L^H, the
    // composite's content through the intermediate inputs, not its final content. No CES
    // instance had intermediate inputs; here space uses 0.2 of the good a unit, and the good
    // needs 0.1 required hours a unit.
    for sigma in [0.5, 2.0] {
        let mut p = from_1a_1e(appendix_b());
        p.intermediate = vec![vec![0.0, 0.0], vec![0.2, 0.0]];
        p.human_required = vec![0.1, 0.0];
        let (_, eq) = checked_1f(ces(p, sigma));
        let good = &eq.base.base.categories[0];
        assert!(good.gross_output > good.output * (1.0 + 1e-3), "{sigma}");
        assert!(eq.base.base.required_hours > 0.0);
    }
}

#[test]
fn an_economy_needs_a_worker_type() {
    // 1d, O28 (a survivor of the first pass): no worker types is refused, by name.
    let mut p = one_type(4.0, 1.0);
    p.worker_types.clear();
    match WorkerEconomy::new(p) {
        Err(ParamError::Invalid { name, .. }) => assert_eq!(name, "worker types"),
        other => panic!("{other:?}"),
    }
}

/// g8's jump schedule (`g8_regimes::Jump`), cloneable as unit 1d's economies need: γ with a jump
/// at x0 and J its exact integral.
#[derive(Clone, Debug, PartialEq)]
struct Jump {
    x0: f64,
    below: f64,
    above: f64,
    slope: f64,
}

impl Schedule for Jump {
    fn gamma(&self, x: f64) -> f64 {
        let level = if x < self.x0 { self.below } else { self.above };
        level + self.slope * x
    }

    fn integral(&self, x: f64) -> f64 {
        let ramp = self.slope * x * x / 2.0;
        if x < self.x0 {
            self.below * x + ramp
        } else {
            self.below * self.x0 + self.above * (x - self.x0) + ramp
        }
    }
}

#[test]
fn a_jump_trips_the_labour_net_in_1d() {
    // 1d, O28 (a survivor of the first pass): 1a's schedule with a jump at x0 = 7/8
    // (g8::a_jump_in_excess_demand_trips_the_labour_residual_net) in worker form. The path
    // closes on a sign change with no root, and the pool misses by about 0.4 of N_a: 1d must
    // refuse it, as 1a, 1b and 1c do.
    let params = Params {
        schedule: Jump {
            x0: 0.875,
            below: 0.2,
            above: 1.0,
            slope: 0.1,
        },
        workers: 4.0,
        land: 10.0,
        space: 1.0,
        a: 0.3,
        lam: 0.05,
        b: 0.4,
        work_cost: UniformWorkCost { chi_max: 1.0 },
        rho: 0.0,
        delta: 1.0,
        build_lag: 1,
    };
    let workers = WorkerParams::from_machines(MachineParams::from_categories(
        CategoryParams::from_one_category(params),
    ));
    let e = WorkerEconomy::new(workers).expect("the sampled check passes a jump");
    match e.solve() {
        Err(SolveError::LaborNotCleared { x_star, relative }) => {
            assert!(
                x_star == 0.875 || x_star == 0.875_f64.next_down(),
                "{x_star:?}"
            );
            assert!(relative > 0.1, "{relative:e}");
        }
        other => panic!("expected LaborNotCleared, got {other:?}"),
    }
}
