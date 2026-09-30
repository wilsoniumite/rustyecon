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
//!   --inst ID              h1-h4, f1-f10, r1a, p7 or p8 (default h1); or demo:<key>@<year>,
//!                          a demo county-date from --counties (D2.4)
//!   --counties PATH        the demo's county table, written by `rustyecon worldgen
//!                          worlds/demo-gb --stage v2a1 --instances PATH`; a demo row's ψ is its
//!                          reservation unless --reserve gives another
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
//! §7.11's transient and stock statistics in long form in `stats.tsv`. Columns 48–50 of
//! `summary.tsv` are the idle market's (L0.4): the scored ticks the maker withheld, its switches
//! between withholding and offering, and its lowest net markup. The last nine are the loop
//! step's (LOOPS-RULES §8.5; `probe::horses::report`), `-` for these instances.
//!
//! A loop instance (P2.2b; `--inst lb1` and the rest of LOOPS-RULES §7.1's ids) runs through
//! `probe::horses::loops::cmd`: the same commands but `openloop`, with the loop's setup options
//! (`--dials c2g|c2g13`, `--one-sided`, `--reserve PSI|none`, `--theta X`, `--plant-delta D`,
//! `--fixed-plants`) and its readouts appended.

use probe::harness::Summary;
use probe::horses::cli::{inst_of, parse};
use probe::horses::harness::{csv_header, items, outputs, run, Record};
use probe::horses::kick::{kick_set, slowest_mode, three_t6};
use probe::horses::loops::{cmd as loops_cmd, Instance as LoopInstance};
use probe::horses::perturb::{battery, family, Perturbation};
use probe::horses::probes::{elasticity, open_loop};
use probe::horses::report::{envelope_name, file_name, g, stats_fields, summary_fields, SUMMARY};
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

/// The summary line of a P2.2a run: P2.2a's 50 columns, and the loop step's nine as `-`
/// (decision 292).
fn summary_line(rec: &Record, s: &Summary) -> String {
    let mut v = summary_fields(
        &rec.name,
        s,
        &rec.stats,
        &rec.setup.instance.markets(),
        &rec.hold_failure,
        &rec.stop,
    );
    v.extend(std::iter::repeat_n(
        "-".to_string(),
        SUMMARY.len() - v.len(),
    ));
    v.join("	")
}

/// §7.11's statistics in long form: run, statistic, where, value.
fn stats_lines(rec: &Record) -> Vec<String> {
    let inst = &rec.setup.instance;
    stats_fields(
        &rec.stats,
        &inst.markets(),
        &outputs(inst),
        &items(),
        &rec.engine_markets,
    )
    .into_iter()
    .map(|(stat, at, value)| format!("{}	{stat}	{at}	{value}", rec.name))
    .collect()
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
    if inst_of(&args[1..]).is_some_and(LoopInstance::is_loop) {
        return loops_cmd::main(cmd, &args[1..]);
    }
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
                let path = dir.join(envelope_name(&inst.id, name));
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
