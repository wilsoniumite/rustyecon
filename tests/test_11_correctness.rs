//! B8, the project's first **correctness** criterion (v2 Phase 4; PLAN, "The
//! missing instrument"; `src/certify/technology.rs`).
//!
//! Falsification first, as with B6 and B7. A guard that cannot fail is not a
//! guard, and this repository has already shipped one — `Rule::Damping`, which
//! rewarded a collapsing economy — so every claim B8 makes is shown here either
//! firing on a defect that is independently known to exist, or reproducing a
//! number that was computed by hand before the code was written.
//!
//! Four things are asserted, and they are different kinds of claim:
//!
//! 1. **The solver is right.** A chain built in this file, whose labour contents
//!    can be worked out on paper, comes out at exactly those numbers — and so
//!    does the shipped `lr_00` tape, at the 0.3 / 0.5 / RealWage 2.0 that PLAN
//!    quotes.
//! 2. **The measurement is sensitive.** `lr_00` under the kernel is off by ~1e11,
//!    a defect nobody disputes, and B8 says so in its own units.
//! 3. **The measurement is directional.** α/100 is *closer* to the implied
//!    vector than α. That is a real, refutable claim about the engine, and it is
//!    the one PLAN's α table rests on.
//! 4. **The failure path fires.** B8 fails only on the threshold-free case, so
//!    that case is provoked twice — once by a tape that does not say how a
//!    traded good is made, once by a run in which the numeraire has no price.

use std::path::PathBuf;

use rustyecon::{
    certify::{
        technology::{labour_values, GapReading, PriceGapWatch, Value},
        Certificate,
    },
    kernel::AgentArm,
    runner::{RunConfig, SimRunner},
    scenario::loader,
    state::{game_data::{KernelParams, SupplyRule}, GameData},
    systems::{clearing::PriceRule, run_tick},
    types::{
        good::{GoodDef, MovementType, ShelfLife},
        ids::{GoodId, MarketNodeId, RecipeId, RegionId},
        market_node::{MarketNodeDef, MarketTier},
        recipe::{InputScaling, RecipeDef, RecipeInput, RecipeOutput, StrategyKind},
    },
};

fn scenario_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/scenarios").join(name)
}

/// Run a scenario the way `--certify` does and hand back the certificate.
fn certify(scenario: &str, ticks: u64, agents: AgentArm) -> Certificate {
    let dir = scenario_dir(scenario);
    let s = loader::load(&dir).expect("scenario loads");
    let out = tempfile::tempdir().unwrap();
    let config = RunConfig {
        agents,
        price_rule: PriceRule::Imbalance,
        ticks,
        checkpoint_every: 0,
        human_save_every: 0,
        output_dir: out.path().to_path_buf(),
        record: false,
        telemetry_every: 1,
        certify: true,
        results_dir: out.path().join("results"),
        // Tape-registered: this run uses whatever kernel.supply_rule the
        // scenario declares, which is the only R2-clean default.
        supply_rule: None,
    };
    let mut runner = SimRunner::new(s.state, s.game_data, s.events, config)
        .with_scenario_dir(&dir)
        .with_criteria(s.criteria);
    runner.run().expect("certify mode yields a certificate")
}

fn battery<'a>(cert: &'a Certificate, id: &str) -> &'a rustyecon::certify::certificate::Battery {
    cert.batteries.iter().find(|b| b.id == id).unwrap_or_else(|| panic!("no {id}"))
}

/// What one run's [`PriceGapWatch`] saw, taken directly rather than through the
/// certificate's rendered line.
struct Measured {
    readings: Vec<GapReading>,
    pass: bool,
    detail: String,
}

impl Measured {
    /// The largest |log distance| any (region, good) recorded — the single
    /// number the "is α/100 closer" claim is made on.
    fn worst_log_gap(&self) -> f64 {
        self.readings
            .iter()
            .map(|r| r.log_gap.abs())
            .fold(0.0f64, f64::max)
    }

    fn find(&self, region: &str, good: &str) -> &GapReading {
        self.readings
            .iter()
            .find(|r| r.region == region && r.good == good)
            .unwrap_or_else(|| panic!("no reading for {region}/{good}"))
    }
}

/// Run a scenario and accumulate B8's statistic over the registered window.
///
/// `alpha_div` divides every good's price-adjustment speed, which is how the α
/// arm of PLAN's table is produced: α is a per-good dial in `game_data.ron`, and
/// there is no CLI flag for it. `mangle` gets the tape before the run, so a
/// defect can be injected into the technology itself.
fn measure(
    scenario: &str,
    ticks: u64,
    agents: AgentArm,
    alpha_div: f64,
    mangle: impl FnOnce(&mut GameData),
    mut before_observe: impl FnMut(&mut rustyecon::state::SimState, &GameData),
) -> Measured {
    let dir = scenario_dir(scenario);
    let mut s = loader::load(&dir).expect("scenario loads");
    for g in s.game_data.goods.iter_mut() {
        g.alpha /= alpha_div;
    }
    mangle(&mut s.game_data);
    let criteria = s.criteria.clone().expect("this scenario registers criteria");
    let mut watch = PriceGapWatch::new(&s.game_data, &criteria);
    for _ in 0..ticks {
        run_tick(&mut s.state, &s.game_data, &s.events, agents, PriceRule::Imbalance);
        if criteria.in_analysis(s.state.tick) {
            before_observe(&mut s.state, &s.game_data);
            watch.observe(&s.state);
        }
    }
    let (pass, detail) = watch.report(&s.game_data);
    Measured { readings: watch.readings(&s.game_data), pass, detail }
}

/// The common case: no injected defect, no state tampering.
fn measure_plain(scenario: &str, ticks: u64, agents: AgentArm, alpha_div: f64) -> Measured {
    measure(scenario, ticks, agents, alpha_div, |_| {}, |_, _| {})
}

// ── 1. The solver is right ────────────────────────────────────────────────────

fn good(i: u32, name: &str) -> GoodDef {
    GoodDef {
        id: GoodId(i),
        name: name.into(),
        alpha: 0.1,
        shelf_life: ShelfLife::Indefinite,
        movement_type: MovementType::Physical,
        divisible: true,
        storage_cost_per_tick: 0.0,
    }
}

fn recipe(i: u32, name: &str, ins: &[(u32, f64)], outs: &[(u32, f64)]) -> RecipeDef {
    RecipeDef {
        id: RecipeId(i),
        name: name.into(),
        inputs: ins
            .iter()
            .map(|(g, q)| RecipeInput {
                good: GoodId(*g),
                qty_per_unit: *q,
                scaling: InputScaling::Variable,
            })
            .collect(),
        outputs: outs
            .iter()
            .map(|(g, q)| RecipeOutput { good: GoodId(*g), qty_per_unit: *q })
            .collect(),
        component_reqs: Vec::new(),
        reversible: false,
        strategy: StrategyKind::CapacityControl,
    }
}

fn tiny_tape(goods: Vec<GoodDef>, recipes: Vec<RecipeDef>, currency: Option<u32>) -> GameData {
    GameData {
        goods,
        recipes,
        kernel: KernelParams {
            s: 4,
            eta_up: 0.04,
            eta_dn: 0.05,
            dead: 0.05,
            b_out: 2.0,
            b_cash: 6.5,
            beta: 1.0,
            epsilon: 0.01,
            phi: 0.6180339887498949,
            fill_alpha: 0.25,
            supply_rule: SupplyRule::Inelastic,
        },
        need_categories: Vec::new(),
        wealth_levels: Vec::new(),
        market_nodes: vec![MarketNodeDef {
            id: MarketNodeId(0),
            tier: MarketTier::Regional,
            region: Some(RegionId(0)),
            currency_good: currency.map(GoodId),
        }],
        channels: Vec::new(),
        regions: vec![rustyecon::state::game_data::RegionDef {
            id: RegionId(0),
            name: "Only".into(),
            market_node: MarketNodeId(0),
            position: None,
        }],
    }
}

/// A four-deep chain whose labour contents can be worked out on paper, with the
/// three features that make the solve non-trivial all present at once: a cycle,
/// a pass-through, a competing route, and a multi-unit output.
///
/// Worked by hand, before the assertions were written:
///
/// ```text
///   dig       2 labour            -> 1 ore        ore   = 2
///   smelt     1 ore + 0.5 labour  -> 2 iron       iron  = (2 + 0.5)/2 = 1.25
///   forge     2 iron + 1 labour   -> 1 tool       tool  = 2*1.25 + 1 = 3.5
///   scrap     1 tool + 0.1 labour -> 4 iron       iron' = (3.5 + 0.1)/4 = 0.9
///                                                 -> tool' = 2*0.9 + 1 = 2.8
///                                                 -> iron'' = (2.8+0.1)/4 = 0.725
///                                                 ... fixed point of
///                                                     x = (2x + 1 + 0.1)/4
///                                                     -> 2x = 1.1 -> x = 0.55
///                                                 tool = 2*0.55 + 1 = 2.1
///   haul      1 tool + 9 labour   -> 1 tool       pass-through, ignored
/// ```
///
/// The `haul` line is the one that matters most: it is `flour_transport`'s shape,
/// and if it were treated as production it would define tool in terms of itself
/// and drag every number in the chain up by 9.
#[test]
fn the_solver_reproduces_a_chain_computed_by_hand() {
    let gd = tiny_tape(
        vec![
            good(0, "labour"),
            good(1, "ore"),
            good(2, "iron"),
            good(3, "tool"),
            good(4, "GBP"),
        ],
        vec![
            recipe(0, "dig", &[(0, 2.0)], &[(1, 1.0)]),
            recipe(1, "smelt", &[(1, 1.0), (0, 0.5)], &[(2, 2.0)]),
            recipe(2, "forge", &[(2, 2.0), (0, 1.0)], &[(3, 1.0)]),
            recipe(3, "scrap", &[(3, 1.0), (0, 0.1)], &[(2, 4.0)]),
            recipe(4, "haul", &[(3, 1.0), (0, 9.0)], &[(3, 1.0)]),
        ],
        Some(4),
    );
    let v = labour_values(&gd, GoodId(0));

    assert_eq!(v.get(GoodId(0)), Value::Labour(1.0), "labour is the unit");
    assert_eq!(v.get(GoodId(1)), Value::Labour(2.0), "ore");
    assert_eq!(v.get(GoodId(4)), Value::Currency, "GBP is money, not a product");

    let iron = v.get(GoodId(2)).relative_price().expect("iron resolves");
    let tool = v.get(GoodId(3)).relative_price().expect("tool resolves");
    assert!((iron - 0.55).abs() < 1e-9, "iron came out {iron}, hand value 0.55");
    assert!((tool - 2.1).abs() < 1e-9, "tool came out {tool}, hand value 2.1");
    assert!(
        tool < 9.0,
        "the 9-labour haul is a pass-through; treating it as production would \
         define tool in terms of itself and put it above 9: got {tool}"
    );
}

/// The shipped tape, at the numbers PLAN quotes. This is the anchor the whole
/// criterion hangs on: if `lr_00` does not give flour 0.5, every distance B8
/// reports is measured from the wrong place.
#[test]
fn lr_00s_own_tape_gives_the_numbers_plan_quotes() {
    let s = loader::load(&scenario_dir("lr_00")).expect("lr_00 loads");
    let labour = s.game_data.goods.iter().find(|g| g.name == "labour").unwrap().id;
    let v = labour_values(&s.game_data, labour);

    let by_name = |n: &str| {
        let id = s.game_data.goods.iter().find(|g| g.name == n).unwrap().id;
        v.get(id)
    };
    // wheat_farm spends 0.3 labour per wheat; grain_mill adds 0.2 on top of one
    // wheat; local_services turns 1 labour into 1 service.
    assert_eq!(by_name("wheat"), Value::Labour(0.3));
    assert_eq!(by_name("flour"), Value::Labour(0.5));
    assert_eq!(by_name("services"), Value::Labour(1.0));
    assert_eq!(by_name("GBP"), Value::Currency);

    // The headline: a zero-profit equilibrium has RealWage = p_labour/p_flour.
    let implied_real_wage = 1.0 / by_name("flour").relative_price().unwrap();
    assert_eq!(implied_real_wage, 2.0, "PLAN, 'The missing instrument'");

    // And flour_transport must not have been mistaken for a mill.
    assert!(v.unresolved(&s.game_data).is_empty(), "{:?}", v.unresolved(&s.game_data));
}

// ── 2. The measurement is sensitive ───────────────────────────────────────────

/// The positive control. `lr_00` under the kernel is not a borderline case: PLAN
/// records its median RealWage at 4.9e-10 against an implied 2.0, off by ~4e9,
/// with a band of 1.25e17. An instrument that reported a modest number here
/// would be measuring nothing.
///
/// Note what this test does **not** assert: that B8 *fails*. It does not, and
/// that is deliberate — see `PriceGapWatch::report`, which argues why the
/// distance carries no threshold. The claim under test is that the number is
/// there and is enormous, which is what makes the criterion usable; the claim
/// that a threshold would add is one this project has no evidence to make yet.
#[test]
fn b8_reports_the_kernels_known_price_failure_in_its_own_units() {
    let m = measure_plain("lr_00", 1000, AgentArm::Kernel, 1.0);
    let worst = m.worst_log_gap();
    assert!(
        worst.exp() > 1e6,
        "the kernel's prices are known to be off by ~1e9; B8 saw only {:.3e}x: {}",
        worst.exp(),
        m.detail
    );

    // And it must be readable as a diagnosis, not as a bare distance. Which
    // market, and which way.
    let flour = m.find("Leeds", "flour");
    assert!(flour.log_gap > 0.0, "flour is the good that ran away upward");
    assert!(flour.phrase().starts_with("flour "), "{}", flour.phrase());
    assert!(flour.phrase().contains("too dear"), "{}", flour.phrase());
    // The other side of the same run: wheat collapsed while flour exploded, and
    // a criterion that only reported a magnitude would have shown these two as
    // the same finding.
    let wheat = m.find("Leeds", "wheat");
    assert!(wheat.log_gap < 0.0, "wheat went the other way: {}", wheat.phrase());
    assert!(wheat.phrase().contains("too cheap"), "{}", wheat.phrase());
}

/// The legacy arm at the registered α is wrong too, and in the same direction
/// everywhere. Recorded because it is the finding, not the baseline: before B8
/// existed the only thing said about these runs was that their bands were wide.
#[test]
fn b8_reports_the_legacy_arms_error_at_the_registered_alpha() {
    let m = measure_plain("lr_00", 1000, AgentArm::Legacy, 1.0);
    // Manchester's RealWage band is 11.5 — the tidiest region in the corpus by
    // the stability criteria — and its flour is still two orders of magnitude
    // too dear. Stability and correctness are different questions, and this is
    // the run that shows it.
    let flour = m.find("Manchester", "flour");
    assert!(
        flour.log_gap.exp() > 50.0,
        "Manchester passes for stable and is still badly mispriced: {}",
        flour.phrase()
    );
    assert!(flour.phrase().contains("too dear"));
}

// ── 3. The measurement is directional ─────────────────────────────────────────

/// The claim PLAN's α table makes, tested: slowing the price rule by 100×
/// **moves prices toward** the vector technology implies. It is refutable — the
/// engine could easily have been no better, or worse — and it is the evidence
/// that the mechanism can form prices at all and is merely being run outside the
/// region where it works.
///
/// Asserted for both arms, because the finding is about the *price rule*, not
/// about who is posting.
#[test]
fn slowing_alpha_moves_prices_toward_the_implied_vector() {
    for arm in [AgentArm::Legacy, AgentArm::Kernel] {
        let fast = measure_plain("lr_00", 1000, arm, 1.0);
        let slow = measure_plain("lr_00", 1000, arm, 100.0);
        assert!(
            slow.worst_log_gap() < fast.worst_log_gap(),
            "{arm:?}: alpha/100 must be closer, got {:.3e}x vs {:.3e}x\n  fast: {}\n  slow: {}",
            slow.worst_log_gap().exp(),
            fast.worst_log_gap().exp(),
            fast.detail,
            slow.detail
        );
    }
}

/// The specific number PLAN quotes, reproduced. `legacy, α×0.01` realises a
/// median RealWage of 0.799 against an implied 2.0 — flour about 2.5× too dear.
/// B8 uses the geometric mean rather than the median (the memory argument is on
/// `PriceGapWatch`), so the digits are not identical and are not meant to be;
/// the order of magnitude is the claim.
#[test]
fn at_alpha_over_100_the_legacy_arm_lands_within_a_small_factor() {
    let m = measure_plain("lr_00", 1000, AgentArm::Legacy, 100.0);
    for region in ["Manchester", "Leeds", "Birmingham"] {
        let flour = m.find(region, "flour");
        let x = flour.log_gap.exp();
        assert!(
            (1.5..4.0).contains(&x),
            "{region}: PLAN puts this near 2.5x too dear; B8 saw {}",
            flour.phrase()
        );
    }
    // And the direction survives at slow α: labour is still underpaid against
    // its embodied value, which is the same market the elasticity probe found
    // has no crossing.
    assert!(m.readings.iter().all(|r| r.log_gap > 0.0), "{}", m.detail);
}

// ── 4. The failure path fires ─────────────────────────────────────────────────

/// Falsification, part one: a tape that does not say how a traded good is made.
///
/// `wheat_farm` is rewritten to consume wheat instead of labour, so it is a
/// pass-through and nothing in the world produces wheat — but wheat is still
/// endowed at genesis and still demanded by the mill, so it trades throughout.
/// B8 must fail, and must name wheat: a benchmark it cannot compute is not a
/// benchmark it may quietly skip.
///
/// This is the exact shape of a defect that already exists in the corpus:
/// `big_region` declares `iron` and `grapes` with no recipe that makes either.
/// That scenario registers no labour good so B8 reports it unscored — but the
/// moment such a world gets a labour market, this is what happens to it.
#[test]
fn b8_fails_when_a_traded_good_has_no_implied_value() {
    let m = measure(
        "lr_00",
        400,
        AgentArm::Legacy,
        1.0,
        |gd| {
            let wheat = gd.goods.iter().find(|g| g.name == "wheat").unwrap().id;
            let farm = gd.recipes.iter_mut().find(|r| r.name == "wheat_farm").unwrap();
            farm.inputs[0].good = wheat;
        },
        |_, _| {},
    );
    assert!(!m.pass, "an unmakeable good must fail B8: {}", m.detail);
    assert!(m.detail.contains("wheat"), "and must name it: {}", m.detail);
    assert!(
        m.detail.contains("no recipe route reaches labour"),
        "and must say why, so the fix is obvious: {}",
        m.detail
    );

    // The control: the same scenario with its tape intact does not fail, so the
    // assertion above is about the injected defect and not about lr_00.
    let clean = measure_plain("lr_00", 400, AgentArm::Legacy, 1.0);
    assert!(clean.pass, "{}", clean.detail);
}

/// Falsification, part two: the numeraire loses its price.
///
/// Every relative price in this criterion is `p_good / p_labour`. With labour at
/// zero the ratio has no logarithm and the criterion has no answer — which is
/// not "a distance of zero", and must never be reported as one. Under the house
/// fail-closed rule (engine.md, "NaN = FAIL") this fails.
///
/// Injected rather than found, because no corpus run currently drives a posted
/// price to exactly zero. The branch is real all the same: the kernel arm
/// already takes wheat and labour toward zero on `lr_00`, and "toward" is one
/// rounding away from "to".
#[test]
fn b8_fails_when_the_numeraire_has_no_price() {
    let m = measure(
        "lr_00",
        400,
        AgentArm::Legacy,
        1.0,
        |_| {},
        |state, gd| {
            let labour = gd.goods.iter().find(|g| g.name == "labour").unwrap().id;
            for node in &gd.market_nodes {
                state.set_price(node.id, labour, 0.0);
            }
        },
    );
    assert!(!m.pass, "a priceless numeraire must fail B8: {}", m.detail);
    assert!(
        m.detail.contains("no computable relative price"),
        "and must say that the distance is missing, not that it is zero: {}",
        m.detail
    );
    assert!(m.readings.is_empty(), "nothing may be reported as measured");
}

// ── 5. Wiring ─────────────────────────────────────────────────────────────────

#[test]
fn b8_is_in_the_certificate_beside_b6_and_b7() {
    let cert = certify("lr_00", 200, AgentArm::Legacy);
    let ids: Vec<&str> = cert.batteries.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, ["B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8"]);
    let b8 = battery(&cert, "B8");
    assert!(!b8.detail.is_empty(), "a battery must carry its measurement");
    assert!(b8.detail.contains("too dear") || b8.detail.contains("too cheap"));
}

/// The certificate must report the same measurement a direct read of the watch
/// gives, over the same window. Two paths to one number is how a certificate
/// starts telling a different story from the run it certifies.
#[test]
fn the_certificates_b8_line_is_the_measurement_taken_directly() {
    let cert = certify("lr_00", 1000, AgentArm::Legacy);
    let direct = measure_plain("lr_00", 1000, AgentArm::Legacy, 1.0);
    assert_eq!(battery(&cert, "B8").detail, direct.detail);
}

/// A world with no labour good has no numeraire, so there is nothing to
/// normalise against. It must say so rather than pass silently — an unscored
/// criterion reported as clean is how an instrument stops measuring without
/// anyone noticing. `big_region` and `supply_chain` both register
/// `labour_good: ""`.
#[test]
fn a_world_without_labour_is_reported_unscored_not_clean() {
    let cert = certify("big_region", 200, AgentArm::Legacy);
    let b8 = battery(&cert, "B8");
    assert!(b8.detail.starts_with("unscored"), "{}", b8.detail);
    assert!(b8.detail.contains("numeraire"), "{}", b8.detail);
}

/// The 225-year horizon. B8 must still be computable at 11,700 ticks, and the
/// tracer's single produced good — 1 labour makes 1 grain — has an implied
/// relative price of exactly 1.
#[test]
fn b8_holds_at_the_full_horizon() {
    let cert = certify("tracer_2r", 11_700, AgentArm::Legacy);
    let b8 = battery(&cert, "B8");
    assert!(b8.pass, "{}", b8.detail);
    assert!(b8.detail.contains("grain"), "{}", b8.detail);

    let s = loader::load(&scenario_dir("tracer_2r")).expect("loads");
    let labour = s.game_data.goods.iter().find(|g| g.name == "labour").unwrap().id;
    let grain = s.game_data.goods.iter().find(|g| g.name == "grain").unwrap().id;
    let v = labour_values(&s.game_data, labour);
    assert_eq!(v.get(grain), Value::Labour(1.0), "grow_grain is 1 labour -> 1 grain");
    // `gbp` is the currency and must be outside the accounting; the `dividend`
    // recipe turns gbp into gbp and would otherwise hand money a labour content.
    let gbp = s.game_data.goods.iter().find(|g| g.name == "gbp").unwrap().id;
    assert_eq!(v.get(gbp), Value::Currency);
}

// ── 6. The 2026-07-31 repair, falsification first ─────────────────────────────
//
// Adversarial review established seven defects in the subject of the sections
// above. Each is provoked here by a test that was written and *watched failing*
// against the pre-repair code before a line of the repair was written; the
// failure each one produced is recorded on the test.

/// Zero the named markets' posted supply and demand on every in-window tick
/// `keep` rejects, so those markets do not trade on those ticks.
///
/// Chosen over zeroing a *price* deliberately. A zero price poisons the price
/// rule and everything after it stops being `lr_00`; a market with nothing
/// posted on either side is the *ordinary* quiet-market state the engine
/// already has a convention for (`BalanceWatch::observe`), and it leaves every
/// other market's dynamics alone. What is under test is the reporter's
/// accounting, not the economy's response to a shock.
fn idle_markets_except(
    goods: &'static [&'static str],
    keep: impl Fn(u64) -> bool + 'static,
) -> impl FnMut(&mut rustyecon::state::SimState, &GameData) {
    move |state, gd| {
        if keep(state.tick) {
            return;
        }
        for name in goods {
            let g = gd.goods.iter().find(|g| &g.name == name).unwrap().id;
            for node in &gd.market_nodes {
                state.set_supply(node.id, g, 0.0);
                state.set_demand(node.id, g, 0.0);
            }
        }
    }
}

/// **Defect 1 (R5).** `GapReading::log_sd` was `f64::NAN` whenever a market
/// produced fewer than two samples, and `report` interpolated it straight into
/// the certificate's detail line. `certify::nan::scan` walks `SimState` only, so
/// nothing else in the run could ever have seen it.
///
/// lr_00's window is ticks 150..=1000, 851 of them. Every *scored* market is
/// idled on all but the last, so each good lands exactly one sample and Welford
/// has nothing to divide by.
///
/// SEEN FAILING before the repair with
/// `worst services 1.198x too dear at Manchester (ln +0.181 sd NaN, n=1);
///  services 1.198x too dear, flour 1.052x too cheap, wheat 1.036x too cheap`
/// — `pass = true`, a NaN on the certificate line.
#[test]
fn a_reading_with_one_sample_never_puts_a_nan_in_the_certificate() {
    let m = measure(
        "lr_00",
        1000,
        AgentArm::Legacy,
        1.0,
        |_| {},
        idle_markets_except(&["wheat", "flour", "services"], |t| t == 1000),
    );
    assert!(
        m.readings.iter().all(|r| r.samples == 1),
        "the setup is meant to leave exactly one sample per market: {:?}",
        m.readings.iter().map(|r| r.samples).collect::<Vec<_>>()
    );
    assert!(
        !m.detail.to_ascii_lowercase().contains("nan"),
        "R5: a metric that cannot be computed FAILS, and must never be printed \
         as a NaN on a passing line: {}",
        m.detail
    );
}

/// **Defect 4.** The same run discarded 850 of its 851 traded ticks and said
/// only `n=1`. A sample count with no denominator cannot be told apart from a
/// market that genuinely only traded once.
///
/// Here the numeraire is idled on every other in-window tick, so the scored
/// markets trade throughout and only half of those ticks can produce a reading.
/// The sample count needs its denominator, and the discards need a reason, on
/// the same line.
///
/// SEEN FAILING before the repair with `... sd 4.825, n=851)`: the pre-repair
/// line reported a bare sample count and, because it never looked at the
/// numeraire at all, counted every idled tick as a good one — defect 2 wearing
/// defect 4's clothes.
///
/// The first version of this provocation idled the numeraire on all but the
/// *last* tick. It was tightened after being watched failing, because the
/// repaired code correctly turns that run into a B8 *failure*: Birmingham's
/// services market trades on 62 of the 851 ticks and on none of the surviving
/// one, so it has no computable distance at all and the census line is never
/// reached. Half-idling keeps every market measurable while still throwing most
/// of the window away, which is the case this test is about.
#[test]
fn a_run_that_discards_most_of_its_ticks_says_how_many() {
    let m = measure(
        "lr_00",
        1000,
        AgentArm::Legacy,
        1.0,
        |_| {},
        idle_markets_except(&["labour"], |t| t % 2 == 0),
    );
    assert!(m.pass, "every market is still measurable here: {}", m.detail);
    assert!(
        m.readings.iter().all(|r| r.discarded() > 0 && r.numeraire_idle > 0),
        "the setup throws away about half of every market's ticks: {:?}",
        m.readings
            .iter()
            .map(|r| (r.samples, r.traded_ticks, r.numeraire_idle))
            .collect::<Vec<_>>()
    );
    let worst = m
        .readings
        .iter()
        .max_by(|a, b| a.log_gap.abs().partial_cmp(&b.log_gap.abs()).unwrap())
        .expect("readings");
    assert!(
        m.detail.contains(&worst.census()),
        "the line must carry the sample count WITH its denominator and where \
         the rest went — expected {:?} in: {}",
        worst.census(),
        m.detail
    );
}

/// **Defect 2.** `observe` gated on the *scored* good's supply and demand and
/// never asked whether the labour market had traded. A numeraire sitting at a
/// stale price makes every per-good reading wrong by the same stale factor, and
/// nothing in the certificate said so.
///
/// SEEN FAILING before the repair: 851 readings accumulated and B8 passed, on a
/// run whose labour market posted nothing on either side for the whole window.
#[test]
fn a_tick_whose_numeraire_did_not_trade_contributes_no_reading() {
    let m = measure(
        "lr_00",
        1000,
        AgentArm::Legacy,
        1.0,
        |_| {},
        idle_markets_except(&["labour"], |_| false),
    );
    assert!(
        m.readings.is_empty(),
        "a stale numeraire divides every reading, so no tick without a traded \
         numeraire may contribute one: got {} readings, {}",
        m.readings.len(),
        m.detail
    );
    assert!(!m.pass, "and B8 had markets to score and could not: {}", m.detail);
    assert!(
        m.detail.contains("numeraire"),
        "and must name the reason, so the fix is obvious: {}",
        m.detail
    );
}

/// **Defect 3.** Every reading divides by the same `p_labour`, so one wage error
/// is reported as N independent-looking per-good errors. lr_00/Manchester under
/// the legacy arm reads wheat 312.3x, flour 208.8x and services 43,415x "too
/// dear" — which is a *common factor* of about 1414x, a statement about the
/// wage, plus three small residuals around it.
///
/// SEEN FAILING before the repair: the detail line listed the three raw gaps and
/// said nothing about what they share.
#[test]
fn the_report_separates_a_wage_error_from_a_per_good_error() {
    let m = measure_plain("lr_00", 1000, AgentArm::Legacy, 1.0);
    assert!(
        m.detail.contains("common factor"),
        "N findings that are one finding must be reported as one: {}",
        m.detail
    );

    // The decomposition, recomputed here from the raw readings rather than
    // parsed out of the line, so the test cannot be satisfied by wording.
    let goods = ["wheat", "flour", "services"];
    let gaps: Vec<f64> = goods
        .iter()
        .map(|g| m.find("Manchester", g).log_gap)
        .collect();
    let common = gaps.iter().sum::<f64>() / gaps.len() as f64;
    let residuals: Vec<f64> = gaps.iter().map(|g| g - common).collect();

    assert!(
        common.exp() > 100.0,
        "Manchester's three goods share a large factor: {:?}",
        gaps.iter().map(|g| g.exp()).collect::<Vec<_>>()
    );
    assert!(
        residuals.iter().sum::<f64>().abs() < 1e-9,
        "a decomposition's residuals sum to zero: {residuals:?}"
    );
    let worst_raw = gaps.iter().fold(0.0f64, |a, b| a.max(b.abs()));
    let worst_res = residuals.iter().fold(0.0f64, |a, b| a.max(b.abs()));
    assert!(
        worst_res * 2.0 < worst_raw,
        "most of what the raw per-good numbers say is the shared factor, not \
         the good: worst raw ln {worst_raw:.3}, worst residual ln {worst_res:.3}"
    );

    // The sharpest part of the finding: at least one good whose raw reading
    // says "too dear" is, once the wage is accounted for, too CHEAP. A report
    // that only listed raw gaps would have sent a reader to the wrong market.
    assert!(
        gaps.iter().zip(&residuals).any(|(g, r)| g.signum() != r.signum()),
        "raw {:?} vs residual {:?}",
        gaps,
        residuals
    );
}

/// **Defect 5.** `solv_labour`'s registered equilibrium price is `p_grain =
/// p_labour`; the labour-value vector says 0.5, and the difference is the firm's
/// capacity rent, not an error. B8 called it "2.000x too dear". The benchmark's
/// assumption has to be on the line, or a reading gets mistaken for a defect —
/// this one already was.
///
/// SEEN FAILING before the repair: the line said only `worst grain 2.000x too
/// dear ...`, with no statement of what it assumed.
#[test]
fn the_line_states_the_benchmarks_assumptions() {
    let cert = certify("solv_labour", 1000, AgentArm::Kernel);
    let b8 = battery(&cert, "B8");
    let d = b8.detail.to_ascii_lowercase();
    for claim in ["zero-rent", "full-utilisation", "single-node", "rent"] {
        assert!(
            d.contains(claim),
            "the line must state that it assumes {claim}: {}",
            b8.detail
        );
    }
    // And the reading itself is unchanged and still 2.000x — the repair states
    // the assumption, it does not model the rent. A test that let the number
    // move would be hiding the very thing being disclosed.
    assert!(
        b8.detail.contains("grain 2.000x too dear"),
        "solv_labour's registered equilibrium is p_grain = p_labour and the \
         zero-rent benchmark says 0.5; the factor of two IS the rent: {}",
        b8.detail
    );
}

/// **Defect 6.** `flour_transport` (flour in, flour out) is skipped as a
/// pass-through, so its 0.1 labour is in no good's value anywhere and a
/// competitive importer that recovers its transport cost scores as an error.
/// The benchmark is a single-node one; the size of the resulting bias is
/// computable from the tape and must be stated rather than left as a surprise.
///
/// SEEN FAILING before the repair: nothing anywhere named `flour_transport` or
/// the 0.1 labour it adds.
#[test]
fn the_single_node_benchmark_names_the_transport_labour_it_omits() {
    let cert = certify("lr_00", 400, AgentArm::Legacy);
    let d = &battery(&cert, "B8").detail;
    assert!(
        d.contains("flour_transport"),
        "the omitted value has a name and a size; both belong on the line: {d}"
    );
}

/// **Defect 7.** A cost-reducing cycle in one corner of the graph marked *every*
/// good `NotConverged`, including labour — which is `Labour(1.0)` by definition
/// and cannot move, because the relaxation never writes to it.
///
/// SEEN FAILING before the repair, with labour reading `NotConverged`.
#[test]
fn a_stalled_cycle_marks_only_the_goods_that_are_still_moving() {
    let gd = tiny_tape(
        vec![good(0, "labour"), good(1, "wheat"), good(2, "a"), good(3, "b")],
        vec![
            recipe(0, "wheat_farm", &[(0, 0.3)], &[(1, 1.0)]),
            // The runaway, in its own corner of the graph: 1 b makes 2 a and
            // 1 a makes 2 b, so a lap quarters the cost and never settles.
            recipe(1, "seed_a", &[(0, 1.0)], &[(2, 1.0)]),
            recipe(2, "a_from_b", &[(3, 1.0)], &[(2, 2.0)]),
            recipe(3, "b_from_a", &[(2, 1.0)], &[(3, 2.0)]),
        ],
        None,
    );
    let v = labour_values(&gd, GoodId(0));
    assert_eq!(
        v.get(GoodId(0)),
        Value::Labour(1.0),
        "labour is the unit and the relaxation never writes to it"
    );
    assert_eq!(
        v.get(GoodId(1)),
        Value::Labour(0.3),
        "wheat is nowhere near the runaway and is not downstream of it"
    );
    assert_eq!(v.get(GoodId(2)), Value::NotConverged, "a is in the cycle");
    assert_eq!(v.get(GoodId(3)), Value::NotConverged, "b is in the cycle");
}

/// **The decision.** B8 cannot fail on being wrong, so it must not be *called*
/// something a PASS reads as a correctness claim.
///
/// SEEN FAILING before the repair: the label was `prices match technology`, and
/// every certificate in the corpus carried `B8 PASS prices match technology`
/// beside gaps of up to 4e9x.
#[test]
fn b8_is_not_labelled_as_a_correctness_verdict() {
    let cert = certify("lr_00", 200, AgentArm::Legacy);
    let b8 = battery(&cert, "B8");
    assert_ne!(
        b8.label, "prices match technology",
        "a PASS beside a 4e9x gap must not read as 'prices match technology'"
    );
    assert!(
        b8.label.contains("report"),
        "the label has to say it is a report, not a bar: {}",
        b8.label
    );
}

