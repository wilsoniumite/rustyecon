//! The stocks probe's command line (HORSES-SPEC §7; docs/probe/HORSES-RULES.md §5).
//!
//! ```text
//! horses run NAME...     [options]  run named perturbations (see `probe::horses::perturb`)
//! horses family FAMILY   [options]  run a family: battery, tier3s, stocks
//! horses list FAMILY     [options]  print a family's run names (the battery with its tier)
//! horses tape [NAME]     [options]  print the tape a run is made from
//! horses kick NAME...    [options]  the kick set at the end of each named run (§7.5)
//! horses slowest NAME    [options]  the kick set's envelope and its slowest mode, g (§7.4)
//! horses elasticity      [options]  the one-tick elasticity probe and L's second term (§7.8)
//! horses openloop        [options]  the open-loop probe, prices frozen (§7.8)
//! horses point           [options]  the oracle's point, at the base and each cost target
//!
//! setup options (probe::horses::cli):
//!   --inst ID              h1-h4, f1-f10, r1a, p7 or p8 (default h1)
//!   --tpy N                ticks a year (default 52)
//!   --dials c2g|c2g13|c2   a registered dial set (default the instance's)
//!   --set KEY=VALUE        set a dial; rate.*, buffer.* and adjust.* scale a family, tilt.* sets
//!   --assign planned|expost, --order target|held, --cover yes|none
//!   --one-sided saturate|hold
//!   --reserve PSI          the maker's reservation (L0.4; IDLE-SPEC): it offers no finished heads
//!                          while its net markup is below PSI; absent, the tape is P2.2a's
//! run options:
//!   --ticks L              the scored length (default 20000; a dated shock adds L/4 before it)
//!   --csv DIR              write DIR/<run>.csv, one row per tick, DIR/summary.tsv, DIR/stats.tsv
//!   --every K              write every K-th tick to the CSV (and the last)
//!   --jobs J               run J runs at once (default 1)
//!   --horizon H            the kick's horizon (default: the scored length)
//!   --h X                  the probes' step in log price (default 0.01)
//! ```
//!
//! Each run prints one summary line: its class (PROBE-SPEC §4.5) and the reported numbers, with
//! §7.11's transient and stock statistics in long form in `stats.tsv`. The last three columns of
//! `summary.tsv` are the idle market's (L0.4): the scored ticks the maker withheld, its switches
//! between withholding and offering, and its lowest net markup.

use probe::harness::{Class, Summary};
use probe::horses::cli::parse;
use probe::horses::harness::{csv_header, items, outputs, run, Record};
use probe::horses::kick::{kick_set, slowest_mode, three_t6};
use probe::horses::perturb::{battery, family, Perturbation};
use probe::horses::probes::{elasticity, open_loop};
use probe::horses::setup::{tape_ron, Setup};
use probe::protocol::RUN_TICKS;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

struct Options {
    ticks: u64,
    setup: Setup,
    csv: Option<PathBuf>,
    every: u64,
    jobs: usize,
    horizon: Option<u64>,
    h: f64,
}

fn options(args: &[String]) -> Result<(Vec<String>, Options), String> {
    let a = parse(
        args,
        &["--ticks", "--csv", "--every", "--jobs", "--horizon", "--h"],
    )?;
    let mut o = Options {
        ticks: RUN_TICKS,
        setup: a.setup,
        csv: None,
        every: 1,
        jobs: 1,
        horizon: None,
        h: 0.01,
    };
    let mut rest = Vec::new();
    let mut it = a.rest.iter();
    while let Some(x) = it.next() {
        let mut val = || it.next().ok_or(format!("{x} takes a value"));
        match x.as_str() {
            "--ticks" => o.ticks = val()?.parse().map_err(|_| "--ticks N")?,
            "--csv" => o.csv = Some(PathBuf::from(val()?)),
            "--every" => o.every = val()?.parse().map_err(|_| "--every K")?,
            "--jobs" => o.jobs = val()?.parse().map_err(|_| "--jobs J")?,
            "--horizon" => o.horizon = Some(val()?.parse().map_err(|_| "--horizon H")?),
            "--h" => o.h = val()?.parse().map_err(|_| "--h X")?,
            y if y.starts_with("--") => return Err(format!("unknown option {y}")),
            _ => rest.push(x.clone()),
        }
    }
    o.every = o.every.max(1);
    o.jobs = o.jobs.max(1);
    Ok((rest, o))
}

fn file_name(run: &str) -> String {
    run.chars()
        .map(|c| match c {
            '*' => 'x',
            '/' => 'd',
            '(' | ')' | ',' | '=' | '@' | '+' | '[' | ']' => '_',
            c => c,
        })
        .collect()
}

const SUMMARY: [&str; 50] = [
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
];

fn g(x: f64) -> String {
    format!("{x:.6e}")
}

fn opt(x: Option<u64>) -> String {
    x.map_or("-".into(), |t| t.to_string())
}

fn summary_line(rec: &Record, s: &Summary) -> String {
    let st = &rec.stats;
    let k = &st.stock;
    let mut v = vec![rec.name.clone(), s.class.name().to_string(), g(s.d0)];
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
    let markets = rec.setup.instance.markets();
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
    v.push(match (&rec.hold_failure, rec.name.as_str()) {
        (None, "hold") if s.class != Class::Error => "PASS".into(),
        (Some(f), "hold") => format!("FAIL: {f}"),
        _ => "-".into(),
    });
    v.push(match &rec.stop {
        probe::harness::Stop::Ran => "ran".into(),
        probe::harness::Stop::Runaway(_) => "runaway".into(),
        probe::harness::Stop::Error(_) => "error".into(),
    });
    v.push(s.why.clone());
    v.push(k.withheld.to_string());
    v.push(k.switches.to_string());
    v.push(g(k.markup_low));
    v.join("\t")
}

/// §7.11's statistics in long form: run, statistic, where, value.
fn stats_lines(rec: &Record) -> Vec<String> {
    let st = &rec.stats;
    let inst = &rec.setup.instance;
    let markets = inst.markets();
    let desks = outputs(inst);
    let item_names = items();
    let mut out = Vec::new();
    let mut put = |stat: &str, at: &str, value: String| {
        out.push(format!("{}\t{stat}\t{at}\t{value}", rec.name));
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
        let market = rec.engine_markets.get(*m).cloned().unwrap_or_default();
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

fn one(o: &Options, name: &str) -> Result<(String, Vec<String>), String> {
    let inst = &o.setup.instance;
    let mut csv = match &o.csv {
        Some(dir) => {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = dir.join(format!("{}.csv", file_name(name)));
            let f = std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut w = std::io::BufWriter::new(f);
            writeln!(w, "{}", csv_header(inst)).map_err(|e| e.to_string())?;
            Some(w)
        }
        None => None,
    };
    let every = o.every;
    let mut last_written = None;
    let mut pending = None;
    let mut failed = None;
    let rec = run(&o.setup, name, o.ticks, &mut |row| {
        if let Some(w) = csv.as_mut() {
            if row.tick % every == 0 {
                if let Err(e) = writeln!(w, "{}", row.csv()) {
                    failed = Some(e.to_string());
                }
                last_written = Some(row.tick);
            } else {
                pending = Some(row.csv());
            }
        }
    })?;
    if let Some(w) = csv.as_mut() {
        if let (Some(line), Some(last)) = (pending, rec.last.as_ref()) {
            if last_written != Some(last.tick) {
                writeln!(w, "{line}").map_err(|e| e.to_string())?;
            }
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    if let Some(e) = failed {
        return Err(e);
    }
    Ok((summary_line(&rec, &rec.summary), stats_lines(&rec)))
}

fn parallel<T: Send>(
    names: &[String],
    jobs: usize,
    f: impl Fn(&str) -> Result<T, String> + Sync,
) -> Vec<(usize, Result<T, String>)> {
    let mut out: Vec<(usize, Result<T, String>)> = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..jobs)
            .map(|j| {
                let f = &f;
                scope.spawn(move || {
                    names
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| i % jobs == j)
                        .map(|(i, n)| (i, f(n)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for h in handles {
            if let Ok(v) = h.join() {
                out.extend(v);
            }
        }
    });
    out.sort_by_key(|(i, _)| *i);
    out
}

fn print_point(setup: &Setup) -> Result<(), String> {
    let inst = &setup.instance;
    let tpy = setup.tpy;
    let show = |label: &str, b: f64| -> Result<(), String> {
        let i = inst.with_b(b);
        let e = i.point(tpy)?;
        println!(
            "{}\t{label}\tb {b:?}\tx* {:?}\t1-x* {:?}\tv {:?}\tP_s {:?}\tY {:?}\tN_a {:?}\tp {:?}\tp_f {:?}\tp_K {:?}\tp_h {:?}\tO {:?}\tq_f {:?}\tq_b {:?}\tsold {:?}\thours {:?}\ttask_hours {:?}\theads_tasks {:?}\theads_maker {:?}\tfunded {} ({:?})",
            i.id, e.x_star, e.one_minus_x, e.v, e.p_s, e.y, e.n_a, e.p, e.pf, e.pk, e.ph, e.o,
            e.qf, e.made, e.sold, e.hours, e.task_hours, e.capacity, e.serving, e.funded,
            e.provider_baskets
        );
        Ok(())
    };
    let b = inst.county.b;
    show("base", b)?;
    for f in [1.1, 0.9, 2.0, 0.5] {
        show(&format!("b*{f}"), b * f)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match real_main() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("horses: {e}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        return Err(
            "run, family, list, tape, kick, slowest, elasticity, openloop or point; see the \
             source's header"
                .into(),
        );
    };
    let (rest, o) = options(&args[1..])?;
    let inst = o.setup.instance.clone();
    match cmd.as_str() {
        "point" => {
            print_point(&o.setup)?;
            return Ok(true);
        }
        "tape" => {
            let mut setup = o.setup.clone();
            for name in &rest {
                Perturbation::parse(name)?.apply(&mut setup, o.ticks)?;
            }
            print!("{}", tape_ron(&setup)?);
            return Ok(true);
        }
        "elasticity" => {
            let e = elasticity(&o.setup, o.h)?;
            println!(
                "{} {} tpy {}: one tick from the mode-A genesis, central difference over {} in ln p",
                inst.id,
                o.setup.dials.set,
                o.setup.tpy,
                2.0 * o.h
            );
            println!("market\tk\teps_d\teps_s\tmultiplier\ttau");
            for (i, m) in e.markets.iter().enumerate() {
                println!(
                    "{m}\t{:.6}\t{:+.4}\t{:+.4}\t{:.4}\t{:.1}",
                    e.k[i], e.eps_d[i][i], e.eps_s[i][i], e.multiplier[i], e.tau[i]
                );
            }
            println!(
                "tau_max {:.1} ticks; 200 tau_max rule: L = {}",
                e.tau_max, e.l
            );
            return Ok(true);
        }
        "openloop" => {
            let lags = [0, 1, 5, 20, 200];
            let ol = open_loop(&o.setup, o.h, &lags)?;
            let actors = inst.actors();
            println!(
                "{} {} tpy {}: every price rate 0; d ln(D/S) and d ln(coin) per unit d ln p",
                inst.id, o.setup.dials.set, o.setup.tpy
            );
            for (j, m) in ol.markets.iter().enumerate() {
                println!(" price of {m}:");
                for (k, lag) in ol.lags.iter().enumerate() {
                    let r: Vec<String> = ol
                        .markets
                        .iter()
                        .zip(&ol.response[j][k])
                        .map(|(n, x)| format!("{n} {x:+.3}"))
                        .collect();
                    let c: Vec<String> = actors
                        .iter()
                        .zip(&ol.coin[j][k])
                        .map(|(n, x)| format!("{n} {x:+.3}"))
                        .collect();
                    println!("   lag {lag:3}: {}", r.join("  "));
                    println!("            coin: {}", c.join("  "));
                }
            }
            println!(" undisplaced, prices frozen: ln(D/S) by market");
            for (k, lag) in ol.lags.iter().enumerate() {
                let r: Vec<String> = ol
                    .markets
                    .iter()
                    .zip(&ol.hold[k])
                    .map(|(n, x)| format!("{n} {x:+.3e}"))
                    .collect();
                println!("   lag {lag:3}: {}", r.join("  "));
            }
            return Ok(true);
        }
        "slowest" => {
            let name = rest.first().map_or("hold", String::as_str);
            let horizon = o.horizon.unwrap_or(o.ticks);
            let (k, env) = kick_set(&o.setup, name, o.ticks, horizon)?;
            let gfit = slowest_mode(&env);
            let tpy = f64::from(o.setup.tpy);
            match gfit {
                Some(gt) => println!(
                    "{}\t{name}\thorizon {horizon}\tkick set {}\tg {gt:.8} a tick, {:.6} a year\t3*T6 {}",
                    inst.id,
                    if k.pass { "PASS" } else { "FAIL" },
                    rustyecon_core::num::pow(gt, tpy),
                    three_t6(gt)
                ),
                None => println!(
                    "{}\t{name}\thorizon {horizon}\tkick set {}\tno fit window",
                    inst.id,
                    if k.pass { "PASS" } else { "FAIL" }
                ),
            }
            if let Some(dir) = &o.csv {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                let path = dir.join(format!("envelope-{}-{}.tsv", inst.id, file_name(name)));
                let body: Vec<String> = env
                    .iter()
                    .enumerate()
                    .map(|(t, x)| format!("{t}\t{x:e}"))
                    .collect();
                std::fs::write(&path, format!("tick\tgain\n{}\n", body.join("\n")))
                    .map_err(|e| format!("{}: {e}", path.display()))?;
            }
            return Ok(true);
        }
        _ => {}
    }
    let names: Vec<String> = match cmd.as_str() {
        "run" | "kick" => rest,
        "family" | "list" => {
            let mut v = Vec::new();
            for f in &rest {
                v.extend(family(&inst, o.setup.tpy, f)?);
            }
            v
        }
        x => return Err(format!("unknown command {x}")),
    };
    if cmd == "list" {
        let tiers = battery(&inst, o.setup.tpy)?;
        for n in names {
            match tiers.iter().find(|r| r.name == n) {
                Some(r) if r.tier == 4 => println!("{n}\ttier 3S"),
                Some(r) => println!("{n}\ttier {}", r.tier),
                None => println!("{n}"),
            }
        }
        return Ok(true);
    }
    if cmd == "kick" {
        let horizon = o.horizon.unwrap_or(o.ticks);
        println!("run\tat\thorizon\tpass\tkicks\tlargest_gain_tail\tlargest_gain_peak\tnotes");
        let mut ok = true;
        for (i, r) in parallel(&names, o.jobs, |n| kick_set(&o.setup, n, o.ticks, horizon)) {
            match r {
                Ok((k, _)) => {
                    let tail = k.kicks.iter().map(|x| x.gain_tail).fold(0.0, f64::max);
                    let peak = k.kicks.iter().map(|x| x.gain_peak).fold(0.0, f64::max);
                    println!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        k.name,
                        k.at,
                        k.horizon,
                        if k.pass { "PASS" } else { "FAIL" },
                        k.kicks.len(),
                        g(tail),
                        g(peak),
                        k.notes.join("; ")
                    );
                    if let Some(dir) = &o.csv {
                        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                        let lines: Vec<String> = k
                            .kicks
                            .iter()
                            .map(|x| {
                                format!(
                                    "{}\t{}\t{}\t{}\t{}\t{}",
                                    x.market,
                                    x.sign,
                                    g(x.size),
                                    g(x.gain_tail),
                                    g(x.gain_peak),
                                    x.error.clone().unwrap_or_default()
                                )
                            })
                            .collect();
                        let path = dir.join(format!("kick-{}.tsv", file_name(&k.name)));
                        std::fs::write(
                            &path,
                            format!(
                                "market\tsign\tsize\tgain_tail\tgain_peak\terror\n{}\n",
                                lines.join("\n")
                            ),
                        )
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    }
                }
                Err(e) => {
                    ok = false;
                    eprintln!("horses: {}: {e}", names[i]);
                }
            }
        }
        return Ok(ok);
    }
    println!("{}", SUMMARY.join("\t"));
    let mut ok = true;
    let mut table = vec![SUMMARY.join("\t")];
    let mut stats = vec!["run\tstat\twhere\tvalue".to_string()];
    for (i, l) in parallel(&names, o.jobs, |n| one(&o, n)) {
        match l {
            Ok((line, st)) => {
                println!("{line}");
                table.push(line);
                stats.extend(st);
            }
            Err(e) => {
                ok = false;
                eprintln!("horses: {}: {e}", names[i]);
            }
        }
    }
    if let Some(dir) = &o.csv {
        for (file, body) in [("summary.tsv", &table), ("stats.tsv", &stats)] {
            let path = dir.join(file);
            if let Err(e) = std::fs::write(&path, body.join("\n") + "\n") {
                eprintln!("horses: {}: {e}", path.display());
                ok = false;
            }
        }
    }
    Ok(ok)
}
