use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use xiuxian_wendao_parsers::{
    MarkdownReference, MarkdownReferenceKind, MarkdownTargetOccurrenceKind, parse_markdown_note,
    split_frontmatter,
};

use crate::lint::MarkdownLintIssue;
use crate::lint::contract::diagnostic_contract;
use crate::lint::diagnostic::{DiagnosticFacts, DynamicDiagnosticText};

const DIRECTORY_LINK_STYLE_MISMATCH: &str = "directory_link_style_mismatch";
const DIRECTORY_LINK_STYLE_AMBIGUOUS: &str = "directory_link_style_ambiguous";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum DirectoryLinkStyle {
    Obsidian,
    Markdown,
}

impl DirectoryLinkStyle {
    fn display_name(self) -> &'static str {
        match self {
            Self::Obsidian => "explicit Obsidian wikilink",
            Self::Markdown => "standard Markdown note link",
        }
    }

    fn canonical_example(self) -> &'static str {
        match self {
            Self::Obsidian => "`[[target|label]]`",
            Self::Markdown => "`[label](target)`",
        }
    }
}

#[derive(Clone, Debug)]
struct LinkStyleOccurrence {
    style: DirectoryLinkStyle,
    target: String,
    label: Option<String>,
    literal: String,
    line: usize,
    column: usize,
    source: String,
}

#[derive(Clone, Debug)]
pub(crate) struct MarkdownFileLinkStyleFacts {
    pub(crate) path: String,
    directory: String,
    occurrences: Vec<LinkStyleOccurrence>,
}

#[derive(Clone, Debug, Default)]
struct DirectoryStyleSummary {
    files_per_style: BTreeMap<DirectoryLinkStyle, BTreeSet<String>>,
    occurrences_per_style: BTreeMap<DirectoryLinkStyle, usize>,
}

pub(crate) fn collect_file_link_style_facts(
    relative_path: &str,
    markdown: &str,
) -> MarkdownFileLinkStyleFacts {
    let directory = directory_path(relative_path);
    let fallback_title = Path::new(relative_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Markdown document");
    let note = parse_markdown_note(markdown, fallback_title);
    let (_frontmatter, body) = split_frontmatter(markdown);
    let body_line_offset = markdown[..markdown.len().saturating_sub(body.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();

    let mut references = note.core.references.iter();
    let mut occurrences = Vec::new();

    for target_occurrence in note.core.targets.iter().filter(|target| {
        matches!(
            target.kind,
            MarkdownTargetOccurrenceKind::MarkdownLink | MarkdownTargetOccurrenceKind::WikiLink
        )
    }) {
        let Some(reference) = references.next() else {
            break;
        };
        let Some(style) = classify_note_link_style(reference) else {
            continue;
        };
        let target = render_reference_target(reference);
        let label = reference_display_label(reference);
        let line = body_line_offset + target_occurrence.line_range.0;
        let column = byte_offset_to_column(body, target_occurrence.byte_range.0);
        let source = source_line_at(markdown, line).unwrap_or_default();

        occurrences.push(LinkStyleOccurrence {
            style,
            target,
            label,
            literal: reference.original.clone(),
            line,
            column,
            source,
        });
    }

    MarkdownFileLinkStyleFacts {
        path: relative_path.to_string(),
        directory,
        occurrences,
    }
}

pub(crate) fn lint_directory_link_style_policy(
    files: &[MarkdownFileLinkStyleFacts],
) -> BTreeMap<String, Vec<MarkdownLintIssue>> {
    files_by_directory(files)
        .into_iter()
        .flat_map(|(directory, directory_files)| {
            directory_link_style_issues(directory, &directory_files).into_iter()
        })
        .fold(BTreeMap::new(), |mut issues_by_file, (path, issue)| {
            issues_by_file
                .entry(path)
                .or_insert_with(Vec::new)
                .push(issue);
            issues_by_file
        })
}

fn files_by_directory(
    files: &[MarkdownFileLinkStyleFacts],
) -> BTreeMap<&str, Vec<&MarkdownFileLinkStyleFacts>> {
    files
        .iter()
        .fold(BTreeMap::new(), |mut by_directory, file| {
            by_directory
                .entry(file.directory.as_str())
                .or_insert_with(Vec::new)
                .push(file);
            by_directory
        })
}

fn directory_link_style_issues(
    directory: &str,
    directory_files: &[&MarkdownFileLinkStyleFacts],
) -> Vec<(String, MarkdownLintIssue)> {
    let summary = summarize_directory_styles(directory_files);
    if summary.files_per_style.len() < 2 {
        return Vec::new();
    }
    match preferred_directory_style(&summary) {
        Some(preferred_style) => {
            preferred_directory_style_issues(directory, directory_files, &summary, preferred_style)
        }
        None => ambiguous_directory_style_issues(directory, directory_files, &summary),
    }
}

fn preferred_directory_style_issues(
    directory: &str,
    directory_files: &[&MarkdownFileLinkStyleFacts],
    summary: &DirectoryStyleSummary,
    preferred_style: DirectoryLinkStyle,
) -> Vec<(String, MarkdownLintIssue)> {
    directory_files
        .iter()
        .filter_map(|file| {
            file.occurrences
                .iter()
                .find(|occurrence| occurrence.style != preferred_style)
                .map(|occurrence| {
                    (
                        file.path.clone(),
                        render_preferred_directory_style_issue(
                            directory,
                            summary,
                            preferred_style,
                            occurrence,
                        ),
                    )
                })
        })
        .collect()
}

fn ambiguous_directory_style_issues(
    directory: &str,
    directory_files: &[&MarkdownFileLinkStyleFacts],
    summary: &DirectoryStyleSummary,
) -> Vec<(String, MarkdownLintIssue)> {
    directory_files
        .iter()
        .filter_map(|file| {
            file.occurrences.first().map(|occurrence| {
                (
                    file.path.clone(),
                    render_ambiguous_directory_style_issue(directory, summary, occurrence),
                )
            })
        })
        .collect()
}

fn render_preferred_directory_style_issue(
    directory: &str,
    summary: &DirectoryStyleSummary,
    preferred_style: DirectoryLinkStyle,
    occurrence: &LinkStyleOccurrence,
) -> MarkdownLintIssue {
    diagnostic_contract().render_issue(&DiagnosticFacts::directory_link_style_policy(
        DIRECTORY_LINK_STYLE_MISMATCH.to_string(),
        occurrence.line,
        occurrence.column,
        Some(occurrence.source.clone()),
        DynamicDiagnosticText {
            problem: format!(
                "Directory `{directory}` mixes explicit Obsidian wikilinks and Markdown note links."
            ),
            detail: format!(
                "Directory `{directory}` already prefers {} style across {} file(s); this file still uses {} style. Keep one note-link style per directory so LLM repair stays deterministic.",
                preferred_style.display_name(),
                summary
                    .files_per_style
                    .get(&preferred_style)
                    .map_or(0, BTreeSet::len),
                occurrence.style.display_name(),
            ),
            found: Some(occurrence.literal.clone()),
            expected: Some(rewrite_for_directory_style(
                occurrence,
                preferred_style,
                directory,
            )),
            tip: Some(directory_style_tip(summary, preferred_style)),
        },
    ))
}

fn render_ambiguous_directory_style_issue(
    directory: &str,
    summary: &DirectoryStyleSummary,
    occurrence: &LinkStyleOccurrence,
) -> MarkdownLintIssue {
    diagnostic_contract().render_issue(&DiagnosticFacts::directory_link_style_policy(
        DIRECTORY_LINK_STYLE_AMBIGUOUS.to_string(),
        occurrence.line,
        occurrence.column,
        Some(occurrence.source.clone()),
        DynamicDiagnosticText {
            problem: format!(
                "Directory `{directory}` mixes explicit Obsidian wikilinks and Markdown note links without a clear local contract."
            ),
            detail: format!(
                "Files in `{directory}` currently split between {} and {} with no dominant local style. Pick one style for this directory and rewrite the outliers consistently instead of mixing both.",
                DirectoryLinkStyle::Obsidian.display_name(),
                DirectoryLinkStyle::Markdown.display_name(),
            ),
            found: Some(occurrence.literal.clone()),
            expected: Some(format!(
                "Choose either {} or {} for directory `{directory}`, then rewrite files consistently.",
                DirectoryLinkStyle::Obsidian.canonical_example(),
                DirectoryLinkStyle::Markdown.canonical_example(),
            )),
            tip: Some(ambiguous_directory_tip(summary)),
        },
    ))
}

fn summarize_directory_styles(files: &[&MarkdownFileLinkStyleFacts]) -> DirectoryStyleSummary {
    let mut summary = DirectoryStyleSummary::default();
    for file in files {
        let styles = file
            .occurrences
            .iter()
            .map(|occurrence| occurrence.style)
            .collect::<BTreeSet<_>>();
        for style in styles {
            summary
                .files_per_style
                .entry(style)
                .or_default()
                .insert(file.path.clone());
        }
        for occurrence in &file.occurrences {
            *summary
                .occurrences_per_style
                .entry(occurrence.style)
                .or_insert(0) += 1;
        }
    }
    summary
}

fn preferred_directory_style(summary: &DirectoryStyleSummary) -> Option<DirectoryLinkStyle> {
    let obsidian_files = summary
        .files_per_style
        .get(&DirectoryLinkStyle::Obsidian)
        .map_or(0, BTreeSet::len);
    let markdown_files = summary
        .files_per_style
        .get(&DirectoryLinkStyle::Markdown)
        .map_or(0, BTreeSet::len);
    match obsidian_files.cmp(&markdown_files) {
        std::cmp::Ordering::Greater => Some(DirectoryLinkStyle::Obsidian),
        std::cmp::Ordering::Less => Some(DirectoryLinkStyle::Markdown),
        std::cmp::Ordering::Equal => {
            let obsidian_occurrences = summary
                .occurrences_per_style
                .get(&DirectoryLinkStyle::Obsidian)
                .copied()
                .unwrap_or(0);
            let markdown_occurrences = summary
                .occurrences_per_style
                .get(&DirectoryLinkStyle::Markdown)
                .copied()
                .unwrap_or(0);
            match obsidian_occurrences.cmp(&markdown_occurrences) {
                std::cmp::Ordering::Greater => Some(DirectoryLinkStyle::Obsidian),
                std::cmp::Ordering::Less => Some(DirectoryLinkStyle::Markdown),
                std::cmp::Ordering::Equal => None,
            }
        }
    }
}

fn classify_note_link_style(reference: &MarkdownReference) -> Option<DirectoryLinkStyle> {
    match reference.kind {
        MarkdownReferenceKind::WikiLink => Some(DirectoryLinkStyle::Obsidian),
        MarkdownReferenceKind::Markdown => {
            is_note_like_markdown_reference(reference).then_some(DirectoryLinkStyle::Markdown)
        }
    }
}

fn render_reference_target(reference: &MarkdownReference) -> String {
    match (
        reference.addressed_target.target.as_deref(),
        reference.addressed_target.target_address.as_deref(),
    ) {
        (Some(target), Some(address)) => format!("{target}{address}"),
        (Some(target), None) => target.to_string(),
        (None, Some(address)) => address.to_string(),
        (None, None) => "target".to_string(),
    }
}

fn reference_display_label(reference: &MarkdownReference) -> Option<String> {
    match reference.kind {
        MarkdownReferenceKind::WikiLink => parse_wikilink_label(reference.original.as_str()),
        MarkdownReferenceKind::Markdown => parse_markdown_link_label(reference.original.as_str()),
    }
}

fn is_note_like_markdown_reference(reference: &MarkdownReference) -> bool {
    let Some(target) = reference.addressed_target.target.as_deref() else {
        return reference.addressed_target.target_address.is_some();
    };
    let trimmed = target.trim();
    if trimmed.is_empty() || looks_like_external_target(trimmed) {
        return false;
    }
    let extension = Path::new(trimmed)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("md" | "markdown") => true,
        Some(_) => false,
        None => {
            reference.addressed_target.target_address.is_some()
                || trimmed.contains('/')
                || trimmed.starts_with('.')
                || trimmed.starts_with('/')
                || !trimmed.contains("://")
        }
    }
}

fn parse_wikilink_label(literal: &str) -> Option<String> {
    let trimmed = literal.trim().trim_start_matches('!');
    let inner = trimmed.strip_prefix("[[")?.strip_suffix("]]")?;
    let (_target, label) = inner.split_once('|')?;
    let label = label.trim();
    (!label.is_empty()).then(|| label.to_string())
}

fn parse_markdown_link_label(literal: &str) -> Option<String> {
    let trimmed = literal.trim();
    let open = trimmed.strip_prefix('[')?;
    let close = open.find("](")?;
    let label = open.get(..close)?;
    let label = label.trim();
    (!label.is_empty()).then(|| label.to_string())
}

fn looks_like_external_target(target: &str) -> bool {
    let lower = target.trim().to_ascii_lowercase();
    lower.starts_with("http:")
        || lower.starts_with("https:")
        || lower.starts_with("mailto:")
        || lower.starts_with("obsidian://")
        || lower.starts_with("file:")
        || lower.contains("://")
}

fn rewrite_for_directory_style(
    occurrence: &LinkStyleOccurrence,
    preferred_style: DirectoryLinkStyle,
    directory: &str,
) -> String {
    let label = occurrence
        .label
        .as_deref()
        .filter(|label| !label.trim().is_empty())
        .unwrap_or("descriptive label");
    match preferred_style {
        DirectoryLinkStyle::Obsidian => format!(
            "Rewrite note links in this file to `[[{}|{}]]` to match directory `{directory}`.",
            occurrence.target, label
        ),
        DirectoryLinkStyle::Markdown => format!(
            "Rewrite note links in this file to `[{}]({})` to match directory `{directory}`.",
            label, occurrence.target
        ),
    }
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

fn directory_path(relative_path: &str) -> String {
    Path::new(relative_path)
        .parent()
        .and_then(|path| {
            let rendered = path.to_string_lossy().replace('\\', "/");
            (!rendered.is_empty() && rendered != ".").then_some(rendered)
        })
        .unwrap_or_else(|| ".".to_string())
}

fn directory_style_tip(
    summary: &DirectoryStyleSummary,
    preferred_style: DirectoryLinkStyle,
) -> String {
    let supporting_files = summary
        .files_per_style
        .get(&preferred_style)
        .map(|files| files.iter().take(3).cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    format!(
        "Neighbor files already using {} style include: {}.",
        preferred_style.canonical_example(),
        supporting_files
    )
}

fn ambiguous_directory_tip(summary: &DirectoryStyleSummary) -> String {
    let obsidian_files = summary
        .files_per_style
        .get(&DirectoryLinkStyle::Obsidian)
        .map(|files| files.iter().take(3).cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    let markdown_files = summary
        .files_per_style
        .get(&DirectoryLinkStyle::Markdown)
        .map(|files| files.iter().take(3).cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    format!("Obsidian-style files: {obsidian_files}. Markdown-style files: {markdown_files}.")
}
