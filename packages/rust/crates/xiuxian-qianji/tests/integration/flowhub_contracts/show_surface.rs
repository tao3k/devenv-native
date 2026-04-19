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
    assert_eq!(show.modules.len(), 6);
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
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "research")
    );

    let rendered = render_flowhub_show(&FlowhubShow::Root(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Flowhub"));
    assert!(rendered.contains("## rust"));
}

#[test]
fn show_flowhub_summarizes_real_research_module() {
    let show = show_flowhub(flowhub_root().join("research"))
        .unwrap_or_else(|error| panic!("research node should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "research");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Composite);
    assert_eq!(show.registered_child_count, 1);
    assert!(
        show.summary
            .child_modules
            .iter()
            .any(|child| child == "research/paper")
    );

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("Module: research"));
    assert!(rendered.contains("Kind: composite"));
    assert!(rendered.contains("Registered children: 1"));
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
        show.localized_work_surface
            .expected_work_surface_tree
            .contains(&"  blueprint/".to_string())
    );
    assert!(
        show.localized_work_surface
            .expected_work_surface_tree
            .contains(&"  plan/".to_string())
    );
    assert!(
        show.localized_work_surface
            .local_contract_template_toml
            .contains("name = \"codex-plan\"")
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
    assert!(rendered.contains("## Expected Work Surface"));
    assert!(rendered.contains("<plan-workdir>/"));
    assert!(rendered.contains("  blueprint/"));
    assert!(rendered.contains("  plan/"));
    assert!(rendered.contains("## Local Contract Template"));
    assert!(rendered.contains("surface = ["));
    assert!(rendered.contains("\"blueprint\""));
    assert!(rendered.contains("\"plan\""));
    assert!(rendered.contains("## Mermaid"));
    assert!(rendered.contains("```mermaid"));
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
        show.module_contract_surface,
        vec!["qianji.toml".to_string(), "docs-search.mmd".to_string()]
    );
    assert!(
        show.localized_work_surface
            .note
            .as_deref()
            .is_some_and(|note| note.contains("[graph.workdir]"))
    );
    assert_eq!(
        show.localized_work_surface.expected_work_surface_tree,
        vec![
            "<localized-workdir>/".to_string(),
            "  qianji.toml".to_string(),
            "  flowchart.mmd".to_string(),
        ]
    );
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
    assert!(rendered.contains("## Expected Work Surface"));
    assert!(rendered.contains("<localized-workdir>/"));
    assert!(rendered.contains("## Local Contract Template"));
    assert!(!rendered.contains("## Persistent Target Surface"));
    assert!(!rendered.contains("blueprint/"));
    assert!(!rendered.contains("plan/"));
}

#[test]
fn show_flowhub_graph_describes_live_research_canonicalize_case() {
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

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("Name: PAPER_CANONICALIZE"));
    assert!(rendered.contains("Path: ./qianji-flowhub/research/paper/paper-canonicalize.mmd"));
    assert!(rendered.contains("Owning module: research/paper"));
    assert!(rendered.contains("## Execution"));
    assert!(rendered.contains("bounded run-local surface"));
    assert!(rendered.contains("## Nodes"));
    assert!(rendered.contains("`pdf_intake` [`process`]"));
    assert!(rendered.contains("`canonicalize_done` [`artifact`]"));
    assert!(rendered.contains("## Expected Work Surface"));
    assert!(rendered.contains("runs/<run_id>/"));
    assert!(rendered.contains("  refs/"));
    assert!(rendered.contains("  staging/"));
    assert!(rendered.contains("## Persistent Target Surface"));
    assert!(rendered.contains("papers/<paper_id>/"));
    assert!(rendered.contains("  extraction/"));
    assert!(rendered.contains("  structure/"));
    assert!(rendered.contains("## Local Contract Template"));
    assert!(rendered.contains("name = \"paper-canonicalize\""));
}

#[test]
fn show_flowhub_graph_describes_live_research_deep_read_case() {
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
        show.localized_work_surface
            .expected_work_surface_tree
            .contains(&"    paper.json".to_string())
    );
    assert!(
        show.localized_work_surface
            .expected_work_surface_tree
            .contains(&"      claim_ledger.patch.jsonl".to_string())
    );
    assert!(
        show.localized_work_surface
            .persistent_target_surface_tree
            .contains(&"    claim_ledger.jsonl".to_string())
    );
    assert!(
        show.localized_work_surface
            .local_contract_template_toml
            .contains("name = \"paper-deep-read\"")
    );

    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("Name: PAPER_DEEP_READ"));
    assert!(rendered.contains("## Expected Work Surface"));
    assert!(rendered.contains("runs/<run_id>/"));
    assert!(rendered.contains("    paper.json"));
    assert!(rendered.contains("      claim_ledger.patch.jsonl"));
    assert!(rendered.contains("      critique_memo.patch.md"));
    assert!(rendered.contains("## Persistent Target Surface"));
    assert!(rendered.contains("papers/<paper_id>/"));
    assert!(rendered.contains("    claim_ledger.jsonl"));
    assert!(rendered.contains("    deep_read.md"));
    assert!(rendered.contains("## Local Contract Template"));
    assert!(rendered.contains("name = \"paper-deep-read\""));
    assert!(rendered.contains("\"refs/topic.json\""));
    assert!(rendered.contains("\"state/current_node.toml\""));
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
            && node.kind.is_none()
            && node.agent_action
                == "do not rely on this node until the Flowhub graph contract is corrected"
    }));
    let rendered = render_flowhub_graph_show(&show);
    assert!(rendered.contains("`style`"));
    assert!(!rendered.contains("Kind: unknown"));
    assert!(
        rendered.contains("do not rely on this node until the Flowhub graph contract is corrected")
    );
    assert!(rendered.contains("Undeclared graph nodes: `style`."));
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
    assert_eq!(
        show.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
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
    assert!(!rendered.contains("`highlight`"));
}
