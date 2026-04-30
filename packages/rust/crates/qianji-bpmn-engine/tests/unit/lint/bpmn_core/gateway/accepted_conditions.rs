use super::super::super::{
    BpmnSourceFile, LintDomain, lint_bpmn_source, native_service_task, native_user_task,
};

#[test]
fn bpmn_linter_accepts_boolean_route_names_that_end_with_context() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-human-provide-context-route.bpmn",
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  id="pkg_context_route">
  <bpmn:process id="context_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    {}
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    {}
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="human_decision" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="human_decision" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_provide" sourceRef="decision" targetRef="provide">
      <bpmn:conditionExpression>humanProvideContext</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_provide_done" sourceRef="provide" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
            native_user_task(
                "human_decision",
                "Provide more context?",
                "confirm",
                &[],
                None,
                "humanProvideContext",
            ),
            native_service_task("provide", "Provide context.", &[], &["provided"])
        ),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}

#[test]
fn bpmn_linter_accepts_boolean_route_names_with_embedded_marker() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-status-is-done-with-concerns-route.bpmn",
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  id="pkg_status_route">
  <bpmn:process id="status_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    {}
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    {}
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="dispatch" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="dispatch" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_review" sourceRef="decision" targetRef="review">
      <bpmn:conditionExpression>statusIsDoneWithConcerns</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_review_done" sourceRef="review" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
            native_service_task(
                "dispatch",
                "Return statusIsDoneWithConcerns as a JSON boolean.",
                &[],
                &["statusIsDoneWithConcerns"],
            ),
            native_service_task("review", "Review concerns.", &[], &["reviewed"])
        ),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}

#[test]
fn bpmn_linter_accepts_escaped_numeric_gateway_condition() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-escaped-numeric-gateway-condition.bpmn",
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  id="pkg_count_like">
  <bpmn:process id="count_like_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    {}
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:userTask id="ask" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="make_question" />
    <bpmn:sequenceFlow id="flow_gateway" sourceRef="make_question" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_more" sourceRef="decision" targetRef="ask">
      <bpmn:conditionExpression>questionsRemaining &gt; 0</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_ask_done" sourceRef="ask" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
            native_service_task(
                "make_question",
                "Return JSON with questionsRemaining as a number.",
                &[],
                &["questionsRemaining"],
            )
        ),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}
