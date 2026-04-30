use crate::test_support::{MustExt as _, data_object_io_bpmn};
use qianji_bpmn_engine::{
    BpmnDataObjectBindingSpec, BpmnEngineError, BpmnParseOptions, BpmnSourceFile,
    BpmnTaskInputSource, parse_bpmn_package,
};

#[test]
fn parser_service_task_preserves_native_io_bindings() {
    let package = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "service-task-io.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_service_task_io">
  <bpmn:process id="service_task_io" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="enrich">
      <bpmn:ioSpecification>
        <bpmn:dataInput id="enrich_customer" name="customer"/>
        <bpmn:dataInput id="enrich_mode" name="mode"/>
        <bpmn:dataOutput id="enrich_approval" name="approval"/>
        <bpmn:inputSet>
          <bpmn:dataInputRefs>enrich_customer</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>enrich_mode</bpmn:dataInputRefs>
        </bpmn:inputSet>
        <bpmn:outputSet>
          <bpmn:dataOutputRefs>enrich_approval</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>customerRecord</bpmn:sourceRef>
        <bpmn:targetRef>enrich_customer</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:targetRef>enrich_mode</bpmn:targetRef>
        <bpmn:assignment>
          <bpmn:from>{"priority":"fast"}</bpmn:from>
          <bpmn:to>enrich_mode</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>enrich_approval</bpmn:sourceRef>
        <bpmn:targetRef>review.approval</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="enrich" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="enrich" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must("service task IO metadata should parse");
    let process = package
        .find_process("service_task_io")
        .must("process should be present");
    let task_io = process.nodes[1]
        .task_io
        .as_ref()
        .must("service task should preserve task IO");

    assert_eq!(task_io.inputs.len(), 2);
    assert_eq!(task_io.inputs[0].name.as_ref(), "customer");
    assert_eq!(
        task_io.inputs[0].source,
        BpmnTaskInputSource::variable("customerRecord")
    );
    assert_eq!(task_io.inputs[1].name.as_ref(), "mode");
    assert_eq!(
        task_io.inputs[1].source,
        BpmnTaskInputSource::literal(r#"{"priority":"fast"}"#)
    );
    assert_eq!(task_io.outputs.len(), 1);
    assert_eq!(task_io.outputs[0].name.as_ref(), "approval");
    assert_eq!(task_io.outputs[0].target_ref.as_ref(), "review.approval");
    assert!(task_io.outputs[0].required);
}

#[test]
fn parser_service_task_resolves_data_object_reference_io_bindings() {
    let package = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "service-task-data-object-io.bpmn",
            data_object_io_bpmn(),
        )],
        &BpmnParseOptions::default(),
    )
    .must("data object task IO metadata should parse");
    let process = package
        .find_process("service_task_data_object_io")
        .must("process should be present");
    let task_io = process.nodes[1]
        .task_io
        .as_ref()
        .must("service task should preserve task IO");

    assert_eq!(
        process.data_object_bindings,
        vec![
            BpmnDataObjectBindingSpec::object("OrderData"),
            BpmnDataObjectBindingSpec::reference("OrderRef", "OrderData"),
        ]
    );
    assert_eq!(task_io.inputs.len(), 1);
    assert_eq!(task_io.inputs[0].name.as_ref(), "order");
    assert_eq!(
        task_io.inputs[0].source,
        BpmnTaskInputSource::variable("OrderData")
    );
    assert_eq!(task_io.outputs.len(), 1);
    assert_eq!(task_io.outputs[0].name.as_ref(), "decision");
    assert_eq!(task_io.outputs[0].target_ref.as_ref(), "OrderData");
}

#[test]
fn parser_service_task_rejects_unknown_data_object_reference_target() {
    let error = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "unknown-data-object-reference.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_unknown_data_object_reference">
  <bpmn:process id="unknown_data_object_reference" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:dataObjectReference id="OrderRef" dataObjectRef="MissingOrderData" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("dataObjectReference should require an existing dataObject");

    assert_eq!(
        error,
        BpmnEngineError::UnknownDataObjectReference {
            process_id: "unknown_data_object_reference".to_string(),
            reference_id: "OrderRef".to_string(),
            data_object_ref: "MissingOrderData".to_string(),
        }
    );
}

#[test]
fn parser_service_task_rejects_multiple_io_source_refs() {
    let error = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "service-task-multiple-sources.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_service_task_multiple_sources">
  <bpmn:process id="service_task_multiple_sources" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="enrich">
      <bpmn:ioSpecification>
        <bpmn:dataInput id="enrich_customer" name="customer"/>
        <bpmn:inputSet><bpmn:dataInputRefs>enrich_customer</bpmn:dataInputRefs></bpmn:inputSet>
        <bpmn:outputSet/>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>customerA</bpmn:sourceRef>
        <bpmn:sourceRef>customerB</bpmn:sourceRef>
        <bpmn:targetRef>enrich_customer</bpmn:targetRef>
      </bpmn:dataInputAssociation>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="enrich" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="enrich" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("multiple IO source refs should be rejected");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: "service_task_multiple_sources".to_string(),
            node_id: "enrich".to_string(),
            detail: "task_io_multiple_source_refs_deferred",
        }
    );
}

#[test]
fn parser_service_task_rejects_io_transformation() {
    let error = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "service-task-transformation.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_service_task_transformation">
  <bpmn:process id="service_task_transformation" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="enrich">
      <bpmn:ioSpecification>
        <bpmn:dataInput id="enrich_customer" name="customer"/>
        <bpmn:inputSet><bpmn:dataInputRefs>enrich_customer</bpmn:dataInputRefs></bpmn:inputSet>
        <bpmn:outputSet/>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>customer</bpmn:sourceRef>
        <bpmn:targetRef>enrich_customer</bpmn:targetRef>
        <bpmn:transformation>customer.id</bpmn:transformation>
      </bpmn:dataInputAssociation>
    </bpmn:serviceTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="enrich" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="enrich" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("IO transformation should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: "service_task_transformation".to_string(),
            node_id: "enrich".to_string(),
            detail: "task_io_transformation_deferred",
        }
    );
}
