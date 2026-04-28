use super::{MarkdownLintIssue, MarkdownLintReport};
use anyhow::{Context, Result};
use ariadne::{CharSet, Config, IndexType, Label, Report, ReportKind, Source};
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub(super) fn render_markdown_lint_text_report(
    report: &MarkdownLintReport,
    source_contents: &BTreeMap<String, String>,
) -> Result<String> {
    if report.issue_count == 0 {
        return Ok(format!(
            "Markdown lint passed: checked {} file(s), 0 issue(s).\n",
            report.checked_files
        ));
    }

    let mut rendered = String::new();
    let _ = writeln!(
        rendered,
        "Markdown lint found {} issue(s) in {} file(s) across {} checked file(s).",
        report.issue_count, report.files_with_issues, report.checked_files
    );
    for file in &report.files {
        let contents = source_contents.get(file.path.as_str());
        for issue in &file.issues {
            rendered.push('\n');
            match contents {
                Some(contents) => {
                    rendered.push_str(&render_ariadne_issue(file.path.as_str(), issue, contents)?);
                }
                None => append_fallback_diagnostic(&mut rendered, file.path.as_str(), issue),
            }
        }
    }
    Ok(rendered)
}

fn render_ariadne_issue(path: &str, issue: &MarkdownLintIssue, contents: &str) -> Result<String> {
    let span = issue_span(issue, contents);
    let mut output = Vec::new();

    let mut report = Report::build(ReportKind::Error, (path.to_string(), span.clone()))
        .with_config(
            Config::new()
                .with_color(false)
                .with_compact(true)
                .with_char_set(CharSet::Ascii)
                .with_index_type(IndexType::Byte),
        )
        .with_code(issue.code.as_str())
        .with_message(issue.problem.as_str())
        .with_label(Label::new((path.to_string(), span)).with_message(issue.message.as_str()));

    for note in issue_notes(issue) {
        report = report.with_note(note);
    }
    for help in issue_helps(issue) {
        report = report.with_help(help);
    }

    report
        .finish()
        .write((path.to_string(), Source::from(contents)), &mut output)
        .map_err(|error| anyhow::anyhow!("failed to render markdown lint diagnostic: {error}"))?;

    String::from_utf8(output)
        .with_context(|| format!("markdown lint diagnostic for `{path}` was not UTF-8"))
}

fn issue_span(issue: &MarkdownLintIssue, contents: &str) -> std::ops::Range<usize> {
    let line_start = line_start_byte(contents, issue.line).unwrap_or(0);
    let line = contents
        .lines()
        .nth(issue.line.saturating_sub(1))
        .unwrap_or("");
    let column_offset = column_byte_offset(line, issue.column);
    let start = line_start.saturating_add(column_offset).min(contents.len());
    let width = issue
        .found
        .as_deref()
        .map(str::len)
        .filter(|width| *width > 0)
        .unwrap_or(1);
    let end = start
        .saturating_add(width)
        .min(contents.len().max(start + 1));
    start..end
}

fn line_start_byte(contents: &str, line: usize) -> Option<usize> {
    if line <= 1 {
        return Some(0);
    }

    let mut current_line = 1;
    for (index, character) in contents.char_indices() {
        if character == '\n' {
            current_line += 1;
            if current_line == line {
                return Some(index + character.len_utf8());
            }
        }
    }
    None
}

fn column_byte_offset(line: &str, column: usize) -> usize {
    let target_column = column.saturating_sub(1);
    line.char_indices()
        .nth(target_column)
        .map_or_else(|| line.len(), |(index, _character)| index)
}

fn issue_notes(issue: &MarkdownLintIssue) -> Vec<String> {
    let mut notes = Vec::new();
    notes.push(format!("kind: {}", issue.kind));
    notes.push(format!("problem: {}", issue.problem));
    if let Some(target) = &issue.target {
        notes.push(format!("target: {target}"));
    }
    if let Some(target_title) = &issue.target_title {
        notes.push(format!("target_title: {target_title}"));
    }
    if let Some(target_heading) = &issue.target_heading {
        notes.push(format!("target_heading: {target_heading}"));
    }
    if let Some(found) = &issue.found {
        notes.push(format!("found: {found}"));
    }
    notes.push(format!("detail: {}", issue.message));
    notes
}

fn issue_helps(issue: &MarkdownLintIssue) -> Vec<String> {
    let mut helps = Vec::new();
    if let Some(expected) = &issue.expected {
        helps.push(format!("expected: {expected}"));
    }
    if let Some(tip) = &issue.tip {
        helps.push(format!("tip: {tip}"));
    }
    helps
}

fn append_fallback_diagnostic(rendered: &mut String, path: &str, issue: &MarkdownLintIssue) {
    let _ = writeln!(rendered, "[{}] Error: {}", issue.code, issue.problem);
    let _ = writeln!(rendered, "--> {}:{}:{}", path, issue.line, issue.column);
    let _ = writeln!(rendered, "Label: {}", issue.message);
    for note in issue_notes(issue) {
        let _ = writeln!(rendered, "Note: {note}");
    }
    for help in issue_helps(issue) {
        let _ = writeln!(rendered, "Help: {help}");
    }
}
