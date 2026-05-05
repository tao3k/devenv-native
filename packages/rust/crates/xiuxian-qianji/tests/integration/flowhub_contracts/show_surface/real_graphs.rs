use super::{
    FlowhubGraphTopology, flowhub_root, real_flowhub_fixture_available, render_flowhub_graph_show,
    show_flowhub_graph,
};

#[test]
fn show_flowhub_graph_extracts_live_mermaid_nodes_edges_and_exports() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub_graph(flowhub_root().join("plan/codex-plan.mmd"))
        .unwrap_or_else(|error| panic!("live Mermaid graph should show: {error}"));

    assert_eq!(show.merimind_graph_name, "codex-plan");
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
            && node.kind.as_deref() == Some("context")
            && node.exports_entry.as_deref() == Some("task.coding-start")
    }));
    assert!(show.nodes.iter().any(|node| {
        node.label == "domain validators"
            && node.kind.as_deref() == Some("validator")
            && node.next == vec!["done gate".to_string(), "diagnostics".to_string()]
    }));
    assert!(show.nodes.iter().any(|node| {
        node.label == "plan"
            && node.kind.as_deref() == Some("artifact")
            && node.next == vec!["Codex write bounded surface".to_string()]
            && node.exports_ready.as_deref() == Some("task.plan-ready")
    }));
    assert!(
        show.module_contract_surface
            .contains(&"qianji.toml".to_string())
    );
    assert!(
        show.module_contract_surface
            .contains(&"codex-plan.mmd".to_string())
    );
    assert!(
        show.declared_check_surface
            .note
            .as_deref()
            .is_some_and(|note| note.contains("localized plan work surface"))
    );
    assert_eq!(
        show.declared_check_surface.root.as_deref(),
        Some("<plan-workdir>")
    );
    assert!(
        show.declared_check_surface
            .required_paths
            .contains(&"blueprint/**/*.md".to_string())
    );
    assert!(
        show.declared_check_surface
            .flowchart_surfaces
            .contains(&"blueprint".to_string())
    );
    assert!(show.owning_module_manifest_toml.contains("[module]"));
    assert!(show.owning_module_manifest_toml.contains("name = \"plan\""));
    assert!(show.missing_registered_modules.is_empty());
    assert!(show.unknown_graph_nodes.is_empty());

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.starts_with("# Graph"));
    assert!(rendered.contains("Name: codex-plan"));
    assert!(rendered.contains("Owning module: plan"));
    assert!(rendered.contains("Direction: LR"));
    assert!(rendered.contains("Topology: bounded_loop"));
    assert!(rendered.contains("Declared topology: bounded_loop"));
    assert!(rendered.contains("## Execution"));
    assert!(rendered.contains("- Start at `coding`."));
    assert!(rendered.contains("- Complete at `done gate`."));
    assert!(rendered.contains("localized plan work surface"));
    assert!(rendered.contains("## Nodes"));
    assert!(rendered.contains("`coding` [`context`]"));
    assert!(rendered.contains("`boundary and drift check` [`guard`]"));
    assert!(rendered.contains("Entry: `task.coding-start`"));
    assert!(rendered.contains("Ready: `task.plan-ready`"));
    assert!(rendered.contains("## Check Surface"));
    assert!(rendered.contains("Run root: `<plan-workdir>`."));
    assert!(rendered.contains("blueprint/**/*.md"));
    assert!(rendered.contains("plan/**/*.md"));
    assert!(!rendered.contains("## Expected Work Surface"));
    assert!(!rendered.contains("## Persistent Target Surface"));
    assert!(!rendered.contains("## Done Gate"));
    assert!(!rendered.contains("## Local Contract Template"));
    assert!(rendered.contains("## Mermaid"));
    assert!(rendered.contains("```mermaid"));
}

#[test]
fn show_flowhub_graph_uses_local_module_contract_for_wendao_leaf_case() {
    if !real_flowhub_fixture_available() {
        return;
    }
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
        show.module_contract_surface,
        vec!["qianji.toml".to_string(), "docs-search.mmd".to_string()]
    );
    assert!(
        show.declared_check_surface
            .note
            .as_deref()
            .is_some_and(|note| note.contains("[graph.workdir]"))
    );
    assert_eq!(show.declared_check_surface.root, None);
    assert!(show.declared_check_surface.required_paths.is_empty());
    assert!(show.declared_check_surface.flowchart_surfaces.is_empty());
    assert!(
        show.owning_module_manifest_toml
            .contains("name = \"wendao\"")
    );
    assert!(!show.owning_module_manifest_toml.contains("blueprint"));

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("Name: DOC_SEARCH"));
    assert!(rendered.contains("Owning module: wendao"));
    assert!(rendered.contains("Direction: LR"));
    assert!(rendered.contains("Topology: bounded_loop"));
    assert!(rendered.contains("Declared topology: bounded_loop"));
    assert!(rendered.contains("## Execution"));
    assert!(rendered.contains("does not yet declare `[graph.workdir]`"));
    assert!(rendered.contains("## Nodes"));
    assert!(rendered.contains("`wendao.docs.search` [`capability_contract`]"));
    assert!(rendered.contains("`wendao.docs.document` [`capability_contract`]"));
    assert!(rendered.contains("`wendao.docs.document_structure` [`capability_contract`]"));
    assert!(rendered.contains("## Check Surface"));
    assert!(rendered.contains("No declared bounded check surface."));
    assert!(!rendered.contains("<localized-workdir>/"));
    assert!(!rendered.contains("## Local Contract Template"));
    assert!(!rendered.contains("## Persistent Target Surface"));
    assert!(!rendered.contains("blueprint/"));
    assert!(!rendered.contains("plan/"));
}

#[test]
fn show_flowhub_graph_describes_live_research_canonicalize_case() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub_graph(flowhub_root().join("research/paper/paper-canonicalize.mmd"))
        .unwrap_or_else(|error| panic!("research canonicalize graph should show: {error}"));

    assert_eq!(show.merimind_graph_name, "PAPER_CANONICALIZE");
    assert_eq!(show.owning_module_ref, "research/paper");
    assert_eq!(show.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(
        show.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
    assert!(show.nodes.iter().any(|node| {
        node.label == "research/paper" && node.kind.as_deref() == Some("artifact")
    }));
    assert!(
        show.nodes
            .iter()
            .any(|node| { node.label == "pdf_intake" && node.kind.as_deref() == Some("process") })
    );
    assert!(show.nodes.iter().any(|node| {
        node.label == "canonicalize_done" && node.kind.as_deref() == Some("artifact")
    }));
    assert!(show.unknown_graph_nodes.is_empty());
    assert!(
        show.declared_check_surface
            .note
            .as_deref()
            .is_some_and(|note| note.contains("canonical paper objects"))
    );
    assert_eq!(
        show.declared_check_surface.root.as_deref(),
        Some("runs/<run_id>")
    );
    assert!(
        show.declared_check_surface
            .required_paths
            .contains(&"refs/paper.json".to_string())
    );
    assert!(
        show.declared_check_surface
            .required_paths
            .contains(&"staging/structure/citation_graph.patch.json".to_string())
    );
    assert!(
        show.declared_check_surface
            .flowchart_surfaces
            .contains(&"staging".to_string())
    );

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("Name: PAPER_CANONICALIZE"));
    assert!(rendered.contains("Path: ./qianji-flowhub/research/paper/paper-canonicalize.mmd"));
    assert!(rendered.contains("Owning module: research/paper"));
    assert!(rendered.contains("## Execution"));
    assert!(rendered.contains("canonical paper objects"));
    assert!(rendered.contains("## Nodes"));
    assert!(rendered.contains("`pdf_intake` [`process`]"));
    assert!(rendered.contains("`canonicalize_done` [`artifact`]"));
    assert!(rendered.contains("## Check Surface"));
    assert!(rendered.contains("Run root: `runs/<run_id>`."));
    assert!(rendered.contains("refs/paper.json"));
    assert!(rendered.contains("staging/structure/citation_graph.patch.json"));
    assert!(rendered.contains("## Persistent Target Surface"));
    assert!(rendered.contains("## Done Gate"));
    assert!(!rendered.contains("## Local Contract Template"));
}

#[test]
fn show_flowhub_graph_describes_live_research_deep_read_case() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub_graph(flowhub_root().join("research/paper/paper-deep-read.mmd"))
        .unwrap_or_else(|error| panic!("research deep-read graph should show: {error}"));

    assert_eq!(show.merimind_graph_name, "PAPER_DEEP_READ");
    assert_eq!(show.owning_module_ref, "research/paper");
    assert_eq!(show.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(
        show.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
    assert!(
        show.declared_check_surface
            .note
            .as_deref()
            .is_some_and(|note| note.contains("bounded run-local surface"))
    );
    assert_eq!(
        show.declared_check_surface.root.as_deref(),
        Some("runs/<run_id>")
    );
    assert!(
        show.declared_check_surface
            .required_paths
            .contains(&"refs/topic.json".to_string())
    );
    assert!(
        show.declared_check_surface
            .required_paths
            .contains(&"staging/semantics/claim_ledger.patch.jsonl".to_string())
    );
    assert!(
        show.declared_check_surface
            .persistent_target_surface_tree
            .contains(&"  semantics/".to_string())
    );
    assert!(
        show.declared_check_surface
            .done_gate_require
            .contains(&"semantics/claim_ledger.jsonl".to_string())
    );

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("Name: PAPER_DEEP_READ"));
    assert!(rendered.contains("## Check Surface"));
    assert!(rendered.contains("Run root: `runs/<run_id>`."));
    assert!(rendered.contains("refs/paper.json"));
    assert!(rendered.contains("staging/semantics/claim_ledger.patch.jsonl"));
    assert!(rendered.contains("staging/syntheses/critique_memo.patch.md"));
    assert!(rendered.contains("## Persistent Target Surface"));
    assert!(rendered.contains("## Done Gate"));
    assert!(rendered.contains("refs/topic.json"));
    assert!(rendered.contains("state/current_node.toml"));
    assert!(!rendered.contains("## Local Contract Template"));
}
