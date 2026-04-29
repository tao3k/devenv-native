use super::process::normalize_node_index;
use super::repeat::normalize_repeat_spec;
use crate::error::{BpmnEngineError, Result};
use crate::ir_edge_api::BpmnEdgeSpec;
use crate::ir_node_api::{
    BpmnHumanTaskAssignmentSpec, BpmnHumanTaskChoiceSpec, BpmnHumanTaskFormSpec,
    BpmnHumanTaskFreeTextSpec, BpmnHumanTaskResourceRoleSpec, BpmnLaneMembershipSpec, BpmnNodeSpec,
    BpmnScriptTaskSpec, BpmnSubProcessKind, BpmnTaskInputBinding, BpmnTaskInputSource,
    BpmnTaskIoSpec, BpmnTaskOutputBinding,
};
use crate::parser::import::{
    RawHumanTaskAssignmentSpec, RawHumanTaskChoiceSpec, RawHumanTaskFormSpec,
    RawHumanTaskFreeTextSpec, RawHumanTaskNativeIoSpec, RawHumanTaskResourceRoleSpec,
    RawLaneMembershipSpec, RawNode, RawProcess, RawScriptTaskSpec, RawSubProcessKind,
    RawTaskInputBinding, RawTaskInputSource, RawTaskIoSpec, RawTaskOutputBinding,
};
use crate::parser::validate::resolve_structured_inclusive_join;
use std::collections::HashMap;

pub(super) fn normalize_nodes(
    raw: &RawProcess,
    index_by_id: &HashMap<String, u32>,
    edges: &[BpmnEdgeSpec],
) -> Result<Vec<BpmnNodeSpec>> {
    raw.nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let node_index =
                normalize_node_index(index, "normalize_process_node_spec_index_overflow")?;
            normalize_node(raw, node, node_index, index_by_id, edges)
        })
        .collect()
}

fn normalize_node(
    raw: &RawProcess,
    node: &RawNode,
    node_index: u32,
    index_by_id: &HashMap<String, u32>,
    edges: &[BpmnEdgeSpec],
) -> Result<BpmnNodeSpec> {
    let spec = BpmnNodeSpec::new(node_index, &node.bpmn_id, node.kind.clone());
    let spec = match node.gateway_kind.clone() {
        Some(gateway_kind) => spec.with_gateway_kind(gateway_kind),
        None => spec,
    };
    let spec = match node.decision.clone() {
        Some(decision) => spec.with_decision(decision),
        None => spec,
    };
    let spec = match &node.lane {
        Some(lane) => spec.with_lane(normalize_lane_membership(lane)),
        None => spec,
    };
    let spec = match &node.called_process_ref {
        Some(called_process_ref) => spec.with_called_process(called_process_ref),
        None => spec,
    };
    let spec = match node.subprocess_kind {
        Some(kind) => spec.with_subprocess_kind(normalize_subprocess_kind(kind)),
        None => spec,
    };
    let spec = match &node.script_task {
        Some(script_task) => spec.with_script_task(normalize_script_task(script_task)),
        None => spec,
    };
    let spec = match &node.human_task_form {
        Some(form) => spec.with_human_task_form(normalize_human_task_form(form)),
        None => spec,
    };
    let spec = match &node.human_task_assignment {
        Some(assignment) => {
            spec.with_human_task_assignment(normalize_human_task_assignment(assignment))
        }
        None => spec,
    };
    let spec = match normalize_node_task_io(node) {
        Some(task_io) => spec.with_task_io(task_io),
        None => spec,
    };
    let spec = normalize_repeat_spec(raw, node, spec)?;
    let spec = if node.is_for_compensation {
        spec.with_compensation_marker(true)
    } else {
        spec
    };
    let spec = attach_default_outgoing_edge(raw, node, spec, index_by_id, edges)?;
    let spec = attach_inclusive_join_node(raw, node, spec, index_by_id)?;
    attach_boundary_host(raw, node, spec, index_by_id)
}

fn attach_default_outgoing_edge(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
    index_by_id: &HashMap<String, u32>,
    edges: &[BpmnEdgeSpec],
) -> Result<BpmnNodeSpec> {
    let Some(default_flow_ref) = node.default_flow_ref.as_deref() else {
        return Ok(spec);
    };
    let Some(source_index) = index_by_id.get(node.bpmn_id.as_str()).copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "normalize_node_missing_source_index_for_default_flow",
        });
    };
    let edge_index = raw
        .flows
        .iter()
        .position(|flow| flow.flow_id == default_flow_ref)
        .ok_or_else(|| BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: raw.process_id.clone(),
            flow_id: default_flow_ref.to_string(),
            endpoint: "default",
            node_id: node.bpmn_id.clone(),
        })?;
    let normalized_edge_index =
        normalize_node_index(edge_index, "normalize_default_flow_edge_index_overflow")?;
    if edges[edge_index].from != source_index {
        return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: raw.process_id.clone(),
            flow_id: default_flow_ref.to_string(),
            endpoint: "default",
            node_id: node.bpmn_id.clone(),
        });
    }
    Ok(spec.with_default_outgoing_edge(normalized_edge_index))
}

fn attach_inclusive_join_node(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
    index_by_id: &HashMap<String, u32>,
) -> Result<BpmnNodeSpec> {
    let Some(join_node_id) = resolve_structured_inclusive_join(raw, node)? else {
        return Ok(spec);
    };
    let join_node_index = index_by_id.get(join_node_id.as_str()).copied().ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "normalize_inclusive_gateway_missing_join_index",
        },
    )?;
    Ok(spec.with_inclusive_join_node(join_node_index))
}

fn attach_boundary_host(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
    index_by_id: &HashMap<String, u32>,
) -> Result<BpmnNodeSpec> {
    match &node.attached_to_ref {
        Some(attached_to_ref) => {
            let attached_to = index_by_id.get(attached_to_ref).copied().ok_or_else(|| {
                BpmnEngineError::UnknownBoundaryAttachment {
                    process_id: raw.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    attached_to_node_id: attached_to_ref.clone(),
                }
            })?;
            Ok(spec.with_boundary_attachment(attached_to, node.cancel_activity))
        }
        None => Ok(spec),
    }
}

fn normalize_lane_membership(raw: &RawLaneMembershipSpec) -> BpmnLaneMembershipSpec {
    let mut lane = BpmnLaneMembershipSpec::new();
    if let Some(lane_set_id) = &raw.set_id {
        lane = lane.with_lane_set_id(lane_set_id);
    }
    if let Some(lane_set_name) = &raw.set_name {
        lane = lane.with_lane_set_name(lane_set_name);
    }
    if let Some(lane_id) = &raw.id {
        lane = lane.with_lane_id(lane_id);
    }
    if let Some(lane_name) = &raw.name {
        lane = lane.with_lane_name(lane_name);
    }
    lane
}

fn normalize_subprocess_kind(kind: RawSubProcessKind) -> BpmnSubProcessKind {
    match kind {
        RawSubProcessKind::CallActivity => BpmnSubProcessKind::CallActivity,
        RawSubProcessKind::EmbeddedSubProcess => BpmnSubProcessKind::Embedded,
        RawSubProcessKind::Transaction => BpmnSubProcessKind::Transaction,
    }
}

fn normalize_script_task(raw: &RawScriptTaskSpec) -> BpmnScriptTaskSpec {
    BpmnScriptTaskSpec::new(raw.script_format.as_deref(), raw.script_body.as_deref())
}

fn normalize_node_task_io(node: &RawNode) -> Option<BpmnTaskIoSpec> {
    node.task_io.as_ref().map(normalize_task_io).or_else(|| {
        node.native_human_task_io
            .as_ref()
            .and_then(normalize_human_native_task_io)
    })
}

fn normalize_task_io(raw: &RawTaskIoSpec) -> BpmnTaskIoSpec {
    let mut task_io = BpmnTaskIoSpec::new();
    for input in &raw.inputs {
        task_io = task_io.with_input(normalize_task_input(input));
    }
    for output in &raw.outputs {
        task_io = task_io.with_output(normalize_task_output(output));
    }
    task_io
}

fn normalize_task_input(raw: &RawTaskInputBinding) -> BpmnTaskInputBinding {
    BpmnTaskInputBinding::new(&raw.name, normalize_task_input_source(&raw.source))
}

fn normalize_task_input_source(raw: &RawTaskInputSource) -> BpmnTaskInputSource {
    match raw {
        RawTaskInputSource::Variable { source_ref } => BpmnTaskInputSource::variable(source_ref),
        RawTaskInputSource::Literal { value } => BpmnTaskInputSource::literal(value),
    }
}

fn normalize_task_output(raw: &RawTaskOutputBinding) -> BpmnTaskOutputBinding {
    BpmnTaskOutputBinding::new(&raw.name, &raw.target_ref)
}

fn normalize_human_native_task_io(raw: &RawHumanTaskNativeIoSpec) -> Option<BpmnTaskIoSpec> {
    let mut task_io = BpmnTaskIoSpec::new();
    if let Some(interaction_type) = &raw.interaction_type {
        task_io = task_io.with_input(BpmnTaskInputBinding::new(
            "interactionType",
            BpmnTaskInputSource::literal(interaction_type),
        ));
    }
    match (
        &raw.question_ref,
        &raw.question_text,
        &raw.documentation_text,
    ) {
        (Some(question_ref), _, _) => {
            task_io = task_io.with_input(BpmnTaskInputBinding::new(
                "question",
                BpmnTaskInputSource::variable(question_ref),
            ));
        }
        (None, Some(question_text), _) => {
            task_io = task_io.with_input(BpmnTaskInputBinding::new(
                "question",
                BpmnTaskInputSource::literal(question_text),
            ));
        }
        (None, None, Some(documentation_text)) => {
            task_io = task_io.with_input(BpmnTaskInputBinding::new(
                "question",
                BpmnTaskInputSource::literal(documentation_text),
            ));
        }
        (None, None, None) => {}
    }
    if let Some(choices_ref) = &raw.choices_ref {
        task_io = task_io.with_input(BpmnTaskInputBinding::new(
            "choices",
            BpmnTaskInputSource::variable(choices_ref),
        ));
    } else if !raw.choices.is_empty() {
        task_io = task_io.with_input(BpmnTaskInputBinding::new(
            "choices",
            BpmnTaskInputSource::literal(human_choices_literal(&raw.choices)),
        ));
    }
    if !raw.free_text_fields.is_empty() {
        task_io = task_io.with_input(BpmnTaskInputBinding::new(
            "freeText",
            BpmnTaskInputSource::literal(human_free_text_literal(&raw.free_text_fields)),
        ));
    }
    if let Some(result_output) = &raw.result_output {
        task_io = task_io.with_output(BpmnTaskOutputBinding::new("answer", result_output));
    }
    if task_io.inputs.is_empty() && task_io.outputs.is_empty() {
        None
    } else {
        Some(task_io)
    }
}

fn human_choices_literal(choices: &[RawHumanTaskChoiceSpec]) -> String {
    serde_json::Value::Array(
        choices
            .iter()
            .map(|choice| {
                let mut item = serde_json::Map::new();
                item.insert(
                    "value".to_string(),
                    serde_json::Value::String(choice.value.clone()),
                );
                if let Some(label) = &choice.label {
                    item.insert(
                        "label".to_string(),
                        serde_json::Value::String(label.clone()),
                    );
                }
                serde_json::Value::Object(item)
            })
            .collect(),
    )
    .to_string()
}

fn human_free_text_literal(fields: &[RawHumanTaskFreeTextSpec]) -> String {
    serde_json::Value::Array(
        fields
            .iter()
            .map(|field| {
                let mut item = serde_json::Map::new();
                item.insert(
                    "name".to_string(),
                    serde_json::Value::String(field.name.clone()),
                );
                item.insert(
                    "optional".to_string(),
                    serde_json::Value::Bool(field.optional),
                );
                serde_json::Value::Object(item)
            })
            .collect(),
    )
    .to_string()
}

fn normalize_human_task_form(raw: &RawHumanTaskFormSpec) -> BpmnHumanTaskFormSpec {
    let mut form = BpmnHumanTaskFormSpec::new(&raw.interaction_type);
    if let Some(question_ref) = &raw.question_ref {
        form = form.with_question_ref(question_ref);
    }
    if let Some(question_text) = &raw.question_text {
        form = form.with_question_text(question_text);
    }
    if let Some(choices_ref) = &raw.choices_ref {
        form = form.with_choices_ref(choices_ref);
    }
    for choice in &raw.choices {
        form = form.with_choice(normalize_human_task_choice(choice));
    }
    for field in &raw.free_text_fields {
        form = form.with_free_text_field(normalize_human_task_free_text(field));
    }
    if let Some(result_output) = &raw.result_output {
        form = form.with_result_output(result_output);
    }
    form
}

fn normalize_human_task_choice(raw: &RawHumanTaskChoiceSpec) -> BpmnHumanTaskChoiceSpec {
    match &raw.label {
        Some(label) => BpmnHumanTaskChoiceSpec::new(&raw.value).with_label(label),
        None => BpmnHumanTaskChoiceSpec::new(&raw.value),
    }
}

fn normalize_human_task_free_text(raw: &RawHumanTaskFreeTextSpec) -> BpmnHumanTaskFreeTextSpec {
    BpmnHumanTaskFreeTextSpec::new(&raw.name, raw.optional)
}

fn normalize_human_task_assignment(
    raw: &RawHumanTaskAssignmentSpec,
) -> BpmnHumanTaskAssignmentSpec {
    let mut assignment = BpmnHumanTaskAssignmentSpec::new();
    for role in &raw.human_performers {
        assignment = assignment.with_human_performer(normalize_human_task_resource_role(role));
    }
    for role in &raw.potential_owners {
        assignment = assignment.with_potential_owner(normalize_human_task_resource_role(role));
    }
    assignment
}

fn normalize_human_task_resource_role(
    raw: &RawHumanTaskResourceRoleSpec,
) -> BpmnHumanTaskResourceRoleSpec {
    let mut role = BpmnHumanTaskResourceRoleSpec::new();
    if let Some(name) = &raw.name {
        role = role.with_name(name);
    }
    if let Some(resource_ref) = &raw.resource_ref {
        role = role.with_resource_ref(resource_ref);
    }
    if let Some(assignment_expression) = &raw.assignment_expression {
        role = role.with_assignment_expression(assignment_expression);
    }
    role
}
