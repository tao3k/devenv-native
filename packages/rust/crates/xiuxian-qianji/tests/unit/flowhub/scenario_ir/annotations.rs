use crate::flowhub::scenario_ir::parse_flowhub_graph_annotations;

#[test]
fn parse_flowhub_graph_annotations_reads_scenario_and_node_lists() {
    let source = r#"
%% qianji.scenario.id: deep_read
%% qianji.scenario.name: PAPER_DEEP_READ
%% qianji.scenario.topology: bounded_loop
%% qianji.scenario.workdir_root: runs/<run_id>
%% qianji.scenario.requires:
%%   - refs/paper.json
%%   - refs/topic.json
%% qianji.scenario.target_root: papers/<paper_id>
%% qianji.scenario.target_paths:
%%   - syntheses/deep_read.md
flowchart LR
  A["research/paper"] --> B["materialize_syntheses"]
  B --> C["done gate"]
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

    assert_eq!(annotations.scenario.id.as_deref(), Some("deep_read"));
    assert_eq!(annotations.scenario.name.as_deref(), Some("PAPER_DEEP_READ"));
    assert_eq!(
        annotations.scenario.workdir_root.as_deref(),
        Some("runs/<run_id>")
    );
    assert_eq!(
        annotations.scenario.requires,
        vec!["refs/paper.json".to_string(), "refs/topic.json".to_string()]
    );
    assert_eq!(
        annotations.scenario.target_paths,
        vec!["syntheses/deep_read.md".to_string()]
    );
    assert_eq!(
        annotations
            .nodes
            .get("materialize_syntheses")
            .and_then(|node| node.checkpoint.as_deref()),
        Some("checkpoints/materialize_syntheses.json")
    );
    assert_eq!(
        annotations
            .nodes
            .get("done gate")
            .and_then(|node| node.kind.as_deref()),
        Some("gate")
    );
    assert_eq!(
        annotations.done_gate_require,
        vec!["syntheses/deep_read.md".to_string()]
    );
}

#[test]
fn parse_flowhub_graph_annotations_rejects_unknown_keys() {
    let error = parse_flowhub_graph_annotations(
        r#"
%% qianji.scenario.unknown: nope
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("unknown annotations should fail"));

    assert!(error.to_string().contains("unsupported"));
}
