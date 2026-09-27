//! The oracle lab (docs/GUI.md §9, G1): presets, the outputs beside their goldens, the field
//! plots and the sweeps, through the lab's own view-models and against the oracle called
//! directly.

use oracle::{Economy, Regime};
use rustyecon_gui::lab::presets::{self, appendix_b};
use rustyecon_gui::lab::{fields, goldens, knobs, Instance, OracleUnit};
use rustyecon_gui::vm::lab;

/// The build string the view-models are told.
const BUILD: &str = "test";

#[test]
fn lab_presets_solve_to_their_goldens() {
    // Every preset is its golden's economy: it solves to an equilibrium, and every output the
    // lab pairs with a golden of the same name agrees to 1e-12 relative, the oracle's own gate,
    // and every published value to 5e-6 absolute. Each unit has at least two presets; each
    // pairs v and at least one more output, and x* wherever its goldens have one.
    for unit in OracleUnit::ALL {
        assert!(presets::of(unit).len() >= 2, "unit {unit} has two presets");
    }
    let mut report = Vec::new();
    for p in presets::all() {
        let vm = lab::build(Some(&p), &p.instance(), BUILD);
        assert_eq!(vm.regime, "Interior", "{}: {:?}", p.id, vm.detail);
        assert!(!vm.edited);
        let keys: Vec<&str> = vm
            .outputs
            .iter()
            .filter(|o| o.golden.is_some())
            .map(|o| o.key.as_str())
            .collect();
        assert!(keys.contains(&"v"), "{}: v is paired: {keys:?}", p.id);
        let has_x = goldens::of(p.file, p.prefix)
            .iter()
            .any(|g| g.key == format!("{}_X_STAR", p.prefix));
        assert_eq!(keys.contains(&"x_star"), has_x, "{}: x* is paired", p.id);
        for o in &vm.outputs {
            if let Some(g) = &o.golden {
                if !g.agrees {
                    report.push(format!(
                        "{} {} = {} against {} = {} (diff {:?})",
                        p.id, o.key, o.value, g.key, g.text, g.diff
                    ));
                }
            }
            if let Some(g) = &o.published {
                assert!(g.agrees, "{} {} against {}", p.id, o.key, g.key);
            }
        }
        assert!(vm.paired.0 >= 2, "{}: {:?}", p.id, vm.paired);
        println!(
            "{}: {} outputs, {} paired, {} agree, {} goldens unpaired",
            p.id,
            vm.outputs.len(),
            vm.paired.0,
            vm.paired.1,
            vm.unpaired.len()
        );
    }
    assert!(report.is_empty(), "outputs off their goldens: {report:#?}");
}

#[test]
fn the_lab_shows_appendix_b_bit_for_bit() {
    // §9 G1's gate: the lab shows x* 0.86315, v 0.54344, Y 7.88061 and N_a 1.34338, bit for bit
    // the oracle's outputs. The oracle is called here directly, not through the lab.
    let eq = match Economy::new(appendix_b()).expect("valid").solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{other:?}"),
    };
    let p = presets::all()
        .into_iter()
        .find(|p| p.id == "G1")
        .expect("G1");
    let vm = lab::build(Some(&p), &p.instance(), BUILD);
    assert_eq!(vm.origin, "oracle");
    for (key, want, published) in [
        ("x_star", eq.x_star, "0.86315"),
        ("v", eq.v, "0.54344"),
        ("y", eq.y, "7.88061"),
        ("n_a", eq.n_a, "1.34338"),
    ] {
        let row = vm.outputs.iter().find(|o| o.key == key).expect(key);
        let shown: f64 = row.value.parse().expect("a number");
        assert_eq!(shown.to_bits(), want.to_bits(), "{key}");
        assert_eq!(row.number.map(f64::to_bits), Some(want.to_bits()), "{key}");
        let pubd = row.published.as_ref().expect("a published golden");
        assert_eq!(pubd.text, published, "{key}");
        assert!(pubd.agrees);
        assert!(row.golden.as_ref().is_some_and(|g| g.agrees), "{key}");
    }
}

#[test]
fn a_points_fields_are_the_oracles_doubles() {
    // The lab reads a point's numbers from the oracle's Debug: at G1, x, γ, n_D and n_S are the
    // oracle's `at(x)` bit for bit, f is its excess_demand, and every unit lists f first.
    let e = Economy::new(appendix_b()).expect("valid");
    for x in [1e-12, 0.25, 0.5, 0.863150418162437, 1.0] {
        let want = e.at(x);
        let inst = Instance::A(appendix_b());
        let v = inst.validate().expect("valid");
        let got = v.at(x).fields();
        let field = |k: &str| {
            got.iter()
                .find(|(n, _)| n == k)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| panic!("{k} in {got:?}"))
        };
        assert_eq!(got[0].0, fields::F);
        for (k, w) in [
            ("x", want.x),
            ("gamma", want.gamma),
            ("j", want.j),
            ("d", want.d),
            ("p_s", want.p_s),
            ("y", want.y),
            ("n_d", want.n_d),
            ("n_s", want.n_s),
            ("f", want.excess_demand()),
        ] {
            assert_eq!(field(k).to_bits(), w.to_bits(), "{k} at {x}");
        }
    }
    for p in presets::all() {
        let v = p.instance().validate().expect("valid");
        let f = v.at(0.5).fields();
        assert_eq!(f[0].0, "f", "{}", p.id);
        assert!(f.len() > 12, "{}: {} fields", p.id, f.len());
        assert!(
            f.iter().any(|(k, _)| k == "n_s" || k.ends_with(".n_s")),
            "{}: {f:?}",
            p.id
        );
    }
}

#[test]
fn f_over_x_shows_its_root_and_bracket() {
    // §9 G1: plot any point field over x in [0, 1], so f(x) = n_D − n_S shows its root and its
    // bracket. At G1, f changes sign at x*, which is a point of the curve, f(lo) > 0 > f(1),
    // and the bracket is the solve's (1e-12, 1).
    let inst = Instance::A(appendix_b());
    let c = lab::curve(&inst, "f", 400);
    assert_eq!(c.error, None);
    assert_eq!(c.regime, "Interior");
    assert_eq!(c.bracket, (oracle::BRACKET_LO, oracle::BRACKET_HI));
    let root = c.root.expect("a root");
    assert_eq!(root, 0.863150418162437);
    assert!(c.at_ends.0.unwrap() > 0.0 && c.at_ends.1.unwrap() < 0.0);
    assert!(c.at_root.unwrap().abs() < 1e-12);
    let pts: Vec<[f64; 2]> = c.segments.concat();
    assert!(pts.len() >= 402, "{}", pts.len());
    assert!(pts.iter().any(|p| p[0] == root));
    assert!(pts.iter().any(|p| p[0] == oracle::BRACKET_LO));
    let above = pts.iter().filter(|p| p[0] < root).all(|p| p[1] > 0.0);
    let below = pts.iter().filter(|p| p[0] > root).all(|p| p[1] < 0.0);
    assert!(above && below, "f changes sign once, at x*");
    // Any other field plots too; one a point lacks is refused by name.
    let g = lab::curve(&inst, "gamma", 10);
    assert_eq!(g.segments.concat().len(), 13);
    let bad = lab::curve(&inst, "no_such_field", 10);
    assert!(bad.error.unwrap().contains("no field no_such_field"));
    assert!(bad.fields.contains(&"n_d".to_string()));
}

#[test]
fn knobs_list_set_and_sweep() {
    // The form's knobs: every number of an instance by its path, set from text, refused when
    // the text does not read; a sweep solves each point and reads its outputs.
    let mut inst = Instance::A(appendix_b());
    let ks = knobs::list(&inst);
    let paths: Vec<&str> = ks.iter().map(|k| k.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "workers",
            "land",
            "space",
            "a",
            "lam",
            "b",
            "schedule.eta",
            "schedule.g0",
            "schedule.g1",
            "schedule.k",
            "work_cost.chi_max",
            "rho",
            "delta",
            "build_lag"
        ]
    );
    assert!(ks.last().unwrap().whole);
    knobs::set(&mut inst, "workers", "5").unwrap();
    assert_eq!(knobs::get(&inst, "workers"), Some(5.0));
    assert!(knobs::set(&mut inst, "workers", "five").is_err());
    assert!(knobs::set(&mut inst, "build_lag", "1.5").is_err());
    assert!(knobs::set(&mut inst, "nope", "1").is_err());
    assert_eq!(knobs::get(&inst, "workers"), Some(5.0));
    for p in presets::all() {
        let inst = p.instance();
        let ks = knobs::list(&inst);
        assert!(ks.len() >= 10, "{}: {}", p.id, ks.len());
        for k in &ks {
            let mut c = inst.clone();
            knobs::set(&mut c, &k.path, &format!("{:?}", k.value)).unwrap();
            assert_eq!(c, inst, "{} {}", p.id, k.path);
        }
    }
    let s = lab::sweep(
        &Instance::A(appendix_b()),
        "workers",
        2.0,
        8.0,
        200,
        &["x_star".to_string(), "v".to_string()],
    )
    .unwrap();
    assert_eq!(s.xs.len(), 200);
    assert_eq!((s.xs[0], s.xs[199]), (2.0, 8.0));
    assert_eq!(s.lines.len(), 2);
    for (i, &x) in s.xs.iter().enumerate() {
        let mut p = appendix_b();
        p.workers = x;
        let want = Economy::new(p).unwrap().solve().unwrap();
        match want {
            Regime::Interior(eq) => {
                assert_eq!(s.regimes[i], "Interior");
                assert_eq!(s.lines[0].values[i], Some(eq.x_star));
                assert_eq!(s.lines[1].values[i], Some(eq.v));
            }
            other => {
                assert_eq!(s.regimes[i], other.name());
                assert_eq!(s.lines[0].values[i], None);
            }
        }
    }
    assert!(lab::sweep(&Instance::A(appendix_b()), "nope", 0.0, 1.0, 10, &[]).is_err());
}

#[test]
fn an_edited_instance_is_no_longer_its_goldens() {
    // A knob changed from the preset: the view says edited, and no golden is paired, so a
    // golden is never shown beside another economy's output.
    let p = presets::all()
        .into_iter()
        .find(|p| p.id == "G1")
        .expect("G1");
    let mut inst = p.instance();
    knobs::set(&mut inst, "workers", "4.5").unwrap();
    let vm = lab::build(Some(&p), &inst, BUILD);
    assert!(vm.edited);
    assert!(vm
        .outputs
        .iter()
        .all(|o| o.golden.is_none() && o.published.is_none()));
    assert_eq!(vm.paired, (0, 0));
    assert!(vm.unpaired.is_empty());
    // An invalid instance says why.
    knobs::set(&mut inst, "a", "1.5").unwrap();
    let vm = lab::build(Some(&p), &inst, BUILD);
    assert_eq!(vm.regime, "invalid");
    assert!(vm.detail[0].1.contains("a = 1.5"), "{:?}", vm.detail);
}

/// A 200-point sweep's view-model builds in under 16 ms (§9 G1's gate; WSL, release, median
/// of 20): G1 swept over N from 2 to 8, reading x*, v, Y and N_a. Ignored by default: a
/// measurement, run by name by `scripts/gui.sh` on Linux.
#[test]
#[ignore = "a measurement: run by name"]
fn a_200_point_sweep_builds_in_under_16_ms() {
    let inst = Instance::A(appendix_b());
    let outs: Vec<String> = ["x_star", "v", "y", "n_a"]
        .iter()
        .map(ToString::to_string)
        .collect();
    let mut times = Vec::new();
    for _ in 0..20 {
        let t0 = std::time::Instant::now();
        let s = lab::sweep(&inst, "workers", 2.0, 8.0, 200, &outs).unwrap();
        times.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(s.xs.len(), 200);
    }
    times.sort_by(f64::total_cmp);
    let median = (times[9] + times[10]) / 2.0;
    println!(
        "a 200-point sweep of G1 over N: median {median:.3} ms of 20 ({:.3} to {:.3} ms)",
        times[0], times[19]
    );
    assert!(median < 16.0, "median {median} ms");
}

/// Each preset's 200-point sweep of its first real knob, ±10% about its value, timed: recorded,
/// never gated (G1). The heavier units' solves scan their paths, and a sweep runs on the UI
/// thread when asked for. Ignored by default: a measurement, run by name.
#[test]
#[ignore = "a measurement: run by name"]
fn every_presets_sweep_time_is_recorded() {
    for p in presets::all() {
        let inst = p.instance();
        let k = knobs::list(&inst)
            .into_iter()
            .find(|k| !k.whole && k.value > 0.0)
            .expect("a real knob");
        let outs = vec!["x_star".to_string(), "v".to_string()];
        let mut times = Vec::new();
        let mut last = None;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let s = lab::sweep(&inst, &k.path, k.value * 0.9, k.value * 1.1, 200, &outs).unwrap();
            times.push(t0.elapsed().as_secs_f64() * 1e3);
            last = Some(s);
        }
        times.sort_by(f64::total_cmp);
        let s = last.unwrap();
        let counts: Vec<String> = s.counts.iter().map(|(r, n)| format!("{r} {n}")).collect();
        println!(
            "{:<11} unit {} {:<28} median {:>8.2} ms of 5: {}",
            p.id,
            p.unit,
            k.path,
            times[2],
            counts.join(", ")
        );
    }
}
