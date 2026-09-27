//! The oracle: the static equilibrium of the pinning paper's economy.
//!
//! Unit 1a (docs/unit-1a.md) solves one category: one final good, one machine type and one
//! land input, with durability and interest entering through the scalar user cost
//! u = (ρ + δ)(1 + ρ)^(J_b − 1). At (ρ, δ, J_b) = (0, 1, 1) it is the SSRN Appendix B
//! economy (SSRN 7226858, pp.28-30), and it reproduces that appendix's published numbers.
//!
//! Unit 1b (docs/unit-1b.md) extends it to many categories bought in a fixed basket, on one
//! task line: [`CategoryEconomy`], with the fork identity, the category bounds and the
//! purchasing-power pair at every equilibrium ([`Eq1b`]). A one-category economy solves to
//! 1a's equilibrium bit for bit. The price block alone, for categories given as task cells
//! at any prices, and the CES share of SSRN eq 26, are [`cell_cost`] and [`ces_share`].
//!
//! Unit 1c (docs/unit-1c.md) adds many machine types, each with an operating recipe and a
//! build recipe over machine services, labour and land, its own depreciation and build lag,
//! and so its own user cost, and lets categories use each other as intermediate inputs:
//! [`MachineEconomy`], with the Leontief identities over every produced good and the income
//! identity with interest at every equilibrium ([`Eq1c`]). The cheapest task type takes the
//! machine tasks; an equilibrium on a switch between types is a tie, and more than one
//! equilibrium is refused. One type with no operating recipe solves to 1b's equilibrium bit
//! for bit. The machine block alone, at any margin, is [`MachineBlock`].
//!
//! Unit 1d (docs/unit-1d.md) adds worker types and the tasks closed to machines:
//! [`WorkerEconomy`]. The paper's human-required set H is hours any pooled worker can do, and
//! reserved tasks are hours only one type can do. Types share the line's shape, each with an
//! efficiency; those selling on the line form a pool with one wage per efficiency hour, and a
//! type whose reserved work takes all its hours is paid a scarcity price at its own wall. The
//! boundary regimes of units 1a-1c are solved: the wall (x* = 1, the wage set by labour
//! clearing above labour's replacement value at the top task), the all-human corner and roots
//! below 10^-12; an economy with no wage that clears labour with land fully rented is
//! [`SolveError::LaborShort`]. One type with no human-required or reserved hours is unit 1c's
//! equilibrium bit for bit ([`Eq1d`]).
//!
//! Unit 1e (docs/unit-1e.md) cuts the land into parcels of an acreage, a quality and an access,
//! enclosed or open (a commons), and prices exit: [`ParcelEconomy`]. Each worker type names its
//! exit form, SSRN's dependence form or main.tex's s(q) = max(s₀ − q·h, s̲) in units of one exit
//! good ([`PricedExit`]), both under SSRN eq 8 with the exit life's value. Exit plots take land:
//! on the commons while it has room, at a shadow rent when it is full, and rented on enclosed
//! land, which leaves production. Idle enclosed land earns zero rent, and the path continues past
//! the wall's end onto an idle stretch with the pool's wage as numeraire, where 1d's
//! `LaborShort` economies have their equilibria; no one working at any wage is
//! [`SolveError::NoMarket`]. An equilibrium can sit where q crosses a type's q_enc, a share of its
//! exiters renting ([`EnclosureTie`]), and coverage, q* and N_crit are reported ([`Eq1e`],
//! [`coverage`]). Unit 1d's economies in parcel form, and priced forms with the exit option
//! switched off, solve to 1d's equilibria bit for bit.
//!
//! Unit 1f (docs/unit-1f.md) adds households and a government: [`HouseholdEconomy`]. The
//! government has SSRN A.1's payroll tax, uniform tax on final purchases, tax on market rent,
//! uniform transfer and program (m_w, m_e), transfers counted in composites at consumer prices,
//! and a budget closed by the owners' levy ([`Budget::RentRate`], whose rate at one composite per
//! person is SSRN eq 16's 1/κ) or by the uniform transfer ([`Budget::Dividend`]). Producers pay
//! gross prices, so the government enters through one participation rule with SSRN eq 9, p.16,
//! eq 27 and 1d's and 1e's forms as cases; an in-work benefit that alone overfills the economy is
//! [`SolveError::SurplusLabour`]. The basket is 1b's fixed one or a CES over the categories
//! ([`Basket::Ces`], SSRN eq 26). Every equilibrium reports the budget, the households' and the
//! provider's accounts and three-taxes' ledger ([`Eq1f`]); with the fixed basket and
//! [`Government::none`] it is unit 1e's equilibrium bit for bit. Unit 1f closes Phase 1.
//!
//! Every price is in units of the land rent r = 1, so v = w/r, except at unit 1e's equilibria
//! on idle land, where r = 0 and prices are in units of the pool's wage
//! ([`Eq1e::land_market`]).
//!
//! The oracle shares types but not logic with the agents. No agent may read it
//! (PLAN R13).
//!
//! ```
//! use oracle::{Economy, Params, PowerSchedule, Regime, UniformWorkCost};
//!
//! // The SSRN Appendix B instance (SSRN p.30).
//! let params = Params {
//!     workers: 4.0, land: 10.0, space: 1.0, a: 0.3, lam: 0.05, b: 0.4,
//!     schedule: PowerSchedule { eta: 1.0, g0: 0.2, g1: 0.8, k: 1.0 },
//!     work_cost: UniformWorkCost { chi_max: 1.0 },
//!     rho: 0.0, delta: 1.0, build_lag: 1,
//! };
//! let economy = Economy::new(params).expect("valid parameters");
//! match economy.solve().expect("finite") {
//!     Regime::Interior(eq) => assert!((eq.x_star - 0.86315).abs() < 5e-6),
//!     other => panic!("expected an interior equilibrium, got {}", other.name()),
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod categories;
mod closure;
pub mod dump;
mod exit;
mod fork;
mod households;
mod leontief;
mod machine_block;
mod machines;
mod params;
mod parcels;
mod schedule;
mod solve;
mod workers;

pub use categories::{
    Category, CategoryEconomy, CategoryEq, CategoryParams, CategoryPoint, Eq1b, Output1b,
    OutputKey, Residuals1b,
};
pub use closure::{closure, Closure, ClosureError};
pub use exit::{coverage, coverage_threshold, crowding_limit, PricedExit};
pub use fork::{cell_cost, ces_share, CategoryCost, Cell};
pub use households::{
    Accounts, Basket, BasketEq, Budget, Eq1f, Government, GovernmentEq, HouseholdEconomy,
    HouseholdParams, HouseholdPoint, Ledger, Program, ProviderAccount, Residuals1f, TransferMode,
    TypeAccount, SIGMA_CEIL,
};
pub use machine_block::{
    BlockPrices, BlockTotals, Envelope, MachineBlock, MachineType, Recipe, Switch,
};
pub use machines::{
    CategoryEq1c, Eq1c, Item, MachineEconomy, MachineParams, MachinePoint, OutputKey1c,
    Residuals1c, SwitchPoint, Tie, TypeEq,
};
pub use params::{
    user_cost, Economy, ParamError, Params, Requirement, UniformWorkCost, SCALE_CEIL, SCALE_FLOOR,
};
pub use parcels::{
    Access, Branch, EnclosurePoint, EnclosureSide, EnclosureTie, Eq1e, ExitForm, ExitLand,
    HomeAccount, LandEq, LandMarket, Parcel, ParcelEconomy, ParcelEq, ParcelParams, ParcelPoint,
    Residuals1e, WorkerEq1e, EXIT_SCAN,
};
pub use schedule::{PowerSchedule, Schedule, CURVATURE_CEIL, VALIDATION_SAMPLES};
pub use solve::{
    CostSystem, Eq1a, Output, Point, Regime, Residuals, SolveError, BRACKET_HI, BRACKET_LO,
    LABOR_RESIDUAL_NET, MAX_BISECTION_STEPS,
};
pub use workers::{
    CategoryEq1d, Edge, Eq1d, Margin, Residuals1d, Shortage, WallEnd, WallSwitch, WorkerEconomy,
    WorkerEq, WorkerParams, WorkerPoint, WorkerType,
};

/// 2^n exactly, for the unit tests' dyadic inputs (`powi` is denied: clippy.toml, A5). The
/// double with that exponent and a zero mantissa; n must be in the normal range.
#[cfg(test)]
pub(crate) fn pow2(n: i32) -> f64 {
    assert!((-1022..=1023).contains(&n), "2^{n} is not a normal double");
    f64::from_bits(((1023 + n) as u64) << 52)
}
