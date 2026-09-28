//! Named runs of the stocks probe (HORSES-SPEC §7.7, §7.10). A run's name is one or more terms
//! joined by `+`:
//!
//! | term | what it does |
//! |---|---|
//! | `hold` | nothing: mode A |
//! | `w*F`, `r*F`, `p[M]*F` | a market's genesis price times F (`w` labour's, `r` land's; M is `fodder`, the horse, `traction` or `good`) |
//! | `s*F` | the good desk's genesis human share 1 − x times F |
//! | `x*/2` | the good desk's x = x\*/2 |
//! | `JA(F)` | w and every price but r times F; s times F |
//! | `JB(F)` | w and the good's price times F, fodder's, the horse's and the horse-day's times 1/F; s times 1/F |
//! | `N(F)` | every price times F, coin unchanged |
//! | `b*F@genesis`, `b=V@genesis` | the county's machine land b is b·F (or V) from tick 0: fodder's land and the build's pasture together; genesis stays at the registered point |
//! | `b*F@dated`, `b=V@dated` | b becomes b·F (or V) by dated `SetParam`s at L/4; the scored clock restarts there |
//! | `coin.A*F` | actor A's genesis coin times F |
//! | `S*F` | stock S's genesis value times F, S one of [`stocks`](super::setup::stocks)' names |
//!
//! A stock or coin displacement moves no observable until the stock is traded or used, so its
//! start distance is the largest D̂ of its first year (HORSES-SPEC §7.5); on the flow path (R1a)
//! it is P2.0's, |ln F| over the tolerance, as P2.1's is.

use super::instance::{Instance, HOURS};
use super::setup::{stocks, Setup, Shock};
use crate::markets::perturb::FACTORS;
use crate::markets::setup::ShareAt;
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
    Share(f64),
    XHalf,
    Ja(f64),
    Jb(f64),
    Nominal(f64),
    Shock(Cost, bool),
    Coin(String, f64),
    Stock(String, f64),
}

/// A cost shock's value: b times a factor, or b set outright.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Cost {
    Times(f64),
    At(f64),
}

fn number(s: &str) -> Result<f64, String> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| format!("{s:?} is not a number"))
}

fn call<'a>(term: &'a str, name: &str) -> Option<&'a str> {
    term.strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')
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
    ] {
        if let Some(a) = call(t, name) {
            return Ok(make(number(a)?));
        }
    }
    if let Some(rest) = t.strip_prefix("b") {
        let dated = |when: &str| match when {
            "genesis" => Ok(false),
            "dated" => Ok(true),
            _ => Err(format!(
                "{t}: b*F@genesis, b=V@genesis, b*F@dated or b=V@dated"
            )),
        };
        if let Some(r) = rest.strip_prefix('*') {
            if let Some((f, when)) = r.split_once('@') {
                return Ok(Term::Shock(Cost::Times(number(f)?), dated(when)?));
            }
        }
        if let Some(r) = rest.strip_prefix('=') {
            let (v, when) = r
                .split_once('@')
                .ok_or_else(|| format!("{t}: b=V@genesis or b=V@dated"))?;
            return Ok(Term::Shock(Cost::At(number(v)?), dated(when)?));
        }
    }
    if let Some(r) = t.strip_prefix("p[") {
        let (m, after) = r.split_once(']').ok_or("p[M]*F")?;
        let f = after.strip_prefix('*').ok_or("p[M]*F")?;
        return Ok(Term::Price(m.to_string(), number(f)?));
    }
    let (what, f) = t
        .rsplit_once('*')
        .ok_or_else(|| format!("{t}: not a run term"))?;
    let f = number(f)?;
    if let Some(actor) = what.strip_prefix("coin.") {
        return Ok(Term::Coin(actor.to_string(), f));
    }
    match what {
        "w" => Ok(Term::Price("labour".into(), f)),
        "r" => Ok(Term::Price("land".into(), f)),
        "s" | "s[good]" => Ok(Term::Share(f)),
        _ => Ok(Term::Stock(what.to_string(), f)),
    }
}

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

    /// How the run's start distance is measured, as P2.0 and P2.1 measure it (PROBE-SPEC
    /// §4.5): a price or technique displacement outranks a cost shock, which outranks a stock.
    pub fn start(&self) -> Start {
        let mut start = Start::Hold;
        for t in &self.terms {
            let this = match t {
                Term::Hold => Start::Hold,
                Term::Nominal(_) => Start::Nominal,
                Term::Coin(..) | Term::Stock(..) => Start::Stocks,
                Term::Shock(..) => Start::Shock,
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
        self.terms.iter().any(|t| matches!(t, Term::Shock(_, true)))
    }

    /// The tick the scored clock starts at: a dated shock restarts it (PROBE-SPEC §4.4).
    pub fn clock_start(&self, ticks: u64) -> u64 {
        if self.dated() {
            ticks / 4
        } else {
            0
        }
    }

    /// Apply the run to a setup; `ticks` is the scored length L, which dates a dated shock at
    /// L/4.
    pub fn apply(&self, s: &mut Setup, ticks: u64) -> Result<(), String> {
        let inst: Instance = s.instance.clone();
        let markets = inst.markets();
        let actors = inst.actors();
        let names = stocks(&inst);
        let x = &mut s.displace;
        // The machine side: every market but labour, land and the good.
        let machine: Vec<usize> = (0..markets.len())
            .filter(|&m| !matches!(markets[m].as_str(), "labour" | "land" | "good"))
            .collect();
        let good = index(&markets, "good", "market")?;
        for t in &self.terms {
            match t {
                Term::Hold => {}
                Term::Price(m, f) => {
                    let m = if m == "hours" { HOURS } else { m.as_str() };
                    x.price[index(&markets, m, "market")?] *= f;
                }
                Term::Share(f) => times_share(&mut x.share, *f)?,
                Term::XHalf => {
                    let e = inst.point(s.tpy)?;
                    x.share = ShareAt::At(0.5 * e.x_star);
                }
                Term::Ja(f) => {
                    x.price[0] *= f;
                    for &m in &machine {
                        x.price[m] *= f;
                    }
                    x.price[good] *= f;
                    times_share(&mut x.share, *f)?;
                }
                Term::Jb(f) => {
                    x.price[0] *= f;
                    for &m in &machine {
                        x.price[m] /= f;
                    }
                    x.price[good] *= f;
                    times_share(&mut x.share, 1.0 / f)?;
                }
                Term::Nominal(f) => {
                    for p in x.price.iter_mut() {
                        *p *= f;
                    }
                }
                Term::Shock(cost, dated) => {
                    let b = match *cost {
                        Cost::Times(f) => inst.county.b * f,
                        Cost::At(v) => v,
                    };
                    if *dated {
                        s.shocks.push(Shock { tick: ticks / 4, b });
                    } else {
                        s.b_genesis = Some(b);
                    }
                }
                Term::Coin(a, f) => x.coin[index(&actors, a, "actor")?] *= f,
                Term::Stock(n, f) => x.stock[index(&names, n, "stock")?] *= f,
            }
        }
        s.shocks.sort_by_key(|sh| sh.tick);
        Ok(())
    }
}

/// One run of a battery: its name, its tier (1, 2, 3, or 4 for Tier 3S), and whether its
/// target is not one (an unfunded cost target, HORSES-SPEC §1.5).
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// The run's name.
    pub name: String,
    /// Its tier: 1 to 3, and 4 for Tier 3S (stocks and coins).
    pub tier: u8,
}

/// The registered battery of an instance (HORSES-SPEC §7.7): each price and the good desk's
/// technique alone, JA, JB and N, at ±5%, ±20%, ×2 and ×0.5; x\*/2; the county's machine land
/// at ×1.1, ×0.9, ×2 and ×0.5, at genesis and dated, where the target is funded; and Tier 3S,
/// every stock and every coin at ×0.5 and ×2.
pub fn battery(inst: &Instance, tpy: u32) -> Result<Vec<Run>, String> {
    let mut out = Vec::new();
    let mut push = |name: String, tier: u8| out.push(Run { name, tier });
    for (fs, _, tier) in FACTORS {
        for m in inst.markets() {
            let name = match m.as_str() {
                "labour" => format!("w*{fs}"),
                "land" => format!("r*{fs}"),
                _ => format!("p[{m}]*{fs}"),
            };
            push(name, tier);
        }
        push(format!("s*{fs}"), tier);
        for shape in ["JA", "JB", "N"] {
            push(format!("{shape}({fs})"), tier);
        }
    }
    push("x*/2".into(), 3);
    for (k, fs) in ["1.1", "0.9", "2", "0.5"].iter().enumerate() {
        let f: f64 = fs.parse().map_err(|_| "a factor")?;
        let target = inst.with_b(inst.county.b * f).point(tpy)?;
        if !target.funded {
            continue;
        }
        let tier = if k < 2 { 2 } else { 3 };
        push(format!("b*{fs}@genesis"), tier);
        push(format!("b*{fs}@dated"), tier);
    }
    for s in tier3s(inst) {
        push(s, 4);
    }
    Ok(out)
}

/// Tier 3S (HORSES-SPEC §7.7): every stock and every coin at ×0.5 and ×2.
pub fn tier3s(inst: &Instance) -> Vec<String> {
    let mut v = Vec::new();
    for s in stocks(inst) {
        for f in ["0.5", "2"] {
            v.push(format!("{s}*{f}"));
        }
    }
    for a in inst.actors() {
        for f in ["0.5", "2"] {
            v.push(format!("coin.{a}*{f}"));
        }
    }
    v
}

/// The added families (HORSES-SPEC §7.10), each a list of run names: `battery`, `tier3s`, and
/// `stocks` (item 9: every coin ×0.02 and ×0.1, every stock ×0.1 and ×10).
pub fn family(inst: &Instance, tpy: u32, name: &str) -> Result<Vec<String>, String> {
    Ok(match name {
        "battery" => battery(inst, tpy)?.into_iter().map(|r| r.name).collect(),
        "tier3s" => tier3s(inst),
        "stocks" => {
            let mut v = Vec::new();
            for a in inst.actors() {
                for f in ["0.02", "0.1"] {
                    v.push(format!("coin.{a}*{f}"));
                }
            }
            for s in stocks(inst) {
                for f in ["0.1", "10"] {
                    v.push(format!("{s}*{f}"));
                }
            }
            v
        }
        _ => return Err(format!("no family {name}: battery, tier3s or stocks")),
    })
}
