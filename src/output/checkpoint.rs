use crate::state::SimState;
use std::error::Error;
use std::fs;
use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Selects the serialisation format for a checkpoint.
/// Binary is fast and compact; HumanReadable is slow but inspectable.
/// The format is inferred from the file extension when loading.
pub enum SaveFormat {
    Binary,
    HumanReadable,
}

impl SaveFormat {
    pub fn extension(&self) -> &str {
        match self {
            SaveFormat::Binary => "bin",
            SaveFormat::HumanReadable => "ron",
        }
    }
}

/// Serialise `state` to `path`. Format is determined by `format`.
/// Binary: bincode. HumanReadable: RON with pretty-printing.
pub fn save(state: &SimState, format: &SaveFormat, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match format {
        SaveFormat::Binary => {
            let bytes = bincode::serialize(state)?;
            fs::write(path, bytes)?;
        }
        SaveFormat::HumanReadable => {
            let text = ron::ser::to_string_pretty(
                state,
                ron::ser::PrettyConfig::default(),
            )?;
            fs::write(path, text)?;
        }
    }
    Ok(())
}

/// Load a `SimState` from `path`. Format is inferred from the file extension:
/// `.bin` → binary (bincode), `.ron` → HumanReadable (RON).
pub fn load(path: &Path) -> Result<SimState> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "bin" => {
            let bytes = fs::read(path)?;
            Ok(bincode::deserialize(&bytes)?)
        }
        "ron" => {
            let text = fs::read_to_string(path)?;
            Ok(ron::from_str(&text)?)
        }
        other => Err(format!("unknown checkpoint extension '.{other}'; expected .bin or .ron").into()),
    }
}

/// Convenience: save a checkpoint into `output_dir/tick_{tick:08}.{ext}`.
pub fn save_checkpoint(
    state: &SimState,
    format: &SaveFormat,
    output_dir: &Path,
) -> Result<()> {
    let filename = format!("tick_{:08}.{}", state.tick, format.extension());
    save(state, format, &output_dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SimState;
    use tempfile::tempdir;

    fn minimal_state() -> SimState {
        SimState::new(2, 1)
    }

    #[test]
    fn binary_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.bin");
        let state = minimal_state();
        save(&state, &SaveFormat::Binary, &path).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(state.tick, loaded.tick);
    }

    #[test]
    fn human_readable_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.ron");
        let state = minimal_state();
        save(&state, &SaveFormat::HumanReadable, &path).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(state.tick, loaded.tick);
    }

    #[test]
    fn unknown_extension_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.xyz");
        // Write something so the file exists
        fs::write(&path, b"garbage").unwrap();
        assert!(load(&path).is_err());
    }
}
