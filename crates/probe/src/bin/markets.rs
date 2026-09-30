//! The markets probe's command line (MARKETS-SPEC §7; docs/probe/MARKETS-RULES.md §6).
//!
//! ```text
//! markets run NAME...     [options]  run named perturbations (see `probe::markets::perturb`)
//! markets family FAMILY   [options]  run a family: battery, tier3s, stocks, joint2, joint4, basin,
//!                                    history, and at the commons enclose (tier3s reads its first
//!                                    year's D̂ as its start)
//! markets list FAMILY     [options]  print a family's run names (the battery with tier and slack)
//! markets tape [NAME]     [options]  print the tape a run is made from
//! markets kick NAME...    [options]  the kick set at the end of each named run (§7.5)
//! markets elasticity      [options]  the one-tick elasticity probe and L (§7.8)
//! markets openloop        [options]  the open-loop probe, prices frozen (§7.8)
//! markets point           [options]  the oracle's point, at the base and each cost target
//!
//! setup options (probe::markets::cli):
//!   --inst ID              i0, i1, i2, i3, l2, l3, g1, the wall's iw1 and ic1, or the commons'
//!                          c1, c2 and c1n (default i1)
//!   --tpy N                ticks a year (default 52)
//!   --dials c2m|c2l        the registered dial set (default c2m)
//!   --set KEY=VALUE        set a dial; rate.*, buffer.* and adjust.* scale a family, tilt.* sets
//!   --one-sided saturate|hold
//! run options:
//!   --ticks L              the scored length (default 20000; a dated shock adds L/4 before it)
//!   --csv DIR              write DIR/<run>.csv, one row per tick, DIR/summary.tsv, DIR/stats.tsv
//!   --every K              write every K-th tick to the CSV (and the last)
//!   --jobs J               run J runs at once (default 1)
//!   --horizon H            the kick's horizon (default: the scored length)
//!   --h X                  the probes' step in log price (default 0.01)
//!   --first-year           a stock or coin start's distance is its first year's largest D̂
//!                          (Tier 3S; `family tier3s` sets it)
//! ```
//!
//! Each run prints one summary line: its class (PROBE-SPEC §4.5) and the reported numbers, with
//! §7.11's transient statistics in long form in `stats.tsv`; at the wall (`--inst iw1`, `ic1`)
//! `stats.tsv` adds the wall's readouts (`wall.*`; docs/probe/WALL-RULES.md §5), and at the
//! commons (`--inst c1`, `c2`, `c1n`) the commons' (`commons.*`; docs/probe/COMMONS-RULES.md §5).

use probe::harness::{Class, Summary};
use probe::markets::cli::parse;
use probe::markets::harness::{csv_header, items, run, Record};
use probe::markets::instance::Instance;
use probe::markets::kick::kick_set;
use probe::markets::perturb::{battery, family, tier3s, Perturbation};
use probe::markets::probes::{elasticity, open_loop};
use probe::markets::setup::{tape_ron, Setup};
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
            "--first-year" => o.setup.first_year_d0 = true,
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

const SUMMARY: [&str; 30] = [
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
    "mode_a",
    "why",
];

fn g(x: f64) -> String {
    format!("{x:.6e}")
}

fn summary_line(rec: &Record, s: &Summary) -> String {
    let st = &rec.stats;
    let mut v = vec![rec.name.clone(), s.class.name().to_string(), g(s.d0)];
    v.extend(s.envelope.iter().map(|&x| g(x)));
    v.push(g(s.kappa));
    v.push(g(s.max_w));
    v.push(g(s.last));
    v.push(s.in_tol_from.map_or("-".into(), |t| t.to_string()));
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
    v.push(match (&rec.hold_failure, rec.name.as_str()) {
        (None, "hold") if s.class != Class::Error => "PASS".into(),
        (Some(f), "hold") => format!("FAIL: {f}"),
        _ => "-".into(),
    });
    v.push(s.why.clone());
    v.join("\t")
}

/// §7.11's statistics in long form: run, statistic, where, value.
fn stats_lines(rec: &Record) -> Vec<String> {
    let st = &rec.stats;
    let inst = &rec.setup.instance;
    let markets = inst.markets();
    let desks = inst.desks();
    let item_names = items(inst);
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
    // The wall's own readouts (the wall frame's §5.3), reported and never scored.
    if let Some(w) = &st.wall {
        put("wall.depth_min", "-", g(w.depth_min.0));
        put("wall.depth_min_tick", "-", w.depth_min.1.to_string());
        put("wall.breach_ticks", "-", w.breach.to_string());
        put(
            "wall.first_breach",
            "-",
            w.first_breach.map_or("-".into(), |t| t.to_string()),
        );
        put("wall.breach_run", "-", w.breach_run.to_string());
        for (c, s) in inst.categories.iter().zip(&w.share_max) {
            put("wall.share_max", &c.key, g(*s));
        }
        for (p, (lo, hi)) in inst.households()[1..].iter().zip(&w.participation) {
            put("wall.participation_lo", p, g(*lo));
            put("wall.participation_hi", p, g(*hi));
        }
        for (p, n) in inst.households()[1..].iter().zip(&w.saturated) {
            put("wall.saturated_ticks", p, n.to_string());
        }
        put("wall.worst_buyer_fill", "-", g(w.worst_buyer_fill));
    }
    // The commons' readouts (the commons frame's §3.8), reported and never scored.
    if let Some(c) = &st.commons {
        for (k, n) in ["Unused", "Commons", "Crowded", "Enclosed", "Split"]
            .iter()
            .zip(c.ticks)
        {
            put("commons.regime_ticks", k, n.to_string());
        }
        put("commons.switches", "-", c.switches.to_string());
        put("commons.regime_end", "-", c.end.clone());
        put("commons.regime_star", "-", c.star.clone());
        put("commons.ro_low", "-", g(c.rent.0));
        put("commons.ro_high", "-", g(c.rent.1));
        put("commons.ro_end", "-", g(c.rent.2));
        put("commons.ro_star", "-", g(c.rent_star));
        put("commons.tp_max", "-", g(c.rented_max));
        put("commons.provider_coin_low", "-", g(c.provider_coin_low));
    }
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
                if let Err(e) = writeln!(w, "{}", row.csv(inst)) {
                    failed = Some(e.to_string());
                }
                last_written = Some(row.tick);
            } else {
                pending = Some(row.csv(inst));
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

fn print_point(inst: &Instance, tpy: u32) -> Result<(), String> {
    let show = |label: &str, i: &Instance| -> Result<(), String> {
        let e = i.point(tpy)?;
        print!(
            "{}\t{label}\tx* {:?}\t1-x* {:?}\tv {:?}\tP_s {:?}\tY {:?}\tN_a {:?}\ttype prices {:?}\ttype services {:?}\ttype traded {:?}\tcategory prices {:?}\tcategory outputs {:?}\tmargin_active {}",
            i.id, e.x_star, e.one_minus_x, e.v, e.p_s, e.y, e.n_a, e.type_price, e.type_services,
            e.type_traded, e.cat_price, e.cat_output, e.margin_active
        );
        if i.worker_form {
            print!(
                "\tmargin {}\tn_D {:?}\treserved wages {:?}\thours {:?}\tpop baskets {:?}\tprovider baskets {:?}",
                e.margin, e.pool, e.wage, e.hours, e.pop_baskets, e.provider_baskets
            );
        }
        if let Some(c) = &e.commons {
            print!(
                "\tplots {}\tland {}\tr_o {:?}\tT_p {:?}\tcommons used {:?}\texit value {:?}\tS {:?}\tworker baskets {:?}\tprovider baskets {:?}\tfunded {}\tcertified {}",
                c.regime, c.land_market, c.plot_rent, c.rented, c.occupied, c.exit_value, e.pool,
                e.worker_baskets, e.provider_baskets, c.funded, c.certified
            );
        }
        println!();
        Ok(())
    };
    show("base", inst)?;
    for c in &inst.coefs {
        for v in &c.values {
            let mut i = inst.clone();
            let x: f64 = v.parse().map_err(|_| format!("{v}: not a number"))?;
            i.set(&c.param, x)?;
            show(&format!("{}={v}", c.name), &i)?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match real_main() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("markets: {e}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        return Err(
            "run, family, list, tape, kick, elasticity, openloop or point; see the source's header"
                .into(),
        );
    };
    let (rest, mut o) = options(&args[1..])?;
    let inst = o.setup.instance.clone();
    // Tier 3S reads its first year's largest D̂ as its start distance (decision 229).
    if cmd == "family" && rest.iter().any(|f| f == "tier3s") {
        if rest.len() > 1 {
            return Err("run tier3s as a family of its own: its start distance differs".into());
        }
        o.setup.first_year_d0 = true;
    }
    match cmd.as_str() {
        "point" => {
            print_point(&inst, o.setup.tpy)?;
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
            println!("eps_S (row: quantity of the market; column: price of the market)");
            for (i, m) in e.markets.iter().enumerate() {
                let r: Vec<String> = e.eps_s[i].iter().map(|x| format!("{x:+.4}")).collect();
                println!("  {m}\t{}", r.join("\t"));
            }
            println!("eps_D");
            for (i, m) in e.markets.iter().enumerate() {
                let r: Vec<String> = e.eps_d[i].iter().map(|x| format!("{x:+.4}")).collect();
                println!("  {m}\t{}", r.join("\t"));
            }
            println!("tau_max {:.1} ticks; L = {}", e.tau_max, e.l);
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
        _ => {}
    }
    let names: Vec<String> = match cmd.as_str() {
        "run" | "kick" => rest.clone(),
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
        let three_s = tier3s(&inst);
        let in_3s = rest.iter().any(|f| f == "tier3s");
        for n in names {
            match tiers.iter().find(|r| r.name == n) {
                Some(r) => println!(
                    "{n}\ttier {}{}",
                    r.tier,
                    if r.slack { "\tslack" } else { "" }
                ),
                None if in_3s && three_s.contains(&n) => println!("{n}\ttier 3S"),
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
                Ok(k) => {
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
                    eprintln!("markets: {}: {e}", names[i]);
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
                eprintln!("markets: {}: {e}", names[i]);
            }
        }
    }
    if let Some(dir) = &o.csv {
        for (file, body) in [("summary.tsv", &table), ("stats.tsv", &stats)] {
            let path = dir.join(file);
            if let Err(e) = std::fs::write(&path, body.join("\n") + "\n") {
                eprintln!("markets: {}: {e}", path.display());
                ok = false;
            }
        }
    }
    Ok(ok)
}
