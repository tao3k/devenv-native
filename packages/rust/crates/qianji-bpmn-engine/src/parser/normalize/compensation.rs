use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::BpmnNodeKind;
use crate::ir_process_compensation::BpmnCompensationHandlerSpec;
use crate::parser::import::{RawAssociation, RawProcess};
use std::collections::HashMap;

pub(super) fn normalize_compensation_handlers(
    raw: &RawProcess,
    index_by_id: &HashMap<String, u32>,
) -> Result<Vec<BpmnCompensationHandlerSpec>> {
    raw.associations
        .iter()
        .filter_map(|association| {
            let source_index = *index_by_id.get(&association.source_ref)?;
            let boundary_node = raw.nodes.get(source_index as usize)?;
            (boundary_node.kind == BpmnNodeKind::BoundaryEvent
                && boundary_node
                    .event
                    .as_ref()
                    .is_some_and(|event| event.kind == BpmnEventKind::Compensation))
            .then_some(association)
        })
        .map(|association| normalize_compensation_handler(raw, association, index_by_id))
        .collect()
}

fn normalize_compensation_handler(
    raw: &RawProcess,
    association: &RawAssociation,
    index_by_id: &HashMap<String, u32>,
) -> Result<BpmnCompensationHandlerSpec> {
    let boundary_node_index = index_by_id
        .get(&association.source_ref)
        .copied()
        .ok_or_else(|| BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: raw.process_id.clone(),
            flow_id: association.association_id.clone(),
            endpoint: "source",
            node_id: association.source_ref.clone(),
        })?;
    let handler_node_index = index_by_id
        .get(&association.target_ref)
        .copied()
        .ok_or_else(|| BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: raw.process_id.clone(),
            flow_id: association.association_id.clone(),
            endpoint: "target",
            node_id: association.target_ref.clone(),
        })?;
    let attached_to_ref = raw.nodes[boundary_node_index as usize]
        .attached_to_ref
        .as_ref()
        .ok_or_else(|| BpmnEngineError::UnknownBoundaryAttachment {
            process_id: raw.process_id.clone(),
            node_id: raw.nodes[boundary_node_index as usize].bpmn_id.clone(),
            attached_to_node_id: String::new(),
        })?;
    let activity_node_index = index_by_id.get(attached_to_ref).copied().ok_or_else(|| {
        BpmnEngineError::UnknownBoundaryAttachment {
            process_id: raw.process_id.clone(),
            node_id: raw.nodes[boundary_node_index as usize].bpmn_id.clone(),
            attached_to_node_id: attached_to_ref.clone(),
        }
    })?;
    Ok(BpmnCompensationHandlerSpec {
        boundary: boundary_node_index,
        activity: activity_node_index,
        handler: handler_node_index,
    })
}
