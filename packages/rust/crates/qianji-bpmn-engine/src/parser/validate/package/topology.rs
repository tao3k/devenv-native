use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::{
    NestedShellKind, RawNode, RawProcess, RawProcessScope, RawSubProcessKind,
};
use crate::repeat_condition::is_supported_gateway_condition;
use std::collections::{HashMap, HashSet};

pub(super) fn validate_process_topology(
    process: &RawProcess,
    all_process_ids: &HashSet<&str>,
    node_ids: &HashSet<&str>,
    process_by_id: &HashMap<&str, &RawProcess>,
    call_activity_owners: &HashMap<
        &str,
        Vec<crate::parser::validate::error_paths::CallActivityOwner<'_>>,
    >,
) -> Result<()> {
    let mut boundary_attachments = HashMap::new();
    validate_event_subprocesses(process, process_by_id)?;
    validate_transaction_cancel_path(process, process_by_id)?;
    crate::parser::validate::error_paths::validate_supported_error_end_paths(
        process,
        process_by_id,
        call_activity_owners,
    )?;
    crate::parser::validate::escalation_paths::validate_supported_escalation_throw_paths(
        process,
        process_by_id,
        call_activity_owners,
    )?;
    for node in &process.nodes {
        validate_node_event_shape(process, node)?;
        if node.kind == BpmnNodeKind::SubProcess {
            validate_called_process_reference(process, node, all_process_ids)?;
        }
        if node.kind == BpmnNodeKind::BoundaryEvent {
            crate::parser::validate::boundary::validate_boundary_event(
                process,
                node,
                node_ids,
                &mut boundary_attachments,
            )?;
        }
    }
    Ok(())
}

fn validate_event_subprocesses(
    process: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<()> {
    let event_subprocesses = process
        .nodes
        .iter()
        .filter(|node| node.subprocess_kind == Some(RawSubProcessKind::EventSubProcess))
        .collect::<Vec<_>>();
    if event_subprocesses.len() > 1 {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (event_subprocesses[1].bpmn_id.clone()).into(),
            detail: "multiple_event_subprocesses",
        });
    }

    for owner in event_subprocesses {
        validate_event_subprocess_owner(process, owner, process_by_id)?;
    }
    Ok(())
}

fn validate_event_subprocess_owner(
    process: &RawProcess,
    owner: &RawNode,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<()> {
    if process
        .flows
        .iter()
        .any(|flow| flow.source_ref == owner.bpmn_id || flow.target_ref == owner.bpmn_id)
    {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "event_subprocess_sequence_flow",
        });
    }

    let called_process_id = owner.called_process_ref.as_ref().ok_or_else(|| {
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "missing_event_subprocess_body",
        }
    })?;
    let child = process_by_id
        .get(called_process_id.as_str())
        .ok_or_else(|| BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "missing_event_subprocess_body",
        })?;
    let start = child
        .nodes
        .iter()
        .find(|node| node.kind == BpmnNodeKind::StartEvent)
        .ok_or_else(|| BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "event_subprocess_start_event_count",
        })?;
    if !start.cancel_activity {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "event_subprocess_non_interrupting",
        });
    }
    let Some(event) = start.event.as_ref() else {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "event_subprocess_start_event_definition",
        });
    };
    if event.kind == BpmnEventKind::Compensation {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "event_subprocess_compensation_deferred",
        });
    }
    if !matches!(
        event.kind,
        BpmnEventKind::Message
            | BpmnEventKind::Signal
            | BpmnEventKind::Timer
            | BpmnEventKind::Conditional
    ) {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (owner.bpmn_id.clone()).into(),
            detail: "event_subprocess_start_event_definition",
        });
    }
    Ok(())
}

fn validate_transaction_cancel_path(
    process: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<()> {
    let cancel_end_nodes = process
        .nodes
        .iter()
        .filter(|node| {
            node.kind == BpmnNodeKind::EndEvent
                && node.event.as_ref().map(|event| event.kind.clone())
                    == Some(BpmnEventKind::Cancel)
        })
        .collect::<Vec<_>>();
    if cancel_end_nodes.is_empty() {
        return Ok(());
    }

    let RawProcessScope::NestedShell {
        owner_process_id,
        owner_node_id,
        kind: NestedShellKind::Transaction,
    } = &process.scope
    else {
        return Err(BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (cancel_end_nodes[0].bpmn_id.clone()).into(),
            detail: "cancel_end_requires_transaction_shell",
        });
    };

    if cancel_end_nodes.len() > 1 {
        return Err(BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id: (owner_process_id.clone()).into(),
            node_id: (owner_node_id.clone()).into(),
            detail: "multiple_transaction_cancel_end_events",
        });
    }

    let Some(parent_process) = process_by_id.get(owner_process_id.as_str()).copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "validate_transaction_cancel_missing_parent_process",
        });
    };

    let has_matching_boundary = parent_process.nodes.iter().any(|node| {
        node.kind == BpmnNodeKind::BoundaryEvent
            && node.attached_to_ref.as_deref() == Some(owner_node_id.as_str())
            && node.cancel_activity
            && node.event.as_ref().map(|event| event.kind.clone()) == Some(BpmnEventKind::Cancel)
    });
    if has_matching_boundary {
        return Ok(());
    }

    Err(BpmnEngineError::UnsupportedTransactionConfiguration {
        process_id: (owner_process_id.clone()).into(),
        node_id: (owner_node_id.clone()).into(),
        detail: "transaction_cancel_missing_boundary",
    })
}

fn validate_node_event_shape(process: &RawProcess, node: &RawNode) -> Result<()> {
    if matches!(
        node.kind,
        BpmnNodeKind::IntermediateThrowEvent
            | BpmnNodeKind::IntermediateCatchEvent
            | BpmnNodeKind::BoundaryEvent
    ) && node.event.is_none()
    {
        return Err(BpmnEngineError::MissingRequiredNodeElement {
            process_id: (process.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            element: "event_definition",
        });
    }

    if let Some(event) = &node.event
        && event.kind == BpmnEventKind::Timer
        && event
            .timer
            .as_ref()
            .is_none_or(|timer| timer.expression.trim().is_empty())
    {
        return Err(BpmnEngineError::MissingRequiredNodeElement {
            process_id: (process.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            element: "timer_expression",
        });
    }

    if let Some(event) = &node.event
        && event.kind == BpmnEventKind::Conditional
    {
        let Some(condition_expression) = event.condition_expression.as_deref() else {
            return Err(BpmnEngineError::MissingRequiredNodeElement {
                process_id: (process.process_id.clone()).into(),
                node_id: (node.bpmn_id.clone()).into(),
                element: "conditional_expression",
            });
        };
        if !is_supported_gateway_condition(condition_expression) {
            return Err(BpmnEngineError::UnsupportedEventConfiguration {
                process_id: (process.process_id.clone()).into(),
                node_id: (node.bpmn_id.clone()).into(),
                detail: "unsupported_conditional_event_expression",
            });
        }
    }

    validate_message_task_shape(process, node)?;

    Ok(())
}

fn validate_message_task_shape(process: &RawProcess, node: &RawNode) -> Result<()> {
    let expected_detail = match node.kind {
        BpmnNodeKind::SendTask => Some("unsupported_send_task_event_kind"),
        BpmnNodeKind::ReceiveTask => Some("unsupported_receive_task_event_kind"),
        _ => None,
    };
    let Some(expected_detail) = expected_detail else {
        return Ok(());
    };

    if node.task_message_ref.is_some() && node.event.is_some() {
        return Err(BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: (process.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            detail: "multiple_task_message_bindings",
        });
    }

    let attribute_binding = node
        .task_message_ref
        .as_deref()
        .map(str::trim)
        .filter(|binding| !binding.is_empty());
    if let Some(event) = &node.event {
        if event.kind != BpmnEventKind::Message {
            return Err(BpmnEngineError::UnsupportedTaskConfiguration {
                process_id: (process.process_id.clone()).into(),
                node_id: (node.bpmn_id.clone()).into(),
                detail: expected_detail,
            });
        }
        let event_binding = event
            .reference_id
            .as_deref()
            .map(str::trim)
            .filter(|binding| !binding.is_empty());
        if event_binding.is_none() {
            return Err(BpmnEngineError::MissingRequiredNodeElement {
                process_id: (process.process_id.clone()).into(),
                node_id: (node.bpmn_id.clone()).into(),
                element: "message_binding",
            });
        }
        return Ok(());
    }

    if attribute_binding.is_some() {
        return Ok(());
    }

    Err(BpmnEngineError::MissingRequiredNodeElement {
        process_id: (process.process_id.clone()).into(),
        node_id: (node.bpmn_id.clone()).into(),
        element: "message_binding",
    })
}

fn validate_called_process_reference(
    process: &RawProcess,
    node: &RawNode,
    all_process_ids: &HashSet<&str>,
) -> Result<()> {
    let called_process_id = node.called_process_ref.as_deref().ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: (process.process_id.clone()).into(),
            node_id: (node.bpmn_id.clone()).into(),
            element: "called_process",
        }
    })?;
    if all_process_ids.contains(called_process_id) {
        return Ok(());
    }
    Err(BpmnEngineError::UnknownCalledProcess {
        process_id: (process.process_id.clone()).into(),
        node_id: (node.bpmn_id.clone()).into(),
        called_process_id: (called_process_id.to_string()).into(),
    })
}
