use super::types::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};
use crate::frontmatter::{
    skill_frontmatter_has_metadata_mapping, skill_frontmatter_name, uses_skill_frontmatter,
};
use regex::Regex;
use serde_yaml::Value;
use std::path::Path;
use std::sync::LazyLock;

static FRONTMATTER_OPENING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| match Regex::new(r"\A---(?:\r?\n|\z)") {
        Ok(regex) => regex,
        Err(error) => panic!("hardcoded frontmatter regex should compile: {error}"),
    });

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
            if requires_skill_frontmatter {
                if skill_frontmatter_name(content).is_none() {
                    issues.push(MarkdownSyntaxLintIssue {
                        code: MarkdownSyntaxLintCode::MissingSkillFrontmatterName,
                        message:
                            "skill frontmatter must include a non-empty top-level `name` field"
                                .to_string(),
                        line: 1,
                        column: 1,
                    });
                }
                if !skill_frontmatter_has_metadata_mapping(content) {
                    issues.push(MarkdownSyntaxLintIssue {
                        code: MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata,
                        message: "skill frontmatter must contain a top-level `metadata` mapping"
                            .to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            } else if frontmatter_title(value.as_mapping()).is_none() {
                issues.push(MarkdownSyntaxLintIssue {
                    code: MarkdownSyntaxLintCode::MissingFrontmatterTitle,
                    message: "frontmatter must include a non-empty `title` field".to_string(),
                    line: 1,
                    column: 1,
                });
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

fn frontmatter_title(mapping: Option<&serde_yaml::Mapping>) -> Option<&str> {
    mapping
        .and_then(|mapping| mapping.get(Value::String("title".to_string())))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
}

fn scan_frontmatter<'a>(
    content: &'a str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> FrontmatterScan<'a> {
    if !FRONTMATTER_OPENING_REGEX.is_match(content) {
        return FrontmatterScan::Missing;
    }

    let opening_len = content.find('\n').map_or(content.len(), |index| index + 1);
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
