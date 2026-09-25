//! Source scans of the engine path, core, markets, agents and engine (docs/ENGINE.md §9 and
//! §11): they read the shipped sources (everything before a file's first `#[cfg(test)]`) with
//! comments stripped and literal contents blanked, and back up clippy.toml's lists (A5, R8, R4,
//! E2).

mod common;

use common::scan::{engine_path_tokens, shipped, strip, tokens, Tok};

fn ident(t: &Tok, name: &str) -> bool {
    matches!(t, Tok::Ident(s) if s == name)
}

fn punct(t: &Tok, ch: char) -> bool {
    matches!(t, Tok::Punct(c) if *c == ch)
}

#[test]
fn the_scanner_reads_what_it_should() {
    // The scans are only as good as the lexer: comments and strings are not code, tuple
    // indices and ranges are not floats, and every float spelling is one.
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
        let h = 9.5;
    "##;
    let stripped = strip(src);
    assert!(!stripped.contains("2.5") && !stripped.contains("3.5") && !stripped.contains("5.5"));
    assert!(!stripped.contains("6.5") && !stripped.contains("7.5") && !stripped.contains('8'));
    let floats: Vec<String> = tokens(shipped(&stripped))
        .into_iter()
        .filter_map(|t| match t {
            Tok::Num(s, true) => Some(s),
            _ => None,
        })
        .collect();
    assert_eq!(floats, ["1.0", "0.0", "2.", "1e-12", "3"]);
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

#[test]
fn no_behavioural_float_literals() {
    // R4: every behavioural number comes from the tape. Shipped code may spell only 0.0 and 1.0
    // (the additive and multiplicative identities, a fill's cap, a price by definition).
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
        for t in &toks {
            if let Tok::Num(s, true) = t {
                if s != "0.0" && s != "1.0" {
                    found.push(format!("{path}: {s}"));
                }
            }
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
    let mut found = Vec::new();
    for (path, toks) in engine_path_tokens() {
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
                    Some(Tok::Ident(m)) if STD.contains(&m.as_str()) => {
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
                                Tok::Ident(m) if depth == 1 && STD.contains(&m.as_str()) => {
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
    }
    assert!(
        found.is_empty(),
        "I/O or global state on the engine path: {found:#?}"
    );
}
