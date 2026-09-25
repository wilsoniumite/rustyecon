//! The state hash (docs/ENGINE.md §2.5, R8), salvaged from `v2p3: certify/hash.rs`.
//!
//! FNV-1a 64 over the bincode 1 encoding. Every container in `SimState` is a `Vec` in id order
//! or a `BTreeMap`, and `Inventory` serialises canonically (goods ascending, lots by life,
//! coalesced), so the bytes, and the hash, depend only on the state. Equality of hashes is what
//! matters (repeat, resume, replay), so a fast non-cryptographic hash is the right tool.

use crate::ext::Ext;
use crate::state::SimState;
use serde::Serialize;

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// An incremental FNV-1a 64 hasher.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Fnv(u64);

impl Fnv {
    pub(crate) fn new() -> Fnv {
        Fnv(OFFSET_BASIS)
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }

    /// Fold in the bincode 1 encoding of `value`. Core's types always encode (every sequence
    /// has a known length, and no impl raises an error); should an extension's `Serialize` fail,
    /// its message is folded in behind a marker byte instead, which no encoding of a value
    /// begins with in that position, so the result still never equals a valid state's.
    pub(crate) fn write_serialized<T: Serialize + ?Sized>(&mut self, value: &T) {
        match bincode::serialize(value) {
            Ok(bytes) => self.write(&bytes),
            Err(e) => {
                self.write(&[0xff, 0xfe, 0xfd]);
                self.write(e.to_string().as_bytes());
            }
        }
    }

    pub(crate) fn finish(self) -> u64 {
        self.0
    }
}

/// FNV-1a 64 over a byte string.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h = Fnv::new();
    h.write(bytes);
    h.finish()
}

/// The hash of the whole state: tick, params, market book, every holding with its lot lives,
/// and the extension state. It does not cover the tape.
pub fn state_hash<E: Ext>(s: &SimState<E>) -> u64 {
    let mut h = Fnv::new();
    h.write_serialized(s);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delta::{Phase, Provenance, StateDelta};
    use crate::ids::{ActorId, DeskId, GoodId, Holder, ParamId};
    use crate::testkit::{self, Bump};

    #[test]
    fn fnv_matches_the_published_vectors() {
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a_64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn identical_states_hash_equal_and_tick_changes_it() {
        let (w, a) = testkit::load();
        let (_, b) = testkit::load();
        assert_eq!(state_hash(&a), state_hash(&b));
        let mut c = a.clone();
        testkit::apply_ok(&mut c, &w, Phase::Prices, &[StateDelta::AdvanceTick]);
        assert_ne!(
            state_hash(&a),
            state_hash(&c),
            "the tick must perturb the hash"
        );
    }

    #[test]
    fn hash_covers_holdings_lives_params_and_ext_state() {
        let (w, base) = testkit::load_ext();
        let h0 = state_hash(&base);
        let farm = Holder::Actor(ActorId::Desk(DeskId(0)));
        let bread = w.id_of::<GoodId>("bread").unwrap();

        // A holding's quantity.
        let mut s = base.clone();
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Production,
            &[StateDelta::Mint {
                to: farm,
                good: bread,
                qty: 1.0,
                prov: Provenance::Production,
            }],
        );
        assert_ne!(state_hash(&s), h0, "holdings");

        // A lot's life alone, with every quantity unchanged.
        let mut s = base.clone();
        let raw = s.holdings.get_mut(&farm).unwrap().raw_mut();
        let lots = &mut raw.iter_mut().find(|(g, _)| *g == bread).unwrap().1;
        let before: f64 = lots.iter().map(|l| l.qty).sum();
        lots[0].life = lots[0].life.map(|n| n - 1);
        let after: f64 = lots.iter().map(|l| l.qty).sum();
        assert_eq!(before, after);
        assert_ne!(state_hash(&s), h0, "lot lives");

        // A param's current value.
        let mut s = base.clone();
        let rate = w.id_of::<ParamId>("rate.grain").unwrap();
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Events,
            &[StateDelta::SetParam {
                param: rate,
                value: 7.0,
            }],
        );
        assert_ne!(state_hash(&s), h0, "params");

        // The extension state.
        let mut s = base.clone();
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Decisions,
            &[StateDelta::Actor(Bump {
                actor: ActorId::Desk(DeskId(1)),
                by: 1,
            })],
        );
        assert_ne!(state_hash(&s), h0, "extension state");

        // And the book.
        let mut s = base.clone();
        let town = w.id_of("town").unwrap();
        testkit::apply_ok(
            &mut s,
            &w,
            Phase::Prices,
            &[StateDelta::SetEma {
                node: town,
                good: bread,
                ema: 2.5,
            }],
        );
        assert_ne!(state_hash(&s), h0, "book");
    }
}
