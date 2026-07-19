use serde::{Deserialize, Serialize};

/// Why a delta changes the total quantity of a good in existence.
///
/// Every delta that creates or destroys goods or currency carries one of these
/// (engine.md, "The conservation ledger"). `Transfer` — the default — means the
/// delta moves units between inventories and is conserved: it contributes
/// nothing to the mint/burn tally. The ledger reconciles the *observed* change
/// in Σ inventory against the sum of the non-`Transfer` lines, so a "transfer"
/// whose two halves fail to balance (goods shipped but never received, money
/// credited but never paid) shows up immediately as drift.
///
/// Deviations from the tag list in engine.md, recorded deliberately:
/// - `Transfer` and `Consumption` are added. Consumption is a real sink today
///   (pops consume their basket out of existence) and needs a burn class.
/// - `Magic` covers test-only `MagicProducer` phantom supply.
/// - `SelfProvision`, `Minting`, and `Recovery` are not yet emitted by any
///   mechanism and are deferred to the phases that introduce them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Provenance {
    /// Moves units between inventories. Conserved; not a mint or a burn.
    #[default]
    Transfer,
    /// Recipe transformation: outputs are minted, inputs are burned.
    Production,
    /// The pop demand sink — goods consumed out of existence.
    Consumption,
    /// Lots that reached the end of their shelf life.
    Spoilage,
    /// Labour supplied by pops. Pops hold no labour stock; it is minted against
    /// their headcount at the point of sale, and burned the same tick by
    /// production or by `Instant` spoilage.
    LabourMint,
    /// Injection or withdrawal scripted by the tape.
    Event,
    /// Test-only phantom supply from a `MagicProducer`, which holds no inventory.
    Magic,
}

impl Provenance {
    /// True when this delta changes the total quantity in existence, i.e. it is
    /// a mint or burn line rather than a conserved movement between inventories.
    pub fn is_mint_or_burn(self) -> bool {
        self != Provenance::Transfer
    }
}
