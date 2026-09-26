//! What the batteries read: one [`Obs`] per tick, made from the engine's `TickReport` and the
//! providers' own records, and the run's [`Segment`]s, which restart the stability windows at
//! each dated shock (docs/CERTIFY.md §5; N15, REPORT §6 criterion 3).
//!
//! An `Obs` holds numbers as the engine reported them, NaN included: the batteries check them
//! (the finite gate, §6) before any predicate reads them.

use crate::fold;
use rustyecon_engine::prelude::{
    ActorId, ClassId, GoodId, Key, NodeId, Provenance, SideTag, Sim, StateDelta, TickReport, World,
};
use rustyecon_engine::rustyecon_agents::ActorState;
use rustyecon_engine::rustyecon_markets::MarketFill;
use serde::{Deserialize, Serialize};

/// One market's tick, as its `MarketLine` reported it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarketObs {
    /// The node.
    pub node: NodeId,
    /// The good.
    pub good: GoodId,
    /// The posted price this tick's settlement used.
    pub price: f64,
    /// The price posted for the next tick.
    pub next_price: f64,
    /// `S`.
    pub supply: f64,
    /// Feasible `D`.
    pub demand: f64,
    /// The good that left the escrow for the buyers.
    pub cleared: f64,
    /// The buyers' fill.
    pub buyer_fill: f64,
    /// The sellers' fill.
    pub seller_fill: f64,
}

impl MarketObs {
    /// Whether the market traded: markets' own predicate, `MarketFill::trades`, as
    /// `MarketLine::trades` reads it. Read only after the finite gate: a NaN reads as no trade.
    pub fn trades(&self) -> bool {
        MarketFill {
            node: self.node,
            good: self.good,
            supply: self.supply,
            demand: self.demand,
            buyer_fill: self.buyer_fill,
            seller_fill: self.seller_fill,
        }
        .trades()
    }
}

/// One rationing line (R12): a (market, class, side) that had an order this tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RationObs {
    /// The market's index in the tick's market list, (node, good) order.
    pub market: u32,
    /// The class.
    pub class: ClassId,
    /// The side.
    pub side: SideTag,
    /// What the class asked for.
    pub requested: f64,
    /// What its budgets could pay for (a seller class: what it offered).
    pub feasible: f64,
    /// What it got (a seller class: what it shipped).
    pub filled: f64,
}

/// One tick, as the batteries and reports read it.
#[derive(Debug, Clone, PartialEq)]
pub struct Obs {
    /// The tick that ran.
    pub tick: u64,
    /// Every market, in (node, good) order.
    pub markets: Vec<MarketObs>,
    /// Every rationing line, in the report's (node, good, class, side) order.
    pub rationing: Vec<RationObs>,
    /// Per good, by id: what `Consumption` burned, `−Σ` of its ledger lines.
    pub consumed: Vec<f64>,
    /// Per good, by id: what spoiled, `−Σ` of its `Spoilage` lines.
    pub spoiled: Vec<f64>,
    /// Each provider's transfer this tick, in `ActorId` order: (actor, due, paid).
    pub transfers: Vec<(ActorId, f64, f64)>,
    /// The tick's ledger margin (`audit.max_margin`).
    pub margin: f64,
    /// The run's ledger margin so far (`run.max_margin`).
    pub run_margin: f64,
    /// The `ScalePrice` firings this tick.
    pub price_shocks: u32,
}

/// `−Σ` of a tick's ledger lines of `prov` for each good, by id: a left fold from 0 in the
/// audit's (good, provenance) order, as the probe summed its spoilage.
pub fn burned(r: &TickReport, n_goods: usize, prov: Provenance) -> Vec<f64> {
    (0..n_goods)
        .map(|g| {
            -r.audit
                .lines
                .iter()
                .filter(|(good, p, _)| good.idx() == g && *p == prov)
                .fold(0.0, |acc, (_, _, q)| acc + q)
        })
        .collect()
}

impl Obs {
    /// The observation of the tick `r` reports, read after the step that made it: the report,
    /// and each provider's record of its transfer from the `Sim`.
    pub fn of(r: &TickReport, sim: &Sim) -> Obs {
        let w = sim.world();
        let markets: Vec<MarketObs> = r
            .markets
            .iter()
            .map(|l| MarketObs {
                node: l.node,
                good: l.good,
                price: l.price,
                next_price: l.next_price,
                supply: l.supply,
                demand: l.demand,
                cleared: l.cleared,
                buyer_fill: l.buyer_fill,
                seller_fill: l.seller_fill,
            })
            .collect();
        let rationing = r
            .rationing
            .iter()
            .map(|l| RationObs {
                market: markets
                    .iter()
                    .position(|m| m.node == l.node && m.good == l.good)
                    .and_then(|i| u32::try_from(i).ok())
                    .unwrap_or(u32::MAX),
                class: l.class,
                side: l.side,
                requested: l.requested,
                feasible: l.feasible,
                filled: l.filled,
            })
            .collect();
        let transfers = w
            .actors
            .iter()
            .filter_map(|a| match sim.actor_state(a.id) {
                Some(ActorState::Provider(p)) => Some((a.id, p.due, p.paid)),
                _ => None,
            })
            .collect();
        Obs {
            tick: r.tick,
            markets,
            rationing,
            consumed: burned(r, w.n_goods(), Provenance::Consumption),
            spoiled: burned(r, w.n_goods(), Provenance::Spoilage),
            transfers,
            margin: r.audit.max_margin,
            run_margin: r.run.max_margin,
            price_shocks: u32::try_from(
                r.events
                    .iter()
                    .filter(|e| matches!(e.action, StateDelta::ScalePrice { .. }))
                    .count(),
            )
            .unwrap_or(u32::MAX),
        }
    }
}

/// The names a certificate gives what it scores: each market as `node/good`, each good, class
/// and actor by its key, in id order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Names {
    /// Each market, `node/good`, in (node, good) order.
    pub markets: Vec<String>,
    /// Each good, by id.
    pub goods: Vec<String>,
    /// Each class, by id.
    pub classes: Vec<String>,
    /// Each actor, in `ActorId` order.
    pub actors: Vec<(ActorId, String)>,
}

impl Names {
    /// The names of a world's markets, goods, classes and actors.
    pub fn of(w: &World) -> Names {
        let key = |k: Option<&Key>| k.map_or_else(|| "?".to_string(), |k| k.to_string());
        Names {
            markets: w
                .markets()
                .map(|(n, g)| format!("{}/{}", key(w.key_of(n)), key(w.key_of(g))))
                .collect(),
            goods: w.goods.iter().map(|g| g.key.to_string()).collect(),
            classes: w.classes.iter().map(|c| c.to_string()).collect(),
            actors: w.actors.iter().map(|a| (a.id, a.key.to_string())).collect(),
        }
    }

    /// Market `m`'s name.
    pub fn market(&self, m: usize) -> String {
        self.markets
            .get(m)
            .cloned()
            .unwrap_or_else(|| format!("market {m}"))
    }

    /// Good `g`'s name.
    pub fn good(&self, g: usize) -> String {
        self.goods
            .get(g)
            .cloned()
            .unwrap_or_else(|| format!("good {g}"))
    }

    /// Class `c`'s name.
    pub fn class(&self, c: ClassId) -> String {
        self.classes
            .get(c.idx())
            .cloned()
            .unwrap_or_else(|| c.to_string())
    }

    /// An actor's name.
    pub fn actor(&self, a: ActorId) -> String {
        self.actors
            .iter()
            .find(|(id, _)| *id == a)
            .map_or_else(|| a.to_string(), |(_, k)| k.clone())
    }
}

/// A stretch of the run between dated shocks, `[from, to)`: the unit Settles, Balance, Kick and
/// the reports restart at (§5, C5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    /// Its first tick.
    pub from: u64,
    /// The tick after its last.
    pub to: u64,
    /// The keys of the dated events that fired at `from` and opened it; none for the first.
    pub opened_by: Vec<Key>,
    /// The keys of the dated events that fell inside it, each too close to the last boundary
    /// to close a segment of its own.
    pub merged: Vec<Key>,
}

impl Segment {
    /// Its length in ticks.
    pub fn len(&self) -> u64 {
        self.to.saturating_sub(self.from)
    }

    /// Whether it has no ticks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Its window from a share `share` of the way in to its end (§5): W or F.
    pub fn window(&self, share: f64) -> (u64, u64) {
        (fold::window_start(self.from, self.len(), share), self.to)
    }
}

/// The ticks the kick fires at (§7, amended at S2.5): every distinct tick inside `(from, until)`
/// at which a dated event fires (`w.schedule.once()`, recurring entries not included), whether or
/// not it closes a segment, and `until`, in order. A kick at T probes the regime in force before
/// T, so every regime is kicked, however short: a shock that C5 merges into its segment still
/// ends one.
pub fn kick_ticks(w: &World, from: u64, until: u64) -> Vec<u64> {
    let mut out: Vec<u64> = w
        .schedule
        .once()
        .iter()
        .map(|f| f.tick)
        .filter(|&t| from < t && t < until)
        .collect();
    out.push(until);
    out.sort_unstable();
    out.dedup();
    out
}

/// The index of the segment that holds tick `t − 1`, the one whose regime a kick at `t` probes.
pub fn segment_before(segments: &[Segment], t: u64) -> Option<usize> {
    segments.iter().position(|s| s.from < t && t <= s.to)
}

/// The run's segments over `[from, until)`: a dated event (`w.schedule.once()`, recurring entries
/// not included) at a tick b inside `(from, until)` closes the current segment `[a, b)` if b − a
/// ≥ `min_len`, and joins its `merged` list otherwise. A last segment shorter than `min_len`
/// joins the one before it. So every segment but a lone one is at least `min_len` long, and no
/// window depends on the tape's event spacing (N15, C5).
pub fn segments(w: &World, from: u64, until: u64, min_len: u64) -> Vec<Segment> {
    // Each distinct firing tick with the keys that fire there, in order.
    let mut boundaries: Vec<(u64, Vec<Key>)> = Vec::new();
    for f in w.schedule.once() {
        if f.tick <= from || f.tick >= until {
            continue;
        }
        let key: Vec<Key> = w.key_of(f.event).cloned().into_iter().collect();
        match boundaries.last_mut() {
            Some((t, keys)) if *t == f.tick => keys.extend(key),
            _ => boundaries.push((f.tick, key)),
        }
    }
    let mut out: Vec<Segment> = Vec::new();
    let mut current = Segment {
        from,
        to: until,
        opened_by: Vec::new(),
        merged: Vec::new(),
    };
    for (b, keys) in boundaries {
        if b - current.from >= min_len {
            current.to = b;
            out.push(current);
            current = Segment {
                from: b,
                to: until,
                opened_by: keys,
                merged: Vec::new(),
            };
        } else {
            current.merged.extend(keys);
        }
    }
    current.to = until;
    match out.last_mut() {
        Some(prev) if current.len() < min_len => {
            prev.to = current.to;
            prev.merged.extend(current.opened_by);
            prev.merged.extend(current.merged);
        }
        _ => out.push(current),
    }
    out
}
