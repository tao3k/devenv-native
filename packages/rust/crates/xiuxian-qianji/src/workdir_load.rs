use std::fs;
use std::path::Path;

use crate::contracts::WorkdirManifest;
use crate::error::QianjiError;

use super::parse::parse_workdir_manifest;

/// Load, parse, and validate a bounded work-surface manifest from disk.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the file cannot be read or the
/// manifest violates the bounded work-surface contract.
pub fn load_workdir_manifest(
    manifest_path: impl AsRef<Path>,
) -> Result<WorkdirManifest, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read bounded work-surface manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    parse_workdir_manifest(&manifest_toml).map_err(|error| match error {
        QianjiError::Topology(message) => QianjiError::Topology(format!(
            "bounded work-surface manifest `{}` failed validation: {message}",
            manifest_path.display()
        )),
        other => other,
    })
}
