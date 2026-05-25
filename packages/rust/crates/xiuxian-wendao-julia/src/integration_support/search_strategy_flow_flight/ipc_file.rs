//! Temporary Arrow IPC files for the embedded local `SearchStrategyFlow` host.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Temporary Arrow IPC file whose parent directory is removed on drop.
#[derive(Debug)]
pub(crate) struct SearchStrategyFlowArrowIpcFile {
    dir: PathBuf,
    path: PathBuf,
}

impl SearchStrategyFlowArrowIpcFile {
    /// Write one Arrow IPC payload to a temporary file.
    ///
    /// # Errors
    ///
    /// Returns an error when the temporary directory or Arrow IPC file cannot
    /// be created.
    pub(crate) fn write(label: &str, payload: &[u8]) -> Result<Self, String> {
        let dir = unique_temp_dir(label)?;
        fs::create_dir(&dir).map_err(|error| {
            format!(
                "create SearchStrategyFlow temporary Arrow IPC dir `{}`: {error}",
                dir.display()
            )
        })?;
        let path = dir.join("payload.arrow");
        fs::write(&path, payload).map_err(|error| {
            format!(
                "write SearchStrategyFlow temporary Arrow IPC file `{}`: {error}",
                path.display()
            )
        })?;
        Ok(Self { dir, path })
    }

    /// Return this temporary file path.
    #[must_use]
    pub(crate) fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for SearchStrategyFlowArrowIpcFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn unique_temp_dir(label: &str) -> Result<PathBuf, String> {
    let mut label = label
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char == '-' {
                char
            } else {
                '-'
            }
        })
        .collect::<String>();
    if label.is_empty() {
        label.push_str("arrow-ipc");
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("resolve system time for temporary Arrow IPC path: {error}"))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "xiuxian-search-strategy-flow-{label}-{}-{nanos}",
        std::process::id()
    )))
}
