//! Manifest inspection helpers for runtime bootstrapping decisions.

use crate::contracts::QianjiManifest;
use crate::error::QianjiError;

/// Returns whether a manifest requires LLM runtime availability.
///
/// This includes:
/// - direct `llm` task nodes
/// - `formal_audit` nodes that declare both `[nodes.annotation]` and `[nodes.llm]`
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the manifest TOML is invalid.
pub fn manifest_requires_llm(manifest_toml: &str) -> Result<bool, QianjiError> {
    let manifest: QianjiManifest = toml::from_str(manifest_toml)
        .map_err(|error| QianjiError::Topology(format!("Failed to parse TOML: {error}")))?;

    Ok(manifest.nodes.iter().any(|node| {
        if node.task_type.trim().eq_ignore_ascii_case("llm") {
            return true;
        }
        node.task_type.trim().eq_ignore_ascii_case("formal_audit")
            && node.annotation.is_some()
            && node.llm.is_some()
    }))
}

/// Returns whether a manifest declares at least one node-level annotation binding.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the manifest TOML is invalid.
pub fn manifest_declares_annotation_bindings(manifest_toml: &str) -> Result<bool, QianjiError> {
    let manifest: QianjiManifest = toml::from_str(manifest_toml)
        .map_err(|error| QianjiError::Topology(format!("Failed to parse TOML: {error}")))?;

    Ok(manifest.nodes.iter().any(|node| node.annotation.is_some()))
}
