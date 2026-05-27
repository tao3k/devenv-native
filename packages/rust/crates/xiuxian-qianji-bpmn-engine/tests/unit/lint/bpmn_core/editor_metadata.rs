use crate::lint::{BpmnSourceFile, LintDomain, lint_bpmn_source};

#[test]
fn bpmn_linter_accepts_non_executable_editor_gateway_without_runtime_conditions() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "non-executable-editor-gateway.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
                  xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
                  id="pkg_non_executable_editor_gateway">
  <bpmn:process id="editor_gateway" isExecutable="false">
    <bpmn:startEvent id="start" />
    <bpmn:exclusiveGateway id="decision" />
    <bpmn:task id="retry" />
    <bpmn:task id="continue" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_retry" sourceRef="decision" targetRef="retry" />
    <bpmn:sequenceFlow id="flow_continue" sourceRef="decision" targetRef="continue" />
    <bpmn:sequenceFlow id="flow_retry_back" sourceRef="retry" targetRef="decision" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="continue" targetRef="done" />
  </bpmn:process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="editor_gateway">
      <bpmndi:BPMNShape id="decision_di" bpmnElement="decision">
        <dc:Bounds x="260" y="160" width="50" height="50" />
      </bpmndi:BPMNShape>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn bpmn_linter_accepts_missing_executable_flag_as_editor_metadata() {
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        "default-non-executable-custom-meta.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
                  xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
                  xmlns:qa="http://some-company/schema/bpmn/qa"
                  id="pkg_default_non_executable_custom_meta">
  <bpmn:process id="custom_meta">
    <bpmn:task id="inspect" qa:suitable="0.7">
      <bpmn:outgoing>flow_done</bpmn:outgoing>
      <bpmn:extensionElements>
        <qa:analysisDetails lastChecked="2015-01-20" />
      </bpmn:extensionElements>
    </bpmn:task>
    <bpmn:sequenceFlow id="flow_done" sourceRef="inspect" targetRef="done" />
    <bpmn:endEvent id="done">
      <bpmn:incoming>flow_done</bpmn:incoming>
      <bpmn:messageEventDefinition id="MessageEventDefinition_1" />
    </bpmn:endEvent>
  </bpmn:process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="custom_meta">
      <bpmndi:BPMNShape id="Task_1_di" bpmnElement="inspect">
        <dc:Bounds x="96" y="196" width="100" height="80" />
      </bpmndi:BPMNShape>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>"#,
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}
