//! Text and JSON rendering for semantic lint reports.

use super::SemanticLintReport;
use anyhow::{Context, Result};
use xiuxian_wendao_sql::semantic_read_model::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME,
};

pub(super) fn render_semantic_text_report(report: &SemanticLintReport) -> String {
    if report.issue_count == 0
        && report.projection_policy_issue_count == 0
        && report.sql_guard_issue_count == 0
    {
        return render_success_text_report(report);
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
    render_semantic_validation_issue_text(report, &mut rendered);
    render_semantic_sql_guard_text(report, &mut rendered);
    rendered
}

fn render_success_text_report(report: &SemanticLintReport) -> String {
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

fn render_semantic_validation_issue_text(report: &SemanticLintReport, rendered: &mut String) {
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

pub(super) fn render_semantic_json_report(
    report: &SemanticLintReport,
    pretty: bool,
) -> Result<String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    }
    .context("failed to serialize semantic lint report")?;
    Ok(format!("{rendered}\n"))
}
