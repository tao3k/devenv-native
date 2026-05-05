//! Public frontmatter parsing API and SKILL.md validation helpers.

use super::raw::{split_frontmatter, split_frontmatter_raw};
use super::types::NoteFrontmatter;
use chrono::{DateTime, NaiveDateTime};
use serde_yaml::{Mapping, Value};
use std::fmt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Error returned when strict SKILL.md frontmatter parsing fails.
#[derive(Debug)]
pub enum SkillFrontmatterParseError {
    /// The frontmatter block exists but is not valid YAML.
    InvalidYaml(serde_yaml::Error),
    /// The frontmatter block is valid YAML but violates the strict schema.
    InvalidSchema(Vec<String>),
}

impl fmt::Display for SkillFrontmatterParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidYaml(error) => write!(formatter, "invalid YAML frontmatter: {error}"),
            Self::InvalidSchema(issues) => write!(formatter, "{}", issues.join("; ")),
        }
    }
}

impl std::error::Error for SkillFrontmatterParseError {}

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

    NoteFrontmatter {
        title: mapping_string(mapping, "title"),
        description: mapping_string(mapping, "description"),
        name: mapping_string(mapping, "name"),
        category: mapping_string(mapping, "category"),
        tags: mapping_string_vec(mapping, "tags"),
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

/// Parse one skill-shaped frontmatter block using the strict parser-owned
/// SKILL.md schema.
///
/// # Errors
///
/// Returns [`SkillFrontmatterParseError`] when the document has no leading
/// frontmatter, invalid YAML, or a schema violation.
pub fn parse_skill_frontmatter(
    content: &str,
) -> Result<NoteFrontmatter, SkillFrontmatterParseError> {
    let Some(parts) = split_frontmatter_raw(content) else {
        return Err(SkillFrontmatterParseError::InvalidSchema(vec![
            "document must start with a YAML frontmatter block".to_string(),
        ]));
    };
    let value = serde_yaml::from_str::<Value>(parts.yaml)
        .map_err(SkillFrontmatterParseError::InvalidYaml)?;
    let Some(mapping) = value.as_mapping() else {
        return Err(SkillFrontmatterParseError::InvalidSchema(vec![
            "skill frontmatter must be a YAML mapping".to_string(),
        ]));
    };
    validate_strict_skill_frontmatter(mapping)?;
    Ok(frontmatter_from_mapping(mapping))
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

fn validate_strict_skill_frontmatter(mapping: &Mapping) -> Result<(), SkillFrontmatterParseError> {
    let mut issues = Vec::new();

    require_string(
        mapping,
        "title",
        "frontmatter must include a non-empty `title` field",
        &mut issues,
    );
    require_string(
        mapping,
        "kind",
        "frontmatter must include a non-empty `kind` field",
        &mut issues,
    );
    require_string(
        mapping,
        "category",
        "frontmatter must include a non-empty `category` field",
        &mut issues,
    );
    require_string(
        mapping,
        "description",
        "frontmatter must include a non-empty `description` field",
        &mut issues,
    );
    require_string(
        mapping,
        "author",
        "frontmatter must include a non-empty `author` field",
        &mut issues,
    );
    require_minute_precision_date(mapping, &mut issues);
    require_string_sequence(
        mapping,
        "tags",
        "frontmatter must include a non-empty `tags` array",
        &mut issues,
    );

    require_number(
        mapping,
        "saliency_base",
        "frontmatter must include numeric top-level `saliency_base`",
        &mut issues,
    );
    require_number(
        mapping,
        "decay_rate",
        "frontmatter must include numeric top-level `decay_rate`",
        &mut issues,
    );

    if string_field(mapping, "type") != Some("skill") {
        issues.push("skill frontmatter top-level `type` must be `skill`".to_string());
    }
    require_string(
        mapping,
        "name",
        "skill frontmatter must include a non-empty top-level `name` field",
        &mut issues,
    );
    let metadata = mapping_mapping(mapping, "metadata");
    let Some(metadata) = metadata else {
        issues.push("skill frontmatter must contain a top-level `metadata` mapping".to_string());
        return schema_result(issues);
    };
    require_string(
        metadata,
        "version",
        "skill frontmatter `metadata.version` must be a non-empty string",
        &mut issues,
    );
    require_string(
        metadata,
        "source",
        "skill frontmatter `metadata.source` must be a non-empty string",
        &mut issues,
    );
    require_string_sequence(
        metadata,
        "routing_keywords",
        "skill frontmatter `metadata.routing_keywords` must be a non-empty string array",
        &mut issues,
    );
    if let Some(intents) = mapping_value(metadata, "intents")
        && !is_non_empty_string_sequence(intents)
    {
        issues.push(
            "skill frontmatter `metadata.intents` must be a non-empty string array".to_string(),
        );
    }
    if mapping.contains_key(Value::String("keywords".to_string())) {
        issues.push(
            "skill frontmatter must use `metadata.routing_keywords`; legacy top-level `keywords` is not allowed"
                .to_string(),
        );
    }

    schema_result(issues)
}

fn schema_result(issues: Vec<String>) -> Result<(), SkillFrontmatterParseError> {
    if issues.is_empty() {
        Ok(())
    } else {
        Err(SkillFrontmatterParseError::InvalidSchema(issues))
    }
}

fn frontmatter_from_mapping(mapping: &Mapping) -> NoteFrontmatter {
    let metadata = mapping_mapping(mapping, "metadata");
    NoteFrontmatter {
        title: mapping_string(mapping, "title"),
        description: mapping_string(mapping, "description"),
        name: mapping_string(mapping, "name"),
        category: mapping_string(mapping, "category"),
        tags: mapping_string_vec(mapping, "tags"),
        routing_keywords: metadata.map_or_else(Vec::new, |value| {
            mapping_string_vec(value, "routing_keywords")
        }),
        intents: metadata.map_or_else(Vec::new, |value| mapping_string_vec(value, "intents")),
    }
}

fn require_string(mapping: &Mapping, key: &str, message: &str, issues: &mut Vec<String>) {
    if string_field(mapping, key).is_none() {
        issues.push(message.to_string());
    }
}

fn require_string_sequence(mapping: &Mapping, key: &str, message: &str, issues: &mut Vec<String>) {
    if !mapping_value(mapping, key).is_some_and(is_non_empty_string_sequence) {
        issues.push(message.to_string());
    }
}

fn require_number(mapping: &Mapping, key: &str, message: &str, issues: &mut Vec<String>) {
    let top_level = mapping_value(mapping, key).and_then(Value::as_f64);
    let legacy_skill_retrieval = mapping_mapping(mapping, "metadata")
        .and_then(|metadata| mapping_mapping(metadata, "retrieval"))
        .and_then(|retrieval| mapping_value(retrieval, key))
        .and_then(Value::as_f64);
    if top_level.or(legacy_skill_retrieval).is_none() {
        issues.push(message.to_string());
    }
}

fn require_minute_precision_date(mapping: &Mapping, issues: &mut Vec<String>) {
    let Some(date) = string_field(mapping, "date") else {
        issues.push("frontmatter must include a minute-precision `date` field".to_string());
        return;
    };
    if !is_minute_precision_timestamp(date) {
        issues.push("frontmatter `date` must use minute precision".to_string());
    }
}

fn string_field<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a str> {
    mapping_value(mapping, key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn is_non_empty_string_sequence(value: &Value) -> bool {
    value.as_sequence().is_some_and(|items| {
        !items.is_empty()
            && items
                .iter()
                .all(|item| item.as_str().is_some_and(|value| !value.trim().is_empty()))
    })
}

fn is_minute_precision_timestamp(value: &str) -> bool {
    DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M%:z").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%MZ").is_ok()
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
