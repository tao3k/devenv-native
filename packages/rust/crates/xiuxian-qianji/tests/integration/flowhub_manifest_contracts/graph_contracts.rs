use super::*;

#[test]
fn load_flowhub_module_manifest_reads_real_plan_graph_topology_contract() {
    let manifest = load_flowhub_module_manifest(flowhub_root().join("plan/qianji.toml"))
        .unwrap_or_else(|error| panic!("plan module manifest should load: {error}"));

    assert_eq!(manifest.graph.len(), 1);
    assert_eq!(manifest.graph[0].path, "codex-plan.mmd");
    assert_eq!(manifest.graph[0].name, None);
    assert_eq!(
        manifest.graph[0].topology,
        FlowhubGraphTopology::BoundedLoop
    );
    assert!(
        manifest.graph[0]
            .node
            .iter()
            .any(|node| node.label == "domain validators")
    );
}

#[test]
fn load_flowhub_module_manifest_reads_real_wendao_graph_topology_contract() {
    let manifest = load_flowhub_module_manifest(flowhub_root().join("wendao/qianji.toml"))
        .unwrap_or_else(|error| panic!("wendao module manifest should load: {error}"));

    assert_eq!(manifest.graph.len(), 1);
    assert_eq!(manifest.graph[0].path, "docs-search.mmd");
    assert_eq!(manifest.graph[0].name.as_deref(), Some("DOC_SEARCH"));
    assert_eq!(
        manifest.graph[0].topology,
        FlowhubGraphTopology::BoundedLoop
    );
    assert!(
        manifest.graph[0]
            .node
            .iter()
            .any(|node| node.label == "diagnostics")
    );
}

#[test]
fn load_flowhub_module_manifest_reads_real_research_graph_node_contracts() {
    let manifest = load_flowhub_module_manifest(flowhub_root().join("research/paper/qianji.toml"))
        .unwrap_or_else(|error| panic!("research paper manifest should load: {error}"));

    assert_eq!(manifest.graph.len(), 3);
    assert_eq!(manifest.graph[0].path, "paper-canonicalize.mmd");
    assert!(
        manifest.graph[0]
            .node
            .iter()
            .any(|node| node.label == "layout_regions_extract")
    );
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
fn flowhub_module_manifest_rejects_graph_without_node_contracts() {
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
topology = "bounded_loop"
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("graph without node contracts should fail"));

    assert!(
        error
            .to_string()
            .contains("requires at least one `[[graph.node]]` entry")
    );
}
