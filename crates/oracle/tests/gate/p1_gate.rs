//! Phase 1's gate, item by item (docs/PLAN.md Phase 1; docs/unit-1f.md §13). One test per gate
//! item, named for it, each checking the item itself on its instances; the table below names
//! every module and test that covers the item in full. When these pass with the rest of the
//! gate on WSL and Windows, Phase 1's gate is complete.
//!
//! | gate item | this module | covered in full by |
//! |---|---|---|
//! | the SSRN Appendix B instance: the published values (x* 0.86315, v 0.54344, Y 7.88061, N_a 1.34338) to 5e-6, and ADDENDUM §5's full-precision values to 1e-12 relative | `appendix_b_instance` | `g1_appendix_b::*` (1a); `c1_nesting::appendix_b_is_bit_identical` (1b), `m1_nesting::appendix_b_is_bit_identical` (1c), `d1_nesting::every_1c_golden_instance_is_bit_identical` (1d), `e1_nesting::every_1d_golden_instance_is_bit_identical` (1e), `f1_nesting::every_1e_golden_instance_is_bit_identical` (1f) |
//! | the replacement closure's worked instance (c = 1, w = 3; at λ = 0, c = 0.4 and w = 1.2) | `replacement_closure_worked_instance` | `g6_closure::*` (1a, price block); `m2_machine_block::closure_per_type` (1c); `f2_three_taxes::the_worked_instance_inside_the_closure`, `f2_three_taxes::the_corner` (1f, inside the full closure) |
//! | the fork identity and the category bounds on random instances | `fork_identity_and_category_bounds` | `c5_random_categories::*`, `c6_price_block::*` (1b); `m6_random_leontief::fork_and_bounds` (1c); `d7_random_workers::identities`, `e8_random_parcels::identities`, `f9_random_households::identities` (1d-1f) |
//! | the income identity to 1e-12 | `income_identity` | `g5_random_economies::*` (1a); `c5_random_categories::identities` (1b); `m6_random_leontief::identities`, `m3_two_recipe::income_with_interest` (1c); `d7_random_workers::identities` (1d); `e8_random_parcels::identities` (1e); `f9_random_households::identities` and every f-golden (1f, with government: (I1)-(I4)) |
//! | three-taxes' resolution ledger, (φ_w, φ_r) = (0.6, 0.4) on its worked instance | `three_taxes_ledger` | `g7_three_taxes::*` (1a); `m2_machine_block::ledger` (1c); `f2_three_taxes::*` (1f, with T6 and T5) |
//! | constructed wall and interior cases recognised correctly | `wall_and_interior_cases` | `g8_regimes::*` (1a); `c7_gaps_and_regimes::regimes` (1b); `m7_regimes_and_validation::regimes` (1c); `d2`-`d8` (1d); `e5_idle_land::*`, `e9_regimes_and_validation::exact_zeros` (1e); `f5_payroll::borne_by_employers_at_the_wall`, `f5_payroll::onto_idle_land`, `f8_ces::no_idle_stretch_with_a_free_category`, `f8_ces::the_good_free_at_the_all_human_corner`, `f8_ces::the_wall_far_out`, `f7_conditionality::surplus_labour` (1f) |
//! | each exit form on its own gate: the dependence form in 1a, s(q) in 1e; the 1d and 1e gates constructed | `each_exit_form_on_its_gate` | the dependence form: `g1_appendix_b::*`, `g2_figure_3::*`, `g3_automation_path::*` (1a) and `d2`-`d9` (1d); s(q): `e2_exit_value::*`, `e3_race::*`, `e4_commons::*`, `e5_idle_land::*`, `e10_goldens_file::*` (1e); both nest through `e1_nesting::exit_option_off` and `f1_nesting::*`, and keep their gates under a government through `f6_consumption_tax::the_wage_leg_is_a_wage_tax` (priced) and `f4_transfers`, `f5_payroll` (dependence) |

use oracle::{
    closure, CategoryEconomy, CategoryParams, Economy, Government, HouseholdParams, LandMarket,
    MachineParams, Margin, ParcelParams, Regime, SolveError, WorkerParams,
};

use crate::goldens::*;
use crate::goldens_1e::*;
use crate::goldens_1f::*;
use crate::support::*;
use crate::support_1d::*;
use crate::support_1e::*;
use crate::support_1f::*;

#[test]
fn appendix_b_instance() {
    // The published values to 5e-6 and the full-precision ones to 1e-12, in 1a; then the same
    // economy through every unit's form, 1b to 1f, bit for bit.
    let Ok(Regime::Interior(g1)) = Economy::new(appendix_b()).unwrap().solve() else {
        panic!("G1 solves in 1a")
    };
    for (name, got, published, full) in [
        ("x*", g1.x_star, PUB_G1_X_STAR, G1_X_STAR),
        ("v", g1.v, PUB_G1_V, G1_V),
        ("Y", g1.y, PUB_G1_Y, G1_Y),
        ("N_a", g1.n_a, PUB_G1_N_A, G1_N_A),
    ] {
        near(name, got, published, PUBLISHED);
        close(name, got, full);
    }
    let one = CategoryParams::from_one_category(appendix_b());
    let Ok(Regime::Interior(b)) = CategoryEconomy::new(one).unwrap().solve() else {
        panic!("G1 solves in 1b")
    };
    let parcels = from_1a_1e(appendix_b());
    let f = solved_1f(HouseholdParams::from_parcels(parcels.clone()));
    let e = solved_1e(parcels);
    let d = &f.base.base;
    for (name, a, x, y, z) in [
        ("x*", g1.x_star, b.x_star, d.x_star, e.base.x_star),
        ("v", g1.v, b.v, d.v, e.base.v),
        ("Y", g1.y, b.y, d.y, e.base.y),
        ("N_a", g1.n_a, b.n_a, d.n_a, e.base.n_a),
        ("P_s", g1.p_s, b.p_s, d.p_s, e.base.p_s),
    ] {
        assert_eq!(a.to_bits(), x.to_bits(), "1b {name}");
        assert_eq!(a.to_bits(), y.to_bits(), "1f {name}");
        assert_eq!(a.to_bits(), z.to_bits(), "1e {name}");
    }
}

#[test]
fn replacement_closure_worked_instance() {
    // c = 1, w = 3; at λ = 0, c = 0.4 and w = 1.2: as a price block (1a) and inside the full
    // closure, where TX's and TX3's margins sit at γ(x*) = 3.
    let c = closure(0.5, 0.1, 3.0, 0.2, 1.0, 1.0).unwrap();
    close("c", c.p_m, 1.0);
    close("w", c.w, 3.0);
    let c0 = closure(0.5, 0.0, 3.0, 0.2, 1.0, 1.0).unwrap();
    close("c at λ = 0", c0.p_m, 0.4);
    close("w at λ = 0", c0.w, 1.2);
    let tx = solved_1f(tx(Government::none()));
    close("TX's c", tx.base.base.types[0].price, TX_P_M);
    close("TX's w", tx.base.base.v, TX_V);
    close("TX's c = closure", tx.base.base.types[0].price, c.p_m);
    let tx3 = solved_1f(household(tx_parcels(0.0, 12.0), Government::none()));
    close("TX3's c", tx3.base.base.types[0].price, TX3_P_M);
    close("TX3's w", tx3.base.base.v, TX3_V);
    close("TX3's c = closure", tx3.base.base.types[0].price, c0.p_m);
}

#[test]
fn fork_identity_and_category_bounds() {
    // The fork identity in both forms and the category bounds on random instances: 1b's C5
    // draws solved in household form, every identity of §4.8 (1d's fork, totals and bounds
    // among them) at each equilibrium, and under a CES basket with a government.
    use crate::c5_random_categories::{draw, Set};
    let mut rng = crate::g5_random_economies::SplitMix64(crate::c5_random_categories::SEED_1B);
    let (mut n, mut draws) = (0, 0);
    while n < 20 {
        draws += 1;
        assert!(draws < 400);
        let p = MachineParams::from_categories(draw(&mut rng, Set::Flow));
        let parcels = e0(WorkerParams::from_machines(p));
        if oracle::ParcelEconomy::new(parcels.clone()).is_err() {
            continue;
        }
        let e = economy_1f(HouseholdParams::from_parcels(parcels.clone()));
        let Ok(Regime::Interior(eq)) = e.solve() else {
            continue;
        };
        check_identities_1f(&e, &eq);
        let r = &eq.base.base.residuals;
        assert!(r.fork <= FULL && r.totals <= FULL);
        let r = eq.base.rent;
        for c in &eq.base.base.categories {
            assert!(r * c.b_tilde <= c.price * (1.0 + FULL));
            assert!(c.price <= (eq.base.base.v * c.l_bar + r * c.chain_land) * (1.0 + FULL));
        }
        n += 1;
    }
    let (_, ces) = checked_1f(ces(g1_parcels(4.0, 0.05, 1.0), 0.5));
    assert!(ces.base.base.residuals.fork <= FULL);
}

#[test]
fn income_identity() {
    // I = Y·P = W + R + interest to 1e-12 in every unit's instance, and with a government (I1)-(I4)
    // (check_identities_1f): 1a's G1, 1c's M4 with interest, 1d's E1, 1e's K2, and 1f's GC, GT,
    // ER and C1.
    let Ok(Regime::Interior(g1)) = Economy::new(appendix_b()).unwrap().solve() else {
        panic!()
    };
    assert!(g1.residuals.income <= FULL);
    for params in [
        HouseholdParams::from_parcels(e0(WorkerParams::from_machines(crate::support_1c::m4(1.0)))),
        HouseholdParams::from_parcels(e0(entrant_trained(8.0, 3.0, 1.0, 1.0))),
        HouseholdParams::from_parcels(k2()),
        g1_with(dividend(0.5)),
        g1_with(consumption(0.25)),
        household(e2_parcels(), er_government()),
        ces(g1_parcels(4.0, 0.05, 1.0), 0.5),
    ] {
        let (_, eq) = checked_1f(params);
        let b = &eq.base.base;
        assert!(b.residuals.income <= FULL && b.residuals.expenditure <= FULL);
        close(
            "I = W + R + interest",
            b.income,
            b.wage_bill + eq.base.rent * eq.base.land.market + b.interest,
        );
        assert!(eq.residuals.spending <= FULL && eq.residuals.composites <= FULL);
    }
}

#[test]
fn three_taxes_ledger() {
    // (φ_w, φ_r) = (0.6, 0.4) on the worked instance: 1a's price block, and TX's machine service
    // and basket inside the full closure.
    let c = closure(0.5, 0.1, 3.0, 0.2, 1.0, 1.0).unwrap();
    close("φ_w, price block", c.phi_w.unwrap(), 0.6);
    close("φ_r, price block", c.phi_r.unwrap(), 0.4);
    let (_, tx) = checked_1f(tx(Government::none()));
    close("φ_w, machine", tx.base.base.phi_w.unwrap(), TX_PHI_W);
    close("φ_r, machine", tx.base.base.phi_r.unwrap(), TX_PHI_R);
    close("φ_w, basket", tx.ledger.basket_wage_share, 0.6);
    close("φ_r, basket", tx.ledger.basket_rent_share.unwrap(), 0.4);
}

#[test]
fn wall_and_interior_cases() {
    // Constructed cases, each recognised: the line (G1), the wall (W1), the all-human corner (W4),
    // idle land (W2 and WI), a wall with land scarce under CES (CW), an enclosure tie (Q2),
    // NotViable, MultipleEquilibria (M), NoMarket (I4 with exit (2, 0, 0.5)) and SurplusLabour
    // (SL).
    let margin = |p: HouseholdParams| {
        let eq = solved_1f(p);
        (eq.base.base.margin, eq.base.land_market)
    };
    assert_eq!(
        margin(g1_with(Government::none())),
        (Margin::Contestable, LandMarket::Scarce)
    );
    assert_eq!(
        margin(household(g1_parcels(4.0, 0.6, 1.0), Government::none())),
        (Margin::Wall, LandMarket::Scarce)
    );
    assert_eq!(
        margin(household(g1_parcels(20.0, 0.05, 0.05), Government::none())),
        (Margin::AllHuman, LandMarket::Scarce)
    );
    assert_eq!(
        margin(household(g1_parcels(0.25, 0.05, 1.0), Government::none())).1,
        LandMarket::Idle
    );
    assert_eq!(
        margin(household(g1_parcels(4.0, 0.6, 1.0), payroll(0.2))).1,
        LandMarket::Idle
    );
    assert_eq!(
        margin(ces(g1_parcels(4.0, 0.6, 3.0), 0.5)),
        (Margin::Wall, LandMarket::Scarce)
    );
    let q2 = solved_1f(HouseholdParams::from_parcels(race(60.0, 2.0)));
    assert!(q2.base.enclosure.is_some());
    close("Q2 x*", q2.base.base.x_star, Q2_X_STAR);
    let solve = |p: ParcelParams, g: Government| economy_1f(household(p, g)).solve();
    assert!(matches!(
        solve(
            e0(WorkerParams::from_machines(crate::support_1c::m4(50.0))),
            Government::none()
        ),
        Ok(Regime::NotViable { .. })
    ));
    assert!(matches!(
        solve(m_economy(), Government::none()),
        Err(SolveError::MultipleEquilibria { .. })
    ));
    assert!(matches!(
        solve(i4(2.0), Government::none()),
        Err(SolveError::NoMarket { .. })
    ));
    assert!(matches!(
        solve(g1_parcels(20.0, 0.05, 0.05), program(0.1, 0.0)),
        Err(SolveError::SurplusLabour { .. })
    ));
}

#[test]
fn each_exit_form_on_its_gate() {
    // The dependence form on 1a's gate (G1's goldens), s(q) on 1e's constructed gate (K1's
    // commons and Q2's enclosure tie), and both under a government: GP (dependence) and KT's pair
    // (priced).
    let g1 = solved_1f(g1_with(Government::none()));
    close("G1 x*", g1.base.base.x_star, G1_X_STAR);
    let k1 = solved_1f(HouseholdParams::from_parcels(k1()));
    close("K1 x*", k1.base.base.x_star, K1_X_STAR);
    close("K1 v", k1.base.base.v, K1_V);
    let q2 = solved_1f(HouseholdParams::from_parcels(race(60.0, 2.0)));
    close("Q2 q = q_enc", q2.base.q, Q2_Q);
    let gp = solved_1f(g1_with(payroll(0.1)));
    close("GP x*", gp.base.base.x_star, GP_X_STAR);
    let a = solved_1f(household(k3(), consumption(0.25)));
    let b = solved_1f(household(k3(), payroll(0.2)));
    same_allocation("KT", &a, &b, 1e-12);
    close("KT x*", a.base.base.x_star, KT_X_STAR);
}

#[test]
fn the_checklist_names_real_tests() {
    // Every `module::test` this module's table names is a test of the gate crate, and every
    // `module::*` a module of it: a test renamed or removed breaks the checklist here.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/gate");
    let table = include_str!("p1_gate.rs")
        .lines()
        .take_while(|l| l.starts_with("//!"))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let mut named = 0;
    for token in table.split('`').skip(1).step_by(2) {
        let Some((module, test)) = token.split_once("::") else {
            continue;
        };
        if !module
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            continue;
        }
        let source = std::fs::read_to_string(dir.join(format!("{module}.rs")))
            .unwrap_or_else(|_| panic!("the checklist names {module}, which is not a module"));
        if test != "*" {
            assert!(
                source.contains(&format!("fn {test}()")),
                "the checklist names {module}::{test}, which is not a test"
            );
        }
        named += 1;
    }
    assert!(named >= 40, "{named}");
}
