//! The tape compiler (PLAN §3.7). Human-editable tables of regions, parcels, deposits,
//! population, channels and dated timelines (technology, enclosure, laws and taxes, monetary
//! regimes, wars, world prices) compile into one validated tape with a signed genesis; the
//! compiler generates the ids and checks references and units. It fills in during Phase 4,
//! starting with England by historic county.
//!
//! [`atlas`] loads the county atlas (docs/GUI.md §6; data/atlas/README.md): the regions, their
//! shared borders and their measures. It lives here, with no egui, so that the GUI's map and the
//! compiler read one definition of the regions.

pub mod atlas;
