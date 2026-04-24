use crate::lint::MarkdownLintIssue;
use crate::lint::contract::diagnostic_contract;
use crate::lint::diagnostic::{
    DiagnosticContext, DiagnosticFacts, LocalTargetFragmentResolution, LocalTargetFragmentViolation,
};
use std::path::Path;
use xiuxian_wendao_parsers::{extract_targets, split_frontmatter};

pub(crate) fn lint_local_target_fragments(
    relative_path: &str,
    source_path: &Path,
    markdown: &str,
    diagnostics: &mut DiagnosticContext<'_>,
) -> Vec<MarkdownLintIssue> {
    let (_frontmatter, body) = split_frontmatter(markdown);
    let body_line_offset = markdown[..markdown.len().saturating_sub(body.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();

    extract_targets(body)
        .into_iter()
        .filter_map(|occurrence| {
            if !should_validate_local_fragment(occurrence.target.as_str()) {
                return None;
            }

            let line = body_line_offset + occurrence.line_range.0;
            let column = byte_offset_to_column(body, occurrence.byte_range.0);
            let source = source_line_at(markdown, line);
            let literal = if occurrence.surface.trim().is_empty() {
                occurrence.target.as_str()
            } else {
                occurrence.surface.as_str()
            };

            match diagnostics.inspect_local_target_fragment(
                source_path,
                markdown,
                occurrence.target.as_str(),
                occurrence.kind,
            ) {
                LocalTargetFragmentResolution::NotAddressed
                | LocalTargetFragmentResolution::Resolved
                | LocalTargetFragmentResolution::TransientDir
                | LocalTargetFragmentResolution::MissingTarget
                | LocalTargetFragmentResolution::OutsideRoot => None,
                LocalTargetFragmentResolution::MissingHeading {
                    fragment,
                    target_title,
                } => Some(diagnostic_contract().render_issue(
                    &DiagnosticFacts::missing_local_fragment(
                        relative_path,
                        line,
                        column,
                        source,
                        LocalTargetFragmentViolation {
                            literal,
                            raw_target: occurrence.target.as_str(),
                            fragment: fragment.as_str(),
                            is_block: false,
                            target_title,
                        },
                    ),
                )),
                LocalTargetFragmentResolution::MissingBlock {
                    fragment,
                    target_title,
                } => Some(diagnostic_contract().render_issue(
                    &DiagnosticFacts::missing_local_fragment(
                        relative_path,
                        line,
                        column,
                        source,
                        LocalTargetFragmentViolation {
                            literal,
                            raw_target: occurrence.target.as_str(),
                            fragment: fragment.as_str(),
                            is_block: true,
                            target_title,
                        },
                    ),
                )),
            }
        })
        .collect()
}

fn should_validate_local_fragment(raw_target: &str) -> bool {
    let trimmed = raw_target.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("id:")
        || trimmed.starts_with("obsidian://")
        || trimmed.starts_with("wendao://")
    {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http:")
        || lower.starts_with("https:")
        || lower.starts_with("mailto:")
        || lower.starts_with("file:")
        || lower.starts_with("data:")
        || lower.starts_with("tel:")
        || lower.contains("://")
    {
        return false;
    }

    trimmed.contains('#')
}

fn byte_offset_to_column(body: &str, byte_offset: usize) -> usize {
    let offset = byte_offset.min(body.len());
    let line_start = body[..offset].rfind('\n').map_or(0, |index| index + 1);
    body[line_start..offset].chars().count() + 1
}

fn source_line_at(markdown: &str, line: usize) -> Option<String> {
    markdown
        .lines()
        .nth(line.saturating_sub(1))
        .map(std::string::ToString::to_string)
}
