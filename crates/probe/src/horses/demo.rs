//! The demo's county instances (docs/demo/WORLD-V2.md §11.3; D2.4, 2026-09-30): the battery of
//! the demo's second pass runs P2.2a's harness at each county's instance in force on 1 January
//! of 1750, 1800, 1825, 1850, 1875 and 1900, 558 county-dates.
//!
//! The rows come from the demo's compiler, `rustyecon worldgen worlds/demo-gb --stage v2a1
//! --instances PATH`, a tab-separated table with a header line naming its columns: `id`
//! (`<county key>@<year>`), the county's v1 row (N and T a year, the rest dimensionless, as
//! [`County`] holds them), the stage's machine type (δ a year, ω, κ a year) and the maker's
//! reservation ψ. Every number is written so that it reads back bit for bit. `horses --counties
//! PATH --inst demo:<id>` reads it, so the probe gains no dependency on worldgen; the test
//! `battery_instances_are_the_harness_instances` holds the rows to the compiler's plans.
//!
//! A demo instance is F5's economy (rule A, wet, C2g) on the county's row, at the stage's δ and
//! ω, with the maker's reservation on at the row's ψ.

use super::instance::{Config, County, Instance, Keys};

/// The prefix of a demo instance's id: `demo:<county key>@<year>`.
pub const PREFIX: &str = "demo:";

/// A demo instance's county basis (never scored: the demo is illustrative, R4, R5).
pub const BASIS: &str =
    "docs/demo/WORLD-V2.md 11.3: a demo-gb county in force on 1 January (illustrative)";

/// The columns the table must have, in any order.
pub const COLUMNS: [&str; 16] = [
    "id", "workers", "land", "space", "chi_max", "eta", "g0", "g1", "k", "a", "lam", "b", "delta",
    "omega", "kappa", "psi",
];

/// One row of the table: a county-date's instance and its maker's reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// `<county key>@<year>`.
    pub id: String,
    /// The instance.
    pub instance: Instance,
    /// The maker's reservation ψ.
    pub psi: f64,
}

/// Whether `id` names a demo instance.
pub fn is_demo(id: &str) -> bool {
    id.starts_with(PREFIX)
}

/// Parse the table's text. Lines starting with `#` are comments.
pub fn parse(text: &str) -> Result<Vec<Row>, String> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.starts_with('#') && !l.trim().is_empty());
    let (_, head) = lines.next().ok_or("the county table has no header")?;
    let head: Vec<&str> = head.split('\t').collect();
    let col = |name: &str| {
        head.iter()
            .position(|h| *h == name)
            .ok_or_else(|| format!("the county table has no column {name}"))
    };
    let at: Vec<usize> = COLUMNS.iter().map(|c| col(c)).collect::<Result<_, _>>()?;
    let mut rows = Vec::new();
    for (n, line) in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        if cells.len() != head.len() {
            return Err(format!(
                "county table line {}: {} cells, the header has {}",
                n + 1,
                cells.len(),
                head.len()
            ));
        }
        let num = |i: usize| -> Result<f64, String> {
            let s = cells[at[i]];
            s.parse::<f64>()
                .map_err(|_| format!("county table line {}: {s:?} is not a number", n + 1))
        };
        let id = cells[at[0]].to_string();
        let county = County {
            workers: num(1)?,
            land: num(2)?,
            space: num(3)?,
            chi_max: num(4)?,
            eta: num(5)?,
            g0: num(6)?,
            g1: num(7)?,
            k: num(8)?,
            a: num(9)?,
            lam: num(10)?,
            b: num(11)?,
        };
        let (delta, omega, kappa, psi) = (num(12)?, num(13)?, num(14)?, num(15)?);
        if !(delta > 0.0 && delta < 1.0 && (0.0..1.0).contains(&omega) && kappa > 0.0) {
            return Err(format!(
                "county table line {}: δ {delta}, ω {omega}, κ {kappa} are not a durable horse",
                n + 1
            ));
        }
        if !(psi > 0.0 && psi < 1.0) {
            return Err(format!(
                "county table line {}: ψ {psi} is not in (0, 1)",
                n + 1
            ));
        }
        let (key, year) = id
            .split_once('@')
            .ok_or_else(|| format!("county table line {}: id {id} is not KEY@YEAR", n + 1))?;
        let instance = Instance {
            id: format!("{PREFIX}{id}"),
            title: format!("{key} in force on 1 January {year}, under rule A (demo v2a.1)"),
            county,
            county_basis: BASIS.into(),
            delta,
            omega,
            kappa,
            config: Config::Wet,
            keys: Keys {
                horse: "horse".into(),
                maker: "maker".into(),
            },
            dials: "c2g".into(),
        };
        if rows.iter().any(|r: &Row| r.id == id) {
            return Err(format!("county table line {}: {id} twice", n + 1));
        }
        rows.push(Row { id, instance, psi });
    }
    Ok(rows)
}

/// The row a demo id (`demo:<key>@<year>`) names.
pub fn find<'a>(rows: &'a [Row], id: &str) -> Result<&'a Row, String> {
    let bare = id
        .strip_prefix(PREFIX)
        .ok_or_else(|| format!("{id} is not a demo instance ({PREFIX}KEY@YEAR)"))?;
    rows.iter()
        .find(|r| r.id == bare)
        .ok_or_else(|| format!("{id}: no such row in the county table"))
}

/// Read the table at `path` and return the row `id` names.
pub fn load(path: &str, id: &str) -> Result<Row, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let rows = parse(&text)?;
    find(&rows, id).cloned()
}
