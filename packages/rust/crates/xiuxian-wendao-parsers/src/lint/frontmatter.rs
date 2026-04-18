use super::types::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};
use regex::Regex;
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

pub(super) fn analyze_frontmatter<'a>(
    content: &'a str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> LintBody<'a> {
    let Some(block) = scan_frontmatter(content, issues) else {
        return LintBody {
            content,
            line_offset: 1,
        };
    };

    if let Err(error) = serde_yaml::from_str::<serde_yaml::Value>(block.yaml) {
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

    LintBody {
        content: block.body,
        line_offset: block.body_line_offset,
    }
}

fn scan_frontmatter<'a>(
    content: &'a str,
    issues: &mut Vec<MarkdownSyntaxLintIssue>,
) -> Option<FrontmatterBlock<'a>> {
    if !FRONTMATTER_OPENING_REGEX.is_match(content) {
        return None;
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
            return Some(FrontmatterBlock {
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
    None
}
