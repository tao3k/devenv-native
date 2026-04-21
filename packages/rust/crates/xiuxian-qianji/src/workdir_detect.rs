use std::fs;
use std::path::Path;

use crate::error::QianjiError;

/// Returns `true` when the directory looks like a bounded work-surface root.
///
/// Detection is intentionally shallow: it only checks whether `qianji.toml`
/// declares the compact `[plan]` and `[check]` tables, so invalid manifests
/// still route to the workdir code path and produce useful errors later.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when `qianji.toml` exists but cannot be
/// read.
pub fn looks_like_workdir_dir(dir: impl AsRef<Path>) -> Result<bool, QianjiError> {
    manifest_declares_tables(dir.as_ref().join("qianji.toml"), &["plan", "check"])
}

fn manifest_declares_tables(
    manifest_path: impl AsRef<Path>,
    required_tables: &[&str],
) -> Result<bool, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    if !manifest_path.is_file() {
        return Ok(false);
    }

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read bounded work-surface manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    if let Ok(value) = toml::from_str::<toml::Value>(&manifest_toml) {
        let Some(table) = value.as_table() else {
            return Ok(false);
        };
        return Ok(required_tables.iter().all(|name| table.contains_key(*name)));
    }

    Ok(required_tables
        .iter()
        .all(|name| manifest_toml.contains(&format!("[{name}]"))))
}
