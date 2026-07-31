//! SECOND adversarial pass against B8 (`src/certify/technology.rs`), written to
//! REFUTE the 2026-07-31 repair. Review artefact, not a shipped instrument.
//!
//!     cargo run --release --example b8_refute_pass2
//!
//! Everything here drives `PriceGapWatch` directly with a hand-built `SimState`,
//! so every number printed is a measurement of the shipped code, not a
//! re-derivation of what it is supposed to do.

use rustyecon::certify::technology::{labour_values, PriceGapWatch, Value};
use rustyecon::scenario::loader;
use rustyecon::state::{GameData, SimState};
use rustyecon::types::ids::{GoodId, MarketNodeId};
use std::path::Path;

fn lr00() -> rustyecon::scenario::loader::Scenario {
    loader::load(Path::new("data/scenarios/lr_00")).expect("lr_00")
}

fn gid(gd: &GameData, n: &str) -> GoodId {
    gd.goods.iter().find(|g| g.name == n).unwrap().id
}

fn rule(t: &str) {
    println!("\n{}\n{t}\n{}", "=".repeat(78), "-".repeat(78));
}

fn main() {
    let sc = lr00();
    let gd = &sc.game_data;
    let cr = sc.criteria.as_ref().unwrap().clone();
    let labour = gid(gd, "labour");
    let v = labour_values(gd, labour);
    let implied = |g: GoodId| v.get(g).relative_price();

    println!("lr_00 implied labour values:");
    for g in &gd.goods {
        println!("   {:<10} {:?}", g.name, v.get(g.id));
    }
    println!("regions -> nodes:");
    for r in &gd.regions {
        println!("   {:<12} node {:?}", r.name, r.market_node);
    }

    let wheat = gid(gd, "wheat");
    let flour = gid(gd, "flour");
    let services = gid(gd, "services");
    let scored = [wheat, flour, services];
    let node = MarketNodeId(0); // Manchester

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK A — THE NUMERAIRE GATE IS "POSTED", NOT "PAID".
    //
    // `observe` gates on `supply(labour) > 0 || demand(labour) > 0`. In
    // `systems/clearing/mod.rs` those two arrays are the summed SELL and BUY
    // order quantities; the quantity that actually changes hands is
    // min(supply, demand). So a labour market with 10 units offered and ZERO
    // bids passes the gate on every tick while no one is hired and no one pays
    // the posted wage — precisely the state the gate's doc comment says it
    // exists to exclude.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK A — labour offered, never bid for: is the gate load-bearing?");
    {
        let mut w = PriceGapWatch::new(gd, &cr);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..851 {
            st.set_price(node, labour, 1.0);
            // Every scored good posted at 1000x its implied relative price.
            for g in scored {
                st.set_price(node, g, implied(g).unwrap() * 1000.0);
                st.set_supply(node, g, 5.0);
                st.set_demand(node, g, 5.0);
            }
            // THE WAGE NOBODY PAYS: sellers only. Traded quantity = 0.
            st.set_supply(node, labour, 10.0);
            st.set_demand(node, labour, 0.0);
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        let rs = w.readings(gd);
        println!("pass = {pass}");
        println!("readings = {}", rs.len());
        for r in &rs {
            println!(
                "   {}/{} gap x{:.1}  {}",
                r.region,
                r.good,
                r.log_gap.exp(),
                r.census()
            );
        }
        println!("detail: {detail}");
        println!(
            "\nVERDICT: gate {} on a market where the traded quantity is min(10,0) = 0.",
            if rs.is_empty() { "HELD" } else { "DID NOT FIRE" }
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK A2 — the same hole on the NUMERATOR. A scored good with sellers
    // and no buyers is "traded" too, so B8 scores prices at which nothing
    // changed hands on both legs of the ratio.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK A2 — scored goods with sellers and no buyers");
    {
        let mut w = PriceGapWatch::new(gd, &cr);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..851 {
            st.set_price(node, labour, 1.0);
            st.set_supply(node, labour, 3.0);
            st.set_demand(node, labour, 3.0);
            for g in scored {
                st.set_price(node, g, implied(g).unwrap() * 42.0);
                st.set_supply(node, g, 7.0);
                st.set_demand(node, g, 0.0); // nobody buys
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("pass = {pass}, readings = {}", w.readings(gd).len());
        println!("detail: {detail}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK B — DOES THE DECOMPOSITION ATTRIBUTE CORRECTLY?
    //   B1: a KNOWN PURE WAGE error, every good exactly at its implied value
    //       and the wage wrong by 8x.  Correct answer: common = 8x,
    //       residuals all exactly 1.000x.
    //   B2: a KNOWN PURE SINGLE-GOOD error, flour 8x too dear and the wage and
    //       every other good exactly right. Correct answer: common = 1.000x
    //       ("the wage is fine"), residual flour = 8x, others 1.000x.
    // ───────────────────────────────────────────────────────────────────────
    for (tag, wage_err, flour_err) in [
        ("B1  PURE WAGE ERROR (wage 8x too low, every good priced right)", 8.0, 1.0),
        ("B2  PURE SINGLE-GOOD ERROR (flour 8x too dear, wage right)", 1.0, 8.0),
    ] {
        rule(tag);
        let mut w = PriceGapWatch::new(gd, &cr);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..100 {
            st.set_price(node, labour, 1.0);
            st.set_supply(node, labour, 5.0);
            st.set_demand(node, labour, 5.0);
            for g in scored {
                let own = if g == flour { flour_err } else { 1.0 };
                st.set_price(node, g, implied(g).unwrap() * wage_err * own);
                st.set_supply(node, g, 5.0);
                st.set_demand(node, g, 5.0);
            }
            w.observe(&st);
        }
        for d in w.decompositions(gd) {
            if d.region != "Manchester" {
                continue;
            }
            println!("  common factor = {:.6}x   ({})", d.common_log.exp(), d.common_phrase());
            for (g, r) in &d.residuals {
                println!("     residual {:<10} {:.6}x", g, r.exp());
            }
        }
        let (_, detail) = w.report(gd);
        println!("  line: {detail}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK C — A TYPO IN criteria.labour_good. `PriceGapWatch::new` resolves
    // the numeraire by NAME and, finding nothing, reports the whole scenario
    // "unscored" — with pass = TRUE — on a world that has a full labour market.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK C — criteria.labour_good misspelled: fail-open?");
    {
        let mut typo = cr.clone();
        typo.labour_good = "labor".into(); // one letter
        let mut w = PriceGapWatch::new(gd, &typo);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..851 {
            st.set_price(node, labour, 1.0);
            st.set_supply(node, labour, 5.0);
            st.set_demand(node, labour, 5.0);
            for g in scored {
                st.set_price(node, g, implied(g).unwrap() * 1e9);
                st.set_supply(node, g, 5.0);
                st.set_demand(node, g, 5.0);
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("pass = {pass}   readings = {}", w.readings(gd).len());
        println!("detail: {detail}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK D — CAN AN INFINITY STILL REACH A PASSING LINE?
    // `uncomputable` sweeps only `readings`. The line ALSO prints
    // `PassThrough::bias()`, which is (base + added)/base and is swept by
    // nothing. Drive `base` toward the denormal floor with a huge output
    // quantity and see what the certificate prints.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK D — an un-swept statistic on the line: PassThrough::bias()");
    {
        let mut gd2 = gd.clone();
        // wheat_farm: make it yield an astronomically large batch, so wheat's
        // labour value underflows toward the denormal floor.
        for r in gd2.recipes.iter_mut() {
            if r.name == "wheat_farm" {
                for o in r.outputs.iter_mut() {
                    o.qty_per_unit = 1e307;
                }
            }
            // and give wheat a pass-through haul that adds real labour
            if r.name == "flour_transport" {
                r.inputs.clear();
                r.inputs.push(rustyecon::types::recipe::RecipeInput {
                    good: wheat,
                    qty_per_unit: 1.0,
                    scaling: rustyecon::types::recipe::InputScaling::Variable,
                });
                r.inputs.push(rustyecon::types::recipe::RecipeInput {
                    good: labour,
                    qty_per_unit: 1.0,
                    scaling: rustyecon::types::recipe::InputScaling::Variable,
                });
                r.outputs.clear();
                r.outputs.push(rustyecon::types::recipe::RecipeOutput {
                    good: wheat,
                    qty_per_unit: 1.0,
                });
            }
        }
        let v2 = labour_values(&gd2, labour);
        println!("wheat value now {:?}", v2.get(wheat));
        for h in v2.pass_through(&gd2) {
            println!("   pass-through: {}   bias={:e}", h.phrase(), h.bias());
        }
        let mut w = PriceGapWatch::new(&gd2, &cr);
        let mut st = SimState::new(gd2.num_goods(), gd2.market_nodes.len());
        for _ in 0..10 {
            st.set_price(node, labour, 1.0);
            st.set_supply(node, labour, 5.0);
            st.set_demand(node, labour, 5.0);
            for g in scored {
                if let Some(i) = v2.get(g).relative_price() {
                    st.set_price(node, g, i * 2.0);
                    st.set_supply(node, g, 5.0);
                    st.set_demand(node, g, 5.0);
                }
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(&gd2);
        println!("pass = {pass}");
        println!("detail: {detail}");
        println!(
            "contains 'inf' = {}",
            detail.to_ascii_lowercase().contains("inf")
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK E — SAMPLE LOSS OUTSIDE THE WORST READING. `census()` is printed
    // for exactly one reading (`worst`). Make a DIFFERENT market throw away
    // almost everything and check whether the line says so.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK E — a non-worst market that discards 99% of its ticks");
    {
        let mut w = PriceGapWatch::new(gd, &cr);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for t in 0..851u64 {
            st.set_price(node, labour, 1.0);
            st.set_supply(node, labour, 5.0);
            st.set_demand(node, labour, 5.0);
            // wheat: enormous, steady error -> it will be "worst".
            st.set_price(node, wheat, implied(wheat).unwrap() * 5000.0);
            st.set_supply(node, wheat, 5.0);
            st.set_demand(node, wheat, 5.0);
            // flour: trades every tick, but its PRICE is unusable (zero) on all
            // but every hundredth tick, so 99% of its samples vanish.
            st.set_supply(node, flour, 5.0);
            st.set_demand(node, flour, 5.0);
            st.set_price(
                node,
                flour,
                if t % 100 == 0 { implied(flour).unwrap() * 3.0 } else { 0.0 },
            );
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("pass = {pass}");
        for r in w.readings(gd) {
            println!("   {}/{} {}", r.region, r.good, r.census());
        }
        println!("detail: {detail}");
        println!(
            "\nline mentions flour's discards = {}",
            detail.contains("flour") && detail.contains("unpriced")
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK F — `Value::Free` and `Value::NotConverged` on a TRADED good are
    // supposed to FAIL. Confirm, then check the one that is not covered: a
    // scored good that trades only on ticks when the numeraire is idle.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK F — a market that only ever trades while the numeraire is idle");
    {
        let mut w = PriceGapWatch::new(gd, &cr);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for t in 0..851u64 {
            st.set_price(node, labour, 1.0);
            let live = t % 2 == 0;
            st.set_supply(node, labour, if live { 5.0 } else { 0.0 });
            st.set_demand(node, labour, if live { 5.0 } else { 0.0 });
            // wheat trades only on the ticks the numeraire is DEAD.
            st.set_price(node, wheat, implied(wheat).unwrap() * 7.0);
            st.set_supply(node, wheat, if live { 0.0 } else { 5.0 });
            st.set_demand(node, wheat, if live { 0.0 } else { 5.0 });
            // flour trades on the live ticks, so the report has something.
            st.set_price(node, flour, implied(flour).unwrap() * 1.1);
            st.set_supply(node, flour, if live { 5.0 } else { 0.0 });
            st.set_demand(node, flour, if live { 5.0 } else { 0.0 });
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("pass = {pass}");
        println!("detail: {detail}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK D2 — same lever, pushed past the overflow edge. If bias() can be
    // made INFINITE, an infinity reaches a PASSING certificate line through a
    // statistic `uncomputable` never looks at.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK D2 — PassThrough::bias() driven to infinity");
    {
        let mut gd2 = gd.clone();
        for r in gd2.recipes.iter_mut() {
            if r.name == "wheat_farm" {
                for o in r.outputs.iter_mut() {
                    o.qty_per_unit = 1e308;
                }
            }
            if r.name == "flour_transport" {
                r.inputs.clear();
                r.inputs.push(rustyecon::types::recipe::RecipeInput {
                    good: wheat,
                    qty_per_unit: 1.0,
                    scaling: rustyecon::types::recipe::InputScaling::Variable,
                });
                r.inputs.push(rustyecon::types::recipe::RecipeInput {
                    good: labour,
                    qty_per_unit: 1.0,
                    scaling: rustyecon::types::recipe::InputScaling::Variable,
                });
                r.outputs.clear();
                r.outputs.push(rustyecon::types::recipe::RecipeOutput {
                    good: wheat,
                    qty_per_unit: 1.0,
                });
            }
        }
        let v2 = labour_values(&gd2, labour);
        println!("wheat value now {:?}", v2.get(wheat));
        for h in v2.pass_through(&gd2) {
            println!("   bias = {:e}  finite={}", h.bias(), h.bias().is_finite());
        }
        let mut w = PriceGapWatch::new(&gd2, &cr);
        let mut st = SimState::new(gd2.num_goods(), gd2.market_nodes.len());
        for _ in 0..10 {
            st.set_price(node, labour, 1.0);
            st.set_supply(node, labour, 5.0);
            st.set_demand(node, labour, 5.0);
            for g in [flour, services] {
                st.set_price(node, g, v2.get(g).relative_price().unwrap() * 2.0);
                st.set_supply(node, g, 5.0);
                st.set_demand(node, g, 5.0);
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(&gd2);
        println!("pass = {pass}");
        println!("detail: {detail}");
        let low = detail.to_ascii_lowercase();
        println!(
            "\nline carries 'inf' = {}   'nan' = {}",
            low.contains("inf"),
            low.contains("nan")
        );
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK H — a world in which NOTHING trades for the whole window.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK H — nothing trades at all for 851 ticks");
    {
        let mut w = PriceGapWatch::new(gd, &cr);
        let st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for _ in 0..851 {
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("pass = {pass}");
        println!("detail: {detail}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK I — the numeraire trades on exactly ONE tick of 851, at a price
    // that is 1000x wrong on that tick and stale for the other 850.
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK I — a wage paid once, on one tick out of 851");
    {
        let mut w = PriceGapWatch::new(gd, &cr);
        let mut st = SimState::new(gd.num_goods(), gd.market_nodes.len());
        for t in 0..851u64 {
            let live = t == 400;
            st.set_price(node, labour, if live { 0.001 } else { 1.0 });
            st.set_supply(node, labour, if live { 5.0 } else { 0.0 });
            st.set_demand(node, labour, if live { 5.0 } else { 0.0 });
            for g in scored {
                st.set_price(node, g, implied(g).unwrap());
                st.set_supply(node, g, 5.0);
                st.set_demand(node, g, 5.0);
            }
            w.observe(&st);
        }
        let (pass, detail) = w.report(gd);
        println!("pass = {pass}");
        for r in w.readings(gd) {
            println!("   {}/{} gap x{:.1} {} {}", r.region, r.good, r.log_gap.exp(), r.sd_phrase(), r.census());
        }
        println!("detail: {detail}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // ATTACK G — is `Value::Free` reachable through a normal-looking tape and
    // does it fail? (the report claims it does; check it, and check the
    // currency-route hole ATTACK 7 of the first pass left open.)
    // ───────────────────────────────────────────────────────────────────────
    rule("ATTACK G — state of Free / Unreachable on the shipped corpus");
    for dir in [
        "data/scenarios/big_region",
        "data/scenarios/supply_chain",
        "data/scenarios/tracer_2r",
        "data/scenarios/multi_region",
    ] {
        let s = loader::load(Path::new(dir)).expect(dir);
        let g2 = &s.game_data;
        let c2 = s.criteria.as_ref().unwrap();
        let l = g2.goods.iter().find(|g| g.name == c2.labour_good).map(|g| g.id);
        print!("{dir}: labour_good={:?} -> ", c2.labour_good);
        match l {
            None => {
                let w = PriceGapWatch::new(g2, c2);
                let (pass, detail) = w.report(g2);
                println!("NO NUMERAIRE, report pass={pass}: {detail}");
            }
            Some(l) => {
                let vv = labour_values(g2, l);
                let bad: Vec<String> = g2
                    .goods
                    .iter()
                    .filter(|g| {
                        !matches!(vv.get(g.id), Value::Labour(_) | Value::Currency)
                    })
                    .map(|g| format!("{}={:?}", g.name, vv.get(g.id)))
                    .collect();
                println!("unresolved: {bad:?}");
            }
        }
    }
}
