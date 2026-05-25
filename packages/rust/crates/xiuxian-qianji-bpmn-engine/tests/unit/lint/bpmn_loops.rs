use super::{
    BpmnSourceFile, LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source,
    native_service_task, native_user_task,
};

#[test]
fn bpmn_linter_reports_invalid_standard_loop_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-standard-loop-missing-limit.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("bpmn_standard_loop_missing_limit_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_parallel_multi_instance_completion_condition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-multi-instance-deferred.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("completionCondition"));
}

#[test]
fn bpmn_linter_reports_missing_multi_instance_cardinality_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-sequential-multi-instance-missing-cardinality.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("loopCardinality"));
    assert!(issue.llm_fix_prompt.contains("loopDataInputRef"));
}

#[test]
fn bpmn_linter_reports_in_place_multi_instance_output_binding_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-sequential-multi-instance-in-place-output.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("loopDataOutputRef"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("different from the input path")
    );
}

#[test]
fn bpmn_linter_reports_interaction_loop_missing_feedback_progress() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-interaction-loop-missing-feedback.bpmn",
        interactive_loop_source(""),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.loop_risk.unbounded_control_cycle");
    assert!(issue.summary.contains("prepare_question"));
    assert!(issue.summary.contains("ask_user"));
    assert!(issue.llm_fix_prompt.contains("dataInputAssociation"));
    assert!(issue.llm_fix_prompt.contains("answer"));
    assert_eq!(
        issue.structured_repair.as_ref().and_then(|repair| {
            repair
                .get("line_fixes")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len)
        }),
        Some(1)
    );
    assert_eq!(
        issue
            .structured_repair
            .as_ref()
            .and_then(|repair| repair.get("contract_message"))
            .and_then(serde_json::Value::as_str),
        Some(
            "native BPMN loop progress requires in-cycle tasks to consume user feedback and emit the gateway route state through standard IO metadata."
        )
    );
    assert!(
        issue
            .source_diagnostic
            .as_ref()
            .is_some_and(|diagnostic| diagnostic.help.contains("keep flow_done"))
    );
    assert_eq!(
        issue.evidence["missing_feedback_inputs"],
        serde_json::json!(["answer"])
    );
    assert!(
        issue.evidence["missing_progress_outputs"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
    assert!(issue.source_diagnostic.is_some());
    assert_lint_json_snapshot(
        "bpmn_interaction_loop_missing_feedback_lint_report",
        &report,
    );
}

#[test]
fn bpmn_linter_accepts_interaction_loop_with_explicit_progress_contract() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-interaction-loop-progress.bpmn",
        interactive_loop_source("answer"),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}

#[test]
fn bpmn_linter_reports_default_branch_reentering_loop() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-default-reentry-loop.bpmn",
        default_reentry_loop_source(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.loop_risk.unbounded_control_cycle");
    assert!(issue.summary.contains("evaluate_safety"));
    assert!(issue.summary.contains("emergency_gate"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("<sequenceFlow id=\"flow_normal\" sourceRef=\"emergency_gate\" targetRef=\"normal_intake\"/>")
    );
    assert_eq!(
        issue.evidence["default_reentry_flows"],
        serde_json::json!([{
            "gateway_id": "emergency_gate",
            "flow_id": "flow_normal",
            "target_id": "evaluate_safety",
            "suggested_exit_target_id": "normal_intake"
        }])
    );
    assert_lint_json_snapshot("bpmn_default_reentry_loop_lint_report", &report);
}

fn interactive_loop_source(service_inputs: &str) -> String {
    let service_inputs = if service_inputs.is_empty() {
        Vec::new()
    } else {
        service_inputs.split(',').collect::<Vec<_>>()
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  id="pkg_interaction_loop">
  <bpmn:process id="interaction_loop" isExecutable="true">
    <bpmn:startEvent id="start" />
    {}
    <bpmn:exclusiveGateway id="more_questions" default="flow_done" />
    {}
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_question" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="prepare_question" targetRef="more_questions" />
    <bpmn:sequenceFlow id="flow_ask" sourceRef="more_questions" targetRef="ask_user">
      <bpmn:conditionExpression>hasMoreQuestions</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="more_questions" targetRef="done" />
    <bpmn:sequenceFlow id="flow_repeat" sourceRef="ask_user" targetRef="prepare_question" />
  </bpmn:process>
</bpmn:definitions>"#,
        native_service_task(
            "prepare_question",
            "Return JSON with the next currentQuestion, currentChoices, and hasMoreQuestions.",
            &service_inputs,
            &["currentQuestion", "currentChoices", "hasMoreQuestions"],
        ),
        native_user_task(
            "ask_user",
            "Ask the current question.",
            "choice_input",
            &["currentQuestion"],
            Some("currentChoices"),
            "answer",
        )
    )
}

fn default_reentry_loop_source() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  id="pkg_default_reentry_loop">
  <bpmn:process id="default_reentry_loop" isExecutable="true">
    <bpmn:startEvent id="start" />
    {}
    {}
    {}
    <bpmn:exclusiveGateway id="emergency_gate" default="flow_normal" />
    <bpmn:serviceTask id="handle_emergency" implementation="${{environment.services.runAgent}}" />
    <bpmn:serviceTask id="normal_intake" implementation="${{environment.services.runAgent}}" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_safety" />
    <bpmn:sequenceFlow id="flow_prepare" sourceRef="prepare_safety" targetRef="safety_screen" />
    <bpmn:sequenceFlow id="flow_answer" sourceRef="safety_screen" targetRef="evaluate_safety" />
    <bpmn:sequenceFlow id="flow_eval" sourceRef="evaluate_safety" targetRef="emergency_gate" />
    <bpmn:sequenceFlow id="flow_emergency" sourceRef="emergency_gate" targetRef="handle_emergency">
      <bpmn:conditionExpression>isEmergency</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_normal" sourceRef="emergency_gate" targetRef="evaluate_safety" />
    <bpmn:sequenceFlow id="flow_emergency_done" sourceRef="handle_emergency" targetRef="done" />
    <bpmn:sequenceFlow id="flow_normal_done" sourceRef="normal_intake" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        native_service_task(
            "prepare_safety",
            "Return safetyQuestion and safetyChoices.",
            &[],
            &["safetyQuestion", "safetyChoices"],
        ),
        native_user_task(
            "safety_screen",
            "Ask the safety question.",
            "choice_input",
            &["safetyQuestion"],
            Some("safetyChoices"),
            "safetyAnswer",
        ),
        native_service_task(
            "evaluate_safety",
            "Evaluate safetyAnswer and return isEmergency.",
            &["safetyAnswer"],
            &["isEmergency"],
        )
    )
}
