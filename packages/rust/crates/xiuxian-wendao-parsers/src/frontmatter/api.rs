use super::raw::{split_frontmatter, split_frontmatter_raw};
use super::types::NoteFrontmatter;
use serde_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn mapping_value<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

fn mapping_string(mapping: &Mapping, key: &str) -> Option<String> {
    mapping_value(mapping, key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn mapping_mapping<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Mapping> {
    mapping_value(mapping, key).and_then(Value::as_mapping)
}

fn mapping_string_vec(mapping: &Mapping, key: &str) -> Vec<String> {
    mapping_value(mapping, key)
        .and_then(Value::as_sequence)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Parse semantic note frontmatter used by shared document consumers.
#[must_use]
pub fn parse_frontmatter(content: &str) -> NoteFrontmatter {
    let (frontmatter, _body) = split_frontmatter(content);
    let Some(mapping) = frontmatter.as_ref().and_then(Value::as_mapping) else {
        return NoteFrontmatter::default();
    };

    let metadata = mapping_value(mapping, "metadata").and_then(Value::as_mapping);
    let mut tags = mapping_string_vec(mapping, "tags");
    if tags.is_empty() {
        tags = metadata.map_or_else(Vec::new, |value| mapping_string_vec(value, "tags"));
    }

    NoteFrontmatter {
        title: mapping_string(mapping, "title"),
        description: mapping_string(mapping, "description"),
        name: mapping_string(mapping, "name"),
        category: mapping_string(mapping, "category"),
        tags,
        routing_keywords: metadata.map_or_else(Vec::new, |value| {
            mapping_string_vec(value, "routing_keywords")
        }),
        intents: metadata.map_or_else(Vec::new, |value| mapping_string_vec(value, "intents")),
    }
}

/// Returns the optional top-level `kind` field from document frontmatter.
#[must_use]
pub fn frontmatter_kind(content: &str) -> Option<String> {
    let (frontmatter, _body) = split_frontmatter(content);
    let mapping = frontmatter.as_ref().and_then(Value::as_mapping)?;
    mapping_string(mapping, "kind")
}

/// Returns the semantic skill identity name when present.
///
/// This follows the parser-owned SKILL.md contract and only accepts the
/// top-level `name` field.
#[must_use]
pub fn skill_frontmatter_name(content: &str) -> Option<String> {
    let (frontmatter, _body) = split_frontmatter(content);
    let mapping = frontmatter.as_ref().and_then(Value::as_mapping)?;
    mapping_string(mapping, "name")
}

/// Parse one skill-shaped frontmatter block using the current runtime lenient
/// contract.
///
/// This helper keeps runtime consumers aligned with the historical
/// `SkillScanner::scan_skill(..., None)` behavior:
///
/// 1. missing frontmatter returns `Ok(None)`
/// 2. invalid YAML returns an error
/// 3. missing `metadata` is allowed
///
/// # Errors
///
/// Returns the underlying YAML parse error when the frontmatter exists but is
/// not valid for the parser-owned `NoteFrontmatter` shape.
pub fn parse_skill_frontmatter_lenient(
    content: &str,
) -> Result<Option<NoteFrontmatter>, serde_yaml::Error> {
    let Some(parts) = split_frontmatter_raw(content) else {
        return Ok(None);
    };
    serde_yaml::from_str::<NoteFrontmatter>(parts.yaml).map(Some)
}

/// Returns true when skill frontmatter contains the required top-level
/// `metadata` mapping defined by the parser-owned SKILL.md contract.
#[must_use]
pub fn skill_frontmatter_has_metadata_mapping(content: &str) -> bool {
    let (frontmatter, _body) = split_frontmatter(content);
    let Some(mapping) = frontmatter.as_ref().and_then(Value::as_mapping) else {
        return false;
    };
    mapping_mapping(mapping, "metadata").is_some()
}

/// Returns true when a Markdown document should use the skill frontmatter
/// variant instead of the ordinary note frontmatter requirements.
#[must_use]
pub fn uses_skill_frontmatter(path: Option<&Path>, content: &str) -> bool {
    is_skill_descriptor_path(path)
        || frontmatter_kind(content).is_some_and(|value| value.eq_ignore_ascii_case("SKILL.md"))
}

/// Returns true when the physical path points to a canonical `SKILL.md`
/// document.
#[must_use]
pub fn is_skill_descriptor_path(path: Option<&Path>) -> bool {
    path.and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md"))
}

/// Discover all `SKILL.md` documents under one root in deterministic order.
#[must_use]
pub fn discover_skill_documents(root: &Path) -> Vec<PathBuf> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut discovered = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| is_skill_descriptor_path(Some(path.as_path())))
        .collect::<Vec<_>>();
    discovered.sort();
    discovered
}
