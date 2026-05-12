//! `skill_runtime::zhixing::resources::registry` owns Wendao zhixing resources registry behavior.

use crate::WendaoResourceRegistry;
use crate::skill_runtime::zhixing::{Error, Result};

use super::paths::{ZHIXING_SKILL_DOC_PATH, embedded_resource_dir};

/// Builds Wendao AST registry from embedded `resources/zhixing`.
///
/// # Errors
///
/// Returns an error when embedded markdown parsing fails or when linked
/// resource targets in markdown cannot be resolved from embedded files.
pub fn build_embedded_wendao_registry() -> Result<WendaoResourceRegistry> {
    WendaoResourceRegistry::build_from_embedded(embedded_resource_dir()).map_err(|error| {
        Error::Internal(format!("failed to build embedded wendao registry: {error}"))
    })
}

/// Resolves linked resource paths under the `zhixing/skills/agenda-management/SKILL.md` section for one id.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn embedded_skill_links_for_id(id: &str) -> Result<Vec<String>> {
    let links_by_id = embedded_skill_links_index()?;
    if let Some(links) = links_by_id.get(id) {
        return Ok(links.clone());
    }

    let registry = build_embedded_wendao_registry()?;
    let mut links = registry
        .files()
        .filter_map(|file| file.links_for_id(id))
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    links.sort();
    links.dedup();
    Ok(links)
}

/// Resolves linked semantic URIs under `zhixing/skills/agenda-management/SKILL.md` for one reference type.
///
/// Type matching is ASCII case-insensitive and based on explicit reference
/// categories derived from parser-owned metadata surfaces.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_skill_links_for_reference_type(reference_type: &str) -> Result<Vec<String>> {
    let registry = build_embedded_wendao_registry()?;
    let Some(skill_file) = registry.file(ZHIXING_SKILL_DOC_PATH) else {
        return Ok(Vec::new());
    };
    Ok(skill_file.links_for_reference_type(reference_type))
}

/// Returns all parsed linked resource paths keyed by heading `id` in `zhixing/skills/agenda-management/SKILL.md`.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_skill_links_index() -> Result<std::collections::HashMap<String, Vec<String>>> {
    let registry = build_embedded_wendao_registry()?;
    Ok(registry
        .file(ZHIXING_SKILL_DOC_PATH)
        .map_or_else(std::collections::HashMap::new, |entry| {
            entry.links_by_id().clone()
        }))
}
