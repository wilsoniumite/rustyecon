//! Source scans of the GUI (docs/GUI.md §8.1): they hold its seams. The egui-free modules name
//! no egui crate and reach nothing that draws (U9); the run's side and the view-models reach no
//! model, file, thread or clock, so they can move into `crates/observe` at G2 (D13); drawing
//! trigonometry lives in `ui/` alone and `core::num` gains none (D12); and the engine's
//! `no_raw_transcendentals` and `no_hashed_collections`, copied, hold over the egui-free
//! modules (U9). Each scan is checked on a fixture first, so an empty scan cannot pass.

mod common;

use common::scan::{
    all_sources, egui_free_sources, shipped, shipped_tokens, sources_under, src, strip, tokens,
    Tok, EGUI_FREE,
};

fn ident(t: &Tok, name: &str) -> bool {
    matches!(t, Tok::Ident(s) if s == name)
}

fn punct(t: &Tok, ch: char) -> bool {
    matches!(t, Tok::Punct(c) if *c == ch)
}

/// The tokens of a fixture, as the scans read a file.
fn fixture(src: &str) -> Vec<Tok> {
    tokens(&shipped(&strip(src)))
}

/// Whether `toks[i]` is the last segment of a path `a::b::…::toks[i]` whose first segment is in
/// `roots`: `crate::model`, `super::super::ui`, `rustyecon_gui::app`.
fn path_from(toks: &[Tok], i: usize, roots: &[&str]) -> bool {
    let mut j = i;
    while j >= 3 && punct(&toks[j - 1], ':') && punct(&toks[j - 2], ':') {
        j -= 3;
        if let Tok::Ident(s) = &toks[j] {
            if roots.contains(&s.as_str()) {
                return true;
            }
        } else {
            return false;
        }
    }
    false
}

/// The roots of a path into this crate.
const CRATE_ROOTS: [&str; 4] = ["crate", "self", "super", "rustyecon_gui"];
/// The roots of a path into the standard library.
const STD_ROOTS: [&str; 3] = ["std", "core", "alloc"];

/// For each token, the first segment of the `use` tree it sits in, if it sits in one: in
/// `use crate::{run::Store, ui as _};` every token after `use` up to the `;` has the root
/// `crate`. A tree that opens with a group gives each of its branches its own root: in
/// `use {std::fmt as _, crate::{ui as _}};` the second branch's tokens have the root `crate`
/// (G1; O20, what G0.1's re-check found). A `use<…>` bound is not a declaration.
fn use_roots(toks: &[Tok]) -> Vec<Option<String>> {
    let mut roots = vec![None; toks.len()];
    let mut i = 0;
    while i < toks.len() {
        let decl = ident(&toks[i], "use") && !toks.get(i + 1).is_some_and(|t| punct(t, '<'));
        if !decl {
            i += 1;
            continue;
        }
        let end = (i + 1..toks.len())
            .find(|&j| punct(&toks[j], ';'))
            .unwrap_or(toks.len());
        let open = (i + 1..end).find(|&j| !punct(&toks[j], ':'));
        if open.is_some_and(|j| punct(&toks[j], '{')) {
            let mut depth = 0_usize;
            let mut root: Option<String> = None;
            for (k, r) in roots.iter_mut().enumerate().take(end).skip(i + 1) {
                match &toks[k] {
                    Tok::Punct('{') => {
                        depth += 1;
                        if depth == 1 {
                            root = None;
                        }
                    }
                    Tok::Punct('}') => depth = depth.saturating_sub(1),
                    Tok::Punct(',') if depth == 1 => root = None,
                    Tok::Ident(s) if depth >= 1 && root.is_none() => root = Some(s.clone()),
                    _ => {}
                }
                r.clone_from(&root);
            }
        } else {
            let root = toks[i + 1..end].iter().find_map(|t| match t {
                Tok::Ident(s) => Some(s.clone()),
                _ => None,
            });
            for r in &mut roots[i + 1..end] {
                r.clone_from(&root);
            }
        }
        i = end;
    }
    roots
}

/// `from` and every name a `use` gives one of them in the file: `use crate as g;` makes `g` a
/// root of this crate, so `g::ui::layout` reaches `ui` (G1; O20, what G0.1's re-check found).
fn with_aliases(toks: &[Tok], roots: &[Option<String>], from: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = from.iter().map(|s| (*s).to_string()).collect();
    for k in 0..toks.len() {
        let Tok::Ident(s) = &toks[k] else { continue };
        let at_root = roots[k].as_deref() == Some(s.as_str()) && from.contains(&s.as_str());
        let renamed = toks.get(k + 1).is_some_and(|t| ident(t, "as"));
        if let (true, true, Some(Tok::Ident(alias))) = (at_root, renamed, toks.get(k + 2)) {
            if alias != "_" && !out.contains(alias) {
                out.push(alias.clone());
            }
        }
    }
    out
}

/// Whether `toks[i]`, an identifier, is a path segment reached from one of `from`: a segment of
/// a `use` tree rooted there, groups and renames included (`use crate::{run::Store, ui as _}`);
/// the last segment of a path from there (`crate::ui::layout`); or the first segment of a bare
/// path (`ui::layout::Pane`), which only a glob import or a `use` brings into scope.
fn reached(toks: &[Tok], roots: &[Option<String>], i: usize, from: &[&str]) -> bool {
    let in_use = roots[i].as_deref().is_some_and(|r| from.contains(&r));
    let colons = |j: usize| {
        toks.get(j).is_some_and(|t| punct(t, ':')) && toks.get(j + 1).is_some_and(|t| punct(t, ':'))
    };
    let bare = colons(i + 1) && !(i >= 2 && colons(i - 2));
    in_use || path_from(toks, i, from) || bare
}

/// Whether `toks[i]` is the `*` of a glob import from one of `from`: `use crate::*;`.
fn glob_from(toks: &[Tok], roots: &[Option<String>], i: usize, from: &[&str]) -> bool {
    punct(&toks[i], '*') && roots[i].as_deref().is_some_and(|r| from.contains(&r))
}

#[test]
fn the_scanner_reads_what_it_should() {
    // The copied lexer, checked as the engine checks its own: comments and strings are not
    // code, tuple indices are not floats, and a `#[cfg(test)]` item is cut out while the code
    // after it is kept.
    let src = r##"
        // .sin( and egui in a comment
        /* nested /* HashMap */ still */
        let s = "f64::ln and eframe in a string";
        let r = r#"libm"#;
        let l = x.0.1;
        #[cfg(test)]
        mod tests { use egui::Ui; fn f(x: f64) -> f64 { x.sin() } }
        let a = 2.5; use crate::run::Store;
    "##;
    let toks = fixture(src);
    let names: Vec<&str> = toks
        .iter()
        .filter_map(|t| match t {
            Tok::Ident(s) => Some(s.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        names,
        ["let", "s", "let", "r", "let", "l", "x", "let", "a", "use", "crate", "run", "Store"]
    );
    assert!(toks.contains(&Tok::Num("2.5".to_string(), true)));
    assert!(toks.contains(&Tok::Num("1".to_string(), false)));
    // The lists of files are what they claim: the egui-free modules hold files, the ui holds
    // the tile layout, and nothing under ui/ counts as egui-free.
    let free = egui_free_sources();
    for m in ["model", "run", "edit", "vm", "drive", "platform", "lab"] {
        assert!(
            free.iter().any(|(p, _)| p.starts_with(&format!("{m}/"))),
            "no sources read under {m}/"
        );
    }
    assert!(free.iter().all(|(p, _)| !p.starts_with("ui/")));
    assert!(all_sources().iter().any(|(p, _)| p == "ui/layout.rs"));
    assert!(all_sources().iter().any(|(p, _)| p == "app.rs"));
}

/// The drawing crates: egui and everything under it.
const DRAWING: [&str; 10] = [
    "egui",
    "eframe",
    "egui_plot",
    "egui_extras",
    "egui_tiles",
    "egui_kittest",
    "epaint",
    "emath",
    "ecolor",
    "winit",
];

/// Uses of a drawing crate, and reaches into this crate's drawing modules, `ui` and `app`: in a
/// path, in a `use` tree, groups and renames included, as a bare first segment, or through a
/// glob import of the crate, which brings them into scope.
fn drawing_uses(toks: &[Tok]) -> Vec<String> {
    let roots = use_roots(toks);
    let ours = with_aliases(toks, &roots, &CRATE_ROOTS);
    let ours: Vec<&str> = ours.iter().map(String::as_str).collect();
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if glob_from(toks, &roots, i, &ours) {
            found.push("crate::*".to_string());
        }
        let Tok::Ident(s) = t else { continue };
        if DRAWING.contains(&s.as_str()) || s.starts_with("egui") || s.starts_with("wgpu") {
            found.push(s.clone());
        }
        if (s == "ui" || s == "app") && reached(toks, &roots, i, &ours) {
            found.push(format!("::{s}"));
        }
    }
    found
}

#[test]
fn model_run_edit_vm_import_no_egui() {
    // U9 and D13: model, run, edit and vm, and the drivers and files beside them, name no egui
    // crate and reach nothing that draws. A panel draws a view-model; nothing else knows egui.
    // A reach into ui/ or app.rs is refused in every form: a path, a group or a rename in a
    // `use` tree, a glob import of the crate, and the bare path such a glob allows.
    let fx = fixture(
        "use egui::Ui; use crate::ui::layout::Pane; fn f(c: &eframe::Frame) {} \
         use super::super::app::GuiApp; use crate::run::Store; let x = egui_plot::Line::new(); \
         use crate::{run::Store, ui::layout::Pane}; #[allow(unused_imports)] use crate::{ui as _}; \
         use crate::*; fn g() -> Option<ui::layout::Pane> { None } use rustyecon_gui::{app}; \
         use super::{Driver, ThreadDriver}; let ui = 1; let x = ui + app;",
    );
    assert_eq!(
        drawing_uses(&fx),
        [
            "egui",
            "::ui",
            "eframe",
            "::app",
            "egui_plot",
            "::ui",
            "::ui",
            "crate::*",
            "::ui",
            "::app"
        ]
    );
    // O20, what G0.1's re-check found (fixed at G1): a reach into ui through a renamed crate
    // root, and a `use` group whose first root is another crate.
    let escapes = fixture(
        "use crate as g; fn h() -> g::ui::layout::Pane { todo!() } \
         use {std::fmt as _, crate::{ui as _}}; use {std::fmt, rustyecon_gui::app::GuiApp}; \
         use {std::fmt as _, crate::run as r}; let x = r::Store::default();",
    );
    assert_eq!(drawing_uses(&escapes), ["::ui", "::ui", "::app"]);
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&egui_free_sources()) {
        for u in drawing_uses(&toks) {
            found.push(format!("{path}: {u}"));
        }
    }
    assert!(
        found.is_empty(),
        "drawing in the egui-free modules: {found:#?}"
    );
    assert!(
        EGUI_FREE.contains(&"edit"),
        "edit/ is scanned once G0.2 adds it"
    );
}

/// The modules of this crate that run/ and vm/ may not reach: the model, the editor, the
/// drivers, the files and the drawing. They stay in crates/gui when run/ and vm/ move to
/// observe (D13).
const NOT_OBSERVE: [&str; 6] = ["model", "edit", "drive", "platform", "ui", "app"];
/// The modules edit/ may not reach: it reads run types, and nothing of the model, the
/// drivers, the files or the drawing.
const NOT_EDIT: [&str; 5] = ["model", "drive", "platform", "ui", "app"];
/// The modules lab/ may not reach (G1): it reads the oracle alone, and nothing of the runs, the
/// model, the editor, the drivers, the files, the view-models or the drawing.
const NOT_LAB: [&str; 8] = [
    "model", "run", "edit", "vm", "drive", "platform", "ui", "app",
];

/// Paths a pure module may not reach: the modules of this crate in `own`, and E2's list of
/// global state and I/O.
fn observe_violations(toks: &[Tok], own: &[&str]) -> Vec<String> {
    const STD: [&str; 7] = ["thread", "time", "fs", "io", "env", "process", "net"];
    const NAMES: [&str; 12] = [
        "Instant",
        "SystemTime",
        "Mutex",
        "RwLock",
        "OnceLock",
        "LazyLock",
        "thread_local",
        "println",
        "eprintln",
        "print",
        "eprint",
        "dbg",
    ];
    let roots = use_roots(toks);
    let ours = with_aliases(toks, &roots, &CRATE_ROOTS);
    let ours: Vec<&str> = ours.iter().map(String::as_str).collect();
    let stds = with_aliases(toks, &roots, &STD_ROOTS);
    let stds: Vec<&str> = stds.iter().map(String::as_str).collect();
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if glob_from(toks, &roots, i, &ours) {
            found.push("crate::*".to_string());
        }
        if glob_from(toks, &roots, i, &stds) {
            found.push("std::*".to_string());
        }
        let Tok::Ident(s) = t else { continue };
        if own.contains(&s.as_str()) && reached(toks, &roots, i, &ours) {
            found.push(format!("crate::{s}"));
        }
        if STD.contains(&s.as_str()) && reached(toks, &roots, i, &stds) {
            found.push(format!("std::{s}"));
        }
        if NAMES.contains(&s.as_str()) {
            found.push(s.clone());
        }
    }
    found
}

#[test]
fn run_and_vm_reach_no_model_file_thread_or_clock() {
    // D13: at G2 the Runner, the Extractor, the Store and the view-models move into
    // crates/observe, which has no egui, thread, clock, file or std::io (E2's list). So run/
    // and vm/ read the engine, certify and each other, and nothing of the model, the editor,
    // the drivers, the files or the drawing.
    // Every form of a reach is refused: a path, a group or a rename in a `use` tree, a glob
    // import, and the bare path a glob or a `use` allows.
    let fx = fixture(
        "use crate::model::Model; use std::time::Instant; use super::super::drive::Host; \
         std::fs::read(p); println!(\"x\"); use crate::run::Store; use std::fmt; \
         use std::{thread as _, fs as _}; use crate::{model as _}; use std::{fmt::Write, io}; \
         use crate::*; use std::*; let h = thread::spawn(f); let m: model::Model; \
         use super::{Obs, Refusal}; let fs = 2; let t = time + fs; use crate::edit::Lineage;",
    );
    assert_eq!(
        observe_violations(&fx, &NOT_OBSERVE),
        [
            "crate::model",
            "std::time",
            "Instant",
            "crate::drive",
            "std::fs",
            "println",
            "std::thread",
            "std::fs",
            "crate::model",
            "std::io",
            "crate::*",
            "std::*",
            "std::thread",
            "crate::model",
            "crate::edit"
        ]
    );
    // O20's two escapes, for the model and std: a renamed root and a group's later branch.
    let escapes = fixture(
        "use crate as g; let m: g::model::Model; use std as s; let t = s::time::Instant; \
         use {super::Obs, crate::{drive as _}}; use {core::fmt, std::{fs as _}};",
    );
    assert_eq!(
        observe_violations(&escapes, &NOT_OBSERVE),
        [
            "crate::model",
            "std::time",
            "Instant",
            "crate::drive",
            "std::fs"
        ]
    );
    let mut files = sources_under(&src().join("run"));
    files.extend(sources_under(&src().join("vm")));
    assert!(files.len() >= 9, "run/ and vm/ are read: {}", files.len());
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&files) {
        for v in observe_violations(&toks, &NOT_OBSERVE) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(found.is_empty(), "run/ and vm/ reach too far: {found:#?}");
}

#[test]
fn edit_reaches_no_model_file_thread_or_clock() {
    // docs/GUI.md §3.2: edit/ is text and tapes in, tapes and text out. It reads run types
    // (the store a plan scans, the series an export writes) and reaches nothing of the model,
    // the drivers, the files or the drawing, and no thread, clock or I/O: platform/ writes
    // what it makes, and the model hands it the date.
    let fx = fixture(
        "use crate::run::Store; use crate::model::Model; use crate::platform::write_new; \
         let t = std::time::SystemTime::now(); use super::keys; std::fs::write(p, s);",
    );
    assert_eq!(
        observe_violations(&fx, &NOT_EDIT),
        [
            "crate::model",
            "crate::platform",
            "std::time",
            "SystemTime",
            "std::fs"
        ]
    );
    let files = sources_under(&src().join("edit"));
    assert!(files.len() >= 6, "edit/ is read: {}", files.len());
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&files) {
        for v in observe_violations(&toks, &NOT_EDIT) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(found.is_empty(), "edit/ reaches too far: {found:#?}");
}

#[test]
fn lab_reaches_no_run_model_file_thread_or_clock() {
    // G1: the oracle lab's domain is parameters in and the oracle's numbers out. It reads the
    // oracle and the engine's `num`, and nothing of the runs, the model, the editor, the
    // drivers, the files, the view-models or the drawing, and no thread, clock or I/O, so it
    // moves with vm/ into crates/observe, which depends on the engine and the oracle (§3.2).
    let fx = fixture(
        "use oracle::Economy; use crate::run::Store; use crate::vm::lab; \
         use super::presets; std::fs::read_to_string(p); let t = Instant::now();",
    );
    assert_eq!(
        observe_violations(&fx, &NOT_LAB),
        ["crate::run", "crate::vm", "std::fs", "Instant"]
    );
    let files = sources_under(&src().join("lab"));
    assert!(files.len() >= 5, "lab/ is read: {}", files.len());
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&files) {
        for v in observe_violations(&toks, &NOT_LAB) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(found.is_empty(), "lab/ reaches too far: {found:#?}");
}

/// Trigonometric functions, and the constants that only serve them.
const TRIG: [&str; 16] = [
    "sin",
    "cos",
    "tan",
    "sin_cos",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sinh",
    "cosh",
    "tanh",
    "asinh",
    "acosh",
    "atanh",
    "to_radians",
    "to_degrees",
];

/// Trigonometry in `toks`: a trigonometric method or function called, or `PI`, `TAU` or a
/// `FRAC_PI_*` named.
fn trig_uses(toks: &[Tok]) -> Vec<String> {
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let Tok::Ident(s) = t else { continue };
        let called = toks.get(i + 1).is_some_and(|n| punct(n, '('));
        let method = i > 0 && punct(&toks[i - 1], '.');
        let path = i > 1 && punct(&toks[i - 1], ':') && punct(&toks[i - 2], ':');
        if TRIG.contains(&s.as_str()) && called && (method || path) {
            found.push(format!("{s}("));
        }
        if s == "PI" || s == "TAU" || s.starts_with("FRAC_PI") {
            found.push(s.clone());
        }
    }
    found
}

#[test]
fn no_trig_outside_ui() {
    // D12: drawing trigonometry is allowed only in ui/, under #[expect(clippy::disallowed_methods,
    // reason = "display only")], which clippy checks there. Everywhere else in the GUI it is
    // refused, app.rs and main.rs included, and core's num gains none.
    let fx = fixture(
        "let a = (t as f32).sin(); let b = f32::atan2(y, x); let c = std::f32::consts::TAU; \
         let d = x.sine(); let e = sin + 1.0; let f = v.sin_cos();",
    );
    assert_eq!(trig_uses(&fx), ["sin(", "atan2(", "TAU", "sin_cos("]);
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&all_sources()) {
        if path.starts_with("ui/") {
            continue;
        }
        for u in trig_uses(&toks) {
            found.push(format!("{path}: {u}"));
        }
    }
    assert!(found.is_empty(), "trigonometry outside ui/: {found:#?}");
    let num = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/src/num.rs");
    let text = std::fs::read_to_string(&num).expect("core's num.rs");
    let toks = tokens(&strip(&text));
    let defined: Vec<&str> = toks
        .windows(2)
        .filter_map(|w| match (&w[0], &w[1]) {
            (Tok::Ident(f), Tok::Ident(name)) if f == "fn" && TRIG.contains(&name.as_str()) => {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect();
    assert!(
        defined.is_empty(),
        "core::num defines trigonometry: {defined:?}"
    );
    assert!(
        toks.windows(2)
            .any(|w| ident(&w[0], "fn") && ident(&w[1], "exp")),
        "core's num.rs is read"
    );
}

/// Raw transcendentals in `toks` (the engine's `no_raw_transcendentals`, copied): an inherent
/// transcendental, `powi`, `powf` or `mul_add` called as a method or through `f64::`/`f32::`,
/// any `f32`, and any `libm`, since the GUI's egui-free modules call `core::num` or nothing.
fn raw_transcendentals(toks: &[Tok]) -> Vec<String> {
    const BANNED: [&str; 27] = [
        "exp", "exp2", "exp_m1", "ln", "ln_1p", "log", "log2", "log10", "powf", "powi", "cbrt",
        "hypot", "sin", "cos", "tan", "sin_cos", "asin", "acos", "atan", "atan2", "sinh", "cosh",
        "tanh", "asinh", "acosh", "atanh", "mul_add",
    ];
    let mut found = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        let next = toks.get(i + 1);
        let method = i > 0
            && punct(&toks[i - 1], '.')
            && BANNED.iter().any(|b| ident(t, b))
            && next.is_some_and(|n| punct(n, '('));
        let path_call = (ident(t, "f64") || ident(t, "f32"))
            && toks.get(i + 1).is_some_and(|n| punct(n, ':'))
            && toks
                .get(i + 3)
                .is_some_and(|n| BANNED.iter().any(|b| ident(n, b)));
        let f32 = ident(t, "f32");
        let libm = ident(t, "libm");
        if method || path_call || f32 || libm {
            found.push(format!("{t:?} {next:?}"));
        }
    }
    found
}

#[test]
fn no_raw_transcendentals() {
    // A5 and U9: transcendentals go through core::num, which calls libm; the platform maths
    // libraries differ in the last bit. Over the egui-free modules: no inherent
    // transcendental, powi, powf or mul_add, no f32, and no libm.
    let fx = fixture("let a = x.ln(); let b = f64::powi(x, 2); let c: f32 = 1.0; libm::exp(x);");
    assert_eq!(
        raw_transcendentals(&fx).len(),
        4,
        "{:?}",
        raw_transcendentals(&fx)
    );
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&egui_free_sources()) {
        for v in raw_transcendentals(&toks) {
            found.push(format!("{path}: {v}"));
        }
    }
    assert!(found.is_empty(), "raw transcendentals: {found:#?}");
}

/// Hashed containers and rayon in `toks` (the engine's `no_hashed_collections`, copied).
fn hashed(toks: &[Tok]) -> Vec<String> {
    let mut found = Vec::new();
    for t in toks {
        if let Tok::Ident(s) = t {
            let hit = s.contains("HashMap")
                || s.contains("HashSet")
                || ["hash_map", "hash_set", "RandomState", "rayon"].contains(&s.as_str());
            if hit {
                found.push(s.clone());
            }
        }
    }
    found
}

#[test]
fn no_hashed_collections() {
    // R8 and U9: no container that iterates in a per-process random order, and no rayon, in the
    // egui-free modules.
    let fx = fixture("use std::collections::HashMap; let s: AHashSet<u8>; use rayon::prelude;");
    assert_eq!(hashed(&fx), ["HashMap", "AHashSet", "rayon"]);
    let mut found = Vec::new();
    for (path, toks) in shipped_tokens(&egui_free_sources()) {
        for h in hashed(&toks) {
            found.push(format!("{path}: {h}"));
        }
    }
    assert!(found.is_empty(), "hashed collections: {found:#?}");
}

/// Every reach into the engine's `raw`, the tape's raw schema (G1.1): a path
/// `rustyecon_engine::raw…`, or `raw` in a `use` tree rooted at the engine, groups included.
fn raw_reaches(toks: &[Tok]) -> Vec<usize> {
    let roots = use_roots(toks);
    (0..toks.len())
        .filter(|&i| {
            ident(&toks[i], "raw")
                && (roots[i].as_deref() == Some("rustyecon_engine")
                    || path_from(toks, i, &["rustyecon_engine"]))
        })
        .collect()
}

/// Every path into core, and every reach into the engine's `raw` when `raw_allowed` is false:
/// core's writer (`apply`, `resolve`, the ledgers) and everything else of core is reached
/// through the engine, whose API holds no writer, and the raw schema is edit/'s alone.
fn core_paths(toks: &[Tok], raw_allowed: bool) -> Vec<String> {
    let show = |i: usize| {
        let next: Vec<String> = toks[i + 1..(i + 4).min(toks.len())]
            .iter()
            .map(|t| format!("{t:?}"))
            .collect();
        format!("{:?} {}", toks[i], next.join(" "))
    };
    let mut found: Vec<String> = (0..toks.len())
        .filter(|&i| ident(&toks[i], "rustyecon_core"))
        .map(show)
        .collect();
    if !raw_allowed {
        found.extend(raw_reaches(toks).into_iter().map(show));
    }
    found
}

#[test]
fn the_gui_reaches_core_through_the_engine_alone() {
    // U1, U9, E1: the GUI reaches the run through the engine, whose API holds no writer of the
    // state. Since G1.1 the engine re-exports `num`, the libm-backed maths a display takes
    // (U6), and the tape's raw schema with `Basis` and `Unit` in its prelude, so the GUI has no
    // edge to core: core's writer is out of its reach by type (the manifest names no core, and
    // cargo refuses a path into a crate that is not a dependency), and this scan says so of
    // every source file, a group or a rename at core's root included. The raw schema, the
    // plain data a new entry is written in, is named in edit/ alone (G0.2). Every source file
    // of the crate is read.
    let fx = fixture(
        "use rustyecon_core::num; let v = rustyecon_core::num::ln(x); use rustyecon_core::apply; \
         use rustyecon_core::{num, Ledger}; use rustyecon_core as core; \
         use rustyecon_engine::raw::RawEvent; use rustyecon_engine::num; \
         use rustyecon_engine::prelude::{Basis, Unit}; let r = rustyecon_engine::raw::RawAct; \
         use rustyecon_engine::rawness; use rustyecon_engine::{num, raw::RawParam}; \
         let raw = 1; use crate::run::raw;",
    );
    let anywhere = core_paths(&fx, true);
    assert_eq!(anywhere.len(), 5, "{anywhere:?}");
    let outside_edit = core_paths(&fx, false);
    assert_eq!(outside_edit.len(), 8, "{outside_edit:?}");
    let mut found = Vec::new();
    let mut raw_in_edit = 0;
    let mut num_named = 0;
    for (path, toks) in shipped_tokens(&all_sources()) {
        let in_edit = path.starts_with("edit/");
        if in_edit {
            raw_in_edit += raw_reaches(&toks).len();
        }
        num_named += toks
            .windows(4)
            .filter(|w| {
                ident(&w[0], "rustyecon_engine")
                    && punct(&w[1], ':')
                    && punct(&w[2], ':')
                    && ident(&w[3], "num")
            })
            .count();
        for p in core_paths(&toks, in_edit) {
            found.push(format!("{path}: {p}"));
        }
    }
    assert!(
        found.is_empty(),
        "paths into core, or raw outside edit/: {found:#?}"
    );
    assert!(
        num_named >= 2,
        "the inspector's ln(p′/p) names the engine's num"
    );
    assert!(raw_in_edit >= 1, "edit/ names the engine's raw schema");
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("the manifest");
    let deps: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .collect();
    assert!(
        !deps.iter().any(|l| l.starts_with("rustyecon-core")),
        "the manifest names core"
    );
    assert!(
        deps.iter()
            .any(|l| l.starts_with("rustyecon-engine.workspace = true")),
        "the manifest is read"
    );
}
