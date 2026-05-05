//! TOML file read and parse helpers for config resolution.

use crate::ConfigCoreError;
use std::path::Path;

pub(super) fn read_toml(path: &Path) -> Result<toml::Value, ConfigCoreError> {
    let content = std::fs::read_to_string(path).map_err(|source| ConfigCoreError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;
    toml::from_str::<toml::Value>(&content).map_err(|source| ConfigCoreError::ParseFile {
        path: path.display().to_string(),
        source,
    })
}
