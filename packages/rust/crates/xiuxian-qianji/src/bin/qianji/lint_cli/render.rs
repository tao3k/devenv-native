use std::fmt::Write as _;
use std::io;
use std::path::Path;

use qianji_bpmn_engine::{LintDomain, LintIssue, LintReport};

use super::command::LintCliOutput;
use crate::json_output::{CliJsonEnvelope, render_cli_json};

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

pub(super) fn render_lint_output(report: &LintReport, resolved_path: &Path) -> LintCliOutput {
    if report.ok {
        return LintCliOutput {
            rendered: format!(
                "# Lint Passed\n\nSource: {}\nPath: {}\nDomain: {}\nStatus: no blocking issues found in the bounded lint contract.\n",
                report.source_id,
                resolved_path.display(),
                lint_domain_name(&report.domain),
            ),
            exit_code: 0,
        };
    }

    let mut rendered = format!(
        "# Lint Failed\n\nSource: {}\nPath: {}\nDomain: {}\nIssues: {}\n",
        report.source_id,
        resolved_path.display(),
        lint_domain_name(&report.domain),
        report.issues.len(),
    );

    for issue in &report.issues {
        append_issue_markdown(&mut rendered, issue);
    }

    LintCliOutput {
        rendered,
        exit_code: 2,
    }
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

fn append_issue_markdown(rendered: &mut String, issue: &LintIssue) {
    let _ = writeln!(rendered, "\n## [{}] {}", issue.code, issue.title);
    let _ = writeln!(rendered, "Severity: error");
    let _ = writeln!(rendered, "Summary: {}", issue.summary);
    let _ = writeln!(rendered, "\n### Why It Failed");
    let _ = writeln!(rendered, "{}", issue.why_it_failed);
    let _ = writeln!(rendered, "\n### Repair Guidance");
    for step in &issue.repair_guidance {
        let _ = writeln!(rendered, "- {step}");
    }
    let _ = writeln!(rendered, "\n### LLM Fix Prompt");
    let _ = writeln!(rendered, "{}", issue.llm_fix_prompt);
    let _ = writeln!(rendered, "\n### Evidence");
    let _ = writeln!(rendered, "```json");
    let evidence = serde_json::to_string_pretty(&issue.evidence)
        .unwrap_or_else(|_error| "{\"error\":\"failed to render lint evidence\"}".to_string());
    let _ = writeln!(rendered, "{evidence}");
    let _ = writeln!(rendered, "```");
}
