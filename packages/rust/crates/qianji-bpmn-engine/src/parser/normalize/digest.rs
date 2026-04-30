use crate::ir_event_api::{BpmnEventKind, BpmnTimerKind};
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::parser::import::{
    RawAssociation, RawHumanTaskAssignmentSpec, RawHumanTaskFormSpec, RawHumanTaskResourceRoleSpec,
    RawLaneMembershipSpec, RawNode, RawParallelMultiInstanceSpec, RawProcess, RawRepeatSpec,
    RawScriptTaskSpec, RawSequenceFlow, RawSequentialMultiInstanceSpec, RawSubProcessKind,
    RawTaskInputSource, RawTaskIoSpec,
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
    if let Some(script_task) = &node.script_task {
        append_script_task_digest(material, script_task);
    }
    if let Some(form) = &node.human_task_form {
        append_human_task_form_digest(material, form);
    }
    if let Some(assignment) = &node.human_task_assignment {
        append_human_task_assignment_digest(material, assignment);
    }
    if let Some(task_io) = &node.task_io {
        append_task_io_digest(material, task_io);
    }
    if let Some(lane) = &node.lane {
        append_lane_membership_digest(material, lane);
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
        if !event.wait_for_completion {
            material.push(':');
            material.push_str("wait_for_completion=false");
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

fn append_task_io_digest(material: &mut String, task_io: &RawTaskIoSpec) {
    for input in &task_io.inputs {
        material.push(':');
        material.push_str("task_input=");
        material.push_str(&input.name);
        match &input.source {
            RawTaskInputSource::Variable { source_ref } => {
                material.push_str(":source_ref=");
                material.push_str(source_ref);
            }
            RawTaskInputSource::Literal { value } => {
                material.push_str(":literal=");
                material.push_str(value);
            }
        }
    }
    for output in &task_io.outputs {
        material.push(':');
        material.push_str("task_output=");
        material.push_str(&output.name);
        material.push_str(":target_ref=");
        material.push_str(&output.target_ref);
    }
}

fn append_lane_membership_digest(material: &mut String, lane: &RawLaneMembershipSpec) {
    material.push(':');
    material.push_str("lane");
    if let Some(lane_set_id) = &lane.set_id {
        material.push_str(":lane_set_id=");
        material.push_str(lane_set_id);
    }
    if let Some(lane_set_name) = &lane.set_name {
        material.push_str(":lane_set_name=");
        material.push_str(lane_set_name);
    }
    if let Some(lane_id) = &lane.id {
        material.push_str(":lane_id=");
        material.push_str(lane_id);
    }
    if let Some(lane_name) = &lane.name {
        material.push_str(":lane_name=");
        material.push_str(lane_name);
    }
}

fn append_human_task_form_digest(material: &mut String, form: &RawHumanTaskFormSpec) {
    material.push(':');
    material.push_str("human_task_form=");
    material.push_str(&form.interaction_type);
    if let Some(question_ref) = &form.question_ref {
        material.push(':');
        material.push_str("question_ref=");
        material.push_str(question_ref);
    }
    if let Some(question_text) = &form.question_text {
        material.push(':');
        material.push_str("question_text=");
        material.push_str(question_text);
    }
    if let Some(choices_ref) = &form.choices_ref {
        material.push(':');
        material.push_str("choices_ref=");
        material.push_str(choices_ref);
    }
    for choice in &form.choices {
        material.push(':');
        material.push_str("choice=");
        material.push_str(&choice.value);
        if let Some(label) = &choice.label {
            material.push(':');
            material.push_str("choice_label=");
            material.push_str(label);
        }
    }
    for field in &form.free_text_fields {
        material.push(':');
        material.push_str("free_text=");
        material.push_str(&field.name);
        material.push(':');
        material.push_str(if field.optional {
            "optional"
        } else {
            "required"
        });
    }
    if let Some(result_output) = &form.result_output {
        material.push(':');
        material.push_str("result_output=");
        material.push_str(result_output);
    }
}

fn append_human_task_assignment_digest(
    material: &mut String,
    assignment: &RawHumanTaskAssignmentSpec,
) {
    material.push(':');
    material.push_str("human_task_assignment");
    for role in &assignment.human_performers {
        append_human_task_resource_role_digest(material, "human_performer", role);
    }
    for role in &assignment.potential_owners {
        append_human_task_resource_role_digest(material, "potential_owner", role);
    }
}

fn append_human_task_resource_role_digest(
    material: &mut String,
    kind: &str,
    role: &RawHumanTaskResourceRoleSpec,
) {
    material.push(':');
    material.push_str(kind);
    if let Some(name) = &role.name {
        material.push(':');
        material.push_str("name=");
        material.push_str(name);
    }
    if let Some(resource_ref) = &role.resource_ref {
        material.push(':');
        material.push_str("resource_ref=");
        material.push_str(resource_ref);
    }
    if let Some(assignment_expression) = &role.assignment_expression {
        material.push(':');
        material.push_str("assignment_expression=");
        material.push_str(assignment_expression);
    }
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
        BpmnNodeKind::IntermediateThrowEvent => "intermediate_throw_event",
        BpmnNodeKind::IntermediateCatchEvent => "intermediate_catch_event",
        BpmnNodeKind::BoundaryEvent => "boundary_event",
        BpmnNodeKind::SendTask => "send_task",
        BpmnNodeKind::ReceiveTask => "receive_task",
        BpmnNodeKind::ServiceTask => "service_task",
        BpmnNodeKind::ScriptTask => "script_task",
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
        BpmnEventKind::Escalation => "escalation",
        BpmnEventKind::Cancel => "cancel",
        BpmnEventKind::Compensation => "compensation",
        BpmnEventKind::Conditional => "conditional",
        BpmnEventKind::Terminate => "terminate",
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
        RawSubProcessKind::EventSubProcess => "event_subprocess",
    }
}

fn append_script_task_digest(material: &mut String, script_task: &RawScriptTaskSpec) {
    if let Some(script_format) = &script_task.script_format {
        material.push(':');
        material.push_str("script_format=");
        material.push_str(script_format);
    }
    if let Some(script_body) = &script_task.script_body {
        material.push(':');
        material.push_str("script_body=");
        material.push_str(script_body);
    }
}

fn gateway_kind_name(kind: &BpmnGatewayKind) -> &'static str {
    match kind {
        BpmnGatewayKind::Parallel => "parallel",
        BpmnGatewayKind::Exclusive => "exclusive",
        BpmnGatewayKind::Inclusive => "inclusive",
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
