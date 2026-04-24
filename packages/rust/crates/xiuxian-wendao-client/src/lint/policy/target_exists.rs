use crate::lint::MarkdownLintIssue;
use crate::lint::contract::diagnostic_contract;
use crate::lint::diagnostic::DiagnosticFacts;
use crate::lint::diagnostic::{
    DiagnosticContext, LocalTargetResolution, LocalTargetScopeViolation,
    LocalTargetTransientViolation,
};
use std::path::Path;
use xiuxian_wendao_parsers::{extract_targets, split_frontmatter};

pub(crate) fn lint_local_target_existence(
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
            let path_target = split_target_path(occurrence.target.as_str());
            if !should_validate_local_target(path_target) {
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
            match diagnostics.inspect_local_target(source_path, path_target, occurrence.kind) {
                LocalTargetResolution::Resolved(resolved_path) => diagnostics
                    .inspect_local_target_transient_dir(resolved_path.as_path())
                    .map(|offending_dir| {
                        diagnostic_contract().render_issue(
                            &DiagnosticFacts::local_target_transient_dir(
                                relative_path,
                                line,
                                column,
                                source,
                                literal,
                                occurrence.target.as_str(),
                                LocalTargetTransientViolation {
                                    resolved_path: resolved_path.as_path(),
                                    lint_root: diagnostics.root(),
                                    offending_dir,
                                },
                            ),
                        )
                    }),
                LocalTargetResolution::Missing => Some(diagnostic_contract().render_issue(
                    &DiagnosticFacts::missing_local_target(
                        relative_path,
                        line,
                        column,
                        source,
                        literal,
                        occurrence.target.as_str(),
                    ),
                )),
                LocalTargetResolution::OutsideRoot(resolved_path) => {
                    Some(diagnostic_contract().render_issue(
                        &DiagnosticFacts::local_target_outside_root(
                            relative_path,
                            line,
                            column,
                            source,
                            literal,
                            occurrence.target.as_str(),
                            LocalTargetScopeViolation {
                                resolved_path: resolved_path.as_path(),
                                lint_root: diagnostics.root(),
                            },
                        ),
                    ))
                }
            }
        })
        .collect()
}

fn split_target_path(raw_target: &str) -> &str {
    raw_target
        .split_once('#')
        .map_or(raw_target, |(path, _heading)| path)
        .trim()
}

fn should_validate_local_target(raw_target: &str) -> bool {
    let trimmed = raw_target
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/');
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("id:")
        || trimmed.starts_with("obsidian://")
        || trimmed.starts_with("wendao://")
    {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    !(lower.starts_with("http:")
        || lower.starts_with("https:")
        || lower.starts_with("mailto:")
        || lower.starts_with("file:")
        || lower.starts_with("data:")
        || lower.starts_with("tel:")
        || lower.contains("://"))
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
