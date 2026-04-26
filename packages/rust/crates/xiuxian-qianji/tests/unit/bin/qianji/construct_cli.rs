use super::*;

#[test]
fn parse_construct_index_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&["qianji", "construct", "index"])),
                "construct index parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Index { json: false },
    );
}

#[test]
fn parse_construct_show_command() {
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&[
                    "qianji",
                    "construct",
                    "show",
                    "gateway.exclusive.bounded",
                ])),
                "construct show parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Show {
            id: "gateway.exclusive.bounded".to_string(),
            json: false,
        },
    );
}

#[test]
fn parse_construct_json_commands() {
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&["qianji", "construct", "index", "--json"])),
                "construct index json parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Index { json: true },
    );
    assert_eq!(
        must_some(
            must_ok(
                parse_construct_command(&to_args(&[
                    "qianji",
                    "construct",
                    "show",
                    "gateway.exclusive.bounded",
                    "--json",
                ])),
                "construct show json parse should succeed",
            ),
            "construct command should be detected",
        ),
        ConstructCliCommand::Show {
            id: "gateway.exclusive.bounded".to_string(),
            json: true,
        },
    );
}

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
    assert!(output.rendered.contains("fill a BPMN or DMN scaffold"));
    assert!(output.rendered.contains("service-task.agent"));
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
    assert_eq!(json[2]["id"], "gateway.exclusive.bounded");
    assert_eq!(json[2]["status"], "stable");
}

#[test]
fn run_construct_show_renders_card() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Show {
            id: "gateway.exclusive.bounded".to_string(),
            json: false,
        }),
        "construct show should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Construct Card: gateway.exclusive.bounded")
    );
    assert!(output.rendered.contains("## Forbids"));
    assert!(output.rendered.contains("== true or == false"));
    assert!(output.rendered.contains("```xml"));
    assert!(output.rendered.contains("<exclusiveGateway"));
    assert!(output.rendered.contains("conditionExpression"));
    assert!(
        output
            .rendered
            .contains("bpmn.unsupported_gateway_configuration")
    );
}

#[test]
fn run_construct_show_renders_user_interaction_scaffold() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Show {
            id: "user-task.interaction".to_string(),
            json: false,
        }),
        "user interaction card should render",
    );

    assert!(output.rendered.contains("```xml"));
    assert!(output.rendered.contains("<userTask"));
    assert!(
        output
            .rendered
            .contains("<qianji:interaction type=\"choice_input\">")
    );
    assert!(output.rendered.contains("<qianji:freeText"));
}

#[test]
fn run_construct_show_renders_json_card() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Show {
            id: "gateway.exclusive.bounded".to_string(),
            json: true,
        }),
        "construct show json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "show output should be valid json",
    );
    let forbids = must_some(json["forbids"].as_array(), "forbids should be an array");

    assert_eq!(json["id"], "gateway.exclusive.bounded");
    assert_eq!(json["status"], "stable");
    assert!(forbids.iter().any(|value| value == "== true or == false"));
}

#[test]
fn run_construct_show_reports_available_ids_for_unknown_card() {
    let result = run_construct_command(&ConstructCliCommand::Show {
        id: "missing.card".to_string(),
        json: false,
    });
    let Err(error) = result else {
        panic!("unknown construct id should fail");
    };

    let message = error.to_string();
    assert!(message.contains("unknown qianji construct `missing.card`"));
    assert!(message.contains("gateway.exclusive.bounded"));
}
