use std::io;
use std::path::Path;

use qianji_bpmn_engine::{
    BpmnParseOptions, BpmnSourceFile, LintReport, parse_bpmn_package,
    parse_gateway_condition_summary,
};

use super::command::LintCliOutput;
use super::render::{issue_repair_plans, lint_domain_name};
use crate::json_output::{CliJsonEnvelope, render_cli_json};

pub(super) fn render_bpmn_lint_json_output(
    report: &LintReport,
    resolved_path: &Path,
    contents: &str,
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
        analysis: Some(serde_json::json!({
            "gateway_conditions": collect_gateway_condition_analysis(contents),
            "repair_plans": issue_repair_plans(report),
        })),
    })?;
    Ok(LintCliOutput {
        rendered,
        exit_code,
    })
}

fn collect_gateway_condition_analysis(contents: &str) -> serde_json::Value {
    let source = BpmnSourceFile::new("<lint-json-analysis>", contents.to_string());
    let Ok(package) = parse_bpmn_package(&[source], &BpmnParseOptions::default()) else {
        return serde_json::Value::Array(Vec::new());
    };
    let mut conditions = Vec::new();
    for process in package.processes {
        for edge in process.edges {
            let Some(raw) = edge.condition_expression.as_deref() else {
                continue;
            };
            let source_node = process
                .nodes
                .get(edge.from as usize)
                .map_or_else(|| edge.from.to_string(), |node| node.bpmn_id.to_string());
            let target_node = process
                .nodes
                .get(edge.to as usize)
                .map_or_else(|| edge.to.to_string(), |node| node.bpmn_id.to_string());
            let parsed = parse_gateway_condition_summary(raw);
            let supported = parsed.is_some();
            conditions.push(serde_json::json!({
                "process_id": process.key.process_id,
                "source_ref": source_node,
                "target_ref": target_node,
                "raw": raw,
                "parsed": parsed,
                "supported": supported,
            }));
        }
    }
    serde_json::Value::Array(conditions)
}
