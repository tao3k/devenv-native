use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::{
    NestedShellKind, RawPackageDocument, RawProcess, RawProcessScope, RawSubProcessKind,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub(in crate::parser) struct CallActivityOwner<'a> {
    pub(super) process_id: &'a str,
    pub(super) node_id: &'a str,
}

#[derive(Clone, Copy, Debug)]
struct ErrorOwnerRequirement<'a> {
    process_id: &'a str,
    node_id: &'a str,
    owner_kind: SupportedErrorOwner,
}

#[derive(Clone, Copy, Debug)]
enum SupportedErrorOwner {
    Transaction,
    EmbeddedSubProcess,
    CallActivity,
}

impl SupportedErrorOwner {
    fn missing_boundary_error(self, process_id: &str, node_id: &str) -> BpmnEngineError {
        match self {
            Self::Transaction => BpmnEngineError::UnsupportedTransactionConfiguration {
                process_id: process_id.to_string(),
                node_id: node_id.to_string(),
                detail: "transaction_error_missing_boundary",
            },
            Self::EmbeddedSubProcess => BpmnEngineError::UnsupportedSubProcessConfiguration {
                process_id: process_id.to_string(),
                node_id: node_id.to_string(),
                detail: "embedded_subprocess_error_missing_boundary",
            },
            Self::CallActivity => BpmnEngineError::UnsupportedSubProcessConfiguration {
                process_id: process_id.to_string(),
                node_id: node_id.to_string(),
                detail: "call_activity_error_missing_boundary",
            },
        }
    }
}

pub(in crate::parser) fn collect_call_activity_owners(
    raw: &RawPackageDocument,
) -> HashMap<&str, Vec<CallActivityOwner<'_>>> {
    let mut owners = HashMap::new();
    for process in &raw.processes {
        for node in &process.nodes {
            if node.kind == BpmnNodeKind::SubProcess
                && node.subprocess_kind == Some(RawSubProcessKind::CallActivity)
                && let Some(called_process_id) = node.called_process_ref.as_deref()
            {
                owners
                    .entry(called_process_id)
                    .or_insert_with(Vec::new)
                    .push(CallActivityOwner {
                        process_id: process.process_id.as_str(),
                        node_id: node.bpmn_id.as_str(),
                    });
            }
        }
    }
    owners
}

pub(in crate::parser) fn validate_supported_error_end_paths(
    process: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
    call_activity_owners: &HashMap<&str, Vec<CallActivityOwner<'_>>>,
) -> Result<()> {
    let error_end_nodes = process
        .nodes
        .iter()
        .filter(|node| {
            node.kind == BpmnNodeKind::EndEvent
                && node.event.as_ref().map(|event| event.kind.clone()) == Some(BpmnEventKind::Error)
        })
        .collect::<Vec<_>>();
    if error_end_nodes.is_empty() {
        return Ok(());
    }

    let owner_requirements = resolve_supported_error_owners(process, call_activity_owners);

    for error_end_node in error_end_nodes {
        let thrown_reference_id = error_end_node
            .event
            .as_ref()
            .and_then(|event| event.reference_id.as_deref());
        for owner in &owner_requirements {
            let Some(parent_process) = process_by_id.get(owner.process_id).copied() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "validate_error_end_missing_parent_process",
                });
            };
            let has_matching_boundary = parent_process.nodes.iter().any(|node| {
                node.kind == BpmnNodeKind::BoundaryEvent
                    && node.attached_to_ref.as_deref() == Some(owner.node_id)
                    && node.cancel_activity
                    && node.event.as_ref().is_some_and(|event| {
                        event.kind == BpmnEventKind::Error
                            && error_boundary_matches(
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

fn resolve_supported_error_owners<'a>(
    process: &'a RawProcess,
    call_activity_owners: &'a HashMap<&str, Vec<CallActivityOwner<'a>>>,
) -> Vec<ErrorOwnerRequirement<'a>> {
    match &process.scope {
        RawProcessScope::NestedShell {
            owner_process_id,
            owner_node_id,
            kind: NestedShellKind::Transaction,
        } => vec![ErrorOwnerRequirement {
            process_id: owner_process_id.as_str(),
            node_id: owner_node_id.as_str(),
            owner_kind: SupportedErrorOwner::Transaction,
        }],
        RawProcessScope::NestedShell {
            owner_process_id,
            owner_node_id,
            kind: NestedShellKind::EmbeddedSubProcess,
        } => vec![ErrorOwnerRequirement {
            process_id: owner_process_id.as_str(),
            node_id: owner_node_id.as_str(),
            owner_kind: SupportedErrorOwner::EmbeddedSubProcess,
        }],
        RawProcessScope::NestedShell {
            kind: NestedShellKind::EventSubProcess,
            ..
        } => Vec::new(),
        RawProcessScope::TopLevel => call_activity_owners
            .get(process.process_id.as_str())
            .map(|owners| {
                owners
                    .iter()
                    .map(|owner| ErrorOwnerRequirement {
                        process_id: owner.process_id,
                        node_id: owner.node_id,
                        owner_kind: SupportedErrorOwner::CallActivity,
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|owners| !owners.is_empty())
            .unwrap_or_default(),
    }
}

fn error_boundary_matches(
    thrown_reference_id: Option<&str>,
    boundary_reference_id: Option<&str>,
) -> bool {
    match boundary_reference_id {
        None => true,
        Some(boundary_reference_id) => thrown_reference_id == Some(boundary_reference_id),
    }
}
