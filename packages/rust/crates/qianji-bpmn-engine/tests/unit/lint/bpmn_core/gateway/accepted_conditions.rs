use super::super::{BpmnSourceFile, LintDomain, lint_bpmn_source};

#[test]
fn bpmn_linter_accepts_boolean_route_names_that_end_with_context() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-human-provide-context-route.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_context_route">
  <bpmn:process id="context_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="human_decision">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Ask whether the human should provide context.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>humanProvideContext</qianji:outputs>
          <qianji:interaction type="confirm">
            <qianji:question>Provide more context?</qianji:question>
            <qianji:result output="humanProvideContext"/>
          </qianji:interaction>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:serviceTask id="provide" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Provide context.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>provided</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="human_decision" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="human_decision" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_provide" sourceRef="decision" targetRef="provide">
      <bpmn:conditionExpression>humanProvideContext</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_provide_done" sourceRef="provide" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}
#[test]
fn bpmn_linter_accepts_boolean_route_names_with_embedded_marker() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-status-is-done-with-concerns-route.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_status_route">
  <bpmn:process id="status_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="dispatch" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return statusIsDoneWithConcerns as a JSON boolean.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>statusIsDoneWithConcerns</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:exclusiveGateway id="decision" default="flow_done" />
    <bpmn:serviceTask id="review" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Review concerns.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>reviewed</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="dispatch" />
    <bpmn:sequenceFlow id="flow_decision" sourceRef="dispatch" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_review" sourceRef="decision" targetRef="review">
      <bpmn:conditionExpression>statusIsDoneWithConcerns</bpmn:conditionExpression>
    </bpmn:sequenceFlow>
    <bpmn:sequenceFlow id="flow_done" sourceRef="decision" targetRef="done" />
    <bpmn:sequenceFlow id="flow_review_done" sourceRef="review" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}
#[test]
fn bpmn_linter_accepts_escaped_numeric_gateway_condition() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "valid-escaped-numeric-gateway-condition.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:qianji="https://qianji.dev/bpmn/extensions"
                  id="pkg_count_like">
  <bpmn:process id="count_like_condition" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="make_question" implementation="${environment.services.runAgent}">
      <bpmn:extensionElements>
        <qianji:config>
          <qianji:prompt>Return JSON with questionsRemaining as a number.</qianji:prompt>
          <qianji:tools></qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>questionsRemaining</qianji:outputs>
        </qianji:config>
      </bpmn:extensionElements>
    </bpmn:serviceTask>
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
</bpmn:definitions>"#
            .to_string(),
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok, "{report:#?}");
}
