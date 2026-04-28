use super::types::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};
use serde_yaml::{Mapping, Value};

pub(super) fn lint_skill_frontmatter_schema(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &Mapping,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    lint_required_skill_type(yaml, yaml_line_offset, mapping, issues);
    lint_required_name(yaml, yaml_line_offset, mapping, issues);
    let Some(metadata) = lint_required_metadata(yaml, yaml_line_offset, mapping, issues) else {
        return;
    };

    lint_required_metadata_string(
        yaml,
        yaml_line_offset,
        metadata,
        "metadata.version",
        "version",
        issues,
    );
    lint_required_metadata_string(
        yaml,
        yaml_line_offset,
        metadata,
        "metadata.source",
        "source",
        issues,
    );
    lint_required_string_sequence(
        yaml,
        yaml_line_offset,
        metadata,
        "metadata.routing_keywords",
        2,
        "routing_keywords",
        issues,
    );
    lint_optional_string_sequence(
        yaml,
        yaml_line_offset,
        metadata,
        "metadata.intents",
        2,
        "intents",
        issues,
    );
    lint_legacy_keywords(yaml, yaml_line_offset, mapping, issues);
}

fn lint_required_skill_type(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &Mapping,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    if string_field(mapping, "type") == Some("skill") {
        return;
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema,
        "skill frontmatter top-level `type` must be `skill`",
        key_position(yaml, yaml_line_offset, "type", 0).unwrap_or((1, 1)),
    );
}

fn lint_required_name(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &Mapping,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    if string_field(mapping, "name").is_some() {
        return;
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::MissingSkillFrontmatterName,
        "skill frontmatter must include a non-empty top-level `name` field",
        key_position(yaml, yaml_line_offset, "name", 0).unwrap_or((1, 1)),
    );
}

fn lint_required_metadata<'a>(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &'a Mapping,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> Option<&'a Mapping> {
    if let Some(metadata) = field(mapping, "metadata").and_then(Value::as_mapping) {
        return Some(metadata);
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata,
        "skill frontmatter must contain a top-level `metadata` mapping",
        key_position(yaml, yaml_line_offset, "metadata", 0).unwrap_or((1, 1)),
    );
    None
}

fn lint_required_metadata_string(
    yaml: &str,
    yaml_line_offset: usize,
    metadata: &Mapping,
    display_path: &str,
    key: &str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    if string_field(metadata, key).is_some() {
        return;
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema,
        format!("skill frontmatter `{display_path}` must be a non-empty string"),
        key_position(yaml, yaml_line_offset, key, 2)
            .or_else(|| key_position(yaml, yaml_line_offset, "metadata", 0))
            .unwrap_or((1, 1)),
    );
}

fn lint_required_string_sequence(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &Mapping,
    display_path: &str,
    indent: usize,
    key: &str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let is_valid = field(mapping, key).is_some_and(is_non_empty_string_sequence);
    if is_valid {
        return;
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema,
        format!("skill frontmatter `{display_path}` must be a non-empty string array"),
        key_position(yaml, yaml_line_offset, key, indent)
            .or_else(|| key_position(yaml, yaml_line_offset, "metadata", 0))
            .unwrap_or((1, 1)),
    );
}

fn lint_optional_string_sequence(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &Mapping,
    display_path: &str,
    indent: usize,
    key: &str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let Some(value) = field(mapping, key) else {
        return;
    };
    if is_non_empty_string_sequence(value) {
        return;
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema,
        format!("skill frontmatter `{display_path}` must be a non-empty string array"),
        key_position(yaml, yaml_line_offset, key, indent).unwrap_or((1, 1)),
    );
}

fn lint_legacy_keywords(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: &Mapping,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    if !mapping.contains_key(Value::String("keywords".to_string())) {
        return;
    }

    push_issue(
        issues,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema,
        "skill frontmatter must use `metadata.routing_keywords`; legacy top-level `keywords` is not allowed",
        key_position(yaml, yaml_line_offset, "keywords", 0).unwrap_or((1, 1)),
    );
}

fn field<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

fn string_field<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a str> {
    field(mapping, key)
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

fn key_position(
    yaml: &str,
    yaml_line_offset: usize,
    key: &str,
    indent: usize,
) -> Option<(usize, usize)> {
    let expected_prefix = format!("{key}:");
    yaml.lines().enumerate().find_map(|(line_index, line)| {
        let leading_spaces = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        if leading_spaces != indent {
            return None;
        }
        let trimmed = line.trim_start();
        trimmed
            .starts_with(expected_prefix.as_str())
            .then_some((yaml_line_offset + line_index, leading_spaces + 1))
    })
}

fn push_issue(
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
    code: MarkdownSyntaxLintCode,
    message: impl Into<String>,
    position: (usize, usize),
) {
    issues.push(MarkdownSyntaxLintIssue {
        code,
        message: message.into(),
        line: position.0,
        column: position.1,
    });
}
