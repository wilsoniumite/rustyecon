//! The probe harness's command line (docs/probe/RULES.md §6).
//!
//! ```text
//! probe run NAME...     [options]   run named perturbations (see `probe::perturb`)
//! probe family FAMILY   [options]   run a family: battery, stocks, joint2, joint4, history
//! probe list FAMILY                 print a family's run names
//! probe tape [NAME]     [options]   print the tape a run is made from (undisplaced: none)
//!
//! options:
//!   --ticks L              the scored length (default 20000; a dated shock adds L/4 before it)
//!   --tpy N                ticks a year (default 52); the tape is regenerated at N
//!   --set KEY=VALUE        set a dial (rate.labour, ..., tilt.desk.mach); rate.* and buffer.*
//!                          scale a family, tilt.* sets both tilts
//!   --assign planned|expost
//!   --scale cash|ceiling|step
//!   --one-sided saturate|hold
//!   --csv DIR              write DIR/<run>.csv, one row per tick, and DIR/summary.tsv
//!   --every K              write every K-th tick to the CSV (and the last)
//!   --jobs J               run J runs at once (default 1)
//! ```
//!
//! Each run prints one summary line: its class (PROBE-SPEC §4.5) and the reported numbers. Every
//! CSV row carries the oracle's values for that tick beside the observables, so gaps can be
//! recomputed.

use probe::harness::{classify, csv_header, run, Class, Record, Summary, MARKETS};
use probe::perturb::{battery, family, Perturbation};
use probe::protocol::RUN_TICKS;
use probe::setup::{tape_ron, Assign, OneSided, ScaleRule, Setup};
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

struct Options {
    ticks: u64,
    setup: Setup,
    csv: Option<PathBuf>,
    every: u64,
    jobs: usize,
}

fn options(args: &[String]) -> Result<(Vec<String>, Options), String> {
    let mut tpy = 52;
    let mut sets = Vec::new();
    let mut assign = Assign::Planned;
    let mut scale = ScaleRule::Cash;
    let mut one_sided = OneSided::Saturate;
    let mut o = Options {
        ticks: RUN_TICKS,
        setup: Setup::registered(52),
        csv: None,
        every: 1,
        jobs: 1,
    };
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or(format!("{a} takes a value"));
        match a.as_str() {
            "--ticks" => o.ticks = val()?.parse().map_err(|_| "--ticks N")?,
            "--tpy" => tpy = val()?.parse().map_err(|_| "--tpy N")?,
            "--set" => {
                let kv = val()?;
                let (k, v) = kv.split_once('=').ok_or("--set KEY=VALUE")?;
                let v: f64 = v.parse().map_err(|_| format!("--set {kv}: not a number"))?;
                sets.push((k.to_string(), v));
            }
            "--assign" => {
                assign = match val()?.as_str() {
                    "planned" => Assign::Planned,
                    "expost" => Assign::ExPost,
                    x => return Err(format!("--assign {x}: planned or expost")),
                }
            }
            "--scale" => {
                scale = match val()?.as_str() {
                    "cash" => ScaleRule::Cash,
                    "ceiling" => ScaleRule::ceiling(),
                    "step" => ScaleRule::july_step(),
                    x => return Err(format!("--scale {x}: cash, ceiling or step")),
                }
            }
            "--one-sided" => {
                one_sided = match val()?.as_str() {
                    "saturate" => OneSided::Saturate,
                    "hold" => OneSided::Hold,
                    x => return Err(format!("--one-sided {x}: saturate or hold")),
                }
            }
            "--csv" => o.csv = Some(PathBuf::from(val()?)),
            "--every" => o.every = val()?.parse().map_err(|_| "--every K")?,
            "--jobs" => o.jobs = val()?.parse().map_err(|_| "--jobs J")?,
            _ if a.starts_with("--") => return Err(format!("unknown option {a}")),
            _ => rest.push(a.clone()),
        }
    }
    let mut s = Setup::registered(tpy);
    for (k, v) in sets {
        s.dials.set(&k, v)?;
    }
    s.assign = assign;
    s.scale = scale;
    s.one_sided = one_sided;
    o.setup = s;
    o.every = o.every.max(1);
    o.jobs = o.jobs.max(1);
    Ok((rest, o))
}

fn file_name(run: &str) -> String {
    run.chars()
        .map(|c| match c {
            '*' => 'x',
            '/' => 'd',
            '(' | ')' | ',' | '=' | '@' | '+' => '_',
            c => c,
        })
        .collect()
}

fn summary_header() -> String {
    let mut h: Vec<String> = [
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
        "band_pi_m",
        "band_pi",
        "r_end",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for m in MARKETS {
        h.push(format!("trough_{m}"));
        h.push(format!("trough_{m}_tick"));
    }
    for m in MARKETS {
        h.push(format!("rationed_{m}_buyers"));
        h.push(format!("rationed_{m}_sellers"));
    }
    h.push("spoiled_mach".into());
    h.push("spoiled_good".into());
    h.push("transfer_short".into());
    h.push("mode_a".into());
    h.push("why".into());
    h.join("\t")
}

fn summary_line(rec: &Record, s: &Summary) -> String {
    let g = |x: f64| format!("{x:.6e}");
    let mut v = vec![rec.name.clone(), s.class.name().to_string(), g(s.d0)];
    v.extend(s.envelope.iter().map(|&x| g(x)));
    v.push(g(s.kappa));
    v.push(g(s.max_w));
    v.push(g(s.last));
    v.push(s.in_tol_from.map_or("-".into(), |t| t.to_string()));
    v.extend(s.dead.iter().map(|d| d.to_string()));
    v.extend(s.band.iter().map(|&x| g(x)));
    v.push(g(s.r_end));
    for (rel, t) in rec.trough {
        v.push(g(rel));
        v.push(t.to_string());
    }
    for m in rec.rationed {
        v.push(m[0].to_string());
        v.push(m[1].to_string());
    }
    v.push(g(rec.spoiled[2]));
    v.push(g(rec.spoiled[3]));
    v.push(g(rec.transfer_short));
    v.push(match (&rec.hold_failure, rec.name.as_str()) {
        (None, "hold") if s.class != Class::Error => "PASS".into(),
        (Some(f), "hold") => format!("FAIL: {f}"),
        _ => "-".into(),
    });
    v.push(s.why.clone());
    v.join("\t")
}

fn one(o: &Options, name: &str) -> Result<String, String> {
    let mut csv = match &o.csv {
        Some(dir) => {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = dir.join(format!("{}.csv", file_name(name)));
            let f = std::fs::File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut w = std::io::BufWriter::new(f);
            writeln!(w, "{}", csv_header()).map_err(|e| e.to_string())?;
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
    Ok(summary_line(&rec, &classify(&rec)))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!(
            "probe: run NAME..., family FAMILY, list FAMILY or tape; see the source's header"
        );
        return ExitCode::FAILURE;
    };
    let (rest, o) = match options(&args[1..]) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("probe: {e}");
            return ExitCode::FAILURE;
        }
    };
    let names: Vec<String> = match cmd.as_str() {
        "run" => rest,
        "family" | "list" => {
            let mut v = Vec::new();
            for f in &rest {
                match family(f) {
                    Ok(n) => v.extend(n),
                    Err(e) => {
                        eprintln!("probe: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            v
        }
        "tape" => {
            let mut setup = o.setup.clone();
            for name in &rest {
                let applied = Perturbation::parse(name).and_then(|p| p.apply(&mut setup, o.ticks));
                if let Err(e) = applied {
                    eprintln!("probe: {e}");
                    return ExitCode::FAILURE;
                }
            }
            return match tape_ron(&setup) {
                Ok(t) => {
                    print!("{t}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("probe: {e}");
                    ExitCode::FAILURE
                }
            };
        }
        x => {
            eprintln!("probe: unknown command {x}");
            return ExitCode::FAILURE;
        }
    };
    if cmd == "list" {
        let tiers = battery();
        for n in names {
            let tier = tiers.iter().find(|(b, _)| *b == n).map(|(_, t)| *t);
            match tier {
                Some(t) => println!("{n}\ttier {t}"),
                None => println!("{n}"),
            }
        }
        return ExitCode::SUCCESS;
    }
    println!("{}", summary_header());
    let mut lines: Vec<(usize, Result<String, String>)> = Vec::new();
    let chunks: Vec<Vec<(usize, &String)>> = {
        let mut c: Vec<Vec<(usize, &String)>> = vec![Vec::new(); o.jobs];
        for (i, n) in names.iter().enumerate() {
            c[i % o.jobs].push((i, n));
        }
        c
    };
    std::thread::scope(|scope| {
        let handles: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                let o = &o;
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|(i, n)| (*i, one(o, n)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for h in handles {
            if let Ok(v) = h.join() {
                lines.extend(v);
            }
        }
    });
    lines.sort_by_key(|(i, _)| *i);
    let mut ok = true;
    let mut table = vec![summary_header()];
    for (i, l) in lines {
        match l {
            Ok(line) => {
                println!("{line}");
                table.push(line);
            }
            Err(e) => {
                ok = false;
                eprintln!("probe: {}: {e}", names[i]);
            }
        }
    }
    if let Some(dir) = &o.csv {
        let path = dir.join("summary.tsv");
        if let Err(e) = std::fs::write(&path, table.join("\n") + "\n") {
            eprintln!("probe: {}: {e}", path.display());
            ok = false;
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
