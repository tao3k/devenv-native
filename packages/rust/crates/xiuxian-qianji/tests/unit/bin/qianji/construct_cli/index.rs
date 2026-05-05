use super::{ConstructCliCommand, must_ok, run_construct_command};

#[test]
fn run_construct_index_renders_toc() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Index { json: false }),
        "construct index should render",
    );

    assert!(output.rendered.starts_with("# Qianji Construct Index"));
    assert!(output.rendered.contains("source task or `SKILL.md`"));
    assert!(
        output
            .rendered
            .contains("autonomous workflow, interactive workflow, or planning workflow")
    );
    assert!(output.rendered.contains("semantic input"));
    assert!(
        output
            .rendered
            .contains("answers to subagent questions, missing context")
    );
    assert!(output.rendered.contains("fill a BPMN or DMN scaffold"));
    assert!(output.rendered.contains("service-task.agent"));
    assert!(
        output
            .rendered
            .contains("service-task.multi-instance.parallel")
    );
    assert!(output.rendered.contains("loop.interactive.progress"));
    assert!(output.rendered.contains("gateway.exclusive.bounded"));
    assert!(output.rendered.contains("dmn.decision-table.unique"));
}

#[test]
fn run_construct_index_renders_json() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Index { json: true }),
        "construct index json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "index output should be valid json",
    );

    assert_eq!(json[0]["id"], "service-task.agent");
    assert_eq!(json[1]["id"], "service-task.multi-instance.parallel");
    assert_eq!(json[3]["id"], "loop.interactive.progress");
    assert_eq!(json[3]["status"], "draft");
    assert_eq!(json[4]["id"], "gateway.exclusive.bounded");
    assert_eq!(json[4]["status"], "stable");
}
