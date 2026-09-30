//! The `horses` binary's commands for a loop instance (P2.2b; LOOPS-RULES §8): `--inst` naming
//! one of LOOPS-RULES §7.1's ids sends the command here. The commands and outputs are P2.2a's
//! (HORSES-RULES §5), with the loop's setup options (`probe::horses::cli::parse_loops`) and its
//! readouts appended (§8.5):
//!
//! ```text
//! horses run NAME...     --inst ID [options]  run named perturbations (`loops::perturb`)
//! horses family FAMILY   --inst ID [options]  run a family: battery, tier3s, stocks
//! horses list FAMILY     --inst ID [options]  print a family's run names (the battery with its tier)
//! horses tape [NAME]     --inst ID [options]  print the tape a run is made from
//! horses kick NAME...    --inst ID [options]  the kick set at the end of each named run
//! horses slowest NAME    --inst ID [options]  the kick set's envelope and its slowest mode, g
//! horses elasticity      --inst ID [options]  the one-tick elasticity probe and the run length
//! horses point           --inst ID [options]  the oracle's point, at the base and each cost target
//! ```
//!
//! Run options: `--ticks L` (default 20,000), `--csv DIR`, `--every K`, `--jobs J`, `--horizon
//! H`, `--h X`, as P2.2a's.

use super::harness::{csv_header, run, stats_lines, summary_line};
use super::perturb::{battery, family, Perturbation};
use super::probes::{elasticity, kick_set, mirror_three_t6, run_length};
use super::{tape_ron, Setup};
use crate::horses::cli::parse_loops;
use crate::horses::kick::{slowest_mode, three_t6};
use crate::horses::report::{file_name, g, SUMMARY};
use crate::protocol::RUN_TICKS;
use std::io::Write as _;
use std::path::PathBuf;

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
    let a = parse_loops(
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

fn one(o: &Options, name: &str) -> Result<(String, Vec<String>), String> {
    let mut csv = match &o.csv {
        Some(dir) => {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = dir.join(format!("{}.csv", file_name(name)));
            let f = std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut w = std::io::BufWriter::new(f);
            writeln!(w, "{}", csv_header(&o.setup)).map_err(|e| e.to_string())?;
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
            if row.row.tick % every == 0 {
                if let Err(e) = writeln!(w, "{}", row.csv()) {
                    failed = Some(e.to_string());
                }
                last_written = Some(row.row.tick);
            } else {
                pending = Some(row.csv());
            }
        }
    })?;
    if let Some(w) = csv.as_mut() {
        if let (Some(line), Some(last)) = (pending, rec.last.as_ref()) {
            if last_written != Some(last.row.tick) {
                writeln!(w, "{line}").map_err(|e| e.to_string())?;
            }
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    if let Some(e) = failed {
        return Err(e);
    }
    Ok((summary_line(&rec), stats_lines(&rec)))
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
            "{}\t{label}\tb {b:?}\tx* {:?}\t1-x* {:?}\tv {:?}\tP_s {:?}\tY {:?}\tN_a {:?}\tp {:?}\tp_f {:?}\tp_K {:?}\tp_h {:?}\tO {:?}\tq_f {:?}\tq_b {:?}\thours {:?}\ttask_hours {:?}\tfodder_hours {:?}\theads {:?}\tfunded {} ({:?})",
            i.id, e.x_star, e.one_minus_x, e.v, e.p_s, e.y, e.n_a, e.p, e.pf, e.pk, e.ph, e.o,
            e.qf, e.made, e.hours, e.task_hours, e.fodder_hours, e.capacity, e.funded,
            e.provider_baskets
        );
        Ok(())
    };
    let b = inst.b;
    show("base", b)?;
    for f in [1.1, 0.9, 2.0, 0.5] {
        show(&format!("b*{f}"), b * f)?;
    }
    Ok(())
}

/// Run the command `cmd` with `args` (every argument after it) on a loop instance. Returns
/// whether every run succeeded.
pub fn main(cmd: &str, args: &[String]) -> Result<bool, String> {
    let (rest, o) = options(args)?;
    let inst = o.setup.instance.clone();
    match cmd {
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
            let t6 = mirror_three_t6(&inst.id)?;
            // Where the mirror's T6 is infinite, LB1's length at this tick length.
            let fallback = if t6.is_none() {
                let mut lb1 = Setup::registered("lb1", o.setup.tpy)?;
                lb1.one_sided = o.setup.one_sided;
                let el = elasticity(&lb1, o.h)?;
                Some(run_length("lb1", o.setup.tpy, el.l, None)?)
            } else {
                None
            };
            let l = run_length(&inst.id, o.setup.tpy, e.l, fallback)?;
            println!(
                "tau_max {:.1} ticks; 200 tau_max term {}; the mirror's 3*T6 at 52 a year {}; L = {l}",
                e.tau_max,
                e.l,
                t6.map_or("infinite (LB1's L)".to_string(), |t| t.to_string())
            );
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
    let names: Vec<String> = match cmd {
        "run" | "kick" => rest,
        "family" | "list" => {
            let mut v = Vec::new();
            for f in &rest {
                v.extend(family(&inst, o.setup.tpy, f)?);
            }
            v
        }
        x => {
            return Err(format!(
                "unknown command {x} for a loop instance: run, family, list, tape, kick, slowest, \
                 elasticity or point"
            ))
        }
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
