//! The Rust constants in goldens.rs equal goldens/goldens.txt, rounded to 20 significant
//! digits (half up), and the two list the same goldens. goldens.txt carries the digests
//! that goldens/generate.py wrote into it, of generate.py and of the goldens, and they
//! must match both files as they are now.

use std::collections::BTreeMap;

use crate::goldens::TABLE;

/// Significant digits the Rust constants keep.
pub(crate) const SIGNIFICANT: usize = 20;

const GOLDENS_TXT: &str = include_str!("../../goldens/goldens.txt");

const GENERATE_PY: &[u8] = include_bytes!("../../goldens/generate.py");

/// A decimal as (negative, significant digits without leading or trailing zeros,
/// decimal exponent of the first digit). Zero is (false, [], 0).
pub(crate) type Decimal = (bool, Vec<u8>, i32);

pub(crate) fn parse_decimal(text: &str) -> Decimal {
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let (negative, unsigned) = match compact.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, compact.as_str()),
    };
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((m, e)) => (
            m,
            e.parse::<i32>()
                .unwrap_or_else(|_| panic!("bad exponent in {text}")),
        ),
        None => (unsigned, 0),
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut digits: Vec<u8> = int
        .bytes()
        .chain(frac.bytes())
        .map(|b| {
            assert!(b.is_ascii_digit(), "not a decimal: {text}");
            b - b'0'
        })
        .collect();
    let mut first = int.len() as i32 - 1 + exponent;
    while digits.first() == Some(&0) {
        digits.remove(0);
        first -= 1;
    }
    while digits.last() == Some(&0) {
        digits.pop();
    }
    if digits.is_empty() {
        return (false, digits, 0);
    }
    (negative, digits, first)
}

pub(crate) fn round_half_up(
    (negative, mut digits, mut first): Decimal,
    significant: usize,
) -> Decimal {
    if digits.len() > significant {
        let up = digits[significant] >= 5;
        digits.truncate(significant);
        if up {
            let mut i = significant;
            loop {
                if i == 0 {
                    digits.insert(0, 1);
                    first += 1;
                    break;
                }
                i -= 1;
                if digits[i] == 9 {
                    digits[i] = 0;
                } else {
                    digits[i] += 1;
                    break;
                }
            }
        }
        while digits.last() == Some(&0) {
            digits.pop();
        }
    }
    (negative, digits, first)
}

fn goldens_txt() -> BTreeMap<&'static str, &'static str> {
    let mut map = BTreeMap::new();
    for line in GOLDENS_TXT.lines() {
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

#[test]
fn rounding_helper() {
    assert_eq!(parse_decimal("0.0"), (false, vec![], 0));
    assert_eq!(parse_decimal("-0.1"), (true, vec![1], -1));
    assert_eq!(parse_decimal("5.44630"), parse_decimal("5.4463"));
    assert_eq!(parse_decimal("1.5e-3"), parse_decimal("0.0015"));
    assert_eq!(
        round_half_up(parse_decimal("0.99996"), 4),
        parse_decimal("1")
    );
    assert_eq!(
        round_half_up(parse_decimal("0.123449"), 4),
        parse_decimal("0.1234")
    );
    assert_eq!(
        round_half_up(parse_decimal("-10.00005"), 6),
        parse_decimal("-10.0001")
    );
}

#[test]
fn constants_match_goldens_txt() {
    let file = goldens_txt();
    let mut seen = BTreeMap::new();
    for &(name, literal) in TABLE {
        assert!(seen.insert(name, ()).is_none(), "{name} declared twice");
        let value = file
            .get(name)
            .unwrap_or_else(|| panic!("{name} is not in goldens.txt"));
        if matches!(*value, "true" | "false") {
            assert_eq!(literal, *value, "{name}");
        } else {
            let want = round_half_up(parse_decimal(value), SIGNIFICANT);
            assert_eq!(
                parse_decimal(literal),
                want,
                "{name}: {literal} vs goldens.txt {value}"
            );
        }
    }
    for name in file.keys() {
        assert!(
            seen.contains_key(name),
            "{name} is in goldens.txt but not in goldens.rs"
        );
    }
}

/// FNV-1a, 64 bits, over the bytes with every CRLF read as LF: generate.py's `fnv1a64`,
/// so the digests do not depend on how git checked the files out.
pub(crate) fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for (i, &byte) in data.iter().enumerate() {
        if byte == b'\r' && data.get(i + 1) == Some(&b'\n') {
            continue;
        }
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

/// The digest on the header line `# fnv1a64 <label> = <hex>`, and the byte offset just
/// past that line.
fn digest_line(label: &str) -> (u64, usize) {
    let prefix = format!("# fnv1a64 {label} = ");
    let start = GOLDENS_TXT
        .find(&prefix)
        .unwrap_or_else(|| panic!("goldens.txt has no {prefix:?} line"));
    let rest = &GOLDENS_TXT[start + prefix.len()..];
    let hex = rest.split(['\r', '\n']).next().unwrap_or("");
    let value = u64::from_str_radix(hex, 16).unwrap_or_else(|_| panic!("bad digest {hex:?}"));
    let end = start + GOLDENS_TXT[start..].find('\n').expect("a newline") + 1;
    (value, end)
}

#[test]
fn fnv1a64_matches_its_reference_values() {
    // The FNV reference values for "" and "a", and CRLF read as LF (a lone CR is kept).
    assert_eq!(fnv1a64(b""), 0xCBF2_9CE4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xAF63_DC4C_8601_EC8C);
    assert_eq!(fnv1a64(b"x\r\ny\r"), fnv1a64(b"x\ny\r"));
    assert_ne!(fnv1a64(b"x\ry"), fnv1a64(b"xy"));
}

#[test]
fn goldens_txt_is_from_generate_py() {
    // generate.py writes the digest of its own source and of the goldens below the
    // header. A goldens.txt edited by hand, or not rewritten after generate.py changed,
    // fails here even where mpmath is not installed. (Only `generate.py --check` proves the
    // values are generate.py's output.)
    let (source, _) = digest_line("generate.py");
    assert_eq!(
        fnv1a64(GENERATE_PY),
        source,
        "generate.py changed since it wrote goldens.txt: rerun it, then --check"
    );
    let (body, end) = digest_line("body");
    let text = &GOLDENS_TXT.as_bytes()[end..];
    assert_eq!(
        fnv1a64(text),
        body,
        "goldens.txt was edited after generate.py wrote it"
    );
    // The digest has teeth: one changed digit changes it.
    let edited = String::from_utf8(text.to_vec()).unwrap().replacen(
        "G1_X_STAR = 0.8631504",
        "G1_X_STAR = 0.8631505",
        1,
    );
    assert_ne!(edited.as_bytes(), text);
    assert_ne!(fnv1a64(edited.as_bytes()), body);
}
