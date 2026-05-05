//! Regression coverage for the pinned Qianji audit-flow contract reference.

use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_zhenfa::validate_contract_reference;

const VALID_AUDIT_PLAN: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<qianji-audit-plan version="1.0">
  <summary>
    <intent>Verify the qianji autonomous audit scenario contract.</intent>
    <total-steps>1</total-steps>
  </summary>
  <implementation-steps>
    <step number="1">
      <title>Resolve contract from audit_flow</title>
      <file-target path="packages/rust/crates/xiuxian-qianji/src/scenarios/audit/audit_flow.toml" action="research"/>
      <rationale>The audit scenario should declare an executable zhenfa contract.</rationale>
      <content>
        <description>Read the scenario and validate a canonical plan against its contract.</description>
      </content>
    </step>
  </implementation-steps>
  <risk-assessment>
    <risk-pair severity="low">
      <potential-issue>Blueprint assets drift away from executable validation.</potential-issue>
      <mitigation-strategy>Keep one regression test pinned to the shipped scenario contract.</mitigation-strategy>
    </risk-pair>
  </risk-assessment>
  <verification-strategy>
    <test-command>cargo test -p xiuxian-qianji --test audit_scenario_contract</test-command>
    <expected-outcome>Scenario contract path resolves and validates the canonical XML plan.</expected-outcome>
  </verification-strategy>
</qianji-audit-plan>
"#;

fn audit_scenario_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scenarios")
        .join("audit")
}

#[test]
fn audit_flow_declares_contract_that_can_validate_plans() {
    let audit_dir = audit_scenario_dir();
    let manifest_path = audit_dir.join("audit_flow.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("audit flow manifest should be readable: {error}"));
    let manifest_value: toml::Value = toml::from_str(&manifest)
        .unwrap_or_else(|error| panic!("audit flow manifest should parse as TOML: {error}"));

    let contract_ref = manifest_value
        .get("nodes")
        .and_then(toml::Value::as_array)
        .and_then(|nodes| {
            nodes.iter().find(|node| {
                node.get("id")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|value| value == "plan")
            })
        })
        .and_then(|node| node.get("zhenfa"))
        .and_then(|zhenfa| zhenfa.get("contract"))
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("audit flow should declare nodes.plan.zhenfa.contract"));

    let resolved = validate_contract_reference(VALID_AUDIT_PLAN, contract_ref, &audit_dir)
        .unwrap_or_else(|error| {
            panic!("audit flow contract should validate canonical plan: {error}")
        });

    assert_eq!(resolved, audit_dir.join("qianji_plan.xsd"));
}
