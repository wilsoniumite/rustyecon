//! The atlas loader (crates/worldgen/src/atlas.rs; data/atlas/README.md). The bundled atlas
//! loads with its recorded digest, every region has geometry, the keys are D7's, the adjacency
//! is symmetric and agrees with the stored borders, and each rule the loader enforces refuses a
//! small atlas that breaks it.

use rustyecon_core::fnv1a_64;
use rustyecon_worldgen::atlas::{Atlas, AtlasError, Country, GB_ATLAS, GB_ATLAS_FNV, KEY_PREFIX};
use std::collections::{BTreeMap, BTreeSet};

fn gb() -> Atlas {
    match Atlas::gb() {
        Ok(a) => a,
        Err(e) => panic!("the bundled atlas does not load: {e}"),
    }
}

#[test]
fn bundled_atlas_digest_is_recorded() {
    // A rebuilt atlas needs its new digest in GB_ATLAS_FNV and data/atlas/README.md.
    assert_eq!(fnv1a_64(GB_ATLAS.as_bytes()), GB_ATLAS_FNV);
    assert_eq!(gb().digest, GB_ATLAS_FNV);
    // LF only (.gitattributes), so every checkout has the same bytes and digest.
    assert!(!GB_ATLAS.contains('\r'));
    assert_eq!(
        gb().basis_source(),
        format!(
            "data/atlas {GB_ATLAS_FNV:016x} (ODbL; OpenStreetMap contributors; Historic Counties \
             Trust)"
        )
    );
}

#[test]
fn every_region_has_geometry() {
    let a = gb();
    let mut worst = (0.0_f64, String::new());
    for r in &a.regions {
        assert!(!r.parts.is_empty(), "{} has no parts", r.key);
        for p in &r.parts {
            assert!(!p.rings.is_empty() && p.rings.iter().all(|ring| ring.len() >= 4));
            for ring in &p.rings {
                for &[x, y] in ring {
                    // The grid's extent, with the six counties of Northern Ireland at its west.
                    assert!((-20_000.0..700_000.0).contains(&x), "{} x {x}", r.key);
                    assert!((0.0..1_300_000.0).contains(&y), "{} y {y}", r.key);
                }
            }
        }
        assert!(r.contains(r.label), "{}'s label point is outside it", r.key);
        let drawn = r.drawn_area_km2();
        let rel = (drawn / r.area_km2 - 1.0).abs();
        if rel > worst.0 {
            worst = (rel, r.key.clone());
        }
    }
    // Simplification at 100 m and the islands under 0.02 km2 it drops move no region's drawn
    // area far from its measured one: the build measured at most 0.17% (Shetland).
    assert!(
        worst.0 < 0.005,
        "drawn area off by {:.2}% in {}",
        worst.0 * 100.0,
        worst.1
    );
}

#[test]
fn keys_are_unique_lower_case_and_chapman() {
    let a = gb();
    let mut seen = BTreeSet::new();
    for (i, r) in a.regions.iter().enumerate() {
        assert!(seen.insert(r.key.as_str()), "{} twice", r.key);
        assert_eq!(r.key, r.key.to_lowercase());
        assert_eq!(r.key, format!("{KEY_PREFIX}{}", r.chapman.to_lowercase()));
        assert_eq!(a.index(&r.key), Some(i));
    }
    assert_eq!(a.index("county.yks"), None);
    assert_eq!(a.index("county.BDF"), None);
}

#[test]
fn adjacency_is_symmetric() {
    let a = gb();
    let mut pairs = BTreeMap::new();
    for (i, r) in a.regions.iter().enumerate() {
        for n in &r.neighbours {
            assert_ne!(n.region, i);
            assert!(pairs.insert((i, n.region), n.border_km.to_bits()).is_none());
        }
    }
    for (&(i, j), km) in &pairs {
        assert_eq!(
            pairs.get(&(j, i)),
            Some(km),
            "{i} -> {j} has no equal way back"
        );
    }
    assert_eq!(pairs.len(), 2 * 201);
}

#[test]
fn stored_borders_agree_with_the_adjacency() {
    let a = gb();
    let drawn: BTreeSet<(usize, usize)> = a
        .borders
        .iter()
        .filter_map(|b| b.right.map(|r| (b.left.min(r), b.left.max(r))))
        .collect();
    let measured: BTreeSet<(usize, usize)> = a
        .regions
        .iter()
        .enumerate()
        .flat_map(|(i, r)| {
            r.neighbours
                .iter()
                .map(move |n| (i.min(n.region), i.max(n.region)))
        })
        .collect();
    // At 100 m every measured border survives simplification as at least one arc.
    assert_eq!(drawn, measured);
    // An inland region has no arc to the sea; a coastal one has.
    for (i, r) in a.regions.iter().enumerate() {
        let sea = a.borders.iter().any(|b| b.left == i && b.right.is_none());
        if !r.coastal {
            assert!(r.sea_km == 0.0, "{} is inland with sea {}", r.key, r.sea_km);
        }
        if r.coastal {
            assert!(sea, "{} is coastal with no arc to the sea", r.key);
        }
    }
}

#[test]
fn the_regions_are_the_uks_historic_counties_in_ridings() {
    let a = gb();
    let mut by_country: BTreeMap<Country, usize> = BTreeMap::new();
    for r in &a.regions {
        *by_country.entry(r.country).or_default() += 1;
    }
    let want = [
        (Country::England, 41),
        (Country::Wales, 13),
        (Country::Scotland, 33),
        (Country::NorthernIreland, 6),
    ];
    assert_eq!(by_country, want.into_iter().collect());
    for k in ["county.ery", "county.nry", "county.wry"] {
        let r = a.region(k).unwrap_or_else(|| panic!("{k} missing"));
        assert_eq!(r.hcs, ["YRK"]);
    }
    let roc = a.region("county.roc").expect("Ross and Cromarty");
    assert_eq!(roc.hcs, ["CRT", "RSS"]);
    // The ridings meet, and the West Riding borders Lancashire.
    let (e, n, w) = (
        a.index("county.ery"),
        a.index("county.nry"),
        a.index("county.wry"),
    );
    let lan = a.index("county.lan");
    let touches = |i: Option<usize>, j: Option<usize>| {
        let (Some(i), Some(j)) = (i, j) else {
            return false;
        };
        a.regions[i].neighbours.iter().any(|x| x.region == j)
    };
    assert!(touches(e, n) && touches(e, w) && touches(n, w) && touches(w, lan));
    // Sites back the port flag.
    let ports: BTreeSet<usize> = a.sites.iter().map(|s| s.region).collect();
    for (i, r) in a.regions.iter().enumerate() {
        assert_eq!(r.port, ports.contains(&i), "{}", r.key);
    }
}

#[test]
fn the_atlas_is_inside_its_budget() {
    let a = gb();
    assert!(GB_ATLAS.len() <= 1_500_000, "{} bytes", GB_ATLAS.len());
    assert!(
        a.ring_points() <= 200_000,
        "{} ring vertices",
        a.ring_points()
    );
    assert!(a.stored_points() <= a.ring_points());
    assert_eq!((a.quantum_m, a.tolerance_m), (1, 100));
    assert_eq!(a.sources.len(), 2);
    for s in &a.sources {
        assert_eq!(s.sha256.len(), 64);
        assert!(s.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
    }
}

// Refusals. Two 10 m squares side by side: arc 0 is their shared edge, arcs 1 and 2 their coasts.

const TWO: &str = r#"(
    schema: 1,
    crs: "EPSG:27700",
    quantum_m: 1,
    tolerance_m: 100,
    sources: [],
    regions: [
        (key: "county.aaa", chapman: "AAA", hcs: ["AAA"], hcs_number: [1], name: "A", country: England, area_km2: 0.0001, centroid: (5, 5), label: (5, 5), sea_km: 0.0, coastal: false, port: false, neighbours: [("county.bbb", 0.01)]),
        (key: "county.bbb", chapman: "BBB", hcs: ["BBB"], hcs_number: [2], name: "B", country: Wales, area_km2: 0.0001, centroid: (15, 5), label: (15, 5), sea_km: 0.0, coastal: false, port: false, neighbours: [("county.aaa", 0.01)]),
    ],
    sites: [],
    shapes: [
        ("county.aaa", [[[0, 1]]]),
        ("county.bbb", [[[2, -1]]]),
    ],
    arcs: [
        (0, Some(1), [10,0,0,10]),
        (0, None, [10,10,-10,0,0,-10,10,0]),
        (1, None, [10,0,10,0,0,10,-10,0]),
    ],
)"#;

fn refused(text: &str, why: &str) {
    match Atlas::parse(text) {
        Ok(_) => panic!("accepted an atlas that should fail with {why:?}"),
        Err(AtlasError::Invalid { why: got, .. }) => {
            assert!(got.contains(why), "refused with {got:?}, not {why:?}");
        }
        Err(e) => panic!("refused with {e}, not {why:?}"),
    }
}

fn edit(from: &str, to: &str) -> String {
    assert!(TWO.contains(from), "the fixture has no {from:?}");
    TWO.replacen(from, to, 1)
}

#[test]
fn the_fixture_loads() {
    let a = match Atlas::parse(TWO) {
        Ok(a) => a,
        Err(e) => panic!("{e}"),
    };
    assert_eq!(a.digest, fnv1a_64(TWO.as_bytes()));
    assert_eq!(a.regions[0].parts[0].rings[0].len(), 5);
    assert!((a.regions[1].drawn_area_km2() - 0.0001).abs() < 1e-12);
    assert!(a.regions[0].contains([5.0, 5.0]) && !a.regions[0].contains([15.0, 5.0]));
    assert_eq!((a.stored_points(), a.ring_points()), (10, 8));
}

#[test]
fn keys_are_refused_unless_unique_lower_case_and_chapman() {
    refused(
        &edit(
            r#"key: "county.bbb", chapman: "BBB""#,
            r#"key: "county.aaa", chapman: "AAA""#,
        ),
        "appears twice",
    );
    refused(
        &edit(r#"key: "county.aaa""#, r#"key: "county.AAA""#),
        "lower-case",
    );
    refused(
        &edit(r#"key: "county.aaa""#, r#"key: "shire.aaa""#),
        "starts with",
    );
    refused(
        &edit(r#"key: "county.aaa""#, r#"key: "county.aab""#),
        "Chapman code>",
    );
}

#[test]
fn asymmetric_adjacency_is_refused() {
    refused(
        &edit(r#"("county.aaa", 0.01)"#, r#"("county.aaa", 0.02)"#),
        "not back",
    );
    refused(
        &edit(r#"neighbours: [("county.aaa", 0.01)]"#, "neighbours: []"),
        "not back",
    );
    refused(
        &edit(r#"("county.bbb", 0.01)"#, r#"("county.ccc", 0.01)"#),
        "not a region",
    );
}

#[test]
fn broken_geometry_is_refused() {
    refused(&edit("[[[0, 1]]]", "[]"), "no geometry");
    refused(&edit("[[[0, 1]]]", "[[[0, 5]]]"), "out of range");
    refused(&edit("[[[0, 1]]]", "[[[-1, 1]]]"), "not its side");
    refused(&edit("[[[0, 1]]]", "[[[0]]]"), "does not close");
    refused(&edit("[[[0, 1]]]", "[[[0, 1]], [[0, 1]]]"), "used once");
    refused(
        &edit("(1, None, [10,0,", "(1, Some(1), [10,0,"),
        "two different regions",
    );
}

#[test]
fn inconsistent_flags_are_refused() {
    refused(
        &edit(
            "coastal: false, port: false, neighbours: [(\"county.bbb\"",
            "coastal: true, port: false, neighbours: [(\"county.bbb\"",
        ),
        "coastal",
    );
    refused(
        &edit(
            "port: false, neighbours: [(\"county.bbb\"",
            "port: true, neighbours: [(\"county.bbb\"",
        ),
        "port site",
    );
    assert!(matches!(
        Atlas::parse("(schema: 1)"),
        Err(AtlasError::Parse(_))
    ));
    refused(&edit("schema: 1", "schema: 2"), "schema 2");
}
