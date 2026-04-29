use crate::bpmn_cli::deps::{
    BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec, BpmnHumanTaskLifecycleEventKind,
    BpmnLaneMembershipSpec,
};

pub(in crate::bpmn_cli::render) fn bpmn_human_task_lifecycle_event_kind_label(
    kind: &BpmnHumanTaskLifecycleEventKind,
) -> &'static str {
    match kind {
        BpmnHumanTaskLifecycleEventKind::Created => "created",
        BpmnHumanTaskLifecycleEventKind::Claimed => "claimed",
        BpmnHumanTaskLifecycleEventKind::Released => "released",
        BpmnHumanTaskLifecycleEventKind::Completed => "completed",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_human_task_form_label(
    form: &BpmnHumanTaskFormSpec,
) -> String {
    let mut label = form.interaction_type.to_string();
    if let Some(result_output) = form.result_output.as_deref() {
        label.push_str(" result=");
        label.push_str(result_output);
    }
    if !form.free_text_fields.is_empty() {
        label.push_str(" fields=");
        label.push_str(
            &form
                .free_text_fields
                .iter()
                .map(|field| {
                    if field.optional {
                        format!("{}?", field.name)
                    } else {
                        field.name.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    label
}

pub(in crate::bpmn_cli::render) fn bpmn_human_task_assignment_label(
    assignment: &BpmnHumanTaskAssignmentSpec,
) -> String {
    let mut roles = Vec::new();
    roles.extend(
        assignment
            .human_performers
            .iter()
            .map(|role| bpmn_human_task_assignment_role_label("human_performer", role)),
    );
    roles.extend(
        assignment
            .potential_owners
            .iter()
            .map(|role| bpmn_human_task_assignment_role_label("potential_owner", role)),
    );
    roles.join(";")
}

fn bpmn_human_task_assignment_role_label(
    kind: &str,
    role: &qianji_bpmn_engine::BpmnHumanTaskResourceRoleSpec,
) -> String {
    let mut label = kind.to_string();
    if let Some(name) = role.name.as_deref() {
        label.push(':');
        label.push_str(name);
    }
    if let Some(resource_ref) = role.resource_ref.as_deref() {
        label.push_str(":ref=");
        label.push_str(resource_ref);
    }
    if let Some(expression) = role.assignment_expression.as_deref() {
        label.push_str(":expr=");
        label.push_str(expression);
    }
    label
}

pub(in crate::bpmn_cli::render) fn bpmn_lane_membership_label(
    lane: &BpmnLaneMembershipSpec,
) -> String {
    let mut label = lane
        .name
        .as_deref()
        .or(lane.id.as_deref())
        .unwrap_or("unlabelled")
        .to_string();
    if let Some(lane_id) = lane.id.as_deref()
        && lane.name.as_deref() != Some(lane_id)
    {
        label.push_str(" id=");
        label.push_str(lane_id);
    }
    if let Some(lane_set) = lane.set_name.as_deref().or(lane.set_id.as_deref()) {
        label.push_str(" set=");
        label.push_str(lane_set);
    }
    label
}
