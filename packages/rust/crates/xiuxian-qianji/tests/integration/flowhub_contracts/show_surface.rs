use super::*;

#[test]
fn classify_flowhub_dir_detects_real_root_and_module() {
    assert_eq!(
        classify_flowhub_dir(flowhub_root())
            .unwrap_or_else(|error| panic!("root should classify: {error}")),
        Some(xiuxian_qianji::FlowhubDirKind::Root)
    );
    assert_eq!(
        classify_flowhub_dir(flowhub_root().join("rust"))
            .unwrap_or_else(|error| panic!("module should classify: {error}")),
        Some(xiuxian_qianji::FlowhubDirKind::Module)
    );
}

#[test]
fn show_flowhub_summarizes_real_root() {
    let show = show_flowhub(flowhub_root())
        .unwrap_or_else(|error| panic!("real Flowhub root should show: {error}"));

    let FlowhubShow::Root(show) = show else {
        panic!("expected Flowhub root summary");
    };
    assert_eq!(show.modules.len(), 5);
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "rust")
    );
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "blueprint")
    );
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "wendao")
    );

    let rendered = render_flowhub_show(&FlowhubShow::Root(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Flowhub"));
    assert!(rendered.contains("## rust"));
}

#[test]
fn show_flowhub_summarizes_real_leaf_module() {
    let show = show_flowhub(flowhub_root().join("rust"))
        .unwrap_or_else(|error| panic!("real Flowhub module should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "rust");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Leaf);
    assert!(show.summary.child_modules.is_empty());

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Flowhub Module"));
    assert!(rendered.contains("Module: rust"));
    assert!(rendered.contains("## Contract"));
    assert!(rendered.contains("Registered children: 0"));
}

#[test]
fn show_flowhub_keeps_required_only_plan_node_as_leaf() {
    let show = show_flowhub(flowhub_root().join("plan"))
        .unwrap_or_else(|error| panic!("plan node should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "plan");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Leaf);
    assert_eq!(show.registered_child_count, 0);
    assert_eq!(show.required_contract_count, 1);
    assert_eq!(
        show.scenario_cases,
        vec![FlowhubScenarioCaseSummary {
            file_name: "codex-plan.mmd".to_string(),
            merimind_graph_name: "codex-plan".to_string(),
        }]
    );

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("Required contract entries: 1"));
    assert!(rendered.contains("## Scenario Cases"));
    assert!(rendered.contains("Graph name: codex-plan"));
    assert!(rendered.contains("Path: ./plan/codex-plan.mmd"));
}

#[test]
fn show_flowhub_prefers_declared_graph_name_for_leaf_module_summary() {
    let show = show_flowhub(flowhub_root().join("wendao"))
        .unwrap_or_else(|error| panic!("wendao node should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "wendao");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Leaf);
    assert_eq!(show.registered_child_count, 0);
    assert_eq!(show.required_contract_count, 1);
    assert_eq!(
        show.scenario_cases,
        vec![FlowhubScenarioCaseSummary {
            file_name: "docs-search.mmd".to_string(),
            merimind_graph_name: "DOC_SEARCH".to_string(),
        }]
    );

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("Required contract entries: 1"));
    assert!(rendered.contains("## Scenario Cases"));
    assert!(rendered.contains("Graph name: DOC_SEARCH"));
    assert!(rendered.contains("Path: ./wendao/docs-search.mmd"));
}

#[test]
fn show_flowhub_graph_extracts_live_mermaid_nodes_edges_and_exports() {
    let show = show_flowhub_graph(flowhub_root().join("plan/codex-plan.mmd"))
        .unwrap_or_else(|error| panic!("live Mermaid graph should show: {error}"));

    assert_eq!(show.merimind_graph_name, "codex-plan");
    assert_eq!(show.kind, "scenario");
    assert_eq!(show.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(
        show.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
    assert_eq!(show.owning_module_ref, "plan");
    assert_eq!(show.direction, "LR");
    assert!(show.mermaid.contains("flowchart LR"));
    assert!(show.nodes.iter().any(|node| {
        node.label == "coding"
            && node.kind == FlowhubGraphNodeKind::Context
            && node.exports_entry.as_deref() == Some("task.coding-start")
    }));
    assert!(show.nodes.iter().any(|node| {
        node.label == "domain validators"
            && node.kind == FlowhubGraphNodeKind::Validator
            && node.next == vec!["done gate".to_string(), "diagnostics".to_string()]
    }));
    assert!(show.nodes.iter().any(|node| {
        node.label == "plan"
            && node.kind == FlowhubGraphNodeKind::Artifact
            && node.next == vec!["Codex write bounded surface".to_string()]
            && node.exports_ready.as_deref() == Some("task.plan-ready")
    }));
    assert!(
        show.expected_work_surface
            .contains(&"qianji.toml".to_string())
    );
    assert!(
        show.expected_work_surface
            .contains(&"codex-plan.mmd".to_string())
    );
    assert!(show.owning_module_manifest_toml.contains("[module]"));
    assert!(show.owning_module_manifest_toml.contains("name = \"plan\""));
    assert!(show.missing_registered_modules.is_empty());
    assert!(show.unknown_graph_nodes.is_empty());

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.starts_with("# Graph"));
    assert!(rendered.contains("Name: codex-plan"));
    assert!(rendered.contains("Kind: scenario"));
    assert!(rendered.contains("Topology: bounded_loop"));
    assert!(rendered.contains("Declared topology: bounded_loop"));
    assert!(rendered.contains("## Mermaid"));
    assert!(rendered.contains("```mermaid"));
    assert!(rendered.contains("## Nodes"));
    assert!(rendered.contains("### coding"));
    assert!(rendered.contains("Kind: context"));
    assert!(rendered.contains("### boundary and drift check"));
    assert!(rendered.contains("Kind: guard"));
    assert!(rendered.contains("## Module contract"));
    assert!(rendered.contains("## Owning qianji.toml"));
}

#[test]
fn show_flowhub_graph_uses_local_module_contract_for_wendao_leaf_case() {
    let show = show_flowhub_graph(flowhub_root().join("wendao/docs-search.mmd"))
        .unwrap_or_else(|error| panic!("wendao Mermaid graph should show: {error}"));

    assert_eq!(show.merimind_graph_name, "DOC_SEARCH");
    assert_eq!(show.owning_module_ref, "wendao");
    assert_eq!(show.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(
        show.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
    assert_eq!(
        show.expected_work_surface,
        vec!["qianji.toml".to_string(), "docs-search.mmd".to_string()]
    );
    assert!(
        show.owning_module_manifest_toml
            .contains("name = \"wendao\"")
    );
    assert!(!show.owning_module_manifest_toml.contains("blueprint"));

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("Name: DOC_SEARCH"));
    assert!(rendered.contains("Topology: bounded_loop"));
    assert!(rendered.contains("Declared topology: bounded_loop"));
    assert!(rendered.contains("## Module contract"));
    assert!(rendered.contains("- qianji.toml"));
    assert!(rendered.contains("- docs-search.mmd"));
    assert!(rendered.contains("## Owning qianji.toml"));
    assert!(!rendered.contains("## Expected work surface"));
    assert!(!rendered.contains("## Local qianji.toml template"));
    assert!(!rendered.contains("blueprint/"));
    assert!(!rendered.contains("plan/"));
}

#[test]
fn show_flowhub_graph_surfaces_unknown_graph_nodes() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_undeclared_mermaid_nodes_case(&temp_dir);

    let show = show_flowhub_graph(root.join("plan/codex-plan.mmd"))
        .unwrap_or_else(|error| panic!("Mermaid graph with unknown nodes should show: {error}"));

    assert_eq!(show.unknown_graph_nodes, vec!["style".to_string()]);
    assert!(show.nodes.iter().any(|node| {
        node.label == "style"
            && node.kind == FlowhubGraphNodeKind::Unknown
            && node.agent_action
                == "do not rely on this node until the Flowhub graph contract is corrected"
    }));
    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("### style"));
    assert!(rendered.contains("Kind: unknown"));
    assert!(rendered.contains(
        "Agent action: do not rely on this node until the Flowhub graph contract is corrected"
    ));
}

#[test]
fn show_flowhub_graph_preserves_raw_mermaid_but_ignores_presentation_directives_in_semantics() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_mermaid_presentation_directives_case(&temp_dir);

    let show = show_flowhub_graph(root.join("plan/codex-plan.mmd")).unwrap_or_else(|error| {
        panic!("Mermaid graph with presentation directives should show: {error}")
    });

    assert_eq!(show.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(show.declared_topology, None);
    assert!(show.mermaid.contains("classDef highlight"));
    assert!(show.mermaid.contains("style C"));
    assert!(show.mermaid.contains("click G"));
    assert!(show.unknown_graph_nodes.is_empty());
    assert!(!show.nodes.iter().any(|node| node.label == "highlight"));
    assert!(
        !show
            .nodes
            .iter()
            .any(|node| node.label.contains("https://"))
    );
    assert!(
        show.nodes
            .iter()
            .any(|node| node.label == "flowchart alignment")
    );

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("classDef highlight"));
    assert!(rendered.contains("style C"));
    assert!(rendered.contains("click G"));
    assert!(!rendered.contains("### highlight"));
}
