use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnEdgeSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnProcessSpec, BpmnSourceFile, BusinessRuleTaskOutcome, BusinessRuleTaskRequest,
    DmnDecisionRef, DmnEvaluationRequest, DmnEvaluationResult, DmnSourceFile,
    PendingHostWorkRequest, PendingHostWorkResult, ProcessKey, create_instance, lint_bpmn_source,
    lint_dmn_source, state_key,
};

#[test]
fn bpmn_engine_dependency_smoke() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg", "review", "digest"),
        vec![BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent)],
        vec![BpmnEdgeSpec::new(0, 0, None::<&str>)],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg", vec![process]));
    let state = create_instance(
        package,
        "review",
        BpmnInstanceInit::new("wf_qianji", json!({ "scope": "linkage" }), 1),
    )
    .unwrap_or_else(|error| panic!("qianji should compile against the scaffold crate: {error:?}"));
    let checkpoint = BpmnCheckpointEnvelope::from_state(state);
    assert_eq!(checkpoint.state.process.process_id.as_ref(), "review");
    assert_eq!(state_key("wf_qianji"), "xq:bpmn:ckpt:wf_qianji:state");

    let business_rule_request = PendingHostWorkRequest::BusinessRule(BusinessRuleTaskRequest {
        instance_id: "wf_qianji".to_string(),
        token_id: 0,
        node_index: 3,
        evaluation: DmnEvaluationRequest::new(
            DmnDecisionRef::new("loan-decision"),
            json!({ "scope": "linkage" }),
        ),
        inputs: json!({}),
        output_bindings: vec![],
        repeat: None,
    });
    assert_eq!(business_rule_request.kind_name(), "business_rule");

    let business_rule_result = PendingHostWorkResult::BusinessRule(BusinessRuleTaskOutcome {
        evaluation: DmnEvaluationResult::new(
            "loan-decision",
            json!({ "approved": true }),
            vec![std::sync::Arc::<str>::from("rule_1")],
        ),
    });
    assert_eq!(business_rule_result.kind_name(), "business_rule");
    assert_eq!(business_rule_result.data(), &json!({ "approved": true }));

    let bpmn_report = lint_bpmn_source(&BpmnSourceFile::new(
        "invalid-lint.bpmn",
        r#"<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"><bpmn:process id="gateway_flow"><bpmn:startEvent id="start" /><bpmn:inclusiveGateway id="decision" /><bpmn:endEvent id="end" /><bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="decision" /><bpmn:sequenceFlow id="flow_2" sourceRef="decision" targetRef="end" /></bpmn:process></bpmn:definitions>"#,
    ));
    assert!(!bpmn_report.ok);
    assert_eq!(
        bpmn_report.issues[0].code,
        "bpmn.unsupported_gateway_configuration"
    );

    let dmn_report = lint_dmn_source(&DmnSourceFile::new(
        "invalid-lint.dmn",
        r#"<definitions xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/" id="Definitions_invalid" name="Invalid Unary Test" namespace="http://example.com/dmn"><decision id="decision_1" name="Decision One"><decisionTable id="table_1" hitPolicy="UNIQUE"><input id="input_1" label="window"><inputExpression id="input_expression_1" typeRef="dayTimeDuration"><text>window</text></inputExpression></input><output id="output_1" name="result" label="result" typeRef="string" /><rule id="rule_1"><inputEntry id="input_entry_1"><text>duration(\"P1.5Y\")</text></inputEntry><outputEntry id="output_entry_1"><text>\"review\"</text></outputEntry></rule></decisionTable></decision></definitions>"#,
    ));
    assert!(!dmn_report.ok);
    assert_eq!(dmn_report.issues[0].code, "dmn.unsupported_unary_test");
}
