use std::io;
use std::path::Path;

use std::fmt::Write as _;
use xiuxian_qianji::{WorkflowPlan, WorkflowPlanValidationReport, validate_workflow_plan};

use super::command::LintCliOutput;
use super::command::LintOutputFormat;
use crate::json_output::{CliJsonEnvelope, render_cli_json};

pub(super) fn run_workflow_plan_lint(
    source_path: &Path,
    resolved_path: &Path,
    contents: &str,
    format: LintOutputFormat,
) -> io::Result<LintCliOutput> {
    let source_id = source_path.display().to_string();
    match parse_workflow_plan(contents) {
        Ok(plan) => {
            let report = validate_workflow_plan(&plan);
            match format {
                LintOutputFormat::Json => {
                    render_workflow_plan_lint_json_output(&report, &source_id, resolved_path)
                }
                LintOutputFormat::Llm => Ok(render_workflow_plan_lint_llm_output(
                    &report,
                    &source_id,
                    resolved_path,
                )),
            }
        }
        Err(error) => run_workflow_plan_parse_lint(&source_id, resolved_path, &error, format),
    }
}

pub(super) fn run_workflow_plan_parse_lint(
    source_id: &str,
    resolved_path: &Path,
    error: &serde_json::Error,
    format: LintOutputFormat,
) -> io::Result<LintCliOutput> {
    match format {
        LintOutputFormat::Json => {
            render_workflow_plan_parse_error_json(source_id, resolved_path, error)
        }
        LintOutputFormat::Llm => Ok(render_workflow_plan_parse_error_llm(
            source_id,
            resolved_path,
            error,
        )),
    }
}

fn parse_workflow_plan(contents: &str) -> serde_json::Result<WorkflowPlan> {
    serde_json::from_str(contents)
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

fn render_workflow_plan_lint_llm_output(
    report: &WorkflowPlanValidationReport,
    source_id: &str,
    resolved_path: &Path,
) -> LintCliOutput {
    if report.ok {
        return LintCliOutput {
            rendered: format!(
                "[ok] {} workflow-plan\nSource: {source_id}\nNo blocking issues found.\n",
                resolved_path.display(),
            ),
            exit_code: 0,
        };
    }

    let mut rendered = format!(
        "[lint:error] {} workflow-plan\nSource: {source_id}\nIssues: {}\n",
        resolved_path.display(),
        report.diagnostics.len(),
    );
    for diagnostic in &report.diagnostics {
        let _ = writeln!(
            rendered,
            "\n[error] {}\n{}\nPath: {}\nFix:\n- {}",
            diagnostic.code, diagnostic.message, diagnostic.path, diagnostic.repair,
        );
    }
    LintCliOutput {
        rendered,
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

fn render_workflow_plan_parse_error_llm(
    source_id: &str,
    resolved_path: &Path,
    error: &serde_json::Error,
) -> LintCliOutput {
    LintCliOutput {
        rendered: format!(
            "[lint:error] {} workflow-plan\nSource: {source_id}\nIssues: 1\n\n[error] construct_plan.invalid_json_shape\nfailed to parse WorkflowPlan JSON: {error}\nFix:\n- Emit one top-level WorkflowPlan object with numeric version, constructs, tasks, and edges.\n",
            resolved_path.display(),
        ),
        exit_code: 2,
    }
}
