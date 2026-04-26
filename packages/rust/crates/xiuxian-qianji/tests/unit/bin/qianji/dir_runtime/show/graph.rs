use super::*;

#[test]
fn run_show_graph_command_renders_flowhub_mermaid_graph() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Graph(flowhub_root().join("plan/codex-plan.mmd")),
        }),
        "show graph command should render Flowhub Mermaid preview",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Graph"));
    assert!(output.rendered.contains("Name: codex-plan"));
    assert!(
        output
            .rendered
            .contains("Path: ./qianji-flowhub/plan/codex-plan.mmd")
    );
    assert!(output.rendered.contains("Owning module: plan"));
    assert!(output.rendered.contains("Direction: LR"));
    assert!(output.rendered.contains("Declared topology: bounded_loop"));
    assert!(output.rendered.contains("## Execution"));
    assert!(output.rendered.contains("- Start at `coding`."));
    assert!(output.rendered.contains("- Complete at `done gate`."));
    assert!(output.rendered.contains("## Nodes"));
    assert!(output.rendered.contains("`coding` [`context`]"));
    assert!(output.rendered.contains("Entry: `task.coding-start`"));
    assert!(output.rendered.contains("`blueprint` [`artifact`]"));
    assert!(
        output
            .rendered
            .contains("`domain validators` [`validator`]")
    );
    assert!(output.rendered.contains("Ready: `task.plan-ready`"));
    assert!(output.rendered.contains("## Check Surface"));
    assert!(
        output
            .rendered
            .contains("Flowhub source surface: `qianji.toml`, `codex-plan.mmd`.")
    );
    assert!(output.rendered.contains("blueprint/**/*.md\nplan/**/*.md"));
    assert!(output.rendered.contains(
        "`qianji check` keeps these surfaces visible in `flowchart.mmd`: `blueprint`, `plan`."
    ));
    assert!(
        !output
            .rendered
            .contains("No declared bounded check surface.")
    );
    assert!(!output.rendered.contains("## Expected Work Surface"));
    assert!(!output.rendered.contains("## Local Contract Template"));
    assert!(output.rendered.contains("## Mermaid"));
    assert!(output.rendered.contains("```mermaid"));
    assert!(output.rendered.contains("flowchart LR"));
}

#[test]
fn run_show_graph_command_prefers_declared_graph_name_override() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Graph(flowhub_root().join("wendao/docs-search.mmd")),
        }),
        "show graph command should respect the local Flowhub graph name override",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.starts_with("# Graph"));
    assert!(output.rendered.contains("Name: DOC_SEARCH"));
    assert!(
        output
            .rendered
            .contains("Path: ./qianji-flowhub/wendao/docs-search.mmd")
    );
    assert!(output.rendered.contains("Owning module: wendao"));
    assert!(output.rendered.contains("Direction: LR"));
    assert!(output.rendered.contains("Declared topology: bounded_loop"));
    assert!(output.rendered.contains("## Execution"));
    assert!(output.rendered.contains("## Check Surface"));
    assert!(
        output
            .rendered
            .contains("does not yet declare `[graph.workdir]`")
    );
    assert!(
        output
            .rendered
            .contains("Flowhub source surface: `qianji.toml`, `docs-search.mmd`.")
    );
    assert!(
        output
            .rendered
            .contains("No declared bounded check surface.")
    );
    assert!(!output.rendered.contains("<localized-workdir>/"));
    assert!(!output.rendered.contains("## Local Contract Template"));
}

#[test]
fn run_show_graph_command_renders_research_deep_read_localized_surfaces() {
    let output = must_ok(
        run_dir_command(DirCliCommand::Show {
            target: ShowCliTarget::Graph(anchored_workdir_fixture_graph()),
        }),
        "show graph command should render localized research surfaces",
    );

    assert_eq!(output.exit_code, 0);
    assert!(output.rendered.contains("Name: PAPER_DEEP_READ"));
    assert!(
        output
            .rendered
            .contains("Owning module: paper_deep_read_workdir")
    );
    assert!(output.rendered.contains("## Check Surface"));
    assert!(output.rendered.contains("Run root: `runs/<run_id>`."));
    assert!(output.rendered.contains("refs/paper.json"));
    assert!(
        output
            .rendered
            .contains("staging/semantics/claim_ledger.patch.jsonl")
    );
    assert!(output.rendered.contains("## Persistent Target Surface"));
    assert!(output.rendered.contains("papers/<paper_id>/"));
    assert!(output.rendered.contains("    claim_ledger.jsonl"));
    assert!(output.rendered.contains("## Done Gate"));
    assert!(output.rendered.contains("refs/topic.json"));
    assert!(!output.rendered.contains("## Local Contract Template"));
}
