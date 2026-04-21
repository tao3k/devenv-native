use crate::contracts::WorkdirManifest;
use crate::error::QianjiError;

use super::validate::validate_workdir_manifest;

/// Parse and validate a bounded work-surface manifest.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when TOML parsing fails or the manifest
/// violates the compact bounded work-surface contract.
pub fn parse_workdir_manifest(manifest_toml: &str) -> Result<WorkdirManifest, QianjiError> {
    let manifest: WorkdirManifest = toml::from_str(manifest_toml).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse bounded work-surface manifest TOML: {error}"
        ))
    })?;
    validate_workdir_manifest(&manifest)?;
    Ok(manifest)
}
