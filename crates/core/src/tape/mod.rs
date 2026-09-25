//! The tape: its schema, its resolution into a `World` and a genesis `SimState`, and the
//! world's identity (docs/ENGINE.md §2.6 and §5; ADDENDUM N1, N2; E8).
//!
//! The tape is the editable document and the only way anything enters a run (E1). The loader
//! reads one schema version, rejects unknown and missing fields, orders every keyed list by key
//! (never by file position), numbers each kind densely in key byte order, checks every
//! reference and unit, and sorts the events, which July binary-searched unsorted
//! (`v2p3: scenario/mod.rs:14, 25-42`). Every load error names its tape path.

pub mod raw;

use crate::clock::{Clock, ClockError, Date};
use crate::delta::{Provenance, StateDelta};
use crate::error::{LoadError, LoadErrorKind};
use crate::ext::Ext;
use crate::hash::Fnv;
use crate::ids::{
    ActorId, ActorKind, ChannelId, ClassId, DeskId, EventId, GoodId, Holder, Key, NodeId, ParamId,
    PopId,
};
use crate::inventory::{Amount, Inventory};
use crate::num::is_clean;
use crate::registry::{ParamDef, Registry};
use crate::state::{MarketBook, SimState};
use crate::units::{Unit, Years};
use crate::world::{
    ActorDecl, ChannelDef, Firing, GoodDef, KeyIndex, Life, MarketConfig, NodeDef, OneSided,
    PriceRule, Recurring, Schedule, ScheduleParam, Tolerances, World,
};
use raw::{
    RawAct, RawActorEntry, RawChannel, RawEvent, RawGenesis, RawGood, RawHeader, RawLife, RawNode,
    RawParam, RawRecurring,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The schema version this loader reads. Every schema change bumps it (docs/TAPE.md).
pub const SCHEMA: u32 = 1;

/// A tape: the editable document a run is made from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound = "")]
pub struct Tape<E: Ext> {
    /// `u32`. The schema version; this loader reads [`SCHEMA`] only. Required, no default.
    pub schema: u32,
    /// The header. Required, no default.
    pub header: RawHeader,
    /// Every registered param, each referenced at least once. Required, no default.
    pub params: Vec<RawParam>,
    /// The goods. Required, no default.
    pub goods: Vec<RawGood>,
    /// The market nodes. Required, no default.
    pub nodes: Vec<RawNode>,
    /// The static channels. Required, no default.
    pub channels: Vec<RawChannel>,
    /// The buyer and seller classes. Required, no default.
    pub classes: Vec<Key>,
    /// The actors, desks and pops. Required, no default.
    pub actors: Vec<RawActorEntry<E::RawActor>>,
    /// The genesis state. Required, no default.
    pub genesis: RawGenesis,
    /// The dated events. Required, no default.
    pub events: Vec<RawEvent<E::RawAction>>,
    /// The recurring entries. Required, no default.
    pub recurring: Vec<RawRecurring<E::RawAction>>,
}

/// Just the schema number, read first so that a tape of another version is refused as such
/// rather than as a list of unknown or missing fields.
#[derive(Deserialize)]
#[serde(rename = "Tape")]
struct SchemaProbe {
    schema: u32,
}

fn parse_error(e: ron::error::SpannedError) -> LoadError {
    LoadError::new("", LoadErrorKind::Parse(e.to_string()))
}

fn pretty() -> ron::ser::PrettyConfig {
    // An explicit new line, so the text is the same on every platform.
    ron::ser::PrettyConfig::new().new_line("\n".to_string())
}

impl<E: Ext> Tape<E> {
    /// Parse a tape. A schema other than [`SCHEMA`] is [`LoadErrorKind::Schema`]; anything the
    /// parser rejects is [`LoadErrorKind::Parse`], with its line and column. Nothing is checked
    /// beyond the schema here; [`resolve`] does that.
    pub fn from_ron(s: &str) -> Result<Tape<E>, LoadError> {
        if let Ok(probe) = ron::from_str::<SchemaProbe>(s) {
            check_schema(probe.schema)?;
        }
        let tape: Tape<E> = ron::from_str(s).map_err(parse_error)?;
        check_schema(tape.schema)?;
        Ok(tape)
    }

    /// The canonical text of the tape: every keyed list in key order, the genesis prices by
    /// (node, good), holdings by (holder, good), and the extension's own lists in its canonical
    /// order. Comments are not kept.
    pub fn to_ron(&self) -> String {
        let t = self.canonical();
        match ron::ser::to_string_pretty(&t, pretty()) {
            Ok(s) => s,
            // Unreachable for core's types; an extension whose `Serialize` fails gets text that
            // `from_ron` rejects.
            Err(e) => format!("/* the tape does not serialise: {e} */"),
        }
    }

    /// A copy with every list in canonical order.
    pub fn canonical(&self) -> Tape<E> {
        let mut t = self.clone();
        t.params.sort_by(|a, b| a.key.cmp(&b.key));
        t.goods.sort_by(|a, b| a.key.cmp(&b.key));
        t.nodes.sort_by(|a, b| a.key.cmp(&b.key));
        t.channels.sort_by(|a, b| a.key.cmp(&b.key));
        t.classes.sort();
        t.actors
            .sort_by(|a, b| (a.kind, &a.key).cmp(&(b.kind, &b.key)));
        for a in &mut t.actors {
            E::canonical_actor(&mut a.spec);
        }
        t.genesis
            .prices
            .sort_by(|a, b| (&a.node, &a.good).cmp(&(&b.node, &b.good)));
        t.genesis.holdings.sort_by(|a, b| a.holder.cmp(&b.holder));
        for h in &mut t.genesis.holdings {
            h.goods.sort_by(|a, b| a.0.cmp(&b.0));
        }
        t.events.sort_by(|a, b| a.key.cmp(&b.key));
        t.recurring.sort_by(|a, b| a.key.cmp(&b.key));
        for e in &mut t.events {
            if let RawAct::Actor(a) = &mut e.act {
                E::canonical_action(a);
            }
        }
        for r in &mut t.recurring {
            if let RawAct::Actor(a) = &mut r.act {
                E::canonical_action(a);
            }
        }
        t
    }
}

fn check_schema(found: u32) -> Result<(), LoadError> {
    if found == SCHEMA {
        Ok(())
    } else {
        Err(LoadError::new(
            "schema",
            LoadErrorKind::Schema {
                found,
                expected: SCHEMA,
            },
        ))
    }
}

/// How a reference uses a param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamUse {
    /// Read at use time from the current value; a dated `SetParam` may change it.
    Live,
    /// Turned into structure at load (a shelf life) or fixing what a past tick meant (a ledger
    /// tolerance). A fixed param cannot be set. The schedule's own references (the value a
    /// `SetParam` copies, a recurring period) are recorded apart, so that a param only the
    /// schedule reads can live in the schedule (docs/ENGINE.md §2.6).
    Fixed,
}

#[derive(Debug, Clone)]
struct ParamInfo {
    key: Key,
    unit: Unit,
    value: f64,
    /// Read at use time by the world.
    live: bool,
    /// Turned into world structure, or fixing a past tick's meaning.
    fixed: bool,
    /// Copied by a `SetParam`.
    copied: bool,
    /// A recurring entry's period.
    period: bool,
}

impl ParamInfo {
    /// Whether the loader turns it into structure, so no `SetParam` may target it.
    fn is_fixed(&self) -> bool {
        self.fixed || self.copied || self.period
    }

    /// Whether only the schedule reads it.
    fn schedule_only(&self) -> bool {
        (self.copied || self.period) && !self.live && !self.fixed
    }
}

/// Resolves keys to ids during loading, records every param reference with its unit and use,
/// and builds tape paths for errors. The extension resolves its specs and actions through it.
#[derive(Debug)]
pub struct Resolver<'a> {
    keys: &'a KeyIndex,
    currency: &'a [bool],
    clock: Clock,
    params: Vec<ParamInfo>,
    /// The params only the schedule reads, by key; not in `params`.
    schedule: &'a [ScheduleParam],
    path: Vec<String>,
}

impl<'a> Resolver<'a> {
    /// Descend into a path segment, such as `buy[village/grain]`.
    pub fn enter(&mut self, segment: impl Into<String>) {
        self.path.push(segment.into());
    }

    /// Leave the last segment entered.
    pub fn leave(&mut self) {
        self.path.pop();
    }

    /// The tape path of `field` under the current segments.
    pub fn path(&self, field: &str) -> String {
        let mut p = self.path.join(".");
        if !field.is_empty() {
            if !p.is_empty() {
                p.push('.');
            }
            p.push_str(field);
        }
        p
    }

    /// A load error at `field`.
    pub fn error(&self, field: &str, kind: LoadErrorKind) -> LoadError {
        LoadError::new(self.path(field), kind)
    }

    fn unknown(&self, field: &str, kind: &'static str, key: &Key) -> LoadError {
        self.error(
            field,
            LoadErrorKind::Unknown {
                kind,
                key: key.to_string(),
            },
        )
    }

    /// The good with this key.
    pub fn good(&self, key: &Key, field: &str) -> Result<GoodId, LoadError> {
        self.lookup(&self.keys.goods, key)
            .map(GoodId)
            .ok_or_else(|| self.unknown(field, "good", key))
    }

    /// The node with this key.
    pub fn node(&self, key: &Key, field: &str) -> Result<NodeId, LoadError> {
        self.lookup(&self.keys.nodes, key)
            .map(NodeId)
            .ok_or_else(|| self.unknown(field, "node", key))
    }

    /// The channel with this key.
    pub fn channel(&self, key: &Key, field: &str) -> Result<ChannelId, LoadError> {
        self.lookup(&self.keys.channels, key)
            .map(ChannelId)
            .ok_or_else(|| self.unknown(field, "channel", key))
    }

    /// The class with this key.
    pub fn class(&self, key: &Key, field: &str) -> Result<ClassId, LoadError> {
        self.lookup(&self.keys.classes, key)
            .map(ClassId)
            .ok_or_else(|| self.unknown(field, "class", key))
    }

    /// The actor with this key.
    pub fn actor(&self, key: &Key, field: &str) -> Result<ActorId, LoadError> {
        self.keys
            .actor(key.as_str())
            .ok_or_else(|| self.unknown(field, "actor", key))
    }

    fn lookup(&self, keys: &[Key], key: &Key) -> Option<u32> {
        keys.binary_search(key)
            .ok()
            .and_then(|i| u32::try_from(i).ok())
    }

    /// A reference to the param with this key, which must be registered with `unit`. The use is
    /// recorded: an unreferenced param does not load, and a param with a fixed use cannot be
    /// the target of a `SetParam`.
    pub fn param(
        &mut self,
        key: &Key,
        unit: Unit,
        use_: ParamUse,
        field: &str,
    ) -> Result<ParamId, LoadError> {
        let (p, registered) = self.param_any(key, use_, field)?;
        if registered != unit {
            return Err(self.error(
                field,
                LoadErrorKind::UnitMismatch {
                    key: key.to_string(),
                    registered,
                    expected: unit,
                },
            ));
        }
        Ok(p)
    }

    /// A reference to a param of any unit; returns its unit.
    fn param_any(
        &mut self,
        key: &Key,
        use_: ParamUse,
        field: &str,
    ) -> Result<(ParamId, Unit), LoadError> {
        let Some(i) = self.lookup(&self.keys.params, key) else {
            return Err(self.unknown(field, "param", key));
        };
        let info = &mut self.params[i as usize];
        match use_ {
            ParamUse::Live => info.live = true,
            ParamUse::Fixed => info.fixed = true,
        }
        Ok((ParamId(i), info.unit))
    }

    /// A fixed `Years` param as whole ticks; a value that rounds to 0 ticks does not load.
    pub fn ticks(&mut self, key: &Key, field: &str) -> Result<(ParamId, u32), LoadError> {
        let p = self.param(key, Unit::Years, ParamUse::Fixed, field)?;
        let v = self.value(p, field)?;
        self.whole_ticks(v, field).map(|t| (p, t))
    }

    /// `v` years as whole ticks; a value that rounds to 0 ticks does not load.
    fn whole_ticks(&self, v: f64, field: &str) -> Result<u32, LoadError> {
        self.clock.ticks(Years(v)).map_err(|e| {
            self.error(
                field,
                match e {
                    ClockError::ZeroTicks { years } => LoadErrorKind::ZeroTicks { years },
                    other => LoadErrorKind::Invalid(other.to_string()),
                },
            )
        })
    }

    /// A reference from the schedule (a `SetParam`'s source, a recurring period): the param's
    /// unit and value. A schedule param is read from the schedule's list; any other is a
    /// registered param, recorded as copied or as a period, which makes it fixed.
    fn schedule_ref(
        &mut self,
        key: &Key,
        period: bool,
        field: &str,
    ) -> Result<(Unit, f64), LoadError> {
        if let Ok(i) = self.schedule.binary_search_by(|p| p.key.cmp(key)) {
            let p = &self.schedule[i];
            return Ok((p.unit, p.value));
        }
        let Some(i) = self.lookup(&self.keys.params, key) else {
            return Err(self.unknown(field, "param", key));
        };
        let info = &mut self.params[i as usize];
        if period {
            info.period = true;
        } else {
            info.copied = true;
        }
        Ok((info.unit, info.value))
    }

    /// A recurring entry's period: a `Years` param as whole ticks.
    fn period(&mut self, key: &Key, field: &str) -> Result<u32, LoadError> {
        let (unit, v) = self.schedule_ref(key, true, field)?;
        if unit != Unit::Years {
            return Err(self.error(
                field,
                LoadErrorKind::UnitMismatch {
                    key: key.to_string(),
                    registered: unit,
                    expected: Unit::Years,
                },
            ));
        }
        self.whole_ticks(v, field)
    }

    /// A param's genesis value, for a reference the loader turns into structure (record it
    /// with [`ParamUse::Fixed`] first).
    pub fn value(&self, p: ParamId, field: &str) -> Result<f64, LoadError> {
        match self.params.get(p.idx()) {
            Some(info) => Ok(info.value),
            None => Err(self.error(field, LoadErrorKind::Invalid(format!("no {p}")))),
        }
    }

    /// Whether some node quotes its prices in `g`.
    pub fn is_currency(&self, g: GoodId) -> bool {
        self.currency.get(g.idx()).copied().unwrap_or(false)
    }

    /// The run's clock.
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// The tick a date falls in; before the start does not load.
    pub fn tick_of(&self, date: Date, field: &str) -> Result<u64, LoadError> {
        self.clock.tick_of(date).map_err(|e| {
            self.error(
                field,
                match e {
                    ClockError::BeforeStart { date, start } => {
                        LoadErrorKind::EventBeforeStart { date, start }
                    }
                    other => LoadErrorKind::Invalid(other.to_string()),
                },
            )
        })
    }

    /// A number that must be finite with a clear sign bit.
    pub fn quantity(&self, v: f64, field: &str) -> Result<f64, LoadError> {
        if is_clean(v) {
            Ok(v)
        } else {
            Err(self.error(field, LoadErrorKind::BadValue(v)))
        }
    }
}

/// Sort keys, rejecting a duplicate. The error names the first duplicate in key order.
fn sorted_keys<'k>(
    keys: impl Iterator<Item = &'k Key>,
    kind: &'static str,
    list: &str,
) -> Result<Vec<Key>, LoadError> {
    let mut v: Vec<Key> = keys.cloned().collect();
    v.sort();
    if let Some(w) = v.windows(2).find(|w| w[0] == w[1]) {
        return Err(LoadError::new(
            format!("{list}[{}]", w[0]),
            LoadErrorKind::Duplicate {
                kind,
                key: w[0].to_string(),
            },
        ));
    }
    Ok(v)
}

/// What a resolved action carries besides its delta.
struct Resolved<E: Ext> {
    action: StateDelta<E>,
    source: Option<Key>,
    /// A `SetParam`'s target and the path to name if it turns out fixed.
    target: Option<(ParamId, String)>,
}

fn resolve_act<E: Ext>(
    act: &RawAct<E::RawAction>,
    r: &mut Resolver<'_>,
) -> Result<Resolved<E>, LoadError> {
    let plain = |action| Resolved {
        action,
        source: None,
        target: None,
    };
    let amount = |r: &Resolver<'_>, a: &Amount, field: &str| match a {
        Amount::Qty(q) => r.quantity(*q, field).map(Amount::Qty),
        Amount::All => Ok(Amount::All),
    };
    Ok(match act {
        RawAct::Mint { holder, good, qty } => plain(StateDelta::Mint {
            to: Holder::Actor(r.actor(holder, "act.holder")?),
            good: r.good(good, "act.good")?,
            qty: r.quantity(*qty, "act.qty")?,
            prov: Provenance::Event,
        }),
        RawAct::Burn {
            holder,
            good,
            amount: a,
        } => plain(StateDelta::Burn {
            from: Holder::Actor(r.actor(holder, "act.holder")?),
            good: r.good(good, "act.good")?,
            amount: amount(r, a, "act.amount")?,
            prov: Provenance::Event,
        }),
        RawAct::Transfer {
            from,
            to,
            good,
            amount: a,
        } => plain(StateDelta::Transfer {
            from: Holder::Actor(r.actor(from, "act.from")?),
            to: Holder::Actor(r.actor(to, "act.to")?),
            good: r.good(good, "act.good")?,
            amount: amount(r, a, "act.amount")?,
        }),
        RawAct::SetParam { param, to } => {
            let (p, unit) = r.param_any(param, ParamUse::Live, "act.param")?;
            let (src_unit, value) = r.schedule_ref(to, false, "act.to")?;
            if unit != src_unit {
                return Err(r.error(
                    "act.to",
                    LoadErrorKind::SetParamAcrossUnits {
                        param: unit,
                        to: src_unit,
                    },
                ));
            }
            Resolved {
                action: StateDelta::SetParam { param: p, value },
                source: Some(to.clone()),
                target: Some((p, r.path("act.param"))),
            }
        }
        RawAct::Actor(a) => {
            r.enter("act");
            let d = E::resolve_action(a, r);
            r.leave();
            plain(StateDelta::Actor(d?))
        }
    })
}

/// The part of a world that decides what a run computes: everything but the schedule, the
/// tape's name and the basis texts. Its bincode 1 encoding, followed by the genesis state's,
/// is what `world_id` hashes.
#[derive(Serialize)]
#[serde(bound = "")]
struct RunContent<'w, E: Ext> {
    clock: &'w Clock,
    params: Vec<(&'w Key, Unit, f64, bool)>,
    tol: &'w Tolerances,
    market: &'w MarketConfig,
    goods: &'w [GoodDef],
    nodes: &'w [NodeDef],
    channels: &'w [ChannelDef],
    classes: &'w [Key],
    actors: &'w [ActorDecl<E::Actor>],
}

/// Resolve a tape into its world and genesis state, checking everything §2.6 lists. Nothing
/// here depends on file order: every list is read in key order.
///
/// A param only the schedule reads (a value a `SetParam` copies, a recurring period, and
/// nothing in the world) belongs to the schedule, not to the registry or the state, so that it
/// stays out of `world_id` (§2.6, E1). Which params those are is known only once everything has
/// resolved, since the extension's specs reference params too; so a first pass resolves with
/// every param registered and records each one's uses, and, when some param turns out to be the
/// schedule's alone, a second pass resolves again with those params moved to the schedule. The
/// first pass reports every load error.
pub fn resolve<E: Ext>(t: &Tape<E>) -> Result<(World<E>, SimState<E>), LoadError> {
    let (world, state, only) = resolve_with(t, &[])?;
    if only.is_empty() {
        return Ok((world, state));
    }
    let (world, state, again) = resolve_with(t, &only)?;
    if !again.is_empty() {
        // Unreachable: the second pass registers exactly the params the world referenced.
        return Err(LoadError::new(
            "params",
            LoadErrorKind::Invalid("the schedule's params did not settle".into()),
        ));
    }
    Ok((world, state))
}

/// What one resolution pass makes: the world, its genesis state, and the registered params that
/// turned out to be read by the schedule alone.
type Pass<E> = (World<E>, SimState<E>, Vec<Key>);

/// One resolution pass, with the params keyed in `schedule` held by the schedule.
fn resolve_with<E: Ext>(t: &Tape<E>, schedule: &[Key]) -> Result<Pass<E>, LoadError> {
    check_schema(t.schema)?;
    let h = &t.header;
    if h.ticks_per_year == 0 {
        return Err(LoadError::new(
            "header.ticks_per_year",
            LoadErrorKind::Invalid("ticks_per_year must be at least 1".into()),
        ));
    }
    let clock = Clock {
        start: h.start,
        ticks_per_year: h.ticks_per_year,
    };

    // Keys: sorted per kind, duplicates rejected; desks and pops share a namespace, and so do
    // events and recurring entries.
    sorted_keys(t.actors.iter().map(|a| &a.key), "actor", "actors")?;
    let event_keys = sorted_keys(
        t.events
            .iter()
            .map(|e| &e.key)
            .chain(t.recurring.iter().map(|r| &r.key)),
        "event",
        "events",
    )?;
    let by_kind = |kind: ActorKind| -> Vec<Key> {
        let mut v: Vec<Key> = t
            .actors
            .iter()
            .filter(|a| a.kind == kind)
            .map(|a| a.key.clone())
            .collect();
        v.sort();
        v
    };
    let keys = KeyIndex {
        goods: sorted_keys(t.goods.iter().map(|g| &g.key), "good", "goods")?,
        nodes: sorted_keys(t.nodes.iter().map(|n| &n.key), "node", "nodes")?,
        channels: sorted_keys(t.channels.iter().map(|c| &c.key), "channel", "channels")?,
        classes: sorted_keys(t.classes.iter(), "class", "classes")?,
        params: sorted_keys(t.params.iter().map(|p| &p.key), "param", "params")?
            .into_iter()
            .filter(|k| !schedule.contains(k))
            .collect(),
        desks: by_kind(ActorKind::Desk),
        pops: by_kind(ActorKind::Pop),
        events: event_keys,
    };

    // Params, in key order, each value finite and not negative: the registered ones, numbered
    // densely, and the schedule's own.
    let mut raw_params: Vec<&RawParam> = t.params.iter().collect();
    raw_params.sort_by(|a, b| a.key.cmp(&b.key));
    for p in &raw_params {
        if !is_clean(p.value) {
            return Err(LoadError::new(
                format!("params[{}].value", p.key),
                LoadErrorKind::BadValue(p.value),
            ));
        }
    }
    let (held, raw_params): (Vec<&RawParam>, Vec<&RawParam>) = raw_params
        .into_iter()
        .partition(|p| schedule.contains(&p.key));
    let sched: Vec<ScheduleParam> = held
        .iter()
        .map(|p| ScheduleParam {
            key: p.key.clone(),
            unit: p.unit,
            value: p.value,
            basis: p.basis.clone(),
        })
        .collect();
    let infos: Vec<ParamInfo> = raw_params
        .iter()
        .map(|p| ParamInfo {
            key: p.key.clone(),
            unit: p.unit,
            value: p.value,
            live: false,
            fixed: false,
            copied: false,
            period: false,
        })
        .collect();

    // Nodes first, since they decide which goods are currencies.
    let mut raw_nodes: Vec<&RawNode> = t.nodes.iter().collect();
    raw_nodes.sort_by(|a, b| a.key.cmp(&b.key));
    let mut currency = vec![false; keys.goods.len()];
    let mut nodes = Vec::with_capacity(raw_nodes.len());
    {
        let mut r = Resolver {
            keys: &keys,
            currency: &[],
            clock,
            params: Vec::new(),
            schedule: &[],
            path: Vec::new(),
        };
        for (i, n) in raw_nodes.iter().enumerate() {
            r.enter(format!("nodes[{}]", n.key));
            let c = r.good(&n.currency, "currency")?;
            r.leave();
            currency[c.idx()] = true;
            nodes.push(NodeDef {
                id: NodeId(i as u32),
                key: n.key.clone(),
                currency: c,
            });
        }
    }

    let mut r = Resolver {
        keys: &keys,
        currency: &currency,
        clock,
        params: infos,
        schedule: &sched,
        path: Vec::new(),
    };

    // Header references.
    r.enter("header");
    if h.market.rule == PriceRule::Ratio && h.market.one_sided == OneSided::Saturate {
        return Err(r.error("market.one_sided", LoadErrorKind::RatioWithSaturate));
    }
    let market = MarketConfig {
        rule: h.market.rule,
        one_sided: h.market.one_sided,
        ema_time_constant: r.param(
            &h.market.ema_time_constant,
            Unit::Years,
            ParamUse::Live,
            "market.ema_time_constant",
        )?,
    };
    let tol = Tolerances {
        rel_flow: r.param(
            &h.ledger.rel_flow,
            Unit::Dimensionless,
            ParamUse::Fixed,
            "ledger.rel_flow",
        )?,
        rel_stock: r.param(
            &h.ledger.rel_stock,
            Unit::Dimensionless,
            ParamUse::Fixed,
            "ledger.rel_stock",
        )?,
    };
    // A relative tolerance of 1 or more would pass a leak of the whole stock or flow, which
    // switches R2 off; it is refused as a structural check, not as a behavioural bound.
    for (field, p) in [
        ("ledger.rel_flow", tol.rel_flow),
        ("ledger.rel_stock", tol.rel_stock),
    ] {
        let v = r.value(p, field)?;
        if v >= 1.0 {
            return Err(r.error(field, LoadErrorKind::ToleranceNotBelowOne(v)));
        }
    }
    r.leave();

    // Goods.
    let mut raw_goods: Vec<&RawGood> = t.goods.iter().collect();
    raw_goods.sort_by(|a, b| a.key.cmp(&b.key));
    let mut goods = Vec::with_capacity(raw_goods.len());
    for (i, g) in raw_goods.iter().enumerate() {
        let id = GoodId(i as u32);
        r.enter(format!("goods[{}]", g.key));
        let life = match &g.life {
            RawLife::Indefinite => Life::Indefinite,
            RawLife::Instant => Life::Instant,
            RawLife::Years(k) => Life::Ticks(r.ticks(k, "life")?.1),
        };
        let price_rate = match &g.price_rate {
            Some(k) => Some(r.param(k, Unit::RatePerYear, ParamUse::Live, "price_rate")?),
            None => None,
        };
        if r.is_currency(id) {
            if life != Life::Indefinite {
                return Err(r.error("life", LoadErrorKind::CurrencyGood));
            }
            if price_rate.is_some() {
                return Err(r.error("price_rate", LoadErrorKind::CurrencyGood));
            }
        } else if price_rate.is_none() {
            return Err(r.error("price_rate", LoadErrorKind::NoPriceRate));
        }
        r.leave();
        goods.push(GoodDef {
            id,
            key: g.key.clone(),
            life,
            price_rate,
        });
    }

    // Channels.
    let mut raw_channels: Vec<&RawChannel> = t.channels.iter().collect();
    raw_channels.sort_by(|a, b| a.key.cmp(&b.key));
    let mut channels = Vec::with_capacity(raw_channels.len());
    for (i, c) in raw_channels.iter().enumerate() {
        r.enter(format!("channels[{}]", c.key));
        let from = r.node(&c.from, "from")?;
        let to = r.node(&c.to, "to")?;
        if from == to {
            return Err(r.error(
                "to",
                LoadErrorKind::Invalid("a channel must join two different nodes".into()),
            ));
        }
        r.leave();
        channels.push(ChannelDef {
            id: ChannelId(i as u32),
            key: c.key.clone(),
            from,
            to,
        });
    }

    // Actors: every desk by key, then every pop by key, which is ActorId order.
    let mut raw_actors: Vec<&RawActorEntry<E::RawActor>> = t.actors.iter().collect();
    raw_actors.sort_by(|a, b| (a.kind, &a.key).cmp(&(b.kind, &b.key)));
    let mut actors = Vec::with_capacity(raw_actors.len());
    let (mut n_desks, mut n_pops) = (0u32, 0u32);
    for a in &raw_actors {
        let id = match a.kind {
            ActorKind::Desk => {
                n_desks += 1;
                ActorId::Desk(DeskId(n_desks - 1))
            }
            ActorKind::Pop => {
                n_pops += 1;
                ActorId::Pop(PopId(n_pops - 1))
            }
        };
        r.enter(format!("actors[{}]", a.key));
        let class = r.class(&a.class, "class")?;
        let home = r.node(&a.home, "home")?;
        r.enter("spec");
        let spec = E::resolve_actor(&a.spec, &mut r);
        r.leave();
        let spec = spec?;
        r.leave();
        actors.push(ActorDecl {
            id,
            key: a.key.clone(),
            class,
            home,
            spec,
        });
    }

    // Genesis.
    let (book, holdings) = resolve_genesis(&t.genesis, &mut r, &goods, &nodes, &actors)?;
    let ext = E::genesis(&actors)?;

    // Events and recurring entries.
    let event_id = |k: &Key| -> Result<EventId, LoadError> {
        match keys.events.binary_search(k) {
            Ok(i) => Ok(EventId(i as u32)),
            Err(_) => Err(LoadError::new(
                format!("events[{k}]"),
                LoadErrorKind::Unknown {
                    kind: "event",
                    key: k.to_string(),
                },
            )),
        }
    };
    let mut targets: Vec<(ParamId, String)> = Vec::new();
    let mut once = Vec::with_capacity(t.events.len());
    for e in &t.events {
        r.enter(format!("events[{}]", e.key));
        let tick = r.tick_of(e.at, "at")?;
        let res = resolve_act::<E>(&e.act, &mut r)?;
        r.leave();
        targets.extend(res.target);
        once.push(Firing {
            tick,
            event: event_id(&e.key)?,
            occurrence: 0,
            action: res.action,
            source: res.source,
        });
    }
    let mut every = Vec::with_capacity(t.recurring.len());
    for e in &t.recurring {
        r.enter(format!("recurring[{}]", e.key));
        let first = r.tick_of(e.first, "first")?;
        let period = u64::from(r.period(&e.every, "every")?);
        let last = match e.last {
            Some(d) => {
                let l = r.tick_of(d, "last")?;
                if l < first {
                    return Err(r.error("last", LoadErrorKind::LastBeforeFirst));
                }
                Some(l)
            }
            None => None,
        };
        let res = resolve_act::<E>(&e.act, &mut r)?;
        r.leave();
        targets.extend(res.target);
        every.push(Recurring {
            event: event_id(&e.key)?,
            first,
            period,
            last,
            action: res.action,
            source: res.source,
        });
    }

    // Param uses: no SetParam on a fixed param, and every param referenced.
    targets.sort();
    for (p, path) in &targets {
        let info = &r.params[p.idx()];
        if info.is_fixed() {
            return Err(LoadError::new(
                path.clone(),
                LoadErrorKind::SetParamOnFixed {
                    key: info.key.to_string(),
                },
            ));
        }
    }
    for info in &r.params {
        if !(info.live || info.is_fixed()) {
            return Err(LoadError::new(
                format!("params[{}]", info.key),
                LoadErrorKind::UnusedParam,
            ));
        }
    }
    let only: Vec<Key> = r
        .params
        .iter()
        .filter(|info| info.schedule_only())
        .map(|info| info.key.clone())
        .collect();
    let registry = Registry::new(
        raw_params
            .iter()
            .zip(&r.params)
            .enumerate()
            .map(|(i, (raw, info))| ParamDef {
                id: ParamId(i as u32),
                key: raw.key.clone(),
                unit: raw.unit,
                genesis: raw.value,
                basis: raw.basis.clone(),
                fixed: info.is_fixed(),
            })
            .collect(),
    );
    let values: Vec<f64> = registry.params().iter().map(|p| p.genesis).collect();
    let state = SimState::genesis(values, book, holdings, ext);

    let mut world = World {
        name: h.name.clone(),
        world_id: 0,
        clock,
        registry,
        tol,
        market,
        goods,
        nodes,
        channels,
        classes: keys.classes.clone(),
        actors,
        schedule: Schedule::new(once, every, sched),
        keys,
        currency,
    };
    world.world_id = world_id(&world, &state);
    Ok((world, state, only))
}

type Genesis = (MarketBook, BTreeMap<Holder, Inventory>);

fn resolve_genesis<A>(
    g: &RawGenesis,
    r: &mut Resolver<'_>,
    goods: &[GoodDef],
    nodes: &[NodeDef],
    actors: &[ActorDecl<A>],
) -> Result<Genesis, LoadError> {
    // Prices: exactly one per (node, non-currency good), finite and positive; EMA = price.
    let mut prices: BTreeMap<(NodeId, GoodId), f64> = BTreeMap::new();
    for p in &g.prices {
        r.enter(format!("genesis.prices[{}/{}]", p.node, p.good));
        let n = r.node(&p.node, "node")?;
        let good = r.good(&p.good, "good")?;
        if r.is_currency(good) {
            return Err(r.error("good", LoadErrorKind::PriceOnCurrency));
        }
        if !(is_clean(p.price) && p.price > 0.0) {
            return Err(r.error("price", LoadErrorKind::BadValue(p.price)));
        }
        if prices.insert((n, good), p.price).is_some() {
            return Err(r.error(
                "",
                LoadErrorKind::Duplicate {
                    kind: "genesis price",
                    key: format!("{}/{}", p.node, p.good),
                },
            ));
        }
        r.leave();
    }
    let mut book = MarketBook::new(nodes.len(), goods.len());
    for n in nodes {
        for good in goods.iter().filter(|good| !r.is_currency(good.id)) {
            let Some(&p) = prices.get(&(n.id, good.id)) else {
                return Err(LoadError::new(
                    format!("genesis.prices[{}/{}]", n.key, good.key),
                    LoadErrorKind::MissingPrice,
                ));
            };
            let Some(i) = book.index(n.id, good.id) else {
                return Err(LoadError::new(
                    format!("genesis.prices[{}/{}]", n.key, good.key),
                    LoadErrorKind::Invalid("the book does not cover this market".into()),
                ));
            };
            book.set_price(i, p);
            book.set_ema(i, p);
        }
    }

    // Holdings: every declared actor holds an inventory; listed goods start as one lot with the
    // good's full life.
    let mut holdings: BTreeMap<Holder, Inventory> = actors
        .iter()
        .map(|a| (Holder::Actor(a.id), Inventory::new()))
        .collect();
    let mut seen: Vec<ActorId> = Vec::new();
    for hold in &g.holdings {
        r.enter(format!("genesis.holdings[{}]", hold.holder));
        let a = r.actor(&hold.holder, "holder")?;
        if seen.contains(&a) {
            return Err(r.error(
                "",
                LoadErrorKind::Duplicate {
                    kind: "genesis holding",
                    key: hold.holder.to_string(),
                },
            ));
        }
        seen.push(a);
        let mut listed: Vec<GoodId> = Vec::new();
        let inv = holdings.entry(Holder::Actor(a)).or_default();
        let mut sorted: Vec<&(Key, f64)> = hold.goods.iter().collect();
        sorted.sort_by(|x, y| x.0.cmp(&y.0));
        for (k, q) in sorted {
            let field = format!("goods[{k}]");
            let good = r.good(k, &field)?;
            if listed.contains(&good) {
                return Err(r.error(
                    &field,
                    LoadErrorKind::Duplicate {
                        kind: "held good",
                        key: k.to_string(),
                    },
                ));
            }
            listed.push(good);
            let q = r.quantity(*q, &field)?;
            let life = goods[good.idx()].life.initial();
            inv.put_qty(good, q, life)
                .map_err(|e| r.error(&field, LoadErrorKind::Invalid(e.to_string())))?;
        }
        r.leave();
    }
    Ok((book, holdings))
}

/// FNV-1a 64 over bincode 1 of the run content and the genesis state.
fn world_id<E: Ext>(w: &World<E>, genesis: &SimState<E>) -> u64 {
    let content = RunContent::<E> {
        clock: &w.clock,
        params: w
            .registry
            .params()
            .iter()
            .map(|p| (&p.key, p.unit, p.genesis, p.fixed))
            .collect(),
        tol: &w.tol,
        market: &w.market,
        goods: &w.goods,
        nodes: &w.nodes,
        channels: &w.channels,
        classes: &w.classes,
        actors: &w.actors,
    };
    let mut h = Fnv::new();
    h.write_serialized(&content);
    h.write_serialized(genesis);
    h.finish()
}

#[cfg(test)]
mod tests {
    //! The tape tests of docs/ENGINE.md §11 (N1, N2, R4, N13, E8, N11). Failure tests are
    //! variants of the one fixture, made by editing its text or the parsed tape.

    use super::*;
    use crate::hash::state_hash;
    use crate::registry::Basis;
    use crate::testkit::{self, edit, load_err, load_text, FIXTURE};
    use crate::NoExt;

    fn kind_at(e: &LoadError) -> (&str, &LoadErrorKind) {
        (e.path.as_str(), &e.kind)
    }

    /// A resolved action with its ids mapped back to keys, so worlds can be compared by key.
    fn describe(w: &World<NoExt>, d: &StateDelta<NoExt>) -> String {
        let h = |h: &Holder| match h {
            Holder::Actor(a) => w.key_of(*a).unwrap().to_string(),
            Holder::Escrow(n, g) => {
                format!("escrow {} {}", w.key_of(*n).unwrap(), w.key_of(*g).unwrap())
            }
        };
        let g = |g: &GoodId| w.key_of(*g).unwrap().to_string();
        match d {
            StateDelta::Mint {
                to,
                good,
                qty,
                prov,
            } => {
                format!("mint {} {} {qty:?} {prov:?}", h(to), g(good))
            }
            StateDelta::Burn {
                from,
                good,
                amount,
                prov,
            } => {
                format!("burn {} {} {amount:?} {prov:?}", h(from), g(good))
            }
            StateDelta::Transfer {
                from,
                to,
                good,
                amount,
            } => {
                format!("transfer {} {} {} {amount:?}", h(from), h(to), g(good))
            }
            StateDelta::SetParam { param, value } => {
                format!("set {} {value:?}", w.key_of(*param).unwrap())
            }
            other => format!("{other:?}"),
        }
    }

    fn firings(w: &World<NoExt>, ticks: std::ops::Range<u64>) -> Vec<(u64, String, u32, String)> {
        ticks
            .flat_map(|t| w.schedule.fire(t))
            .map(|f| {
                (
                    f.tick,
                    w.key_of(f.event).unwrap().to_string(),
                    f.occurrence,
                    describe(w, &f.action),
                )
            })
            .collect()
    }

    fn event_at(w: &World<NoExt>, key: &str, tick: u64, qty: f64) -> RawEvent<crate::Never> {
        RawEvent {
            key: Key::new(key).unwrap(),
            at: w.clock.date_of(tick).unwrap(),
            basis: Basis::Assumed("test".into()),
            act: RawAct::Mint {
                holder: Key::new("pensioners").unwrap(),
                good: Key::new("grain").unwrap(),
                qty,
            },
        }
    }

    #[test]
    fn unsorted_events_fire_in_order() {
        // July binary-searched an unsorted list: with ticks listed 5, 2, 9, 9 it lost two (N1).
        let (w, _) = testkit::load();
        let mut t = testkit::tape();
        for (key, tick, qty) in [
            ("e.five", 5, 5.0),
            ("e.two", 2, 2.0),
            ("e.nine.b", 9, 9.5),
            ("e.nine.a", 9, 9.0),
            ("z.last", 51, 51.5),
            ("a.first", 51, 51.0),
        ] {
            t.events.push(event_at(&w, key, tick, qty));
        }
        let (w, _) = resolve(&t).unwrap();
        let fired = firings(&w, 0..60);
        let keys: Vec<(u64, &str)> = fired.iter().map(|f| (f.0, f.1.as_str())).collect();
        assert_eq!(
            keys,
            vec![
                (2, "e.two"),
                (5, "e.five"),
                (9, "e.nine.a"),
                (9, "e.nine.b"),
                (51, "a.first"),
                (51, "pension"),
                (51, "z.last"),
            ],
            "every event fires, in (tick, key) order, merged with the recurring entries"
        );
        assert_eq!(fired[2].3, "mint pensioners grain 9.0 Event");
        // The recurring entry fires once a year from its first date, and counts occurrences.
        let pension: Vec<(u64, u32)> = firings(&w, 0..300)
            .into_iter()
            .filter(|f| f.1 == "pension")
            .map(|f| (f.0, f.2))
            .collect();
        assert_eq!(
            pension,
            vec![(51, 0), (103, 1), (155, 2), (207, 3), (259, 4)]
        );
    }

    #[test]
    fn recurring_last_is_inclusive() {
        // ENGINE §2.6: `last` is the last tick a recurring entry may fire in. The fixture's
        // pension fires at ticks 51, 103, 155, ...; with `last` on the date of tick 155 it fires
        // there and never after (N1), and the prefix a checkpoint stores covers exactly the
        // firings `fire` produced (N11), on both sides of `last`.
        let (w0, _) = testkit::load();
        let pension = |w: &World<NoExt>| -> Vec<(u64, u32)> {
            firings(w, 0..400)
                .into_iter()
                .filter(|f| f.1 == "pension")
                .map(|f| (f.0, f.2))
                .collect()
        };
        let fired_hash = |w: &World<NoExt>, until: u64| {
            let mut h = Fnv::new();
            for tick in 0..until {
                for f in w.schedule.fire(tick) {
                    let key = w.key_of(f.event).map_or("", Key::as_str);
                    h.write_serialized(&(f.tick, key, f.occurrence, &f.action));
                }
            }
            h.finish()
        };
        let last = 155;
        for (at, fires) in [
            (last, vec![(51, 0), (103, 1), (155, 2)]),
            (last - 1, vec![(51, 0), (103, 1)]),
            (last + 1, vec![(51, 0), (103, 1), (155, 2)]),
        ] {
            let mut t = testkit::tape();
            t.recurring[0].last = Some(w0.clock.date_of(at).unwrap());
            let (w, _) = resolve(&t).unwrap();
            assert_eq!(w.schedule.every()[0].last, Some(at));
            assert_eq!(pension(&w), fires, "last at {at}");
            for until in [0, 51, 52, 103, 104, 154, 155, 156, 157, 207, 208, 400] {
                assert_eq!(
                    w.prefix_id(until),
                    fired_hash(&w, until),
                    "last at {at}, prefix at {until}"
                );
            }
            // Until the entry would have fired past `last`, its prefix is the open entry's.
            let first_cut = fires.last().unwrap().0 + 1;
            assert_eq!(w.prefix_id(first_cut), w0.prefix_id(first_cut), "{at}");
            assert_ne!(w.prefix_id(208), w0.prefix_id(208), "{at}");
        }
    }

    #[test]
    fn every_zero_is_rejected() {
        // July's `every = 0` never fired (`v2p3: scenario/mod.rs:42`); now it does not load.
        for v in ["0.0", "0.005"] {
            let e = load_err(&edit(
                r#"(key: "pension.period", value: 1.0,"#,
                &format!(r#"(key: "pension.period", value: {v},"#),
            ));
            assert!(
                matches!(
                    kind_at(&e),
                    ("recurring[pension].every", LoadErrorKind::ZeroTicks { .. })
                ),
                "{e}"
            );
        }
        // A shelf life that rounds to 0 ticks does not load either.
        let e = load_err(&edit(
            r#"(key: "life.bread", value: 0.0577,"#,
            r#"(key: "life.bread", value: 0.001,"#,
        ));
        assert!(
            matches!(
                kind_at(&e),
                ("goods[bread].life", LoadErrorKind::ZeroTicks { .. })
            ),
            "{e}"
        );
    }

    #[test]
    fn undefined_good_is_a_load_error() {
        let cases = [
            (
                (r#"good: "grain", qty: 2.0"#, r#"good: "gold", qty: 2.0"#),
                "events[grain.gift].act.good",
            ),
            (
                (r#"("coin", 20.0)"#, r#"("gold", 20.0)"#),
                "genesis.holdings[workers].goods[gold]",
            ),
            (
                (
                    r#"(key: "town", currency: "coin")"#,
                    r#"(key: "town", currency: "gold")"#,
                ),
                "nodes[town].currency",
            ),
        ];
        for ((from, to), path) in cases {
            let e = load_err(&edit(from, to));
            assert_eq!(e.path, path);
            assert!(
                matches!(&e.kind, LoadErrorKind::Unknown { kind: "good", key } if key == "gold"),
                "{e}"
            );
        }
    }

    #[test]
    fn unknown_actor_or_param_is_a_load_error() {
        let cases = [
            (
                (
                    r#"holder: "pensioners", good: "grain""#,
                    r#"holder: "retirees", good: "grain""#,
                ),
                "events[grain.gift].act.holder",
                "actor",
            ),
            (
                (r#"(holder: "mill","#, r#"(holder: "baker","#),
                "genesis.holdings[baker].holder",
                "actor",
            ),
            (
                (
                    r#"price_rate: Some("rate.grain")"#,
                    r#"price_rate: Some("rate.gold")"#,
                ),
                "goods[grain].price_rate",
                "param",
            ),
            (
                (r#"every: "pension.period""#, r#"every: "pension.span""#),
                "recurring[pension].every",
                "param",
            ),
            (
                (
                    r#"(key: "mill", kind: Desk, class: "producers", home: "town""#,
                    r#"(key: "mill", kind: Desk, class: "producers", home: "city""#,
                ),
                "actors[mill].home",
                "node",
            ),
            (
                (
                    r#"(key: "farm", kind: Desk, class: "producers""#,
                    r#"(key: "farm", kind: Desk, class: "farmers""#,
                ),
                "actors[farm].class",
                "class",
            ),
        ];
        for ((from, to), path, what) in cases {
            let e = load_err(&edit(from, to));
            assert_eq!(e.path, path, "{e}");
            assert!(
                matches!(&e.kind, LoadErrorKind::Unknown { kind, .. } if *kind == what),
                "{e}"
            );
        }
    }

    #[test]
    fn duplicate_keys_are_rejected() {
        let good = r#"(key: "labour", life: Instant, price_rate: Some("rate.labour")),"#;
        let pop = r#"(key: "workers", kind: Pop,"#;
        let cases: Vec<(String, &str)> = vec![
            (
                edit(
                    good,
                    &format!(r#"{good} (key: "grain", life: Instant, price_rate: None),"#),
                ),
                "goods[grain]",
            ),
            // Desks and pops share one namespace.
            (edit(pop, r#"(key: "farm", kind: Pop,"#), "actors[farm]"),
            // Events and recurring entries share one namespace.
            (
                edit(r#"(key: "grain.gift", at:"#, r#"(key: "pension", at:"#),
                "events[pension]",
            ),
            (
                edit(
                    r#"(key: "rate.labour", value: 5.2,"#,
                    r#"(key: "rate.bread", value: 5.2,"#,
                ),
                "params[rate.bread]",
            ),
            (
                edit(r#"classes: ["households","#, r#"classes: ["producers","#),
                "classes[producers]",
            ),
            (
                edit(
                    r#"(node: "town", good: "grain", price: 1.0),"#,
                    r#"(node: "town", good: "grain", price: 1.0), (node: "town", good: "grain", price: 1.5),"#,
                ),
                "genesis.prices[town/grain]",
            ),
            (
                edit(
                    r#"(holder: "workers", goods: [("coin", 20.0)]),"#,
                    r#"(holder: "workers", goods: [("coin", 20.0)]), (holder: "workers", goods: []),"#,
                ),
                "genesis.holdings[workers]",
            ),
            (
                edit(
                    r#"goods: [("coin", 20.0)]"#,
                    r#"goods: [("coin", 20.0), ("coin", 1.0)]"#,
                ),
                "genesis.holdings[workers].goods[coin]",
            ),
        ];
        for (text, path) in cases {
            let e = load_err(&text);
            assert_eq!(e.path, path, "{e}");
            assert!(matches!(e.kind, LoadErrorKind::Duplicate { .. }), "{e}");
        }
    }

    #[test]
    fn missing_genesis_price_is_rejected() {
        // July priced every market at 1.0 when the tape gave none (`v2p3:
        // state/sim_state.rs:56,59`); now a missing price does not load.
        let e = load_err(&edit(
            r#"(node: "village", good: "labour", price: 0.8),"#,
            "",
        ));
        assert_eq!(
            e,
            LoadError::new(
                "genesis.prices[village/labour]",
                LoadErrorKind::MissingPrice
            )
        );
        let e = load_err(&edit(
            r#"(node: "town", good: "grain", price: 1.0),"#,
            r#"(node: "town", good: "grain", price: 1.0), (node: "town", good: "coin", price: 1.0),"#,
        ));
        assert_eq!(
            e,
            LoadError::new(
                "genesis.prices[town/coin].good",
                LoadErrorKind::PriceOnCurrency
            )
        );
        for bad in ["0.0", "-1.0", "-0.0", "NaN", "inf"] {
            let e = load_err(&edit(
                r#"(node: "town", good: "grain", price: 1.0),"#,
                &format!(r#"(node: "town", good: "grain", price: {bad}),"#),
            ));
            assert!(
                matches!(
                    kind_at(&e),
                    (
                        "genesis.prices[town/grain].price",
                        LoadErrorKind::BadValue(_)
                    )
                ),
                "{bad}: {e}"
            );
        }
        // The genesis EMA is the genesis price, and a currency's slots are 1 and 1.
        let (w, s) = testkit::load();
        let (town, village) = (w.id_of("town").unwrap(), w.id_of("village").unwrap());
        let (bread, coin) = (testkit::good(&w, "bread"), testkit::good(&w, "coin"));
        assert_eq!(
            (s.price(village, bread), s.ema(village, bread)),
            (Some(2.2), Some(2.2))
        );
        assert_eq!(
            (s.supply(town, bread), s.demand(town, bread)),
            (Some(0.0), Some(0.0))
        );
        assert_eq!(
            (s.price(town, coin), s.ema(town, coin)),
            (Some(1.0), Some(1.0))
        );
    }

    #[test]
    fn unused_param_is_rejected() {
        let e = load_err(&edit(
            "    params: [\n",
            "    params: [\n        (key: \"unused.dial\", value: 3.0, unit: Dimensionless, basis: Assumed(\"nobody reads it\")),\n",
        ));
        assert_eq!(
            e,
            LoadError::new("params[unused.dial]", LoadErrorKind::UnusedParam)
        );
    }

    #[test]
    fn unit_mismatch_is_rejected() {
        let e = load_err(&edit(
            r#"(key: "price.ema_tc", value: 0.5, unit: Years,"#,
            r#"(key: "price.ema_tc", value: 0.5, unit: RatePerYear,"#,
        ));
        assert_eq!(
            e,
            LoadError::new(
                "header.market.ema_time_constant",
                LoadErrorKind::UnitMismatch {
                    key: "price.ema_tc".into(),
                    registered: Unit::RatePerYear,
                    expected: Unit::Years,
                }
            )
        );
        let e = load_err(&edit(
            r#"(key: "rate.bread", value: 5.2, unit: RatePerYear,"#,
            r#"(key: "rate.bread", value: 5.2, unit: FlowPerYear,"#,
        ));
        assert_eq!(e.path, "goods[bread].price_rate");
        assert!(matches!(e.kind, LoadErrorKind::UnitMismatch { .. }));
    }

    #[test]
    fn event_before_start_is_rejected() {
        let e = load_err(&edit(r#"at: "1751-06-01""#, r#"at: "1749-06-01""#));
        assert!(
            matches!(
                kind_at(&e),
                (
                    "events[grain.gift].at",
                    LoadErrorKind::EventBeforeStart { .. }
                )
            ),
            "{e}"
        );
        let e = load_err(&edit(r#"first: "1750-12-31""#, r#"first: "1740-01-01""#));
        assert!(
            matches!(
                kind_at(&e),
                (
                    "recurring[pension].first",
                    LoadErrorKind::EventBeforeStart { .. }
                )
            ),
            "{e}"
        );
        let e = load_err(&edit("last: None", r#"last: Some("1750-06-01")"#));
        assert_eq!(
            e,
            LoadError::new("recurring[pension].last", LoadErrorKind::LastBeforeFirst)
        );
    }

    #[test]
    fn set_param_on_fixed_or_other_unit_is_rejected() {
        let act = r#"SetParam(param: "rate.grain", to: "rate.grain.high")"#;
        // A ledger tolerance is fixed: changing it would change what a past tick meant.
        let e = load_err(&edit(
            act,
            r#"SetParam(param: "ledger.rel_flow", to: "ledger.rel_stock")"#,
        ));
        assert_eq!(
            e,
            LoadError::new(
                "events[rate.up].act.param",
                LoadErrorKind::SetParamOnFixed {
                    key: "ledger.rel_flow".into()
                }
            )
        );
        // A shelf life is structure.
        let e = load_err(&edit(
            act,
            r#"SetParam(param: "life.bread", to: "pension.period")"#,
        ));
        assert!(
            matches!(e.kind, LoadErrorKind::SetParamOnFixed { .. }),
            "{e}"
        );
        // A value another SetParam copies is structure too.
        let text = edit(
            r#"act: Mint(holder: "pensioners", good: "grain", qty: 2.0)"#,
            r#"act: SetParam(param: "rate.grain.high", to: "rate.bread")"#,
        );
        let e = load_err(&text);
        assert_eq!(e.path, "events[grain.gift].act.param");
        assert!(
            matches!(e.kind, LoadErrorKind::SetParamOnFixed { .. }),
            "{e}"
        );
        // Across units.
        let e = load_err(&edit(
            act,
            r#"SetParam(param: "rate.grain", to: "price.ema_tc")"#,
        ));
        assert_eq!(
            e,
            LoadError::new(
                "events[rate.up].act.to",
                LoadErrorKind::SetParamAcrossUnits {
                    param: Unit::RatePerYear,
                    to: Unit::Years
                }
            )
        );
        // The good case: the value comes from the source at load, and its key stays on the
        // firing, so the new value keeps a basis. Only the schedule reads the source, so the
        // schedule holds it, not the registry.
        let (w, _) = testkit::load();
        let f = &w.schedule.once()[1];
        assert_eq!(w.key_of(f.event).unwrap().as_str(), "rate.up");
        assert_eq!(describe(&w, &f.action), "set rate.grain 10.4");
        assert_eq!(f.source.as_ref().map(Key::as_str), Some("rate.grain.high"));
        assert!(
            !w.registry
                .get(w.id_of("rate.grain").unwrap())
                .unwrap()
                .fixed
        );
        assert_eq!(w.id_of::<ParamId>("rate.grain.high"), None);
        let source = w.schedule.param("rate.grain.high").unwrap();
        assert_eq!((source.unit, source.value), (Unit::RatePerYear, 10.4));
    }

    #[test]
    fn schedule_params_stay_out_of_the_world() {
        // E1, §2.6: a param only the schedule reads (a SetParam's source, a recurring period)
        // is schedule content, like the events themselves. It is not registered, not in the
        // state and not in world_id, and the firings it shapes are in prefix_id. So a dated
        // SetParam to a new value, or a new period, keeps the world and every past.
        let (w0, s0) = testkit::load();
        let only: Vec<&str> = w0
            .schedule
            .params()
            .iter()
            .map(|p| p.key.as_str())
            .collect();
        assert_eq!(only, ["pension.period", "rate.grain.high"]);
        assert_eq!(w0.registry.len(), testkit::tape().params.len() - 2);
        assert_eq!(s0.param_values().len(), w0.registry.len());
        let rate_up = 103;
        assert_eq!(
            w0.schedule.fire(rate_up)[1].action,
            StateDelta::SetParam {
                param: w0.id_of("rate.grain").unwrap(),
                value: 10.4
            }
        );
        // A new value for the source: the same world, the same past until the SetParam fires.
        let (w, s) = load_text(&edit(
            r#"(key: "rate.grain.high", value: 10.4,"#,
            r#"(key: "rate.grain.high", value: 15.6,"#,
        ))
        .unwrap();
        assert_eq!((w.world_id, state_hash(&s)), (w0.world_id, state_hash(&s0)));
        assert_eq!(w.prefix_id(rate_up), w0.prefix_id(rate_up));
        assert_ne!(w.prefix_id(rate_up + 1), w0.prefix_id(rate_up + 1));
        // A new dial and a new event that sets it: the same world, the same past until then.
        let text = edit(
            "    params: [\n",
            "    params: [\n        (key: \"rate.grain.dial\", value: 20.8, unit: RatePerYear, basis: Assumed(\"a dial\")),\n",
        );
        let text = text.replacen(
            "    events: [\n",
            "    events: [\n        (key: \"dial\", at: \"1753-01-01\", basis: Assumed(\"a dial\"), act: SetParam(param: \"rate.grain\", to: \"rate.grain.dial\")),\n",
            1,
        );
        let (w, s) = load_text(&text).unwrap();
        assert_eq!((w.world_id, state_hash(&s)), (w0.world_id, state_hash(&s0)));
        let dial = w
            .schedule
            .once()
            .iter()
            .find(|f| w.key_of(f.event).unwrap().as_str() == "dial");
        let at = dial.unwrap().tick;
        assert_eq!(w.prefix_id(at), w0.prefix_id(at));
        assert_ne!(w.prefix_id(at + 1), w0.prefix_id(at + 1));
        // A new period: the same world; the past changes from the first firing that moves.
        let (w, s) = load_text(&edit(
            r#"(key: "pension.period", value: 1.0,"#,
            r#"(key: "pension.period", value: 2.0,"#,
        ))
        .unwrap();
        assert_eq!((w.world_id, state_hash(&s)), (w0.world_id, state_hash(&s0)));
        assert_eq!(w.prefix_id(rate_up), w0.prefix_id(rate_up));
        assert_ne!(w.prefix_id(rate_up + 1), w0.prefix_id(rate_up + 1));
        // A source the world also reads (here a price rate) stays registered and fixed.
        let (w, _) = load_text(&edit(
            r#"act: Mint(holder: "pensioners", good: "grain", qty: 2.0)"#,
            r#"act: SetParam(param: "rate.grain", to: "rate.bread")"#,
        ))
        .unwrap();
        let bread_rate = w.registry.get(w.id_of("rate.bread").unwrap()).unwrap();
        assert!(bread_rate.fixed);
        assert!(w.schedule.param("rate.bread").is_none());
        let gift = &w.schedule.once()[0];
        assert_eq!(w.key_of(gift.event).unwrap().as_str(), "grain.gift");
        assert_eq!(gift.source.as_ref().map(Key::as_str), Some("rate.bread"));
        // A period of the wrong unit is refused as a schedule reference too.
        let e = load_err(&edit(
            r#"every: "pension.period""#,
            r#"every: "rate.grain.high""#,
        ));
        assert!(
            matches!(
                kind_at(&e),
                (
                    "recurring[pension].every",
                    LoadErrorKind::UnitMismatch { .. }
                )
            ),
            "{e}"
        );
    }

    #[test]
    fn ledger_tolerances_must_be_below_one() {
        // R2: a relative tolerance of 1 or more would pass a leak of a whole stock or flow, so
        // a tape could switch conservation off. It does not load.
        for (from, field) in [
            (
                r#"(key: "ledger.rel_flow", value: 1e-12,"#,
                "header.ledger.rel_flow",
            ),
            (
                r#"(key: "ledger.rel_stock", value: 1e-11,"#,
                "header.ledger.rel_stock",
            ),
        ] {
            for v in ["1.0", "1e300"] {
                let to = from.replace(&from[from.find("value: ").unwrap()..], "")
                    + &format!("value: {v},");
                let e = load_err(&edit(from, &to));
                assert_eq!(e.path, field, "{e}");
                assert!(
                    matches!(e.kind, LoadErrorKind::ToleranceNotBelowOne(x) if x >= 1.0),
                    "{e}"
                );
            }
        }
        // Below 1 still loads: the check is structural, not a tuning bar.
        let text = edit(
            r#"(key: "ledger.rel_stock", value: 1e-11,"#,
            r#"(key: "ledger.rel_stock", value: 0.5,"#,
        );
        assert!(load_text(&text).is_ok());
    }

    #[test]
    fn ratio_with_saturate_is_rejected() {
        let market = "market: (rule: Imbalance, one_sided: Hold,";
        let e = load_err(&edit(market, "market: (rule: Ratio, one_sided: Saturate,"));
        assert_eq!(
            e,
            LoadError::new("header.market.one_sided", LoadErrorKind::RatioWithSaturate)
        );
        for ok in [
            "market: (rule: Ratio, one_sided: Hold,",
            "market: (rule: Imbalance, one_sided: Saturate,",
        ] {
            let (w, _) = load_text(&edit(market, ok)).unwrap();
            assert_ne!(
                w.world_id,
                testkit::load().0.world_id,
                "the rule is part of the world"
            );
        }
        // No rule, no load: the field has no default (N13).
        let e = load_err(&edit("rule: Imbalance, ", ""));
        assert!(matches!(e.kind, LoadErrorKind::Parse(_)), "{e}");
    }

    #[test]
    fn currencies_and_prices_rates_are_checked() {
        let e = load_err(&edit(
            r#"(key: "coin", life: Indefinite, price_rate: None),"#,
            r#"(key: "coin", life: Indefinite, price_rate: Some("rate.bread")),"#,
        ));
        assert_eq!(
            e,
            LoadError::new("goods[coin].price_rate", LoadErrorKind::CurrencyGood)
        );
        let e = load_err(&edit(
            r#"(key: "coin", life: Indefinite, price_rate: None),"#,
            r#"(key: "coin", life: Instant, price_rate: None),"#,
        ));
        assert_eq!(
            e,
            LoadError::new("goods[coin].life", LoadErrorKind::CurrencyGood)
        );
        let text = edit(
            r#"(key: "grain", life: Indefinite, price_rate: Some("rate.grain")),"#,
            r#"(key: "grain", life: Indefinite, price_rate: None),"#,
        );
        let e = load_err(&text);
        assert_eq!(
            e,
            LoadError::new("goods[grain].price_rate", LoadErrorKind::NoPriceRate)
        );
        // Values on the tape are finite with a clear sign bit.
        let e = load_err(&edit(r#"qty: 2.0)"#, r#"qty: -0.0)"#));
        assert!(
            matches!(
                kind_at(&e),
                ("events[grain.gift].act.qty", LoadErrorKind::BadValue(_))
            ),
            "{e}"
        );
        let e = load_err(&edit(
            r#"value: 5.2, unit: RatePerYear, basis: Assumed("as grain")"#,
            r#"value: NaN, unit: RatePerYear, basis: Assumed("as grain")"#,
        ));
        assert!(
            matches!(
                kind_at(&e),
                ("params[rate.labour].value", LoadErrorKind::BadValue(_))
            ),
            "{e}"
        );
        let e = load_err(&edit("ticks_per_year: 52,", "ticks_per_year: 0,"));
        assert_eq!(e.path, "header.ticks_per_year");
        let e = load_err(&edit(
            r#"(key: "road", from: "village", to: "town")"#,
            r#"(key: "road", from: "town", to: "town")"#,
        ));
        assert_eq!(e.path, "channels[road].to");
    }

    #[test]
    fn genesis_holdings_start_with_full_lives() {
        let (w, s) = testkit::load();
        let (farm, pensioners) = (
            testkit::holder(&w, "farm"),
            testkit::holder(&w, "pensioners"),
        );
        let (bread, coin) = (testkit::good(&w, "bread"), testkit::good(&w, "coin"));
        assert_eq!(w.good(bread).unwrap().life, Life::Ticks(3));
        let inv = s.holding(farm).unwrap();
        assert_eq!(
            inv.lots(bread),
            &[crate::Lot {
                qty: 4.0,
                life: Some(3)
            }]
        );
        assert_eq!(
            inv.lots(coin),
            &[crate::Lot {
                qty: 100.0,
                life: None
            }]
        );
        assert!(
            s.holding(pensioners).unwrap().is_empty(),
            "an unlisted actor holds nothing"
        );
        assert_eq!(s.holdings().len(), 4);
    }

    #[test]
    fn extension_errors_name_their_path() {
        let text = testkit::ext_text().replace(
            "    recurring: [",
            "    recurring: [\n        (key: \"bump\", first: \"1751-01-01\", every: \"pension.period\", last: None, basis: Assumed(\"x\"), act: Actor((actor: \"mill\", by: 0))),",
        );
        let t: Tape<testkit::TestExt> = Tape::from_ron(&text).unwrap();
        let e = resolve(&t).unwrap_err();
        assert_eq!(e.path, "recurring[bump].act.by");
        let t: Tape<testkit::TestExt> = Tape::from_ron(&text.replace("by: 0", "by: 2")).unwrap();
        let (w, _) = resolve(&t).unwrap();
        let bump = &w.schedule.every()[0];
        assert_eq!(w.key_of(bump.event).unwrap().as_str(), "bump");
        assert_eq!(
            bump.action,
            StateDelta::Actor(testkit::Bump {
                actor: w.id_of("mill").unwrap(),
                by: 2
            })
        );
    }

    #[test]
    fn schema_version_is_checked() {
        let e = load_err(&edit("schema: 1,", "schema: 2,"));
        assert_eq!(
            e,
            LoadError::new(
                "schema",
                LoadErrorKind::Schema {
                    found: 2,
                    expected: SCHEMA
                }
            )
        );
        // A newer tape with fields this loader does not know is refused for its schema, not for
        // the fields.
        let newer = edit("schema: 1,", "schema: 3,").replace(
            "ticks_per_year: 52,",
            "ticks_per_year: 52, calendar: Julian,",
        );
        let e = load_err(&newer);
        assert_eq!(
            e.kind,
            LoadErrorKind::Schema {
                found: 3,
                expected: SCHEMA
            }
        );
        // A tape built in code is checked as well.
        let mut t = testkit::tape();
        t.schema = 0;
        assert!(matches!(
            resolve(&t).unwrap_err().kind,
            LoadErrorKind::Schema { .. }
        ));
    }

    #[test]
    fn unknown_field_is_rejected() {
        for (from, to) in [
            (
                r#"(key: "coin", life: Indefinite, price_rate: None),"#,
                r#"(key: "coin", life: Indefinite, price_rate: None, colour: "gold"),"#,
            ),
            ("ticks_per_year: 52,", "ticks_per_year: 52, tick_length: 7,"),
            ("    recurring: [", "    extras: [],\n    recurring: ["),
            (
                r#"basis: Assumed("July REL_FLOW")"#,
                r#"basis: Measured(source: "x", vintage: "y", page: 3)"#,
            ),
        ] {
            let e = load_err(&edit(from, to));
            assert!(matches!(e.kind, LoadErrorKind::Parse(_)), "{e}");
        }
    }

    #[test]
    fn missing_optional_field_is_rejected() {
        // serde reads a missing `Option` as `None` unless told otherwise; here it is required.
        for (from, to) in [
            (
                r#"(key: "coin", life: Indefinite, price_rate: None),"#,
                r#"(key: "coin", life: Indefinite),"#,
            ),
            ("last: None,", ""),
            (
                r#", basis: Assumed("core fixture"), spec: ()),"#,
                r#", spec: ()),"#,
            ),
        ] {
            let text = FIXTURE.replacen(from, to, 1);
            assert_ne!(text, FIXTURE);
            let e = load_err(&text);
            assert!(
                matches!(&e.kind, LoadErrorKind::Parse(m) if m.contains("missing field")),
                "{e}"
            );
        }
    }

    #[test]
    fn tape_round_trips() {
        let t = testkit::tape();
        let text = t.to_ron();
        let back = Tape::<NoExt>::from_ron(&text).unwrap();
        assert_eq!(back, t.canonical());
        assert_eq!(back.to_ron(), text, "the canonical text is a fixed point");
        let (w1, s1) = resolve(&t).unwrap();
        let (w2, s2) = resolve(&back).unwrap();
        assert_eq!(w1.world_id, w2.world_id);
        assert_eq!(state_hash(&s1), state_hash(&s2));
        for tick in [0, 60, 80, 200] {
            assert_eq!(w1.prefix_id(tick), w2.prefix_id(tick));
        }
    }

    #[test]
    fn file_order_is_irrelevant() {
        let t = testkit::tape();
        let mut p = t.clone();
        p.params.reverse();
        p.goods.reverse();
        p.nodes.reverse();
        p.channels.reverse();
        p.classes.reverse();
        p.actors.rotate_left(1);
        p.actors.reverse();
        p.genesis.prices.reverse();
        p.genesis.holdings.reverse();
        for h in &mut p.genesis.holdings {
            h.goods.reverse();
        }
        p.events.reverse();
        p.recurring.reverse();
        assert_ne!(p, t);
        let (w1, s1) = resolve(&t).unwrap();
        let (w2, s2) = resolve(&p).unwrap();
        assert_eq!(w1.world_id, w2.world_id);
        assert_eq!(state_hash(&s1), state_hash(&s2));
        assert_eq!(w1.goods, w2.goods);
        assert_eq!(w1.actors, w2.actors);
        assert_eq!(w1.schedule, w2.schedule);
        for tick in [0, 51, 52, 80, 1000] {
            assert_eq!(w1.prefix_id(tick), w2.prefix_id(tick));
        }
        assert_eq!(p.to_ron(), t.to_ron());
    }

    #[test]
    fn reformatted_tape_keeps_its_ids() {
        let (w0, s0) = testkit::load();
        let variants = [
            FIXTURE.replace('\n', "\r\n"),
            FIXTURE.replace(",\n", ", // a comment\n"),
            FIXTURE
                .replace("\n", "\n    ")
                .replace("Tape(", "/* block */ Tape("),
            FIXTURE.replace(": ", ":   "),
            testkit::tape().to_ron(),
            // The name and the basis texts change no number the run computes.
            FIXTURE
                .replace("core-fixture", "renamed")
                .replace("three weeks", "21 days"),
        ];
        for text in variants {
            let (w, s) = load_text(&text).unwrap();
            assert_eq!(w.world_id, w0.world_id);
            assert_eq!(state_hash(&s), state_hash(&s0));
            assert_eq!(w.prefix_id(500), w0.prefix_id(500));
        }
        // Any number, unit, key or structure does change it.
        for (from, to) in [
            ("value: 0.5,", "value: 0.25,"),
            ("(\"coin\", 100.0)", "(\"coin\", 100.5)"),
            (
                "unit: Dimensionless, basis: Assumed(\"July REL_STOCK\")",
                "unit: Dimensionless, basis: Assumed(\"July REL_STOCK\")",
            ),
            (
                r#"classes: ["households", "producers"]"#,
                r#"classes: ["households", "makers", "producers"]"#,
            ),
            ("ticks_per_year: 52,", "ticks_per_year: 53,"),
        ]
        .into_iter()
        .filter(|(a, b)| a != b)
        {
            let (w, _) = load_text(&edit(from, to)).unwrap();
            assert_ne!(w.world_id, w0.world_id, "{from} -> {to}");
        }
    }

    #[test]
    fn new_entity_keeps_existing_ids() {
        let (w0, s0) = testkit::load();
        let mut t = testkit::tape();
        // Keys that sort first, so every dense id after them shifts.
        let mut cloth = t.goods[2].clone();
        cloth.key = Key::new("aaa.cloth").unwrap();
        t.goods.push(cloth);
        for node in ["town", "village"] {
            t.genesis.prices.push(raw::RawPrice {
                node: Key::new(node).unwrap(),
                good: Key::new("aaa.cloth").unwrap(),
                price: 3.0,
            });
        }
        let mut desk = t.actors[0].clone();
        desk.key = Key::new("aaa.desk").unwrap();
        t.actors.push(desk);
        t.events.push(event_at(&w0, "aaa.event", 7, 1.0));
        let (w, s) = resolve(&t).unwrap();
        assert_eq!(w.id_of::<GoodId>("grain"), Some(GoodId(3)), "ids shifted");
        // Every existing key names the same entity with the same resolved content.
        for g in &w0.goods {
            let n = w.good(w.id_of(g.key.as_str()).unwrap()).unwrap();
            assert_eq!(n.life, g.life);
            assert_eq!(
                n.price_rate.map(|p| w.key_of(p).unwrap()),
                g.price_rate.map(|p| w0.key_of(p).unwrap())
            );
            for node in &w0.nodes {
                let n2 = w.id_of(node.key.as_str()).unwrap();
                assert_eq!(s.price(n2, n.id), s0.price(node.id, g.id));
            }
        }
        for a in &w0.actors {
            let b = w.actor(w.id_of(a.key.as_str()).unwrap()).unwrap();
            assert_eq!(w.key_of(b.class), w0.key_of(a.class));
            assert_eq!(w.key_of(b.home), w0.key_of(a.home));
            let (h0, h1) = (
                s0.holding(Holder::Actor(a.id)).unwrap(),
                s.holding(Holder::Actor(b.id)).unwrap(),
            );
            for (g, q) in h0.goods() {
                let g1 = w.id_of::<GoodId>(w0.key_of(g).unwrap().as_str()).unwrap();
                assert_eq!(h1.get(g1), q);
                assert_eq!(h1.lots(g1), h0.lots(g));
            }
        }
        for p in w0.registry.params() {
            let q = w.registry.get(w.id_of(p.key.as_str()).unwrap()).unwrap();
            assert_eq!((q.unit, q.genesis, q.fixed), (p.unit, p.genesis, p.fixed));
        }
        let old: Vec<_> = firings(&w0, 0..400);
        let new: Vec<_> = firings(&w, 0..400)
            .into_iter()
            .filter(|f| f.1 != "aaa.event")
            .collect();
        assert_eq!(old, new);
    }

    #[test]
    fn prefix_id_covers_only_past_firings() {
        let (w0, _) = testkit::load();
        let gift = w0.schedule.once()[0].tick;
        assert_eq!(
            w0.key_of(w0.schedule.once()[0].event).unwrap().as_str(),
            "grain.gift"
        );
        // Editing a dated event changes the prefix only from the tick after it fires, and never
        // the world's identity.
        let (w, _) = load_text(&edit("qty: 2.0)", "qty: 2.5)")).unwrap();
        assert_eq!(w.world_id, w0.world_id);
        for t in [0, gift] {
            assert_eq!(w.prefix_id(t), w0.prefix_id(t), "{t}");
        }
        assert_ne!(w.prefix_id(gift + 1), w0.prefix_id(gift + 1));
        // Adding an event: the same.
        let mut t = testkit::tape();
        t.events.push(event_at(&w0, "late", 30, 1.0));
        let (w, _) = resolve(&t).unwrap();
        assert_eq!(w.world_id, w0.world_id);
        assert_eq!(w.prefix_id(30), w0.prefix_id(30));
        assert_ne!(w.prefix_id(31), w0.prefix_id(31));
        // A recurring entry is in the prefix from its first firing on.
        let (w, _) = load_text(&edit("qty: 5.0)", "qty: 6.0)")).unwrap();
        assert_eq!(w.prefix_id(51), w0.prefix_id(51));
        assert_ne!(w.prefix_id(52), w0.prefix_id(52));
        // Nothing has fired before tick 0.
        assert_eq!(w0.prefix_id(0), crate::hash::fnv1a_64(b""));
    }
}
