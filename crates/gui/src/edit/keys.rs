//! Keys for new entries (docs/GUI.md §5.1 item 2; E8). A key matches `[a-z0-9_.-]+` and is new
//! to every tape open in this session (the bases, every branch, siblings and ancestors, and a
//! saved branch reopened as its own root), to every run closed in it, and to the edits waiting
//! to be applied. A minted key is `gui.<s>.<n>`, `s` the session's serial and `n` above every
//! `n` this session has seen under `s`: in those keys, and in every key it staged. So one
//! session never mints a key twice, even after a branch removes it or a run closes. Each
//! launch that reads `session.ron` takes the next serial; a key minted in another session is
//! new here only when that session's serial differs.

use super::TapeEdit;
use rustyecon_engine::prelude::*;
use std::collections::BTreeSet;

/// Every key a tape uses, of every kind: params, goods, nodes, channels, classes, actors,
/// events and recurring entries.
pub fn tape_keys(t: &Tape) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    out.extend(t.params.iter().map(|p| p.key.to_string()));
    out.extend(t.goods.iter().map(|g| g.key.to_string()));
    out.extend(t.nodes.iter().map(|n| n.key.to_string()));
    out.extend(t.channels.iter().map(|c| c.key.to_string()));
    out.extend(t.classes.iter().map(Key::to_string));
    out.extend(t.actors.iter().map(|a| a.key.to_string()));
    out.extend(t.events.iter().map(|e| e.key.to_string()));
    out.extend(t.recurring.iter().map(|r| r.key.to_string()));
    out
}

/// The keys edits add.
pub fn added_keys(edits: &[TapeEdit]) -> BTreeSet<String> {
    edits
        .iter()
        .filter_map(|e| e.op.added())
        .map(Key::to_string)
        .collect()
}

/// A key typed into a form: one that matches `[a-z0-9_.-]+`, or why it is not one.
pub fn valid(text: &str) -> Result<Key, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("a key is required".to_string());
    }
    Key::new(text).map_err(|_| format!("the key {text:?} must match [a-z0-9_.-]+"))
}

/// The `n` of a key `gui.<serial>.<n>`, if it is one.
pub fn minted_n(key: &str, serial: u32) -> Option<u64> {
    key.strip_prefix(&format!("gui.{serial}."))?.parse().ok()
}

/// A new key, `gui.<serial>.<n>`, with `n` one more than the largest of `floor` and every
/// `n` of a `gui.<serial>.<n>` that `taken` holds: never one that `taken` holds, and never
/// below a key this session minted before (`floor`).
pub fn mint(serial: u32, taken: &BTreeSet<String>, floor: u64) -> Key {
    let top = taken
        .iter()
        .filter_map(|k| minted_n(k, serial))
        .fold(floor, u64::max);
    Key::new(format!("gui.{serial}.{}", top.saturating_add(1))).expect("gui.<s>.<n> is a key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_checked_and_minted() {
        assert_eq!(valid(" gui.1.1 ").unwrap().as_str(), "gui.1.1");
        assert!(valid("Mine Cut").is_err());
        assert!(valid("").is_err());
        let mut taken = BTreeSet::new();
        assert_eq!(mint(3, &taken, 0).as_str(), "gui.3.1");
        taken.insert("gui.3.1".to_string());
        taken.insert("gui.3.3".to_string());
        // Above every n under the serial, so a gap left by a removal is not filled.
        assert_eq!(mint(3, &taken, 0).as_str(), "gui.3.4");
        assert_eq!(mint(3, &taken, 7).as_str(), "gui.3.8");
        assert_eq!(mint(4, &taken, 0).as_str(), "gui.4.1");
        assert_eq!(minted_n("gui.3.12", 3), Some(12));
        assert_eq!(minted_n("gui.3.12", 31), None);
        assert_eq!(minted_n("gui.31.2", 3), None);
        assert_eq!(minted_n("gui.3.x", 3), None);
    }
}
