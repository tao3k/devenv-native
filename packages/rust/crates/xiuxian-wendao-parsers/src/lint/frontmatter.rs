use super::types::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};
use crate::frontmatter::uses_skill_frontmatter;
use chrono::{DateTime, NaiveDateTime};
use serde_yaml::Value;
use std::path::Path;

pub(super) struct LintBody<'a> {
    pub(super) content: &'a str,
    pub(super) line_offset: usize,
}

#[derive(Clone, Copy, Debug)]
struct FrontmatterBlock<'a> {
    yaml: &'a str,
    body: &'a str,
    yaml_line_offset: usize,
    body_line_offset: usize,
}

enum FrontmatterScan<'a> {
    Missing,
    Unclosed,
    Found(FrontmatterBlock<'a>),
}

#[derive(Clone, Copy)]
struct CommonFrontmatterRequiredStringField {
    key: &'static str,
    code: MarkdownSyntaxLintCode,
    message: &'static str,
}

const COMMON_FRONTMATTER_IDENTITY_FIELDS: [CommonFrontmatterRequiredStringField; 3] = [
    CommonFrontmatterRequiredStringField {
        key: "title",
        code: MarkdownSyntaxLintCode::MissingFrontmatterTitle,
        message: "frontmatter must include a non-empty `title` field",
    },
    CommonFrontmatterRequiredStringField {
        key: "kind",
        code: MarkdownSyntaxLintCode::MissingFrontmatterKind,
        message: "frontmatter must include a non-empty `kind` field",
    },
    CommonFrontmatterRequiredStringField {
        key: "category",
        code: MarkdownSyntaxLintCode::MissingFrontmatterCategory,
        message: "frontmatter must include a non-empty `category` field",
    },
];

const COMMON_FRONTMATTER_PROVENANCE_FIELDS: [CommonFrontmatterRequiredStringField; 3] = [
    CommonFrontmatterRequiredStringField {
        key: "description",
        code: MarkdownSyntaxLintCode::MissingFrontmatterDescription,
        message: "frontmatter must include a non-empty `description` field",
    },
    CommonFrontmatterRequiredStringField {
        key: "author",
        code: MarkdownSyntaxLintCode::MissingFrontmatterAuthor,
        message: "frontmatter must include a non-empty `author` field",
    },
    CommonFrontmatterRequiredStringField {
        key: "date",
        code: MarkdownSyntaxLintCode::MissingFrontmatterDate,
        message: "frontmatter must include a minute-precision `date` field",
    },
];

pub(super) fn analyze_frontmatter<'a>(
    path: Option<&Path>,
    content: &'a str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> LintBody<'a> {
    let block = match scan_frontmatter(content, issues) {
        FrontmatterScan::Found(block) => block,
        FrontmatterScan::Missing => {
            issues.push(MarkdownSyntaxLintIssue {
                code: MarkdownSyntaxLintCode::MissingFrontmatter,
                message: "document must start with a YAML frontmatter block".to_string(),
                line: 1,
                column: 1,
            });
            return LintBody {
                content,
                line_offset: 1,
            };
        }
        FrontmatterScan::Unclosed => {
            return LintBody {
                content,
                line_offset: 1,
            };
        }
    };

    let requires_skill_frontmatter = uses_skill_frontmatter(path, content);
    match serde_yaml::from_str::<Value>(block.yaml) {
        Ok(value) => {
            if let Some(mapping) = value.as_mapping() {
                lint_common_frontmatter_schema(
                    block.yaml,
                    block.yaml_line_offset,
                    Some(mapping),
                    issues,
                );
                if requires_skill_frontmatter {
                    super::skill_frontmatter::lint_skill_frontmatter_schema(
                        block.yaml,
                        block.yaml_line_offset,
                        mapping,
                        issues,
                    );
                }
            } else {
                lint_common_frontmatter_schema(block.yaml, block.yaml_line_offset, None, issues);
                if requires_skill_frontmatter {
                    issues.push(MarkdownSyntaxLintIssue {
                        code: MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema,
                        message: "skill frontmatter must be a YAML mapping".to_string(),
                        line: block.yaml_line_offset,
                        column: 1,
                    });
                }
            }
        }
        Err(error) => {
            let (line, column) = error
                .location()
                .map_or((block.yaml_line_offset, 1), |location| {
                    (
                        block.yaml_line_offset + location.line().saturating_sub(1),
                        location.column().max(1),
                    )
                });
            issues.push(MarkdownSyntaxLintIssue {
                code: MarkdownSyntaxLintCode::InvalidFrontmatterYaml,
                message: format!("frontmatter is not valid YAML: {error}"),
                line,
                column,
            });
        }
    }

    LintBody {
        content: block.body,
        line_offset: block.body_line_offset,
    }
}

fn lint_common_frontmatter_schema(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: Option<&serde_yaml::Mapping>,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    for field in COMMON_FRONTMATTER_IDENTITY_FIELDS {
        let is_missing = mapping
            .and_then(|mapping| frontmatter_string(mapping, field.key))
            .is_none();
        if is_missing {
            issues.push(MarkdownSyntaxLintIssue {
                code: field.code,
                message: field.message.to_string(),
                line: frontmatter_key_line(yaml, yaml_line_offset, field.key).unwrap_or(1),
                column: 1,
            });
        }
    }
    lint_common_frontmatter_tags(yaml, yaml_line_offset, mapping, issues);
    for field in COMMON_FRONTMATTER_PROVENANCE_FIELDS {
        let value = mapping.and_then(|mapping| frontmatter_string(mapping, field.key));
        if value.is_none() {
            issues.push(MarkdownSyntaxLintIssue {
                code: field.code,
                message: field.message.to_string(),
                line: frontmatter_key_line(yaml, yaml_line_offset, field.key).unwrap_or(1),
                column: 1,
            });
        }
    }
    lint_common_frontmatter_date_precision(yaml, yaml_line_offset, mapping, issues);
    lint_common_frontmatter_retrieval(yaml, yaml_line_offset, mapping, issues);
}

fn lint_common_frontmatter_tags(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: Option<&serde_yaml::Mapping>,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let has_tags = mapping
        .and_then(|mapping| mapping.get(Value::String("tags".to_string())))
        .and_then(Value::as_sequence)
        .is_some_and(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .any(|value| !value.is_empty())
        });
    if !has_tags {
        issues.push(MarkdownSyntaxLintIssue {
            code: MarkdownSyntaxLintCode::MissingFrontmatterTags,
            message: "frontmatter must include a non-empty `tags` array".to_string(),
            line: frontmatter_key_line(yaml, yaml_line_offset, "tags").unwrap_or(1),
            column: 1,
        });
    }
}

fn lint_common_frontmatter_date_precision(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: Option<&serde_yaml::Mapping>,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let Some(date) = mapping.and_then(|mapping| frontmatter_string(mapping, "date")) else {
        return;
    };
    if !is_minute_precision_timestamp(date) {
        issues.push(MarkdownSyntaxLintIssue {
            code: MarkdownSyntaxLintCode::InvalidFrontmatterDatePrecision,
            message: "frontmatter `date` must use minute precision, e.g. `2026-04-26T09:30-07:00`"
                .to_string(),
            line: frontmatter_key_line(yaml, yaml_line_offset, "date").unwrap_or(1),
            column: 1,
        });
    }
}

fn is_minute_precision_timestamp(value: &str) -> bool {
    DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M%:z").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%MZ").is_ok()
}

fn lint_common_frontmatter_retrieval(
    yaml: &str,
    yaml_line_offset: usize,
    mapping: Option<&serde_yaml::Mapping>,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) {
    let retrieval = mapping
        .and_then(|mapping| mapping.get(Value::String("metadata".to_string())))
        .and_then(Value::as_mapping)
        .and_then(|metadata| metadata.get(Value::String("retrieval".to_string())))
        .and_then(Value::as_mapping);
    if retrieval
        .and_then(|retrieval| retrieval.get(Value::String("saliency_base".to_string())))
        .and_then(Value::as_f64)
        .is_none()
    {
        issues.push(MarkdownSyntaxLintIssue {
            code: MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase,
            message: "frontmatter must include numeric `metadata.retrieval.saliency_base`"
                .to_string(),
            line: frontmatter_key_line(yaml, yaml_line_offset, "saliency_base")
                .or_else(|| frontmatter_key_line(yaml, yaml_line_offset, "retrieval"))
                .or_else(|| frontmatter_key_line(yaml, yaml_line_offset, "metadata"))
                .unwrap_or(1),
            column: 1,
        });
    }
    if retrieval
        .and_then(|retrieval| retrieval.get(Value::String("decay_rate".to_string())))
        .and_then(Value::as_f64)
        .is_none()
    {
        issues.push(MarkdownSyntaxLintIssue {
            code: MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate,
            message: "frontmatter must include numeric `metadata.retrieval.decay_rate`".to_string(),
            line: frontmatter_key_line(yaml, yaml_line_offset, "decay_rate")
                .or_else(|| frontmatter_key_line(yaml, yaml_line_offset, "retrieval"))
                .or_else(|| frontmatter_key_line(yaml, yaml_line_offset, "metadata"))
                .unwrap_or(1),
            column: 1,
        });
    }
}

fn frontmatter_string<'a>(mapping: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    mapping
        .get(Value::String(key.to_string()))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn frontmatter_key_line(yaml: &str, yaml_line_offset: usize, key: &str) -> Option<usize> {
    yaml.lines()
        .position(|line| line.trim_start().starts_with(&format!("{key}:")))
        .map(|index| yaml_line_offset + index)
}

fn scan_frontmatter<'a>(
    content: &'a str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> FrontmatterScan<'a> {
    let Some(opening_len) = frontmatter_opening_len(content) else {
        return FrontmatterScan::Missing;
    };

    let remainder = &content[opening_len..];
    let mut offset = 0usize;
    let mut current_line = 2usize;

    while offset <= remainder.len() {
        let line_end = remainder[offset..]
            .find('\n')
            .map_or(remainder.len(), |index| offset + index);
        let next_offset = if line_end < remainder.len() {
            line_end + 1
        } else {
            line_end
        };
        let line = remainder[offset..line_end].trim_end_matches('\r');
        if line == "---" || line == "..." {
            let yaml = &remainder[..offset];
            let body = &remainder[next_offset..];
            return FrontmatterScan::Found(FrontmatterBlock {
                yaml,
                body,
                yaml_line_offset: 2,
                body_line_offset: current_line + 1,
            });
        }
        if next_offset == offset {
            break;
        }
        offset = next_offset;
        current_line += 1;
    }

    issues.push(MarkdownSyntaxLintIssue {
        code: MarkdownSyntaxLintCode::UnclosedFrontmatter,
        message: "frontmatter starts with `---` but has no closing `---` or `...` fence"
            .to_string(),
        line: 1,
        column: 1,
    });
    FrontmatterScan::Unclosed
}

fn frontmatter_opening_len(content: &str) -> Option<usize> {
    if content == "---" {
        return Some(content.len());
    }
    content
        .strip_prefix("---\n")
        .map(|remainder| content.len() - remainder.len())
        .or_else(|| {
            content
                .strip_prefix("---\r\n")
                .map(|remainder| content.len() - remainder.len())
        })
}
