//! Checkpoints (docs/ENGINE.md §2.5 and §7.6; ADDENDUM N11), salvaged from
//! `v2p3: output/checkpoint.rs`.
//!
//! Bytes in and bytes out: paths and file extensions belong to the cli (E2). A checkpoint names
//! the world it belongs to (`world_id`) and the tape events that had fired (`prefix_id`), so a
//! resume can refuse a checkpoint from another tape or another past; July's recorded neither.
//! It also carries the run's ledger, so a resumed run continues the conservation audit of the
//! run that reached it (R2; amended at P0.9, O8). Its format is checked before its state is
//! read. Lot lives survive both forms (defect 5).
//!
//! Both forms carry a digest, FNV-1a 64 over bincode 1 of `(world_id, prefix_id, state, run)`,
//! and a decoder refuses a checkpoint whose fields do not hash to it, so a checkpoint edited or
//! corrupted after it was saved does not resume (R2, E1): not its state, and since format 3 not
//! its identity or its run ledger either (O7). The fields are private: a `Checkpoint` in memory
//! is made only by [`Checkpoint::of`] or by a decoder, and nothing hands out `&mut` to its
//! state. The digest is FNV-1a, not a signature: it catches an edit, not a forger who
//! recomputes it, and a state built with core's own writer and passed to `of` is trusted as
//! given. That boundary, and the replay audit that proves a run's states came from its tape, are
//! in docs/ENGINE.md §7.6.
//!
//! The state cannot be changed in place:
//!
//! ```compile_fail
//! fn edit(cp: &mut rustyecon_core::Checkpoint<rustyecon_core::NoExt>) {
//!     let _ = &mut cp.state;
//! }
//! ```
//!
//! It is read through its accessors:
//!
//! ```
//! fn tick(cp: &rustyecon_core::Checkpoint<rustyecon_core::NoExt>) -> u64 {
//!     cp.state().tick()
//! }
//! ```

use crate::error::{CheckpointError, CoreError};
use crate::ext::Ext;
use crate::hash::Fnv;
use crate::ledger::RunLedger;
use crate::state::SimState;
use crate::world::World;
use bincode::Options;
use serde::{Deserialize, Serialize};

/// The checkpoint format this build writes and reads. Format 2 added the state digest; format 3
/// put `world_id` and `prefix_id` inside the digest and added the run's ledger (P0.9).
pub const CHECKPOINT_FORMAT: u32 = 3;

/// The first bytes of a binary checkpoint.
const MAGIC: &[u8; 8] = b"RUSTYECK";

/// A saved run: its state, the run's ledger, and the identity of the world and past they belong
/// to. Made only by [`Checkpoint::of`] and the decoders, and read through its accessors.
#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint<E: Ext> {
    world_id: u64,
    prefix_id: u64,
    state: SimState<E>,
    run: RunLedger,
}

/// The RON form. `format` is the first field.
#[derive(Serialize, Deserialize)]
#[serde(rename = "Checkpoint", deny_unknown_fields, bound = "")]
struct Repr<E: Ext> {
    format: u32,
    world_id: u64,
    prefix_id: u64,
    digest: u64,
    state: SimState<E>,
    run: RunLedger,
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

/// FNV-1a 64 over bincode 1 of `(world_id, prefix_id, state, run)`.
fn digest_of<E: Ext>(world_id: u64, prefix_id: u64, state: &SimState<E>, run: &RunLedger) -> u64 {
    let mut h = Fnv::new();
    h.write_serialized(&(world_id, prefix_id, state, run));
    h.finish()
}

/// A decoded checkpoint, provided its fields hash to the digest stored with them.
fn checked<E: Ext>(
    world_id: u64,
    prefix_id: u64,
    stored: u64,
    state: SimState<E>,
    run: RunLedger,
) -> Result<Checkpoint<E>, CheckpointError> {
    let computed = digest_of(world_id, prefix_id, &state, &run);
    if computed != stored {
        return Err(CheckpointError::Digest { stored, computed });
    }
    Ok(Checkpoint {
        world_id,
        prefix_id,
        state,
        run,
    })
}

impl<E: Ext> Checkpoint<E> {
    /// A checkpoint of `state`, which belongs to `w`, with `run`, the ledger of the run that
    /// reached it: it records `w.world_id` and `w.prefix_id(state.tick())`. The run's ledger
    /// must cover `w`'s goods and have closed every tick before the state's
    /// ([`RunLedger::fits`]); a new run's is [`RunLedger::open`] on the state.
    pub fn of(
        w: &World<E>,
        state: SimState<E>,
        run: RunLedger,
    ) -> Result<Checkpoint<E>, CoreError> {
        run.fits(w, state.tick())?;
        let prefix_id = w.prefix_id(state.tick());
        Ok(Checkpoint {
            world_id: w.world_id,
            prefix_id,
            state,
            run,
        })
    }

    /// The `world_id` of the world the state belongs to.
    pub fn world_id(&self) -> u64 {
        self.world_id
    }

    /// The world's `prefix_id` at the state's tick.
    pub fn prefix_id(&self) -> u64 {
        self.prefix_id
    }

    /// The state.
    pub fn state(&self) -> &SimState<E> {
        &self.state
    }

    /// The ledger of the run that reached the state, which a resumed run continues.
    pub fn run(&self) -> &RunLedger {
        &self.run
    }

    /// The digest both forms store: FNV-1a 64 over bincode 1 of `(world_id, prefix_id, state,
    /// run)`.
    pub fn digest(&self) -> u64 {
        digest_of(self.world_id, self.prefix_id, &self.state, &self.run)
    }

    /// The binary form: the magic, [`CHECKPOINT_FORMAT`] as a little-endian `u32`, then
    /// bincode 1 of `(world_id, prefix_id, digest, state, run)`. Core's types always encode;
    /// should an extension's `Serialize` fail, the bytes stop after the header, which
    /// `from_bytes` rejects.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4096);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&CHECKPOINT_FORMAT.to_le_bytes());
        let body = (
            self.world_id,
            self.prefix_id,
            self.digest(),
            &self.state,
            &self.run,
        );
        if let Ok(body) = bincode::serialize(&body) {
            out.extend_from_slice(&body);
        }
        out
    }

    /// Read the binary form. The magic and format are checked before the state is read; the
    /// state is then decoded with lots re-sorted and re-coalesced, and NaN, `-0.0` and negative
    /// values rejected; last, the fields must hash to the stored digest
    /// ([`CheckpointError::Digest`]). Check it against the world with [`Checkpoint::validate`].
    pub fn from_bytes(b: &[u8]) -> Result<Checkpoint<E>, CheckpointError> {
        if b.len() < MAGIC.len() || &b[..MAGIC.len()] != MAGIC {
            return Err(CheckpointError::Magic);
        }
        let rest = &b[MAGIC.len()..];
        let Some((head, body)) = rest.split_first_chunk::<4>() else {
            return Err(decode("no format after the magic"));
        };
        check_format(u32::from_le_bytes(*head))?;
        let opts = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .reject_trailing_bytes();
        let (world_id, prefix_id, digest, state, run): (u64, u64, u64, SimState<E>, RunLedger) =
            opts.deserialize(body).map_err(decode)?;
        checked(world_id, prefix_id, digest, state, run)
    }

    /// The human-readable form: RON with `format` first, then `world_id`, `prefix_id`,
    /// `digest`, the state and the run's ledger. Core's types always encode; should an
    /// extension's `Serialize` fail, the text is a comment that `from_ron` rejects.
    pub fn to_ron(&self) -> String {
        let repr = Repr {
            format: CHECKPOINT_FORMAT,
            world_id: self.world_id,
            prefix_id: self.prefix_id,
            digest: self.digest(),
            state: self.state.clone(),
            run: self.run.clone(),
        };
        let config = ron::ser::PrettyConfig::new().new_line("\n".to_string());
        match ron::ser::to_string_pretty(&repr, config) {
            Ok(s) => s,
            Err(e) => format!("/* the checkpoint does not serialise: {e} */"),
        }
    }

    /// Read the human-readable form. `format` is read first and checked before the state is
    /// decoded (the rest is skipped unread); the rest is then decoded and checked against its
    /// digest as in `from_bytes`.
    pub fn from_ron(s: &str) -> Result<Checkpoint<E>, CheckpointError> {
        let probe: FormatProbe = ron::from_str(s).map_err(decode)?;
        check_format(probe.format)?;
        let r: Repr<E> = ron::from_str(s).map_err(decode)?;
        checked(r.world_id, r.prefix_id, r.digest, r.state, r.run)
    }

    /// Check the state against the world it claims (see [`SimState::validate`]), and the run's
    /// ledger against the world and the state's tick ([`RunLedger::fits`]). Identity (the
    /// `world_id` and `prefix_id`) is the resume's to check.
    pub fn validate(&self, w: &World<E>) -> Result<(), CoreError> {
        self.state.validate(w)?;
        self.run.fits(w, self.state.tick())
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

    /// A checkpoint of `s` that starts a run there.
    fn cp_of(w: &World<TestExt>, s: SimState<TestExt>) -> Checkpoint<TestExt> {
        let run = RunLedger::open(&s, w).unwrap();
        Checkpoint::of(w, s, run).unwrap()
    }

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
        let cp = cp_of(&w, s.clone());
        let back = Checkpoint::<TestExt>::from_bytes(&cp.to_bytes()).unwrap();
        assert_eq!(back, cp);
        assert_eq!(state_hash(back.state()), state_hash(&s));
        assert_eq!(
            (back.world_id(), back.prefix_id()),
            (w.world_id, w.prefix_id(1))
        );
        assert_eq!(back.run(), &RunLedger::open(&s, &w).unwrap());
        assert_eq!(back.digest(), cp.digest());
        assert_ne!(
            back.digest(),
            state_hash(&s),
            "the digest covers more than the state"
        );
        back.validate(&w).unwrap();
    }

    #[test]
    fn human_readable_round_trip() {
        let (w, s) = evolved();
        let cp = cp_of(&w, s.clone());
        let text = cp.to_ron();
        assert!(text.starts_with("(\n    format: 3,"), "format comes first");
        let back = Checkpoint::<TestExt>::from_ron(&text).unwrap();
        assert_eq!(back, cp);
        assert_eq!(state_hash(back.state()), state_hash(&s));
        back.validate(&w).unwrap();
    }

    #[test]
    fn checkpoint_format_is_checked() {
        let (w, s) = evolved();
        let cp = cp_of(&w, s);
        // Bytes: the magic, then the format, both before the state. Format 1, which had no
        // digest, and format 2, whose digest covered the state alone, are refused like any
        // other.
        for found in [1u32, 2, 4] {
            let mut b = cp.to_bytes();
            b[MAGIC.len()..MAGIC.len() + 4].copy_from_slice(&found.to_le_bytes());
            assert_eq!(
                Checkpoint::<TestExt>::from_bytes(&b),
                Err(CheckpointError::Format {
                    found,
                    expected: CHECKPOINT_FORMAT
                })
            );
        }
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
        let text = cp.to_ron().replacen("format: 3,", "format: 1,", 1);
        assert_eq!(
            Checkpoint::<TestExt>::from_ron(&text),
            Err(CheckpointError::Format {
                found: 1,
                expected: CHECKPOINT_FORMAT
            })
        );
        let mut bad = cp.clone();
        bad.state.params[0] = f64::NAN;
        let text = bad.to_ron().replacen("format: 3,", "format: 7,", 1);
        assert!(matches!(
            Checkpoint::<TestExt>::from_ron(&text),
            Err(CheckpointError::Format { found: 7, .. })
        ));
    }

    #[test]
    fn checkpoint_digest_refuses_an_edited_state() {
        // R2, E1: a checkpoint carries a digest of its fields, and a state edited after saving
        // (one lot's quantity here) is refused in either form before any resume sees it.
        // Without the digest, the edit would decode and resume, and the goods it made would
        // appear in no ledger line.
        let (w, s) = evolved();
        let cp = cp_of(&w, s);
        let mut edited = cp.clone();
        edited
            .state
            .holdings
            .get_mut(&holder(&w, "mill"))
            .unwrap()
            .raw_mut()[0]
            .1[0]
            .qty += 1.0;
        assert_ne!(edited.digest(), cp.digest());
        let refused = Err(CheckpointError::Digest {
            stored: cp.digest(),
            computed: edited.digest(),
        });
        // Bytes: the edited state behind the original digest, which follows the magic (8
        // bytes), the format (4), world_id (8) and prefix_id (8).
        let mut b = edited.to_bytes();
        b[28..36].copy_from_slice(&cp.digest().to_le_bytes());
        assert_eq!(Checkpoint::<TestExt>::from_bytes(&b), refused);
        // RON: the same.
        let digest = |d: u64| format!("digest: {d},");
        let text = edited.to_ron();
        assert_eq!(text.matches(&digest(edited.digest())).count(), 1);
        let text = text.replacen(&digest(edited.digest()), &digest(cp.digest()), 1);
        assert_eq!(Checkpoint::<TestExt>::from_ron(&text), refused);
        // A changed digest alone is refused too, and the untouched forms still load.
        let text = cp
            .to_ron()
            .replacen(&digest(cp.digest()), &digest(cp.digest() ^ 1), 1);
        assert!(matches!(
            Checkpoint::<TestExt>::from_ron(&text),
            Err(CheckpointError::Digest { .. })
        ));
        assert_eq!(
            Checkpoint::<TestExt>::from_ron(&cp.to_ron()),
            Ok(cp.clone())
        );
        assert_eq!(Checkpoint::<TestExt>::from_bytes(&cp.to_bytes()), Ok(cp));
    }

    #[test]
    fn a_free_price_of_zero_checkpoints_and_no_other_does() {
        // Amended at P2.4.11 (FREE-SPEC §6.1): a book price or EMA of +0.0 decodes, since a good
        // with a free step may post it and the book has no world to tell such a good from
        // another; validation against the world refuses it for every other good, and −0.0 still
        // does not decode.
        let (w, s) = evolved();
        let (town, bread) = (w.id_of("town").unwrap(), good(&w, "bread"));
        let mut zero = cp_of(&w, s.clone());
        let i = zero.state.book.index(town, bread).unwrap();
        zero.state.book.set_price(i, 0.0);
        for back in [
            Checkpoint::<TestExt>::from_bytes(&zero.to_bytes()),
            Checkpoint::<TestExt>::from_ron(&zero.to_ron()),
        ] {
            let back = back.expect("a clean 0 decodes");
            assert!(matches!(
                back.validate(&w),
                Err(CoreError::BadValue { value, .. }) if value == 0.0
            ));
        }
        let mut ema = cp_of(&w, s.clone());
        ema.state.book.set_ema(i, 0.0);
        assert!(matches!(
            Checkpoint::<TestExt>::from_bytes(&ema.to_bytes())
                .unwrap()
                .validate(&w),
            Err(CoreError::BadValue { .. })
        ));
        let mut neg = cp_of(&w, s);
        neg.state.book.set_price(i, -0.0);
        assert!(matches!(
            Checkpoint::<TestExt>::from_bytes(&neg.to_bytes()),
            Err(CheckpointError::Decode(_))
        ));
        // With a free step on bread its price of 0 checkpoints and validates.
        let text = testkit::ext_text()
            .replace(
                r#"(key: "bread", life: Years("life.bread"), price_rate: Some("rate.bread")),"#,
                r#"(key: "bread", life: Years("life.bread"), price_rate: Some("rate.bread"), free: Some((reference: "grain", scale: "free.bread"))),"#,
            )
            .replace(
                r#"(key: "pension.period","#,
                r#"(key: "free.bread", value: 0.5, unit: Dimensionless, basis: Assumed("c")),
        (key: "pension.period","#,
            );
        let t: Tape<TestExt> = Tape::from_ron(&text).expect("the free fixture parses");
        let (wf, sf) = crate::tape::resolve(&t).expect("the free fixture resolves");
        assert!(wf.free_step(good(&wf, "bread")).is_some());
        let mut cp = cp_of(&wf, sf);
        let i = cp.state.book.index(town, good(&wf, "bread")).unwrap();
        cp.state.book.set_price(i, 0.0);
        cp.state.book.set_ema(i, 0.0);
        let back = Checkpoint::<TestExt>::from_bytes(&cp.to_bytes()).unwrap();
        back.validate(&wf).expect("a free price of 0 validates");
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
        let mut nan_lot = cp_of(&w, s.clone());
        nan_lot.state.holdings.get_mut(&mill).unwrap().raw_mut()[0].1[0].qty = f64::NAN;
        let mut neg_param = cp_of(&w, s.clone());
        neg_param.state.params[1] = -0.0;
        let mut neg_price = cp_of(&w, s.clone());
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
        let mut cp = cp_of(&w, s.clone());
        cp.state.holdings.insert(ghost, Default::default());
        cases.push((cp, CoreError::UnknownHolder(ghost)));
        let mut cp = cp_of(&w, s.clone());
        testkit::write_direct(&mut cp.state, mill, GoodId(99), 1.0);
        cases.push((cp, CoreError::UnknownGood(GoodId(99))));
        let mut cp = cp_of(&w, s.clone());
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
        let mut cp = cp_of(&w, s.clone());
        cp.state.holdings.remove(&mill);
        assert!(matches!(cp.validate(&w), Err(CoreError::Shape(_))));
        let mut cp = cp_of(&w, s.clone());
        cp.state.holdings.get_mut(&mill).unwrap().raw_mut()[0].1[0].life = Some(40);
        assert!(matches!(cp.validate(&w), Err(CoreError::Shape(_))));
        let mut cp = cp_of(&w, s.clone());
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
            cp_of(&w, s).validate(&w2),
            Err(CoreError::Shape(_))
        ));
    }

    #[test]
    fn checkpoint_digest_covers_identity_and_run() {
        // O7, O8 (P0.9): under format 2 the digest covered the state alone, so a refused
        // checkpoint's world_id or prefix_id, edited to the value the refusal printed, resumed
        // under another world or another past. Format 3's digest covers world_id, prefix_id,
        // the state and the run's ledger, and an edit of any of them is refused in either form.
        let (w, s) = evolved();
        let cp = cp_of(&w, s.clone());
        let refused = |r: Result<Checkpoint<TestExt>, CheckpointError>| matches!(r, Err(CheckpointError::Digest { stored, .. }) if stored == cp.digest());
        // Bytes: world_id is at 12..20 and prefix_id at 20..28, after the magic and the format.
        for (at, value) in [(12, cp.world_id()), (20, cp.prefix_id())] {
            let mut b = cp.to_bytes();
            assert_eq!(b[at..at + 8], value.to_le_bytes());
            b[at..at + 8].copy_from_slice(&(value ^ 0x9e37).to_le_bytes());
            assert!(
                refused(Checkpoint::<TestExt>::from_bytes(&b)),
                "bytes at {at}"
            );
        }
        // RON: the same two fields, and the run's ledger.
        let text = cp.to_ron();
        for (from, to) in [
            (
                format!("world_id: {},", cp.world_id()),
                format!("world_id: {},", cp.world_id() ^ 1),
            ),
            (
                format!("prefix_id: {},", cp.prefix_id()),
                format!("prefix_id: {},", cp.prefix_id() ^ 1),
            ),
            ("since: 1,".to_string(), "since: 0,".to_string()),
        ] {
            assert_eq!(text.matches(&from).count(), 1, "{from}");
            let edited = text.replacen(&from, &to, 1);
            assert!(refused(Checkpoint::<TestExt>::from_ron(&edited)), "{from}");
        }
        assert_eq!(Checkpoint::<TestExt>::from_ron(&text), Ok(cp.clone()));
        // A run's ledger must fit the state it travels with: its next tick is the state's.
        let genesis = testkit::load_ext().1;
        let stale = RunLedger::open(&genesis, &w).unwrap();
        assert!(matches!(
            Checkpoint::of(&w, s.clone(), stale.clone()),
            Err(CoreError::Shape(_))
        ));
        let mut odd = cp.clone();
        odd.run = stale;
        assert!(matches!(odd.validate(&w), Err(CoreError::Shape(_))));
        cp.validate(&w).unwrap();
    }
}
