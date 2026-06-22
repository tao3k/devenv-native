use super::{
    flowhub_root, load_flowhub_module_manifest, parse_flowhub_module_manifest,
    real_flowhub_fixture_available,
};

#[test]
fn load_flowhub_module_manifest_reads_real_plan_org_bpmn_contract() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest = load_flowhub_module_manifest(flowhub_root().join("plan/qianji.toml"))
        .unwrap_or_else(|error| panic!("plan module manifest should load: {error}"));

    assert!(manifest.graph.is_empty());
    assert!(manifest.contract.as_ref().is_some_and(|contract| {
        contract.register.is_empty()
            && contract
                .required
                .iter()
                .any(|path| path == "PLAN_POLICY.org")
            && contract
                .required
                .iter()
                .any(|path| path == "agent-coding.org")
            && contract
                .required
                .iter()
                .any(|path| path == "agent-coding.bpmn")
    }));
}

#[test]
fn load_flowhub_module_manifest_reads_real_wendao_org_bpmn_contract() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest = load_flowhub_module_manifest(flowhub_root().join("wendao/qianji.toml"))
        .unwrap_or_else(|error| panic!("wendao module manifest should load: {error}"));

    assert!(manifest.graph.is_empty());
    assert!(manifest.contract.as_ref().is_some_and(|contract| {
        contract.register == vec!["client".to_string()]
            && contract
                .required
                .iter()
                .any(|path| path == "WENDAO_POLICY.org")
            && contract
                .required
                .iter()
                .any(|path| path == "docs-search.org")
            && contract
                .required
                .iter()
                .any(|path| path == "docs-search.bpmn")
            && contract
                .required
                .iter()
                .any(|path| path == "client/qianji.toml")
    }));
}

#[test]
fn load_flowhub_module_manifest_reads_real_research_org_bpmn_contracts() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest = load_flowhub_module_manifest(flowhub_root().join("research/paper/qianji.toml"))
        .unwrap_or_else(|error| panic!("research paper manifest should load: {error}"));

    assert!(manifest.graph.is_empty());
    assert!(manifest.contract.as_ref().is_some_and(|contract| {
        contract.register.is_empty()
            && contract
                .required
                .iter()
                .any(|path| path == "PAPER_POLICY.org")
            && contract
                .required
                .iter()
                .any(|path| path == "paper-canonicalize.org")
            && contract
                .required
                .iter()
                .any(|path| path == "paper-canonicalize.bpmn")
            && contract
                .required
                .iter()
                .any(|path| path == "paper-deep-read.org")
            && contract
                .required
                .iter()
                .any(|path| path == "paper-deep-read.bpmn")
            && contract
                .required
                .iter()
                .any(|path| path == "paper-compare.org")
            && contract
                .required
                .iter()
                .any(|path| path == "paper-compare.bpmn")
    }));
}

#[test]
fn load_flowhub_module_manifest_reads_real_wendao_client_plan_policy_bpmn_contract() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let manifest =
        load_flowhub_module_manifest(flowhub_root().join("wendao/client/plan/qianji.toml"))
            .unwrap_or_else(|error| panic!("wendao client plan manifest should load: {error}"));

    assert!(manifest.graph.is_empty());
    assert!(manifest.contract.as_ref().is_some_and(|contract| {
        contract.register.is_empty()
            && contract
                .required
                .iter()
                .any(|path| path == "PLAN_POLICY.org")
            && contract
                .required
                .iter()
                .any(|path| path == "PLAN_POLICY.bpmn")
            && contract
                .required
                .iter()
                .any(|path| path == "_sdd_template.org")
            && contract
                .required
                .iter()
                .any(|path| path == "_execplan_template.org")
    }));
}

#[test]
fn flowhub_module_manifest_rejects_graph_entries_outside_contract_required() {
    let error = parse_flowhub_module_manifest(
        r#"
version = 1

[module]
name = "wendao"

[exports]
entry = "task.wendao-start"
ready = "task.wendao-ready"

[contract]
required = ["qianji.toml"]

[[graph]]
path = "docs-search.mmd"
topology = "bounded_loop"

[[graph.node]]
label = "done gate"
kind = "gate"
role = "allow completion only when required guards and validators pass"
agent_action = "do not treat the slice as complete before qianji check passes"
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("invalid graph contract should fail"));

    assert!(
        error
            .to_string()
            .contains("must also be declared in `contract.required`")
    );
}

#[test]
fn flowhub_module_manifest_rejects_blank_graph_name_override() {
    let error = parse_flowhub_module_manifest(
        r#"
version = 1

[module]
name = "wendao"

[exports]
entry = "task.wendao-start"
ready = "task.wendao-ready"

[contract]
required = ["docs-search.mmd"]

[[graph]]
path = "docs-search.mmd"
name = "   "
topology = "bounded_loop"

[[graph.node]]
label = "done gate"
kind = "gate"
role = "allow completion only when required guards and validators pass"
agent_action = "do not treat the slice as complete before qianji check passes"
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("blank graph name should fail"));

    let message = error.to_string();
    assert!(message.contains("[[graph]].name"));
    assert!(message.contains("non-empty"));
}

#[test]
fn flowhub_module_manifest_accepts_graph_without_node_contracts() {
    let manifest = parse_flowhub_module_manifest(
        r#"
version = 1

[module]
name = "wendao"

[exports]
entry = "task.wendao-start"
ready = "task.wendao-ready"

[contract]
required = ["docs-search.mmd"]

[[graph]]
path = "docs-search.mmd"
topology = "bounded_loop"
"#,
    )
    .unwrap_or_else(|error| panic!("graph without node contracts should load: {error}"));

    assert_eq!(manifest.graph.len(), 1);
    assert!(manifest.graph[0].node.is_empty());
}

#[test]
fn flowhub_module_manifest_rejects_invalid_graph_workdir_contract() {
    let error = parse_flowhub_module_manifest(
        r#"
version = 1

[module]
name = "plan"

[exports]
entry = "task.plan-start"
ready = "task.plan-ready"

[contract]
required = ["codex-plan.mmd"]

[[graph]]
path = "codex-plan.mmd"
topology = "bounded_loop"

[graph.workdir]
root = "<plan-workdir>"

[graph.workdir.check]
require = ["blueprint"]
flowchart = ["blueprint"]

[[graph.node]]
label = "done gate"
kind = "gate"
role = "allow completion only when required guards and validators pass"
agent_action = "do not treat the slice as complete before qianji check passes"
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("invalid graph workdir contract should fail"));

    let message = error.to_string();
    assert!(message.contains("flowchart.mmd"), "{message}");
}
