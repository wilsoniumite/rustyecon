//! The lineage (docs/GUI.md §5.1 item 6; D3, U2): where a branch's tape came from. It names
//! the nearest ancestor that exists on disk, a base file or a saved tape, by its `tape_hash`
//! and path, and lists every edit since it, through each unsaved branch between, in order,
//! each with its note, its date, and the value and basis it replaced or the entry it removed.
//!
//! "Save tape as…" writes it beside the tape as `<name>.lineage.ron`, and every export
//! carries it. It also names the `tape_hash` of the tape it describes, so a lineage read
//! beside a tape edited since it was saved is seen not to describe it.

use super::Applied;
use crate::run::SeriesKey;
use certify::Hex;
use serde::{Deserialize, Serialize};

/// The format this build reads and writes.
pub const LINEAGE_FORMAT: u32 = 1;

/// The nearest ancestor on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ancestor {
    /// `certify::tape_hash` of its tape.
    pub tape_hash: Hex,
    /// The path it was opened from or saved to.
    pub path: String,
}

/// A tape's lineage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lineage {
    /// [`LINEAGE_FORMAT`].
    pub format: u32,
    /// `certify::tape_hash` of the tape this lineage describes.
    pub tape_hash: Hex,
    /// The nearest ancestor on disk.
    pub parent: Ancestor,
    /// Every edit since it, in the order applied.
    pub edits: Vec<Applied>,
    /// The scored series overlaid while the edits were made: empty until G5 (U5).
    pub records_viewed: Vec<SeriesKey>,
}

impl Lineage {
    /// The lineage of a tape (`tape_hash`) made from `parent` by `edits`. When the parent is
    /// on disk it is the ancestor; otherwise the parent's own lineage names it, and its edits
    /// come first.
    pub fn of(
        tape_hash: u64,
        parent_on_disk: Option<Ancestor>,
        parent_lineage: Option<&Lineage>,
        edits: &[Applied],
    ) -> Option<Lineage> {
        let (parent, mut all) = match (parent_on_disk, parent_lineage) {
            (Some(a), _) => (a, Vec::new()),
            (None, Some(l)) => (l.parent.clone(), l.edits.clone()),
            (None, None) => return None,
        };
        all.extend(edits.iter().cloned());
        Some(Lineage {
            format: LINEAGE_FORMAT,
            tape_hash: Hex(tape_hash),
            parent,
            edits: all,
            records_viewed: Vec::new(),
        })
    }

    /// The lineage as RON, with struct names.
    pub fn to_ron(&self) -> String {
        let pretty = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .new_line("\n".to_string());
        let mut s = ron::ser::to_string_pretty(self, pretty).expect("a lineage serialises");
        s.push('\n');
        s
    }

    /// A lineage from RON, of this build's format.
    pub fn from_ron(text: &str) -> Result<Lineage, String> {
        let l: Lineage = ron::from_str(text).map_err(|e| e.to_string())?;
        if l.format != LINEAGE_FORMAT {
            return Err(format!(
                "lineage format {}; this build reads {LINEAGE_FORMAT}",
                l.format
            ));
        }
        Ok(l)
    }

    /// The lineage in lines, as the compare panel lists it: the ancestor, then each edit.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "from {} (tape_hash {})",
            self.parent.path, self.parent.tape_hash
        )];
        out.extend(
            self.edits
                .iter()
                .enumerate()
                .map(|(i, a)| format!("{}. {a}", i + 1)),
        );
        out
    }
}

/// Where a tape's lineage lives: beside it, `<name>.lineage.ron` for `<name>.ron`.
pub fn lineage_path(tape: &str) -> String {
    match tape.strip_suffix(".ron") {
        Some(stem) => format!("{stem}.lineage.ron"),
        None => format!("{tape}.lineage.ron"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lineage_lives_beside_its_tape() {
        assert_eq!(lineage_path("d/branch.ron"), "d/branch.lineage.ron");
        assert_eq!(lineage_path("d/branch"), "d/branch.lineage.ron");
    }
}
