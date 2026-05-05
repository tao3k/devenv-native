use super::{ConstructCliCommand, must_ok, must_some, run_construct_command};

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
    assert!(output.rendered.contains("!approved"));
    assert!(
        output
            .rendered
            .contains("conditionExpression on the default sequenceFlow")
    );
    assert!(
        output
            .rendered
            .contains("default sequenceFlow is one of the gateway's outgoing flows")
    );
    assert!(
        output
            .rendered
            .contains("two-way boolean routing uses one conditional true branch")
    );
    assert!(
        output
            .rendered
            .contains("paired boolean conditions such as `ready` and `not ready`")
    );
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
fn run_construct_show_renders_parallel_multi_instance_scaffold() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Show {
            id: "service-task.multi-instance.parallel".to_string(),
            json: false,
        }),
        "parallel multi-instance card should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Construct Card: service-task.multi-instance.parallel")
    );
    assert!(output.rendered.contains("multiInstanceLoopCharacteristics"));
    assert!(output.rendered.contains("isSequential=\"false\""));
    assert!(
        output
            .rendered
            .contains("<bpmn:loopDataInputRef>agentTasks")
    );
    assert!(
        output
            .rendered
            .contains("<bpmn:inputDataItem id=\"agentTask\"")
    );
    assert!(
        output
            .rendered
            .contains("<bpmn:loopDataOutputRef>agentResults")
    );
    assert!(
        output
            .rendered
            .contains("<bpmn:outputDataItem id=\"agentResult\"")
    );
    assert!(
        output
            .rendered
            .contains("hiding per-item parallel dispatch inside one serviceTask prompt")
    );
    assert!(
        output
            .rendered
            .contains("bpmn.unsupported_loop_configuration")
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
    assert!(output.rendered.contains("<dataInput"));
    assert!(
        output
            .rendered
            .contains("<dataInput id=\"Task_Answer_Input_question\" name=\"question\"")
    );
    assert!(output.rendered.contains("<from>choice_input</from>"));
    assert!(
        output
            .rendered
            .contains("<sourceRef>currentChoices</sourceRef>")
    );
    assert!(
        output
            .rendered
            .contains("JSON array objects with required value")
    );
    assert!(
        output
            .rendered
            .contains("answers to subagent questions, missing context, or escalation handling")
    );
    assert!(
        output
            .rendered
            .contains("Ask the user for approval, selection, missing context, or escalation")
    );
    assert!(output.rendered.contains("required value"));
    assert!(output.rendered.contains("empty option lists"));
    assert!(
        output
            .rendered
            .contains("numbered option prose embedded inside currentQuestion")
    );
    assert!(output.rendered.contains("name=\"freeText\""));
    assert!(output.rendered.contains("loop.interactive.progress"));
}

#[test]
fn run_construct_show_renders_interactive_loop_progress_scaffold() {
    let output = must_ok(
        run_construct_command(&ConstructCliCommand::Show {
            id: "loop.interactive.progress".to_string(),
            json: false,
        }),
        "interactive loop progress card should render",
    );

    assert!(
        output
            .rendered
            .starts_with("# Qianji Construct Card: loop.interactive.progress")
    );
    assert!(output.rendered.contains("Task_PrepareNextQuestion"));
    assert!(output.rendered.contains("Task_AnswerQuestion"));
    assert!(
        output
            .rendered
            .contains("Task_PrepareNextQuestion_Input_userAnswer")
    );
    assert!(
        output
            .rendered
            .contains("Task_PrepareNextQuestion_Output_questionsRemaining")
    );
    assert!(
        output
            .rendered
            .contains("currentChoices is emitted as JSON array objects")
    );
    assert!(
        output
            .rendered
            .contains("<sourceRef>currentChoices</sourceRef>")
    );
    assert!(output.rendered.contains("Flow_Answer_Prepare"));
    assert!(output.rendered.contains("questionsRemaining &gt; 0"));
    assert!(
        output
            .rendered
            .contains("bpmn.loop_risk.unbounded_control_cycle")
    );
    assert!(
        output
            .rendered
            .contains("pi-wendao.runtime.user_prompt_stall")
    );
    assert!(
        output
            .rendered
            .contains("pi-wendao.runtime.invalid_dynamic_choices")
    );
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
    assert!(forbids.iter().any(|value| {
        value
            .as_str()
            .is_some_and(|text| text.contains("questionsRemaining"))
    }));
    assert!(forbids.iter().any(|value| {
        value
            .as_str()
            .is_some_and(|text| text.contains("conditionExpression on the default"))
    }));
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
