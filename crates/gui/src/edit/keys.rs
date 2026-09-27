//! Keys for new entries (docs/GUI.md §5.1 item 2; E8). A key matches `[a-z0-9_.-]+` and is new
//! to the whole run tree: the base, every branch of it, siblings and ancestors included, and
//! the edits waiting to be applied. A minted key is `gui.<s>.<n>`, `s` the session's serial
//! and `n` the least that is new, so two sessions never mint the same key and one session never
//! mints one twice in a tree, even after a branch removes it.

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

/// A new key, `gui.<serial>.<n>`, with the least `n` from 1 that `taken` does not hold.
pub fn mint(serial: u32, taken: &BTreeSet<String>) -> Key {
    let mut n: u64 = 1;
    loop {
        let k = format!("gui.{serial}.{n}");
        if !taken.contains(&k) {
            return Key::new(k).expect("gui.<s>.<n> is a key");
        }
        n += 1;
    }
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
        assert_eq!(mint(3, &taken).as_str(), "gui.3.1");
        taken.insert("gui.3.1".to_string());
        taken.insert("gui.3.3".to_string());
        assert_eq!(mint(3, &taken).as_str(), "gui.3.2");
        assert_eq!(mint(4, &taken).as_str(), "gui.4.1");
    }
}
