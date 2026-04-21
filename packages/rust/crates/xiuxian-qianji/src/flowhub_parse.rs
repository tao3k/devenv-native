use crate::contracts::{FlowhubModuleManifest, FlowhubScenarioManifest};
use crate::error::QianjiError;

use super::validate::{validate_flowhub_module_manifest, validate_flowhub_scenario_manifest};

/// Parse and validate a Flowhub module manifest.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when TOML parsing fails or the manifest
/// violates the current Flowhub module contract.
pub fn parse_flowhub_module_manifest(
    manifest_toml: &str,
) -> Result<FlowhubModuleManifest, QianjiError> {
    let manifest: FlowhubModuleManifest = toml::from_str(manifest_toml).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse Flowhub module manifest TOML: {error}"
        ))
    })?;
    validate_flowhub_module_manifest(&manifest)?;
    Ok(manifest)
}

/// Parse and validate a Flowhub scenario manifest.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when TOML parsing fails or the manifest
/// violates the current scenario grammar contract.
pub fn parse_flowhub_scenario_manifest(
    manifest_toml: &str,
) -> Result<FlowhubScenarioManifest, QianjiError> {
    let manifest: FlowhubScenarioManifest = toml::from_str(manifest_toml).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse Flowhub scenario manifest TOML: {error}"
        ))
    })?;
    validate_flowhub_scenario_manifest(&manifest)?;
    Ok(manifest)
}
