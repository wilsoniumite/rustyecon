//! PNG snapshots, never citable (docs/GUI.md §9, G1; U12): the file carries its mark in its
//! text chunks and the picture its banner, and no snapshot is written over another file.

use egui::{Color32, ColorImage};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use rustyecon_gui::app::{GuiApp, Launch};
use rustyecon_gui::platform::snapshot::{self, BANNER, NEVER_CITABLE};
use rustyecon_gui::platform::Files;
use rustyecon_gui::run::RunStatus;
use rustyecon_gui::ui::Snap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A PNG's size, its RGBA pixels and its tEXt chunks.
fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>, Vec<(String, String)>) {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes.to_vec()));
    let mut reader = decoder.read_info().expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size().expect("a size")];
    let frame = reader.next_frame(&mut buf).expect("a frame");
    buf.truncate(frame.buffer_size());
    let text = reader
        .info()
        .uncompressed_latin1_text
        .iter()
        .map(|t| (t.keyword.clone(), t.text.clone()))
        .collect();
    (frame.width, frame.height, buf, text)
}

#[test]
fn png_snapshots_say_never_citable() {
    // The encoder writes the pixels as given and the never-citable mark first among its text
    // chunks, then the marks it is handed; an image of the wrong size is refused.
    let rgba: Vec<u8> = (0..3 * 2 * 4).map(|i| (i * 7 % 256) as u8).collect();
    let marks = vec![
        ("Title", "rustyecon GUI snapshot".to_string()),
        ("Description", "run 0: gate · lab: unit 1a G1".to_string()),
    ];
    let bytes = snapshot::png(&rgba, 3, 2, &marks).expect("it encodes");
    let (w, h, pixels, text) = decode(&bytes);
    assert_eq!((w, h), (3, 2));
    assert_eq!(pixels, rgba);
    assert_eq!(text[0], ("Comment".to_string(), NEVER_CITABLE.to_string()));
    assert!(NEVER_CITABLE.starts_with("NEVER CITABLE"));
    assert_eq!(text[1], ("Title".to_string(), marks[0].1.clone()));
    assert_eq!(text[2].1, "run 0: gate · lab: unit 1a G1");
    assert!(snapshot::png(&rgba, 4, 2, &marks)
        .unwrap_err()
        .contains("24 bytes for a 4 × 2 RGBA image, which has 32"));
    // Written under snapshots/, each under a new name, never over a file.
    let dir = tempfile::tempdir().unwrap();
    let a = snapshot::write(dir.path(), "2026-09-27", &bytes).unwrap();
    let b = snapshot::write(dir.path(), "2026-09-27", b"second").unwrap();
    assert_eq!(
        a,
        dir.path()
            .join("snapshots/rustyecon-snapshot-2026-09-27-1.png")
    );
    assert_eq!(
        b,
        dir.path()
            .join("snapshots/rustyecon-snapshot-2026-09-27-2.png")
    );
    assert_eq!(std::fs::read(&a).unwrap(), bytes);
}

#[test]
fn the_snapshot_script_marks_the_picture_never_citable() {
    // The toolbar's Snapshot: the window paints the banner across its top and asks egui for a
    // picture of a later frame; when the picture comes (here it is handed in, since a headless
    // window has no renderer) it is written to snapshots/ beside the session, a PNG whose text
    // says it is never citable and names the run, and the banner goes.
    let dir = tempfile::tempdir().unwrap();
    let tape = format!("{}/../../tapes/gate.ron", env!("CARGO_MANIFEST_DIR"));
    let files = Files::at(dir.path().join("gui"));
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 1000.0))
        .build_eframe(move |cc| {
            GuiApp::new(
                &cc.egui_ctx,
                Launch {
                    tape: Some(tape),
                    files: Some(files),
                    smoke: None,
                },
            )
        });
    let t0 = Instant::now();
    while h.state().model().focused().map(|r| r.store.status())
        != Some(RunStatus::Paused { tick: 0, why: None })
    {
        h.step();
        assert!(
            t0.elapsed() < Duration::from_secs(60),
            "the tape did not load"
        );
    }
    h.get_by_label("Snapshot").click_accesskit();
    h.step();
    h.step();
    assert!(matches!(h.state().ui_state().snapshot, Snap::Waiting(_)));
    let painted = |h: &Harness<'_, GuiApp>| {
        fn walk(s: &egui::Shape, out: &mut Vec<String>) {
            match s {
                egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
                egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
                _ => {}
            }
        }
        let mut out = Vec::new();
        for c in &h.output().shapes {
            walk(&c.shape, &mut out);
        }
        out
    };
    assert!(
        painted(&h).iter().any(|t| t == BANNER),
        "the banner is painted"
    );
    assert!(
        painted(&h)
            .iter()
            .any(|t| t.contains("run 0: gate, origin run")),
        "the banner names the run"
    );
    // The renderer's picture of the frame.
    let image = ColorImage::new([4, 3], vec![Color32::from_rgb(10, 20, 30); 12]);
    h.event(egui::Event::Screenshot {
        viewport_id: egui::ViewportId::ROOT,
        user_data: egui::UserData::default(),
        image: Arc::new(image),
    });
    h.step();
    h.step();
    assert_eq!(h.state().ui_state().snapshot, Snap::Idle);
    assert!(!painted(&h).iter().any(|t| t == BANNER), "the banner goes");
    let shots: Vec<_> = std::fs::read_dir(dir.path().join("gui/snapshots"))
        .expect("snapshots/ is made")
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(shots.len(), 1, "{shots:?}");
    let (w, hgt, pixels, text) = decode(&std::fs::read(&shots[0]).unwrap());
    assert_eq!((w, hgt), (4, 3));
    assert_eq!(&pixels[..4], &[10, 20, 30, 255]);
    let get = |k: &str| {
        text.iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    assert_eq!(get("Comment"), NEVER_CITABLE);
    assert!(get("Description").starts_with("run 0: gate, origin run, tape_hash 0x"));
    assert!(get("Software").starts_with("rustyecon-gui, build "));
    let log = h.state().model().log();
    assert!(log
        .iter()
        .any(|l| l.text.starts_with("snapshot saved as ") && l.text.contains("never citable")));
    // A picture nobody asked for is not written.
    h.event(egui::Event::Screenshot {
        viewport_id: egui::ViewportId::ROOT,
        user_data: egui::UserData::default(),
        image: Arc::new(ColorImage::filled([2, 2], Color32::WHITE)),
    });
    h.step();
    let n = std::fs::read_dir(dir.path().join("gui/snapshots"))
        .unwrap()
        .count();
    assert_eq!(n, 1);
}
