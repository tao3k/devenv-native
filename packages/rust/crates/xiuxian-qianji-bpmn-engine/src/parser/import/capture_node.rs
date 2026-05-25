use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnTimerKind;
use crate::parser::import::model::RawTimerSpec;
use crate::parser::import::{
    RawHumanTaskResourceRoleKind, RawHumanTaskResourceRoleSpec, RawNode, RawProcess, RawRepeatSpec,
};

pub(in crate::parser::import) fn apply_timer_expression(
    process: &mut RawProcess,
    kind: BpmnTimerKind,
    expression: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_timer_expression_without_node",
        })?;
    let event = node
        .event
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_timer_expression_without_event_definition",
        })?;
    event.timer = Some(RawTimerSpec {
        kind,
        expression: expression.to_string(),
    });
    Ok(())
}

pub(in crate::parser::import) fn apply_conditional_expression(
    process: &mut RawProcess,
    expression: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_conditional_expression_without_node",
        })?;
    let event = node
        .event
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_conditional_expression_without_event_definition",
        })?;
    event.condition_expression =
        (!expression.trim().is_empty()).then(|| expression.trim().to_string());
    Ok(())
}

pub(in crate::parser::import) fn last_process_node_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawNode> {
    process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedElement {
            source_id: (source.source_id.clone()).into(),
            process_id: (process.process_id.clone()).into(),
            element: "event_definition_without_node".to_string(),
        })
}

pub(in crate::parser::import) fn apply_standard_loop_condition(
    process: &mut RawProcess,
    loop_condition: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_standard_loop_condition_missing_node",
        })?;
    let Some(RawRepeatSpec::StandardLoop(loop_spec)) = node.repeat.as_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_standard_loop_condition_missing_repeat_spec",
        });
    };
    loop_spec.loop_condition =
        (!loop_condition.trim().is_empty()).then(|| loop_condition.to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_sequence_flow_condition_expression(
    process: &mut RawProcess,
    condition_expression: &str,
) -> Result<()> {
    let flow = process
        .flows
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_sequence_flow_condition_expression_missing_flow",
        })?;
    flow.condition_expression =
        (!condition_expression.trim().is_empty()).then(|| condition_expression.trim().to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_script_task_body(
    process: &mut RawProcess,
    script_body: &str,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_script_task_body_missing_node",
        })?;
    let script_task = node
        .script_task
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_script_task_body_missing_script_task_spec",
        })?;
    script_task.script_body =
        (!script_body.trim().is_empty()).then(|| script_body.trim().to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_human_task_resource_ref(
    process: &mut RawProcess,
    kind: RawHumanTaskResourceRoleKind,
    resource_ref: &str,
) -> Result<()> {
    let role = last_human_task_resource_role_mut(process, kind)?;
    role.resource_ref = (!resource_ref.trim().is_empty()).then(|| resource_ref.trim().to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_human_task_assignment_expression(
    process: &mut RawProcess,
    kind: RawHumanTaskResourceRoleKind,
    assignment_expression: &str,
) -> Result<()> {
    let role = last_human_task_resource_role_mut(process, kind)?;
    role.assignment_expression = (!assignment_expression.trim().is_empty())
        .then(|| assignment_expression.trim().to_string());
    Ok(())
}

pub(in crate::parser::import) fn push_human_task_resource_role(
    process: &mut RawProcess,
    kind: RawHumanTaskResourceRoleKind,
    name: Option<String>,
) -> Result<()> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "push_human_task_resource_role_missing_node",
        })?;
    let assignment = node
        .human_task_assignment
        .get_or_insert_with(crate::parser::import::RawHumanTaskAssignmentSpec::new);
    let role = RawHumanTaskResourceRoleSpec {
        name,
        resource_ref: None,
        assignment_expression: None,
    };
    match kind {
        RawHumanTaskResourceRoleKind::HumanPerformer => {
            assignment.human_performers.push(role);
        }
        RawHumanTaskResourceRoleKind::PotentialOwner => {
            assignment.potential_owners.push(role);
        }
    }
    assignment.last_role_kind = Some(kind);
    Ok(())
}

fn last_human_task_resource_role_mut(
    process: &mut RawProcess,
    kind: RawHumanTaskResourceRoleKind,
) -> Result<&mut RawHumanTaskResourceRoleSpec> {
    let node = process
        .nodes
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "human_task_resource_role_missing_node",
        })?;
    let assignment =
        node.human_task_assignment
            .as_mut()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "human_task_resource_role_missing_assignment",
            })?;
    let role = match kind {
        RawHumanTaskResourceRoleKind::HumanPerformer => assignment.human_performers.last_mut(),
        RawHumanTaskResourceRoleKind::PotentialOwner => assignment.potential_owners.last_mut(),
    };
    role.ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "human_task_resource_role_missing_role",
    })
}
