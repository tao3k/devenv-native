use super::contract::diagnostic_contract;
use super::diagnostic::DiagnosticContext;
use super::discovery::{collect_markdown_files, display_path};
use super::policy::{collect_file_link_style_facts, lint_directory_link_style_policy};
use super::{MarkdownLintArgs, MarkdownLintFileReport, MarkdownLintIssue, MarkdownLintReport};
use crate::{ClientContext, CommandOutcome, OutputFormat};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use xiuxian_wendao_parsers::lint_markdown_syntax;

pub(crate) fn run_markdown_lint(
    args: &MarkdownLintArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let files = collect_markdown_files(context.root(), args)?;
    let mut report = MarkdownLintReport {
        checked_files: files.len(),
        ..MarkdownLintReport::default()
    };
    let mut diagnostics = DiagnosticContext::new(context.root());
    let mut file_reports = BTreeMap::<String, MarkdownLintFileReport>::new();
    let mut style_facts = Vec::new();

    for path in files {
        let relative_path = display_path(path.as_path(), context.root());
        let bytes = std::fs::read(path.as_path())
            .with_context(|| format!("failed to read markdown file `{}`", path.display()))?;
        let file_report = match String::from_utf8(bytes) {
            Ok(markdown) => {
                style_facts.push(collect_file_link_style_facts(
                    relative_path.as_str(),
                    &markdown,
                ));
                build_file_report(relative_path, path.as_path(), &markdown, &mut diagnostics)
            }
            Err(error) => MarkdownLintFileReport {
                path: relative_path,
                issue_count: 1,
                issues: vec![diagnostic_contract().render_issue(
                    &super::diagnostic::DiagnosticFacts::invalid_utf8(error.to_string()),
                )],
            },
        };
        if file_report.issue_count > 0 {
            file_reports.insert(file_report.path.clone(), file_report);
        }
    }

    for (path, issues) in lint_directory_link_style_policy(style_facts.as_slice()) {
        let file_report =
            file_reports
                .entry(path.clone())
                .or_insert_with(|| MarkdownLintFileReport {
                    path,
                    issue_count: 0,
                    issues: Vec::new(),
                });
        file_report.issue_count += issues.len();
        file_report.issues.extend(issues);
        file_report
            .issues
            .sort_by_key(|issue| (issue.line, issue.column, issue.code.clone()));
    }

    report.files = file_reports.into_values().collect();
    report.files_with_issues = report.files.len();
    report.issue_count = report.files.iter().map(|file| file.issue_count).sum();

    emit_report(&report, context.output())?;
    Ok(if report.issue_count == 0 {
        CommandOutcome::success()
    } else {
        CommandOutcome::failure(1)
    })
}

fn build_file_report(
    path: String,
    source_path: &std::path::Path,
    markdown: &str,
    diagnostics: &mut DiagnosticContext<'_>,
) -> MarkdownLintFileReport {
    let issues = lint_markdown_syntax(markdown)
        .issues
        .into_iter()
        .map(|issue| diagnostics.build_issue(source_path, markdown, issue))
        .collect::<Vec<_>>();
    MarkdownLintFileReport {
        path,
        issue_count: issues.len(),
        issues,
    }
}

fn emit_report(report: &MarkdownLintReport, output: OutputFormat) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_text_report(report),
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn render_text_report(report: &MarkdownLintReport) -> String {
    if report.issue_count == 0 {
        return format!(
            "Markdown lint passed: checked {} file(s), 0 issue(s).\n",
            report.checked_files
        );
    }

    let mut rendered = String::new();
    let _ = writeln!(
        rendered,
        "Markdown lint found {} issue(s) in {} file(s) across {} checked file(s).",
        report.issue_count, report.files_with_issues, report.checked_files
    );
    for file in &report.files {
        rendered.push('\n');
        rendered.push_str(file.path.as_str());
        rendered.push('\n');
        for issue in &file.issues {
            let _ = writeln!(rendered, "  - line {}, column {}", issue.line, issue.column);
            let _ = writeln!(rendered, "    rule: {}", issue.code);
            let _ = writeln!(rendered, "    kind: {}", issue.kind);
            let _ = writeln!(rendered, "    problem: {}", issue.problem);
            if let Some(target) = &issue.target {
                let _ = writeln!(rendered, "    target: {target}");
            }
            if let Some(target_title) = &issue.target_title {
                let _ = writeln!(rendered, "    target_title: {target_title}");
            }
            if let Some(target_heading) = &issue.target_heading {
                let _ = writeln!(rendered, "    target_heading: {target_heading}");
            }
            if let Some(found) = &issue.found {
                let _ = writeln!(rendered, "    found: {found}");
            }
            if let Some(expected) = &issue.expected {
                let _ = writeln!(rendered, "    expected: {expected}");
            }
            let _ = writeln!(rendered, "    detail: {}", issue.message);
            if let Some(tip) = &issue.tip {
                let _ = writeln!(rendered, "    tip: {tip}");
            }
            if let Some(source) = &issue.source {
                let _ = writeln!(rendered, "    source: {source}");
                rendered.push_str("            ");
                let pointer = pointer_line(issue, source);
                rendered.push_str(&pointer);
                rendered.push('\n');
            }
        }
    }
    rendered
}

fn render_json_report(report: &MarkdownLintReport, pretty: bool) -> Result<String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    }
    .context("failed to serialize markdown lint report")?;
    Ok(format!("{rendered}\n"))
}

fn pointer_line(issue: &MarkdownLintIssue, source: &str) -> String {
    let source_width = source.chars().count();
    let start = issue.column.saturating_sub(1).min(source_width);
    let width = issue
        .found
        .as_deref()
        .map(|value| value.chars().count())
        .filter(|width| *width > 0)
        .unwrap_or(1);
    format!(
        "{}{}",
        " ".repeat(start),
        "^".repeat(width.min(source_width.max(1)))
    )
}
