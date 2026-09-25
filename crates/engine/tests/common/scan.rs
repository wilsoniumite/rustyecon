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

/// The `pub use` statements in `code` (shipped source) that would hand a frontend core's
/// writer (E1): core itself, whole or by glob, under any name, or one of its writer items.
/// Returns each offending statement, whitespace collapsed, and counts the `pub use`
/// statements read in `seen`.
pub fn writer_reexports(code: &str, seen: &mut usize) -> Vec<String> {
    const WRITERS: [&str; 6] = ["apply", "resolve", "Ledger", "RunLedger", "Resolver", "Ext"];
    let mut found = Vec::new();
    let mut rest = code;
    while let Some(k) = rest.find("pub use ") {
        let end = rest[k..].find(';').map_or(rest.len(), |e| k + e);
        let stmt = rest[k..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        rest = &rest[end..];
        *seen += 1;
        let path = stmt.trim_start_matches("pub use ").trim();
        let Some(tail) = path.strip_prefix("rustyecon_core") else {
            continue;
        };
        let words: Vec<&str> = tail
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|w| !w.is_empty())
            .collect();
        let whole = tail.is_empty() || tail.starts_with(" as ") || tail.contains('*');
        if whole || WRITERS.iter().any(|w| words.contains(w)) {
            found.push(stmt);
        }
    }
    found
}

/// Public engine functions that could hand a frontend a way to change a run (E1, E4): a
/// `&mut self` method outside `allowed`, or `&mut` to a `SimState`, `World` or `Inventory`
/// anywhere in a signature (a closure's parameters and where clauses included), or `&mut`
/// anything in a return type. `code` is shipped source; returns the offending signatures, and
/// counts the public functions read in `public`.
pub fn api_violations(code: &str, allowed: &[&str], public: &mut usize) -> Vec<String> {
    const GUARDED: [&str; 3] = ["SimState", "World", "Inventory"];
    let mut found = Vec::new();
    let mut rest = code;
    while let Some(k) = rest.find("pub fn ") {
        let sig_end = rest[k..].find(['{', ';']).map_or(rest.len(), |e| k + e);
        let sig = &rest[k..sig_end];
        *public += 1;
        let toks = tokens(sig);
        let name = match toks.get(2) {
            Some(Tok::Ident(n)) => n.clone(),
            _ => String::new(),
        };
        let mut bad = Vec::new();
        for (i, t) in toks.iter().enumerate() {
            if !matches!(t, Tok::Punct('&')) {
                continue;
            }
            // `&mut`, or `&'a mut`.
            let mut m = i + 1;
            if matches!(toks.get(m), Some(Tok::Punct('\''))) {
                m += 2;
            }
            if !matches!(toks.get(m), Some(Tok::Ident(s)) if s == "mut") {
                continue;
            }
            // `&mut self` needs to be allowed; `&mut` a path ending in a guarded type is never.
            match toks.get(m + 1) {
                Some(Tok::Ident(s)) if s == "self" => {
                    if !allowed.contains(&name.as_str()) {
                        bad.push("&mut self");
                    }
                }
                _ => {
                    let mut j = m + 1;
                    let mut last = None;
                    while let Some(Tok::Ident(seg)) = toks.get(j) {
                        last = Some(seg.as_str());
                        if matches!(toks.get(j + 1), Some(Tok::Punct(':')))
                            && matches!(toks.get(j + 2), Some(Tok::Punct(':')))
                        {
                            j += 3;
                        } else {
                            break;
                        }
                    }
                    if last.is_some_and(|l| GUARDED.contains(&l)) {
                        bad.push("&mut to the state or world");
                    }
                }
            }
        }
        // The return type: whatever follows the parameter list's `->`, at bracket depth 0.
        let mut depth = 0i32;
        let mut ret = None;
        let chars: Vec<char> = sig.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            match ch {
                '(' | '[' | '<' => depth += 1,
                ')' | ']' => depth -= 1,
                '>' if i > 0 && chars[i - 1] == '-' => {
                    if depth == 0 {
                        ret = Some(chars[i + 1..].iter().collect::<String>());
                        break;
                    }
                }
                '>' => depth -= 1,
                _ => {}
            }
        }
        if ret.is_some_and(|r| r.contains("mut")) {
            bad.push("&mut in the return type");
        }
        if !bad.is_empty() {
            found.push(format!(
                "{}: {bad:?}",
                sig.split_whitespace().collect::<Vec<_>>().join(" ")
            ));
        }
        rest = &rest[sig_end..];
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
