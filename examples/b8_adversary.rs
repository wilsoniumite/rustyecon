//! ADVERSARIAL probe against B8 (`src/certify/technology.rs`). Review artefact,
//! not a shipped instrument. Every section tries to make B8 report something
//! that is not true, or to make the Leontief solve disagree with hand arithmetic.
//!
//!     cargo run --release --example b8_adversary -- <scenario-dir> [more dirs...]

use rustyecon::certify::technology::{labour_values, PriceGapWatch, Value};
use rustyecon::scenario::loader;
use rustyecon::state::SimState;
use rustyecon::types::ids::{GoodId, MarketNodeId};
use std::path::Path;

fn dump_values(dir: &str) {
    let sc = loader::load(Path::new(dir)).expect("load");
    let gd = &sc.game_data;
    let Some(cr) = sc.criteria.as_ref() else {
        println!("{dir}: no criteria");
        return;
    };
    let labour = gd.goods.iter().find(|g| g.name == cr.labour_good).map(|g| g.id);
    println!("--- {dir}  labour_good={:?}", cr.labour_good);
    let Some(l) = labour else {
        println!("    NO LABOUR GOOD -> B8 unscored");
        return;
    };
    let v = labour_values(gd, l);
    println!("    sweeps={}", v.sweeps());
    for g in &gd.goods {
        println!("    {:<20} {:?}", g.name, v.get(g.id));
    }
    // INDEPENDENT OPTIMALITY CHECK. Do not re-run the relaxation; check the
    // fixed-point condition the answer is supposed to satisfy. For every good,
    // the reported value must equal the cheapest recipe cost evaluated at the
    // reported values, and no recipe may beat it. This catches a relaxation that
    // stopped early or in the wrong order without sharing its algorithm.
    let currency: Vec<bool> = {
        let mut c = vec![false; gd.num_goods()];
        for n in &gd.market_nodes {
            if let Some(cg) = n.currency_good {
                c[cg.idx()] = true;
            }
        }
        c
    };
    for g in &gd.goods {
        let Value::Labour(reported) = v.get(g.id) else { continue };
        if g.id == l {
            continue;
        }
        let mut best = f64::INFINITY;
        for r in &gd.recipes {
            for out in &r.outputs {
                if out.good != g.id || out.qty_per_unit <= 0.0 {
                    continue;
                }
                if r.inputs.iter().any(|i| i.good == out.good) {
                    continue; // solver's pass-through rule
                }
                let mut tot = 0.0;
                let mut ok = true;
                for i in &r.inputs {
                    if currency[i.good.idx()] {
                        continue;
                    }
                    match v.get(i.good).relative_price() {
                        Some(x) => tot += x * i.qty_per_unit,
                        None if i.good == l => tot += i.qty_per_unit,
                        None => ok = false,
                    }
                }
                if ok {
                    best = best.min(tot / out.qty_per_unit);
                }
            }
        }
        let bad = !(best.is_finite() && (best - reported).abs() <= 1e-9 * reported.max(1.0));
        println!(
            "    CHECK {:<16} reported={reported:.12} cheapest_recipe={best:.12} {}",
            g.name,
            if bad { "<<< MISMATCH" } else { "ok" }
        );
    }
}

/// A minimal one-node, N-good SimState we can drive by hand.
fn synth(num_goods: usize, num_nodes: usize) -> SimState {
    SimState::new(num_goods, num_nodes)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    for d in &args {
        dump_values(d);
    }

    // ────────────────────────────────────────────────────────────────────────
    // ATTACK 1 — OSCILLATION BLINDNESS.
    // A market whose relative price alternates x100 dear / x100 cheap every
    // tick is never within 100x of the implied value. The mean of logs is 0.
    // What does B8 print, and is it selected as "worst"?
    // ────────────────────────────────────────────────────────────────────────
    {
        let sc = loader::load(Path::new("data/scenarios/lr_00")).expect("lr_00");
        let gd = &sc.game_data;
        let cr = sc.criteria.as_ref().unwrap();
        let mut w = PriceGapWatch::new(gd, cr);
        let idx = |n: &str| gd.goods.iter().find(|g| g.name == n).unwrap().id;
        let (labour, flour, wheat) = (idx("labour"), idx("flour"), idx("wheat"));
        let node = MarketNodeId(0);
        let mut st = synth(gd.num_goods(), gd.market_nodes.len());
        for t in 0..800 {
            st.set_price(node, labour, 1.0);
            // flour: implied 0.5. Alternate 50.0 and 0.005 -> 100x either way.
            st.set_price(node, flour, if t % 2 == 0 { 50.0 } else { 0.005 });
            // wheat: implied 0.3. A steady, mild 1.5x too dear.
            st.set_price(node, wheat, 0.45);
            for g in [labour, flour, wheat] {
                st.set_supply(node, g, 10.0);
                st.set_demand(node, g, 10.0);
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("\n[ATTACK 1 oscillation] pass={pass}\n  {detail}");
        for r in w.readings(gd) {
            println!(
                "  {:<12} implied={} ln={:+.4} {} {}",
                r.good, r.implied, r.log_gap, r.sd_phrase(), r.census()
            );
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // ATTACK 2 — n=1 SAMPLE, sd = NaN, PASS.
    // R5: "a NaN metric FAILS". Drive one market to have exactly one usable
    // tick out of many and see what reaches the certificate line.
    //
    // FOUND (pre-repair): `worst flour 1000.000x too dear (ln +6.908 sd NaN,
    // n=1)` with pass=true. REPAIRED 2026-07-31: `log_sd` is `Option<f64>` and
    // renders as an absence, the 850 discarded ticks are on the line, and
    // `PriceGapWatch::uncomputable` fails the report closed if a NaN ever
    // reaches it anyway.
    //
    // The labour market is posted here as well as priced. It was not, in the
    // review's version, and post-repair that alone made the attack fail on the
    // numeraire gate instead — which is ATTACK 3's finding, not this one's.
    // ────────────────────────────────────────────────────────────────────────
    {
        let sc = loader::load(Path::new("data/scenarios/lr_00")).expect("lr_00");
        let gd = &sc.game_data;
        let cr = sc.criteria.as_ref().unwrap();
        let mut w = PriceGapWatch::new(gd, cr);
        let idx = |n: &str| gd.goods.iter().find(|g| g.name == n).unwrap().id;
        let (labour, flour) = (idx("labour"), idx("flour"));
        let node = MarketNodeId(0);
        let mut st = synth(gd.num_goods(), gd.market_nodes.len());
        for t in 0..851 {
            st.set_price(node, labour, 1.0);
            // Price is zero (unpriced) on every tick but one; on that tick it
            // is 1000x too dear.
            st.set_price(node, flour, if t == 400 { 500.0 } else { 0.0 });
            for g in [labour, flour] {
                st.set_supply(node, g, 10.0);
                st.set_demand(node, g, 10.0);
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("\n[ATTACK 2 single sample] pass={pass}\n  {detail}");
        for r in w.readings(gd) {
            println!(
                "  {:<12} ln={:+.4} {} {}",
                r.good, r.log_gap, r.sd_phrase(), r.census()
            );
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // ATTACK 3 — STALE NUMERAIRE.
    // `observe` gates on the SCORED GOOD's supply/demand. It never asks whether
    // the labour market traded. If labour is dead all window, every other good
    // is still divided by labour's frozen price and reported as a measurement.
    //
    // FOUND (pre-repair): 851 confident readings, flour "100x too cheap", on a
    // world whose flour price is right in every real sense and whose wage
    // nobody had paid. REPAIRED 2026-07-31: pass=false, naming the idle
    // numeraire. Kept as the regression exhibit.
    // ────────────────────────────────────────────────────────────────────────
    {
        let sc = loader::load(Path::new("data/scenarios/lr_00")).expect("lr_00");
        let gd = &sc.game_data;
        let cr = sc.criteria.as_ref().unwrap();
        let mut w = PriceGapWatch::new(gd, cr);
        let idx = |n: &str| gd.goods.iter().find(|g| g.name == n).unwrap().id;
        let (labour, flour) = (idx("labour"), idx("flour"));
        let node = MarketNodeId(0);
        let mut st = synth(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..851 {
            // Labour market completely dead: nobody posts either side, price is
            // a stale genesis number that is 100x wrong.
            st.set_price(node, labour, 0.01);
            st.set_supply(node, labour, 0.0);
            st.set_demand(node, labour, 0.0);
            // Flour trades happily at a price that is CORRECT relative to a
            // labour price of 1.0 (0.5), i.e. correct in every real sense.
            st.set_price(node, flour, 0.5);
            st.set_supply(node, flour, 10.0);
            st.set_demand(node, flour, 10.0);
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("\n[ATTACK 3 stale numeraire] pass={pass}\n  {detail}");
    }

    // ────────────────────────────────────────────────────────────────────────
    // ATTACK 6 — ONE GLOBAL VALUE VECTOR FOR EVERY REGION.
    // `labour_values` is a function of the tape alone; there is no region in it.
    // `flour_transport` (1 flour + 0.1 labour -> 1 flour) is skipped as a
    // "pass-through", so the 0.1 labour a transported flour genuinely embodies
    // is in NO value anywhere. Give B8 a price vector that is CORRECT in every
    // region including the transport wedge, and see what it says.
    //
    // FOUND: "flour 1.200x too dear" on a perfectly competitive importer.
    // 2026-07-31: NOT repaired — per-node valuation was considered and
    // rejected, with reasons, on `LabourValues::pass_through`. What changed is
    // that the same line now *states* the benchmark is single-node and prints
    // this tape's whole bias, "flour_transport adds 0.100 labour ... 1.200x at
    // the far end" — the exact number the attack produces. Disclosed, bounded,
    // still there.
    // ────────────────────────────────────────────────────────────────────────
    {
        let sc = loader::load(Path::new("data/scenarios/lr_00")).expect("lr_00");
        let gd = &sc.game_data;
        let cr = sc.criteria.as_ref().unwrap();
        let mut w = PriceGapWatch::new(gd, cr);
        let idx = |n: &str| gd.goods.iter().find(|g| g.name == n).unwrap().id;
        let (labour, flour, wheat) = (idx("labour"), idx("flour"), idx("wheat"));
        let mut st = synth(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..851 {
            for (ri, r) in gd.regions.iter().enumerate() {
                let node = r.market_node;
                st.set_price(node, labour, 1.0);
                st.set_price(node, wheat, 0.3);
                // Region 0 mills locally at 0.5; regions 1 and 2 import, and a
                // competitive importer must recover the 0.1 labour of transport,
                // so 0.6 is the RIGHT price there, not an error.
                st.set_price(node, flour, if ri == 0 { 0.5 } else { 0.6 });
                for g in [labour, flour, wheat] {
                    st.set_supply(node, g, 10.0);
                    st.set_demand(node, g, 10.0);
                }
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("\n[ATTACK 6 transport wedge, prices CORRECT] pass={pass}\n  {detail}");
    }

    // ────────────────────────────────────────────────────────────────────────
    // ATTACK 4 — CONTRACTIVE CYCLE, REAL FIXED POINT, FALSE `NotConverged`?
    // The relaxation is geometric. A cycle with round-trip factor r needs
    // ln(1e-12)/ln(r) laps to get inside IMPROVE_REL. For r=0.9 that is ~262
    // laps against MAX_SWEEPS=256. Build one and see whether a perfectly
    // well-posed technology gets reported as a defect.
    //
    // MEASURED, and it is the reason MAX_SWEEPS's comment was rewritten
    // (2026-07-31, R14): f=0.89 lands in 255 sweeps, f=0.90 trips the cap — at
    // a graph with the unique positive fixed point iron = 19.0. Hitting the cap
    // is therefore a statement about the SOLVER, not about the graph, and
    // `Value::NotConverged` no longer claims otherwise. Still open: the cap is
    // where a well-posed technology becomes unscoreable, and nothing warns.
    // ────────────────────────────────────────────────────────────────────────
    {
        use rustyecon::state::game_data::{GameData, KernelParams, RegionDef, SupplyRule};
        use rustyecon::types::good::{GoodDef, MovementType, ShelfLife};
        use rustyecon::types::ids::{RecipeId, RegionId};
        use rustyecon::types::market_node::{MarketNodeDef, MarketTier};
        use rustyecon::types::recipe::{
            InputScaling, RecipeDef, RecipeInput, RecipeOutput, StrategyKind,
        };

        let good = |i: u32, n: &str| GoodDef {
            id: GoodId(i),
            name: n.into(),
            alpha: 0.1,
            shelf_life: ShelfLife::Indefinite,
            movement_type: MovementType::Physical,
            divisible: true,
            storage_cost_per_tick: 0.0,
        };
        let recipe = |i: u32, n: &str, ins: &[(u32, f64)], outs: &[(u32, f64)]| RecipeDef {
            id: RecipeId(i),
            name: n.into(),
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
        };
        let kp = KernelParams {
            s: 4,
            eta_up: 0.04,
            eta_dn: 0.05,
            dead: 0.05,
            b_out: 2.0,
            b_cash: 6.5,
            beta: 1.0,
            epsilon: 0.01,
            phi: 0.618_033_988_749_894_9,
            fill_alpha: 0.25,
            supply_rule: SupplyRule::Inelastic,
        };
        let mk = |recipes: Vec<RecipeDef>, goods: Vec<GoodDef>| GameData {
            goods,
            recipes,
            kernel: kp.clone(),
            need_categories: Vec::new(),
            wealth_levels: Vec::new(),
            market_nodes: vec![MarketNodeDef {
                id: MarketNodeId(0),
                tier: MarketTier::Regional,
                region: Some(RegionId(0)),
                currency_good: None,
            }],
            channels: Vec::new(),
            regions: vec![RegionDef {
                id: RegionId(0),
                name: "Only".into(),
                market_node: MarketNodeId(0),
                position: None,
            }],
        };

        // iron = 1 + f*tools ; tools = iron + 1  =>  round-trip factor f.
        // Fixed point: iron = (1+f)/(1-f), tools = iron + 1.
        for f in [0.5f64, 0.8, 0.85, 0.86, 0.87, 0.88, 0.89, 0.90, 0.95, 0.99] {
            let gd = mk(
                vec![
                    recipe(0, "dig_iron", &[(0, 1000.0)], &[(1, 1.0)]),
                    recipe(1, "smithy", &[(1, 1.0), (0, 1.0)], &[(2, 1.0)]),
                    recipe(2, "mine_iron", &[(2, f), (0, 1.0)], &[(1, 1.0)]),
                ],
                vec![good(0, "labour"), good(1, "iron"), good(2, "tools")],
            );
            let v = labour_values(&gd, GoodId(0));
            let exact_iron = ((1.0 + f) / (1.0 - f)).min(1000.0);
            let got = match v.get(GoodId(1)) {
                Value::Labour(x) => format!("{x:.12}"),
                other => format!("{other:?}"),
            };
            println!(
                "[ATTACK 4 cycle f={f:.2}] sweeps={:<4} iron={got}  exact={exact_iron:.12}",
                v.sweeps()
            );
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // ATTACK 5 — GENUINE SELF-CONSUMING PRODUCTION.
    // The "pass-through, not production" rule keys on the good appearing on
    // both sides at all, regardless of quantity. Seed grain (0.1 grain +
    // 1 labour -> 1 grain) is real production with a real fixed point
    // (lambda = 1/(1-0.1) = 1.1111). What does the solver say?
    // ────────────────────────────────────────────────────────────────────────
    {
        use rustyecon::state::game_data::{GameData, KernelParams, RegionDef, SupplyRule};
        use rustyecon::types::good::{GoodDef, MovementType, ShelfLife};
        use rustyecon::types::ids::{RecipeId, RegionId};
        use rustyecon::types::market_node::{MarketNodeDef, MarketTier};
        use rustyecon::types::recipe::{
            InputScaling, RecipeDef, RecipeInput, RecipeOutput, StrategyKind,
        };
        let kp = KernelParams {
            s: 4,
            eta_up: 0.04,
            eta_dn: 0.05,
            dead: 0.05,
            b_out: 2.0,
            b_cash: 6.5,
            beta: 1.0,
            epsilon: 0.01,
            phi: 0.618_033_988_749_894_9,
            fill_alpha: 0.25,
            supply_rule: SupplyRule::Inelastic,
        };
        let gd = GameData {
            goods: vec![
                GoodDef {
                    id: GoodId(0),
                    name: "labour".into(),
                    alpha: 0.1,
                    shelf_life: ShelfLife::Indefinite,
                    movement_type: MovementType::Physical,
                    divisible: true,
                    storage_cost_per_tick: 0.0,
                },
                GoodDef {
                    id: GoodId(1),
                    name: "grain".into(),
                    alpha: 0.1,
                    shelf_life: ShelfLife::Indefinite,
                    movement_type: MovementType::Physical,
                    divisible: true,
                    storage_cost_per_tick: 0.0,
                },
                GoodDef {
                    id: GoodId(2),
                    name: "bread".into(),
                    alpha: 0.1,
                    shelf_life: ShelfLife::Indefinite,
                    movement_type: MovementType::Physical,
                    divisible: true,
                    storage_cost_per_tick: 0.0,
                },
            ],
            recipes: vec![
                RecipeDef {
                    id: RecipeId(0),
                    name: "seed_grain".into(),
                    inputs: vec![
                        RecipeInput {
                            good: GoodId(1),
                            qty_per_unit: 0.1,
                            scaling: InputScaling::Variable,
                        },
                        RecipeInput {
                            good: GoodId(0),
                            qty_per_unit: 1.0,
                            scaling: InputScaling::Variable,
                        },
                    ],
                    outputs: vec![RecipeOutput { good: GoodId(1), qty_per_unit: 1.0 }],
                    component_reqs: Vec::new(),
                    reversible: false,
                    strategy: StrategyKind::CapacityControl,
                },
                RecipeDef {
                    id: RecipeId(1),
                    name: "bakery".into(),
                    inputs: vec![RecipeInput {
                        good: GoodId(1),
                        qty_per_unit: 1.0,
                        scaling: InputScaling::Variable,
                    }],
                    outputs: vec![RecipeOutput { good: GoodId(2), qty_per_unit: 1.0 }],
                    component_reqs: Vec::new(),
                    reversible: false,
                    strategy: StrategyKind::CapacityControl,
                },
            ],
            kernel: kp,
            need_categories: Vec::new(),
            wealth_levels: Vec::new(),
            market_nodes: vec![MarketNodeDef {
                id: MarketNodeId(0),
                tier: MarketTier::Regional,
                region: Some(RegionId(0)),
                currency_good: None,
            }],
            channels: Vec::new(),
            regions: vec![RegionDef {
                id: RegionId(0),
                name: "Only".into(),
                market_node: MarketNodeId(0),
                position: None,
            }],
        };
        let v = labour_values(&gd, GoodId(0));
        println!(
            "\n[ATTACK 5 seed grain] grain={:?} bread={:?}  (true lambda_grain = 1.11111, bread = same)",
            v.get(GoodId(1)),
            v.get(GoodId(2))
        );

        // ATTACK 7 — `Free` SHADOWS A REAL ROUTE, AND THE SHADOW WINS.
        // Currency legs are dropped from the input bill, so any recipe that
        // buys a good for money reads as costing nothing. "Cheapest route wins"
        // then makes that the good's value: 0 -> `Free` -> excluded from B8
        // entirely, even though a genuine labour route exists and is what the
        // world actually uses.
        let mut gd2 = gd.clone();
        gd2.goods.push(GoodDef {
            id: GoodId(3),
            name: "GBP".into(),
            alpha: 0.0,
            shelf_life: ShelfLife::Indefinite,
            movement_type: MovementType::Physical,
            divisible: true,
            storage_cost_per_tick: 0.0,
        });
        gd2.market_nodes[0].currency_good = Some(GoodId(3));
        // bread from labour: 2.0. bread from money: reads as 0.0.
        gd2.recipes.push(RecipeDef {
            id: RecipeId(2),
            name: "import_bread".into(),
            inputs: vec![RecipeInput {
                good: GoodId(3),
                qty_per_unit: 7.0,
                scaling: InputScaling::Variable,
            }],
            outputs: vec![RecipeOutput { good: GoodId(2), qty_per_unit: 1.0 }],
            component_reqs: Vec::new(),
            reversible: false,
            strategy: StrategyKind::CapacityControl,
        });
        // Give bread a plain labour route too, so a real value exists.
        gd2.recipes.push(RecipeDef {
            id: RecipeId(3),
            name: "bakery_direct".into(),
            inputs: vec![RecipeInput {
                good: GoodId(0),
                qty_per_unit: 2.0,
                scaling: InputScaling::Variable,
            }],
            outputs: vec![RecipeOutput { good: GoodId(2), qty_per_unit: 1.0 }],
            component_reqs: Vec::new(),
            reversible: false,
            strategy: StrategyKind::CapacityControl,
        });
        let v2 = labour_values(&gd2, GoodId(0));
        println!(
            "[ATTACK 7 currency shadow] bread={:?}  (a real 2.0-labour route exists and is ignored)",
            v2.get(GoodId(2))
        );
    }
}
