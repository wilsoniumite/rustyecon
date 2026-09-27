//! The county atlas (docs/GUI.md §6, ruling 3; data/atlas/README.md), 2026-09-27.
//!
//! `data/atlas/gb.atlas.ron` holds the historic counties of the United Kingdom as 93 regions:
//! Yorkshire in its three ridings, Ross-shire and Cromartyshire as Ross and Cromarty. Each region
//! has its codes, measures (area, sea frontage, shared-border lengths) and shape; the shapes are
//! built from arcs, each border stored once with the region on each side, so that neighbours
//! share their borders exactly. Coordinates are British National Grid (EPSG:27700) metres.
//!
//! [`Atlas::parse`] reads the text and checks it whole: the keys (D7: `county.<chapman>`, lower
//! case, unique), every reference, every ring's closure and orientation, each arc used once from
//! each side, and the adjacency's symmetry. It does no file I/O, no trigonometry (D12) and uses
//! no hashed container (R8), so the GUI and the tape compiler can both call it.
//!
//! The atlas is a database under ODbL 1.0 (data/atlas/LICENSE and ATTRIBUTION); the code is not.
//! A number taken from it into a tape carries the basis that [`Atlas::basis_source`] names
//! (docs/GUI.md §6).

use rustyecon_core::fnv1a_64;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The bundled atlas, `data/atlas/gb.atlas.ron`.
pub const GB_ATLAS: &str = include_str!("../../../data/atlas/gb.atlas.ron");

/// FNV-1a 64 (core's [`fnv1a_64`]) of [`GB_ATLAS`]'s bytes, recorded when the atlas was built.
/// data/atlas/README.md records the file's SHA-256 beside it. [`Atlas::gb`] refuses a file
/// whose digest differs, so a rebuilt atlas is used only once its digest is recorded here.
pub const GB_ATLAS_FNV: u64 = 0xec2e_e410_0a48_3601;

/// The atlas schema this loader reads.
pub const SCHEMA: u32 = 1;

/// The one coordinate system: British National Grid, metres.
pub const CRS: &str = "EPSG:27700";

/// Every region key starts with this (D7).
pub const KEY_PREFIX: &str = "county.";

/// A point in British National Grid metres: `[easting, northing]`.
pub type Point = [f64; 2];

/// The country a region lies in. Monmouthshire is listed with Wales, as Chapman lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Country {
    England,
    Wales,
    Scotland,
    NorthernIreland,
}

/// What a site is. Only ports so far; coalfields and other sites join as new variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum SiteKind {
    Port,
}

/// A pinned source the atlas was built from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub key: String,
    pub title: String,
    pub publisher: String,
    pub release: String,
    pub url: String,
    pub sha256: String,
    pub licence: String,
}

/// One region: a historic county, or a riding of Yorkshire.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// `county.<chapman>`, lower case (D7).
    pub key: String,
    /// The Chapman code, upper case.
    pub chapman: String,
    /// The Historic Counties Standard codes and numbers the region covers: one, or two for Ross
    /// and Cromarty; each riding names Yorkshire's.
    pub hcs: Vec<String>,
    pub hcs_number: Vec<u16>,
    pub name: String,
    pub country: Country,
    /// Planar area in the grid, on the unsimplified coverage.
    pub area_km2: f64,
    /// The area centroid, which can lie outside the region.
    pub centroid: Point,
    /// A label point inside the region's largest part (its pole of inaccessibility).
    pub label: Point,
    /// Border length with the open sea, on the unsimplified coverage.
    pub sea_km: f64,
    /// `sea_km` is 1 km or more.
    pub coastal: bool,
    /// The region holds a port site.
    pub port: bool,
    /// Regions sharing a border line with this one, by index, in key order.
    pub neighbours: Vec<Neighbour>,
    /// The simplified shape: parts (including detached parts and islands), each an exterior
    /// ring then its holes.
    pub parts: Vec<Part>,
}

/// A shared border, measured on the unsimplified coverage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Neighbour {
    /// Index into [`Atlas::regions`].
    pub region: usize,
    pub border_km: f64,
}

/// One polygon of a region. `rings[0]` is the exterior, anticlockwise; the rest are holes,
/// clockwise. Every ring is closed: its last point equals its first.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub rings: Vec<Vec<Point>>,
}

/// One arc of border, stored once. `left` lies on its left as stored; `right` is the region on
/// the other side, or `None` where that side is sea or unmapped water.
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    pub left: usize,
    pub right: Option<usize>,
    pub points: Vec<Point>,
}

/// A named place in a region.
#[derive(Debug, Clone, PartialEq)]
pub struct Site {
    pub key: String,
    pub kind: SiteKind,
    pub name: String,
    /// Index into [`Atlas::regions`].
    pub region: usize,
    pub at: Point,
}

/// A checked atlas.
#[derive(Debug, Clone, PartialEq)]
pub struct Atlas {
    pub schema: u32,
    pub crs: String,
    /// The grid the geometry was snapped to, metres.
    pub quantum_m: u32,
    /// The topology-preserving simplification's tolerance, metres.
    pub tolerance_m: u32,
    pub sources: Vec<Source>,
    /// In key order.
    pub regions: Vec<Region>,
    pub borders: Vec<Border>,
    pub sites: Vec<Site>,
    /// FNV-1a 64 of the text parsed.
    pub digest: u64,
}

/// Why an atlas was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtlasError {
    /// The text is not the schema's RON.
    Parse(String),
    /// The text parsed, and `at` breaks a rule.
    Invalid { at: String, why: String },
    /// The bundled file's digest is not the one recorded in [`GB_ATLAS_FNV`].
    Digest { found: u64, recorded: u64 },
}

impl fmt::Display for AtlasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtlasError::Parse(e) => write!(f, "atlas: not the schema's RON: {e}"),
            AtlasError::Invalid { at, why } => write!(f, "atlas: {at}: {why}"),
            AtlasError::Digest { found, recorded } => write!(
                f,
                "atlas: digest {found:#018x} is not the recorded {recorded:#018x}"
            ),
        }
    }
}

impl std::error::Error for AtlasError {}

fn invalid(at: impl Into<String>, why: impl Into<String>) -> AtlasError {
    AtlasError::Invalid {
        at: at.into(),
        why: why.into(),
    }
}

// The file as written by data/atlas/build_atlas.py.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAtlas {
    schema: u32,
    crs: String,
    quantum_m: u32,
    tolerance_m: u32,
    sources: Vec<Source>,
    regions: Vec<RawRegion>,
    sites: Vec<RawSite>,
    /// Per region, in the same order: its key, then parts of rings of signed arc references.
    shapes: Vec<(String, Vec<Vec<Vec<i64>>>)>,
    /// `(left, right, points)`: the first pair absolute, each later pair a difference, in units
    /// of `quantum_m`.
    arcs: Vec<(usize, Option<usize>, Vec<i64>)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegion {
    key: String,
    chapman: String,
    hcs: Vec<String>,
    hcs_number: Vec<u16>,
    name: String,
    country: Country,
    area_km2: f64,
    centroid: (i64, i64),
    label: (i64, i64),
    sea_km: f64,
    coastal: bool,
    port: bool,
    neighbours: Vec<(String, f64)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSite {
    key: String,
    kind: SiteKind,
    name: String,
    region: String,
    at: (i64, i64),
}

impl Atlas {
    /// The bundled atlas, refused unless its digest is the recorded one.
    pub fn gb() -> Result<Atlas, AtlasError> {
        let found = fnv1a_64(GB_ATLAS.as_bytes());
        if found != GB_ATLAS_FNV {
            return Err(AtlasError::Digest {
                found,
                recorded: GB_ATLAS_FNV,
            });
        }
        Atlas::parse(GB_ATLAS)
    }

    /// Parse and check an atlas.
    pub fn parse(text: &str) -> Result<Atlas, AtlasError> {
        let raw: RawAtlas = ron::from_str(text).map_err(|e| AtlasError::Parse(e.to_string()))?;
        check(raw, fnv1a_64(text.as_bytes()))
    }

    /// The index of the region with this key.
    pub fn index(&self, key: &str) -> Option<usize> {
        self.regions
            .binary_search_by(|r| r.key.as_str().cmp(key))
            .ok()
    }

    /// The region with this key.
    pub fn region(&self, key: &str) -> Option<&Region> {
        self.index(key).map(|i| &self.regions[i])
    }

    /// Points stored in the borders, each shared border once.
    pub fn stored_points(&self) -> usize {
        self.borders.iter().map(|b| b.points.len()).sum()
    }

    /// Vertices of every ring, a closing point not counted, shared borders once per side.
    pub fn ring_points(&self) -> usize {
        self.regions
            .iter()
            .flat_map(|r| &r.parts)
            .flat_map(|p| &p.rings)
            .map(|ring| ring.len() - 1)
            .sum()
    }

    /// The `source` of a `Measured` basis for a number taken from this atlas (docs/GUI.md §6).
    pub fn basis_source(&self) -> String {
        format!(
            "data/atlas {:016x} (ODbL; OpenStreetMap contributors; Historic Counties Trust)",
            self.digest
        )
    }
}

impl Region {
    /// The simplified shape's area, square kilometres.
    pub fn drawn_area_km2(&self) -> f64 {
        let m2: f64 = self
            .parts
            .iter()
            .flat_map(|p| &p.rings)
            .map(|r| signed_area(r))
            .sum();
        m2 / 1e6
    }

    /// Whether `p` lies inside the simplified shape (even-odd over every ring).
    pub fn contains(&self, p: Point) -> bool {
        let mut inside = false;
        for ring in self.parts.iter().flat_map(|q| &q.rings) {
            for w in ring.windows(2) {
                let ([x0, y0], [x1, y1]) = (w[0], w[1]);
                if (y0 > p[1]) != (y1 > p[1]) && p[0] < x0 + (p[1] - y0) * (x1 - x0) / (y1 - y0) {
                    inside = !inside;
                }
            }
        }
        inside
    }
}

/// The shoelace area of a closed ring, square metres, positive when anticlockwise.
fn signed_area(ring: &[Point]) -> f64 {
    let [ox, oy] = ring[0];
    let mut s = 0.0;
    for w in ring.windows(2) {
        let (a, b) = (w[0], w[1]);
        s += (a[0] - ox) * (b[1] - oy) - (b[0] - ox) * (a[1] - oy);
    }
    s / 2.0
}

/// Grid units to metres. Atlas coordinates are far below 2^53, so f64 holds them exactly.
fn point(q: f64, (x, y): (i64, i64)) -> Point {
    [x as f64 * q, y as f64 * q]
}

fn check_key(key: &str, chapman: &str) -> Result<(), AtlasError> {
    let Some(code) = key.strip_prefix(KEY_PREFIX) else {
        return Err(invalid(
            key,
            format!("a region key starts with {KEY_PREFIX}"),
        ));
    };
    if code.is_empty()
        || !code
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    {
        return Err(invalid(
            key,
            "a region key's code is lower-case letters and digits",
        ));
    }
    if chapman.len() != 3 || !chapman.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(invalid(key, "a Chapman code is three upper-case letters"));
    }
    if code != chapman.to_ascii_lowercase() {
        return Err(invalid(key, "a region key is county.<its Chapman code>"));
    }
    Ok(())
}

fn finite(at: &str, what: &str, v: f64, min: f64) -> Result<(), AtlasError> {
    if v.is_finite() && v >= min {
        Ok(())
    } else {
        Err(invalid(
            at,
            format!("{what} {v} is not a finite number >= {min}"),
        ))
    }
}

fn decode_arc(i: usize, q: f64, flat: &[i64]) -> Result<Vec<Point>, AtlasError> {
    if flat.len() < 4 || !flat.len().is_multiple_of(2) {
        return Err(invalid(
            format!("arc {i}"),
            "an arc has two or more points, as x, y pairs",
        ));
    }
    let (mut x, mut y) = (flat[0], flat[1]);
    let mut out = vec![point(q, (x, y))];
    for d in flat[2..].chunks_exact(2) {
        match (x.checked_add(d[0]), y.checked_add(d[1])) {
            (Some(nx), Some(ny)) if nx.abs() < 1 << 31 && ny.abs() < 1 << 31 => {
                (x, y) = (nx, ny);
            }
            _ => return Err(invalid(format!("arc {i}"), "a coordinate is out of range")),
        }
        out.push(point(q, (x, y)));
    }
    Ok(out)
}

fn check(raw: RawAtlas, digest: u64) -> Result<Atlas, AtlasError> {
    if raw.schema != SCHEMA {
        return Err(invalid(
            "schema",
            format!("schema {} is not {SCHEMA}", raw.schema),
        ));
    }
    if raw.crs != CRS {
        return Err(invalid("crs", format!("{} is not {CRS}", raw.crs)));
    }
    if raw.quantum_m == 0 || raw.tolerance_m == 0 {
        return Err(invalid("header", "quantum_m and tolerance_m are positive"));
    }
    let q = f64::from(raw.quantum_m);
    let n = raw.regions.len();

    // Keys: well formed, unique, in order.
    let mut index: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, r) in raw.regions.iter().enumerate() {
        check_key(&r.key, &r.chapman)?;
        if index.insert(r.key.as_str(), i).is_some() {
            return Err(invalid(&r.key, "a region key appears twice"));
        }
        if i > 0 && raw.regions[i - 1].key >= r.key {
            return Err(invalid(&r.key, "regions are in key order"));
        }
        if r.hcs.is_empty() || r.hcs.len() != r.hcs_number.len() || r.name.is_empty() {
            return Err(invalid(
                &r.key,
                "a region names its HCS codes and numbers, and a name",
            ));
        }
        finite(&r.key, "area_km2", r.area_km2, f64::MIN_POSITIVE)?;
        finite(&r.key, "sea_km", r.sea_km, 0.0)?;
        if r.coastal != (r.sea_km >= 1.0) {
            return Err(invalid(&r.key, "coastal is sea_km >= 1"));
        }
    }

    // Adjacency: resolved, not to itself, one entry per neighbour, symmetric.
    let mut neighbours: Vec<Vec<Neighbour>> = Vec::with_capacity(n);
    let mut pairs: BTreeMap<(usize, usize), u64> = BTreeMap::new();
    for (i, r) in raw.regions.iter().enumerate() {
        let mut seen = BTreeSet::new();
        let mut list = Vec::with_capacity(r.neighbours.len());
        for (k, km) in &r.neighbours {
            let Some(&j) = index.get(k.as_str()) else {
                return Err(invalid(&r.key, format!("neighbour {k} is not a region")));
            };
            if j == i || !seen.insert(j) {
                return Err(invalid(
                    &r.key,
                    format!("neighbour {k} is itself or listed twice"),
                ));
            }
            finite(&r.key, "border_km", *km, f64::MIN_POSITIVE)?;
            pairs.insert((i, j), km.to_bits());
            list.push(Neighbour {
                region: j,
                border_km: *km,
            });
        }
        neighbours.push(list);
    }
    for (&(i, j), &km) in &pairs {
        if pairs.get(&(j, i)) != Some(&km) {
            return Err(invalid(
                &raw.regions[i].key,
                format!(
                    "{} lists it as a neighbour, but not back with the same length",
                    raw.regions[j].key
                ),
            ));
        }
    }

    // Arcs.
    let mut borders = Vec::with_capacity(raw.arcs.len());
    for (i, (left, right, flat)) in raw.arcs.iter().enumerate() {
        if *left >= n || right.is_some_and(|r| r >= n || r == *left) {
            return Err(invalid(
                format!("arc {i}"),
                "an arc's sides are two different regions, or a region and None",
            ));
        }
        if let Some(r) = *right {
            if !pairs.contains_key(&(*left, r)) {
                return Err(invalid(
                    format!("arc {i}"),
                    format!(
                        "{} and {} share an arc but are not neighbours",
                        raw.regions[*left].key, raw.regions[r].key
                    ),
                ));
            }
        }
        borders.push(Border {
            left: *left,
            right: *right,
            points: decode_arc(i, q, flat)?,
        });
    }

    // Shapes: every ring closes from arcs whose side it is, anticlockwise outside, clockwise
    // holes; every arc used once from each side it has.
    if raw.shapes.len() != n {
        return Err(invalid(
            "shapes",
            "one shape per region, in the regions' order",
        ));
    }
    let mut forward = vec![0_u32; borders.len()];
    let mut backward = vec![0_u32; borders.len()];
    let mut shapes = Vec::with_capacity(n);
    for (i, (key, raw_parts)) in raw.shapes.iter().enumerate() {
        if *key != raw.regions[i].key {
            return Err(invalid(key, "shapes are in the regions' order"));
        }
        if raw_parts.is_empty() {
            return Err(invalid(key, "a region has no geometry"));
        }
        let mut parts = Vec::with_capacity(raw_parts.len());
        for raw_rings in raw_parts {
            if raw_rings.is_empty() {
                return Err(invalid(key, "a part has no rings"));
            }
            let mut rings = Vec::with_capacity(raw_rings.len());
            for (h, refs) in raw_rings.iter().enumerate() {
                let mut ring: Vec<Point> = Vec::new();
                for &r in refs {
                    let (a, fwd) = if r >= 0 {
                        (usize::try_from(r).unwrap_or(usize::MAX), true)
                    } else {
                        (usize::try_from(-1 - r).unwrap_or(usize::MAX), false)
                    };
                    let Some(b) = borders.get(a) else {
                        return Err(invalid(key, format!("arc reference {r} is out of range")));
                    };
                    let side = if fwd { Some(b.left) } else { b.right };
                    if side != Some(i) {
                        return Err(invalid(key, format!("arc reference {r} is not its side")));
                    }
                    if fwd {
                        forward[a] += 1;
                    } else {
                        backward[a] += 1;
                    }
                    let pts: Vec<Point> = if fwd {
                        b.points.clone()
                    } else {
                        b.points.iter().rev().copied().collect()
                    };
                    if let Some(last) = ring.last() {
                        if *last != pts[0] {
                            return Err(invalid(key, format!("arc reference {r} does not join")));
                        }
                        ring.extend_from_slice(&pts[1..]);
                    } else {
                        ring = pts;
                    }
                }
                if ring.len() < 4 || ring.first() != ring.last() {
                    return Err(invalid(
                        key,
                        "a ring does not close, or has under three points",
                    ));
                }
                let a = signed_area(&ring);
                if (h == 0) != (a > 0.0) || a == 0.0 {
                    return Err(invalid(
                        key,
                        "exteriors run anticlockwise and holes clockwise",
                    ));
                }
                rings.push(ring);
            }
            parts.push(Part { rings });
        }
        shapes.push(parts);
    }
    for (a, b) in borders.iter().enumerate() {
        let back = u32::from(b.right.is_some());
        if forward[a] != 1 || backward[a] != back {
            return Err(invalid(
                format!("arc {a}"),
                "an arc is used once from each side it has",
            ));
        }
    }

    // Sites, and the port flag they back.
    let mut site_keys = BTreeSet::new();
    let mut with_port = BTreeSet::new();
    let mut sites = Vec::with_capacity(raw.sites.len());
    for s in raw.sites {
        let Some(&region) = index.get(s.region.as_str()) else {
            return Err(invalid(
                &s.key,
                format!("site region {} is not a region", s.region),
            ));
        };
        if s.key.is_empty() || s.name.is_empty() || !site_keys.insert(s.key.clone()) {
            return Err(invalid(&s.key, "a site has a unique key and a name"));
        }
        if s.kind == SiteKind::Port {
            with_port.insert(region);
        }
        sites.push(Site {
            key: s.key,
            kind: s.kind,
            name: s.name,
            region,
            at: point(1.0, s.at),
        });
    }

    let regions = raw
        .regions
        .into_iter()
        .zip(neighbours)
        .zip(shapes)
        .enumerate()
        .map(|(i, ((r, neighbours), parts))| {
            if r.port != with_port.contains(&i) {
                return Err(invalid(
                    &r.key,
                    "port is true exactly when a port site is in it",
                ));
            }
            Ok(Region {
                key: r.key,
                chapman: r.chapman,
                hcs: r.hcs,
                hcs_number: r.hcs_number,
                name: r.name,
                country: r.country,
                area_km2: r.area_km2,
                centroid: point(1.0, r.centroid),
                label: point(1.0, r.label),
                sea_km: r.sea_km,
                coastal: r.coastal,
                port: r.port,
                neighbours,
                parts,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Atlas {
        schema: raw.schema,
        crs: raw.crs,
        quantum_m: raw.quantum_m,
        tolerance_m: raw.tolerance_m,
        sources: raw.sources,
        regions,
        borders,
        sites,
        digest,
    })
}
