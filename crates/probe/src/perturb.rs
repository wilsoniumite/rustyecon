//! Named perturbations (PROBE-SPEC §4.7 and the judges' added families; docs/probe/RULES.md
//! §6). A run's name is one or more terms joined by `+`:
//!
//! | term | what it does |
//! |---|---|
//! | `hold` | nothing: mode A |
//! | `w*F`, `r*F`, `pm*F`, `p*F` | that genesis price times F |
//! | `s*F` | the genesis human share 1 − x times F |
//! | `JA(F)` | w, p_m and p times F, s times F, r fixed |
//! | `JB(F)` | w times F, p_m times 1/F, p times F, s times 1/F, r fixed |
//! | `N(F)` | w, r, p_m and p times F, coin unchanged: a real-balance shock |
//! | `x*/2` | x = x\*/2 |
//! | `b=B@genesis` | the tape's b is B from tick 0; genesis stays at the b = 0.4 point |
//! | `b=B@dated` | b becomes B by a dated `SetParam` at tick L/4 of a mode-A run |
//! | `coin.ACTOR*F` | that actor's genesis coin times F |
//! | `good*F`, `mach*F` | the good desk's good, or the machine desk's machine services, times F |
//! | `joint(F,SEED)` | w, r, p_m and p each times F^u, s times 2^u, u uniform on [−1, 1] from a fixed generator seeded by SEED |
//! | `bcycle(P,C)` | C dated changes of b, P ticks apart from tick P, cycling 0.44, 0.36, 0.8, 0.2, 0.4 |

use crate::setup::{Setup, Share, Shock, ACTORS};

/// How a run's start distance D̂_0 is measured (PROBE-SPEC §4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// Mode A: nothing displaced.
    Hold,
    /// Over the displaced prices and the human share at genesis.
    Prices,
    /// Nominal, N(f): D̂_0 = 0, so VACUOUS and κ do not apply.
    Nominal,
    /// Over the displaced stocks, |ln f| of each (the reservation design's harness fix).
    Stocks,
    /// The distance between the two oracle points of a b shock.
    Shock,
}

/// A parsed run: its setup edits and how its start is measured.
#[derive(Debug, Clone, PartialEq)]
pub struct Perturbation {
    /// The name it was parsed from.
    pub name: String,
    terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq)]
enum Term {
    Hold,
    Price(usize, f64),
    Share(f64),
    Ja(f64),
    Jb(f64),
    Nominal(f64),
    XHalf,
    BGenesis(f64),
    BDated(f64),
    Coin(usize, f64),
    Good(f64),
    Mach(f64),
    Joint(f64, u64),
    BCycle(u64, u64),
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

fn term(t: &str) -> Result<Term, String> {
    if t == "hold" {
        return Ok(Term::Hold);
    }
    if t == "x*/2" {
        return Ok(Term::XHalf);
    }
    if let Some(a) = call(t, "JA") {
        return Ok(Term::Ja(number(a)?));
    }
    if let Some(a) = call(t, "JB") {
        return Ok(Term::Jb(number(a)?));
    }
    if let Some(a) = call(t, "N") {
        return Ok(Term::Nominal(number(a)?));
    }
    if let Some(a) = call(t, "joint") {
        let (f, seed) = a.split_once(',').ok_or("joint(F,SEED)")?;
        return Ok(Term::Joint(number(f)?, integer(seed)?));
    }
    if let Some(a) = call(t, "bcycle") {
        let (p, c) = a.split_once(',').ok_or("bcycle(P,C)")?;
        return Ok(Term::BCycle(integer(p)?, integer(c)?));
    }
    if let Some(rest) = t.strip_prefix("b=") {
        let (b, when) = rest.split_once('@').ok_or("b=B@genesis or b=B@dated")?;
        return match when {
            "genesis" => Ok(Term::BGenesis(number(b)?)),
            "dated" => Ok(Term::BDated(number(b)?)),
            _ => Err(format!("{t}: b=B@genesis or b=B@dated")),
        };
    }
    let (what, f) = t
        .rsplit_once('*')
        .ok_or_else(|| format!("{t}: not a run term"))?;
    let f = number(f)?;
    if let Some(actor) = what.strip_prefix("coin.") {
        let i = ACTORS
            .iter()
            .position(|a| *a == actor)
            .ok_or_else(|| format!("{t}: no actor {actor}"))?;
        return Ok(Term::Coin(i, f));
    }
    match what {
        "w" => Ok(Term::Price(0, f)),
        "r" => Ok(Term::Price(1, f)),
        "pm" => Ok(Term::Price(2, f)),
        "p" => Ok(Term::Price(3, f)),
        "s" => Ok(Term::Share(f)),
        "good" => Ok(Term::Good(f)),
        "mach" => Ok(Term::Mach(f)),
        _ => Err(format!("{t}: not a run term")),
    }
}

/// The deterministic generator for `joint`: SplitMix64.
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

/// The b values a `bcycle` visits, in order.
pub const B_CYCLE: [f64; 5] = [0.44, 0.36, 0.8, 0.2, 0.4];

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

    /// How the run's start distance is measured.
    pub fn start(&self) -> Start {
        let mut start = Start::Hold;
        for t in &self.terms {
            let this = match t {
                Term::Hold => Start::Hold,
                Term::Nominal(_) => Start::Nominal,
                Term::Coin(..) | Term::Good(_) | Term::Mach(_) => Start::Stocks,
                Term::BGenesis(_) | Term::BDated(_) | Term::BCycle(..) => Start::Shock,
                _ => Start::Prices,
            };
            // A price displacement outranks the rest, then a shock, then stocks.
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

    /// The factors of the displaced stocks, for a `Stocks` start.
    pub fn stock_factors(&self) -> Vec<f64> {
        self.terms
            .iter()
            .filter_map(|t| match t {
                Term::Coin(_, f) | Term::Good(f) | Term::Mach(f) => Some(*f),
                _ => None,
            })
            .collect()
    }

    /// Apply the run to a setup; `ticks` is the scored length L, which dates a `b=B@dated`
    /// shock at L/4.
    pub fn apply(&self, s: &mut Setup, ticks: u64) -> Result<(), String> {
        let x = &mut s.displace;
        for t in &self.terms {
            match *t {
                Term::Hold => {}
                Term::Price(i, f) => match i {
                    0 => x.w *= f,
                    1 => x.r *= f,
                    2 => x.pm *= f,
                    _ => x.p *= f,
                },
                Term::Share(f) => times_share(&mut x.share, f)?,
                Term::Ja(f) => {
                    x.w *= f;
                    x.pm *= f;
                    x.p *= f;
                    times_share(&mut x.share, f)?;
                }
                Term::Jb(f) => {
                    x.w *= f;
                    x.pm /= f;
                    x.p *= f;
                    times_share(&mut x.share, 1.0 / f)?;
                }
                Term::Nominal(f) => {
                    x.w *= f;
                    x.r *= f;
                    x.pm *= f;
                    x.p *= f;
                }
                Term::XHalf => {
                    let e = crate::setup::equilibrium(&s.instance, s.genesis_b, s.tpy)?;
                    x.share = Share::TechniqueAt(0.5 * e.x_star);
                }
                Term::BGenesis(b) => s.instance.b = b,
                Term::BDated(b) => s.shocks.push(Shock { tick: ticks / 4, b }),
                Term::Coin(i, f) => x.coin[i] *= f,
                Term::Good(f) => x.good *= f,
                Term::Mach(f) => x.mach *= f,
                Term::Joint(f, seed) => {
                    let mut g = SplitMix(seed);
                    let mut draw = |base: f64| rustyecon_core::num::pow(base, g.signed());
                    x.w *= draw(f);
                    x.r *= draw(f);
                    x.pm *= draw(f);
                    x.p *= draw(f);
                    let fs = draw(2.0);
                    times_share(&mut x.share, fs)?;
                }
                Term::BCycle(period, count) => {
                    for k in 0..count {
                        s.shocks.push(Shock {
                            tick: period * (k + 1),
                            b: B_CYCLE[(k % 5) as usize],
                        });
                    }
                }
            }
        }
        s.shocks.sort_by_key(|sh| sh.tick);
        Ok(())
    }

    /// The tick the scored clock starts at: a dated shock restarts it (PROBE-SPEC §4.4).
    pub fn clock_start(&self, ticks: u64) -> u64 {
        if self.terms.iter().any(|t| matches!(t, Term::BDated(_))) {
            ticks / 4
        } else {
            0
        }
    }
}

fn times_share(share: &mut Share, f: f64) -> Result<(), String> {
    match share {
        Share::Times(g) => {
            *g *= f;
            Ok(())
        }
        Share::TechniqueAt(_) => Err("the human share is set outright and then scaled".into()),
    }
}

/// The pre-registered battery of PROBE-SPEC §4.7: 57 runs, each with its tier.
pub fn battery() -> Vec<(String, u8)> {
    let mut out = Vec::new();
    for (f, tier) in [
        ("1.05", 1),
        ("0.95", 1),
        ("1.2", 2),
        ("0.8", 2),
        ("2", 3),
        ("0.5", 3),
    ] {
        for kind in ["w", "r", "pm", "p", "s"] {
            out.push((format!("{kind}*{f}"), tier));
        }
        for shape in ["JA", "JB", "N"] {
            out.push((format!("{shape}({f})"), tier));
        }
    }
    out.push(("x*/2".to_string(), 3));
    for (b, tier) in [("0.44", 2), ("0.36", 2), ("0.8", 3), ("0.2", 3)] {
        out.push((format!("b={b}@genesis"), tier));
        out.push((format!("b={b}@dated"), tier));
    }
    out
}

/// The added families (docs/probe/RULES.md §6), each a list of run names.
pub fn family(name: &str) -> Result<Vec<String>, String> {
    Ok(match name {
        "battery" => battery().into_iter().map(|(n, _)| n).collect(),
        // Q11's stock family, widened by the judges: design-adaptive's 22-run list, where it
        // applies to these rules, and the machine-stock shocks of the economist judge.
        "stocks" => {
            let mut v = Vec::new();
            for f in ["0.02", "0.1", "0.5", "2"] {
                v.push(format!("coin.desk.good*{f}"));
                v.push(format!("coin.desk.mach*{f}"));
            }
            for f in ["0.1", "2"] {
                v.push(format!("coin.workers*{f}"));
            }
            for f in ["0.5", "2"] {
                v.push(format!("coin.provider*{f}"));
            }
            for f in ["0.1", "0.5", "2"] {
                v.push(format!("good*{f}"));
                v.push(format!("mach*{f}"));
            }
            v.push("mach*0.01".to_string());
            v.push("mach*10".to_string());
            v.push("mach*0.1+coin.desk.mach*0.1".to_string());
            v
        }
        // The economist judge's joint random displacements: prices within ×/÷2 and ×/÷4.
        "joint2" => (1..=60).map(|k| format!("joint(2,{k})")).collect(),
        "joint4" => (1..=40).map(|k| format!("joint(4,{k})")).collect(),
        // The dynamics judge's shock history: b cycled through the battery's values.
        "history" => vec!["bcycle(1500,80)".to_string()],
        _ => return Err(format!("no family {name}")),
    })
}
