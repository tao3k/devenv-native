use crate::ir_event_api::{BpmnEventKind, BpmnTimerKind};
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::parser::import::{
    RawAssociation, RawNode, RawParallelMultiInstanceSpec, RawProcess, RawRepeatSpec,
    RawSequenceFlow, RawSequentialMultiInstanceSpec, RawSubProcessKind,
};

pub(super) fn process_digest_hex(package_id: &str, source_id: &str, raw: &RawProcess) -> String {
    let mut material = String::new();
    material.push_str(package_id);
    material.push('\n');
    material.push_str(source_id);
    material.push('\n');
    material.push_str(&raw.process_id);
    material.push('\n');
    for node in &raw.nodes {
        append_node_digest(&mut material, node);
    }
    for flow in &raw.flows {
        append_flow_digest(&mut material, flow);
    }
    for association in &raw.associations {
        append_association_digest(&mut material, association);
    }
    format!("{:x}", md5::compute(material))
}

fn append_node_digest(material: &mut String, node: &RawNode) {
    material.push_str(&node.bpmn_id);
    material.push(':');
    material.push_str(node_kind_name(&node.kind));
    if let Some(gateway_kind) = &node.gateway_kind {
        material.push(':');
        material.push_str(gateway_kind_name(gateway_kind));
    }
    if let Some(decision) = &node.decision {
        material.push(':');
        material.push_str(decision.decision_id.as_ref());
    }
    if let Some(called_process_ref) = &node.called_process_ref {
        material.push(':');
        material.push_str("called_process=");
        material.push_str(called_process_ref);
    }
    if let Some(subprocess_kind) = node.subprocess_kind {
        material.push(':');
        material.push_str("subprocess_kind=");
        material.push_str(subprocess_kind_name(subprocess_kind));
    }
    if let Some(repeat) = &node.repeat {
        append_repeat_digest(material, repeat);
    }
    if let Some(attached_to_ref) = &node.attached_to_ref {
        material.push(':');
        material.push_str("attached_to=");
        material.push_str(attached_to_ref);
        material.push(':');
        material.push_str(if node.cancel_activity {
            "interrupting"
        } else {
            "non_interrupting"
        });
    }
    if node.is_for_compensation {
        material.push(':');
        material.push_str("is_for_compensation=true");
    }
    if let Some(event) = &node.event {
        material.push(':');
        material.push_str(event_kind_name(&event.kind));
        if let Some(reference_id) = &event.reference_id {
            material.push(':');
            material.push_str(reference_id);
        }
        if let Some(name) = &event.name {
            material.push(':');
            material.push_str(name);
        }
        if let Some(timer) = &event.timer {
            material.push(':');
            material.push_str(timer_kind_name(&timer.kind));
            material.push(':');
            material.push_str(&timer.expression);
        }
    }
    material.push('\n');
}

fn append_repeat_digest(material: &mut String, repeat: &RawRepeatSpec) {
    match repeat {
        RawRepeatSpec::StandardLoop(loop_spec) => {
            material.push(':');
            material.push_str("repeat=standard_loop");
            material.push(':');
            material.push_str(if loop_spec.test_before {
                "test_before"
            } else {
                "test_after"
            });
            if let Some(loop_maximum) = loop_spec.loop_maximum {
                material.push(':');
                material.push_str("loop_maximum=");
                material.push_str(&loop_maximum.to_string());
            }
            if let Some(loop_condition) = &loop_spec.loop_condition {
                material.push(':');
                material.push_str("loop_condition=");
                material.push_str(loop_condition);
            }
        }
        RawRepeatSpec::SequentialMultiInstance(loop_spec) => {
            material.push(':');
            material.push_str("repeat=sequential_multi_instance");
            if let Some(loop_cardinality) = loop_spec.loop_cardinality {
                material.push(':');
                material.push_str("loop_cardinality=");
                material.push_str(&loop_cardinality.to_string());
            }
            append_sequential_multi_instance_data_binding_digest(material, loop_spec);
            if let Some(completion_condition) = &loop_spec.completion_condition {
                material.push(':');
                material.push_str("completion_condition=");
                material.push_str(completion_condition);
            }
        }
        RawRepeatSpec::ParallelMultiInstance(loop_spec) => {
            material.push(':');
            material.push_str("repeat=parallel_multi_instance");
            if let Some(loop_cardinality) = loop_spec.loop_cardinality {
                material.push(':');
                material.push_str("loop_cardinality=");
                material.push_str(&loop_cardinality.to_string());
            }
            append_parallel_multi_instance_data_binding_digest(material, loop_spec);
            if let Some(completion_condition) = &loop_spec.completion_condition {
                material.push(':');
                material.push_str("completion_condition=");
                material.push_str(completion_condition);
            }
        }
    }
}

fn append_sequential_multi_instance_data_binding_digest(
    material: &mut String,
    loop_spec: &RawSequentialMultiInstanceSpec,
) {
    append_multi_instance_data_binding_digest(
        material,
        loop_spec.loop_data_input_ref.as_deref(),
        loop_spec.input_data_item.as_deref(),
        loop_spec.loop_data_output_ref.as_deref(),
        loop_spec.output_data_item.as_deref(),
    );
}

fn append_parallel_multi_instance_data_binding_digest(
    material: &mut String,
    loop_spec: &RawParallelMultiInstanceSpec,
) {
    append_multi_instance_data_binding_digest(
        material,
        loop_spec.loop_data_input_ref.as_deref(),
        loop_spec.input_data_item.as_deref(),
        loop_spec.loop_data_output_ref.as_deref(),
        loop_spec.output_data_item.as_deref(),
    );
}

fn append_multi_instance_data_binding_digest(
    material: &mut String,
    loop_data_input_ref: Option<&str>,
    input_data_item: Option<&str>,
    loop_data_output_ref: Option<&str>,
    output_data_item: Option<&str>,
) {
    if let Some(loop_data_input_ref) = loop_data_input_ref {
        material.push(':');
        material.push_str("loop_data_input_ref=");
        material.push_str(loop_data_input_ref);
    }
    if let Some(input_data_item) = input_data_item {
        material.push(':');
        material.push_str("input_data_item=");
        material.push_str(input_data_item);
    }
    if let Some(loop_data_output_ref) = loop_data_output_ref {
        material.push(':');
        material.push_str("loop_data_output_ref=");
        material.push_str(loop_data_output_ref);
    }
    if let Some(output_data_item) = output_data_item {
        material.push(':');
        material.push_str("output_data_item=");
        material.push_str(output_data_item);
    }
}

fn append_flow_digest(material: &mut String, flow: &RawSequenceFlow) {
    material.push_str(&flow.flow_id);
    material.push(':');
    material.push_str(&flow.source_ref);
    material.push(':');
    material.push_str(&flow.target_ref);
    if let Some(label) = &flow.label {
        material.push(':');
        material.push_str(label);
    }
    material.push('\n');
}

fn node_kind_name(kind: &BpmnNodeKind) -> &'static str {
    match kind {
        BpmnNodeKind::StartEvent => "start_event",
        BpmnNodeKind::EndEvent => "end_event",
        BpmnNodeKind::IntermediateCatchEvent => "intermediate_catch_event",
        BpmnNodeKind::BoundaryEvent => "boundary_event",
        BpmnNodeKind::ServiceTask => "service_task",
        BpmnNodeKind::UserTask => "user_task",
        BpmnNodeKind::ManualTask => "manual_task",
        BpmnNodeKind::BusinessRuleTask => "business_rule_task",
        BpmnNodeKind::Gateway => "gateway",
        BpmnNodeKind::SubProcess => "sub_process",
    }
}

fn event_kind_name(kind: &BpmnEventKind) -> &'static str {
    match kind {
        BpmnEventKind::Timer => "timer",
        BpmnEventKind::Message => "message",
        BpmnEventKind::Signal => "signal",
        BpmnEventKind::Error => "error",
        BpmnEventKind::Cancel => "cancel",
        BpmnEventKind::Compensation => "compensation",
        BpmnEventKind::Conditional => "conditional",
    }
}

fn append_association_digest(material: &mut String, association: &RawAssociation) {
    material.push_str(&association.association_id);
    material.push(':');
    material.push_str(&association.source_ref);
    material.push(':');
    material.push_str(&association.target_ref);
    material.push('\n');
}

fn subprocess_kind_name(kind: RawSubProcessKind) -> &'static str {
    match kind {
        RawSubProcessKind::CallActivity => "call_activity",
        RawSubProcessKind::EmbeddedSubProcess => "embedded",
        RawSubProcessKind::Transaction => "transaction",
    }
}

fn gateway_kind_name(kind: &BpmnGatewayKind) -> &'static str {
    match kind {
        BpmnGatewayKind::Parallel => "parallel",
        BpmnGatewayKind::Exclusive => "exclusive",
        BpmnGatewayKind::EventBased => "event_based",
    }
}

fn timer_kind_name(kind: &BpmnTimerKind) -> &'static str {
    match kind {
        BpmnTimerKind::Date => "date",
        BpmnTimerKind::Duration => "duration",
        BpmnTimerKind::Cycle => "cycle",
    }
}
