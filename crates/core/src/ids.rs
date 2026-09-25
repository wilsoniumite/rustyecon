//! Dense ids, the actor and holder enums, and keys (docs/ENGINE.md §2.1, E8).
//!
//! Every tape entity has a [`Key`]; the loader numbers each kind densely in key byte order, so a
//! dense id is valid against one `World` only and anything kept across a tape edit is kept by
//! key. No map on the delta path is keyed by a bare number: holdings are keyed by [`Holder`] and
//! actors by [`ActorId`], so a desk and a pop that share a number can never be confused. July
//! keyed currency flows by `(u32, u32)` and paid the wrong kind (REVIEW §2.2 defect 10).

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

macro_rules! id {
    ($(#[$meta:meta])* $name:ident, $label:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub u32);

        impl $name {
            /// The id as an index into its kind's dense, key-ordered list.
            pub fn idx(self) -> usize {
                self.0 as usize
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($label, "#{}"), self.0)
            }
        }
    };
}

id!(
    /// A good, numbered in key byte order.
    GoodId,
    "good"
);
id!(
    /// A market node, numbered in key byte order.
    NodeId,
    "node"
);
id!(
    /// A channel between two nodes, numbered in key byte order. Static in Phase 0.
    ChannelId,
    "channel"
);
id!(
    /// A buyer or seller class, the unit rationing is recorded by (R12).
    ClassId,
    "class"
);
id!(
    /// A desk, numbered in key byte order among the desks.
    DeskId,
    "desk"
);
id!(
    /// A pop, numbered in key byte order among the pops.
    PopId,
    "pop"
);
id!(
    /// A registered parameter, numbered in key byte order.
    ParamId,
    "param"
);
id!(
    /// A dated or recurring tape event, numbered in key byte order over both lists together.
    EventId,
    "event"
);

/// An actor. The derived order puts every desk before every pop, which is the canonical order
/// hooks are applied in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActorId {
    /// A producer desk.
    Desk(DeskId),
    /// A population group.
    Pop(PopId),
}

impl ActorId {
    /// The actor's kind.
    pub fn kind(self) -> ActorKind {
        match self {
            ActorId::Desk(_) => ActorKind::Desk,
            ActorId::Pop(_) => ActorKind::Pop,
        }
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorId::Desk(d) => write!(f, "{d}"),
            ActorId::Pop(p) => write!(f, "{p}"),
        }
    }
}

/// The two kinds of actor, as the tape names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActorKind {
    /// A producer desk.
    Desk,
    /// A population group.
    Pop,
}

/// Anything that holds an inventory. The derived order puts every actor (in [`ActorId`] order)
/// before every escrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Holder {
    /// A declared actor. Every declared actor holds an inventory, even an empty one.
    Actor(ActorId),
    /// The settlement escrow of one market, (node, non-currency good). It exists only inside
    /// phase 3 and is dropped as soon as it is empty.
    Escrow(NodeId, GoodId),
}

impl fmt::Display for Holder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Holder::Actor(a) => write!(f, "{a}"),
            Holder::Escrow(n, g) => write!(f, "escrow({n}, {g})"),
        }
    }
}

/// A stable name for a tape entity: one or more of `a-z`, `0-9`, `_`, `.` and `-`.
///
/// Every `Key` value is valid: [`Key::new`] and deserialisation both check the character set.
/// Keys order by their bytes, which is the order the loader numbers each kind in.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Key(String);

impl Key {
    /// A key, if `s` is a valid one.
    pub fn new(s: impl Into<String>) -> Result<Key, InvalidKey> {
        let s = s.into();
        if Key::is_valid(&s) {
            Ok(Key(s))
        } else {
            Err(InvalidKey(s))
        }
    }

    /// Whether `s` is non-empty and uses only `a-z`, `0-9`, `_`, `.` and `-`.
    pub fn is_valid(s: &str) -> bool {
        !s.is_empty()
            && s.bytes()
                .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-'))
    }

    /// The key's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Key::new(s).map_err(serde::de::Error::custom)
    }
}

/// A string that is not a valid [`Key`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidKey(pub String);

impl fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid key {:?}: a key is one or more of a-z, 0-9, '_', '.' and '-'",
            self.0
        )
    }
}

impl std::error::Error for InvalidKey {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_check_their_character_set() {
        assert!(Key::new("mine.capacity_base-2").is_ok());
        for bad in ["", "Mill", "a b", "grain/village", "é"] {
            assert!(Key::new(bad).is_err(), "{bad:?} must be rejected");
            let quoted = format!("{bad:?}");
            assert!(ron::from_str::<Key>(&quoted).is_err());
        }
    }

    #[test]
    fn actor_and_holder_orders_are_canonical() {
        let mut a = vec![
            ActorId::Pop(PopId(0)),
            ActorId::Desk(DeskId(1)),
            ActorId::Desk(DeskId(0)),
        ];
        a.sort();
        assert_eq!(
            a,
            vec![
                ActorId::Desk(DeskId(0)),
                ActorId::Desk(DeskId(1)),
                ActorId::Pop(PopId(0))
            ]
        );
        assert!(
            Holder::Actor(ActorId::Pop(PopId(u32::MAX))) < Holder::Escrow(NodeId(0), GoodId(0))
        );
    }
}
