//! f11: the Rust constants in goldens_1f.rs equal goldens/goldens_1f.txt, rounded to 20
//! significant digits (half up), and the two list the same goldens. goldens_1f.txt carries the
//! digests that goldens/generate_1f.py wrote into it, of generate_1f.py, of generate_1e.py,
//! generate_1d.py, generate_1c.py, generate_1b.py and generate.py (which it builds on and imports
//! for the nesting assertions) and of the goldens; they must match all seven files as they are
//! now. Only `generate_1f.py --check` proves the values are its output.

use std::collections::BTreeMap;

use crate::goldens_1f::TABLE_1F;
use crate::goldens_file::{fnv1a64, parse_decimal, round_half_up, SIGNIFICANT};

const GOLDENS_1F_TXT: &str = include_str!("../../goldens/goldens_1f.txt");

const GENERATE_1F_PY: &[u8] = include_bytes!("../../goldens/generate_1f.py");

const GENERATE_1E_PY: &[u8] = include_bytes!("../../goldens/generate_1e.py");

const GENERATE_1D_PY: &[u8] = include_bytes!("../../goldens/generate_1d.py");

const GENERATE_1C_PY: &[u8] = include_bytes!("../../goldens/generate_1c.py");

const GENERATE_1B_PY: &[u8] = include_bytes!("../../goldens/generate_1b.py");

const GENERATE_PY: &[u8] = include_bytes!("../../goldens/generate.py");

fn goldens_1f_txt() -> BTreeMap<&'static str, &'static str> {
    let mut map = BTreeMap::new();
    for line in GOLDENS_1F_TXT.lines() {
        let body = line.split('#').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let (key, value) = body.split_once('=').expect("KEY = value");
        assert!(
            map.insert(key.trim(), value.trim()).is_none(),
            "duplicate {key}"
        );
    }
    map
}

/// The digest on the header line `# fnv1a64 <label> = <hex>`, and the byte offset just
/// past that line.
fn digest_line(label: &str) -> (u64, usize) {
    let prefix = format!("# fnv1a64 {label} = ");
    let start = GOLDENS_1F_TXT
        .find(&prefix)
        .unwrap_or_else(|| panic!("goldens_1f.txt has no {prefix:?} line"));
    let rest = &GOLDENS_1F_TXT[start + prefix.len()..];
    let hex = rest.split(['\r', '\n']).next().unwrap_or("");
    let value = u64::from_str_radix(hex, 16).unwrap_or_else(|_| panic!("bad digest {hex:?}"));
    let end = start + GOLDENS_1F_TXT[start..].find('\n').expect("a newline") + 1;
    (value, end)
}

#[test]
fn constants_match_goldens_1f_txt() {
    let file = goldens_1f_txt();
    let mut seen = BTreeMap::new();
    for &(name, literal) in TABLE_1F {
        assert!(seen.insert(name, ()).is_none(), "{name} declared twice");
        let value = file
            .get(name)
            .unwrap_or_else(|| panic!("{name} is not in goldens_1f.txt"));
        let want = round_half_up(parse_decimal(value), SIGNIFICANT);
        assert_eq!(
            parse_decimal(literal),
            want,
            "{name}: {literal} vs goldens_1f.txt {value}"
        );
    }
    for name in file.keys() {
        assert!(
            seen.contains_key(name),
            "{name} is in goldens_1f.txt but not in goldens_1f.rs"
        );
    }
    assert_eq!(TABLE_1F.len(), 235);
}

#[test]
fn goldens_1f_txt_is_from_generate_1f_py() {
    for (label, bytes, advice) in [
        (
            "generate_1f.py",
            GENERATE_1F_PY,
            "generate_1f.py changed since it wrote goldens_1f.txt: rerun it, then --check",
        ),
        (
            "generate_1e.py",
            GENERATE_1E_PY,
            "generate_1e.py changed since generate_1f.py wrote goldens_1f.txt: rerun it",
        ),
        (
            "generate_1d.py",
            GENERATE_1D_PY,
            "generate_1d.py changed since generate_1f.py wrote goldens_1f.txt: rerun it",
        ),
        (
            "generate_1c.py",
            GENERATE_1C_PY,
            "generate_1c.py changed since generate_1f.py wrote goldens_1f.txt: rerun it",
        ),
        (
            "generate_1b.py",
            GENERATE_1B_PY,
            "generate_1b.py changed since generate_1f.py wrote goldens_1f.txt: rerun it",
        ),
        (
            "generate.py",
            GENERATE_PY,
            "generate.py changed since generate_1f.py wrote goldens_1f.txt: rerun it",
        ),
    ] {
        let (digest, _) = digest_line(label);
        assert_eq!(fnv1a64(bytes), digest, "{advice}");
    }
    // The body is everything after the last digest line.
    let (body, end) = digest_line("body");
    let text = &GOLDENS_1F_TXT.as_bytes()[end..];
    assert_eq!(
        fnv1a64(text),
        body,
        "goldens_1f.txt was edited after generate_1f.py wrote it"
    );
    // The digest has teeth: one changed digit changes it.
    let edited = String::from_utf8(text.to_vec()).unwrap().replacen(
        "GB_X_STAR = 0.9374608339874",
        "GB_X_STAR = 0.9374608339875",
        1,
    );
    assert_ne!(edited.as_bytes(), text);
    assert_ne!(fnv1a64(edited.as_bytes()), body);
}
