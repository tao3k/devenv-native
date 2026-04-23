use std::path::Path;

use crate::frontmatter::strict_parse;
use crate::skills::metadata::ReferenceRecord;

use super::model::{ReferenceFrontmatter, ReferenceMetadataBlock, UnifiedMetadataType};

mod build;
mod filesystem;

/// Scan `references/` under a skill directory and return reference records.
/// Parses YAML frontmatter for `for_tools`; `for_skills` and `skill_name`
/// are derived from tool full names.
pub(super) fn scan_references(skill_path: &Path, skill_name: &str) -> Vec<ReferenceRecord> {
    let paths = filesystem::discover_reference_markdown_files(skill_path);
    let records: Vec<ReferenceRecord> = paths
        .iter()
        .filter_map(|path| scan_reference_file(path.as_path(), skill_name))
        .collect();

    if log::log_enabled!(log::Level::Debug) && !records.is_empty() {
        log::debug!(
            "Scanned {} reference(s) for skill {}",
            records.len(),
            skill_name
        );
    }

    records
}

fn scan_reference_file(path: &Path, skill_name: &str) -> Option<ReferenceRecord> {
    let content = filesystem::read_reference_content(path)?;
    let (reference_name, file_path) = filesystem::reference_identity(path);
    let metadata: Option<ReferenceMetadataBlock> =
        build::parse_reference_metadata(content.as_str());

    Some(build::build_reference_record(
        reference_name,
        file_path,
        skill_name,
        metadata.as_ref(),
    ))
}

pub(super) fn validate_references_strict(skill_path: &Path) -> Result<(), String> {
    let paths = filesystem::discover_reference_markdown_files(skill_path);
    for path in paths {
        let content = filesystem::read_reference_content(path.as_path())
            .ok_or_else(|| format!("Failed to read reference file: {}", path.display()))?;
        let frontmatter = strict_parse::<ReferenceFrontmatter>(&content).map_err(|error| {
            format!(
                "reference metadata validation failed: invalid YAML frontmatter in reference markdown {}: {error}",
                path.display()
            )
        })?;
        if matches!(frontmatter.metadata_type, UnifiedMetadataType::Persona) {
            let role_valid = frontmatter
                .metadata
                .role_class
                .as_deref()
                .map(str::trim)
                .as_ref()
                .is_some_and(|value| !value.is_empty());
            if !role_valid {
                return Err(format!(
                    "reference metadata validation failed: invalid persona metadata in {}: `metadata.role_class` is required when type=persona",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}
