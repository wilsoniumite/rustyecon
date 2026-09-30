//! The batteries and the reports (docs/CERTIFY.md §6), each a pure function of what it reads, so
//! that each has a test that fails it.
//!
//! **The finite gate (N12).** Before any predicate, every field in a battery's "Reads" column
//! is checked with `is_finite` on every tick, market and run it scores. The first failure fails
//! the battery, and its note names the tick and the path. After the gate, a comparison is
//! written `x <= bar`, never `!(x > bar)`, and every max and min propagates NaN. This closes
//! July's leaks: statistics that dropped NaN one sample at a time, `fold(0.0, f64::max)`, and a
//! BalanceWatch that skipped a non-finite imbalance (`v2p3: verdict.rs:104-106`,
//! `invariants.rs:167`).
//!
//! **No vacuous pass.** Every battery records at least one reading for each thing it scores:
//! each (market, segment), each market, each window, each resume, each kick. A battery with
//! nothing to score fails with "no samples", so a world with no market fails every per-market
//! battery.
//!
//! **Reports** have no gate and are never scored. They fold with the same NaN-keeping folds, and
//! a sample any of whose inputs is not finite is itself NaN, so a non-finite input reaches the
//! certificate, where the seal fails it (§8). No report is a ratio that could be 0/0.

use crate::certificate::{BatteryId, BatteryResult, Limit, Reading, NONFINITE_READING};
use crate::fold::{self, max_nan, min2, min_nan};
use crate::leaves::{leaves, Leaf};
use crate::obs::{MarketObs, Names, Obs, Segment};
use rustyecon_engine::prelude::{SideTag, TickReport};
use rustyecon_engine::rustyecon_markets::imbalance;

/// The first non-finite value a battery read: where and when.
#[derive(Debug, Default)]
struct Gate {
    first: Option<(u64, String, f64)>,
}

impl Gate {
    /// Check `x`, read at `tick` from `path()`; the first non-finite one is kept.
    fn read(&mut self, tick: u64, x: f64, path: impl FnOnce() -> String) {
        if self.first.is_none() && !x.is_finite() {
            self.first = Some((tick, path(), x));
        }
    }

    /// The failed battery: its note names the value, the tick and the path.
    fn fail(self, id: BatteryId) -> Option<BatteryResult> {
        let (tick, path, x) = self.first?;
        Some(BatteryResult {
            id,
            pass: false,
            readings: vec![reading(
                NONFINITE_READING,
                &path,
                None,
                tick_value(tick),
                None,
            )],
            notes: vec![format!("the finite gate: {x} at tick {tick}, {path}")],
        })
    }
}

/// A tick or count as a reading's value.
fn tick_value(t: u64) -> f64 {
    t as f64
}

fn reading(name: &str, at: &str, segment: Option<u32>, value: f64, bar: Option<Limit>) -> Reading {
    Reading {
        name: name.to_string(),
        at: at.to_string(),
        segment,
        value,
        bar,
    }
}

fn seg_index(i: usize) -> Option<u32> {
    u32::try_from(i).ok()
}

/// A battery with nothing to score (§6): it fails.
fn no_samples(id: BatteryId, what: &str) -> BatteryResult {
    BatteryResult {
        id,
        pass: false,
        readings: vec![reading(
            "samples",
            "run",
            None,
            0.0,
            Some(Limit::AtLeast(1.0)),
        )],
        notes: vec![format!("no samples: {what}")],
    }
}

/// The ticks of `obs` in `[from, to)`: `obs` is in tick order.
pub fn in_ticks(obs: &[Obs], from: u64, to: u64) -> &[Obs] {
    let lo = obs.partition_point(|o| o.tick < from);
    let hi = obs.partition_point(|o| o.tick < to);
    &obs[lo..hi.max(lo)]
}

fn market_path(names: &Names, m: usize, field: &str) -> String {
    format!("markets[{}].{field}", names.market(m))
}

/// A `MarketObs` field a battery reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Price,
    NextPrice,
    Supply,
    Demand,
    Cleared,
    BuyerFill,
    SellerFill,
}

impl Field {
    fn name(self) -> &'static str {
        match self {
            Field::Price => "price",
            Field::NextPrice => "next_price",
            Field::Supply => "supply",
            Field::Demand => "demand",
            Field::Cleared => "cleared",
            Field::BuyerFill => "buyer_fill",
            Field::SellerFill => "seller_fill",
        }
    }

    fn of(self, m: &MarketObs) -> f64 {
        match self {
            Field::Price => m.price,
            Field::NextPrice => m.next_price,
            Field::Supply => m.supply,
            Field::Demand => m.demand,
            Field::Cleared => m.cleared,
            Field::BuyerFill => m.buyer_fill,
            Field::SellerFill => m.seller_fill,
        }
    }
}

/// Gate every field a market-reading battery reads, on every tick of `obs` and every market.
fn gate_markets(g: &mut Gate, obs: &[Obs], names: &Names, fields: &[Field]) {
    for o in obs {
        for (m, mo) in o.markets.iter().enumerate() {
            for f in fields {
                g.read(o.tick, f.of(mo), || market_path(names, m, f.name()));
            }
        }
    }
}

/// The markets every tick of `obs` reports: 0 when there are none or ticks disagree.
fn market_count(obs: &[Obs]) -> usize {
    let n = obs.first().map_or(0, |o| o.markets.len());
    if obs.iter().all(|o| o.markets.len() == n) {
        n
    } else {
        0
    }
}

// ── Conservation (R2) ───────────────────────────────────────────────────────────────────────

/// Conservation (R2): the run reached `until`, and every tick's margin and the run's margin are
/// at most 1. The engine stops a run on a breach; this reads the same numbers a second time.
pub fn conservation(obs: &[Obs], reached: u64, until: u64) -> BatteryResult {
    let id = BatteryId::Conservation;
    if obs.is_empty() {
        return no_samples(id, "the run has no ticks");
    }
    let mut g = Gate::default();
    for o in obs {
        g.read(o.tick, o.margin, || "audit.max_margin".to_string());
        g.read(o.tick, o.run_margin, || "run.max_margin".to_string());
    }
    if let Some(f) = g.fail(id) {
        return f;
    }
    let margin = max_nan(obs.iter().map(|o| o.margin)).unwrap_or(0.0);
    let run = max_nan(obs.iter().map(|o| o.run_margin)).unwrap_or(0.0);
    let reached_ok = until <= reached;
    let closed = margin <= 1.0 && run <= 1.0;
    let pass = reached_ok && closed;
    let mut notes = Vec::new();
    if !reached_ok {
        notes.push(format!("the run stopped at tick {reached} of {until}"));
    }
    if !closed {
        notes.push(format!(
            "a ledger margin above 1: tick {margin:?}, run {run:?}"
        ));
    }
    BatteryResult {
        id,
        pass,
        readings: vec![
            reading(
                "conservation.reached",
                "run",
                None,
                tick_value(reached),
                Some(Limit::AtLeast(tick_value(until))),
            ),
            reading(
                "conservation.max_margin",
                "run",
                None,
                margin,
                Some(Limit::AtMost(1.0)),
            ),
            reading(
                "conservation.max_run_margin",
                "run",
                None,
                run,
                Some(Limit::AtMost(1.0)),
            ),
        ],
        notes,
    }
}

// ── Determinism (R8) ────────────────────────────────────────────────────────────────────────

/// One resume of the base run: its checkpoint's tick, and what the resumed run gave, its hashes
/// to the end and its last report, or why it could not.
#[derive(Debug, Clone, PartialEq)]
pub struct ResumeRun {
    /// The checkpoint's state tick.
    pub at: u64,
    /// The resumed run's hashes from `at` to the end and its last report, or its error.
    pub outcome: Result<(Vec<u64>, TickReport), String>,
}

/// What Determinism reads: the base run's hash stream and last report, a second run's stream,
/// the replay audit's final hash, and each resume.
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismInputs {
    /// The base run's `TickReport.hash`, every tick.
    pub base: Vec<u64>,
    /// The base run's last report.
    pub base_last: Option<TickReport>,
    /// A second `Sim::new` run's hashes, or its error.
    pub second: Result<Vec<u64>, String>,
    /// `audit_replay`'s final hash, or its error.
    pub replay: Result<u64, String>,
    /// Each resume.
    pub resumes: Vec<ResumeRun>,
}

/// How many leaves of two values differ (a float by its bits), counting a leaf one has and the
/// other lacks.
fn differing<T: serde::Serialize>(a: &T, b: &T) -> u64 {
    let (la, lb) = (leaves(a), leaves(b));
    let mut n = la.len().abs_diff(lb.len()) as u64;
    for ((pa, xa), (pb, xb)) in la.iter().zip(&lb) {
        if pa != pb || !xa.same(xb) {
            n += 1;
        }
    }
    n
}

fn mismatches(a: &[u64], b: &[u64]) -> u64 {
    let n = a.iter().zip(b).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len());
    n as u64
}

/// Determinism (R8): a second run gives the base run's hashes tick by tick, the replay audit
/// ends at the base run's final hash, and each resumed run's hashes are the base run's and its
/// last report equals the base run's in full, every float by its bits.
pub fn determinism(d: &DeterminismInputs) -> BatteryResult {
    let id = BatteryId::Determinism;
    let Some(base_last) = &d.base_last else {
        return no_samples(id, "the base run has no ticks");
    };
    if d.base.is_empty() || d.resumes.is_empty() {
        return no_samples(id, "no resume to score");
    }
    // The gate: every float of the base run's last report and each resumed run's.
    let mut g = Gate::default();
    let mut gate_report = |r: &TickReport, whose: &str| {
        for (p, l) in leaves(r) {
            if let Leaf::F64(x) = l {
                g.read(r.tick, x, || format!("{whose}.{p}"));
            }
        }
    };
    gate_report(base_last, "base");
    for r in &d.resumes {
        if let Ok((_, last)) = &r.outcome {
            gate_report(last, &format!("resume[{}]", r.at));
        }
    }
    if let Some(f) = g.fail(id) {
        return f;
    }
    let mut readings = Vec::new();
    let mut notes = Vec::new();
    let mut pass = true;
    let repeat = match &d.second {
        Ok(second) => mismatches(&d.base, second),
        Err(e) => {
            notes.push(format!("the second run failed: {e}"));
            tick_count(d.base.len())
        }
    };
    readings.push(reading(
        "determinism.repeat",
        "run",
        None,
        tick_value(repeat),
        Some(Limit::AtMost(0.0)),
    ));
    pass &= repeat == 0 && d.second.is_ok();
    let replay = match &d.replay {
        Ok(h) => u64::from(d.base.last() != Some(h)),
        Err(e) => {
            notes.push(format!("the replay audit failed: {e}"));
            1
        }
    };
    readings.push(reading(
        "determinism.replay",
        "run",
        None,
        tick_value(replay),
        Some(Limit::AtMost(0.0)),
    ));
    pass &= replay == 0;
    for r in &d.resumes {
        let at = format!("tick {}", r.at);
        let differs = match &r.outcome {
            Ok((hashes, last)) => {
                let from = usize::try_from(r.at)
                    .unwrap_or(usize::MAX)
                    .min(d.base.len());
                mismatches(&d.base[from..], hashes) + differing(base_last, last)
            }
            Err(e) => {
                notes.push(format!("the resume at {at} failed: {e}"));
                pass = false;
                1
            }
        };
        readings.push(reading(
            "determinism.resume",
            &at,
            None,
            tick_value(differs),
            Some(Limit::AtMost(0.0)),
        ));
        pass &= differs == 0;
    }
    if !pass && notes.is_empty() {
        notes.push("a run differs from the base run".to_string());
    }
    BatteryResult {
        id,
        pass,
        readings,
        notes,
    }
}

fn tick_count(n: usize) -> u64 {
    n as u64
}

// ── Runaway (A12) ───────────────────────────────────────────────────────────────────────────

/// Whether a price ratio to genesis lies within `[1/bound, bound]` (A12): relative, so it means
/// the same at any price level. The probe's runaway bound is this (§11).
pub fn within_bound(ratio: f64, bound: f64) -> bool {
    1.0 / bound <= ratio && ratio <= bound
}

/// Runaway (A12): every market's posted price, `next_price` on every tick, stays within
/// `[1/bound, bound]` of its genesis price. It does not stop the run.
pub fn runaway(obs: &[Obs], genesis: &[f64], bound: f64, names: &Names) -> BatteryResult {
    let id = BatteryId::Runaway;
    let n = market_count(obs);
    if obs.is_empty() || n == 0 || genesis.len() != n {
        return no_samples(id, "no market, or no tick");
    }
    let mut g = Gate::default();
    for (m, &p0) in genesis.iter().enumerate() {
        g.read(0, p0, || format!("genesis[{}].price", names.market(m)));
    }
    gate_markets(&mut g, obs, names, &[Field::NextPrice]);
    if let Some(f) = g.fail(id) {
        return f;
    }
    let mut readings = Vec::new();
    let mut notes = Vec::new();
    let mut pass = true;
    for (m, &p0) in genesis.iter().enumerate() {
        let name = names.market(m);
        let ratios: Vec<(u64, f64)> = obs
            .iter()
            .map(|o| (o.tick, o.markets[m].next_price / p0))
            .collect();
        let hi = max_nan(ratios.iter().map(|r| r.1)).unwrap_or(0.0);
        let lo = min_nan(ratios.iter().map(|r| r.1)).unwrap_or(0.0);
        readings.push(reading(
            "runaway.max_ratio",
            &name,
            None,
            hi,
            Some(Limit::AtMost(bound)),
        ));
        readings.push(reading(
            "runaway.min_ratio",
            &name,
            None,
            lo,
            Some(Limit::AtLeast(1.0 / bound)),
        ));
        let ok = within_bound(lo, bound) && within_bound(hi, bound);
        if !ok {
            pass = false;
            if let Some((t, r)) = ratios.iter().find(|(_, r)| !within_bound(*r, bound)) {
                notes.push(format!(
                    "{name} left [1/{bound:?}, {bound:?}] of its genesis price at tick {t}: \
                     {r:?} of genesis"
                ));
            }
        }
    }
    BatteryResult {
        id,
        pass,
        readings,
        notes,
    }
}

// ── Trades ──────────────────────────────────────────────────────────────────────────────────

/// The Trades windows over `[0, n)` (§6): `[k·E, (k+1)·E)` while it fits, and `[n − E, n)` when E
/// does not divide n, so the last full window ends at `n`. Empty when E is 0 or above n.
pub fn trade_windows(n: u64, every: u64) -> Vec<(u64, u64)> {
    if every == 0 || every > n {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut a = 0;
    while a + every <= n {
        out.push((a, a + every));
        a += every;
    }
    if a < n {
        out.push((n - every, n));
    }
    out
}

/// Trades: every market trades at least once in every window (a liveness check over the run;
/// it does not restart at shocks).
pub fn trades(obs: &[Obs], windows: &[(u64, u64)], names: &Names) -> BatteryResult {
    let id = BatteryId::Trades;
    let n = market_count(obs);
    if obs.is_empty() || n == 0 || windows.is_empty() {
        return no_samples(id, "no market, or no window");
    }
    let mut g = Gate::default();
    gate_markets(
        &mut g,
        obs,
        names,
        &[
            Field::Supply,
            Field::Demand,
            Field::BuyerFill,
            Field::SellerFill,
        ],
    );
    if let Some(f) = g.fail(id) {
        return f;
    }
    let mut readings = vec![reading(
        "trades.windows",
        "run",
        None,
        tick_value(tick_count(windows.len())),
        None,
    )];
    let mut notes = Vec::new();
    let mut pass = true;
    for m in 0..n {
        let name = names.market(m);
        let silent: Vec<&(u64, u64)> = windows
            .iter()
            .filter(|(a, b)| {
                let span = in_ticks(obs, *a, *b);
                span.len() as u64 != b - a || !span.iter().any(|o| o.markets[m].trades())
            })
            .collect();
        readings.push(reading(
            "trades.silent_windows",
            &name,
            None,
            tick_value(tick_count(silent.len())),
            Some(Limit::AtMost(0.0)),
        ));
        if let Some((a, b)) = silent.first() {
            pass = false;
            notes.push(format!(
                "{name} did not trade in [{a}, {b}), one of {} silent windows",
                silent.len()
            ));
        }
    }
    BatteryResult {
        id,
        pass,
        readings,
        notes,
    }
}

// ── Balance (BalanceWatch) ──────────────────────────────────────────────────────────────────

/// The Balance bars (§4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BalanceBars {
    /// An imbalance above this in size can be a pin.
    pub level: f64,
    /// A standard deviation below this is no movement.
    pub spread: f64,
    /// The observations a verdict needs.
    pub min_samples: u64,
    /// An unbroken run of one value over this share of the observations is a pin.
    pub run_share: f64,
}

/// One market's imbalance over a stretch of ticks: a pin is a constant, not a small number.
///
/// Ported from July (`v2p3: certify/invariants.rs`), where it caught the legacy sell cap's
/// imbalance pinned at exactly −1/6 over 10,100 ticks. Its bars are the criteria's now, not
/// constants in code (R4), and it fails closed: a non-finite imbalance is counted and fails the
/// verdict, where July skipped it (`invariants.rs:167`, N12). A one-sided tick is observed at
/// ±1 (C13); only a tick with S = D = 0 is skipped, by the caller, since its 0 is a convention.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceWatch {
    count: u64,
    mean: f64,
    m2: f64,
    run_len: u64,
    run_value: f64,
    max_run: u64,
    max_run_value: f64,
    nonfinite: u64,
}

impl BalanceWatch {
    /// Observe one tick's imbalance: Welford's running mean and spread, and the longest
    /// unbroken run of one exact value.
    pub fn observe(&mut self, x: f64) {
        if !x.is_finite() {
            self.nonfinite += 1;
        }
        self.count += 1;
        let delta = x - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (x - self.mean);
        // Exact equality, not a tolerance: a structural pin is arithmetic returning the same
        // number, not a series that happens to be quiet.
        if self.run_len > 0 && self.run_value.to_bits() == x.to_bits() {
            self.run_len += 1;
        } else {
            self.run_len = 1;
            self.run_value = x;
        }
        if self.run_len > self.max_run {
            self.max_run = self.run_len;
            self.max_run_value = x;
        }
    }

    /// The observations.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// The mean imbalance.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// The sample standard deviation; `None` below two observations.
    pub fn sd(&self) -> Option<f64> {
        (self.count >= 2).then(|| (self.m2 / (self.count - 1) as f64).sqrt())
    }

    /// The longest unbroken run of one value, and the value.
    pub fn longest_run(&self) -> (u64, f64) {
        (self.max_run, self.max_run_value)
    }

    /// The verdict under `bars`: too few samples, a non-finite imbalance, or a pin (a spread
    /// below `spread` about a mean above `level` in size, or an unbroken run of one value above
    /// `level` in size over at least `run_share` of the observations) fail, with why.
    pub fn judge(&self, bars: &BalanceBars) -> Result<(), String> {
        if self.count < bars.min_samples {
            return Err(format!(
                "too few samples: {} of {} (a pin cannot be told from a quiet market)",
                self.count, bars.min_samples
            ));
        }
        let Some(sd) = self.sd() else {
            return Err("too few samples for a spread".to_string());
        };
        if self.nonfinite > 0 || !(self.mean.is_finite() && sd.is_finite()) {
            return Err(format!(
                "{} non-finite imbalances: mean {:?}, sd {sd:?}",
                self.nonfinite, self.mean
            ));
        }
        if sd < bars.spread && bars.level < self.mean.abs() {
            return Err(format!(
                "pinned at {:?} for all {} observations (sd {sd:?})",
                self.mean, self.count
            ));
        }
        let (run, v) = self.longest_run();
        if bars.level < v.abs() && bars.run_share * self.count as f64 <= run as f64 {
            return Err(format!(
                "pinned at {v:?} for {run} of {} observations unbroken",
                self.count
            ));
        }
        Ok(())
    }
}

/// Balance: in every segment, every market has at least `min_samples` observations and is not
/// pinned (BalanceWatch). A tick is an observation unless S = D = 0 (C13).
pub fn balance(
    obs: &[Obs],
    segments: &[Segment],
    bars: &BalanceBars,
    names: &Names,
) -> BatteryResult {
    let id = BatteryId::Balance;
    let n = market_count(obs);
    if obs.is_empty() || n == 0 || segments.is_empty() {
        return no_samples(id, "no market, or no segment");
    }
    let mut g = Gate::default();
    gate_markets(&mut g, obs, names, &[Field::Supply, Field::Demand]);
    if let Some(f) = g.fail(id) {
        return f;
    }
    let mut readings = Vec::new();
    let mut notes = Vec::new();
    let mut pass = true;
    for (k, s) in segments.iter().enumerate() {
        let span = in_ticks(obs, s.from, s.to);
        for m in 0..n {
            let name = names.market(m);
            let mut w = BalanceWatch::default();
            for o in span {
                let (sup, dem) = (o.markets[m].supply, o.markets[m].demand);
                if sup == 0.0 && dem == 0.0 {
                    continue;
                }
                w.observe(imbalance(sup, dem));
            }
            let seg = seg_index(k);
            readings.push(reading(
                "balance.samples",
                &name,
                seg,
                tick_value(w.count()),
                Some(Limit::AtLeast(tick_value(bars.min_samples))),
            ));
            if let Some(sd) = w.sd() {
                let (run, v) = w.longest_run();
                readings.push(reading(
                    "balance.mean",
                    &name,
                    seg,
                    w.mean(),
                    Some(Limit::Ref(bars.level)),
                ));
                readings.push(reading(
                    "balance.sd",
                    &name,
                    seg,
                    sd,
                    Some(Limit::Ref(bars.spread)),
                ));
                // The run is compared with run_share of the observations.
                readings.push(reading(
                    "balance.longest_run",
                    &name,
                    seg,
                    tick_value(run),
                    Some(Limit::Ref(bars.run_share * tick_value(w.count()))),
                ));
                readings.push(reading(
                    "balance.run_value",
                    &name,
                    seg,
                    v,
                    Some(Limit::Ref(bars.level)),
                ));
            }
            if let Err(why) = w.judge(bars) {
                pass = false;
                notes.push(format!("{name}, segment {k}: {why}"));
            }
        }
    }
    BatteryResult {
        id,
        pass,
        readings,
        notes,
    }
}

// ── Settles (REPORT §6, criteria 1 and 3) ───────────────────────────────────────────────────

/// The Settles bars (§4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettlesBars {
    /// W starts this share of the way into a segment.
    pub w_from: f64,
    /// F starts this share of the way in.
    pub f_from: f64,
    /// The dead ticks W may hold, as a share of it.
    pub dead_share: f64,
    /// The widest band, in log, of price and of cleared volume over F.
    pub band: f64,
}

/// The dead-share rule (§6, and the probe's DEAD class): at most `⌊dead_share·|W|⌋` dead ticks
/// in W and none in F.
pub fn dead_share_ok(dead_w: u64, w_len: u64, dead_f: u64, dead_share: f64) -> bool {
    dead_w <= fold::floor_share(dead_share, w_len) && dead_f == 0
}

/// Whether a tick is dead: some market did not trade.
fn dead(o: &Obs) -> bool {
    o.markets.iter().any(|m| !m.trades())
}

/// Settles: in each segment, W holds few dead ticks and F none, and every market's price and
/// cleared volume stay within `band` in log over F. The windows are relative to each segment,
/// so they restart at each dated shock and scale with the run (N15).
pub fn settles(
    obs: &[Obs],
    segments: &[Segment],
    bars: &SettlesBars,
    names: &Names,
) -> BatteryResult {
    let id = BatteryId::Settles;
    let n = market_count(obs);
    if obs.is_empty() || n == 0 || segments.is_empty() {
        return no_samples(id, "no market, or no segment");
    }
    let mut g = Gate::default();
    for s in segments {
        let (w0, w1) = s.window(bars.w_from);
        gate_markets(
            &mut g,
            in_ticks(obs, w0, w1),
            names,
            &[
                Field::Supply,
                Field::Demand,
                Field::BuyerFill,
                Field::SellerFill,
                Field::Price,
                Field::Cleared,
            ],
        );
    }
    if let Some(f) = g.fail(id) {
        return f;
    }
    let mut readings = Vec::new();
    let mut notes = Vec::new();
    let mut pass = true;
    for (k, s) in segments.iter().enumerate() {
        let seg = seg_index(k);
        let (w0, w1) = s.window(bars.w_from);
        let (f0, f1) = s.window(bars.f_from);
        let (wspan, fspan) = (in_ticks(obs, w0, w1), in_ticks(obs, f0, f1));
        let (w_len, f_len) = (w1.saturating_sub(w0), f1.saturating_sub(f0));
        if fspan.len() < 2 || fspan.len() as u64 != f_len || wspan.len() as u64 != w_len {
            pass = false;
            notes.push(format!(
                "segment {k}: no samples: F holds {} ticks",
                fspan.len()
            ));
            readings.push(reading(
                "settles.f_ticks",
                "run",
                seg,
                tick_value(tick_count(fspan.len())),
                Some(Limit::Exactly(tick_value(f_len))),
            ));
            continue;
        }
        let dead_w = tick_count(wspan.iter().filter(|o| dead(o)).count());
        let dead_f = tick_count(fspan.iter().filter(|o| dead(o)).count());
        let allowed = fold::floor_share(bars.dead_share, w_len);
        readings.push(reading(
            "settles.dead_w",
            "run",
            seg,
            tick_value(dead_w),
            Some(Limit::AtMost(tick_value(allowed))),
        ));
        readings.push(reading(
            "settles.dead_f",
            "run",
            seg,
            tick_value(dead_f),
            Some(Limit::AtMost(0.0)),
        ));
        if !dead_share_ok(dead_w, w_len, dead_f, bars.dead_share) {
            pass = false;
            notes.push(format!(
                "segment {k} [{}, {}): {dead_w} dead ticks in W [{w0}, {w1}) (at most \
                 {allowed}), {dead_f} in F [{f0}, {f1})",
                s.from, s.to
            ));
        }
        for m in 0..n {
            let name = names.market(m);
            let price = fold::ln_range(fspan.iter().map(|o| o.markets[m].price)).unwrap_or(0.0);
            let cleared = fold::ln_range(fspan.iter().map(|o| o.markets[m].cleared)).unwrap_or(0.0);
            readings.push(reading(
                "settles.price_range",
                &name,
                seg,
                price,
                Some(Limit::AtMost(bars.band)),
            ));
            readings.push(reading(
                "settles.cleared_range",
                &name,
                seg,
                cleared,
                Some(Limit::AtMost(bars.band)),
            ));
            let (price_rests, cleared_rests) = (price <= bars.band, cleared <= bars.band);
            if !(price_rests && cleared_rests) {
                pass = false;
                let mut moved = Vec::new();
                if !price_rests {
                    moved.push(format!("ln price moved {price:?}"));
                }
                if !cleared_rests {
                    moved.push(format!("ln cleared moved {cleared:?}"));
                }
                notes.push(format!(
                    "{name}, segment {k}: not at rest in F [{f0}, {f1}): {}, past the band {:?}",
                    moved.join(" and "),
                    bars.band
                ));
            }
        }
    }
    BatteryResult {
        id,
        pass,
        readings,
        notes,
    }
}

// ── Kick (REPORT §5, §6 criterion 1) ────────────────────────────────────────────────────────

/// A kick's readings (§7): the realized kick g(T), and its largest ratio to g(T) over the tail
/// and over the whole horizon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KickGain {
    /// g(T): the largest |ln(p_kick/p_base)| over markets on the kicked run's first report.
    pub size: f64,
    /// The largest g(t)/g(T) over the last `tail` ticks.
    pub gain_tail: f64,
    /// The largest g(t)/g(T) over the horizon.
    pub gain_peak: f64,
}

/// The gain of a kick (§7), a pure function of the two price series: `base[t]` and `kicked[t]`
/// hold every market's `MarketLine.price` on the t-th tick after the kick, the first being the
/// kicked tick. g(t) is the NaN-keeping largest |ln(p_kick/p_base)| over markets; the readings
/// divide by g(T), the realized kick, never by the nominal size, so a kick that rounding ate
/// scores as what it was. `None` when the series are empty, of different lengths or shapes, or
/// `tail` is 0 or longer than them.
pub fn kick_gain(base: &[Vec<f64>], kicked: &[Vec<f64>], tail: u64) -> Option<KickGain> {
    kick_gain_free(base, kicked, tail, &[])
}

/// [`kick_gain`] with `free[m]` marking each market whose good has a free step (amended at
/// P2.4.11; FREE-SPEC §6.1): there a posted price of 0 is a price, and equal prices, 0 in both
/// series included, read a gap of 0. Every other market's gap is |ln(p_kick/p_base)|, NaN-keeping,
/// as [`kick_gain`]'s, so a price of 0 anywhere else still fails closed (N12). A mask shorter than
/// the markets marks the rest not free.
pub fn kick_gain_free(
    base: &[Vec<f64>],
    kicked: &[Vec<f64>],
    tail: u64,
    free: &[bool],
) -> Option<KickGain> {
    let h = base.len();
    let tail = usize::try_from(tail).ok()?;
    if h == 0 || kicked.len() != h || tail == 0 || tail > h {
        return None;
    }
    let mut g = Vec::with_capacity(h);
    for (b, k) in base.iter().zip(kicked) {
        if b.len() != k.len() || b.is_empty() {
            return None;
        }
        let gap = b.iter().zip(k).enumerate().map(|(m, (pb, pk))| {
            if pk == pb && free.get(m).copied().unwrap_or(false) {
                0.0
            } else {
                rustyecon_core::num::ln(pk / pb).abs()
            }
        });
        g.push(max_nan(gap)?);
    }
    let size = g[0];
    let ratio = |x: f64| x / size;
    let gain_tail = max_nan(g[h - tail..].iter().map(|&x| ratio(x)))?;
    let gain_peak = max_nan(g.iter().map(|&x| ratio(x)))?;
    Some(KickGain {
        size,
        gain_tail,
        gain_peak,
    })
}

/// The sign of a kick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// The price times `1 + size`.
    Up,
    /// The price times `1 − size`.
    Down,
}

impl Sign {
    /// `+` or `−`, as a reading's name.
    pub fn mark(self) -> &'static str {
        match self {
            Sign::Up => "+",
            Sign::Down => "-",
        }
    }
}

/// One kicked run: the market and sign kicked, its tape's hash, and every market's price on each
/// tick of the horizon, or why it could not run.
#[derive(Debug, Clone, PartialEq)]
pub struct KickedRun {
    /// The market kicked, by index.
    pub market: usize,
    /// Up or down.
    pub sign: Sign,
    /// The kicked tape's hash.
    pub tape_hash: u64,
    /// The prices, or the error.
    pub prices: Result<Vec<Vec<f64>>, String>,
}

/// The kicks at one tick T (§7, amended at S2.5): the end of the run, or a tick a dated event
/// fires, whether or not it closes a segment. Each kicked run resumes the base run's checkpoint
/// at T under the tape with every dated event at or after T deferred, so it probes the regime in
/// force before T. It holds the base continuation's prices and each kicked run, or why the tick
/// could not be kicked.
#[derive(Debug, Clone, PartialEq)]
pub struct KickSegment {
    /// The segment whose regime the kick probes: the one that holds tick T − 1.
    pub segment: usize,
    /// T: the tick the kick fires in.
    pub at: u64,
    /// The base continuation and the kicked runs, or the error.
    pub runs: Result<(Vec<Vec<f64>>, Vec<KickedRun>), String>,
    /// Each market's good has a free step (amended at P2.4.11), in market order: its prices may
    /// be 0, and [`kick_gain_free`] reads them so. Empty for a world without one.
    pub free: Vec<bool>,
}

/// The Kick bars, in ticks where §7 needs ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KickBars {
    /// The kick's relative size.
    pub size: f64,
    /// H.
    pub horizon: u64,
    /// `⌈tail·H⌉`.
    pub tail: u64,
    /// The largest gain the tail may show.
    pub max_gain: f64,
    /// The largest gain the horizon may show.
    pub max_peak: f64,
}

/// Kick: at each kick tick (the run's end and every dated shock, amended at S2.5), every kick
/// of ± each market's price is realized (g(T) > 0), decays (its `gain_tail` at most
/// `max_gain`) and never grows far (its `gain_peak` at most `max_peak`), and no kicked or base
/// run fails (`kick.errors`, 0). A decaying kick ends near its rounding floor, a neutral one near
/// 1 and a growing one above it, so a rounding freeze at an unstable point fails here though it
/// looks settled (REPORT §5). The peak catches the other way an unstable point hides: a kick that
/// swings out through dead markets and comes back to the point it left, where rounding freezes it
/// again (the desk-turnover ×16 cell; amended at S2.3). A regime shorter than `min_segment` is
/// kicked too, at the shock that ends it, though its segment merged on.
pub fn kick(points: &[KickSegment], bars: &KickBars, names: &Names) -> BatteryResult {
    let id = BatteryId::Kick;
    if points.is_empty() {
        return no_samples(id, "no tick to kick");
    }
    let mut readings = Vec::new();
    let mut notes = Vec::new();
    // Each kicked tape's hash, after the failures: a rerun of `certify` reruns every kick.
    let mut hashes = Vec::new();
    let mut pass = true;
    let mut scored = 0usize;
    for ks in points {
        let seg = seg_index(ks.segment);
        let point = format!("tick {}", ks.at);
        let (base, runs) = match &ks.runs {
            Ok(x) => x,
            Err(e) => {
                pass = false;
                notes.push(format!("{point} (segment {}): {e}", ks.segment));
                readings.push(reading(
                    "kick.errors",
                    &point,
                    seg,
                    1.0,
                    Some(Limit::AtMost(0.0)),
                ));
                continue;
            }
        };
        if runs.is_empty() {
            pass = false;
            notes.push(format!("{point}: no samples: no market to kick"));
            continue;
        }
        let mut g = Gate::default();
        for (t, ps) in base.iter().enumerate() {
            for (m, &p) in ps.iter().enumerate() {
                g.read(ks.at + tick_count(t), p, || {
                    format!("kick[{}].base.{}", ks.at, market_path(names, m, "price"))
                });
            }
        }
        for r in runs {
            if let Ok(prices) = &r.prices {
                for (t, ps) in prices.iter().enumerate() {
                    for (m, &p) in ps.iter().enumerate() {
                        g.read(ks.at + tick_count(t), p, || {
                            format!(
                                "kick[{}][{} {}].{}",
                                ks.at,
                                names.market(r.market),
                                r.sign.mark(),
                                market_path(names, m, "price")
                            )
                        });
                    }
                }
            }
        }
        if let Some(f) = g.fail(id) {
            pass = false;
            notes.extend(f.notes);
            readings.extend(f.readings);
            continue;
        }
        // H, as the base continuation ran it: `ticks(horizon)`, whatever the segment's length.
        readings.push(reading(
            "kick.horizon",
            &point,
            seg,
            tick_value(tick_count(base.len())),
            Some(Limit::Exactly(tick_value(bars.horizon))),
        ));
        if tick_count(base.len()) != bars.horizon {
            pass = false;
            notes.push(format!(
                "{point}: the base continuation ran {} ticks, not the horizon's {}",
                base.len(),
                bars.horizon
            ));
        }
        let mut errors = 0u64;
        for r in runs {
            let at = format!("{} {} at {point}", names.market(r.market), r.sign.mark());
            hashes.push(format!("kick {at}: tape_hash 0x{:016x}", r.tape_hash));
            let prices = match &r.prices {
                Ok(p) => p,
                Err(e) => {
                    pass = false;
                    errors += 1;
                    notes.push(format!("kick {at} failed: {e}"));
                    continue;
                }
            };
            let Some(k) = kick_gain_free(base, prices, bars.tail, &ks.free) else {
                pass = false;
                errors += 1;
                notes.push(format!("kick {at}: no samples over the horizon"));
                continue;
            };
            scored += 1;
            readings.push(reading(
                "kick.size",
                &at,
                seg,
                k.size,
                Some(Limit::Above(0.0)),
            ));
            readings.push(reading(
                "kick.gain_tail",
                &at,
                seg,
                k.gain_tail,
                Some(Limit::AtMost(bars.max_gain)),
            ));
            readings.push(reading(
                "kick.gain_peak",
                &at,
                seg,
                k.gain_peak,
                Some(Limit::AtMost(bars.max_peak)),
            ));
            let realized = 0.0 < k.size;
            let decays = k.gain_tail <= bars.max_gain;
            let bounded = k.gain_peak <= bars.max_peak;
            if !realized {
                pass = false;
                notes.push(format!("kick {at}: a zero kick"));
            } else if !(decays && bounded) {
                pass = false;
                notes.push(format!(
                    "kick {at}: the kick did not decay: gain {:?} in the tail, {:?} at its peak",
                    k.gain_tail, k.gain_peak
                ));
            }
        }
        // The kicked runs that failed or had nothing to score: a count a verdict reads.
        readings.push(reading(
            "kick.errors",
            &point,
            seg,
            tick_value(errors),
            Some(Limit::AtMost(0.0)),
        ));
    }
    if scored == 0 {
        // Nothing scored, whatever the reason: no samples, after the reason if there is one.
        pass = false;
        readings.push(reading(
            "samples",
            "run",
            None,
            0.0,
            Some(Limit::AtLeast(1.0)),
        ));
        notes.push("no samples: no kick was scored".to_string());
    }
    notes.extend(hashes);
    BatteryResult {
        id,
        pass,
        readings,
        notes,
    }
}

// ── Reports (REPORT §6 criterion 2; O14) ────────────────────────────────────────────────────

/// `x`, or NaN if any input it was made from is not finite: a non-finite input `v` gives
/// `v·0`, which is NaN. How a report carries a non-finite input to the certificate.
fn poisoned(x: f64, inputs: &[f64]) -> f64 {
    inputs
        .iter()
        .find(|v| !v.is_finite())
        .map_or(x, |v| v * 0.0)
}

/// A count of one tick: 1 or 0, or NaN if an input is not finite.
fn one_if(yes: bool, inputs: &[f64]) -> f64 {
    poisoned(if yes { 1.0 } else { 0.0 }, inputs)
}

/// Whether a side exists to be judged: its size is positive, or not a number a report can read
/// (and the sample is then NaN).
fn side(size: f64) -> bool {
    size > 0.0 || !size.is_finite()
}

/// Whether a fill counts as rationed: below `1 − rationed_below` (the probe's rationed ticks
/// per market side are this, §11).
pub fn rationed(fill: f64, rationed_below: f64) -> bool {
    fill < 1.0 - rationed_below
}

/// The reports (§6): per segment and per market, good, class or provider, never scored. Each
/// value is a sum, a count or a minimum over the segment, folded with the NaN-keeping folds,
/// and a sample made from a non-finite input is NaN.
pub fn reports(
    obs: &[Obs],
    segments: &[Segment],
    rationed_below: f64,
    names: &Names,
) -> Vec<Reading> {
    let mut out = Vec::new();
    let n = market_count(obs);
    let goods = obs.first().map_or(0, |o| o.consumed.len());
    // A good counts as consumed if anything consumed it anywhere in the run.
    let consumed_anywhere: Vec<bool> = (0..goods)
        .map(|g| {
            obs.iter()
                .any(|o| o.consumed.get(g).is_some_and(|c| *c != 0.0))
        })
        .collect();
    for (k, s) in segments.iter().enumerate() {
        let seg = seg_index(k);
        let span = in_ticks(obs, s.from, s.to);
        let Some(last) = span.last() else { continue };
        for m in 0..n {
            let name = names.market(m);
            // Troughs: the minimum cleared volume, its tick, and the level at the segment's end.
            if let Some((low, tick)) =
                fold::trough(span.iter().map(|o| (o.tick, o.markets[m].cleared)))
            {
                out.push(reading("trough.cleared", &name, seg, low, None));
                out.push(reading("trough.tick", &name, seg, tick_value(tick), None));
            }
            out.push(reading(
                "trough.end",
                &name,
                seg,
                last.markets[m].cleared,
                None,
            ));
            let (mut dead_t, mut no_s, mut no_d) = (0.0, 0.0, 0.0);
            let (mut worst_b, mut worst_s): (Option<f64>, Option<f64>) = (None, None);
            let (mut rat_b, mut rat_s, mut subnormal) = (0.0, 0.0, 0.0);
            for o in span {
                let mo = o.markets[m];
                let inputs = [
                    mo.supply,
                    mo.demand,
                    mo.buyer_fill,
                    mo.seller_fill,
                    mo.cleared,
                ];
                // Dead ticks: no trade; among them, no supply and no demand.
                let no_trade = !mo.trades();
                dead_t += one_if(no_trade, &inputs);
                no_s += one_if(no_trade && mo.supply == 0.0, &inputs);
                no_d += one_if(no_trade && mo.demand == 0.0, &inputs);
                // Fills over the ticks where their side exists, so a dead tick, whose fills
                // markets set to 0 (clearing.rs), is not counted as rationing.
                if side(mo.demand) {
                    let b = poisoned(mo.buyer_fill, &[mo.demand, mo.buyer_fill]);
                    worst_b = Some(worst_b.map_or(b, |w| min2(w, b)));
                    rat_b += one_if(rationed(b, rationed_below), &[b]);
                }
                if side(mo.supply) {
                    let v = poisoned(mo.seller_fill, &[mo.supply, mo.seller_fill]);
                    worst_s = Some(worst_s.map_or(v, |w| min2(w, v)));
                    rat_s += one_if(rationed(v, rationed_below), &[v]);
                }
                // A subnormal fill or volume (P0.4 amendment 6).
                let tiny = mo.buyer_fill.is_subnormal()
                    || mo.seller_fill.is_subnormal()
                    || mo.cleared.is_subnormal();
                subnormal += one_if(tiny, &inputs);
            }
            out.push(reading("dead.ticks", &name, seg, dead_t, None));
            out.push(reading("dead.no_supply", &name, seg, no_s, None));
            out.push(reading("dead.no_demand", &name, seg, no_d, None));
            if let Some(w) = worst_b {
                out.push(reading("fill.worst_buyer", &name, seg, w, None));
            }
            if let Some(w) = worst_s {
                out.push(reading("fill.worst_seller", &name, seg, w, None));
            }
            out.push(reading(
                "fill.rationed_buyer_ticks",
                &name,
                seg,
                rat_b,
                None,
            ));
            out.push(reading(
                "fill.rationed_seller_ticks",
                &name,
                seg,
                rat_s,
                None,
            ));
            out.push(reading("fill.subnormal_ticks", &name, seg, subnormal, None));
        }
        // Rationing per (market, class, side), R12: the worst filled/feasible over lines with
        // something feasible, its rationed ticks, Σ(requested − feasible), which is budget
        // rationing (N7) and never shows in a fill, and Σ(feasible − filled).
        let mut lines: Vec<(u32, usize, SideTag)> = span
            .iter()
            .flat_map(|o| {
                o.rationing
                    .iter()
                    .map(|r| (r.market, r.class.idx(), r.side))
            })
            .collect();
        lines.sort();
        lines.dedup();
        for (market, class, side_tag) in lines {
            let side_name = match side_tag {
                SideTag::Buy => "buy",
                SideTag::Sell => "sell",
            };
            let class_name = names
                .classes
                .get(class)
                .cloned()
                .unwrap_or_else(|| format!("class {class}"));
            let at = format!(
                "{}/{class_name}/{side_name}",
                names.market(usize::try_from(market).unwrap_or(usize::MAX))
            );
            let (mut worst, mut ticks, mut budget, mut short) = (None, 0.0, 0.0, 0.0);
            for o in span {
                let mine = o
                    .rationing
                    .iter()
                    .filter(|r| r.market == market && r.class.idx() == class && r.side == side_tag);
                for r in mine {
                    let inputs = [r.requested, r.feasible, r.filled];
                    budget += poisoned(r.requested - r.feasible, &inputs);
                    short += poisoned(r.feasible - r.filled, &inputs);
                    if side(r.feasible) {
                        let f = poisoned(r.filled / r.feasible, &inputs);
                        worst = Some(worst.map_or(f, |w| min2(w, f)));
                        ticks += one_if(rationed(f, rationed_below), &[f]);
                    }
                }
            }
            if let Some(w) = worst {
                out.push(reading("ration.worst", &at, seg, w, None));
            }
            out.push(reading("ration.rationed_ticks", &at, seg, ticks, None));
            out.push(reading("ration.budget_short", &at, seg, budget, None));
            out.push(reading("ration.market_short", &at, seg, short, None));
        }
        // Goods: the ticks with no consumption, for each good consumed anywhere; spoilage.
        for (g, &consumed) in consumed_anywhere.iter().enumerate() {
            let name = names.good(g);
            if consumed {
                let none = fold::sum(span.iter().map(|o| {
                    let c = o.consumed.get(g).copied().unwrap_or(0.0);
                    one_if(c == 0.0, &[c])
                }));
                out.push(reading("consumption.none_ticks", &name, seg, none, None));
            }
            let spoiled = fold::sum(
                span.iter()
                    .map(|o| o.spoiled.get(g).copied().unwrap_or(0.0)),
            );
            out.push(reading("spoiled.total", &name, seg, spoiled, None));
        }
        // Each provider's Σ(due − paid) and its ticks short.
        for &(a, _, _) in &last.transfers {
            let name = names.actor(a);
            let (mut short, mut short_ticks) = (0.0, 0.0);
            for o in span {
                for &(_, due, paid) in o.transfers.iter().filter(|(b, _, _)| *b == a) {
                    short += due - paid;
                    short_ticks += one_if(paid < due, &[due, paid]);
                }
            }
            out.push(reading("transfer.short", &name, seg, short, None));
            out.push(reading(
                "transfer.short_ticks",
                &name,
                seg,
                short_ticks,
                None,
            ));
        }
    }
    out
}

/// The run's `ScalePrice` firings.
pub fn price_shocks(obs: &[Obs]) -> u64 {
    obs.iter().map(|o| u64::from(o.price_shocks)).sum()
}
