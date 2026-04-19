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
    assert_eq!(
        manifest.graph[0]
            .workdir
            .as_ref()
            .and_then(|workdir| workdir.note.as_deref()),
        Some(
            "`qianji check` evaluates the localized plan work surface, not the source Flowhub module."
        )
    );
    assert!(manifest.graph[0].workdir.as_ref().is_some_and(|workdir| {
        workdir
            .check
            .require
            .iter()
            .any(|path| path == "blueprint/**/*.md")
    }));
    assert!(
        manifest.graph[0]
            .resolved_workdir_name()
            .is_some_and(|name| name == "codex-plan")
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

    assert_eq!(manifest.graph.len(), 2);
    assert_eq!(manifest.graph[0].path, "paper-canonicalize.mmd");
    assert_eq!(manifest.graph[1].path, "paper-compare.mmd");
    assert!(manifest.graph[0].workdir.as_ref().is_some_and(|workdir| {
        workdir
            .target
            .as_ref()
            .into_iter()
            .flat_map(|target| target.paths.iter())
            .any(|path| path == "structure/section_tree.json")
    }));
    assert!(
        manifest.graph[0]
            .node
            .iter()
            .any(|node| node.label == "layout_regions_extract")
    );
    assert!(
        manifest
            .contract
            .as_ref()
            .is_some_and(|contract| contract.required.iter().any(|path| path == "paper-deep-read.mmd"))
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
