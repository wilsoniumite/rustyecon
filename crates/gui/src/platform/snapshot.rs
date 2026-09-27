//! PNG snapshots of the window, never citable (docs/GUI.md §9, G1; U12). A snapshot is a
//! picture of the screen for the author's notes, not a figure: a figure is rebuilt from its
//! data bundle (G6–7), and a number is cited from its run's export or the oracle, never read
//! off an image. So every snapshot says so twice: the window paints a banner across its top
//! while the frame it keeps is drawn, and the file carries the same words in its text chunks,
//! with the build, the run or the lab it shows, and the date.
//!
//! Snapshots go in `snapshots/` under the session's directory, each under a new name, and no
//! file is written over.

use std::path::{Path, PathBuf};

/// What every snapshot says of itself, in its banner and its `Comment` chunk.
pub const NEVER_CITABLE: &str = "NEVER CITABLE: a picture of the rustyecon GUI's screen, not a \
figure. Cite a number from its run's export or from the oracle, never from this image.";

/// The banner's words, shorter, painted across the window's top.
pub const BANNER: &str = "NOT CITABLE · rustyecon GUI snapshot";

/// The PNG of an RGBA image, `width` × `height`, row by row with no padding, with `marks` as
/// its tEXt chunks, `(keyword, text)`, after `Comment` = [`NEVER_CITABLE`]. Keywords and texts
/// are written as Latin-1; a character outside it is written `?`.
pub fn png(
    rgba: &[u8],
    width: u32,
    height: u32,
    marks: &[(&str, String)],
) -> Result<Vec<u8>, String> {
    let want = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or("the image is too large")?;
    if rgba.len() != want {
        return Err(format!(
            "{} bytes for a {width} × {height} RGBA image, which has {want}",
            rgba.len()
        ));
    }
    let latin1 = |s: &str| -> String {
        s.chars()
            .map(|c| if (c as u32) < 256 { c } else { '?' })
            .collect()
    };
    let mut out = Vec::new();
    {
        let mut e = ::png::Encoder::new(&mut out, width, height);
        e.set_color(::png::ColorType::Rgba);
        e.set_depth(::png::BitDepth::Eight);
        e.add_text_chunk("Comment".to_string(), latin1(NEVER_CITABLE))
            .map_err(|x| x.to_string())?;
        for (k, v) in marks {
            e.add_text_chunk(latin1(k), latin1(v))
                .map_err(|x| x.to_string())?;
        }
        let mut w = e.write_header().map_err(|x| x.to_string())?;
        w.write_image_data(rgba).map_err(|x| x.to_string())?;
        w.finish().map_err(|x| x.to_string())?;
    }
    Ok(out)
}

/// A new path for a snapshot made on `date` in `dir/snapshots/`: the first
/// `rustyecon-snapshot-<date>-<n>.png` with no file there, n from 1.
pub fn path_in(dir: &Path, date: &str) -> PathBuf {
    let shots = dir.join("snapshots");
    (1..)
        .map(|n| shots.join(format!("rustyecon-snapshot-{date}-{n}.png")))
        .find(|p| !p.exists())
        .expect("some n is free")
}

/// Write a snapshot's bytes to a new path in `dir/snapshots/`, never over a file.
pub fn write(dir: &Path, date: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let path = path_in(dir, date);
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    std::io::Write::write_all(&mut f, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}
