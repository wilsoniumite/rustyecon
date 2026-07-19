use crate::state::SimState;

/// Canonical, portable 64-bit fingerprint of the full simulation state.
///
/// `SimState` is composed entirely of ordered `Vec`s and scalars — no unordered
/// containers anywhere in it or the types it owns — and [`Inventory`] has a
/// canonical `Serialize` impl (sorted goods, lots with lives, exhausted lots
/// dropped). So the bincode encoding is deterministic across runs and machines,
/// and FNV-1a over those bytes is a stable state hash.
///
/// This is the primitive the golden-hash determinism tests compare (repeat /
/// resume / replay); it is equality-of-hash that matters, not any committed
/// constant, so a fast non-cryptographic hash is the right tool.
///
/// [`Inventory`]: crate::types::inventory::Inventory
pub fn state_hash(state: &SimState) -> u64 {
    let bytes = bincode::serialize(state).expect("SimState always serializes");
    fnv1a_64(&bytes)
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_states_hash_equal_and_tick_changes_it() {
        let a = SimState::new(3, 2);
        let b = SimState::new(3, 2);
        assert_eq!(state_hash(&a), state_hash(&b));

        let mut c = a.clone();
        c.tick += 1;
        assert_ne!(state_hash(&a), state_hash(&c), "tick must perturb the hash");
    }
}
