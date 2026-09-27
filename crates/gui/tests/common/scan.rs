//! Source scans of the GUI (docs/GUI.md §8.1, U9, D12): the engine's lexer, copied from
//! `crates/engine/tests/common/scan.rs` (`strip`, `shipped`, `Tok`, `tokens`), and the GUI's own
//! list of what to read. The copy is tested in `scans.rs` as the engine tests its own.

use std::path::{Path, PathBuf};

/// This crate's `src`.
pub fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The egui-free modules (docs/GUI.md §3.2): directories under `src`. `edit` arrives at G0.2.
pub const EGUI_FREE: [&str; 6] = ["model", "run", "edit", "vm", "drive", "platform"];

/// Every `.rs` file under `dir`, with its path relative to `src`, sorted. A directory that does
/// not exist yet gives none.
pub fn sources_under(dir: &Path) -> Vec<(String, String)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = read.map(|e| e.expect("an entry").path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(root, &p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&p).expect("a source file");
                let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                out.push((rel.replace('\\', "/"), text));
            }
        }
    }
    let mut out = Vec::new();
    walk(&src(), dir, &mut out);
    out
}

/// Every source file of the egui-free modules, with its path relative to `src`.
pub fn egui_free_sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for m in EGUI_FREE {
        out.extend(sources_under(&src().join(m)));
    }
    out
}

/// Every source file of this crate, with its path relative to `src`.
pub fn all_sources() -> Vec<(String, String)> {
    sources_under(&src())
}

/// The shipped tokens of each of `files`.
pub fn shipped_tokens(files: &[(String, String)]) -> Vec<(String, Vec<Tok>)> {
    files
        .iter()
        .map(|(p, text)| (p.clone(), tokens(&shipped(&strip(text)))))
        .collect()
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
