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
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use xiuxian_wendao_parsers::{
    SemanticProjectionStaleness, SemanticValidationIssue, lint_markdown_syntax_with_path,
    load_semantic_repository, semantic_projection_source_revision, split_frontmatter_raw,
};
use xiuxian_wendao_sql::DataFusionLocalRelationEngine;
use xiuxian_wendao_sql::semantic_read_model::{
    SemanticSqlGuardEvidence, run_semantic_sql_projection_freshness_guard_with_engine,
};

#[derive(serde::Serialize)]
struct SemanticLintRootReport {
    root: PathBuf,
    object_count: usize,
    projection_count: usize,
    change_intent_count: usize,
    refreshed_projection_count: usize,
    issues: Vec<SemanticValidationIssue>,
    sql_guard: Option<SemanticLintSqlGuardReport>,
}

#[derive(serde::Serialize)]
struct SemanticLintReport {
    checked_roots: usize,
    object_count: usize,
    projection_count: usize,
    change_intent_count: usize,
    refreshed_projection_count: usize,
    issue_count: usize,
    sql_guard_issue_count: usize,
    roots: Vec<SemanticLintRootReport>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticLintSqlGuardReport {
    guard_id: String,
    semantic_object_id: String,
    status: String,
    failing_row_count: usize,
    message: String,
    local_relation_engine: Option<String>,
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
            let refreshed_projection_count = if args.refresh_projections {
                refresh_semantic_projection_sources(root)?
            } else {
                0
            };
            let repository = load_semantic_repository(root);
            let semantic_issue_count = repository.report.issues.len();
            let sql_guard = if args.semantic_sql_guard && semantic_issue_count == 0 {
                Some(semantic_sql_guard_report(&repository)?)
            } else {
                None
            };
            Ok(SemanticLintRootReport {
                root: display_semantic_root(root, context.root()),
                object_count: repository.objects.len(),
                projection_count: repository.projections.len(),
                change_intent_count: repository.change_intents.len(),
                refreshed_projection_count,
                issues: repository.report.issues,
                sql_guard,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let report = SemanticLintReport {
        checked_roots: root_reports.len(),
        object_count: root_reports.iter().map(|root| root.object_count).sum(),
        projection_count: root_reports.iter().map(|root| root.projection_count).sum(),
        change_intent_count: root_reports
            .iter()
            .map(|root| root.change_intent_count)
            .sum(),
        refreshed_projection_count: root_reports
            .iter()
            .map(|root| root.refreshed_projection_count)
            .sum(),
        issue_count: root_reports.iter().map(|root| root.issues.len()).sum(),
        sql_guard_issue_count: root_reports
            .iter()
            .filter_map(|root| root.sql_guard.as_ref())
            .map(|guard| guard.failing_row_count)
            .sum(),
        roots: root_reports,
    };

    emit_semantic_report(&report, context.output())?;
    Ok(
        if report.issue_count == 0 && report.sql_guard_issue_count == 0 {
            CommandOutcome::success()
        } else {
            CommandOutcome::failure(1)
        },
    )
}

fn refresh_semantic_projection_sources(root: &Path) -> Result<usize> {
    let repository = load_semantic_repository(root);
    ensure_projection_refreshable(&repository.report.issues).with_context(|| {
        format!(
            "cannot refresh semantic projections under `{}`",
            root.display()
        )
    })?;

    let mut refreshed_count = 0usize;
    for projection in &repository.projections {
        let Some(current_revision) = semantic_projection_source_revision(&repository, projection)
        else {
            continue;
        };
        if projection.source_revision.as_str() == current_revision.as_str()
            && projection.staleness == SemanticProjectionStaleness::Fresh
        {
            continue;
        }
        let projection_path = root.join(&projection.source_path);
        refresh_projection_file(projection_path.as_path(), current_revision.as_str())?;
        refreshed_count += 1;
    }
    Ok(refreshed_count)
}

fn ensure_projection_refreshable(issues: &[SemanticValidationIssue]) -> Result<()> {
    let blocking_issues = issues
        .iter()
        .filter(|issue| {
            !issue
                .message
                .starts_with("semantic projection source revision is stale:")
        })
        .collect::<Vec<_>>();
    if blocking_issues.is_empty() {
        return Ok(());
    }

    let rendered = blocking_issues
        .iter()
        .map(|issue| match &issue.path {
            Some(path) => format!("{}: {}", path.display(), issue.message),
            None => issue.message.clone(),
        })
        .collect::<Vec<_>>()
        .join("; ");
    bail!("semantic repository has non-refreshable issue(s): {rendered}")
}

fn refresh_projection_file(path: &Path, current_revision: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read semantic projection `{}`", path.display()))?;
    let parts = split_frontmatter_raw(&content).with_context(|| {
        format!(
            "semantic projection `{}` is missing frontmatter",
            path.display()
        )
    })?;
    let mut frontmatter =
        serde_yaml::from_str::<serde_yaml::Value>(parts.yaml).with_context(|| {
            format!(
                "failed to parse semantic projection frontmatter `{}`",
                path.display()
            )
        })?;
    update_projection_frontmatter(
        &mut frontmatter,
        current_revision,
        &SemanticProjectionStaleness::Fresh,
    )?;
    let rendered = render_projection_document(&frontmatter, parts.body)?;
    std::fs::write(path, rendered)
        .with_context(|| format!("failed to write semantic projection `{}`", path.display()))?;
    Ok(())
}

fn update_projection_frontmatter(
    frontmatter: &mut serde_yaml::Value,
    current_revision: &str,
    staleness: &SemanticProjectionStaleness,
) -> Result<()> {
    let Some(mapping) = frontmatter.as_mapping_mut() else {
        bail!("semantic projection frontmatter must be a YAML mapping");
    };
    mapping.insert(
        serde_yaml::Value::String("source_revision".to_string()),
        serde_yaml::Value::String(current_revision.to_string()),
    );
    mapping.insert(
        serde_yaml::Value::String("staleness".to_string()),
        serde_yaml::Value::String(semantic_projection_staleness_token(staleness).to_string()),
    );
    Ok(())
}

fn semantic_projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}

fn render_projection_document(frontmatter: &serde_yaml::Value, body: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(frontmatter)
        .context("failed to render semantic projection frontmatter")?;
    Ok(format!("---\n{}---\n\n{}", yaml.trim_start(), body.trim()))
}

fn semantic_sql_guard_report(
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
) -> Result<SemanticLintSqlGuardReport> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to create semantic SQL guard runtime")?;
    let query_engine = DataFusionLocalRelationEngine::new_with_information_schema();
    let evidence = runtime
        .block_on(run_semantic_sql_projection_freshness_guard_with_engine(
            repository,
            &query_engine,
        ))
        .map_err(anyhow::Error::msg)
        .context("failed to run semantic SQL projection freshness guard")?;
    Ok(semantic_sql_guard_report_from_evidence(&evidence))
}

fn semantic_sql_guard_report_from_evidence(
    evidence: &SemanticSqlGuardEvidence,
) -> SemanticLintSqlGuardReport {
    SemanticLintSqlGuardReport {
        guard_id: evidence.guard_id.clone(),
        semantic_object_id: evidence.semantic_object_id.clone(),
        status: evidence.status.as_str().to_string(),
        failing_row_count: evidence.failing_row_count,
        message: evidence.message.clone(),
        local_relation_engine: evidence.local_relation_engine.clone(),
    }
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
    if report.issue_count == 0 && report.sql_guard_issue_count == 0 {
        let mut rendered = format!(
            "Semantic lint passed: checked {} root(s), {} object(s), {} projection(s), {} change intent(s), 0 issue(s).\n",
            report.checked_roots,
            report.object_count,
            report.projection_count,
            report.change_intent_count
        );
        render_semantic_refresh_text(report, &mut rendered);
        render_semantic_sql_guard_text(report, &mut rendered);
        return rendered;
    }

    let mut rendered = format!(
        "Semantic lint found {} issue(s) and {} SQL guard issue(s) across {} root(s), {} object(s), {} projection(s), and {} change intent(s).\n",
        report.issue_count,
        report.sql_guard_issue_count,
        report.checked_roots,
        report.object_count,
        report.projection_count,
        report.change_intent_count
    );
    render_semantic_refresh_text(report, &mut rendered);
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
    render_semantic_sql_guard_text(report, &mut rendered);
    rendered
}

fn render_semantic_refresh_text(report: &SemanticLintReport, rendered: &mut String) {
    if report.refreshed_projection_count == 0 {
        return;
    }
    rendered.push_str("- Refreshed ");
    rendered.push_str(report.refreshed_projection_count.to_string().as_str());
    rendered.push_str(" semantic projection source revision(s).\n");
}

fn render_semantic_sql_guard_text(report: &SemanticLintReport, rendered: &mut String) {
    for root in &report.roots {
        let Some(guard) = &root.sql_guard else {
            continue;
        };
        rendered.push_str("- ");
        rendered.push_str(root.root.display().to_string().as_str());
        rendered.push_str(": SQL guard ");
        rendered.push_str(guard.guard_id.as_str());
        rendered.push(' ');
        rendered.push_str(guard.status.as_str());
        rendered.push_str(" (");
        rendered.push_str(guard.failing_row_count.to_string().as_str());
        rendered.push_str(" failing row(s))");
        if !guard.message.is_empty() {
            rendered.push_str(": ");
            rendered.push_str(guard.message.as_str());
        }
        rendered.push('\n');
    }
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
