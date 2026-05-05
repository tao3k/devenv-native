use super::contract::diagnostic_contract;
use super::diagnostic::DiagnosticContext;
use super::discovery::{collect_markdown_files, display_path};
use super::policy::{
    collect_file_link_style_facts, lint_directory_link_style_policy, lint_local_target_existence,
    lint_local_target_fragments,
};
use super::text_output::render_markdown_lint_text_report;
use super::{MarkdownLintArgs, MarkdownLintFileReport, MarkdownLintReport, SemanticLintArgs};
use crate::{ClientContext, CommandOutcome, OutputFormat};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use xiuxian_wendao_parsers::{
    SemanticValidationIssue, lint_markdown_syntax_with_path, load_semantic_repository,
};

#[derive(serde::Serialize)]
struct SemanticLintRootReport {
    root: PathBuf,
    object_count: usize,
    projection_count: usize,
    issues: Vec<SemanticValidationIssue>,
}

#[derive(serde::Serialize)]
struct SemanticLintReport {
    checked_roots: usize,
    object_count: usize,
    projection_count: usize,
    issue_count: usize,
    roots: Vec<SemanticLintRootReport>,
}

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
    let mut source_contents = BTreeMap::<String, String>::new();
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
                let file_report = build_file_report(
                    relative_path.clone(),
                    path.as_path(),
                    &markdown,
                    &mut diagnostics,
                );
                source_contents.insert(relative_path, markdown);
                file_report
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

    emit_report(&report, &source_contents, context.output())?;
    Ok(if report.issue_count == 0 {
        CommandOutcome::success()
    } else {
        CommandOutcome::failure(1)
    })
}

pub(crate) fn run_semantic_lint(
    args: &SemanticLintArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let roots = semantic_lint_roots(args, context.root());
    let root_reports = roots
        .iter()
        .map(|root| {
            let repository = load_semantic_repository(root);
            SemanticLintRootReport {
                root: display_semantic_root(root, context.root()),
                object_count: repository.objects.len(),
                projection_count: repository.projections.len(),
                issues: repository.report.issues,
            }
        })
        .collect::<Vec<_>>();
    let report = SemanticLintReport {
        checked_roots: root_reports.len(),
        object_count: root_reports.iter().map(|root| root.object_count).sum(),
        projection_count: root_reports.iter().map(|root| root.projection_count).sum(),
        issue_count: root_reports.iter().map(|root| root.issues.len()).sum(),
        roots: root_reports,
    };

    emit_semantic_report(&report, context.output())?;
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
    let mut issues = lint_markdown_syntax_with_path(Some(source_path), markdown)
        .issues
        .into_iter()
        .map(|issue| diagnostics.build_issue(source_path, markdown, issue))
        .collect::<Vec<_>>();
    issues.extend(lint_local_target_existence(
        path.as_str(),
        source_path,
        markdown,
        diagnostics,
    ));
    issues.extend(lint_local_target_fragments(
        path.as_str(),
        source_path,
        markdown,
        diagnostics,
    ));
    issues.sort_by_key(|issue| (issue.line, issue.column, issue.code.clone()));
    MarkdownLintFileReport {
        path,
        issue_count: issues.len(),
        issues,
    }
}

fn emit_report(
    report: &MarkdownLintReport,
    source_contents: &BTreeMap<String, String>,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_markdown_lint_text_report(report, source_contents)?,
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
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

fn semantic_lint_roots(args: &SemanticLintArgs, context_root: &Path) -> Vec<PathBuf> {
    if args.paths.is_empty() {
        return vec![context_root.join("semantic")];
    }
    args.paths
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                context_root.join(path)
            }
        })
        .collect()
}

fn display_semantic_root(root: &Path, context_root: &Path) -> PathBuf {
    root.strip_prefix(context_root)
        .map_or_else(|_| root.to_path_buf(), std::path::Path::to_path_buf)
}

fn emit_semantic_report(report: &SemanticLintReport, output: OutputFormat) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_semantic_text_report(report),
        OutputFormat::Json => render_semantic_json_report(report, false)?,
        OutputFormat::Pretty => render_semantic_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn render_semantic_text_report(report: &SemanticLintReport) -> String {
    if report.issue_count == 0 {
        return format!(
            "Semantic lint passed: checked {} root(s), {} object(s), {} projection(s), 0 issue(s).\n",
            report.checked_roots, report.object_count, report.projection_count
        );
    }

    let mut rendered = format!(
        "Semantic lint found {} issue(s) across {} root(s), {} object(s), and {} projection(s).\n",
        report.issue_count, report.checked_roots, report.object_count, report.projection_count
    );
    for root in &report.roots {
        for issue in &root.issues {
            let path = issue.path.as_ref().map_or_else(
                || root.root.display().to_string(),
                |path| root.root.join(path).display().to_string(),
            );
            rendered.push_str("- ");
            rendered.push_str(path.as_str());
            rendered.push_str(": ");
            rendered.push_str(issue.message.as_str());
            rendered.push('\n');
        }
    }
    rendered
}

fn render_semantic_json_report(report: &SemanticLintReport, pretty: bool) -> Result<String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    }
    .context("failed to serialize semantic lint report")?;
    Ok(format!("{rendered}\n"))
}
