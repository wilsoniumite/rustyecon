//! The oracle's goldens, for the lab to show beside its outputs: the six committed files the
//! generators write at 70 digits from the equations (`crates/oracle/goldens/goldens*.txt`),
//! bundled here as text. Each line is `KEY = value  # note`; a key starting `PUB_` was
//! transcribed from the paper. The files carry 30 significant digits, and a value is shown as
//! written, beside the double nearest to it.
//!
//! An output is paired with the golden of its instance's prefix and its own key, upper-cased
//! with `.` read as `_`: at G1, `x_star` is `G1_X_STAR`, and `PUB_G1_X_STAR` is the paper's.
//! A golden a generator named otherwise is listed apart, never guessed at.

use serde::Serialize;

/// The goldens files, by name, as committed.
pub const FILES: [(&str, &str); 6] = [
    (
        "goldens.txt",
        include_str!("../../../oracle/goldens/goldens.txt"),
    ),
    (
        "goldens_1b.txt",
        include_str!("../../../oracle/goldens/goldens_1b.txt"),
    ),
    (
        "goldens_1c.txt",
        include_str!("../../../oracle/goldens/goldens_1c.txt"),
    ),
    (
        "goldens_1d.txt",
        include_str!("../../../oracle/goldens/goldens_1d.txt"),
    ),
    (
        "goldens_1e.txt",
        include_str!("../../../oracle/goldens/goldens_1e.txt"),
    ),
    (
        "goldens_1f.txt",
        include_str!("../../../oracle/goldens/goldens_1f.txt"),
    ),
];

/// A golden's value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum GoldenValue {
    /// A number: the double nearest to what is written.
    Number(f64),
    /// A flag.
    Flag(bool),
}

/// One golden line.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Golden {
    /// Its key.
    pub key: String,
    /// Its value as written.
    pub text: String,
    /// Its value.
    pub value: GoldenValue,
    /// The generator's note.
    pub note: String,
}

/// The text of a goldens file by name.
pub fn file(name: &str) -> Option<&'static str> {
    FILES.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

/// Every golden of a file's text, in order.
pub fn parse(text: &str) -> Vec<Golden> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (body, note) = match line.split_once('#') {
            Some((b, n)) => (b.trim(), n.trim()),
            None => (line, ""),
        };
        let Some((key, value)) = body.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        let parsed = match value {
            "true" => GoldenValue::Flag(true),
            "false" => GoldenValue::Flag(false),
            v => match v.parse::<f64>() {
                Ok(x) => GoldenValue::Number(x),
                Err(_) => continue,
            },
        };
        out.push(Golden {
            key: key.to_string(),
            text: value.to_string(),
            value: parsed,
            note: note.to_string(),
        });
    }
    out
}

/// The goldens of one instance: every key of `file` that starts with `prefix_` or
/// `PUB_prefix_`.
pub fn of(file_name: &str, prefix: &str) -> Vec<Golden> {
    let Some(text) = file(file_name) else {
        return Vec::new();
    };
    let own = format!("{prefix}_");
    let published = format!("PUB_{prefix}_");
    parse(text)
        .into_iter()
        .filter(|g| g.key.starts_with(&own) || g.key.starts_with(&published))
        .collect()
}

/// The generator's heading of an instance, the file's `# PREFIX: …` line, if it has one.
pub fn heading(file_name: &str, prefix: &str) -> Option<String> {
    let tag = format!("# {prefix}: ");
    file(file_name)?
        .lines()
        .find_map(|l| l.strip_prefix(&tag).map(str::to_string))
}

/// The key an output's golden goes by at `prefix`: `x_star` at G1 is `G1_X_STAR`, and
/// `cat0.price` would be `C3_CAT0_PRICE`.
pub fn key_of(prefix: &str, output: &str) -> String {
    format!("{prefix}_{}", output.replace('.', "_").to_uppercase())
}

/// The relative difference of a value from a golden, |got − want|/|want|, or the absolute
/// difference where the golden is 0.
pub fn relative(got: f64, want: f64) -> f64 {
    if want == 0.0 {
        (got - want).abs()
    } else {
        ((got - want) / want).abs()
    }
}
