//! Built-in crate gate helpers for shared policy and modularity enforcement.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::contracts::{
    CollectionContext, ContractFinding, FindingSeverity, ModularityRulePack, RulePack,
};

/// Collect deterministic modularity findings for one crate source tree.
///
/// # Errors
///
/// Returns an error when the crate root cannot be inspected or when the
/// modularity rule pack fails to collect or evaluate source artifacts.
pub fn collect_crate_modularity_findings(
    crate_root: &Path,
) -> Result<Vec<ContractFinding>, String> {
    let Some(crate_name) = crate_root.file_name().and_then(|value| value.to_str()) else {
        return Err(format!(
            "failed to derive crate name from {}",
            crate_root.display()
        ));
    };
    let context = CollectionContext {
        suite_id: "xiuxian-testing-gate".to_string(),
        crate_name: Some(crate_name.to_string()),
        workspace_root: Some(resolve_workspace_root(crate_root)),
        labels: BTreeMap::new(),
    };
    let pack = ModularityRulePack;
    let artifacts = pack
        .collect(&context)
        .map_err(|error| format!("failed to collect modularity artifacts: {error}"))?;
    pack.evaluate(&artifacts)
        .map_err(|error| format!("failed to evaluate modularity artifacts: {error}"))
}

/// Assert the built-in modularity gate for one crate.
///
/// Warning-, error-, and critical-severity findings all block by default.
///
/// # Panics
///
/// Panics when modularity findings reach blocking severity or when the
/// modularity scan fails.
#[track_caller]
pub fn assert_crate_modularity_gate(crate_root: &Path) {
    let findings =
        collect_crate_modularity_findings(crate_root).unwrap_or_else(|error| panic!("{error}"));
    let blocking_findings = findings
        .iter()
        .filter(|finding| is_blocking_modularity_finding(finding))
        .collect::<Vec<_>>();

    assert!(
        blocking_findings.is_empty(),
        "{}",
        format_crate_modularity_gate_report(&blocking_findings)
    );
}

/// Format one modularity gate failure report.
#[must_use]
pub fn format_crate_modularity_gate_report(blocking_findings: &[&ContractFinding]) -> String {
    let mut output = String::new();
    output.push_str("modularity gate failed with blocking findings (severity >= Warning):\n");

    for finding in blocking_findings {
        let _ = writeln!(
            output,
            "- [{}][{}] {} :: {}:{}",
            finding.rule_id,
            finding_severity_label(finding.severity),
            finding.title,
            finding_path(finding),
            finding_locator(finding)
        );
        let _ = writeln!(output, "  summary: {}", finding.summary);
        if !finding.why_it_matters.trim().is_empty() {
            let _ = writeln!(output, "  why: {}", finding.why_it_matters);
        }
        if !finding.remediation.trim().is_empty() {
            let _ = writeln!(output, "  fix: {}", finding.remediation);
        }
    }

    output
}

fn is_blocking_modularity_finding(finding: &ContractFinding) -> bool {
    finding.severity >= FindingSeverity::Warning
}

fn finding_severity_label(severity: FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Info => "Info",
        FindingSeverity::Warning => "Warning",
        FindingSeverity::Error => "Error",
        FindingSeverity::Critical => "Critical",
    }
}

fn resolve_workspace_root(crate_root: &Path) -> PathBuf {
    crate_root
        .ancestors()
        .find_map(|candidate| {
            let manifest_path = candidate.join("Cargo.toml");
            let content = fs::read_to_string(manifest_path).ok()?;
            content
                .contains("[workspace]")
                .then(|| candidate.to_path_buf())
        })
        .unwrap_or_else(|| crate_root.to_path_buf())
}

fn finding_path(finding: &ContractFinding) -> String {
    if let Some(path) = finding
        .evidence
        .iter()
        .find_map(|evidence| evidence.path.as_ref())
    {
        return path.display().to_string();
    }
    finding
        .labels
        .get("path")
        .cloned()
        .unwrap_or_else(|| "<unknown-path>".to_string())
}

fn finding_locator(finding: &ContractFinding) -> String {
    finding
        .evidence
        .iter()
        .find_map(|evidence| evidence.locator.as_deref())
        .unwrap_or("<unknown-locator>")
        .to_string()
}

#[cfg(test)]
#[path = "../tests/unit/gate.rs"]
mod tests;
