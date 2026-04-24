use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_testing::{
    CollectionContext, ContractFinding, FindingSeverity, ModularityRulePack, RulePack,
    assert_crate_test_policy_with_workspace_config,
};

#[test]
fn enforce_crate_test_policy_gate() {
    assert_crate_test_policy_with_workspace_config(Path::new(env!("CARGO_MANIFEST_DIR")));
}

#[test]
fn enforce_modularity_contract_gate() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let findings = collect_modularity_findings(crate_root);
    let blocking_findings = findings
        .iter()
        .filter(|finding| finding.severity >= FindingSeverity::Error)
        .collect::<Vec<_>>();

    assert!(
        blocking_findings.is_empty(),
        "{}",
        format_modularity_gate_report(&findings, &blocking_findings)
    );
}

fn collect_modularity_findings(crate_root: &Path) -> Vec<ContractFinding> {
    let Some(crate_name) = crate_root.file_name().and_then(|value| value.to_str()) else {
        panic!("failed to derive crate name from {}", crate_root.display());
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
        .unwrap_or_else(|error| panic!("failed to collect modularity artifacts: {error}"));
    pack.evaluate(&artifacts)
        .unwrap_or_else(|error| panic!("failed to evaluate modularity artifacts: {error}"))
}

fn resolve_workspace_root(crate_root: &Path) -> PathBuf {
    crate_root
        .ancestors()
        .find_map(|candidate| {
            let manifest_path = candidate.join("Cargo.toml");
            let content = fs::read_to_string(manifest_path).ok()?;
            if content.contains("[workspace]") {
                return Some(candidate.to_path_buf());
            }
            None
        })
        .unwrap_or_else(|| {
            panic!(
                "failed to resolve workspace root from crate root {}",
                crate_root.display()
            )
        })
}

fn format_modularity_gate_report(
    findings: &[ContractFinding],
    blocking_findings: &[&ContractFinding],
) -> String {
    let mut output = String::new();
    output.push_str("modularity gate failed with blocking findings (severity >= Error):\n");

    for finding in blocking_findings {
        let _ = writeln!(
            output,
            "- [{}] {} :: {}:{}",
            finding.rule_id,
            finding.summary,
            finding_path(finding),
            finding_locator(finding)
        );
    }

    let warning_count = findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Warning)
        .count();
    if warning_count > 0 {
        let _ = writeln!(output, "non-blocking warnings: {warning_count}");
    }

    output
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
