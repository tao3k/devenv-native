use std::io;
use std::path::Path;

use xiuxian_qianji::{
    WorkflowPlan, WorkflowPlanValidationReport, render_workflow_plan_validation_report,
    validate_workflow_plan,
};

use super::command::LintCliOutput;
use crate::json_output::{CliJsonEnvelope, render_cli_json};

pub(super) fn run_workflow_plan_lint(
    source_path: &Path,
    resolved_path: &Path,
    contents: &str,
    json: bool,
) -> io::Result<LintCliOutput> {
    let source_id = source_path.display().to_string();
    match parse_workflow_plan(contents) {
        Ok(plan) => {
            let report = validate_workflow_plan(&plan);
            if json {
                render_workflow_plan_lint_json_output(&report, &source_id, resolved_path)
            } else {
                Ok(render_workflow_plan_lint_output(
                    &report,
                    &source_id,
                    resolved_path,
                ))
            }
        }
        Err(error) => run_workflow_plan_parse_lint(&source_id, resolved_path, &error, json),
    }
}

pub(super) fn run_workflow_plan_parse_lint(
    source_id: &str,
    resolved_path: &Path,
    error: &serde_json::Error,
    json: bool,
) -> io::Result<LintCliOutput> {
    if json {
        render_workflow_plan_parse_error_json(source_id, resolved_path, error)
    } else {
        Ok(render_workflow_plan_parse_error(
            source_id,
            resolved_path,
            error,
        ))
    }
}

fn parse_workflow_plan(contents: &str) -> serde_json::Result<WorkflowPlan> {
    serde_json::from_str(contents)
}

fn render_workflow_plan_lint_output(
    report: &WorkflowPlanValidationReport,
    source_id: &str,
    resolved_path: &Path,
) -> LintCliOutput {
    if report.ok {
        return LintCliOutput {
            rendered: format!(
                "# Lint Passed\n\nSource: {source_id}\nPath: {}\nDomain: workflow-plan\nStatus: no blocking issues found in the bounded lint contract.\n",
                resolved_path.display(),
            ),
            exit_code: 0,
        };
    }

    LintCliOutput {
        rendered: format!(
            "# Lint Failed\n\nSource: {source_id}\nPath: {}\nDomain: workflow-plan\nIssues: {}\n\n{}",
            resolved_path.display(),
            report.diagnostics.len(),
            render_workflow_plan_validation_report(report),
        ),
        exit_code: 2,
    }
}

fn render_workflow_plan_lint_json_output(
    report: &WorkflowPlanValidationReport,
    source_id: &str,
    resolved_path: &Path,
) -> io::Result<LintCliOutput> {
    let exit_code = if report.ok { 0 } else { 2 };
    let ok = report.ok;
    let report = serde_json::json!({
        "domain": "workflow_plan",
        "source_id": source_id,
        "ok": ok,
        "diagnostics": report.diagnostics,
    });
    let rendered = render_cli_json(CliJsonEnvelope {
        kind: "qianji_lint_report",
        command: "lint",
        domain: "workflow_plan",
        path: resolved_path,
        source_id,
        ok,
        exit_code,
        report,
        analysis: None,
    })?;
    Ok(LintCliOutput {
        rendered,
        exit_code,
    })
}

fn render_workflow_plan_parse_error(
    source_id: &str,
    resolved_path: &Path,
    error: &serde_json::Error,
) -> LintCliOutput {
    let rendered = format!(
        "# Lint Failed\n\nSource: {source_id}\nPath: {}\nDomain: workflow-plan\nIssues: 1\n\n",
        resolved_path.display(),
    );
    LintCliOutput {
        rendered: format!("{rendered}{}", workflow_plan_parse_error_body(error)),
        exit_code: 2,
    }
}

fn render_workflow_plan_parse_error_json(
    source_id: &str,
    resolved_path: &Path,
    error: &serde_json::Error,
) -> io::Result<LintCliOutput> {
    let report = serde_json::json!({
        "domain": "workflow_plan",
        "source_id": source_id,
        "ok": false,
        "diagnostics": [{
            "code": "construct_plan.invalid_json_shape",
            "severity": "error",
            "path": "$",
            "message": format!("failed to parse WorkflowPlan JSON: {error}"),
            "repair": "Emit one top-level WorkflowPlan object with numeric version, constructs, tasks, and edges.",
        }],
    });
    let rendered = render_cli_json(CliJsonEnvelope {
        kind: "qianji_lint_report",
        command: "lint",
        domain: "workflow_plan",
        path: resolved_path,
        source_id,
        ok: false,
        exit_code: 2,
        report,
        analysis: None,
    })?;
    Ok(LintCliOutput {
        rendered,
        exit_code: 2,
    })
}

fn workflow_plan_parse_error_body(error: &serde_json::Error) -> String {
    format!(
        "## [construct_plan.invalid_json_shape] WorkflowPlan JSON shape is invalid\nSeverity: error\nSummary: failed to parse WorkflowPlan JSON: {error}\n\n### Repair Guidance\n- Emit one top-level WorkflowPlan object, not a wrapper such as `plan`.\n- Use numeric `\"version\": 1`, not string `\"1\"`.\n- Use `constructs`, `tasks`, and `edges`; do not use `nodes` or BPMN element names as the IR shape.\n- Each task must use `construct`, not `type`, and the construct value must come from `qianji construct index`.\n- Treat `constructs` as a set: list each selected construct id once.\n\n### Minimal Shape\n```json\n{{\n  \"version\": 1,\n  \"name\": \"example-plan\",\n  \"constructs\": [\"service-task.agent\"],\n  \"tasks\": [\n    {{\"id\": \"Task_DoWork\", \"construct\": \"service-task.agent\", \"outputs\": [\"result\"]}}\n  ],\n  \"edges\": [\n    {{\"from\": \"start\", \"to\": \"Task_DoWork\"}},\n    {{\"from\": \"Task_DoWork\", \"to\": \"end\"}}\n  ]\n}}\n```\n"
    )
}
