use crate::contracts::FlowhubGraphTopology;
use crate::flowhub::mermaid::parse_mermaid_flowchart;
use crate::flowhub::scenario_ir::{
    compile_flowhub_scenario_ir, parse_flowhub_graph_annotations, resolve_flowhub_graph_name,
};

#[test]
fn compile_flowhub_scenario_ir_derives_runtime_surface_from_annotations() {
    let source = r#"
%% qianji.scenario.id: deep_read
%% qianji.scenario.name: PAPER_DEEP_READ
%% qianji.scenario.topology: bounded_loop
%% qianji.scenario.workdir_root: runs/<run_id>
%% qianji.scenario.requires:
%%   - refs/paper.json
%% qianji.scenario.target_root: papers/<paper_id>
%% qianji.scenario.target_paths:
%%   - syntheses/deep_read.md
flowchart LR
  A["research/paper"] --> B["materialize_syntheses"]
  B --> C["done gate"]
%% qianji.node.research/paper.kind: artifact
%% qianji.node.materialize_syntheses.checkpoint: checkpoints/materialize_syntheses.json
%% qianji.node.materialize_syntheses.writes:
%%   - staging/syntheses/deep_read.patch.md
%% qianji.node.materialize_syntheses.merge_target:
%%   - syntheses/deep_read.md
%% qianji.node.done gate.kind: gate
%% qianji.done_gate.require:
%%   - syntheses/deep_read.md
"#;

    let annotations = parse_flowhub_graph_annotations(source)
        .unwrap_or_else(|error| panic!("annotations should parse: {error}"))
        .unwrap_or_else(|| panic!("annotations should exist"));
    let graph_name = resolve_flowhub_graph_name(Some(&annotations), None, "paper-deep-read");
    let flowchart = parse_mermaid_flowchart(source, graph_name.as_str(), &["research/paper".into()])
        .unwrap_or_else(|error| panic!("mermaid should parse: {error}"));
    let scenario_ir = compile_flowhub_scenario_ir(
        std::path::Path::new("paper-deep-read.mmd"),
        graph_name.as_str(),
        &flowchart,
        Some(&annotations),
        None,
    )
    .unwrap_or_else(|error| panic!("scenario ir should compile: {error}"))
    .unwrap_or_else(|| panic!("scenario ir should exist"));

    assert_eq!(scenario_ir.merimind_graph_name, "PAPER_DEEP_READ");
    assert_eq!(
        scenario_ir.declared_topology,
        Some(FlowhubGraphTopology::BoundedLoop)
    );
    let workdir = scenario_ir
        .workdir
        .as_ref()
        .unwrap_or_else(|| panic!("workdir should exist"));
    assert_eq!(workdir.root, "runs/<run_id>");
    assert!(workdir.check.require.iter().any(|path| path == "qianji.toml"));
    assert!(
        workdir
            .check
            .require
            .iter()
            .any(|path| path == "refs/paper.json")
    );
    assert!(
        workdir
            .check
            .require
            .iter()
            .any(|path| path == "checkpoints/materialize_syntheses.json")
    );
    assert!(
        workdir
            .check
            .require
            .iter()
            .any(|path| path == "staging/syntheses/deep_read.patch.md")
    );
    assert_eq!(
        workdir.check.flowchart,
        vec![
            "state".to_string(),
            "checkpoints".to_string(),
            "staging".to_string()
        ]
    );
    assert_eq!(
        workdir
            .target
            .as_ref()
            .unwrap_or_else(|| panic!("target should exist"))
            .paths,
        vec!["syntheses/deep_read.md".to_string()]
    );
    assert_eq!(
        scenario_ir
            .node_contract("done gate")
            .and_then(|node| node.kind.as_deref()),
        Some("gate")
    );
}

#[test]
fn compile_flowhub_scenario_ir_rejects_merge_targets_outside_target_surface() {
    let source = r#"
%% qianji.scenario.workdir_root: runs/<run_id>
%% qianji.scenario.topology: dag
%% qianji.scenario.target_root: papers/<paper_id>
%% qianji.scenario.target_paths:
%%   - syntheses/deep_read.md
flowchart LR
  A["research/paper"] --> B["materialize_syntheses"]
%% qianji.node.materialize_syntheses.merge_target:
%%   - syntheses/other.md
"#;

    let annotations = parse_flowhub_graph_annotations(source)
        .unwrap_or_else(|error| panic!("annotations should parse: {error}"))
        .unwrap_or_else(|| panic!("annotations should exist"));
    let graph_name = resolve_flowhub_graph_name(Some(&annotations), None, "paper-deep-read");
    let flowchart = parse_mermaid_flowchart(source, graph_name.as_str(), &["research/paper".into()])
        .unwrap_or_else(|error| panic!("mermaid should parse: {error}"));
    let error = compile_flowhub_scenario_ir(
        std::path::Path::new("paper-deep-read.mmd"),
        graph_name.as_str(),
        &flowchart,
        Some(&annotations),
        None,
    )
    .err()
    .unwrap_or_else(|| panic!("out-of-surface merge target should fail"));

    assert!(error.to_string().contains("outside `qianji.scenario.target_paths`"));
}
