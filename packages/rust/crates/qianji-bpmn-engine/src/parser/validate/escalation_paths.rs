use super::error_paths::CallActivityOwner;
use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::{NestedShellKind, RawProcess, RawProcessScope};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
struct EscalationOwnerRequirement<'a> {
    process_id: &'a str,
    node_id: &'a str,
    owner_kind: SupportedEscalationOwner,
}

#[derive(Clone, Copy, Debug)]
enum SupportedEscalationOwner {
    Transaction,
    EmbeddedSubProcess,
    CallActivity,
}

impl SupportedEscalationOwner {
    fn missing_boundary_error(self, process_id: &str, node_id: &str) -> BpmnEngineError {
        match self {
            Self::Transaction => BpmnEngineError::UnsupportedTransactionConfiguration {
                process_id: process_id.to_string(),
                node_id: node_id.to_string(),
                detail: "transaction_escalation_missing_boundary",
            },
            Self::EmbeddedSubProcess => BpmnEngineError::UnsupportedSubProcessConfiguration {
                process_id: process_id.to_string(),
                node_id: node_id.to_string(),
                detail: "embedded_subprocess_escalation_missing_boundary",
            },
            Self::CallActivity => BpmnEngineError::UnsupportedSubProcessConfiguration {
                process_id: process_id.to_string(),
                node_id: node_id.to_string(),
                detail: "call_activity_escalation_missing_boundary",
            },
        }
    }
}

pub(super) fn validate_supported_escalation_end_paths(
    process: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
    call_activity_owners: &HashMap<&str, Vec<CallActivityOwner<'_>>>,
) -> Result<()> {
    let escalation_end_nodes = process
        .nodes
        .iter()
        .filter(|node| {
            node.kind == BpmnNodeKind::EndEvent
                && node.event.as_ref().map(|event| event.kind.clone())
                    == Some(BpmnEventKind::Escalation)
        })
        .collect::<Vec<_>>();
    if escalation_end_nodes.is_empty() {
        return Ok(());
    }

    let owner_requirements = resolve_supported_escalation_owners(process, call_activity_owners);
    if owner_requirements.is_empty() {
        return Err(BpmnEngineError::UnsupportedEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: escalation_end_nodes[0].bpmn_id.clone(),
            detail: "escalation_end_requires_supported_parent_boundary",
        });
    }

    for escalation_end_node in escalation_end_nodes {
        let thrown_reference_id = escalation_end_node
            .event
            .as_ref()
            .and_then(|event| event.reference_id.as_deref());
        for owner in &owner_requirements {
            let Some(parent_process) = process_by_id.get(owner.process_id).copied() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "validate_escalation_end_missing_parent_process",
                });
            };
            let has_matching_boundary = parent_process.nodes.iter().any(|node| {
                node.kind == BpmnNodeKind::BoundaryEvent
                    && node.attached_to_ref.as_deref() == Some(owner.node_id)
                    && node.cancel_activity
                    && node.event.as_ref().is_some_and(|event| {
                        event.kind == BpmnEventKind::Escalation
                            && escalation_boundary_matches(
                                thrown_reference_id,
                                event.reference_id.as_deref(),
                            )
                    })
            });
            if !has_matching_boundary {
                return Err(owner
                    .owner_kind
                    .missing_boundary_error(owner.process_id, owner.node_id));
            }
        }
    }

    Ok(())
}

fn resolve_supported_escalation_owners<'a>(
    process: &'a RawProcess,
    call_activity_owners: &'a HashMap<&str, Vec<CallActivityOwner<'a>>>,
) -> Vec<EscalationOwnerRequirement<'a>> {
    match &process.scope {
        RawProcessScope::NestedShell {
            owner_process_id,
            owner_node_id,
            kind: NestedShellKind::Transaction,
        } => vec![EscalationOwnerRequirement {
            process_id: owner_process_id.as_str(),
            node_id: owner_node_id.as_str(),
            owner_kind: SupportedEscalationOwner::Transaction,
        }],
        RawProcessScope::NestedShell {
            owner_process_id,
            owner_node_id,
            kind: NestedShellKind::EmbeddedSubProcess,
        } => vec![EscalationOwnerRequirement {
            process_id: owner_process_id.as_str(),
            node_id: owner_node_id.as_str(),
            owner_kind: SupportedEscalationOwner::EmbeddedSubProcess,
        }],
        RawProcessScope::TopLevel => call_activity_owners
            .get(process.process_id.as_str())
            .map(|owners| {
                owners
                    .iter()
                    .map(|owner| EscalationOwnerRequirement {
                        process_id: owner.process_id,
                        node_id: owner.node_id,
                        owner_kind: SupportedEscalationOwner::CallActivity,
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|owners| !owners.is_empty())
            .unwrap_or_default(),
    }
}

fn escalation_boundary_matches(
    thrown_reference_id: Option<&str>,
    boundary_reference_id: Option<&str>,
) -> bool {
    match boundary_reference_id {
        None => true,
        Some(boundary_reference_id) => thrown_reference_id == Some(boundary_reference_id),
    }
}
