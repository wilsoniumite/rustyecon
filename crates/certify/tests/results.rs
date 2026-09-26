//! The committed results (docs/CERTIFY.md §8, §13, §14; C3, C10): `results/<tape>/certificate.ron`
//! and `manifest.ron` for the gate world and appb, made by a clean build from the criteria
//! registered in `criteria/` before any certified run of either tape. They are verdicts: a rerun
//! is a diff, and a FAIL stays a FAIL until a new dated criteria file says why.

use certify::{certify, tape_hash, Certificate, Criteria, Hex, Manifest, Scoring};
use rustyecon_engine::prelude::{Sim, Tape};
use std::path::{Path, PathBuf};

/// The certified tapes.
const TAPES: [&str; 2] = ["gate", "appb"];
/// The date their criteria were registered.
const REGISTERED: &str = "2026-09-26";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// One tape's committed inputs and results.
struct Committed {
    tape: Tape,
    criteria: Criteria,
    file: String,
    certificate: Certificate,
    certificate_text: String,
    manifest: Manifest,
    manifest_text: String,
}

fn committed(name: &str) -> Committed {
    let r = root();
    let tape = Tape::from_ron(&read(&r.join("tapes").join(format!("{name}.ron"))))
        .unwrap_or_else(|e| panic!("tapes/{name}.ron: {e}"));
    let file = format!("{name}-{REGISTERED}.ron");
    let criteria = Criteria::from_ron(&read(&r.join("criteria").join(&file)), &file)
        .unwrap_or_else(|e| panic!("criteria/{file}: {e}"));
    let dir = r.join("results").join(name);
    let certificate_text = read(&dir.join("certificate.ron"));
    let certificate = Certificate::from_ron(&certificate_text)
        .unwrap_or_else(|e| panic!("results/{name}/certificate.ron: {e}"));
    let manifest_text = read(&dir.join("manifest.ron"));
    let manifest = Manifest::from_ron(&manifest_text)
        .unwrap_or_else(|e| panic!("results/{name}/manifest.ron: {e}"));
    Committed {
        tape,
        criteria,
        file,
        certificate,
        certificate_text,
        manifest,
        manifest_text,
    }
}

#[test]
fn registered_criteria_load_and_fit() {
    // C3 and N10: each registered criteria file loads, fits its tape's clock, and names the tape
    // by its current tape_hash, so an edit to a certified tape without a new dated criteria file
    // fails here, not only in the ignored recompute.
    for name in TAPES {
        let c = committed(name);
        assert_eq!(c.criteria.tape.name, name);
        assert_eq!(c.criteria.date.to_string(), REGISTERED);
        assert_eq!(
            c.criteria.tape.tape_hash,
            Hex(tape_hash(&c.tape)),
            "tapes/{name}.ron changed since criteria/{} was registered",
            c.file
        );
        let sim = Sim::new(&c.tape).expect("the tape loads");
        c.criteria
            .fit(&sim.world().clock)
            .unwrap_or_else(|e| panic!("criteria/{} does not fit: {e}", c.file));
    }
}

#[test]
fn committed_certificates_name_a_clean_build() {
    // C3 and C10: each committed certificate reads back through the seal (its verdict, failures
    // and non-finite list recomputed), names a clean build by a 40-hex commit (gate.sh checks
    // that the commit is an ancestor of HEAD), its registered criteria by file, date and hash,
    // and its tape by tape_hash; and it agrees with its manifest: the same run key and tape, the
    // run's full hash stream ending at the certificate's final hash, and no telemetry or
    // checkpoint, since committed results are made without them.
    for name in TAPES {
        let c = committed(name);
        let cert = &c.certificate;
        let build = &cert.run().build;
        assert!(!build.dirty, "{name}: {build:?}");
        assert_eq!(build.commit.len(), 40, "{name}: {build:?}");
        assert!(
            build
                .commit
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "{name}: {build:?}"
        );
        assert_eq!(cert.tape(), name);
        assert_eq!(cert.run().tape_hash, Hex(tape_hash(&c.tape)));
        let cref = cert.criteria().expect("registered criteria");
        assert_eq!(cref.file, c.file);
        assert_eq!(cref.date, c.criteria.date);
        assert_eq!(cref.hash, Hex(c.criteria.hash()));
        assert_eq!(cref.tape_hash, c.criteria.tape.tape_hash);
        assert_eq!(cref.listed, c.criteria.listed());
        let m = &c.manifest;
        assert_eq!(&m.run, cert.run());
        assert_eq!(m.tape, name);
        assert_eq!(m.genesis_hash, cert.genesis_hash());
        assert_eq!((m.from, m.until), (0, cert.reached()));
        assert_eq!(m.hashes.count, cert.reached());
        assert_eq!(m.hashes.last, Some((cert.reached(), cert.final_hash())));
        assert_eq!(m.resumed_from, None);
        assert!(m.checkpoints.is_empty(), "{name}");
        assert_eq!(m.telemetry, None, "{name}");
        // The files are the serialiser's own text, so a rerun can compare them byte for byte.
        assert_eq!(cert.to_ron(), c.certificate_text, "{name}");
        assert_eq!(m.to_ron(), c.manifest_text, "{name}");
    }
}

#[test]
#[ignore = "reruns both certified runs, about 10 s in release: gate.sh runs it by name (C10)"]
fn committed_certificates_recompute() {
    // C10: rerun certify on each tape with its registered criteria and the committed build. On
    // every machine the verdict and each battery's pass flag must be the committed ones. On
    // Linux (WSL, the primary machine, and CI) the certificate and the manifest must equal the
    // committed files byte for byte; on Windows that equality is recorded, not gated
    // (decision 2).
    for name in TAPES {
        let c = committed(name);
        let got = certify(
            &c.tape,
            Scoring::Criteria {
                criteria: &c.criteria,
                file: &c.file,
            },
            &c.certificate.run().build,
            &mut |_| {},
        )
        .unwrap_or_else(|e| panic!("{name}: certify: {e}"));
        let cert = &got.certificate;
        assert_eq!(cert.verdict(), c.certificate.verdict(), "{name}");
        let flags = |x: &Certificate| {
            x.batteries()
                .iter()
                .map(|b| (b.id, b.pass))
                .collect::<Vec<_>>()
        };
        assert_eq!(flags(cert), flags(&c.certificate), "{name}");
        let same_certificate = cert.to_ron() == c.certificate_text;
        let same_manifest = got.manifest.to_ron() == c.manifest_text;
        println!(
            "{name}: verdict {}; certificate {}; manifest {}",
            cert.verdict(),
            if same_certificate {
                "byte-equal"
            } else {
                "DIFFERS"
            },
            if same_manifest {
                "byte-equal"
            } else {
                "DIFFERS"
            },
        );
        if cfg!(windows) {
            println!("{name}: recorded on Windows, not gated (C10, decision 2)");
        } else {
            assert_eq!(cert.to_ron(), c.certificate_text, "{name}: certificate");
            assert_eq!(got.manifest.to_ron(), c.manifest_text, "{name}: manifest");
        }
    }
}
