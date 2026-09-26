//! Source scans: read the sources of the engine path with comments stripped and literal
//! contents blanked, and split them into tokens.

/// The engine path's crates, whose sources the scans read (docs/ENGINE.md §11).
pub const ENGINE_PATH: [&str; 4] = ["core", "markets", "agents", "engine"];

/// Every `.rs` file under `crates/<name>/src`, with its path, sorted.
pub fn sources(name: &str) -> Vec<(String, String)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("a source directory")
            .map(|e| e.expect("an entry").path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&p).expect("a source file");
                out.push((p.display().to_string(), text));
            }
        }
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(name)
        .join("src");
    let mut out = Vec::new();
    walk(&dir, &mut out);
    assert!(!out.is_empty(), "no sources under {}", dir.display());
    out
}

/// Rust source with comments removed and the contents of string and character literals
/// blanked, so that neither a comment citing §2.4 nor a message quoting a number reads as code.
pub fn strip(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let n = c.len();
    let at = |i: usize| if i < n { c[i] } else { '\0' };
    let ident = |ch: char| ch.is_alphanumeric() || ch == '_';
    let raw_open = |i: usize| -> Option<(usize, usize)> {
        // A raw string r"..", r#".."# or br"..": the index of its opening quote and its hashes.
        let mut j = match (at(i), at(i + 1)) {
            ('r', _) => i + 1,
            ('b', 'r') => i + 2,
            _ => return None,
        };
        let mut hashes = 0;
        while at(j) == '#' {
            hashes += 1;
            j += 1;
        }
        (at(j) == '"').then_some((j, hashes))
    };
    let mut out = String::with_capacity(n);
    let mut i = 0;
    while i < n {
        let (a, b) = (c[i], at(i + 1));
        let prev = if i > 0 { c[i - 1] } else { ' ' };
        if a == '/' && b == '/' {
            while i < n && c[i] != '\n' {
                i += 1;
            }
        } else if a == '/' && b == '*' {
            let mut depth = 1;
            i += 2;
            while i < n && depth > 0 {
                if c[i] == '/' && at(i + 1) == '*' {
                    depth += 1;
                    i += 2;
                } else if c[i] == '*' && at(i + 1) == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if c[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
        } else if let Some((open, hashes)) = raw_open(i).filter(|_| !ident(prev)) {
            let mut j = open + 1;
            while j < n && !(c[j] == '"' && (1..=hashes).all(|k| at(j + k) == '#')) {
                j += 1;
            }
            out.push_str("\"\"");
            i = j + 1 + hashes;
        } else if a == '"' || (a == 'b' && b == '"' && !ident(prev)) {
            let mut j = if a == 'b' { i + 2 } else { i + 1 };
            while j < n && c[j] != '"' {
                if c[j] == '\\' {
                    j += 1;
                }
                j += 1;
            }
            out.push_str("\"\"");
            i = j + 1;
        } else if a == '\'' && (b == '\\' || at(i + 2) == '\'') {
            // A character literal; a lifetime has no closing quote two places on.
            let mut j = i + 1;
            if c[j] == '\\' {
                j += 2;
                while j < n && c[j] != '\'' {
                    j += 1;
                }
            } else {
                j += 1;
            }
            out.push_str("' '");
            i = j + 1;
        } else {
            out.push(a);
            i += 1;
        }
    }
    out
}

/// The code that ships: stripped source with every `#[cfg(test)]` item removed, the attribute
/// and the item or statement after it. The item ends at the `;` or the closing `}` that brings
/// its bracket depth back to 0, so shipped code after an early test item (a test-only helper
/// method, say) is still scanned.
pub fn shipped(stripped: &str) -> String {
    const ATTR: &str = "#[cfg(test)]";
    let mut out = String::with_capacity(stripped.len());
    let mut rest = stripped;
    while let Some(k) = rest.find(ATTR) {
        out.push_str(&rest[..k]);
        let item = &rest[k + ATTR.len()..];
        let mut depth = 0usize;
        let mut end = item.len();
        for (i, ch) in item.char_indices() {
            match ch {
                '{' | '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                ';' if depth == 0 => {
                    end = i + 1;
                    break;
                }
                _ => {}
            }
        }
        rest = &item[end..];
    }
    out.push_str(rest);
    out
}

/// Float literals other than `0.0` and `1.0`, and named float constants (`EPSILON`,
/// `MIN_POSITIVE`, an `f64::` or `f32::` associated constant, anything under `consts`), in
/// shipped tokens (R4, A12). `num` is whether the file is `core::num`, the one module that may
/// name the float limits it searches between.
pub fn float_violations(toks: &[Tok], num: bool) -> Vec<String> {
    let ident = |i: usize| match toks.get(i) {
        Some(Tok::Ident(s)) => Some(s.as_str()),
        _ => None,
    };
    let path = |i: usize| {
        matches!(toks.get(i), Some(Tok::Punct(':')))
            && matches!(toks.get(i + 1), Some(Tok::Punct(':')))
    };
    let float_type = |s: Option<&str>| matches!(s, Some("f64" | "f32"));
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        match t {
            Tok::Num(s, true) if s != "0.0" && s != "1.0" => found.push(s.clone()),
            Tok::Ident(s) if !num => {
                // `f64::NAME`, a constant (all capitals) of a float type.
                let constant = ident(i + 3)
                    .is_some_and(|c| c.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_'));
                let assoc = float_type(Some(s)) && path(i + 1) && constant;
                // A named constant reached another way (imported, or under `consts`), unless
                // it was the `NAME` of an `f64::NAME` just counted.
                let after_float = i >= 3 && path(i - 2) && float_type(ident(i - 3));
                let named = ["EPSILON", "MIN_POSITIVE", "consts"].contains(&s.as_str())
                    && !(after_float && s != "consts");
                if assoc || named {
                    found.push(format!("{s} {:?}", ident(i + 3)));
                }
            }
            _ => {}
        }
    }
    found
}

/// Integer numbers made float in shipped tokens (R4, A12; amended at P0.9, O13): a behavioural
/// number spelled as an integer and converted is as much a literal as `1e-9`. Flags an integer
/// literal other than `0` and `1`, or a named integer constant (a name in capitals, such as
/// `WEEKS`), that reaches a float through `as f64` or `as f32` (bare or inside the parentheses
/// before it), `f64::from(..)` or `f32::from(..)`, or `.into()`; any literal but `0` in
/// `from_bits(..)`, a float spelled as its bits; and a string parsed as a float, `parse::<f64>`.
/// An integer type's own limits (`u32::MAX`) are not behavioural, and `allowed` names the
/// constants that may be converted (clock.rs's calendar constants).
pub fn int_float_violations(toks: &[Tok], allowed: &[&str]) -> Vec<String> {
    const INTS: [&str; 12] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    ];
    let ident = |i: usize| match toks.get(i) {
        Some(Tok::Ident(s)) => Some(s.as_str()),
        _ => None,
    };
    let punct = |i: usize, c: char| matches!(toks.get(i), Some(Tok::Punct(p)) if *p == c);
    let float = |i: usize| matches!(ident(i), Some("f64" | "f32"));
    let constant = |s: &str| {
        s.len() > 1
            && s.chars().any(|c| c.is_ascii_uppercase())
            && s.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    };
    // What in `toks[lo..hi]` is a behavioural integer: a literal but 0 and 1, or a constant that
    // is neither allowed nor an integer type's limit.
    let offenders = |lo: usize, hi: usize| -> Vec<String> {
        let mut out = Vec::new();
        for (k, t) in toks.iter().enumerate().take(hi).skip(lo) {
            match t {
                Tok::Num(s, false) if s != "0" && s != "1" => out.push(s.clone()),
                Tok::Ident(s) if constant(s) && !allowed.contains(&s.as_str()) => {
                    let limit = k >= 3
                        && punct(k - 1, ':')
                        && punct(k - 2, ':')
                        && ident(k - 3).is_some_and(|t| INTS.contains(&t));
                    if !limit {
                        out.push(s.clone());
                    }
                }
                _ => {}
            }
        }
        out
    };
    // The index after the bracket that closes the one opening at `open`.
    let close = |open: usize| {
        let mut depth = 0i32;
        for (k, t) in toks.iter().enumerate().skip(open) {
            match t {
                Tok::Punct('(') => depth += 1,
                Tok::Punct(')') => {
                    depth -= 1;
                    if depth == 0 {
                        return k;
                    }
                }
                _ => {}
            }
        }
        toks.len()
    };
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let hits = match t {
            // `x as f64`: x a literal or constant, or a parenthesised expression holding one.
            Tok::Ident(s) if s == "as" && float(i + 1) && i > 0 => {
                if punct(i - 1, ')') {
                    let mut depth = 0i32;
                    let mut open = 0;
                    for (k, t) in toks[..i].iter().enumerate().rev() {
                        match t {
                            Tok::Punct(')') => depth += 1,
                            Tok::Punct('(') => {
                                depth -= 1;
                                if depth == 0 {
                                    open = k;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    offenders(open, i)
                } else {
                    offenders(i - 1, i)
                }
            }
            // `f64::from(..)`.
            Tok::Ident(s)
                if (s == "f64" || s == "f32")
                    && punct(i + 1, ':')
                    && punct(i + 2, ':')
                    && ident(i + 3) == Some("from")
                    && punct(i + 4, '(') =>
            {
                offenders(i + 4, close(i + 4))
            }
            // `2u8.into()`.
            Tok::Num(_, false) if punct(i + 1, '.') && ident(i + 2) == Some("into") => {
                offenders(i, i + 1)
            }
            // `from_bits(..)`: any literal but 0.
            Tok::Ident(s) if s == "from_bits" && punct(i + 1, '(') => {
                let end = close(i + 1);
                toks[i + 1..end]
                    .iter()
                    .filter_map(|t| match t {
                        Tok::Num(n, _) if n != "0" => Some(n.clone()),
                        _ => None,
                    })
                    .collect()
            }
            // `parse::<f64>()`.
            Tok::Ident(s)
                if s == "parse" && punct(i + 1, ':') && punct(i + 3, '<') && float(i + 4) =>
            {
                vec!["parse::<float>".to_string()]
            }
            _ => Vec::new(),
        };
        for h in hits {
            found.push(format!("{h} made float at token {i}"));
        }
    }
    found
}

/// Core's read-only items, the ones the engine's prelude re-exports. A `pub use` of anything
/// else from core is flagged, so the list is an allow-list: a new type joins the prelude only
/// by joining it too (E1; amended at P0.9, O11). `ClockMethod` and `Site` joined at S2.2.
pub const CORE_READ_ONLY: [&str; 31] = [
    "ActorId",
    "ActorKind",
    "Amount",
    "Breach",
    "CheckpointError",
    "ClassId",
    "Clock",
    "ClockMethod",
    "CoreError",
    "Date",
    "DeskId",
    "EventId",
    "GoodId",
    "Holder",
    "Inventory",
    "Key",
    "Life",
    "LoadError",
    "LoadErrorKind",
    "Lot",
    "NodeId",
    "ParamId",
    "Phase",
    "PopId",
    "Provenance",
    "RunAudit",
    "ShortfallLine",
    "SimState",
    "Site",
    "StateDelta",
    "TickAudit",
];

/// The public re-exports in `code` (shipped source) that would hand a frontend core's writer
/// (E1): core itself, whole, by glob, by `self` in a group, or under any name; `pub extern
/// crate` of it; or any item of core but [`CORE_READ_ONLY`] (a module path, `apply` and
/// `Ledger` included). A private alias of core (`use rustyecon_core as c;`) counts as core, and
/// a leading `::` is ignored. Returns each offending statement, whitespace collapsed, and counts
/// the statements read in `seen`.
pub fn writer_reexports(code: &str, seen: &mut usize) -> Vec<String> {
    let collapse = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let words = |s: &str| -> Vec<String> {
        s.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|w| !w.is_empty())
            .map(str::to_string)
            .collect()
    };
    // Every `use` and `extern crate` statement, collapsed, with whether it is public.
    let mut stmts = Vec::new();
    for (i, _) in code
        .match_indices("use ")
        .chain(code.match_indices("extern crate "))
    {
        let before = code[..i].trim_end();
        if i > 0 && code[..i].ends_with(|c: char| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let public = before.ends_with("pub");
        let end = code[i..].find(';').map_or(code.len(), |e| i + e);
        stmts.push((i, public, collapse(&code[i..end])));
    }
    stmts.sort();
    // The names core goes by here: its own, and any alias.
    let mut core = vec!["rustyecon_core".to_string()];
    for (_, _, s) in &stmts {
        let body = s
            .trim_start_matches("use ")
            .trim_start_matches("extern crate ");
        let body = body.trim_start_matches("::");
        if let Some(alias) = body.strip_prefix("rustyecon_core as ") {
            core.push(alias.trim().to_string());
        }
    }
    let mut found = Vec::new();
    for (_, public, s) in stmts {
        if !public {
            continue;
        }
        *seen += 1;
        let stmt = format!("pub {s}");
        let crate_stmt = s.starts_with("extern crate ");
        let body = s
            .trim_start_matches("use ")
            .trim_start_matches("extern crate ");
        let body = body.trim_start_matches("::");
        let Some(first) = words(body).into_iter().next() else {
            continue;
        };
        if !core.contains(&first) {
            continue;
        }
        let at = body.find(first.as_str()).unwrap_or(0);
        let tail = &body[at + first.len()..];
        let tail_words = words(tail);
        // Words right after `as` name the re-export; they are not items of core.
        let named: Vec<&String> = tail_words
            .iter()
            .enumerate()
            .filter(|&(k, w)| w != "as" && (k == 0 || tail_words[k - 1] != "as"))
            .map(|(_, w)| w)
            .collect();
        let whole = crate_stmt
            || tail.trim().is_empty()
            || tail.trim_start().starts_with("as ")
            || tail.contains('*')
            || named.iter().any(|w| *w == "self");
        if whole || named.iter().any(|w| !CORE_READ_ONLY.contains(&w.as_str())) {
            found.push(stmt);
        }
    }
    found
}

/// Types a frontend holds that must not hand it a writer (E1, E4): `&mut` to any of them in a
/// public signature is a way to change a run.
pub const GUARDED: [&str; 5] = ["Sim", "Checkpoint", "SimState", "World", "Inventory"];

/// Trait impls that hand out `&mut` to what they are implemented for.
pub const MUT_TRAITS: [&str; 4] = ["DerefMut", "AsMut", "BorrowMut", "IndexMut"];

/// What a signature (its tokens, up to its body) hands out.
struct Muts {
    /// `&mut self`, `&'a mut self` or `self: &mut Self`.
    mut_self: bool,
    /// `&mut` to a type naming a guarded type anywhere in it (inside a generic, a slice or a
    /// closure's parameters too).
    guarded: bool,
    /// `mut` anywhere in the return type.
    returns_mut: bool,
}

fn muts(sig: &[Tok]) -> Muts {
    let mut m = Muts {
        mut_self: false,
        guarded: false,
        returns_mut: false,
    };
    let punct = |i: usize, c: char| matches!(sig.get(i), Some(Tok::Punct(p)) if *p == c);
    let ident = |i: usize| match sig.get(i) {
        Some(Tok::Ident(s)) => Some(s.as_str()),
        _ => None,
    };
    for i in 0..sig.len() {
        if !punct(i, '&') {
            continue;
        }
        // `&mut`, or `&'a mut`.
        let mut k = i + 1;
        if punct(k, '\'') {
            k += 2;
        }
        if ident(k) != Some("mut") {
            continue;
        }
        if matches!(ident(k + 1), Some("self" | "Self")) {
            m.mut_self = true;
            continue;
        }
        // The referenced type: up to a `,`, `;`, `=` or `{` at its own depth, or the bracket
        // that closes around it.
        let mut depth = 0i32;
        let mut j = k + 1;
        while j < sig.len() {
            match &sig[j] {
                Tok::Punct('(' | '[' | '<') => depth += 1,
                Tok::Punct('>') if !punct(j - 1, '-') => depth -= 1,
                Tok::Punct(')' | ']') => depth -= 1,
                Tok::Punct(',' | ';' | '=' | '{') if depth == 0 => break,
                Tok::Ident(s) if GUARDED.contains(&s.as_str()) => m.guarded = true,
                _ => {}
            }
            if depth < 0 {
                break;
            }
            j += 1;
        }
    }
    // The return type: what follows a `->` outside every bracket of the parameters.
    let mut depth = 0i32;
    for (i, t) in sig.iter().enumerate() {
        match t {
            Tok::Punct('(' | '[') => depth += 1,
            Tok::Punct(')' | ']') => depth -= 1,
            Tok::Punct('<') => depth += 1,
            Tok::Punct('>') if i > 0 && punct(i - 1, '-') => {
                if depth == 0 {
                    m.returns_mut = sig[i + 1..]
                        .iter()
                        .any(|t| matches!(t, Tok::Ident(s) if s == "mut"));
                    break;
                }
            }
            Tok::Punct('>') => depth -= 1,
            _ => {}
        }
    }
    m
}

/// Every `fn` in `toks`: whether it is `pub` (not `pub(crate)`), its name, and its signature's
/// tokens, up to its body or `;`.
fn fns(toks: &[Tok]) -> Vec<(bool, String, &[Tok])> {
    const QUALIFIERS: [&str; 4] = ["const", "unsafe", "async", "extern"];
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if !matches!(t, Tok::Ident(s) if s == "fn") {
            continue;
        }
        let Some(Tok::Ident(name)) = toks.get(i + 1) else {
            continue;
        };
        // Back over qualifiers (and an `extern "C"`'s blanked string) to a `pub`.
        let mut b = i;
        while b > 0 {
            match &toks[b - 1] {
                Tok::Ident(s) if QUALIFIERS.contains(&s.as_str()) => b -= 1,
                Tok::Punct('"') => b -= 1,
                _ => break,
            }
        }
        let public = b > 0 && matches!(&toks[b - 1], Tok::Ident(s) if s == "pub");
        let mut depth = 0i32;
        let mut end = toks.len();
        for (j, t) in toks.iter().enumerate().skip(i) {
            match t {
                Tok::Punct('(' | '[') => depth += 1,
                Tok::Punct(')' | ']') => depth -= 1,
                Tok::Punct('{' | ';') if depth == 0 => {
                    end = j;
                    break;
                }
                _ => {}
            }
        }
        out.push((public, name.clone(), &toks[i..end]));
    }
    out
}

fn text(toks: &[Tok]) -> String {
    let mut s = String::new();
    for t in toks {
        match t {
            Tok::Ident(w) | Tok::Num(w, _) => {
                if s.ends_with(|c: char| c.is_alphanumeric() || c == '_') {
                    s.push(' ');
                }
                s.push_str(w);
            }
            Tok::Punct(c) => s.push(*c),
        }
    }
    s
}

/// Public functions in `code` (shipped source) that could hand a frontend a way to change a
/// run (E1, E4): a `&mut self` method outside `allowed`, `&mut` to a [`GUARDED`] type anywhere
/// in a signature (a free `fn(&mut Sim)`, a closure's parameters and where clauses included), or
/// `&mut` anything in a return type. Returns the offending signatures, and counts the public
/// functions read in `public`.
pub fn api_violations(code: &str, allowed: &[&str], public: &mut usize) -> Vec<String> {
    let toks = tokens(code);
    let mut found = Vec::new();
    for (is_pub, name, sig) in fns(&toks) {
        if !is_pub {
            continue;
        }
        *public += 1;
        let m = muts(sig);
        let mut bad = Vec::new();
        if m.mut_self && !allowed.contains(&name.as_str()) {
            bad.push("&mut self");
        }
        if m.guarded {
            bad.push("&mut to a run's state, world or Sim");
        }
        if m.returns_mut {
            bad.push("&mut in the return type");
        }
        if !bad.is_empty() {
            found.push(format!("pub {}: {bad:?}", text(sig)));
        }
    }
    found
}

/// The `impl` blocks in `code` (shipped source) for the types in `types`, each with the `&mut
/// self` methods it may have, that could hand a frontend a writer (E1, E4): a trait impl of one
/// of [`MUT_TRAITS`], a trait method with `&mut self`, `&mut` a [`GUARDED`] type or `&mut` in its
/// return type (a trait's methods are as public as the trait), or an inherent `pub fn` that
/// [`api_violations`] would flag. Returns the offenders, and counts the impls of those types
/// read in `seen`.
pub fn impl_violations(code: &str, types: &[(&str, &[&str])], seen: &mut usize) -> Vec<String> {
    let toks = tokens(code);
    let mut found = Vec::new();
    let item_start = |i: usize| {
        i == 0
            || matches!(&toks[i - 1], Tok::Punct('{' | '}' | ';' | ']'))
            || matches!(&toks[i - 1], Tok::Ident(s) if s == "unsafe" || s == "default")
    };
    // The last identifier of a path, before its generic arguments.
    let last_name = |ts: &[Tok]| -> Option<String> {
        let mut depth = 0i32;
        let mut name = None;
        for t in ts {
            match t {
                Tok::Punct('<') => depth += 1,
                Tok::Punct('>') => depth -= 1,
                Tok::Ident(s) if depth == 0 && s != "dyn" && s != "mut" => name = Some(s.clone()),
                _ => {}
            }
        }
        name
    };
    let mut i = 0;
    while i < toks.len() {
        if !(matches!(&toks[i], Tok::Ident(s) if s == "impl") && item_start(i)) {
            i += 1;
            continue;
        }
        // The impl's own generics.
        let mut j = i + 1;
        if matches!(toks.get(j), Some(Tok::Punct('<'))) {
            let mut depth = 0i32;
            while j < toks.len() {
                match &toks[j] {
                    Tok::Punct('<') => depth += 1,
                    Tok::Punct('>') => depth -= 1,
                    _ => {}
                }
                j += 1;
                if depth == 0 {
                    break;
                }
            }
        }
        // The header, up to the body's `{`; a `for` outside every `<>` splits trait and type.
        let start = j;
        let mut depth = 0i32;
        let mut split = None;
        while j < toks.len() && !(matches!(&toks[j], Tok::Punct('{')) && depth == 0) {
            match &toks[j] {
                Tok::Punct('<') => depth += 1,
                Tok::Punct('>') => depth -= 1,
                Tok::Ident(s) if s == "for" && depth == 0 => split = Some(j),
                Tok::Ident(s) if s == "where" && depth == 0 => break,
                _ => {}
            }
            j += 1;
        }
        let header_end = j;
        while j < toks.len() && !matches!(&toks[j], Tok::Punct('{')) {
            j += 1;
        }
        let (tr, ty) = match split {
            Some(k) => (
                last_name(&toks[start..k]),
                last_name(&toks[k + 1..header_end]),
            ),
            None => (None, last_name(&toks[start..header_end])),
        };
        // The body, to its matching `}`.
        let body_start = j + 1;
        let mut depth = 0i32;
        let mut end = toks.len();
        for (k, t) in toks.iter().enumerate().skip(j) {
            match t {
                Tok::Punct('{') => depth += 1,
                Tok::Punct('}') => {
                    depth -= 1;
                    if depth == 0 {
                        end = k;
                        break;
                    }
                }
                _ => {}
            }
        }
        let body = &toks[body_start.min(end)..end];
        let guarded = ty
            .as_deref()
            .and_then(|t| types.iter().find(|(name, _)| *name == t));
        if let (Some(&(name, allowed)), Some(ty)) = (guarded, ty.as_deref()) {
            *seen += 1;
            let head = text(&toks[i..header_end]);
            if let Some(tr) = &tr {
                if MUT_TRAITS.contains(&tr.as_str()) {
                    found.push(format!("{head}: a trait that hands out &mut {ty}"));
                }
            }
            for (is_pub, f, sig) in fns(body) {
                let m = muts(sig);
                let bad = if tr.is_some() {
                    m.mut_self || m.guarded || m.returns_mut
                } else {
                    is_pub
                        && ((m.mut_self && !allowed.contains(&f.as_str()))
                            || m.guarded
                            || m.returns_mut)
                };
                if bad {
                    found.push(format!("{head}: {} ({name})", text(sig)));
                }
            }
        }
        i = end.max(i + 1);
    }
    found
}

/// Public fields of the structs in `names`, in `code` (shipped source): a `pub` field, braced
/// or tuple, not `pub(crate)` (E1, E4). Returns `name.field`-like descriptions, and counts the
/// structs found in `seen`.
pub fn pub_fields(code: &str, names: &[&str], seen: &mut usize) -> Vec<String> {
    let toks = tokens(code);
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let is_struct = matches!(t, Tok::Ident(s) if s == "struct");
        let Some(Tok::Ident(name)) = toks.get(i + 1) else {
            continue;
        };
        if !is_struct || !names.contains(&name.as_str()) {
            continue;
        }
        *seen += 1;
        // Skip generics, then read the body at depth 1.
        let mut j = i + 2;
        let mut depth = 0i32;
        while j < toks.len() {
            match &toks[j] {
                Tok::Punct('<') => depth += 1,
                Tok::Punct('>') => depth -= 1,
                Tok::Punct('{' | '(') if depth == 0 => break,
                Tok::Punct(';') if depth == 0 => break,
                _ => {}
            }
            j += 1;
        }
        let mut depth = 0i32;
        for (k, t) in toks.iter().enumerate().skip(j) {
            match t {
                Tok::Punct('{' | '(' | '[' | '<') => depth += 1,
                Tok::Punct('}' | ')' | ']' | '>') => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Tok::Ident(s)
                    if s == "pub"
                        && depth == 1
                        && !matches!(toks.get(k + 1), Some(Tok::Punct('('))) =>
                {
                    found.push(format!("{name}: a pub field before {:?}", toks.get(k + 1)));
                }
                _ => {}
            }
        }
    }
    found
}

/// A token of stripped source.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// An identifier or keyword.
    Ident(String),
    /// A numeric literal: its text without underscores or suffix, and whether it is a float.
    Num(String, bool),
    /// Any other character.
    Punct(char),
}

/// Split stripped source into tokens. A number right after a `.` is a tuple index.
pub fn tokens(s: &str) -> Vec<Tok> {
    let c: Vec<char> = s.chars().collect();
    let n = c.len();
    let at = |i: usize| if i < n { c[i] } else { '\0' };
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < n && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Ident(c[start..i].iter().collect()));
        } else if ch.is_ascii_digit() {
            let mut text = String::new();
            if matches!(out.last(), Some(Tok::Punct('.'))) {
                while i < n && c[i].is_ascii_digit() {
                    text.push(c[i]);
                    i += 1;
                }
                out.push(Tok::Num(text, false));
                continue;
            }
            if ch == '0' && matches!(at(i + 1), 'x' | 'o' | 'b') {
                while i < n && (c[i].is_alphanumeric() || c[i] == '_') {
                    text.push(c[i]);
                    i += 1;
                }
                out.push(Tok::Num(text, false));
                continue;
            }
            let mut float = false;
            let digits = |i: &mut usize, text: &mut String| {
                while *i < n && (c[*i].is_ascii_digit() || c[*i] == '_') {
                    if c[*i] != '_' {
                        text.push(c[*i]);
                    }
                    *i += 1;
                }
            };
            digits(&mut i, &mut text);
            let next = at(i + 1);
            if at(i) == '.' && next.is_ascii_digit() {
                float = true;
                text.push('.');
                i += 1;
                digits(&mut i, &mut text);
            } else if at(i) == '.' && next != '.' && !(next.is_alphabetic() || next == '_') {
                float = true;
                text.push('.');
                i += 1;
            }
            let e = at(i + 1);
            if matches!(at(i), 'e' | 'E')
                && (e.is_ascii_digit() || (matches!(e, '+' | '-') && at(i + 2).is_ascii_digit()))
            {
                float = true;
                text.push('e');
                i += 1;
                if matches!(c[i], '+' | '-') {
                    text.push(c[i]);
                    i += 1;
                }
                digits(&mut i, &mut text);
            }
            let suffix_start = i;
            while i < n && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            if c[suffix_start..i].iter().find(|&&ch| ch != '_') == Some(&'f') {
                float = true;
            }
            out.push(Tok::Num(text, float));
        } else {
            out.push(Tok::Punct(ch));
            i += 1;
        }
    }
    out
}

/// The shipped tokens of every source file of the engine path, with the file's path.
pub fn engine_path_tokens() -> Vec<(String, Vec<Tok>)> {
    let mut out = Vec::new();
    for name in ENGINE_PATH {
        for (path, text) in sources(name) {
            out.push((path, tokens(&shipped(&strip(&text)))));
        }
    }
    out
}
