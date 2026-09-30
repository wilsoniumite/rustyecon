//! Named runs of the markets probe (MARKETS-SPEC §7.7, §7.10). A run's name is one or more terms
//! joined by `+`:
//!
//! | term | what it does |
//! |---|---|
//! | `hold` | nothing: mode A |
//! | `p[M]*F`, `w*F`, `r*F` | market M's genesis price times F (`w` is labour's, `r` land's) |
//! | `s[D]*F` | category desk D's genesis human share 1 − x times F |
//! | `s[D]=V` | category desk D's genesis human share set to V exactly (the wall's; P2.3) |
//! | `x*/2` | every category desk's x = x\*/2 |
//! | `JA(F)` | w (every labour market's price) and every good and type price times F, r fixed; every s times F |
//! | `JB(F)` | w (every labour market's price) times F, every type price times 1/F, every good price times F, every s times 1/F |
//! | `N(F)` | every price times F, coin unchanged: a real-balance shock |
//! | `RC(F)` | the first half of the categories' prices times F, the rest times 1/F |
//! | `RT(F)` | the first type's price times F, the second's times 1/F |
//! | `RW(F)` | the pool's wage times F, each reserved wage times 1/F (several labour markets; P2.3) |
//! | `C=V@genesis` | the cost coefficient C (by its §1.5 name or its param) is V from tick 0; genesis stays at the registered point |
//! | `C=V@dated` | C becomes V by a dated `SetParam` at tick L/4 of a mode-A run; the scored clock restarts there |
//! | `coin.A*F` | actor A's genesis coin times F |
//! | `stock.D*F` | desk D's genesis stock of its output times F |
//! | `joint(F,SEED)` | every price times F^u, then every s times 2^u, u uniform on [−1, 1] from a fixed generator seeded by SEED |
//! | `cycle(C,P,N)` | N dated changes of C, P ticks apart from tick P, cycling ×1.1, ×0.9, ×2, ×0.5 and ×1 of its registered value |
//! | `enclose=F@genesis`, `enclose=F@dated` | enclosure by law (the commons; P2.3): a share F of the commons T_o moved to the enclosed land T, both changed from tick 0 or at L/4 |
//!
//! On I0 the grammar is P2.0's with the markets named: `p[mach]` is `pm`, `p[good]` is `p`,
//! `s[good]` is `s`, `stock.mach` is `mach`, `land.mach` is `b`, and `joint` draws in P2.0's
//! order. At the wall (a worker-form instance, the wall frame's §5.2) `joint` draws in the
//! frame's mirror's market order, labour, land, the categories, the types, then the reserved
//! labour markets, and the battery and families are the frame's, named as its mirror names them.
//! At the open-commons instances (the commons frame's §3.8; its registration §3) `joint` draws in
//! its mirror's order, labour, land, the categories, then the type; the battery and families are
//! named and ordered as its mirror's (`battery_c.run_list`), the commons' coefficient `commons`;
//! and the basin's factors are written as the mirror ran them, `1.05**j` to 12 significant digits.

use super::instance::Instance;
use super::setup::{Setup, ShareAt, Shock};
use crate::perturb::Start;

/// A parsed run: its terms, and the name it was parsed from.
#[derive(Debug, Clone, PartialEq)]
pub struct Perturbation {
    /// The name.
    pub name: String,
    terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq)]
enum Term {
    Hold,
    Price(String, f64),
    Share(String, f64),
    ShareIs(String, f64),
    XHalf,
    Ja(f64),
    Jb(f64),
    Nominal(f64),
    Rc(f64),
    Rt(f64),
    Rw(f64),
    Genesis(String, String),
    Dated(String, String),
    Coin(String, f64),
    Stock(String, f64),
    Joint(f64, u64),
    Cycle(String, u64, u64),
    Enclose(f64, bool),
}

fn number(s: &str) -> Result<f64, String> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| format!("{s:?} is not a number"))
}

fn integer(s: &str) -> Result<u64, String> {
    s.trim()
        .parse::<u64>()
        .map_err(|_| format!("{s:?} is not a whole number"))
}

fn call<'a>(term: &'a str, name: &str) -> Option<&'a str> {
    term.strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')
}

/// `name[inner]*F` → (inner, F).
fn bracket<'a>(t: &'a str, name: &str) -> Option<Result<(&'a str, f64), String>> {
    let rest = t.strip_prefix(name)?.strip_prefix('[')?;
    let (inner, after) = rest.split_once(']')?;
    let f = after.strip_prefix('*')?;
    Some(number(f).map(|f| (inner, f)))
}

fn term(t: &str) -> Result<Term, String> {
    if t == "hold" {
        return Ok(Term::Hold);
    }
    if t == "x*/2" {
        return Ok(Term::XHalf);
    }
    for (name, make) in [
        ("JA", Term::Ja as fn(f64) -> Term),
        ("JB", Term::Jb),
        ("N", Term::Nominal),
        ("RC", Term::Rc),
        ("RT", Term::Rt),
        ("RW", Term::Rw),
    ] {
        if let Some(a) = call(t, name) {
            return Ok(make(number(a)?));
        }
    }
    if let Some((d, v)) = t.strip_prefix("s[").and_then(|r| r.split_once("]=")) {
        return Ok(Term::ShareIs(d.to_string(), number(v)?));
    }
    if let Some(a) = call(t, "joint") {
        let (f, seed) = a.split_once(',').ok_or("joint(F,SEED)")?;
        return Ok(Term::Joint(number(f)?, integer(seed)?));
    }
    if let Some(a) = call(t, "cycle") {
        let parts: Vec<&str> = a.split(',').collect();
        let [c, p, n] = parts[..] else {
            return Err("cycle(C,P,N)".into());
        };
        return Ok(Term::Cycle(c.trim().to_string(), integer(p)?, integer(n)?));
    }
    if let Some(r) = bracket(t, "p") {
        let (m, f) = r?;
        return Ok(Term::Price(m.to_string(), f));
    }
    if let Some(r) = bracket(t, "s") {
        let (d, f) = r?;
        return Ok(Term::Share(d.to_string(), f));
    }
    if let Some((c, rest)) = t.split_once('=') {
        let (v, when) = rest.split_once('@').ok_or("C=V@genesis or C=V@dated")?;
        number(v)?;
        if c == "enclose" {
            return match when {
                "genesis" => Ok(Term::Enclose(number(v)?, false)),
                "dated" => Ok(Term::Enclose(number(v)?, true)),
                _ => Err(format!("{t}: enclose=F@genesis or enclose=F@dated")),
            };
        }
        return match when {
            "genesis" => Ok(Term::Genesis(c.to_string(), v.to_string())),
            "dated" => Ok(Term::Dated(c.to_string(), v.to_string())),
            _ => Err(format!("{t}: C=V@genesis or C=V@dated")),
        };
    }
    let (what, f) = t
        .rsplit_once('*')
        .ok_or_else(|| format!("{t}: not a run term"))?;
    let f = number(f)?;
    if let Some(actor) = what.strip_prefix("coin.") {
        return Ok(Term::Coin(actor.to_string(), f));
    }
    if let Some(desk) = what.strip_prefix("stock.") {
        return Ok(Term::Stock(desk.to_string(), f));
    }
    match what {
        "w" => Ok(Term::Price("labour".into(), f)),
        "r" => Ok(Term::Price("land".into(), f)),
        _ => Err(format!("{t}: not a run term")),
    }
}

/// The deterministic generator for `joint`: SplitMix64, as P2.0's.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform on [−1, 1].
    fn signed(&mut self) -> f64 {
        let unit = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        2.0 * unit - 1.0
    }
}

/// The factors a `cycle` visits, in order, on the coefficient's registered value.
pub const CYCLE: [f64; 5] = [1.1, 0.9, 2.0, 0.5, 1.0];

fn index(list: &[String], key: &str, what: &str) -> Result<usize, String> {
    list.iter()
        .position(|k| k == key)
        .ok_or_else(|| format!("no {what} {key} (the instance has {})", list.join(", ")))
}

fn times_share(share: &mut ShareAt, f: f64) -> Result<(), String> {
    match share {
        ShareAt::Times(g) => {
            *g *= f;
            Ok(())
        }
        ShareAt::At(_) | ShareAt::Is(_) => {
            Err("the human share is set outright and then scaled".into())
        }
    }
}

impl Perturbation {
    /// Parse a run name.
    pub fn parse(name: &str) -> Result<Perturbation, String> {
        let terms = name
            .split('+')
            .map(|t| term(t.trim()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Perturbation {
            name: name.to_string(),
            terms,
        })
    }

    /// How the run's start distance is measured, as P2.0 measures it (PROBE-SPEC §4.5): a
    /// price or technique displacement outranks a cost shock, which outranks a stock.
    pub fn start(&self) -> Start {
        let mut start = Start::Hold;
        for t in &self.terms {
            let this = match t {
                Term::Hold => Start::Hold,
                Term::Nominal(_) => Start::Nominal,
                Term::Coin(..) | Term::Stock(..) => Start::Stocks,
                Term::Genesis(..) | Term::Dated(..) | Term::Cycle(..) | Term::Enclose(..) => {
                    Start::Shock
                }
                _ => Start::Prices,
            };
            start = match (start, this) {
                (Start::Prices, _) | (_, Start::Prices) => Start::Prices,
                (Start::Shock, _) | (_, Start::Shock) => Start::Shock,
                (Start::Stocks, _) | (_, Start::Stocks) => Start::Stocks,
                (Start::Nominal, _) | (_, Start::Nominal) => Start::Nominal,
                _ => Start::Hold,
            };
        }
        start
    }

    /// The factors of the displaced stocks and coins, for a `Stocks` start.
    pub fn stock_factors(&self) -> Vec<f64> {
        self.terms
            .iter()
            .filter_map(|t| match t {
                Term::Coin(_, f) | Term::Stock(_, f) => Some(*f),
                _ => None,
            })
            .collect()
    }

    /// Whether the run has a dated shock, which restarts the scored clock at L/4.
    pub fn dated(&self) -> bool {
        self.terms
            .iter()
            .any(|t| matches!(t, Term::Dated(..) | Term::Enclose(_, true)))
    }

    /// The tick the scored clock starts at: a dated shock restarts it (PROBE-SPEC §4.4).
    pub fn clock_start(&self, ticks: u64) -> u64 {
        if self.dated() {
            ticks / 4
        } else {
            0
        }
    }

    /// Apply the run to a setup; `ticks` is the scored length L, which dates a `C=V@dated`
    /// shock at L/4.
    pub fn apply(&self, s: &mut Setup, ticks: u64) -> Result<(), String> {
        let inst: Instance = s.instance.clone();
        let markets = inst.markets();
        let cats: Vec<String> = inst.categories.iter().map(|c| c.key.clone()).collect();
        let types: Vec<String> = inst.types.iter().map(|t| t.key.clone()).collect();
        let actors = inst.actors();
        let desks = inst.desks();
        let (nt, nc, nw) = (types.len(), cats.len(), inst.wtypes.len());
        let x = &mut s.displace;
        // Market m's index in `price`: labour 0, land 1, types from 2, categories after them,
        // then the reserved labour markets.
        let type_at = |k: usize| 2 + k;
        let cat_at = |j: usize| 2 + nt + j;
        let wage_at = |i: usize| 2 + nt + nc + i;
        for t in &self.terms {
            match t {
                Term::Hold => {}
                Term::Price(m, f) => x.price[index(&markets, m, "market")?] *= f,
                Term::Share(d, f) => {
                    times_share(&mut x.share[index(&cats, d, "category desk")?], *f)?
                }
                Term::ShareIs(d, v) => {
                    x.share[index(&cats, d, "category desk")?] = ShareAt::Is(*v);
                }
                Term::XHalf => {
                    let e = inst.point(s.tpy)?;
                    for sh in x.share.iter_mut() {
                        *sh = ShareAt::At(0.5 * e.x_star);
                    }
                }
                Term::Ja(f) => {
                    x.price[0] *= f;
                    for k in 0..nt {
                        x.price[type_at(k)] *= f;
                    }
                    for j in 0..nc {
                        x.price[cat_at(j)] *= f;
                    }
                    // Every labour market's price is scaled as w is (the wall frame's §5.2).
                    for i in 0..nw {
                        x.price[wage_at(i)] *= f;
                    }
                    for sh in x.share.iter_mut() {
                        times_share(sh, *f)?;
                    }
                }
                Term::Jb(f) => {
                    x.price[0] *= f;
                    for k in 0..nt {
                        x.price[type_at(k)] /= f;
                    }
                    for j in 0..nc {
                        x.price[cat_at(j)] *= f;
                    }
                    for i in 0..nw {
                        x.price[wage_at(i)] *= f;
                    }
                    for sh in x.share.iter_mut() {
                        times_share(sh, 1.0 / f)?;
                    }
                }
                Term::Rw(f) => {
                    if nw == 0 {
                        return Err("RW needs several labour markets".into());
                    }
                    x.price[0] *= f;
                    for i in 0..nw {
                        x.price[wage_at(i)] /= f;
                    }
                }
                Term::Nominal(f) => {
                    for p in x.price.iter_mut() {
                        *p *= f;
                    }
                }
                Term::Rc(f) => {
                    if nc < 2 {
                        return Err("RC needs several categories".into());
                    }
                    let half = nc.div_ceil(2);
                    for j in 0..nc {
                        if j < half {
                            x.price[cat_at(j)] *= f;
                        } else {
                            x.price[cat_at(j)] /= f;
                        }
                    }
                }
                Term::Rt(f) => {
                    if nt != 2 {
                        return Err("RT needs two machine types".into());
                    }
                    x.price[type_at(0)] *= f;
                    x.price[type_at(1)] /= f;
                }
                Term::Genesis(c, v) => {
                    let param = inst
                        .coef(c)
                        .map(|c| c.param.clone())
                        .or_else(|_| inst.get(c).map(|_| c.clone()))?;
                    s.at_genesis.push((param, number(v)?));
                }
                Term::Dated(c, v) => {
                    let param = inst
                        .coef(c)
                        .map(|c| c.param.clone())
                        .or_else(|_| inst.get(c).map(|_| c.clone()))?;
                    s.shocks.push(Shock {
                        tick: ticks / 4,
                        param,
                        value: number(v)?,
                    });
                }
                Term::Coin(a, f) => x.coin[index(&actors, a, "actor")?] *= f,
                Term::Stock(d, f) => x.stock[index(&desks, d, "desk")?] *= f,
                Term::Joint(f, seed) => {
                    let mut g = SplitMix(*seed);
                    let mut draw = |base: f64| rustyecon_core::num::pow(base, g.signed());
                    // The draw order: the markets' order, or at the wall and the commons the
                    // frame's mirror's (labour, land, the categories, the types, the reserved
                    // labour markets).
                    let order: Vec<usize> = if inst.worker_form || inst.exit.is_some() {
                        let mut o = vec![0, 1];
                        o.extend((0..nc).map(cat_at));
                        o.extend((0..nt).map(type_at));
                        o.extend((0..nw).map(wage_at));
                        o
                    } else {
                        (0..x.price.len()).collect()
                    };
                    for m in order {
                        x.price[m] *= draw(*f);
                    }
                    for sh in x.share.iter_mut() {
                        let fs = draw(2.0);
                        times_share(sh, fs)?;
                    }
                }
                Term::Enclose(f, dated) => {
                    // Enclosure by law (unit-1e.md §2.2; the commons frame's §2.6): F·T_o of
                    // the commons becomes enclosed land, as `cm.shocked` computes it, per year.
                    let x = inst
                        .exit
                        .as_ref()
                        .ok_or("enclose needs an instance with a commons")?;
                    let moved = f * x.commons;
                    let changes = [
                        ("inst.commons".to_string(), x.commons - moved),
                        ("inst.land".to_string(), inst.land + moved),
                    ];
                    for (param, value) in changes {
                        if *dated {
                            s.shocks.push(Shock {
                                tick: ticks / 4,
                                param,
                                value,
                            });
                        } else {
                            s.at_genesis.push((param, value));
                        }
                    }
                }
                Term::Cycle(c, period, count) => {
                    let coef = inst.coef(c)?;
                    let base = number(&coef.base)?;
                    for k in 0..*count {
                        s.shocks.push(Shock {
                            tick: period * (k + 1),
                            param: coef.param.clone(),
                            value: base * CYCLE[(k % 5) as usize],
                        });
                    }
                }
            }
        }
        s.shocks.sort_by_key(|sh| sh.tick);
        Ok(())
    }
}

/// One run of a battery: its name, its tier, and whether it is slack (a technique displacement
/// that moves nothing real, MARKETS-SPEC §7.7).
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// The run's name.
    pub name: String,
    /// Its tier, 1 to 3.
    pub tier: u8,
    /// Whether it is slack.
    pub slack: bool,
}

/// The factors of the battery's singles and shapes, with their tiers (PROBE-SPEC §4.7).
pub const FACTORS: [(&str, f64, u8); 6] = [
    ("1.05", 1.05, 1),
    ("0.95", 0.95, 1),
    ("1.2", 1.2, 2),
    ("0.8", 0.8, 2),
    ("2", 2.0, 3),
    ("0.5", 0.5, 3),
];

/// Whether moving category desk `j`'s threshold from x\* to `x` leaves its recipe unchanged:
/// no segment it has tasks on meets the open interval between them.
pub fn slack(inst: &Instance, j: usize, x_star: f64, x: f64) -> bool {
    let (a, b) = if x < x_star { (x, x_star) } else { (x_star, x) };
    let mut edges = vec![0.0];
    edges.extend(inst.edges.iter().copied());
    edges.push(1.0);
    inst.categories[j]
        .density
        .iter()
        .enumerate()
        .all(|(s, &mu)| mu <= 0.0 || edges[s + 1].min(b) <= edges[s].max(a))
}

/// The registered battery of an instance (MARKETS-SPEC §7.7): each price and each category
/// desk's technique alone, JA, JB and N, RC where there are several categories, RT where there
/// are two types, at ±5%, ±20%, ×2 and ×0.5; x\*/2; and each registered cost coefficient at
/// ×1.1, ×0.9, ×2 and ×0.5, at genesis and dated. Slack runs are marked.
pub fn battery(inst: &Instance, tpy: u32) -> Result<Vec<Run>, String> {
    if inst.worker_form {
        return Ok(wall_battery(inst));
    }
    if inst.exit.is_some() {
        return commons_battery(inst, tpy);
    }
    let e = inst.point(tpy)?;
    let mut out = Vec::new();
    let mut push = |name: String, tier: u8, slack: bool| out.push(Run { name, tier, slack });
    for (fs, fv, tier) in FACTORS {
        for m in inst.markets() {
            let name = match m.as_str() {
                "labour" => format!("w*{fs}"),
                "land" => format!("r*{fs}"),
                _ => format!("p[{m}]*{fs}"),
            };
            push(name, tier, false);
        }
        for (j, c) in inst.categories.iter().enumerate() {
            let x = 1.0 - e.one_minus_x * fv;
            push(
                format!("s[{}]*{fs}", c.key),
                tier,
                slack(inst, j, e.x_star, x),
            );
        }
        for shape in ["JA", "JB", "N"] {
            push(format!("{shape}({fs})"), tier, false);
        }
        if inst.categories.len() > 1 {
            push(format!("RC({fs})"), tier, false);
        }
        if inst.types.len() == 2 {
            push(format!("RT({fs})"), tier, false);
        }
    }
    push("x*/2".into(), 3, false);
    for c in &inst.coefs {
        for (k, v) in c.values.iter().enumerate() {
            let tier = if k < 2 { 2 } else { 3 };
            push(format!("{}={v}@genesis", c.name), tier, false);
            push(format!("{}={v}@dated", c.name), tier, false);
        }
    }
    Ok(out)
}

/// x^j for a whole j, correctly rounded: the power in double-double arithmetic (each product's
/// rounding error kept by a fused multiply-add and carried), a double-double reciprocal for
/// j < 0, and one rounding to a double at the end. The wall frame's mirror names its basin runs
/// by Python's `1.05 ** j`, which is correctly rounded at every j in ±1 … ±43 (checked against
/// exact rationals), where libm's `pow` is an ulp off at some; so the wall's basin family
/// carries the registered names. P2.1's basin keeps `num::pow`.
pub fn pow_whole(x: f64, j: i32) -> f64 {
    let (mut hi, mut lo) = (1.0_f64, 0.0_f64);
    for _ in 0..j.unsigned_abs() {
        let p = hi * x;
        let e = rustyecon_core::num::fma(hi, x, -p) + lo * x;
        (hi, lo) = rustyecon_core::num::two_sum(p, e);
    }
    if j >= 0 {
        return hi + lo;
    }
    let q1 = 1.0 / hi;
    let r = rustyecon_core::num::fma(-q1, hi, 1.0) - q1 * lo;
    q1 + r / hi
}

/// The shares `s[D]=V` takes at the wall, with their tiers (the wall frame's §5.2).
pub const WALL_SHARES: [(&str, u8); 3] = [("0.05", 1), ("0.2", 2), ("0.5", 3)];

/// The wall's registered battery (the wall frame's §5.2; decision 396), in its mirror's order and
/// with its names (`wb.run_list`): each market's price at the six factors, factor by factor; each
/// category desk's share set to 0.05, 0.2 and 0.5; JA, JB, N, RC and RW at the six factors;
/// x\*/2; and each registered cost coefficient at ×1.1, ×0.9, ×2 and ×0.5, at genesis and dated.
/// The markets are named in the mirror's order: labour, land, the categories, the types, the
/// reserved labour markets. No run is slack.
pub fn wall_battery(inst: &Instance) -> Vec<Run> {
    let mut out = Vec::new();
    let mut push = |name: String, tier: u8| {
        out.push(Run {
            name,
            tier,
            slack: false,
        })
    };
    for (fs, _, tier) in FACTORS {
        for m in wall_markets(inst) {
            push(format!("p[{m}]*{fs}"), tier);
        }
    }
    for c in &inst.categories {
        for (v, tier) in WALL_SHARES {
            push(format!("s[{}]={v}", c.key), tier);
        }
    }
    for shape in ["JA", "JB", "N", "RC", "RW"] {
        for (fs, _, tier) in FACTORS {
            push(format!("{shape}({fs})"), tier);
        }
    }
    push("x*/2".into(), 3);
    for c in &inst.coefs {
        for (k, v) in c.values.iter().enumerate() {
            let tier = if k < 2 { 2 } else { 3 };
            push(format!("{}={v}@genesis", c.name), tier);
            push(format!("{}={v}@dated", c.name), tier);
        }
    }
    out
}

/// The open-commons battery (the commons frame's §5.3; decision 399), in its mirror's order and
/// with its names (`battery_c.run_list`): each market's price at the six factors, market by market
/// in the mirror's order (labour, land, the categories, the type); each category desk's share at
/// the six factors; JA, JB and N, then RC, at the six factors; x\*/2; and each registered cost
/// coefficient (land.mach, b.food, the commons) at ×1.1, ×0.9, ×2 and ×0.5, at genesis and dated.
/// 115 runs; the share runs whose recipe does not change are marked slack.
pub fn commons_battery(inst: &Instance, tpy: u32) -> Result<Vec<Run>, String> {
    let e = inst.point(tpy)?;
    let mut out = Vec::new();
    let mut push = |name: String, tier: u8, slack: bool| out.push(Run { name, tier, slack });
    for m in wall_markets(inst) {
        for (fs, _, tier) in FACTORS {
            push(format!("p[{m}]*{fs}"), tier, false);
        }
    }
    for (j, c) in inst.categories.iter().enumerate() {
        for (fs, fv, tier) in FACTORS {
            let x = 1.0 - e.one_minus_x * fv;
            push(
                format!("s[{}]*{fs}", c.key),
                tier,
                slack(inst, j, e.x_star, x),
            );
        }
    }
    let mut shapes = vec!["JA", "JB", "N"];
    if inst.categories.len() > 1 {
        shapes.push("RC");
    }
    for shape in shapes {
        for (fs, _, tier) in FACTORS {
            push(format!("{shape}({fs})"), tier, false);
        }
    }
    push("x*/2".into(), 3, false);
    for c in &inst.coefs {
        for (k, v) in c.values.iter().enumerate() {
            let tier = if k < 2 { 2 } else { 3 };
            push(format!("{}={v}@genesis", c.name), tier, false);
            push(format!("{}={v}@dated", c.name), tier, false);
        }
    }
    Ok(out)
}

/// Python's `%.12g` of x (the commons mirror's basin names, `battery_c`): x to 12 significant
/// digits, correctly rounded, trailing zeros and a trailing point dropped, positional where the
/// decimal exponent is in [−4, 12) and scientific (`1.5e-05`) outside it.
pub fn g12(x: f64) -> String {
    let sci = format!("{x:.11e}");
    let Some((mant, exp)) = sci.split_once('e') else {
        return sci;
    };
    let Ok(exp) = exp.parse::<i32>() else {
        return sci;
    };
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(char::is_ascii_digit).collect();
    let trim = |s: String| -> String {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    };
    let body = if !(-4..12).contains(&exp) {
        let m = trim(format!("{}.{}", &digits[..1], &digits[1..]));
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{m}e{sign}{:02}", exp.abs())
    } else if exp >= 0 {
        let k = (exp + 1) as usize;
        trim(format!("{}.{}", &digits[..k], &digits[k..]))
    } else {
        let zeros = "0".repeat((-exp - 1) as usize);
        trim(format!("0.{zeros}{digits}"))
    };
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

/// The markets in the wall frame's mirror's order: labour, land, the categories, the types, then
/// the reserved labour markets.
pub fn wall_markets(inst: &Instance) -> Vec<String> {
    let mut m = vec!["labour".to_string(), "land".to_string()];
    m.extend(inst.categories.iter().map(|c| c.key.clone()));
    m.extend(inst.types.iter().map(|t| t.key.clone()));
    m.extend(inst.wtypes.iter().map(|t| t.market()));
    m
}

/// Tier 3S (decisions 229, 368 and 397; the wall frame's §5.2): each desk's stock, each desk's
/// coin, and each household's coin (the workers, each reserved pop, then the provider), at ×0.5
/// and ×2, in its mirror's order (`run_battery.tier3s_names`). Its start distance is the largest
/// D̂ of its first year (`Setup::first_year_d0`).
pub fn tier3s(inst: &Instance) -> Vec<String> {
    let mut v = Vec::new();
    for d in inst.desks() {
        for f in ["0.5", "2"] {
            v.push(format!("stock.{d}*{f}"));
        }
    }
    for d in inst.desks() {
        for f in ["0.5", "2"] {
            v.push(format!("coin.desk.{d}*{f}"));
        }
    }
    let mut pops = vec!["workers".to_string()];
    pops.extend(inst.wtypes.iter().map(|t| t.pop()));
    pops.push("provider".into());
    for a in pops {
        for f in ["0.5", "2"] {
            v.push(format!("coin.{a}*{f}"));
        }
    }
    v
}

/// The added families (MARKETS-SPEC §7.10), each a list of run names.
pub fn family(inst: &Instance, tpy: u32, name: &str) -> Result<Vec<String>, String> {
    Ok(match name {
        "battery" => battery(inst, tpy)?.into_iter().map(|r| r.name).collect(),
        "tier3s" => tier3s(inst),
        // §7.10 item 5.
        "stocks" => {
            let mut v = Vec::new();
            for d in inst.desks() {
                for f in ["0.02", "0.1", "0.5", "2"] {
                    v.push(format!("coin.desk.{d}*{f}"));
                }
            }
            for d in inst.desks() {
                for f in ["0.1", "0.5", "2"] {
                    v.push(format!("stock.{d}*{f}"));
                }
            }
            for t in &inst.types {
                for f in ["0.01", "10"] {
                    v.push(format!("stock.{}*{f}", t.key));
                }
            }
            for f in ["0.1", "2"] {
                v.push(format!("coin.workers*{f}"));
            }
            // Each reserved pop's coin, as the workers' (the wall frame's §5.2).
            for t in &inst.wtypes {
                for f in ["0.1", "2"] {
                    v.push(format!("coin.{}*{f}", t.pop()));
                }
            }
            for f in ["0.5", "2"] {
                v.push(format!("coin.provider*{f}"));
            }
            v
        }
        // §7.10 item 6.
        "joint2" => (1..=60).map(|k| format!("joint(2,{k})")).collect(),
        "joint4" => (1..=40).map(|k| format!("joint(4,{k})")).collect(),
        // §7.10 item 8: the first coefficient cycled through its battery values and back; at
        // the commons (the commons frame's §5.4 item 4) the commons as well, across its regimes.
        "history" if inst.exit.is_some() => vec![
            "cycle(land.mach,1500,80)".to_string(),
            "cycle(commons,1500,80)".to_string(),
        ],
        "history" => match inst.coefs.first() {
            Some(c) => vec![format!("cycle({},1500,80)", c.name)],
            None => Vec::new(),
        },
        // The commons frame's §5.4 item 7: enclosure by law, half and all of the commons.
        "enclose" if inst.exit.is_some() => {
            let mut v = Vec::new();
            for f in ["0.5", "1"] {
                for when in ["genesis", "dated"] {
                    v.push(format!("enclose={f}@{when}"));
                }
            }
            v
        }
        // The commons frame's §5.4 item 3 (`run_battery_c.basin_jobs`): w, r, the exit good
        // (food, the largest share), the machine and the exit good's technique, at 1.05^j written
        // to 12 significant digits, j = −43 … 43 but 0; a share past 1 is set to 1 (genesis).
        "basin" if inst.exit.is_some() => {
            let x = inst.exit.as_ref().ok_or("no exit")?;
            let mut vars = vec!["labour".to_string(), "land".to_string(), x.good.clone()];
            vars.extend(inst.types.iter().map(|t| t.key.clone()));
            let mut v = Vec::new();
            let js = || (-43i32..=43).filter(|&j| j != 0);
            for var in &vars {
                for j in js() {
                    v.push(format!("p[{var}]*{}", g12(pow_whole(1.05, j))));
                }
            }
            for j in js() {
                v.push(format!("s[{}]*{}", x.good, g12(pow_whole(1.05, j))));
            }
            v
        }
        // §7.10 item 7: 1.05^j out to ×/÷8 for w, r, the largest-share good, each type, and
        // the technique of the desk with the most tasks at x* (s only while s ≤ 1).
        // At the wall (the wall frame's §5.2): w, r, the largest-share good, each type and each
        // reserved wage, named p[M] as its mirror names them; the technique has no displacement
        // at s* = 0, so none is taken.
        "basin" if inst.worker_form => {
            let e = inst.point(tpy)?;
            let share = |j: usize| inst.categories[j].weight * e.cat_price[j];
            let big = (0..inst.categories.len())
                .max_by(|&a, &b| share(a).total_cmp(&share(b)))
                .ok_or("no category")?;
            let mut vars = vec!["labour".to_string(), "land".to_string()];
            vars.push(inst.categories[big].key.clone());
            vars.extend(inst.types.iter().map(|t| t.key.clone()));
            vars.extend(inst.wtypes.iter().map(|t| t.market()));
            let mut v = Vec::new();
            for var in vars {
                for j in -43i32..=43 {
                    if j != 0 {
                        let f = pow_whole(1.05, j);
                        v.push(format!("p[{var}]*{f:?}"));
                    }
                }
            }
            v
        }
        "basin" => {
            let e = inst.point(tpy)?;
            let mut vars: Vec<String> = vec!["w".into(), "r".into()];
            let share = |j: usize| inst.categories[j].weight * e.cat_price[j];
            let big = (0..inst.categories.len())
                .max_by(|&a, &b| share(a).total_cmp(&share(b)))
                .ok_or("no category")?;
            vars.push(format!("p[{}]", inst.categories[big].key));
            for t in &inst.types {
                vars.push(format!("p[{}]", t.key));
            }
            let tasks = |j: usize| {
                let c = &inst.categories[j];
                let mut edges = vec![0.0];
                edges.extend(inst.edges.iter().copied());
                edges.push(1.0);
                let mut m = 0.0;
                for (s, mu) in c.density.iter().enumerate() {
                    m += mu * (edges[s + 1].min(e.x_star) - edges[s].min(e.x_star));
                }
                m * c.weight
            };
            let most = (0..inst.categories.len())
                .max_by(|&a, &b| tasks(a).total_cmp(&tasks(b)))
                .ok_or("no category")?;
            let s_var = format!("s[{}]", inst.categories[most].key);
            vars.push(s_var.clone());
            let mut v = Vec::new();
            for var in vars {
                for j in -43i32..=43 {
                    if j == 0 {
                        continue;
                    }
                    let f = rustyecon_core::num::pow(1.05, f64::from(j));
                    if var == s_var && f * e.one_minus_x > 1.0 {
                        continue;
                    }
                    v.push(format!("{var}*{f:?}"));
                }
            }
            v
        }
        _ => return Err(format!("no family {name}")),
    })
}
