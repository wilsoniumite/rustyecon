//! e7: multiplicity and the scan (docs/unit-1e.md §2.11, §5.3 and §5.4): M has three equilibria,
//! two inside one region of the line whose ends have the same sign; Lemma 5's σ_i against a
//! finite difference; the root does not depend on the scan.

use oracle::{
    Branch, ExitForm, ParcelEconomy, ParcelPoint, Regime, SolveError, BRACKET_HI, EXIT_SCAN,
};
use rustyecon_core::num;

use crate::goldens_1e::*;
use crate::support::*;
use crate::support_1e::*;

#[test]
fn three_equilibria() {
    let e = economy_1e(m_economy());
    match e.solve() {
        Err(SolveError::MultipleEquilibria { sign_changes, .. }) => assert_eq!(sign_changes, 3),
        other => panic!("M gave {other:?}"),
    }
    // with the scan off it counts one, and reports the corner's
    match e.solve_scanned(0) {
        Ok(Regime::Interior(eq)) => close("the corner's v", eq.base.v, M_V_CORNER),
        other => panic!("M without the scan gave {other:?}"),
    }
    // the line's ends have the same sign; f changes sign at the goldens' two roots
    let env = e.workers().machines().envelope();
    let t = env.first;
    close("f_line(0)", e.at_with(0.0, t).excess_demand(), M_F_LINE_0);
    close("f_line(1)", e.at_with(1.0, t).excess_demand(), M_F_LINE_1);
    for x in [M_X_LOW, M_X_HIGH] {
        let (below, above) = (
            e.at_with(x * (1.0 - 1e-9), t).excess_demand(),
            e.at_with(x * (1.0 + 1e-9), t).excess_demand(),
        );
        assert!(below.signum() != above.signum(), "a root at {x}");
    }
    close("v low", e.at_with(M_X_LOW, t).point.v, M_V_LOW);
    close("v high", e.at_with(M_X_HIGH, t).point.v, M_V_HIGH);
    // the corner's root: f changes sign at M_V_CORNER on the all-human corner
    let (below, above) = (
        e.at_wage(0.0, M_V_CORNER * (1.0 - 1e-9), t).excess_demand(),
        e.at_wage(0.0, M_V_CORNER * (1.0 + 1e-9), t).excess_demand(),
    );
    assert!(below > 0.0 && above < 0.0);
    // a scan too coarse to separate the line's pair misses it
    assert!(matches!(e.solve_scanned(1), Ok(Regime::Interior(_))));
}

/// σ_i of Lemma 5 at a point: ν_i·B_s·(ε_i·v − e_i) plus (s₀·b̃_g − r_o·h)(ν_i·P_s + ε_i·v) on
/// the plot branch, s̲·b̃_g(…) on the floor, with B_s and b̃_g the point's price-side land totals
/// (the prices at v = 0 at the same x and technique).
fn sigma(e: &ParcelEconomy, q: &ParcelPoint, i: usize) -> f64 {
    let p = e.params();
    let ty = &p.worker_types[i];
    let land = e.at_wage(q.point.x, 0.0, q.point.technique);
    let (b_s, b_g) = (land.point.base_p_s, land.point.base_prices[p.exit_good]);
    let (v, p_s, e_i) = (q.point.v, q.point.p_s, q.exit_values[i]);
    let base = ty.support * b_s * (ty.efficiency * v - e_i);
    let scale = ty.support * p_s + ty.efficiency * v;
    match (p.exits[i], q.branches[i]) {
        (ExitForm::Priced(x), Branch::Plot) => {
            base + (x.gross * b_g - q.plot_rent * x.plot) * scale
        }
        (ExitForm::Priced(x), _) => base + x.floor * b_g * scale,
        (ExitForm::Dependence, _) => ty.support * b_s * ty.efficiency * v,
    }
}

/// The marginal worker's cost z at `q`'s prices with the plot rent `plot_rent` held.
fn cost_at(e: &ParcelEconomy, q: &ParcelPoint, i: usize, plot_rent: f64, branch: Branch) -> f64 {
    let p = e.params();
    let ty = &p.worker_types[i];
    let p_g = q.exit_good_price;
    let exit = match (p.exits[i], branch) {
        (ExitForm::Priced(x), Branch::Plot) => num::fma(p_g, x.gross, -(plot_rent * x.plot)),
        (ExitForm::Priced(x), _) => p_g * x.floor,
        (ExitForm::Dependence, _) => 0.0,
    };
    num::ln1p((ty.efficiency * q.point.v - exit) / (ty.support * q.point.p_s + exit))
}

/// Lemma 5 at points along the line and a corner of an economy: returns (checked, negative).
fn lemma5(e: &ParcelEconomy) -> (usize, usize) {
    let env = e.workers().machines().envelope();
    let (mut checked, mut negative) = (0, 0);
    let mut check = |a: &ParcelPoint, b: &ParcelPoint| {
        for i in 0..a.branches.len() {
            let dz = cost_at(e, b, i, a.plot_rent, a.branches[i])
                - cost_at(e, a, i, a.plot_rent, a.branches[i]);
            let s = sigma(e, a, i);
            if dz.abs() > 1e-11 && s.abs() > 1e-9 {
                assert_eq!(dz > 0.0, s > 0.0, "Lemma 5 at v {} type {i}", a.point.v);
                checked += 1;
                negative += usize::from(s < 0.0);
            }
        }
    };
    let t = env.first;
    for k in 1..40 {
        let x = f64::from(k) / 40.0;
        let (a, b) = (e.at_with(x, t), e.at_with(x + 1e-7, t));
        if a.branches == b.branches && a.exit_land == b.exit_land {
            check(&a, &b);
        }
    }
    let v0 = e.at_with(0.0, t).point.v;
    for k in 1..20 {
        let v = v0 * f64::from(k) / 20.0;
        let (a, b) = (e.at_wage(0.0, v, t), e.at_wage(0.0, v * (1.0 + 1e-7), t));
        if a.branches == b.branches && a.exit_land == b.exit_land {
            check(&a, &b);
        }
    }
    let one = e.at_with(BRACKET_HI, env.last());
    for k in 1..20 {
        let v = one.point.v * (1.0 + f64::from(k) / 4.0);
        let (a, b) = (
            e.at_wage(BRACKET_HI, v, env.last()),
            e.at_wage(BRACKET_HI, v * (1.0 + 1e-7), env.last()),
        );
        if a.branches == b.branches && a.exit_land == b.exit_land {
            check(&a, &b);
        }
    }
    (checked, negative)
}

#[test]
fn supply_slope() {
    // Lemma 5: a type's supply moves along the path with σ_i's sign, at a fixed plot rent. K1
    // and K3 hold plots on land-free goods; M's plot is land-extensive, and its σ is negative
    // somewhere: both signs.
    let (mut checked, mut negative) = (0, 0);
    for p in [k1(), k3(), m_economy()] {
        let (c, n) = lemma5(&economy_1e(p));
        checked += c;
        negative += n;
    }
    assert!(checked >= 100, "{checked}");
    assert!(
        negative > 0 && negative < checked,
        "{negative} of {checked}"
    );
}

#[test]
fn root_does_not_depend_on_the_scan() {
    // §5.3 step 4: the root is found on the piece's own ends, never the scan cell's.
    for p in [k3(), race(50.0, 2.5)] {
        let e = economy_1e(p);
        let (Ok(Regime::Interior(a)), Ok(Regime::Interior(b))) = (e.solve(), e.solve_scanned(17))
        else {
            panic!("an equilibrium");
        };
        assert_eq!(a.scan_points % EXIT_SCAN as u32, 0);
        assert_eq!(b.scan_points % 17, 0);
        let (mut a_bits, mut b_bits) = (bits_1e(&a), bits_1e(&b));
        a_bits.remove("scan_points");
        b_bits.remove("scan_points");
        assert_eq!(a_bits, b_bits);
    }
}
