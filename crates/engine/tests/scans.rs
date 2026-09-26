//! Source scans of the engine path, core, markets, agents and engine (docs/ENGINE.md §9 and
//! §11): they read the shipped sources (every file with its `#[cfg(test)]` items removed) with
//! comments stripped and literal contents blanked, and back up clippy.toml's lists (A5, R8, R4,
//! A12, E2).

mod common;

use common::scan::{
    engine_path_tokens, float_violations, int_float_violations, shipped, sources, strip, tokens,
    Tok,
};

fn ident(t: &Tok, name: &str) -> bool {
    matches!(t, Tok::Ident(s) if s == name)
}

fn punct(t: &Tok, ch: char) -> bool {
    matches!(t, Tok::Punct(c) if *c == ch)
}

#[test]
fn the_scanner_reads_what_it_should() {
    // The scans are only as good as the lexer: comments and strings are not code, tuple
    // indices and ranges are not floats, and every float spelling is one. A `#[cfg(test)]`
    // item is not shipped, but the code after it is, even when a test-only helper comes early
    // in a file.
    let src = r##"
        // 2.5 in a comment, and .exp( too
        /* 3.5 /* nested 4.5 */ still 5.5 */
        let s = "6.5 and println!";
        let r = r#"7.5"#;
        let c = '8';
        let l: &'a str = x.0.1;
        for i in 0..10 {}
        let a = 1.0; let b = 0.0; let d = 2.; let e = 1e-12; let f = 3f64; let g = 0x1e5;
        fn t<'a>() {}
        #[cfg(test)]
        pub(crate) fn raw_mut(&mut self, k: [f64; 2]) -> &mut Vec<(u32, Vec<f64>)> { let z = 3.25; &mut self.0 }
        impl Deserialize for Inventory { fn f() { let floor = 4.25; } }
        #[cfg(test)]
        mod testkit;
        let m = 6.75;
        #[cfg(test)]
        mod tests { fn g() { let q = 7.25; } }
        #[cfg(test)]
        let h = 9.5;
    "##;
    let stripped = strip(src);
    assert!(!stripped.contains("2.5") && !stripped.contains("3.5") && !stripped.contains("5.5"));
    assert!(!stripped.contains("6.5") && !stripped.contains("7.5") && !stripped.contains('8'));
    let floats: Vec<String> = tokens(&shipped(&stripped))
        .into_iter()
        .filter_map(|t| match t {
            Tok::Num(s, true) => Some(s),
            _ => None,
        })
        .collect();
    assert_eq!(floats, ["1.0", "0.0", "2.", "1e-12", "3", "4.25", "6.75"]);
    // Named float constants are behavioural numbers too (A12): an absolute threshold spelled
    // `f64::EPSILON` is caught like one spelled `1e-12`. Integer limits and the identities are
    // not.
    let src = r"
        if part > f64::EPSILON {}
        if x <= f32::MIN_POSITIVE {}
        let cap = f64::MAX; let pi = std::f64::consts::PI; use std::f64::EPSILON;
        let n = u32::MAX; let z = 0.0; let one = 1.0; let w = f64::from(n);
    ";
    let found = float_violations(&tokens(&shipped(&strip(src))), false);
    assert_eq!(found.len(), 5, "{found:#?}");
    // core::num may name the limits it searches between.
    let limits = tokens(&strip("let top = f64::INFINITY; let big = f64::MAX;"));
    assert!(float_violations(&limits, true).is_empty());
    assert_eq!(float_violations(&limits, false).len(), 2);
    // An integer made float is a behavioural number too (O13): the review's two mutants, a
    // dead band of 1/1e9 and a fraction of 1/2, and the other spellings. A variable, an
    // integer type's limit, 0 and 1, and an allowed calendar constant are not.
    let src = r"
        if x.abs() < 1.0 / f64::from(1_000_000_000u32) { 0.0 } else { x }
        SellQty::AllHeld => left / f64::from(2u8),
        let w = 52 as f64; let v = (n * 26) as f64; let z: f64 = 7u8.into();
        let b = f64::from_bits(0x3fe0000000000000); let p = s.parse::<f64>();
        let c = f64::from(WEEKS); let d = DAYS as f64;
        let n = f64::from(self.ticks_per_year); let m = f64::from(u32::MAX); let k = t as f64;
        let one = 1 as f64; let zero = f64::from(0u8); let bits = f64::from_bits(0);
        let y = f64::from(YEAR_E4_DAYS); let len = v.len() as f64;
    ";
    let found = int_float_violations(&tokens(&shipped(&strip(src))), &["YEAR_E4_DAYS"]);
    let named: Vec<&str> = found.iter().map(|f| f.split(' ').next().unwrap()).collect();
    assert_eq!(
        named,
        [
            "1000000000",
            "2",
            "52",
            "26",
            "7",
            "0x3fe0000000000000",
            "parse::<float>",
            "WEEKS",
            "DAYS"
        ],
        "{found:#?}"
    );
}

#[test]
fn no_raw_transcendentals() {
    // A5: transcendentals go through core::num, which calls libm; the platform maths libraries
    // differ in the last bit. No inherent transcendental, powi, powf or mul_add, no f32, and
    // libm only inside core's num module.
    const BANNED: [&str; 27] = [
        "exp", "exp2", "exp_m1", "ln", "ln_1p", "log", "log2", "log10", "powf", "powi", "cbrt",
        "hypot", "sin", "cos", "tan", "sin_cos", "asin", "acos", "atan", "atan2", "sinh", "cosh",
        "tanh", "asinh", "acosh", "atanh", "mul_add",
    ];
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
        let num_module = path.replace('\\', "/").ends_with("core/src/num.rs");
        for (i, t) in toks.iter().enumerate() {
            let next = toks.get(i + 1);
            let method = i > 0
                && punct(&toks[i - 1], '.')
                && BANNED.iter().any(|b| ident(t, b))
                && next.is_some_and(|n| punct(n, '('));
            let path_call = (ident(t, "f64") || ident(t, "f32"))
                && toks.get(i + 1).is_some_and(|n| punct(n, ':'))
                && toks
                    .get(i + 3)
                    .is_some_and(|n| BANNED.iter().any(|b| ident(n, b)));
            let f32 = ident(t, "f32");
            let libm = ident(t, "libm") && !num_module;
            if method || path_call || f32 || libm {
                found.push(format!("{path}: {t:?} {next:?}"));
            }
        }
    }
    assert!(found.is_empty(), "raw transcendentals: {found:#?}");
}

#[test]
fn no_hashed_collections() {
    // R8: no container that iterates in a per-process random order, and no rayon.
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
        for t in &toks {
            if let Tok::Ident(s) = t {
                let hashed = s.contains("HashMap")
                    || s.contains("HashSet")
                    || ["hash_map", "hash_set", "RandomState", "rayon"].contains(&s.as_str());
                if hashed {
                    found.push(format!("{path}: {s}"));
                }
            }
        }
    }
    assert!(found.is_empty(), "hashed collections: {found:#?}");
}

/// The `Clock` conversions of docs/ENGINE.md §6 that turn a param into a per-tick value.
const CONVERSIONS: [&str; 6] = [
    "flow", "share", "log_step", "compound", "fraction", "weight",
];

/// Reads of a param that go around its `Site`: a conversion called as a method or through a
/// path, a typed `Params::get`, or an untyped `params.value`.
fn site_bypasses(toks: &[Tok]) -> Vec<String> {
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let Tok::Ident(name) = t else { continue };
        let at = |k: usize| toks.get(i + k);
        let back = |k: usize| i.checked_sub(k).and_then(|j| toks.get(j));
        let called = at(1).is_some_and(|n| punct(n, '('));
        let method = back(1).is_some_and(|p| punct(p, '.'));
        let path = back(1).is_some_and(|p| punct(p, ':')) && back(2).is_some_and(|p| punct(p, ':'));
        if CONVERSIONS.contains(&name.as_str()) && called && (method || path) {
            found.push(format!("{name}("));
        }
        let turbofish = at(1).is_some_and(|n| punct(n, ':'))
            && at(2).is_some_and(|n| punct(n, ':'))
            && at(3).is_some_and(|n| punct(n, '<'));
        if name == "get" && method && turbofish {
            found.push("get::<".to_string());
        }
        if name == "params"
            && at(1).is_some_and(|n| punct(n, '.'))
            && at(2).is_some_and(|n| ident(n, "value"))
        {
            found.push("params.value".to_string());
        }
    }
    found
}

#[test]
fn params_are_read_only_through_sites() {
    // D10 item 4 (S2.2): a resolved spec, good or market holds each param it reads as a Site,
    // whose method is the one the registry lists, and the run converts a param only through
    // it. So no shipped code but core's clock.rs, where `ClockMethod` and `Site` live, calls a
    // conversion itself or reads a param by type. A rule written the old way,
    // `v.clock.log_step(v.params.get::<RatePerYear>(up)?)`, would convert as its code says
    // whatever the registry lists.
    let src = r"
        let k = w.clock.log_step(params.get::<RatePerYear>(rate)?);
        let s = Clock::share(&c, RatePerYear(v)); let x = v.params.value(p)?;
        let ok = site.per_tick(&v.params, v.clock)?; let f = d.technique.share;
        let o = rate.value(&v.params)?; let t = turnover.convert(v.clock, o)?;
        let g = self.params.get(i); let m = s.method.per_tick(&w.clock, v);
    ";
    let found = site_bypasses(&tokens(&shipped(&strip(src))));
    assert_eq!(found, ["log_step(", "get::<", "share(", "params.value"]);
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
        if path.replace('\\', "/").ends_with("core/src/clock.rs") {
            continue;
        }
        for v in site_bypasses(&toks) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(found.is_empty(), "param reads around a Site: {found:#?}");
}

#[test]
fn no_behavioural_float_literals() {
    // R4, A12: every behavioural number comes from the tape. Shipped code may spell only 0.0
    // and 1.0 (the additive and multiplicative identities, a fill's cap, a price by
    // definition), and may name no float constant (an `f64::EPSILON` threshold is an absolute
    // epsilon), except core::num, which searches between the float limits.
    // Nor may it make an integer other than 0 and 1 float (O13), except clock.rs's calendar
    // constants, which the date-to-tick map needs.
    const CALENDAR: [&str; 2] = ["YEAR_E4_DAYS", "E4"];
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
        let path_fwd = path.replace('\\', "/");
        let num = path_fwd.ends_with("core/src/num.rs");
        for v in float_violations(&toks, num) {
            found.push(format!("{path}: {v}"));
        }
        let allowed: &[&str] = if path_fwd.ends_with("core/src/clock.rs") {
            &CALENDAR
        } else {
            &[]
        };
        for v in int_float_violations(&toks, allowed) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(
        found.is_empty(),
        "float literals in shipped code: {found:#?}"
    );
}

#[test]
fn engine_path_does_no_io() {
    // E2: no global or interior-mutable state, no file, clock, thread, environment, process or
    // network access, and no printing on the engine path. All I/O is bytes in and bytes out.
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
        found.extend(io_violations(&path, &toks, false));
    }
    assert!(
        found.is_empty(),
        "I/O or global state on the engine path: {found:#?}"
    );
}

/// The global state, standard modules and print macros the engine path and certify never name
/// (E2), in `toks`, the shipped tokens of one file. `io` names `std::io` allowed (certify's
/// telemetry writer, docs/CERTIFY.md §1).
fn io_violations(path: &str, toks: &[Tok], io: bool) -> Vec<String> {
    const STATE: [&str; 9] = [
        "thread_local",
        "Rc",
        "RefCell",
        "Cell",
        "OnceCell",
        "OnceLock",
        "LazyLock",
        "Mutex",
        "RwLock",
    ];
    const STD: [&str; 7] = ["fs", "time", "thread", "env", "process", "net", "io"];
    const PRINT: [&str; 5] = ["println", "print", "eprintln", "eprint", "dbg"];
    let banned = |m: &str| STD.contains(&m) && !(io && m == "io");
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let next = |k: usize| toks.get(i + k);
        let Tok::Ident(s) = t else { continue };
        let bad = STATE.contains(&s.as_str())
            || s.starts_with("Atomic")
            || s == "atomic"
            || (s == "static" && next(1).is_some_and(|n| ident(n, "mut")))
            || (PRINT.contains(&s.as_str()) && next(1).is_some_and(|n| punct(n, '!')));
        if bad {
            found.push(format!("{path}: {s}"));
        }
        // std::fs and the rest, alone or inside a `std::{..}` group.
        if s == "std" && next(1).is_some_and(|n| punct(n, ':')) {
            match next(3) {
                Some(Tok::Ident(m)) if banned(m) => {
                    found.push(format!("{path}: std::{m}"));
                }
                Some(Tok::Punct('{')) => {
                    let mut depth = 0;
                    for u in &toks[i + 3..] {
                        match u {
                            Tok::Punct('{') => depth += 1,
                            Tok::Punct('}') => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            Tok::Ident(m) if depth == 1 && banned(m) => {
                                found.push(format!("{path}: std::{{{m}}}"));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// Certify's shipped sources, each file's tokens with its path (docs/CERTIFY.md §1, §13).
fn certify_tokens() -> Vec<(String, Vec<Tok>)> {
    sources("certify")
        .into_iter()
        .map(|(path, text)| (path, tokens(&shipped(&strip(&text)))))
        .collect()
}

#[test]
fn certify_holds_no_threshold() {
    // R4 on certify (docs/CERTIFY.md §1): every bar a verdict compares with is in a dated
    // criteria file, so certify's shipped code spells no float but 0.0 and 1.0, names no float
    // constant, and makes no integer but 0 and 1 float. July had 13 certification thresholds in
    // code (ledger 3, verdict 6, invariants 4).
    let fixture = tokens(&shipped(&strip(
        "const PIN_SPREAD: f64 = 1e-12; let s = x < f64::EPSILON; let n = 32 as f64;",
    )));
    assert_eq!(float_violations(&fixture, false).len(), 2);
    assert_eq!(int_float_violations(&fixture, &[]).len(), 1);
    let mut found = Vec::new();
    for (path, toks) in certify_tokens() {
        for v in float_violations(&toks, false) {
            found.push(format!("{path}: {v}"));
        }
        for v in int_float_violations(&toks, &[]) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(found.is_empty(), "thresholds in certify's code: {found:#?}");
}

#[test]
fn certify_does_no_file_io() {
    // E2's rule, carried to certify (docs/CERTIFY.md §1): no global state, no print macro, and
    // no std::fs, env, process, net, thread or time anywhere; only the telemetry writer names
    // std::io or parquet. Paths live in the cli.
    let fixture = tokens(&shipped(&strip(
        "use std::{fs, io::Write}; let t = std::time::Instant::now(); println!(\"x\"); \
         let w: parquet::W = todo!();",
    )));
    assert_eq!(io_violations("fixture", &fixture, false).len(), 4);
    assert_eq!(io_violations("fixture", &fixture, true).len(), 3);
    let mut found = Vec::new();
    for (path, toks) in certify_tokens() {
        let telemetry = path
            .replace('\\', "/")
            .ends_with("certify/src/telemetry.rs");
        found.extend(io_violations(&path, &toks, telemetry));
        if !telemetry && toks.iter().any(|t| ident(t, "parquet")) {
            found.push(format!("{path}: parquet"));
        }
    }
    assert!(
        found.is_empty(),
        "I/O or global state in certify: {found:#?}"
    );
}

/// What in `toks` hands out core's writer (E1): a re-export of core or of the engine (whose
/// prelude holds core's types), a call to core's writer (`apply`, `resolve`, `Ledger`,
/// `RunLedger`, `Checkpoint::of`), or a public function that returns a `Checkpoint`, `Sim` or
/// `SimState`.
fn writer_violations(toks: &[Tok]) -> Vec<String> {
    const WRITER: [&str; 4] = ["apply", "resolve", "Ledger", "RunLedger"];
    const MADE: [&str; 3] = ["Checkpoint", "Sim", "SimState"];
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let at = |k: usize| toks.get(i + k);
        let Tok::Ident(s) = t else { continue };
        if WRITER.contains(&s.as_str()) {
            found.push(s.clone());
        }
        if s == "Checkpoint"
            && at(1).is_some_and(|n| punct(n, ':'))
            && at(3).is_some_and(|n| ident(n, "of"))
        {
            found.push("Checkpoint::of".to_string());
        }
        let public = s == "pub" && !at(1).is_some_and(|n| punct(n, '('));
        if public
            && (at(1).is_some_and(|n| ident(n, "use")) || at(1).is_some_and(|n| ident(n, "extern")))
        {
            let names_core = toks[i..]
                .iter()
                .take_while(|u| !punct(u, ';'))
                .any(|u| ident(u, "rustyecon_core") || ident(u, "rustyecon_engine"));
            if names_core {
                found.push("a re-export of core or the engine".to_string());
            }
        }
        if public && at(1).is_some_and(|n| ident(n, "fn")) {
            // The return type: from `->` to the body or `;`, outside the parameters.
            let mut depth = 0i32;
            let mut ret = false;
            for u in &toks[i..] {
                match u {
                    Tok::Punct('(' | '[' | '<') if !ret => depth += 1,
                    Tok::Punct(')' | ']') if !ret => depth -= 1,
                    Tok::Punct('>') if !ret => depth -= 1,
                    Tok::Punct('-') if depth == 0 => ret = true,
                    Tok::Punct('{' | ';') if depth == 0 => break,
                    Tok::Ident(n) if ret && MADE.contains(&n.as_str()) => {
                        found.push(format!("a public fn returning {n}"));
                    }
                    _ => {}
                }
            }
        }
    }
    found
}

#[test]
fn certify_hands_out_no_core_writer() {
    // E1 on certify (docs/CERTIFY.md §1): it depends on core for read-only helpers, re-exports
    // nothing of core, calls none of core's writer, and no public function returns a
    // Checkpoint, Sim or SimState it built. The scanner is checked on fixtures first.
    for (src, n) in [
        ("pub use rustyecon_core::Tape;", 1),
        ("pub use rustyecon_engine::prelude::*;", 1),
        ("let x = rustyecon_core::apply(s, w, p, d, l);", 1),
        ("let (w, s) = resolve(t)?; let l = Ledger::open(&s, &w);", 2),
        ("let cp = Checkpoint::of(&w, s, r)?;", 1),
        (
            "pub fn kicked(&self) -> Result<Checkpoint, String> { todo!() }",
            1,
        ),
        ("pub fn run(t: &Tape) -> Sim { todo!() }", 1),
        (
            "pub fn take(cp: &Checkpoint, f: fn(u8) -> u8) -> u64 { 0 }",
            0,
        ),
        ("pub(crate) fn inner() -> Sim { todo!() }", 0),
        ("use rustyecon_core::Basis;", 0),
    ] {
        let toks = tokens(&shipped(&strip(src)));
        assert_eq!(writer_violations(&toks).len(), n, "{src}");
    }
    let mut found = Vec::new();
    for (path, toks) in certify_tokens() {
        for v in writer_violations(&toks) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(
        found.is_empty(),
        "certify hands out core's writer: {found:#?}"
    );
}
