//! Named runs of the markets probe (MARKETS-SPEC §7.7, §7.10). A run's name is one or more terms
//! joined by `+`:
//!
//! | term | what it does |
//! |---|---|
//! | `hold` | nothing: mode A |
//! | `p[M]*F`, `w*F`, `r*F` | market M's genesis price times F (`w` is labour's, `r` land's) |
//! | `s[D]*F` | category desk D's genesis human share 1 − x times F |
//! | `x*/2` | every category desk's x = x\*/2 |
//! | `JA(F)` | w and every good and type price times F, r fixed; every s times F |
//! | `JB(F)` | w times F, every type price times 1/F, every good price times F, every s times 1/F |
//! | `N(F)` | every price times F, coin unchanged: a real-balance shock |
//! | `RC(F)` | the first half of the categories' prices times F, the rest times 1/F |
//! | `RT(F)` | the first type's price times F, the second's times 1/F |
//! | `C=V@genesis` | the cost coefficient C (by its §1.5 name or its param) is V from tick 0; genesis stays at the registered point |
//! | `C=V@dated` | C becomes V by a dated `SetParam` at tick L/4 of a mode-A run; the scored clock restarts there |
//! | `coin.A*F` | actor A's genesis coin times F |
//! | `stock.D*F` | desk D's genesis stock of its output times F |
//! | `joint(F,SEED)` | every price times F^u, then every s times 2^u, u uniform on [−1, 1] from a fixed generator seeded by SEED |
//! | `cycle(C,P,N)` | N dated changes of C, P ticks apart from tick P, cycling ×1.1, ×0.9, ×2, ×0.5 and ×1 of its registered value |
//!
//! On I0 the grammar is P2.0's with the markets named: `p[mach]` is `pm`, `p[good]` is `p`,
//! `s[good]` is `s`, `stock.mach` is `mach`, `land.mach` is `b`, and `joint` draws in P2.0's
//! order.

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
    XHalf,
    Ja(f64),
    Jb(f64),
    Nominal(f64),
    Rc(f64),
    Rt(f64),
    Genesis(String, String),
    Dated(String, String),
    Coin(String, f64),
    Stock(String, f64),
    Joint(f64, u64),
    Cycle(String, u64, u64),
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
    ] {
        if let Some(a) = call(t, name) {
            return Ok(make(number(a)?));
        }
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
        ShareAt::At(_) => Err("the human share is set outright and then scaled".into()),
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
                Term::Genesis(..) | Term::Dated(..) | Term::Cycle(..) => Start::Shock,
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
        self.terms.iter().any(|t| matches!(t, Term::Dated(..)))
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
        let (nt, nc) = (types.len(), cats.len());
        let x = &mut s.displace;
        // Market m's index in `price`: labour 0, land 1, types from 2, categories after them.
        let type_at = |k: usize| 2 + k;
        let cat_at = |j: usize| 2 + nt + j;
        for t in &self.terms {
            match t {
                Term::Hold => {}
                Term::Price(m, f) => x.price[index(&markets, m, "market")?] *= f,
                Term::Share(d, f) => {
                    times_share(&mut x.share[index(&cats, d, "category desk")?], *f)?
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
                    for sh in x.share.iter_mut() {
                        times_share(sh, 1.0 / f)?;
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
                    for p in x.price.iter_mut() {
                        *p *= draw(*f);
                    }
                    for sh in x.share.iter_mut() {
                        let fs = draw(2.0);
                        times_share(sh, fs)?;
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

/// The added families (MARKETS-SPEC §7.10), each a list of run names.
pub fn family(inst: &Instance, tpy: u32, name: &str) -> Result<Vec<String>, String> {
    Ok(match name {
        "battery" => battery(inst, tpy)?.into_iter().map(|r| r.name).collect(),
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
            for f in ["0.5", "2"] {
                v.push(format!("coin.provider*{f}"));
            }
            v
        }
        // §7.10 item 6.
        "joint2" => (1..=60).map(|k| format!("joint(2,{k})")).collect(),
        "joint4" => (1..=40).map(|k| format!("joint(4,{k})")).collect(),
        // §7.10 item 8: the first coefficient cycled through its battery values and back.
        "history" => match inst.coefs.first() {
            Some(c) => vec![format!("cycle({},1500,80)", c.name)],
            None => Vec::new(),
        },
        // §7.10 item 7: 1.05^j out to ×/÷8 for w, r, the largest-share good, each type, and
        // the technique of the desk with the most tasks at x* (s only while s ≤ 1).
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
