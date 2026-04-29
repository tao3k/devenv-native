use crate::bpmn_model_api::{
    BpmnAssociationSnapshot, BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationNodeSnapshot, BpmnDataAssociationExpressionSnapshot,
    BpmnDataAssociationSnapshot, BpmnDataStateSnapshot, BpmnDocumentSnapshot,
    BpmnFlowElementMetadataSnapshot, BpmnGlobalTaskSnapshot, BpmnGroupSnapshot,
    BpmnIoBindingSnapshot, BpmnParticipantSnapshot, BpmnPartnerEntitySnapshot,
    BpmnPartnerRoleSnapshot, BpmnProcessSnapshot, BpmnResourceRoleSnapshot,
    BpmnTextAnnotationSnapshot,
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
        | "partnerEntity"
        | "partnerRole"
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
        "end_point_count": snapshot.root.end_point_count,
        "resource_count": snapshot.root.resource_count,
        "category_count": snapshot.root.category_count,
        "correlation_property_count": snapshot.root.correlation_property_count,
        "error_count": snapshot.root.error_count,
        "escalation_count": snapshot.root.escalation_count,
        "signal_count": snapshot.root.signal_count,
        "data_store_count": snapshot.root.data_store_count,
        "partner_entity_count": snapshot.root.partner_entity_count,
        "partner_role_count": snapshot.root.partner_role_count,
        "global_task_count": snapshot.root.global_task_count,
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
    let partner_entities = partner_entity_evidence(snapshot);
    let partner_roles = partner_role_evidence(snapshot);
    let correlation_properties = correlation_property_evidence(snapshot);
    let process_callable = process_callable_summary(snapshot);
    let resource_roles = resource_role_summary(snapshot);
    let flow_element_metadata = flow_element_metadata_summary(snapshot);

    json!({
        "root": root_snapshot_summary(snapshot),
        "collaboration_count": snapshot.collaborations.len(),
        "partner_entity_count": snapshot.root.partner_entity_count,
        "partner_role_count": snapshot.root.partner_role_count,
        "end_point_count": snapshot.root.end_point_count,
        "participant_count": counts.participant,
        "participant_interface_ref_count": counts.participant_interface_ref,
        "participant_end_point_ref_count": counts.participant_end_point_ref,
        "participant_multiplicity_count": counts.participant_multiplicity,
        "message_flow_count": counts.message_flow,
        "conversation_node_count": counts.conversation_node,
        "conversation_link_count": counts.conversation_link,
        "conversation_association_count": counts.conversation_association,
        "participant_association_count": counts.participant_association,
        "message_flow_association_count": counts.message_flow_association,
        "correlation_key_count": counts.correlation_key,
        "choreography_activity_count": counts.choreography_activity,
        "artifact_association_count": counts.association,
        "artifact_group_count": counts.group,
        "text_annotation_count": counts.text_annotation,
        "item_definition_count": snapshot.root.item_definition_count,
        "message_count": snapshot.root.message_count,
        "interface_count": snapshot.root.interface_count,
        "correlation_property_count": snapshot.root.correlation_property_count,
        "partner_entities_truncated": snapshot.root.partner_entities.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "partner_roles_truncated": snapshot.root.partner_roles.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "collaborations_truncated": snapshot.collaborations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "item_definitions_truncated": snapshot.root.item_definitions.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "messages_truncated": snapshot.root.messages.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "interfaces_truncated": snapshot.root.interfaces.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "correlation_properties_truncated": snapshot.root.correlation_properties.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "process_callable": process_callable,
        "resource_roles": resource_roles,
        "flow_element_metadata": flow_element_metadata,
        "item_definitions": item_definitions,
        "messages": messages,
        "interfaces": interfaces,
        "partner_entities": partner_entities,
        "partner_roles": partner_roles,
        "correlation_properties": correlation_properties,
        "collaborations": collaborations,
    })
}

#[derive(Debug, Default)]
struct CollaborationCounts {
    participant: usize,
    participant_interface_ref: usize,
    participant_end_point_ref: usize,
    participant_multiplicity: usize,
    message_flow: usize,
    conversation_node: usize,
    conversation_link: usize,
    conversation_association: usize,
    participant_association: usize,
    message_flow_association: usize,
    correlation_key: usize,
    choreography_activity: usize,
    association: usize,
    group: usize,
    text_annotation: usize,
}

#[derive(Debug, Default)]
struct ProcessCallableCounts {
    support: usize,
    property: usize,
    correlation_subscription: usize,
    correlation_binding: usize,
    process_io_binding: usize,
    global_task_io_specification: usize,
    global_task_io_binding: usize,
}

#[derive(Debug, Default)]
struct ResourceRoleCounts {
    process_role: usize,
    global_task_role: usize,
    parameter_binding: usize,
    assignment_expression: usize,
}

#[derive(Debug, Default)]
struct FlowElementMetadataCounts {
    element: usize,
    auditing: usize,
    monitoring: usize,
    category_value_ref: usize,
}

fn flow_element_metadata_counts(snapshot: &BpmnDocumentSnapshot) -> FlowElementMetadataCounts {
    let mut counts = FlowElementMetadataCounts::default();
    for process in &snapshot.processes {
        counts.element += process.flow_element_metadata_count;
        for metadata in &process.flow_element_metadata {
            counts.auditing += usize::from(metadata.has_auditing);
            counts.monitoring += usize::from(metadata.has_monitoring);
            counts.category_value_ref += metadata.category_value_refs.len();
        }
    }
    counts
}

fn flow_element_metadata_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let counts = flow_element_metadata_counts(snapshot);
    json!({
        "element_count": counts.element,
        "auditing_count": counts.auditing,
        "monitoring_count": counts.monitoring,
        "category_value_ref_count": counts.category_value_ref,
        "processes_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "processes": process_flow_element_metadata_evidence(snapshot),
    })
}

fn process_flow_element_metadata_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| process.flow_element_metadata_count > 0)
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "flow_element_metadata_count": process.flow_element_metadata_count,
                "flow_elements": process.flow_element_metadata.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(flow_element_metadata_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn flow_element_metadata_evidence(metadata: &BpmnFlowElementMetadataSnapshot) -> Value {
    json!({
        "element_kind": metadata.element_kind,
        "element_id": metadata.element_id,
        "name": metadata.name,
        "has_auditing": metadata.has_auditing,
        "auditing_id": metadata.auditing_id,
        "has_monitoring": metadata.has_monitoring,
        "monitoring_id": metadata.monitoring_id,
        "category_value_refs": metadata.category_value_refs,
    })
}

fn resource_role_counts(snapshot: &BpmnDocumentSnapshot) -> ResourceRoleCounts {
    let mut counts = ResourceRoleCounts::default();
    for process in &snapshot.processes {
        counts.process_role += process.resource_role_count;
        for role in &process.resource_roles {
            counts.parameter_binding += role.parameter_bindings.len();
            counts.assignment_expression += usize::from(role.assignment_expression.is_some());
        }
    }
    for task in &snapshot.root.global_tasks {
        counts.global_task_role += task.resource_role_count;
        for role in &task.resource_roles {
            counts.parameter_binding += role.parameter_bindings.len();
            counts.assignment_expression += usize::from(role.assignment_expression.is_some());
        }
    }
    counts
}

fn resource_role_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let counts = resource_role_counts(snapshot);
    json!({
        "process_role_count": counts.process_role,
        "global_task_role_count": counts.global_task_role,
        "parameter_binding_count": counts.parameter_binding,
        "assignment_expression_count": counts.assignment_expression,
        "processes_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "global_tasks_truncated": snapshot.root.global_tasks.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "processes": process_resource_role_evidence(snapshot),
        "global_tasks": global_task_resource_role_evidence(snapshot),
    })
}

fn process_resource_role_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| process.resource_role_count > 0)
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "resource_role_count": process.resource_role_count,
                "resource_roles": process.resource_roles.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(resource_role_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn global_task_resource_role_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .global_tasks
        .iter()
        .filter(|task| task.resource_role_count > 0)
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(global_task_resource_role_item_evidence)
        .collect()
}

fn global_task_resource_role_item_evidence(task: &BpmnGlobalTaskSnapshot) -> Value {
    json!({
        "task_kind": task.task_kind,
        "task_id": task.task_id,
        "resource_role_count": task.resource_role_count,
        "resource_roles": task.resource_roles.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(resource_role_evidence).collect::<Vec<_>>(),
    })
}

fn resource_role_evidence(role: &BpmnResourceRoleSnapshot) -> Value {
    json!({
        "role_kind": role.role_kind,
        "role_id": role.role_id,
        "name": role.name,
        "resource_ref": role.resource_ref,
        "assignment_expression_id": role.assignment_expression_id,
        "assignment_expression": role.assignment_expression,
        "assignment_expression_language": role.assignment_expression_language,
        "assignment_expression_evaluates_to_type_ref": role.assignment_expression_evaluates_to_type_ref,
        "parameter_binding_count": role.parameter_bindings.len(),
        "parameter_bindings": role.parameter_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|binding| {
            json!({
                "binding_id": binding.binding_id,
                "parameter_ref": binding.parameter_ref,
                "expression": binding.expression,
                "expression_language": binding.expression_language,
                "expression_evaluates_to_type_ref": binding.expression_evaluates_to_type_ref,
            })
        }).collect::<Vec<_>>(),
    })
}

fn process_callable_counts(snapshot: &BpmnDocumentSnapshot) -> ProcessCallableCounts {
    let mut counts =
        snapshot
            .processes
            .iter()
            .fold(ProcessCallableCounts::default(), |mut counts, process| {
                counts.support += process.support_count;
                counts.property += process.property_count;
                counts.correlation_subscription += process.correlation_subscription_count;
                counts.correlation_binding += process
                    .correlation_subscriptions
                    .iter()
                    .map(|subscription| subscription.bindings.len())
                    .sum::<usize>();
                counts.process_io_binding += process.io_binding_count;
                counts
            });
    counts.global_task_io_specification = snapshot
        .root
        .global_tasks
        .iter()
        .map(|task| task.io_specification_count)
        .sum();
    counts.global_task_io_binding = snapshot
        .root
        .global_tasks
        .iter()
        .map(|task| task.io_binding_count)
        .sum();
    counts
}

fn process_callable_summary(snapshot: &BpmnDocumentSnapshot) -> Value {
    let counts = process_callable_counts(snapshot);
    json!({
        "support_count": counts.support,
        "property_count": counts.property,
        "correlation_subscription_count": counts.correlation_subscription,
        "correlation_binding_count": counts.correlation_binding,
        "process_io_binding_count": counts.process_io_binding,
        "global_task_io_specification_count": counts.global_task_io_specification,
        "global_task_io_binding_count": counts.global_task_io_binding,
        "metadata_truncated": snapshot.processes.len() > SNAPSHOT_EVIDENCE_LIMIT || snapshot.root.global_tasks.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "processes": process_callable_metadata_evidence(snapshot),
        "global_tasks": global_task_callable_metadata_evidence(snapshot),
    })
}

fn process_callable_metadata_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .processes
        .iter()
        .filter(|process| has_process_callable_metadata(process))
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|process| {
            json!({
                "process_id": process.process_id,
                "process_type": process.process_type,
                "is_closed": process.is_closed,
                "is_executable": process.is_executable,
                "definitional_collaboration_ref": process.definitional_collaboration_ref,
                "support_count": process.support_count,
                "supports": process.supports,
                "property_count": process.property_count,
                "properties": process.properties.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|property| {
                    json!({
                        "property_id": property.property_id,
                        "name": property.name,
                        "item_subject_ref": property.item_subject_ref,
                    })
                }).collect::<Vec<_>>(),
                "correlation_subscription_count": process.correlation_subscription_count,
                "correlation_subscriptions": process.correlation_subscriptions.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|subscription| {
                    json!({
                        "subscription_id": subscription.subscription_id,
                        "correlation_key_ref": subscription.correlation_key_ref,
                        "binding_count": subscription.bindings.len(),
                        "bindings": subscription.bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|binding| {
                            json!({
                                "binding_id": binding.binding_id,
                                "correlation_property_ref": binding.correlation_property_ref,
                                "data_path": binding.data_path,
                                "data_path_language": binding.data_path_language,
                                "data_path_evaluates_to_type_ref": binding.data_path_evaluates_to_type_ref,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "io_binding_count": process.io_binding_count,
                "io_bindings": process.io_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(io_binding_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn global_task_callable_metadata_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .global_tasks
        .iter()
        .filter(|task| has_global_task_callable_metadata(task))
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(|task| {
            json!({
                "task_kind": task.task_kind,
                "task_id": task.task_id,
                "name": task.name,
                "supported_interface_refs": task.supported_interface_refs,
                "io_specification_count": task.io_specification_count,
                "io_specifications": task.io_specifications.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|spec| {
                    json!({
                        "io_specification_id": spec.io_specification_id,
                        "data_input_count": spec.data_inputs.len(),
                        "data_inputs": spec.data_inputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|input| {
                            json!({
                                "data_id": input.data_id,
                                "name": input.name,
                                "item_subject_ref": input.item_subject_ref,
                                "is_collection": input.is_collection,
                            })
                        }).collect::<Vec<_>>(),
                        "data_output_count": spec.data_outputs.len(),
                        "data_outputs": spec.data_outputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|output| {
                            json!({
                                "data_id": output.data_id,
                                "name": output.name,
                                "item_subject_ref": output.item_subject_ref,
                                "is_collection": output.is_collection,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "io_binding_count": task.io_binding_count,
                "io_bindings": task.io_bindings.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(io_binding_evidence).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn io_binding_evidence(binding: &BpmnIoBindingSnapshot) -> Value {
    json!({
        "binding_id": binding.binding_id,
        "operation_ref": binding.operation_ref,
        "input_data_ref": binding.input_data_ref,
        "output_data_ref": binding.output_data_ref,
    })
}

fn has_process_callable_metadata(process: &BpmnProcessSnapshot) -> bool {
    process.process_type.is_some()
        || process.is_closed.is_some()
        || process.definitional_collaboration_ref.is_some()
        || process.support_count > 0
        || process.property_count > 0
        || process.correlation_subscription_count > 0
        || process.io_binding_count > 0
}

fn has_global_task_callable_metadata(task: &BpmnGlobalTaskSnapshot) -> bool {
    !task.supported_interface_refs.is_empty()
        || task.io_specification_count > 0
        || task.io_binding_count > 0
}

fn collaboration_counts(snapshot: &BpmnDocumentSnapshot) -> CollaborationCounts {
    snapshot.collaborations.iter().fold(
        CollaborationCounts::default(),
        |mut counts, collaboration| {
            counts.participant += collaboration.participants.len();
            for participant in &collaboration.participants {
                counts.participant_interface_ref += participant.interface_refs.len();
                counts.participant_end_point_ref += participant.end_point_refs.len();
                counts.participant_multiplicity +=
                    usize::from(participant.participant_multiplicity.is_some());
            }
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
            counts.association += collaboration.associations.len();
            counts.group += collaboration.groups.len();
            counts.text_annotation += collaboration.text_annotations.len();
            counts
        },
    )
}

fn collaboration_evidence(collaboration: &BpmnCollaborationSnapshot) -> Value {
    json!({
        "collaboration_id": collaboration.collaboration_id,
        "participant_count": collaboration.participants.len(),
        "participant_interface_ref_count": collaboration.participants.iter().map(|participant| participant.interface_refs.len()).sum::<usize>(),
        "participant_end_point_ref_count": collaboration.participants.iter().map(|participant| participant.end_point_refs.len()).sum::<usize>(),
        "participant_multiplicity_count": collaboration.participants.iter().filter(|participant| participant.participant_multiplicity.is_some()).count(),
        "message_flow_count": collaboration.message_flows.len(),
        "conversation_node_count": collaboration.conversation_nodes.iter().map(conversation_node_count).sum::<usize>(),
        "conversation_link_count": collaboration.conversation_links.len(),
        "conversation_association_count": collaboration.conversation_associations.len(),
        "participant_association_count": collaboration.participant_associations.len(),
        "message_flow_association_count": collaboration.message_flow_associations.len(),
        "correlation_key_count": collaboration_correlation_key_count(collaboration),
        "choreography_ref_count": collaboration.choreography_refs.len(),
        "choreography_activity_count": collaboration.choreography_activities.iter().map(choreography_activity_count).sum::<usize>(),
        "artifact_association_count": collaboration.associations.len(),
        "artifact_group_count": collaboration.groups.len(),
        "text_annotation_count": collaboration.text_annotations.len(),
        "initiating_participant_ref": collaboration.initiating_participant_ref,
        "participants": collaboration.participants.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(participant_evidence).collect::<Vec<_>>(),
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
        "associations": collaboration.associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(artifact_association_evidence).collect::<Vec<_>>(),
        "groups": collaboration.groups.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(artifact_group_evidence).collect::<Vec<_>>(),
        "text_annotations": collaboration.text_annotations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(text_annotation_evidence).collect::<Vec<_>>(),
        "conversation_links": collaboration.conversation_links.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|link| {
            json!({
                "link_id": link.link_id,
                "source_ref": link.source_ref,
                "target_ref": link.target_ref,
            })
        }).collect::<Vec<_>>(),
    })
}

fn participant_evidence(participant: &BpmnParticipantSnapshot) -> Value {
    json!({
        "participant_id": participant.participant_id,
        "name": participant.name,
        "process_ref": participant.process_ref,
        "interface_refs": participant.interface_refs,
        "end_point_refs": participant.end_point_refs,
        "participant_multiplicity": participant.participant_multiplicity.as_ref().map(|multiplicity| {
            json!({
                "multiplicity_id": multiplicity.multiplicity_id,
                "minimum": multiplicity.minimum,
                "maximum": multiplicity.maximum,
            })
        }),
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

fn artifact_association_evidence(association: &BpmnAssociationSnapshot) -> Value {
    json!({
        "association_id": association.association_id,
        "source_ref": association.source_ref,
        "target_ref": association.target_ref,
        "association_direction": association.association_direction,
    })
}

fn artifact_group_evidence(group: &BpmnGroupSnapshot) -> Value {
    json!({
        "group_id": group.group_id,
        "category_value_ref": group.category_value_ref,
    })
}

fn text_annotation_evidence(annotation: &BpmnTextAnnotationSnapshot) -> Value {
    json!({
        "annotation_id": annotation.annotation_id,
        "text_format": annotation.text_format,
        "text": annotation.text,
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

fn partner_entity_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .partner_entities
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(partner_entity_item_evidence)
        .collect()
}

fn partner_entity_item_evidence(partner_entity: &BpmnPartnerEntitySnapshot) -> Value {
    json!({
        "partner_entity_id": partner_entity.partner_entity_id,
        "name": partner_entity.name,
        "participant_refs": partner_entity.participant_refs,
    })
}

fn partner_role_evidence(snapshot: &BpmnDocumentSnapshot) -> Vec<Value> {
    snapshot
        .root
        .partner_roles
        .iter()
        .take(SNAPSHOT_EVIDENCE_LIMIT)
        .map(partner_role_item_evidence)
        .collect()
}

fn partner_role_item_evidence(partner_role: &BpmnPartnerRoleSnapshot) -> Value {
    json!({
        "partner_role_id": partner_role.partner_role_id,
        "name": partner_role.name,
        "participant_refs": partner_role.participant_refs,
    })
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
        "data_stores": snapshot.root.data_stores.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|store| {
            json!({
                "data_store_id": store.data_store_id,
                "name": store.name,
                "item_subject_ref": store.item_subject_ref,
                "capacity": store.capacity,
                "is_unlimited": store.is_unlimited,
                "data_state": data_state_evidence(store.data_state.as_ref()),
            })
        }).collect::<Vec<_>>(),
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
                        "is_collection": object.is_collection,
                        "data_state": data_state_evidence(object.data_state.as_ref()),
                    })
                }).collect::<Vec<_>>(),
                "data_object_references": process.data_object_references.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|reference| {
                    json!({
                        "data_object_reference_id": reference.data_object_reference_id,
                        "name": reference.name,
                        "data_object_ref": reference.data_object_ref,
                        "item_subject_ref": reference.item_subject_ref,
                        "data_state": data_state_evidence(reference.data_state.as_ref()),
                    })
                }).collect::<Vec<_>>(),
                "data_store_references": process.data_store_references.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|reference| {
                    json!({
                        "data_store_reference_id": reference.data_store_reference_id,
                        "name": reference.name,
                        "data_store_ref": reference.data_store_ref,
                        "item_subject_ref": reference.item_subject_ref,
                        "data_state": data_state_evidence(reference.data_state.as_ref()),
                    })
                }).collect::<Vec<_>>(),
                "io_specifications": process.io_specifications.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|spec| {
                    json!({
                        "io_specification_id": spec.io_specification_id,
                        "data_inputs": spec.data_inputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|input| {
                            json!({
                                "data_id": input.data_id,
                                "name": input.name,
                                "item_subject_ref": input.item_subject_ref,
                                "is_collection": input.is_collection,
                                "data_state": data_state_evidence(input.data_state.as_ref()),
                            })
                        }).collect::<Vec<_>>(),
                        "data_outputs": spec.data_outputs.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|output| {
                            json!({
                                "data_id": output.data_id,
                                "name": output.name,
                                "item_subject_ref": output.item_subject_ref,
                                "is_collection": output.is_collection,
                                "data_state": data_state_evidence(output.data_state.as_ref()),
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "data_input_associations": process.data_input_associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|association| {
                    data_association_evidence(association)
                }).collect::<Vec<_>>(),
                "data_output_associations": process.data_output_associations.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|association| {
                    data_association_evidence(association)
                }).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn data_association_evidence(association: &BpmnDataAssociationSnapshot) -> Value {
    json!({
        "association_id": association.association_id,
        "source_refs": association.source_refs,
        "target_ref": association.target_ref,
        "transformation": data_association_expression_evidence(association.transformation.as_ref()),
        "assignment_count": association.assignments.len(),
        "assignments": association.assignments.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|assignment| {
            json!({
                "assignment_id": assignment.assignment_id,
                "from": data_association_expression_evidence(assignment.from.as_ref()),
                "to": data_association_expression_evidence(assignment.to.as_ref()),
            })
        }).collect::<Vec<_>>(),
        "assignments_truncated": association.assignments.len() > SNAPSHOT_EVIDENCE_LIMIT,
    })
}

fn data_association_expression_evidence(
    expression: Option<&BpmnDataAssociationExpressionSnapshot>,
) -> Value {
    expression.map_or(Value::Null, |expression| {
        json!({
            "expression_id": expression.expression_id,
            "body": expression.body,
            "language": expression.language,
            "evaluates_to_type_ref": expression.evaluates_to_type_ref,
        })
    })
}

fn data_state_evidence(state: Option<&BpmnDataStateSnapshot>) -> Value {
    state.map_or(Value::Null, |state| {
        json!({
            "data_state_id": state.data_state_id,
            "name": state.name,
        })
    })
}
