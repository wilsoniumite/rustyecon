//! Where a branch starts (docs/GUI.md §5.1 item 5; D4): from its parent's latest ring
//! checkpoint whose prefix the branch shares, or from genesis, with the reason.
//!
//! A branch can resume only in its parent's world: the same `world_id`. The ring is scanned
//! from the newest checkpoint; the first whose stored `prefix_id` equals the branch world's
//! `prefix_id` at the checkpoint's tick is the resume point, since every firing before that
//! tick is the same in both, so the state there is the branch's too. A checkpoint at genesis
//! gains nothing over `Sim::new`, so the branch reruns instead. `Sim::resume` has the final
//! word: the Runner refuses a checkpoint it does not accept, reruns from genesis and says why.

use crate::run::{RingCheckpoint, Store};
use certify::Hex;
use rustyecon_engine::prelude::*;

/// Where a branch starts.
#[derive(Debug, Clone)]
pub enum Plan {
    /// From this ring checkpoint of the parent.
    Resume(RingCheckpoint),
    /// From genesis, and why.
    Rerun {
        /// Why no checkpoint serves.
        reason: String,
    },
}

/// The plan for a branch of world `child` from the run `parent` recorded.
pub fn plan(parent: &Store, child: &World) -> Plan {
    let Some(pw) = parent.world() else {
        return Plan::Rerun {
            reason: "the parent has no loaded world".to_string(),
        };
    };
    if pw.world_id != child.world_id {
        return Plan::Rerun {
            reason: format!(
                "the edit changes the world: world_id {} became {}, so no checkpoint of the \
                 parent can resume it",
                Hex(pw.world_id),
                Hex(child.world_id)
            ),
        };
    }
    let agrees = parent
        .ring()
        .iter()
        .rev()
        .find(|cp| cp.prefix_id() == child.prefix_id(cp.tick()));
    match agrees {
        Some(cp) if cp.tick() > 0 => Plan::Resume(cp.clone()),
        Some(_) => Plan::Rerun {
            reason: "the only ring checkpoint of the parent that shares the branch's schedule \
                     is genesis"
                .to_string(),
        },
        None => Plan::Rerun {
            reason: "no ring checkpoint of the parent shares the branch's schedule".to_string(),
        },
    }
}
