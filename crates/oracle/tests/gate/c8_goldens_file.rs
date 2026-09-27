//! C8: the Rust constants in goldens_1b.rs equal goldens/goldens_1b.txt, rounded to 20
//! significant digits (half up), and the two list the same goldens. goldens_1b.txt carries
//! the digests that goldens/generate_1b.py wrote into it, of generate_1b.py, of generate.py
//! (which it imports for the nesting assertions) and of the goldens; they must match all
//! three files as they are now. Only `generate_1b.py --check` proves the values are its
//! output.

use std::collections::BTreeMap;

use crate::goldens_1b::TABLE_1B;
use crate::goldens_file::{fnv1a64, parse_decimal, round_half_up, SIGNIFICANT};

const GOLDENS_1B_TXT: &str = include_str!("../../goldens/goldens_1b.txt");

const GENERATE_1B_PY: &[u8] = include_bytes!("../../goldens/generate_1b.py");

const GENERATE_PY: &[u8] = include_bytes!("../../goldens/generate.py");

fn goldens_1b_txt() -> BTreeMap<&'static str, &'static str> {
    let mut map = BTreeMap::new();
    for line in GOLDENS_1B_TXT.lines() {
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
    let start = GOLDENS_1B_TXT
        .find(&prefix)
        .unwrap_or_else(|| panic!("goldens_1b.txt has no {prefix:?} line"));
    let rest = &GOLDENS_1B_TXT[start + prefix.len()..];
    let hex = rest.split(['\r', '\n']).next().unwrap_or("");
    let value = u64::from_str_radix(hex, 16).unwrap_or_else(|_| panic!("bad digest {hex:?}"));
    let end = start + GOLDENS_1B_TXT[start..].find('\n').expect("a newline") + 1;
    (value, end)
}

#[test]
fn constants_match_goldens_1b_txt() {
    let file = goldens_1b_txt();
    let mut seen = BTreeMap::new();
    for &(name, literal) in TABLE_1B {
        assert!(seen.insert(name, ()).is_none(), "{name} declared twice");
        let value = file
            .get(name)
            .unwrap_or_else(|| panic!("{name} is not in goldens_1b.txt"));
        if matches!(*value, "true" | "false") {
            assert_eq!(literal, *value, "{name}");
        } else {
            let want = round_half_up(parse_decimal(value), SIGNIFICANT);
            assert_eq!(
                parse_decimal(literal),
                want,
                "{name}: {literal} vs goldens_1b.txt {value}"
            );
        }
    }
    for name in file.keys() {
        assert!(
            seen.contains_key(name),
            "{name} is in goldens_1b.txt but not in goldens_1b.rs"
        );
    }
    assert_eq!(TABLE_1B.len(), 273);
}

#[test]
fn goldens_1b_txt_is_from_generate_1b_py() {
    let (source, _) = digest_line("generate_1b.py");
    assert_eq!(
        fnv1a64(GENERATE_1B_PY),
        source,
        "generate_1b.py changed since it wrote goldens_1b.txt: rerun it, then --check"
    );
    let (imported, _) = digest_line("generate.py");
    assert_eq!(
        fnv1a64(GENERATE_PY),
        imported,
        "generate.py changed since generate_1b.py wrote goldens_1b.txt: rerun it"
    );
    // The body is everything after the last digest line.
    let (body, end) = digest_line("body");
    let text = &GOLDENS_1B_TXT.as_bytes()[end..];
    assert_eq!(
        fnv1a64(text),
        body,
        "goldens_1b.txt was edited after generate_1b.py wrote it"
    );
    // The digest has teeth: one changed digit changes it.
    let edited = String::from_utf8(text.to_vec()).unwrap().replacen(
        "C3_X_STAR = 0.7109034",
        "C3_X_STAR = 0.7109035",
        1,
    );
    assert_ne!(edited.as_bytes(), text);
    assert_ne!(fnv1a64(edited.as_bytes()), body);
}
