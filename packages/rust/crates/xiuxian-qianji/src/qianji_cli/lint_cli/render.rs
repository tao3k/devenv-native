use std::io;
use std::path::Path;

use xiuxian_qianji_bpmn_engine::{LintDomain, LintReport};

use super::types::LintCliOutput;
use crate::qianji_cli::json_output::{CliJsonEnvelope, render_cli_json};

pub(super) fn render_lint_json_output(
    report: &LintReport,
    resolved_path: &Path,
) -> io::Result<LintCliOutput> {
    let exit_code = if report.ok { 0 } else { 2 };
    let rendered = render_cli_json(CliJsonEnvelope {
        kind: "qianji_lint_report",
        command: "lint",
        domain: lint_domain_name(&report.domain),
        path: resolved_path,
        source_id: &report.source_id,
        ok: report.ok,
        exit_code,
        report,
        analysis: lint_repair_plan_analysis(report),
    })?;
    Ok(LintCliOutput {
        rendered,
        exit_code,
    })
}

pub(super) fn lint_repair_plan_analysis(report: &LintReport) -> Option<serde_json::Value> {
    let repair_plans = issue_repair_plans(report);
    if repair_plans.is_empty() {
        None
    } else {
        Some(serde_json::json!({
            "repair_plans": repair_plans,
        }))
    }
}

pub(super) fn issue_repair_plans(report: &LintReport) -> Vec<serde_json::Value> {
    report
        .issues
        .iter()
        .filter_map(|issue| {
            let structured_repair = issue.structured_repair.as_ref()?;
            Some(serde_json::json!({
                "code": issue.code,
                "title": issue.title,
                "structured_repair": structured_repair,
            }))
        })
        .collect()
}

pub(super) fn lint_domain_name(domain: &LintDomain) -> &'static str {
    match domain {
        LintDomain::Bpmn => "bpmn",
        LintDomain::Dmn => "dmn",
    }
}
