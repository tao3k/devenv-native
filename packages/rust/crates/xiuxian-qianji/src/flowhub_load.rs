use std::fs;
use std::path::Path;

use crate::contracts::{FlowhubModuleManifest, FlowhubRootManifest, FlowhubScenarioManifest};
use crate::error::QianjiError;

use super::parse::{parse_flowhub_module_manifest, parse_flowhub_scenario_manifest};

/// Load, parse, and validate a Flowhub module manifest from disk.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the file cannot be read or the
/// manifest violates the current Flowhub module contract.
pub fn load_flowhub_module_manifest(
    manifest_path: impl AsRef<Path>,
) -> Result<FlowhubModuleManifest, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub module manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    parse_flowhub_module_manifest(&manifest_toml)
        .map_err(|error| with_manifest_path(error, "Flowhub module manifest", manifest_path))
}

/// Load, parse, and validate a Flowhub scenario manifest from disk.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the file cannot be read or the
/// manifest violates the current Flowhub scenario contract.
pub fn load_flowhub_scenario_manifest(
    manifest_path: impl AsRef<Path>,
) -> Result<FlowhubScenarioManifest, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub scenario manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    parse_flowhub_scenario_manifest(&manifest_toml)
        .map_err(|error| with_manifest_path(error, "Flowhub scenario manifest", manifest_path))
}

/// Load, parse, and validate a Flowhub root manifest from disk.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the file cannot be read or the
/// manifest violates the current Flowhub root contract.
pub fn load_flowhub_root_manifest(
    manifest_path: impl AsRef<Path>,
) -> Result<FlowhubRootManifest, QianjiError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_toml = fs::read_to_string(manifest_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read Flowhub root manifest `{}`: {error}",
            manifest_path.display()
        ))
    })?;

    let manifest = toml::from_str::<FlowhubRootManifest>(&manifest_toml).map_err(|error| {
        QianjiError::Topology(format!(
            "Flowhub root manifest `{}` failed validation: {error}",
            manifest_path.display()
        ))
    })?;
    super::validate::validate_flowhub_root_manifest(&manifest)
        .map_err(|error| with_manifest_path(error, "Flowhub root manifest", manifest_path))?;
    Ok(manifest)
}

fn with_manifest_path(error: QianjiError, kind: &str, manifest_path: &Path) -> QianjiError {
    match error {
        QianjiError::Topology(message) => QianjiError::Topology(format!(
            "{kind} `{}` failed validation: {message}",
            manifest_path.display()
        )),
        other => other,
    }
}
