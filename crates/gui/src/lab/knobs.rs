//! Every number of an instance's parameters, by its path, for the lab's form and its sweeps.
//!
//! A path names the field as the oracle's parameter types do: `workers`, `schedule.eta`,
//! `work_cost.chi_max`, `categories[1].density[0]`, `machine_types[0].build.labor`,
//! `worker_types[1].efficiency`, `exits[0].gross`, `economy.parcels[1].acreage`,
//! `government.budget.dividend`. Each type's fields are read by name, so a field the oracle
//! adds is not editable here until it is named, and nothing breaks. The structure of an
//! instance (how many categories, types or parcels, an access, an exit form, the budget's
//! rule) comes from its preset and is not a knob.

use super::Instance;
use oracle::{
    Basket, Budget, Category, ExitForm, Government, MachineType, Parcel, ParcelParams,
    PowerSchedule, Recipe, UniformWorkCost, WorkerType,
};
use serde::Serialize;

/// A number the form can set.
pub enum Slot<'a> {
    /// A real.
    Real(&'a mut f64),
    /// A whole number of periods (a build lag).
    Periods(&'a mut u32),
    /// An index (the exit good's category).
    Index(&'a mut usize),
}

impl Slot<'_> {
    fn get(&self) -> f64 {
        match self {
            Slot::Real(v) => **v,
            Slot::Periods(n) => f64::from(**n),
            Slot::Index(n) => **n as f64,
        }
    }

    fn whole(&self) -> bool {
        !matches!(self, Slot::Real(_))
    }

    /// Set from text: a real parses as a double, a whole number as one.
    fn set(&mut self, text: &str) -> Result<(), String> {
        let t = text.trim();
        match self {
            Slot::Real(v) => {
                **v = t
                    .parse::<f64>()
                    .map_err(|_| format!("{t:?} is not a number"))?;
            }
            Slot::Periods(n) => {
                **n = whole(t)
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or_else(|| format!("{t:?} is not a whole number of periods"))?;
            }
            Slot::Index(n) => {
                **n = whole(t)
                    .and_then(|v| usize::try_from(v).ok())
                    .ok_or_else(|| format!("{t:?} is not an index"))?;
            }
        }
        Ok(())
    }
}

/// A whole number written as one (`3`) or as a double with no fraction (`3.0`).
fn whole(t: &str) -> Option<u64> {
    if let Ok(n) = t.parse::<u64>() {
        return Some(n);
    }
    let v = t.parse::<f64>().ok()?;
    (v.fract() == 0.0 && (0.0..9.0e15).contains(&v)).then_some(v as u64)
}

/// One knob as the form lists it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Knob {
    /// Its path.
    pub path: String,
    /// Its value.
    pub value: f64,
    /// Whether it is a whole number (a build lag, an index).
    pub whole: bool,
}

type Visit<'v> = dyn FnMut(&str, Slot<'_>) + 'v;

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}.{name}")
    }
}

fn reals(prefix: &str, name: &str, v: &mut [f64], f: &mut Visit<'_>) {
    for (i, x) in v.iter_mut().enumerate() {
        f(&format!("{}[{i}]", join(prefix, name)), Slot::Real(x));
    }
}

fn schedule(prefix: &str, s: &mut PowerSchedule, f: &mut Visit<'_>) {
    let p = join(prefix, "schedule");
    f(&join(&p, "eta"), Slot::Real(&mut s.eta));
    f(&join(&p, "g0"), Slot::Real(&mut s.g0));
    f(&join(&p, "g1"), Slot::Real(&mut s.g1));
    f(&join(&p, "k"), Slot::Real(&mut s.k));
}

fn work_cost(prefix: &str, w: &mut UniformWorkCost, f: &mut Visit<'_>) {
    f(
        &join(&join(prefix, "work_cost"), "chi_max"),
        Slot::Real(&mut w.chi_max),
    );
}

fn categories(prefix: &str, cats: &mut [Category], f: &mut Visit<'_>) {
    for (j, c) in cats.iter_mut().enumerate() {
        let p = format!("{}[{j}]", join(prefix, "categories"));
        f(&join(&p, "weight"), Slot::Real(&mut c.weight));
        f(&join(&p, "direct_land"), Slot::Real(&mut c.direct_land));
        reals(&p, "density", &mut c.density, f);
    }
}

fn recipe(prefix: &str, r: &mut Recipe, f: &mut Visit<'_>) {
    reals(prefix, "machines", &mut r.machines, f);
    f(&join(prefix, "labor"), Slot::Real(&mut r.labor));
    f(&join(prefix, "land"), Slot::Real(&mut r.land));
}

fn machine_types(prefix: &str, types: &mut [MachineType], f: &mut Visit<'_>) {
    for (k, t) in types.iter_mut().enumerate() {
        let p = format!("{}[{k}]", join(prefix, "machine_types"));
        f(
            &join(&p, "task_efficiency"),
            Slot::Real(&mut t.task_efficiency),
        );
        recipe(&join(&p, "operating"), &mut t.operating, f);
        recipe(&join(&p, "build"), &mut t.build, f);
        f(&join(&p, "delta"), Slot::Real(&mut t.delta));
        f(&join(&p, "build_lag"), Slot::Periods(&mut t.build_lag));
    }
}

fn matrix(prefix: &str, name: &str, m: &mut [Vec<f64>], f: &mut Visit<'_>) {
    for (j, row) in m.iter_mut().enumerate() {
        reals(prefix, &format!("{name}[{j}]"), row, f);
    }
}

fn worker_types(prefix: &str, types: &mut [WorkerType], f: &mut Visit<'_>) {
    for (i, w) in types.iter_mut().enumerate() {
        let p = format!("{}[{i}]", join(prefix, "worker_types"));
        f(&join(&p, "workers"), Slot::Real(&mut w.workers));
        work_cost(&p, &mut w.work_cost, f);
        f(&join(&p, "efficiency"), Slot::Real(&mut w.efficiency));
        f(&join(&p, "support"), Slot::Real(&mut w.support));
    }
}

fn parcels(prefix: &str, ps: &mut [Parcel], f: &mut Visit<'_>) {
    for (z, p) in ps.iter_mut().enumerate() {
        let q = format!("{}[{z}]", join(prefix, "parcels"));
        f(&join(&q, "acreage"), Slot::Real(&mut p.acreage));
        f(&join(&q, "quality"), Slot::Real(&mut p.quality));
    }
}

fn exits(prefix: &str, es: &mut [ExitForm], f: &mut Visit<'_>) {
    for (i, e) in es.iter_mut().enumerate() {
        if let ExitForm::Priced(x) = e {
            let p = format!("{}[{i}]", join(prefix, "exits"));
            f(&join(&p, "gross"), Slot::Real(&mut x.gross));
            f(&join(&p, "floor"), Slot::Real(&mut x.floor));
            f(&join(&p, "plot"), Slot::Real(&mut x.plot));
        }
    }
}

fn parcel_params(prefix: &str, p: &mut ParcelParams, f: &mut Visit<'_>) {
    parcels(prefix, &mut p.parcels, f);
    schedule(prefix, &mut p.schedule, f);
    f(&join(prefix, "rho"), Slot::Real(&mut p.rho));
    machine_types(prefix, &mut p.machine_types, f);
    reals(prefix, "edges", &mut p.edges, f);
    categories(prefix, &mut p.categories, f);
    matrix(prefix, "intermediate", &mut p.intermediate, f);
    worker_types(prefix, &mut p.worker_types, f);
    reals(prefix, "human_required", &mut p.human_required, f);
    matrix(prefix, "reserved", &mut p.reserved, f);
    exits(prefix, &mut p.exits, f);
    f(&join(prefix, "exit_good"), Slot::Index(&mut p.exit_good));
}

fn government(g: &mut Government, f: &mut Visit<'_>) {
    f("government.payroll", Slot::Real(&mut g.payroll));
    f("government.consumption", Slot::Real(&mut g.consumption));
    match &mut g.budget {
        Budget::RentRate { dividend } => f("government.budget.dividend", Slot::Real(dividend)),
        Budget::Dividend { rent_tax } => f("government.budget.rent_tax", Slot::Real(rent_tax)),
    }
    f("government.program.work", Slot::Real(&mut g.program.work));
    f("government.program.exit", Slot::Real(&mut g.program.exit));
}

/// Visit every knob of `inst`, in the order its type lists its fields.
pub fn visit(inst: &mut Instance, f: &mut Visit<'_>) {
    match inst {
        Instance::A(p) => {
            f("workers", Slot::Real(&mut p.workers));
            f("land", Slot::Real(&mut p.land));
            f("space", Slot::Real(&mut p.space));
            f("a", Slot::Real(&mut p.a));
            f("lam", Slot::Real(&mut p.lam));
            f("b", Slot::Real(&mut p.b));
            schedule("", &mut p.schedule, f);
            work_cost("", &mut p.work_cost, f);
            f("rho", Slot::Real(&mut p.rho));
            f("delta", Slot::Real(&mut p.delta));
            f("build_lag", Slot::Periods(&mut p.build_lag));
        }
        Instance::B(p) => {
            f("workers", Slot::Real(&mut p.workers));
            f("land", Slot::Real(&mut p.land));
            f("a", Slot::Real(&mut p.a));
            f("lam", Slot::Real(&mut p.lam));
            f("b", Slot::Real(&mut p.b));
            schedule("", &mut p.schedule, f);
            work_cost("", &mut p.work_cost, f);
            f("rho", Slot::Real(&mut p.rho));
            f("delta", Slot::Real(&mut p.delta));
            f("build_lag", Slot::Periods(&mut p.build_lag));
            reals("", "edges", &mut p.edges, f);
            categories("", &mut p.categories, f);
        }
        Instance::C(p) => {
            f("workers", Slot::Real(&mut p.workers));
            f("land", Slot::Real(&mut p.land));
            schedule("", &mut p.schedule, f);
            work_cost("", &mut p.work_cost, f);
            f("rho", Slot::Real(&mut p.rho));
            machine_types("", &mut p.machine_types, f);
            reals("", "edges", &mut p.edges, f);
            categories("", &mut p.categories, f);
            matrix("", "intermediate", &mut p.intermediate, f);
        }
        Instance::D(p) => {
            f("land", Slot::Real(&mut p.land));
            schedule("", &mut p.schedule, f);
            f("rho", Slot::Real(&mut p.rho));
            machine_types("", &mut p.machine_types, f);
            reals("", "edges", &mut p.edges, f);
            categories("", &mut p.categories, f);
            matrix("", "intermediate", &mut p.intermediate, f);
            worker_types("", &mut p.worker_types, f);
            reals("", "human_required", &mut p.human_required, f);
            matrix("", "reserved", &mut p.reserved, f);
        }
        Instance::E(p) => parcel_params("", p, f),
        Instance::F(p) => {
            parcel_params("economy", &mut p.economy, f);
            if let Basket::Ces { sigma } = &mut p.basket {
                f("basket.sigma", Slot::Real(sigma));
            }
            government(&mut p.government, f);
        }
    }
}

/// Every knob of `inst`, with its value.
pub fn list(inst: &Instance) -> Vec<Knob> {
    let mut copy = inst.clone();
    let mut out = Vec::new();
    visit(&mut copy, &mut |path, slot| {
        out.push(Knob {
            path: path.to_string(),
            value: slot.get(),
            whole: slot.whole(),
        });
    });
    out
}

/// The value of the knob at `path`.
pub fn get(inst: &Instance, path: &str) -> Option<f64> {
    list(inst)
        .into_iter()
        .find(|k| k.path == path)
        .map(|k| k.value)
}

/// Set the knob at `path` from text. The instance is changed only when the text reads; the
/// oracle validates the value when the instance is solved.
pub fn set(inst: &mut Instance, path: &str, text: &str) -> Result<(), String> {
    let mut result = Err(format!("no knob {path}"));
    visit(inst, &mut |p, mut slot| {
        if p == path {
            result = slot.set(text);
        }
    });
    result
}

/// Set the knob at `path` to a number: a whole-number knob takes it rounded to the nearest
/// whole number, and refuses one outside its range.
pub fn set_value(inst: &mut Instance, path: &str, v: f64) -> Result<(), String> {
    let mut result = Err(format!("no knob {path}"));
    visit(inst, &mut |p, slot| {
        if p != path {
            return;
        }
        result = match slot {
            Slot::Real(x) => {
                *x = v;
                Ok(())
            }
            Slot::Periods(n) => {
                let r = v.round();
                if (0.0..=f64::from(u32::MAX)).contains(&r) {
                    *n = r as u32;
                    Ok(())
                } else {
                    Err(format!("{v} is not a whole number of periods"))
                }
            }
            Slot::Index(n) => {
                let r = v.round();
                if (0.0..=1e15).contains(&r) {
                    *n = r as usize;
                    Ok(())
                } else {
                    Err(format!("{v} is not an index"))
                }
            }
        };
    });
    result
}
