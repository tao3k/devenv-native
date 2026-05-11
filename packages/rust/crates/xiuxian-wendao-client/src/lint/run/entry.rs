//! Command runners that coordinate Markdown and semantic lint owners.

use crate::lint::contract::diagnostic_contract;
use crate::lint::diagnostic::{DiagnosticContext, DiagnosticFacts};
use crate::lint::discovery::{collect_markdown_files, display_path};
use crate::lint::lifecycle::{
    SemanticLifecyclePlanReport, apply_semantic_lifecycle_plan, semantic_lifecycle_plan_report,
};
use crate::lint::policy::{
    collect_file_link_style_facts, lint_directory_link_style_policy, lint_local_target_existence,
    lint_local_target_fragments,
};
use crate::lint::projection_policy::{
    SemanticProjectionFreshnessPolicyReport, semantic_projection_freshness_policy_report,
};
use crate::lint::text_output::render_markdown_lint_text_report;
use crate::lint::{MarkdownLintArgs, MarkdownLintFileReport, MarkdownLintReport, SemanticLintArgs};
use crate::{ClientContext, CommandOutcome, OutputFormat};
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use xiuxian_wendao_parsers::{
    SemanticProjectionRefreshPlanReport, SemanticProjectionStaleness, SemanticValidationIssue,
    lint_markdown_syntax_with_path, load_semantic_repository,
    semantic_projection_refresh_plan_report, semantic_projection_source_revision,
    split_frontmatter_raw,
};
use xiuxian_wendao_sql::DataFusionLocalRelationEngine;
use xiuxian_wendao_sql::semantic_read_model::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, SemanticReadModelRows, SemanticSqlGuardEvidence,
    build_semantic_read_model_rows, run_semantic_sql_projection_freshness_guard_with_engine,
};

#[derive(serde::Serialize)]
pub(crate) struct SemanticLintRootReport {
    pub(crate) root: PathBuf,
    pub(crate) object_count: usize,
    pub(crate) projection_count: usize,
    pub(crate) change_intent_count: usize,
    pub(crate) refreshed_projection_count: usize,
    pub(crate) applied_lifecycle_count: usize,
    pub(crate) issues: Vec<SemanticValidationIssue>,
    pub(crate) lifecycle_plan: Option<SemanticLifecyclePlanReport>,
    pub(crate) projection_refresh_plan: Option<SemanticProjectionRefreshPlanReport>,
    pub(crate) projection_policy: Option<SemanticProjectionFreshnessPolicyReport>,
    pub(crate) read_model_summary: Option<SemanticReadModelSummaryReport>,
    pub(crate) sql_guard: Option<SemanticLintSqlGuardReport>,
}

#[derive(serde::Serialize)]
pub(crate) struct SemanticLintReport {
    pub(crate) checked_roots: usize,
    pub(crate) object_count: usize,
    pub(crate) projection_count: usize,
    pub(crate) change_intent_count: usize,
    pub(crate) refreshed_projection_count: usize,
    pub(crate) applied_lifecycle_count: usize,
    pub(crate) projection_refresh_plan_count: usize,
    pub(crate) read_model_summary_count: usize,
    pub(crate) issue_count: usize,
    pub(crate) projection_policy_issue_count: usize,
    pub(crate) sql_guard_issue_count: usize,
    pub(crate) roots: Vec<SemanticLintRootReport>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticLintSqlGuardReport {
    pub(crate) guard_id: String,
    pub(crate) semantic_object_id: String,
    pub(crate) status: String,
    pub(crate) failing_row_count: usize,
    pub(crate) message: String,
    pub(crate) local_relation_engine: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticReadModelSummaryReport {
    pub(crate) status: String,
    pub(crate) advisory: bool,
    pub(crate) authority: String,
    pub(crate) object_row_count: usize,
    pub(crate) relation_row_count: usize,
    pub(crate) projection_state_row_count: usize,
    pub(crate) stale_projection_count: usize,
    pub(crate) registered_table_count: usize,
    pub(crate) registered_tables: Vec<String>,
    pub(crate) message: String,
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
                issues: vec![
                    diagnostic_contract()
                        .render_issue(&DiagnosticFacts::invalid_utf8(error.to_string())),
                ],
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
        .map(|root| semantic_lint_root_report(root, args, context.root()))
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
        applied_lifecycle_count: root_reports
            .iter()
            .map(|root| root.applied_lifecycle_count)
            .sum(),
        projection_refresh_plan_count: root_reports
            .iter()
            .filter_map(|root| root.projection_refresh_plan.as_ref())
            .map(|plan| plan.refreshable_projection_count)
            .sum(),
        read_model_summary_count: root_reports
            .iter()
            .filter(|root| root.read_model_summary.is_some())
            .count(),
        issue_count: root_reports.iter().map(|root| root.issues.len()).sum(),
        projection_policy_issue_count: root_reports
            .iter()
            .filter_map(|root| root.projection_policy.as_ref())
            .map(|policy| policy.failing_projection_count)
            .sum(),
        sql_guard_issue_count: root_reports
            .iter()
            .filter_map(|root| root.sql_guard.as_ref())
            .map(|guard| guard.failing_row_count)
            .sum(),
        roots: root_reports,
    };

    emit_semantic_report(&report, context.output())?;
    Ok(
        if report.issue_count == 0
            && report.projection_policy_issue_count == 0
            && report.sql_guard_issue_count == 0
        {
            CommandOutcome::success()
        } else {
            CommandOutcome::failure(1)
        },
    )
}

fn semantic_lint_root_report(
    root: &Path,
    args: &SemanticLintArgs,
    context_root: &Path,
) -> Result<SemanticLintRootReport> {
    let applied_lifecycle_count = if args.writeback.apply_lifecycle_plan {
        apply_semantic_lifecycle_plan(root)?
    } else {
        0
    };
    let refreshed_projection_count = if args.writeback.refresh_projections {
        refresh_semantic_projection_sources(root)?
    } else {
        0
    };
    let repository = load_semantic_repository(root);
    let semantic_issue_count = repository.report.issues.len();
    let lifecycle_plan = semantic_lifecycle_plan_for_lint(args, &repository, semantic_issue_count);
    let projection_refresh_plan = semantic_projection_refresh_plan_for_lint(args, &repository);
    let projection_policy =
        semantic_projection_policy_for_lint(args, &repository, semantic_issue_count);
    let read_model_summary =
        semantic_read_model_summary_for_lint(args, &repository, semantic_issue_count)?;
    let sql_guard = semantic_sql_guard_for_lint(args, &repository, semantic_issue_count)?;
    Ok(SemanticLintRootReport {
        root: display_semantic_root(root, context_root),
        object_count: repository.objects.len(),
        projection_count: repository.projections.len(),
        change_intent_count: repository.change_intents.len(),
        refreshed_projection_count,
        applied_lifecycle_count,
        issues: repository.report.issues,
        lifecycle_plan,
        projection_refresh_plan,
        projection_policy,
        read_model_summary,
        sql_guard,
    })
}

fn semantic_lifecycle_plan_for_lint(
    args: &SemanticLintArgs,
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
    semantic_issue_count: usize,
) -> Option<SemanticLifecyclePlanReport> {
    ((args.validation.lifecycle_plan || args.writeback.apply_lifecycle_plan)
        && semantic_issue_count == 0)
        .then(|| semantic_lifecycle_plan_report(repository))
}

fn semantic_projection_refresh_plan_for_lint(
    args: &SemanticLintArgs,
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
) -> Option<SemanticProjectionRefreshPlanReport> {
    (args.validation.projection.projection_refresh_plan
        && projection_refresh_plan_renderable(&repository.report.issues))
    .then(|| semantic_projection_refresh_plan_report(repository))
}

fn semantic_projection_policy_for_lint(
    args: &SemanticLintArgs,
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
    semantic_issue_count: usize,
) -> Option<SemanticProjectionFreshnessPolicyReport> {
    (args.validation.projection.require_fresh_projections && semantic_issue_count == 0)
        .then(|| semantic_projection_freshness_policy_report(repository))
}

fn semantic_sql_guard_for_lint(
    args: &SemanticLintArgs,
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
    semantic_issue_count: usize,
) -> Result<Option<SemanticLintSqlGuardReport>> {
    if args.validation.semantic_sql_guard && semantic_issue_count == 0 {
        return semantic_sql_guard_report(repository).map(Some);
    }
    Ok(None)
}

fn semantic_read_model_summary_for_lint(
    args: &SemanticLintArgs,
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
    semantic_issue_count: usize,
) -> Result<Option<SemanticReadModelSummaryReport>> {
    if args.validation.read_model_summary && semantic_issue_count == 0 {
        return semantic_read_model_summary_report(repository).map(Some);
    }
    Ok(None)
}

fn refresh_semantic_projection_sources(root: &Path) -> Result<usize> {
    let repository = load_semantic_repository(root);
    ensure_projection_refreshable(&repository.report.issues).with_context(|| {
        format!(
            "cannot refresh semantic projections under `{}`",
            root.display()
        )
    })?;

    repository
        .projections
        .iter()
        .filter_map(|projection| stale_projection_refresh(root, &repository, projection))
        .try_fold(0usize, |refreshed_count, refresh| {
            refresh_projection_file(refresh.path.as_path(), refresh.revision.as_str())
                .map(|()| refreshed_count + 1)
        })
}

struct ProjectionRefresh {
    path: std::path::PathBuf,
    revision: String,
}

fn stale_projection_refresh(
    root: &Path,
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
    projection: &xiuxian_wendao_parsers::semantic_ssot::SemanticProjection,
) -> Option<ProjectionRefresh> {
    let current_revision = semantic_projection_source_revision(repository, projection)?;
    let is_current = projection.source_revision.as_str() == current_revision.as_str()
        && projection.staleness == SemanticProjectionStaleness::Fresh;
    (!is_current).then(|| ProjectionRefresh {
        path: root.join(&projection.source_path),
        revision: current_revision,
    })
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

fn projection_refresh_plan_renderable(issues: &[SemanticValidationIssue]) -> bool {
    issues.iter().all(|issue| {
        issue
            .message
            .starts_with("semantic projection source revision is stale:")
    })
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
    let frontmatter = serde_yaml::from_str::<serde_yaml::Value>(parts.yaml).with_context(|| {
        format!(
            "failed to parse semantic projection frontmatter `{}`",
            path.display()
        )
    })?;
    ensure_projection_frontmatter_mapping(&frontmatter)?;
    let rendered = render_projection_document_with_updated_frontmatter(
        parts.yaml,
        parts.body,
        current_revision,
        semantic_projection_staleness_token(&SemanticProjectionStaleness::Fresh),
    );
    std::fs::write(path, rendered)
        .with_context(|| format!("failed to write semantic projection `{}`", path.display()))?;
    Ok(())
}

fn ensure_projection_frontmatter_mapping(frontmatter: &serde_yaml::Value) -> Result<()> {
    if !frontmatter.is_mapping() {
        bail!("semantic projection frontmatter must be a YAML mapping");
    }
    Ok(())
}

fn semantic_projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}

fn render_projection_document_with_updated_frontmatter(
    frontmatter: &str,
    body: &str,
    current_revision: &str,
    staleness: &str,
) -> String {
    let mut saw_source_revision = false;
    let mut saw_staleness = false;
    let mut rendered_frontmatter = frontmatter
        .lines()
        .map(|line| {
            if line.starts_with("source_revision:") {
                saw_source_revision = true;
                format!("source_revision: \"{current_revision}\"")
            } else if line.starts_with("staleness:") {
                saw_staleness = true;
                format!("staleness: {staleness}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if !saw_source_revision {
        rendered_frontmatter.push(format!("source_revision: \"{current_revision}\""));
    }
    if !saw_staleness {
        rendered_frontmatter.push(format!("staleness: {staleness}"));
    }
    format!(
        "---\n{}\n---\n\n{}\n",
        rendered_frontmatter.join("\n"),
        body.trim()
    )
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

fn semantic_read_model_summary_report(
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
) -> Result<SemanticReadModelSummaryReport> {
    let rows = build_semantic_read_model_rows(repository)
        .map_err(anyhow::Error::msg)
        .context("failed to build advisory semantic read-model rows")?;
    Ok(semantic_read_model_summary_report_from_rows(&rows))
}

fn semantic_read_model_summary_report_from_rows(
    rows: &SemanticReadModelRows,
) -> SemanticReadModelSummaryReport {
    let registered_tables = semantic_read_model_table_names();
    SemanticReadModelSummaryReport {
        status: "projected".to_string(),
        advisory: true,
        authority: "repo_native_semantic_artifacts".to_string(),
        object_row_count: rows.objects.len(),
        relation_row_count: rows.relations.len(),
        projection_state_row_count: rows.projection_state.len(),
        stale_projection_count: rows
            .projection_state
            .iter()
            .filter(|row| row.staleness == "stale")
            .count(),
        registered_table_count: registered_tables.len(),
        registered_tables,
        message: "semantic read-model rows are advisory; repo-native semantic artifacts remain authoritative"
            .to_string(),
    }
}

fn semantic_read_model_table_names() -> Vec<String> {
    vec![
        SEMANTIC_OBJECTS_TABLE_NAME.to_string(),
        SEMANTIC_RELATIONS_TABLE_NAME.to_string(),
        SEMANTIC_PROJECTION_STATE_TABLE_NAME.to_string(),
    ]
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
        OutputFormat::Text => super::semantic_render::render_semantic_text_report(report),
        OutputFormat::Json => super::semantic_render::render_semantic_json_report(report, false)?,
        OutputFormat::Pretty => super::semantic_render::render_semantic_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}
