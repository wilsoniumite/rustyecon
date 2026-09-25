//! Checkpoints (docs/ENGINE.md §2.5 and §7.6; ADDENDUM N11), salvaged from
//! `v2p3: output/checkpoint.rs`.
//!
//! Bytes in and bytes out: paths and file extensions belong to the cli (E2). A checkpoint names
//! the world it belongs to (`world_id`) and the tape events that had fired (`prefix_id`), so a
//! resume can refuse a checkpoint from another tape or another past; July's recorded neither.
//! Its format is checked before its state is read. Lot lives survive both forms (defect 5).

use crate::error::{CheckpointError, CoreError};
use crate::ext::Ext;
use crate::state::SimState;
use crate::world::World;
use bincode::Options;
use serde::{Deserialize, Serialize};

/// The checkpoint format this build writes and reads.
pub const CHECKPOINT_FORMAT: u32 = 1;

/// The first bytes of a binary checkpoint.
const MAGIC: &[u8; 8] = b"RUSTYECK";

/// A saved state, with the identity of the world and past it belongs to.
#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint<E: Ext> {
    /// The checkpoint format.
    pub format: u32,
    /// The `world_id` of the world the state belongs to.
    pub world_id: u64,
    /// The world's `prefix_id` at the state's tick.
    pub prefix_id: u64,
    /// The state.
    pub state: SimState<E>,
}

/// The RON form. `format` is the first field.
#[derive(Serialize, Deserialize)]
#[serde(rename = "Checkpoint", deny_unknown_fields, bound = "")]
struct Repr<E: Ext> {
    format: u32,
    world_id: u64,
    prefix_id: u64,
    state: SimState<E>,
}

/// The RON form's `format` alone; the rest is skipped unread.
#[derive(Deserialize)]
#[serde(rename = "Checkpoint")]
struct FormatProbe {
    format: u32,
}

fn decode(e: impl std::fmt::Display) -> CheckpointError {
    CheckpointError::Decode(e.to_string())
}

fn check_format(found: u32) -> Result<(), CheckpointError> {
    if found == CHECKPOINT_FORMAT {
        Ok(())
    } else {
        Err(CheckpointError::Format {
            found,
            expected: CHECKPOINT_FORMAT,
        })
    }
}

impl<E: Ext> Checkpoint<E> {
    /// A checkpoint of `state` in the current format.
    pub fn new(world_id: u64, prefix_id: u64, state: SimState<E>) -> Checkpoint<E> {
        Checkpoint {
            format: CHECKPOINT_FORMAT,
            world_id,
            prefix_id,
            state,
        }
    }

    /// A checkpoint of `state`, which belongs to `w`: it records `w.world_id` and
    /// `w.prefix_id(state.tick())`.
    pub fn of(w: &World<E>, state: SimState<E>) -> Checkpoint<E> {
        let prefix_id = w.prefix_id(state.tick());
        Checkpoint::new(w.world_id, prefix_id, state)
    }

    /// The binary form: the magic, `format` as a little-endian `u32`, then bincode 1 of
    /// `(world_id, prefix_id, state)`. Core's types always encode; should an extension's
    /// `Serialize` fail, the bytes stop after the header, which `from_bytes` rejects.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4096);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.format.to_le_bytes());
        if let Ok(body) = bincode::serialize(&(self.world_id, self.prefix_id, &self.state)) {
            out.extend_from_slice(&body);
        }
        out
    }

    /// Read the binary form. The magic and format are checked before the state is read; the
    /// state is then decoded with lots re-sorted and re-coalesced, and NaN, `-0.0` and negative
    /// values rejected. Check it against the world with [`Checkpoint::validate`].
    pub fn from_bytes(b: &[u8]) -> Result<Checkpoint<E>, CheckpointError> {
        if b.len() < MAGIC.len() || &b[..MAGIC.len()] != MAGIC {
            return Err(CheckpointError::Magic);
        }
        let rest = &b[MAGIC.len()..];
        let Some((head, body)) = rest.split_first_chunk::<4>() else {
            return Err(decode("no format after the magic"));
        };
        let format = u32::from_le_bytes(*head);
        check_format(format)?;
        let opts = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .reject_trailing_bytes();
        let (world_id, prefix_id, state): (u64, u64, SimState<E>) =
            opts.deserialize(body).map_err(decode)?;
        Ok(Checkpoint {
            format,
            world_id,
            prefix_id,
            state,
        })
    }

    /// The human-readable form: RON with `format` first. Core's types always encode; should an
    /// extension's `Serialize` fail, the text is a comment that `from_ron` rejects.
    pub fn to_ron(&self) -> String {
        let repr = Repr {
            format: self.format,
            world_id: self.world_id,
            prefix_id: self.prefix_id,
            state: self.state.clone(),
        };
        let config = ron::ser::PrettyConfig::new().new_line("\n".to_string());
        match ron::ser::to_string_pretty(&repr, config) {
            Ok(s) => s,
            Err(e) => format!("/* the checkpoint does not serialise: {e} */"),
        }
    }

    /// Read the human-readable form. `format` is read first and checked before the state is
    /// decoded (the rest is skipped unread); the state is then decoded as in `from_bytes`.
    pub fn from_ron(s: &str) -> Result<Checkpoint<E>, CheckpointError> {
        let probe: FormatProbe = ron::from_str(s).map_err(decode)?;
        check_format(probe.format)?;
        let r: Repr<E> = ron::from_str(s).map_err(decode)?;
        Ok(Checkpoint {
            format: r.format,
            world_id: r.world_id,
            prefix_id: r.prefix_id,
            state: r.state,
        })
    }

    /// Check the state against the world it claims: see [`SimState::validate`]. Identity (the
    /// `world_id` and `prefix_id`) is the resume's to check.
    pub fn validate(&self, w: &World<E>) -> Result<(), CoreError> {
        self.state.validate(w)
    }
}

#[cfg(test)]
mod tests {
    //! Salvaged from `v2p3: output/checkpoint.rs:84-102`, now bytes in and bytes out, and
    //! compared by full state hash: July's compared only the tick (N11).

    use super::*;
    use crate::delta::{Phase, Provenance, StateDelta};
    use crate::hash::state_hash;
    use crate::ids::{ActorId, DeskId, GoodId, Holder, PopId};
    use crate::tape::Tape;
    use crate::testkit::{self, good, holder, Bump, TestExt};

    /// A state that has moved: a perishable lot minted and aged, a param set, a price posted
    /// and the extension's counters bumped.
    fn evolved() -> (World<TestExt>, SimState<TestExt>) {
        let (w, mut s) = testkit::load_ext();
        let (mill, bread) = (holder(&w, "mill"), good(&w, "bread"));
        let town = w.id_of("town").unwrap();
        let rate = w.id_of("rate.grain").unwrap();
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Events,
            &[StateDelta::SetParam {
                param: rate,
                value: 7.25,
            }],
        );
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Decisions,
            &[StateDelta::Actor(Bump {
                actor: ActorId::Pop(PopId(1)),
                by: 4,
            })],
        );
        let mint = |qty| StateDelta::Mint {
            to: mill,
            good: bread,
            qty,
            prov: Provenance::Production,
        };
        testkit::apply_ok(&mut s, &w, Phase::Production, &[mint(0.1)]);
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Upkeep,
            &[StateDelta::Age { holder: mill }],
        );
        testkit::apply_ok(&mut s, &w, Phase::Production, &[mint(0.2)]);
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Prices,
            &[
                StateDelta::SetPrice {
                    node: town,
                    good: bread,
                    price: 2.0000000000000004,
                },
                StateDelta::AdvanceTick,
            ],
        );
        assert_eq!(
            s.holding(mill).unwrap().lots(bread).len(),
            2,
            "two lives held"
        );
        (w, s)
    }

    #[test]
    fn binary_round_trip() {
        let (w, s) = evolved();
        let cp = Checkpoint::of(&w, s.clone());
        let back = Checkpoint::<TestExt>::from_bytes(&cp.to_bytes()).unwrap();
        assert_eq!(back, cp);
        assert_eq!(state_hash(&back.state), state_hash(&s));
        assert_eq!(
            (back.world_id, back.prefix_id),
            (w.world_id, w.prefix_id(1))
        );
        back.validate(&w).unwrap();
    }

    #[test]
    fn human_readable_round_trip() {
        let (w, s) = evolved();
        let cp = Checkpoint::of(&w, s.clone());
        let text = cp.to_ron();
        assert!(text.starts_with("(\n    format: 1,"), "format comes first");
        let back = Checkpoint::<TestExt>::from_ron(&text).unwrap();
        assert_eq!(back, cp);
        assert_eq!(state_hash(&back.state), state_hash(&s));
        back.validate(&w).unwrap();
    }

    #[test]
    fn checkpoint_format_is_checked() {
        let (w, s) = evolved();
        let cp = Checkpoint::of(&w, s);
        // Bytes: the magic, then the format, both before the state.
        let mut b = cp.to_bytes();
        b[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&2u32.to_le_bytes());
        assert_eq!(
            Checkpoint::<TestExt>::from_bytes(&b),
            Err(CheckpointError::Format {
                found: 2,
                expected: CHECKPOINT_FORMAT
            })
        );
        let mut b = cp.to_bytes();
        b[0] = b'X';
        assert_eq!(
            Checkpoint::<TestExt>::from_bytes(&b),
            Err(CheckpointError::Magic)
        );
        let b = cp.to_bytes();
        assert!(matches!(
            Checkpoint::<TestExt>::from_bytes(&b[..b.len() - 1]),
            Err(CheckpointError::Decode(_))
        ));
        let mut long = cp.to_bytes();
        long.push(0);
        assert!(matches!(
            Checkpoint::<TestExt>::from_bytes(&long),
            Err(CheckpointError::Decode(_))
        ));
        // RON: `format` is read first. A wrong one is refused even when the state behind it
        // would not decode, which shows the state is not read first.
        let text = cp.to_ron().replacen("format: 1,", "format: 2,", 1);
        assert_eq!(
            Checkpoint::<TestExt>::from_ron(&text),
            Err(CheckpointError::Format {
                found: 2,
                expected: CHECKPOINT_FORMAT
            })
        );
        let mut bad = cp.clone();
        bad.state.params[0] = f64::NAN;
        let text = bad.to_ron().replacen("format: 1,", "format: 7,", 1);
        assert!(matches!(
            Checkpoint::<TestExt>::from_ron(&text),
            Err(CheckpointError::Format { found: 7, .. })
        ));
    }

    #[test]
    fn checkpoint_rejects_nan_and_unknown_ids() {
        let (w, s) = evolved();
        let (mill, bread) = (holder(&w, "mill"), good(&w, "bread"));
        let town = w.id_of("town").unwrap();
        let both = |cp: &Checkpoint<TestExt>| {
            (
                Checkpoint::<TestExt>::from_bytes(&cp.to_bytes()),
                Checkpoint::<TestExt>::from_ron(&cp.to_ron()),
            )
        };
        // Values: NaN or -0.0 in a lot, a param or the book do not decode.
        let mut nan_lot = Checkpoint::of(&w, s.clone());
        nan_lot.state.holdings.get_mut(&mill).unwrap().raw_mut()[0].1[0].qty = f64::NAN;
        let mut neg_param = Checkpoint::of(&w, s.clone());
        neg_param.state.params[1] = -0.0;
        let mut neg_price = Checkpoint::of(&w, s.clone());
        let i = neg_price.state.book.index(town, bread).unwrap();
        neg_price.state.book.set_price(i, -2.0);
        for cp in [nan_lot, neg_param, neg_price] {
            let (b, r) = both(&cp);
            assert!(matches!(b, Err(CheckpointError::Decode(_))), "{b:?}");
            assert!(matches!(r, Err(CheckpointError::Decode(_))), "{r:?}");
        }
        // Ids: they decode, and validation against the world refuses them.
        let ghost = Holder::Actor(ActorId::Desk(DeskId(9)));
        let mut cases: Vec<(Checkpoint<TestExt>, CoreError)> = Vec::new();
        let mut cp = Checkpoint::of(&w, s.clone());
        cp.state.holdings.insert(ghost, Default::default());
        cases.push((cp, CoreError::UnknownHolder(ghost)));
        let mut cp = Checkpoint::of(&w, s.clone());
        testkit::write_direct(&mut cp.state, mill, GoodId(99), 1.0);
        cases.push((cp, CoreError::UnknownGood(GoodId(99))));
        let mut cp = Checkpoint::of(&w, s.clone());
        testkit::write_direct(&mut cp.state, Holder::Escrow(town, bread), bread, 1.0);
        cases.push((
            cp,
            CoreError::EscrowLeft {
                node: town,
                good: bread,
            },
        ));
        for (cp, want) in cases {
            let (b, r) = both(&cp);
            assert_eq!(b.unwrap().validate(&w), Err(want.clone()));
            assert_eq!(r.unwrap().validate(&w), Err(want));
        }
        // Shapes: a missing actor, a lot life its good cannot have, the extension's own state,
        // and a checkpoint of another world.
        let mut cp = Checkpoint::of(&w, s.clone());
        cp.state.holdings.remove(&mill);
        assert!(matches!(cp.validate(&w), Err(CoreError::Shape(_))));
        let mut cp = Checkpoint::of(&w, s.clone());
        cp.state.holdings.get_mut(&mill).unwrap().raw_mut()[0].1[0].life = Some(40);
        assert!(matches!(cp.validate(&w), Err(CoreError::Shape(_))));
        let mut cp = Checkpoint::of(&w, s.clone());
        cp.state.ext.remove(&ActorId::Pop(PopId(0)));
        assert!(matches!(cp.validate(&w), Err(CoreError::Shape(_))));
        let text = testkit::ext_text()
            .replace(
                r#"(key: "coin", life: Indefinite, price_rate: None),"#,
                r#"(key: "cloth", life: Indefinite, price_rate: Some("rate.grain")),
                   (key: "coin", life: Indefinite, price_rate: None),"#,
            )
            .replace(
                r#"(node: "town", good: "bread", price: 2.0),"#,
                r#"(node: "town", good: "bread", price: 2.0),
                   (node: "town", good: "cloth", price: 1.0),
                   (node: "village", good: "cloth", price: 1.0),"#,
            );
        let other: Tape<TestExt> = Tape::from_ron(&text).unwrap();
        let (w2, _) = crate::resolve(&other).unwrap();
        assert_eq!(w2.n_goods(), 5);
        assert!(matches!(
            Checkpoint::of(&w, s).validate(&w2),
            Err(CoreError::Shape(_))
        ));
    }
}
