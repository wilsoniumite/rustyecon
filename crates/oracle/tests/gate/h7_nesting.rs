//! h7: unit 1g's economies through units 1d, 1e and 1f (docs/unit-1g.md §2.7 and §8). D-G10 is
//! 1c's validation, which 1d, 1e and 1f call, so an economy it admits is admitted by each; and
//! with one worker type, the dependence exit in parcel form, the fixed basket and no government,
//! each solves it bit for bit as 1c does (decision 63).

use oracle::{
    Basket, Government, HouseholdParams, MachineParams, Margin, ParcelParams, PlantEconomy, Regime,
    WorkerParams,
};

use crate::support_1c::*;
use crate::support_1d::*;
use crate::support_1e::*;
use crate::support_1f::*;
use crate::support_1g::*;

/// 1c's equilibrium is 1d's, 1d's is 1e's base, and 1e's is 1f's base, bit for bit.
fn nests(what: &str, params: MachineParams) {
    let c = interior_1c(params.clone());
    let workers = WorkerParams::from_machines(params);
    let d = solved_1d(workers.clone());
    let d_bits = bits_1d(&d);
    let mut compared = 0;
    for (key, bits) in bits_1c(&c) {
        assert_eq!(Some(&bits), d_bits.get(&key), "{what}: {key} in 1d");
        compared += 1;
    }
    assert!(compared >= 52, "{what}: {compared}");
    assert_eq!(d.margin, Margin::Contestable, "{what}");
    let parcels = ParcelParams::from_workers(workers);
    let e = solved_1e(parcels.clone());
    assert_eq!(bits_1d(&e.base), d_bits, "{what}: 1e's base");
    let f = solved_1f(HouseholdParams {
        economy: parcels,
        basket: Basket::Fixed,
        government: Government::none(),
    });
    assert_eq!(bits_1e(&f.base), bits_1e(&e), "{what}: 1f's base");
}

#[test]
fn chains_nest_through_1d_1e_and_1f() {
    for (what, chain) in [
        ("S1", s1(0.05)),
        ("S1Z", s1(0.0)),
        ("S2", s2(false, 0.05)),
        ("S2H", s2(true, 0.05)),
        ("A0", a0(0.0)),
        ("A0R", a0(rho_per_week(0.05))),
        ("HORSE", horse()),
    ] {
        nests(what, chain.to_machine_params().unwrap());
    }
    nests("A0 as one type", a0_one_type(0.0));
}

#[test]
fn plants_nest_through_1d_1e_and_1f() {
    for plants in [
        vec![(0, bundle_plant(None)), (1, bundle_plant(None))],
        vec![(0, labour_land_plant()), (1, labour_land_plant())],
    ] {
        let economy = PlantEconomy::new(l2(0.0), plants).unwrap();
        let Ok(Regime::Interior(eq)) = economy.solve() else {
            panic!("interior")
        };
        nests("a plant's long run", economy.long_run(&eq.ratios));
    }
}
