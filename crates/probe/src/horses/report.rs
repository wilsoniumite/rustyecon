//! The stocks probe's reports, which the `horses` binary writes for P2.2a's instances and the
//! loop step's alike (HORSES-RULES §5; LOOPS-RULES §8.5): the summary line of a run, in
//! `summary.tsv`'s columns, and HORSES-SPEC §7.11's statistics in long form, in `stats.tsv`'s.
//!
//! P2.2a's 50 columns keep their place and meaning; the loop step appends nine (decision 292),
//! which a P2.2a instance writes as `-`.

use super::harness::Stats;
use crate::harness::{Class, Stop, Summary};

/// `summary.tsv`'s columns: P2.2a's 50, then the loop step's nine (LOOPS-RULES §8.5).
pub const SUMMARY: [&str; 59] = [
    "run",
    "class",
    "d0",
    "E1",
    "E2",
    "E3",
    "E4",
    "kappa",
    "max_W",
    "last",
    "in_tol_from",
    "dead",
    "dead_W",
    "dead_F",
    "band_v",
    "r_end",
    "peak_dhat",
    "peak_tick",
    "worst_fill",
    "low_baskets",
    "low_baskets_tick",
    "no_basket_ticks",
    "transfer_short",
    "transfer_short_ticks",
    "depth_eq",
    "depth_trough_y0",
    "depth_trough_y1",
    "lowest_market",
    "peak_dhat_ex_horse",
    "heads_low",
    "heads_high",
    "in_5pct_from",
    "paper_ticks",
    "quasi_3",
    "quasi_1_over_delta",
    "no_order_ticks",
    "idle_ticks",
    "pk_low",
    "finished_high",
    "utilisation_low",
    "hour_over_o_low",
    "investment_low",
    "investment_high",
    "nine_in_tol_from",
    "mode_a",
    "stop",
    "why",
    "withheld",
    "switches",
    "markup_low",
    "dead_by",
    "bound_ticks",
    "bound_peak",
    "hours_tasks_low",
    "hours_fodder_low",
    "pk_high",
    "plants_in_5pct_from",
    "plants_low",
    "plants_high",
];

/// A number as the reports write it.
pub fn g(x: f64) -> String {
    format!("{x:.6e}")
}

/// A tick, or `-`.
pub fn opt(x: Option<u64>) -> String {
    x.map_or("-".into(), |t| t.to_string())
}

/// A run name as a file name.
pub fn file_name(run: &str) -> String {
    run.chars()
        .map(|c| match c {
            '*' => 'x',
            '/' => 'd',
            '(' | ')' | ',' | '=' | '@' | '+' | '[' | ']' => '_',
            c => c,
        })
        .collect()
}

/// P2.2a's 50 summary columns of a run: its name, its class and numbers (PROBE-SPEC §4.5), and
/// §7.11's transient and stock statistics, with the markets in the harness's order.
pub fn summary_fields(
    name: &str,
    s: &Summary,
    st: &Stats,
    markets: &[String],
    hold_failure: &Option<String>,
    stop: &Stop,
) -> Vec<String> {
    let k = &st.stock;
    let mut v = vec![name.to_string(), s.class.name().to_string(), g(s.d0)];
    v.extend(s.envelope.iter().map(|&x| g(x)));
    v.push(g(s.kappa));
    v.push(g(s.max_w));
    v.push(g(s.last));
    v.push(opt(s.in_tol_from));
    v.extend(s.dead.iter().map(|d| d.to_string()));
    v.push(g(s.band[0]));
    v.push(g(s.r_end));
    v.push(g(st.peak.0));
    v.push(st.peak.1.to_string());
    v.push(g(st.worst_fill));
    v.push(g(st.baskets.0));
    v.push(st.baskets.1.to_string());
    v.push(st.baskets.2.to_string());
    v.push(g(st.transfer.0));
    v.push(st.transfer.1.to_string());
    match st.depth {
        Some((a, b, c)) => v.extend([g(a), g(b), g(c)]),
        None => v.extend(["-".into(), "-".into(), "-".into()]),
    }
    let low = st
        .trough
        .iter()
        .enumerate()
        .min_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
        .map_or("-".to_string(), |(m, t)| {
            format!("{}:{}", markets[m], g(t.0))
        });
    v.push(low);
    v.push(g(k.peak_ex.0));
    v.push(g(k.heads_range.0));
    v.push(g(k.heads_range.1));
    v.push(opt(k.in_five));
    v.push(g(k.paper));
    v.push(g(k.quasi.0));
    v.push(g(k.quasi.1));
    v.push(k.no_order.to_string());
    v.push(k.idle.to_string());
    v.push(g(k.pk_low));
    v.push(g(k.finished_high));
    v.push(g(k.utilisation_low));
    v.push(g(k.hour_over_o_low));
    v.push(g(k.investment.0));
    v.push(g(k.investment.1));
    v.push(opt(k.nine_in_tol));
    v.push(match (hold_failure, name) {
        (None, "hold") if s.class != Class::Error => "PASS".into(),
        (Some(f), "hold") => format!("FAIL: {f}"),
        _ => "-".into(),
    });
    v.push(match stop {
        Stop::Ran => "ran".into(),
        Stop::Runaway(_) => "runaway".into(),
        Stop::Error(_) => "error".into(),
    });
    v.push(s.why.clone());
    v.push(k.withheld.to_string());
    v.push(k.switches.to_string());
    v.push(g(k.markup_low));
    v
}

/// §7.11's statistics in long form, (statistic, where, value), with the markets, the desks'
/// outputs and the basket items in the harness's order, and the engine's market names for the
/// rationing lines.
pub fn stats_fields(
    st: &Stats,
    markets: &[String],
    desks: &[String],
    item_names: &[String],
    engine_markets: &[String],
) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut put = |stat: &str, at: &str, value: String| {
        out.push((stat.to_string(), at.to_string(), value));
    };
    put("peak.dhat", "-", g(st.peak.0));
    put("peak.tick", "-", st.peak.1.to_string());
    for (m, name) in markets.iter().enumerate() {
        let d = st.dead_market[m];
        put("dead.no_trade", name, d[0].to_string());
        put("dead.below_floor", name, d[1].to_string());
        put("dead.no_supply", name, d[2].to_string());
        put("dead.no_demand", name, d[3].to_string());
        let (low, at, end) = st.trough[m];
        put("trough.cleared", name, g(low));
        put("trough.tick", name, at.to_string());
        put("trough.end", name, g(end));
        let (sp, su) = st.spoilage[m];
        put("spoiled.total", name, g(sp));
        put(
            "spoiled.share",
            name,
            g(if su > 0.0 { sp / su } else { f64::NAN }),
        );
    }
    for (d, name) in desks.iter().enumerate() {
        let (low, at) = st.output_trough[d];
        put("trough.output", name, g(low));
        put("trough.output_tick", name, at.to_string());
    }
    put("baskets.trough", "-", g(st.baskets.0));
    put("baskets.trough_tick", "-", st.baskets.1.to_string());
    put("baskets.none_ticks", "-", st.baskets.2.to_string());
    for (i, name) in item_names.iter().enumerate() {
        put("item.trough", name, g(st.item[i].0));
        put("item.none_ticks", name, st.item[i].1.to_string());
        put("binding.ticks", name, st.binding.0[i].to_string());
    }
    put("binding.short_ticks", "-", st.binding.1.to_string());
    for ((m, class, side), r) in &st.rationing {
        let market = engine_markets.get(*m).cloned().unwrap_or_default();
        let at = format!("{market}/{class}/{side}");
        put("ration.worst", &at, g(r.worst));
        put("ration.rationed_ticks", &at, r.ticks.to_string());
        put("ration.budget_short", &at, g(r.budget));
        put("ration.market_short", &at, g(r.market));
    }
    put("transfer.short", "provider", g(st.transfer.0));
    put(
        "transfer.short_ticks",
        "provider",
        st.transfer.1.to_string(),
    );
    if let Some((a, b, c)) = st.depth {
        put("depth.equilibrium", "-", g(a));
        put("depth.trough_y0", "-", g(b));
        put("depth.trough_y1", "-", g(c));
    }
    let k = &st.stock;
    put("stock.peak_dhat_ex_horse", "-", g(k.peak_ex.0));
    put("stock.peak_ex_tick", "-", k.peak_ex.1.to_string());
    put("stock.heads_low", "-", g(k.heads_range.0));
    put("stock.heads_high", "-", g(k.heads_range.1));
    put("stock.in_5pct_from", "-", opt(k.in_five));
    put("stock.paper_ticks", "-", g(k.paper));
    put("stock.quasi_rent_3", "-", g(k.quasi.0));
    put("stock.quasi_rent_1_over_delta", "-", g(k.quasi.1));
    put("glut.no_order_ticks", "-", k.no_order.to_string());
    put("glut.idle_ticks", "-", k.idle.to_string());
    put("glut.pk_low", "-", g(k.pk_low));
    put("glut.finished_high", "-", g(k.finished_high));
    put("glut.utilisation_low", "-", g(k.utilisation_low));
    put("glut.hour_over_o_low", "-", g(k.hour_over_o_low));
    put("investment.low", "-", g(k.investment.0));
    put("investment.high", "-", g(k.investment.1));
    put("nine.in_tol_from", "-", opt(k.nine_in_tol));
    put("idle.withheld", "-", k.withheld.to_string());
    put("idle.switches", "-", k.switches.to_string());
    put("idle.markup_low", "-", g(k.markup_low));
    out
}
