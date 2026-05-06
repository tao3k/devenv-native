use super::contract::diagnostic_contract;
use super::diagnostic::DiagnosticContext;
use super::discovery::{collect_markdown_files, display_path};
use super::lifecycle::{
    SemanticLifecyclePlanReport, apply_semantic_lifecycle_plan, semantic_lifecycle_plan_report,
};
use super::policy::{
    collect_file_link_style_facts, lint_directory_link_style_policy, lint_local_target_existence,
    lint_local_target_fragments,
};
use super::projection_policy::{
    SemanticProjectionFreshnessPolicyReport, semantic_projection_freshness_policy_report,
};
use super::text_output::render_markdown_lint_text_report;
use super::{MarkdownLintArgs, MarkdownLintFileReport, MarkdownLintReport, SemanticLintArgs};
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
struct SemanticLintRootReport {
    root: PathBuf,
    object_count: usize,
    projection_count: usize,
    change_intent_count: usize,
    refreshed_projection_count: usize,
    applied_lifecycle_count: usize,
    issues: Vec<SemanticValidationIssue>,
    lifecycle_plan: Option<SemanticLifecyclePlanReport>,
    projection_refresh_plan: Option<SemanticProjectionRefreshPlanReport>,
    projection_policy: Option<SemanticProjectionFreshnessPolicyReport>,
    read_model_summary: Option<SemanticReadModelSummaryReport>,
    sql_guard: Option<SemanticLintSqlGuardReport>,
}

#[derive(serde::Serialize)]
struct SemanticLintReport {
    checked_roots: usize,
    object_count: usize,
    projection_count: usize,
    change_intent_count: usize,
    refreshed_projection_count: usize,
    applied_lifecycle_count: usize,
    projection_refresh_plan_count: usize,
    read_model_summary_count: usize,
    issue_count: usize,
    projection_policy_issue_count: usize,
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelSummaryReport {
    status: String,
    advisory: bool,
    authority: String,
    object_row_count: usize,
    relation_row_count: usize,
    projection_state_row_count: usize,
    stale_projection_count: usize,
    registered_table_count: usize,
    registered_tables: Vec<String>,
    message: String,
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
        OutputFormat::Text => render_semantic_text_report(report),
        OutputFormat::Json => render_semantic_json_report(report, false)?,
        OutputFormat::Pretty => render_semantic_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn render_semantic_text_report(report: &SemanticLintReport) -> String {
    if report.issue_count == 0
        && report.projection_policy_issue_count == 0
        && report.sql_guard_issue_count == 0
    {
        let mut rendered = format!(
            "Semantic lint passed: checked {} root(s), {} object(s), {} projection(s), {} change intent(s), 0 issue(s).\n",
            report.checked_roots,
            report.object_count,
            report.projection_count,
            report.change_intent_count
        );
        render_semantic_lifecycle_apply_text(report, &mut rendered);
        render_semantic_refresh_text(report, &mut rendered);
        render_semantic_lifecycle_plan_text(report, &mut rendered);
        render_semantic_projection_refresh_plan_text(report, &mut rendered);
        render_semantic_projection_policy_text(report, &mut rendered);
        render_semantic_read_model_summary_text(report, &mut rendered);
        render_semantic_sql_guard_text(report, &mut rendered);
        return rendered;
    }

    let mut rendered = format!(
        "Semantic lint found {} issue(s), {} projection policy issue(s), and {} SQL guard issue(s) across {} root(s), {} object(s), {} projection(s), and {} change intent(s).\n",
        report.issue_count,
        report.projection_policy_issue_count,
        report.sql_guard_issue_count,
        report.checked_roots,
        report.object_count,
        report.projection_count,
        report.change_intent_count
    );
    render_semantic_lifecycle_apply_text(report, &mut rendered);
    render_semantic_refresh_text(report, &mut rendered);
    render_semantic_lifecycle_plan_text(report, &mut rendered);
    render_semantic_projection_refresh_plan_text(report, &mut rendered);
    render_semantic_projection_policy_text(report, &mut rendered);
    render_semantic_read_model_summary_text(report, &mut rendered);
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

fn render_semantic_lifecycle_apply_text(report: &SemanticLintReport, rendered: &mut String) {
    if report.applied_lifecycle_count == 0 {
        return;
    }
    rendered.push_str("- Applied ");
    rendered.push_str(report.applied_lifecycle_count.to_string().as_str());
    rendered.push_str(" semantic lifecycle writeback(s).\n");
}

fn render_semantic_lifecycle_plan_text(report: &SemanticLintReport, rendered: &mut String) {
    for root in &report.roots {
        let Some(plan) = &root.lifecycle_plan else {
            continue;
        };
        rendered.push_str("- ");
        rendered.push_str(root.root.display().to_string().as_str());
        rendered.push_str(": Lifecycle plan ");
        rendered.push_str(plan.promotion_count.to_string().as_str());
        rendered.push_str(" promotion(s), ");
        rendered.push_str(plan.demotion_count.to_string().as_str());
        rendered.push_str(" demotion(s), ");
        rendered.push_str(plan.other_transition_count.to_string().as_str());
        rendered.push_str(" other transition(s), ");
        rendered.push_str(plan.pending_apply_count.to_string().as_str());
        rendered.push_str(" pending apply target(s), ");
        rendered.push_str(plan.already_applied_count.to_string().as_str());
        rendered.push_str(" already-applied writeback target(s), ");
        rendered.push_str(plan.blocked_count.to_string().as_str());
        rendered.push_str(" blocked target(s).\n");
        for entry in &plan.entries {
            rendered.push_str("  - ");
            rendered.push_str(entry.change_intent_id.as_str());
            rendered.push_str(": ");
            rendered.push_str(entry.object_id.as_str());
            rendered.push(' ');
            rendered.push_str(entry.from.as_str());
            rendered.push_str(" -> ");
            rendered.push_str(entry.to.as_str());
            rendered.push_str(" (");
            rendered.push_str(entry.outcome.as_str());
            rendered.push_str(", ");
            rendered.push_str(entry.writeback_action.as_str());
            rendered.push_str(")\n");
        }
    }
}

fn render_semantic_refresh_text(report: &SemanticLintReport, rendered: &mut String) {
    if report.refreshed_projection_count == 0 {
        return;
    }
    rendered.push_str("- Refreshed ");
    rendered.push_str(report.refreshed_projection_count.to_string().as_str());
    rendered.push_str(" semantic projection source revision(s).\n");
}

fn render_semantic_projection_refresh_plan_text(
    report: &SemanticLintReport,
    rendered: &mut String,
) {
    for root in &report.roots {
        let Some(plan) = &root.projection_refresh_plan else {
            continue;
        };
        rendered.push_str("- ");
        rendered.push_str(root.root.display().to_string().as_str());
        rendered.push_str(": Projection refresh plan ");
        rendered.push_str(plan.status.as_str());
        rendered.push_str(" (");
        rendered.push_str(plan.refreshable_projection_count.to_string().as_str());
        rendered.push_str(" refreshable projection(s))");
        if !plan.message.is_empty() {
            rendered.push_str(": ");
            rendered.push_str(plan.message.as_str());
        }
        rendered.push('\n');
        for projection in &plan.projections {
            rendered.push_str("  - ");
            rendered.push_str(projection.projection.as_str());
            rendered.push_str(" -> ");
            rendered.push_str(projection.action.as_str());
            rendered.push_str(" (");
            rendered.push_str(projection.reason.as_str());
            rendered.push_str(", ");
            rendered.push_str(projection.staleness.as_str());
            rendered.push_str(")\n");
        }
    }
}

fn render_semantic_projection_policy_text(report: &SemanticLintReport, rendered: &mut String) {
    for root in &report.roots {
        let Some(policy) = &root.projection_policy else {
            continue;
        };
        rendered.push_str("- ");
        rendered.push_str(root.root.display().to_string().as_str());
        rendered.push_str(": Projection freshness policy ");
        rendered.push_str(policy.policy_id.as_str());
        rendered.push(' ');
        rendered.push_str(policy.status.as_str());
        rendered.push_str(" (");
        rendered.push_str(policy.failing_projection_count.to_string().as_str());
        rendered.push_str(" failing projection(s))");
        if !policy.message.is_empty() {
            rendered.push_str(": ");
            rendered.push_str(policy.message.as_str());
        }
        rendered.push('\n');
        for projection in &policy.projections {
            rendered.push_str("  - ");
            rendered.push_str(projection.projection.as_str());
            rendered.push_str(" (");
            rendered.push_str(projection.reason.as_str());
            rendered.push_str(", ");
            rendered.push_str(projection.staleness.as_str());
            rendered.push_str(")\n");
        }
    }
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

fn render_semantic_read_model_summary_text(report: &SemanticLintReport, rendered: &mut String) {
    for root in &report.roots {
        let Some(summary) = &root.read_model_summary else {
            continue;
        };
        rendered.push_str("- ");
        rendered.push_str(root.root.display().to_string().as_str());
        rendered.push_str(": Read-model summary ");
        rendered.push_str(summary.status.as_str());
        rendered.push_str(" (");
        rendered.push_str(SEMANTIC_OBJECTS_TABLE_NAME);
        rendered.push(' ');
        rendered.push_str(summary.object_row_count.to_string().as_str());
        rendered.push_str(" row(s), ");
        rendered.push_str(SEMANTIC_RELATIONS_TABLE_NAME);
        rendered.push(' ');
        rendered.push_str(summary.relation_row_count.to_string().as_str());
        rendered.push_str(" row(s), ");
        rendered.push_str(SEMANTIC_PROJECTION_STATE_TABLE_NAME);
        rendered.push(' ');
        rendered.push_str(summary.projection_state_row_count.to_string().as_str());
        rendered.push_str(" row(s), ");
        rendered.push_str(summary.stale_projection_count.to_string().as_str());
        rendered.push_str(" stale projection row(s))");
        if !summary.message.is_empty() {
            rendered.push_str(": ");
            rendered.push_str(summary.message.as_str());
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
