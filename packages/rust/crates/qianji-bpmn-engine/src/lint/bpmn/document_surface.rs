use crate::bpmn_model_api::{
    BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot, BpmnConversationNodeSnapshot,
    BpmnDocumentSnapshot,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint_api::LintIssue;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::{Value, json};

const SNAPSHOT_EVIDENCE_LIMIT: usize = 8;

pub(super) fn deferred_document_surface_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event)) => {
                let name = event.name();
                let tag = local_name(name.as_ref())?;
                if let Some(issue) = issue_for_tag(source, tag) {
                    return Some(issue);
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn local_name(raw: &[u8]) -> Option<&str> {
    let name = std::str::from_utf8(raw).ok()?;
    Some(name.rsplit_once(':').map_or(name, |(_, local)| local))
}

fn issue_for_tag(source: &BpmnSourceFile, tag: &str) -> Option<LintIssue> {
    match tag {
        "collaboration"
        | "participant"
        | "messageFlow"
        | "conversation"
        | "choreography"
        | "globalChoreographyTask"
        | "choreographyTask"
        | "subChoreography"
        | "callChoreography" => Some(collaboration_issue(source, tag)),
        "dataObject" | "dataObjectReference" | "dataStore" | "dataStoreReference" => {
            Some(data_artifact_issue(source, tag))
        }
        _ => None,
    }
}

fn collaboration_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.unsupported_collaboration_surface",
        "Collaboration, choreography, and pool semantics are deferred",
        format!("Source '{source_id}' contains collaboration-level BPMN element '<{tag}>'."),
        "The bounded engine executes one process graph at a time and does not yet own pool, participant, message-flow, conversation, or choreography semantics.",
        vec![
            "Move the executable control flow into one supported `<bpmn:process>` before running it with this engine.".to_string(),
            "Preserve pool or participant ownership as documentation metadata outside the executable BPMN subset.".to_string(),
            "If cross-pool messaging or choreography is required, remodel the current slice as explicit host-dispatched tasks or wait events until collaboration execution is implemented.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by removing executable dependency on `<{tag}>`. Keep one supported `<bpmn:process>` with explicit sequence flows, and preserve pool/participant intent as non-executable documentation or host-level routing metadata."
        ),
        document_surface_evidence(source, tag, "collaboration"),
    )
}

fn data_artifact_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.unsupported_data_surface",
        "BPMN data-object and data-store semantics are deferred",
        format!("Source '{source_id}' contains BPMN data element '<{tag}>'."),
        "The bounded engine keeps workflow data in JSON variables and host payloads; it does not yet execute BPMN data objects or data stores.",
        vec![
            "Represent runtime data through workflow variables, host-work input/output payloads, or DMN decision inputs.".to_string(),
            "Remove `<bpmn:dataObject*>` and `<bpmn:dataStore*>` dependencies from the executable slice.".to_string(),
            "If the data artifact is documentation-only, keep that meaning outside the executable BPMN subset.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing `<{tag}>` execution semantics with explicit JSON variables, host-work payload fields, or DMN inputs. Preserve workflow intent, but remove BPMN data-object or data-store dependencies from this bounded executable slice."
        ),
        document_surface_evidence(source, tag, "data"),
    )
}

fn document_surface_evidence(source: &BpmnSourceFile, tag: &str, family: &str) -> Value {
    let Ok(snapshot) = snapshot_bpmn_source(source) else {
        return json!({
            "source_id": source.source_id,
            "element": tag,
            "deferred_family": family,
            "snapshot_available": false,
        });
    };

    json!({
        "source_id": source.source_id,
        "element": tag,
        "deferred_family": family,
        "snapshot_available": true,
        "snapshot": snapshot_family_summary(&snapshot, family),
    })
}

fn snapshot_family_summary(snapshot: &BpmnDocumentSnapshot, family: &str) -> Value {
    match family {
        "collaboration" => collaboration_snapshot_summary(snapshot),
        "data" => data_snapshot_summary(snapshot),
        _ => json!({ "root": root_snapshot_summary(snapshot) }),
    }
}

fn root_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    json!({
        "definitions_id": snapshot.root.definitions_id,
        "model_namespace_uri": snapshot.root.model_namespace_uri,
        "import_count": snapshot.root.import_count,
        "extension_count": snapshot.root.extension_count,
        "relationship_count": snapshot.root.relationship_count,
        "diagram_count": snapshot.root.diagram_count,
        "collaboration_count": snapshot.root.collaboration_count,
        "process_count": snapshot.root.process_count,
        "item_definition_count": snapshot.root.item_definition_count,
        "message_count": snapshot.root.message_count,
        "interface_count": snapshot.root.interface_count,
        "resource_count": snapshot.root.resource_count,
        "category_count": snapshot.root.category_count,
        "correlation_property_count": snapshot.root.correlation_property_count,
        "error_count": snapshot.root.error_count,
        "escalation_count": snapshot.root.escalation_count,
        "signal_count": snapshot.root.signal_count,
        "data_store_count": snapshot.root.data_store_count,
    })
}

fn collaboration_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let counts = collaboration_counts(snapshot);
    let collaborations = snapshot
        .collaborations
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(collaboration_evidence)
        .collect::<Vec<_>>();
    let item_definitions = item_definition_evidence(snapshot);
    let messages = message_evidence(snapshot);
    let interfaces = interface_evidence(snapshot);
    let correlation_properties = correlation_property_evidence(snapshot);

    json!({
        "root": root_snapshot_summary(snapshot),
        "collaboration_count": snapshot.collaborations.len(),
        "participant_count": counts.participant,
        "message_flow_count": counts.message_flow,
        "conversation_node_count": counts.conversation_node,
        "conversation_link_count": counts.conversation_link,
        "conversation_association_count": counts.conversation_association,
        "participant_association_count": counts.participant_association,
        "message_flow_association_count": counts.message_flow_association,
        "correlation_key_count": counts.correlation_key,
        "choreography_activity_count": counts.choreography_activity,
        "item_definition_count": snapshot.root.item_definition_count,
        "message_count": snapshot.root.message_count,
        "interface_count": snapshot.root.interface_count,
        "correlation_property_count": snapshot.root.correlation_property_count,
        "collaborations_truncated": snapshot.collaborations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "item_definitions_truncated": snapshot.root.item_definitions.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "messages_truncated": snapshot.root.messages.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "interfaces_truncated": snapshot.root.interfaces.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "correlation_properties_truncated": snapshot.root.correlation_properties.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "item_definitions": item_definitions,
        "messages": messages,
        "interfaces": interfaces,
        "correlation_properties": correlation_properties,
        "collaborations": collaborations,
    })
}

#[derive(Debug, Default)]
struct CollaborationCounts {
    participant: usize,
    message_flow: usize,
    conversation_node: usize,
    conversation_link: usize,
    conversation_association: usize,
    participant_association: usize,
    message_flow_association: usize,
    correlation_key: usize,
    choreography_activity: usize,
}

fn collaboration_counts(snapshot: &BpmnDocumentSnapshot) -> CollaborationCounts {
    snapshot.collaborations.iter().fold(
        CollaborationCounts::default(),
        |mut counts, collaboration| {
            counts.participant += collaboration.participants.len();
            counts.message_flow += collaboration.message_flows.len();
            counts.conversation_node += collaboration
                .conversation_nodes
                .iter()
                .map(conversation_node_count)
                .sum::<usize>();
            counts.conversation_link += collaboration.conversation_links.len();
            counts.conversation_association += collaboration.conversation_associations.len();
            counts.participant_association += collaboration.participant_associations.len();
            counts.message_flow_association += collaboration.message_flow_associations.len();
            counts.correlation_key += collaboration_correlation_key_count(collaboration);
            counts.choreography_activity += collaboration
                .choreography_activities
                .iter()
                .map(choreography_activity_count)
                .sum::<usize>();
            counts
        },
    )
}

fn collaboration_evidence(collaboration: &BpmnCollaborationSnapshot) -> Value {
    json!({
        "collaboration_id": collaboration.collaboration_id,
        "participant_count": collaboration.participants.len(),
        "message_flow_count": collaboration.message_flows.len(),
        "conversation_node_count": collaboration.conversation_nodes.iter().map(conversation_node_count).sum::<usize>(),
        "conversation_link_count": collaboration.conversation_links.len(),
        "conversation_association_count": collaboration.conversation_associations.len(),
        "participant_association_count": collaboration.participant_associations.len(),
        "message_flow_association_count": collaboration.message_flow_associations.len(),
        "correlation_key_count": collaboration_correlation_key_count(collaboration),
        "choreography_ref_count": collaboration.choreography_refs.len(),
        "choreography_activity_count": collaboration.choreography_activities.iter().map(choreography_activity_count).sum::<usize>(),
        "initiating_participant_ref": collaboration.initiating_participant_ref,
        "participants": collaboration.participants.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|participant| {
            json!({
                "participant_id": participant.participant_id,
                "process_ref": participant.process_ref,
            })
        }).collect::<Vec<_>>(),
        "message_flows": collaboration.message_flows.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|flow| {
            json!({
                "message_flow_id": flow.message_flow_id,
                "source_ref": flow.source_ref,
                "target_ref": flow.target_ref,
                "message_ref": flow.message_ref,
            })
        }).collect::<Vec<_>>(),
        "conversation_nodes": collaboration.conversation_nodes.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(conversation_node_evidence).collect::<Vec<_>>(),
        "choreography_activities": collaboration.choreography_activities.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(choreography_activity_evidence).collect::<Vec<_>>(),
        "conversation_links": collaboration.conversation_links.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|link| {
            json!({
                "link_id": link.link_id,
                "source_ref": link.source_ref,
                "target_ref": link.target_ref,
            })
        }).collect::<Vec<_>>(),
    })
}

fn conversation_node_count(node: &BpmnConversationNodeSnapshot) -> usize {
    1 + node
        .child_nodes
        .iter()
        .map(conversation_node_count)
        .sum::<usize>()
}

fn collaboration_correlation_key_count(collaboration: &BpmnCollaborationSnapshot) -> usize {
    collaboration.correlation_keys.len()
        + collaboration
            .conversation_nodes
            .iter()
            .map(conversation_node_correlation_key_count)
            .sum::<usize>()
        + collaboration
            .choreography_activities
            .iter()
            .map(choreography_activity_correlation_key_count)
            .sum::<usize>()
}

fn conversation_node_correlation_key_count(node: &BpmnConversationNodeSnapshot) -> usize {
    node.correlation_keys.len()
        + node
            .child_nodes
            .iter()
            .map(conversation_node_correlation_key_count)
            .sum::<usize>()
}

fn conversation_node_evidence(node: &BpmnConversationNodeSnapshot) -> Value {
    json!({
        "node_kind": node.node_kind,
        "node_id": node.node_id,
        "called_collaboration_ref": node.called_collaboration_ref,
        "participant_refs": node.participant_refs,
        "message_flow_refs": node.message_flow_refs,
        "correlation_key_count": conversation_node_correlation_key_count(node),
        "participant_association_count": node.participant_associations.len(),
        "child_node_count": node.child_nodes.iter().map(conversation_node_count).sum::<usize>(),
    })
}

fn choreography_activity_count(activity: &BpmnChoreographyActivitySnapshot) -> usize {
    1 + activity
        .child_activities
        .iter()
        .map(choreography_activity_count)
        .sum::<usize>()
}

fn choreography_activity_correlation_key_count(
    activity: &BpmnChoreographyActivitySnapshot,
) -> usize {
    activity.correlation_keys.len()
        + activity
            .child_activities
            .iter()
            .map(choreography_activity_correlation_key_count)
            .sum::<usize>()
}

fn choreography_activity_evidence(activity: &BpmnChoreographyActivitySnapshot) -> Value {
    json!({
        "activity_kind": activity.activity_kind,
        "activity_id": activity.activity_id,
        "initiating_participant_ref": activity.initiating_participant_ref,
        "loop_type": activity.loop_type,
        "called_choreography_ref": activity.called_choreography_ref,
        "participant_refs": activity.participant_refs,
        "message_flow_refs": activity.message_flow_refs,
        "correlation_key_count": choreography_activity_correlation_key_count(activity),
        "participant_association_count": activity.participant_associations.len(),
        "child_activity_count": activity.child_activities.iter().map(choreography_activity_count).sum::<usize>(),
    })
}

fn item_definition_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .item_definitions
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|item_definition| {
            json!({
                "item_definition_id": item_definition.item_definition_id,
                "structure_ref": item_definition.structure_ref,
                "item_kind": item_definition.item_kind,
                "is_collection": item_definition.is_collection,
            })
        })
        .collect()
}

fn message_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .messages
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|message| {
            json!({
                "message_id": message.message_id,
                "name": message.name,
                "item_ref": message.item_ref,
            })
        })
        .collect()
}

fn interface_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .interfaces
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|interface| {
            json!({
                "interface_id": interface.interface_id,
                "name": interface.name,
                "implementation_ref": interface.implementation_ref,
                "operation_count": interface.operations.len(),
                "operations": interface.operations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|operation| {
                    json!({
                        "operation_id": operation.operation_id,
                        "name": operation.name,
                        "implementation_ref": operation.implementation_ref,
                        "in_message_ref": operation.in_message_ref,
                        "out_message_ref": operation.out_message_ref,
                        "error_refs": operation.error_refs,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn correlation_property_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .correlation_properties
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|property| {
            json!({
                "correlation_property_id": property.correlation_property_id,
                "name": property.name,
                "type_ref": property.type_ref,
                "retrieval_expression_count": property.retrieval_expressions.len(),
                "retrieval_expressions": property.retrieval_expressions.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|retrieval| {
                    json!({
                        "retrieval_expression_id": retrieval.retrieval_expression_id,
                        "message_ref": retrieval.message_ref,
                        "message_path": retrieval.message_path,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn data_snapshot_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let data_object_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_object_count)
        .sum::<usize>();
    let data_object_reference_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_object_reference_count)
        .sum::<usize>();
    let data_store_reference_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_store_reference_count)
        .sum::<usize>();
    let io_specification_count = snapshot
        .processes
        .iter()
        .map(|process| process.io_specification_count)
        .sum::<usize>();
    let data_input_association_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_input_association_count)
        .sum::<usize>();
    let data_output_association_count = snapshot
        .processes
        .iter()
        .map(|process| process.data_output_association_count)
        .sum::<usize>();
    json!({
        "root": root_snapshot_summary(snapshot),
        "item_definition_count": snapshot.root.item_definition_count,
        "item_definitions": item_definition_evidence(snapshot),
        "data_object_count": data_object_count,
        "data_object_reference_count": data_object_reference_count,
        "data_store_count": snapshot.root.data_store_count,
        "data_store_reference_count": data_store_reference_count,
        "io_specification_count": io_specification_count,
        "data_input_association_count": data_input_association_count,
        "data_output_association_count": data_output_association_count,
        "process_data_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "process_data": process_data_evidence(snapshot),
    })
}

fn process_data_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| {
            process.data_object_count
                + process.data_object_reference_count
                + process.data_store_reference_count
                + process.io_specification_count
                + process.data_input_association_count
                + process.data_output_association_count
                > 0
        })
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "data_object_count": process.data_object_count,
                "data_object_reference_count": process.data_object_reference_count,
                "data_store_reference_count": process.data_store_reference_count,
                "io_specification_count": process.io_specification_count,
                "data_input_association_count": process.data_input_association_count,
                "data_output_association_count": process.data_output_association_count,
                "data_objects": process.data_objects.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|object| {
                    json!({
                        "data_object_id": object.data_object_id,
                        "name": object.name,
                        "item_subject_ref": object.item_subject_ref,
                    })
                }).collect::<Vec<_>>(),
                "data_object_references": process.data_object_references.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|reference| {
                    json!({
                        "data_object_reference_id": reference.data_object_reference_id,
                        "data_object_ref": reference.data_object_ref,
                    })
                }).collect::<Vec<_>>(),
                "data_store_references": process.data_store_references.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|reference| {
                    json!({
                        "data_store_reference_id": reference.data_store_reference_id,
                        "data_store_ref": reference.data_store_ref,
                    })
                }).collect::<Vec<_>>(),
                "data_input_associations": process.data_input_associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|association| {
                    json!({
                        "association_id": association.association_id,
                        "source_refs": association.source_refs,
                        "target_ref": association.target_ref,
                    })
                }).collect::<Vec<_>>(),
                "data_output_associations": process.data_output_associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|association| {
                    json!({
                        "association_id": association.association_id,
                        "source_refs": association.source_refs,
                        "target_ref": association.target_ref,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}
