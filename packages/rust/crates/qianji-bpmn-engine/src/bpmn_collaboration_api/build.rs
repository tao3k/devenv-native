use super::api::{
    BpmnCollaborationHostEnvelope, BpmnCollaborationIntent, BpmnCorrelationKeyIntent,
    BpmnCorrelationKeyScope, BpmnCorrelationPropertyBindingIntent, BpmnCorrelationPropertyIntent,
    BpmnCorrelationPropertyRetrievalIntent, BpmnMessageFlowIntent, BpmnParticipantIntent,
    BpmnParticipantMultiplicityIntent, BpmnProcessCorrelationSubscriptionIntent,
};
use crate::bpmn_model_api::{
    BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot, BpmnConversationNodeSnapshot,
    BpmnCorrelationKeySnapshot, BpmnCorrelationPropertyBindingSnapshot,
    BpmnCorrelationPropertySnapshot, BpmnCorrelationSubscriptionSnapshot, BpmnDocumentSnapshot,
    BpmnMessageFlowSnapshot, BpmnParticipantMultiplicitySnapshot, BpmnParticipantSnapshot,
    BpmnSnapshotFlag,
};
use std::sync::Arc;

impl BpmnCollaborationHostEnvelope {
    /// Builds a host envelope from a BPMN document snapshot.
    #[must_use]
    pub(crate) fn from_document_snapshot(snapshot: &BpmnDocumentSnapshot) -> Self {
        let mut envelope = Self {
            source_id: (Some(Arc::<str>::from(snapshot.source_id.as_str()))),
            ..Self::default()
        };

        for collaboration in &snapshot.collaborations {
            envelope
                .collaborations
                .push(collaboration_intent(collaboration));
            envelope.participants.extend(
                collaboration
                    .participants
                    .iter()
                    .map(|participant| participant_intent(collaboration, participant)),
            );
            envelope.message_flows.extend(
                collaboration
                    .message_flows
                    .iter()
                    .map(|message_flow| message_flow_intent(collaboration, message_flow)),
            );
            push_collaboration_correlation_keys(&mut envelope, collaboration);
        }

        envelope.correlation_properties.extend(
            snapshot
                .root
                .correlation_properties
                .iter()
                .map(correlation_property_intent),
        );
        for process in &snapshot.processes {
            let Some(process_id) = process.process_id.as_deref() else {
                continue;
            };
            envelope.process_correlation_subscriptions.extend(
                process
                    .correlation_subscriptions
                    .iter()
                    .map(|subscription| {
                        process_correlation_subscription_intent(process_id, subscription)
                    }),
            );
        }

        if envelope.is_empty() {
            Self::default()
        } else {
            envelope
        }
    }
}

fn collaboration_intent(collaboration: &BpmnCollaborationSnapshot) -> BpmnCollaborationIntent {
    BpmnCollaborationIntent {
        collaboration_id: optional_arc(collaboration.collaboration_id.as_deref()),
        name: optional_arc(collaboration.name.as_deref()),
        is_closed: collaboration.is_closed.map(BpmnSnapshotFlag::get),
        initiating_participant_ref: optional_arc(
            collaboration.initiating_participant_ref.as_deref(),
        ),
    }
}

fn participant_intent(
    collaboration: &BpmnCollaborationSnapshot,
    participant: &BpmnParticipantSnapshot,
) -> BpmnParticipantIntent {
    BpmnParticipantIntent {
        collaboration_id: optional_arc(collaboration.collaboration_id.as_deref()),
        participant_id: optional_arc(participant.participant_id.as_deref()),
        name: optional_arc(participant.name.as_deref()),
        process_ref: optional_arc(participant.process_ref.as_deref()),
        interface_refs: arc_vec(&participant.interface_refs),
        end_point_refs: arc_vec(&participant.end_point_refs),
        participant_multiplicity: participant
            .participant_multiplicity
            .as_ref()
            .map(participant_multiplicity_intent),
    }
}

fn participant_multiplicity_intent(
    multiplicity: &BpmnParticipantMultiplicitySnapshot,
) -> BpmnParticipantMultiplicityIntent {
    BpmnParticipantMultiplicityIntent {
        multiplicity_id: optional_arc(multiplicity.multiplicity_id.as_deref()),
        minimum: optional_arc(multiplicity.minimum.as_deref()),
        maximum: optional_arc(multiplicity.maximum.as_deref()),
    }
}

fn message_flow_intent(
    collaboration: &BpmnCollaborationSnapshot,
    message_flow: &BpmnMessageFlowSnapshot,
) -> BpmnMessageFlowIntent {
    BpmnMessageFlowIntent {
        collaboration_id: optional_arc(collaboration.collaboration_id.as_deref()),
        message_flow_id: optional_arc(message_flow.message_flow_id.as_deref()),
        name: optional_arc(message_flow.name.as_deref()),
        source_ref: optional_arc(message_flow.source_ref.as_deref()),
        target_ref: optional_arc(message_flow.target_ref.as_deref()),
        message_ref: optional_arc(message_flow.message_ref.as_deref()),
    }
}

fn correlation_property_intent(
    property: &BpmnCorrelationPropertySnapshot,
) -> BpmnCorrelationPropertyIntent {
    BpmnCorrelationPropertyIntent {
        correlation_property_id: optional_arc(property.correlation_property_id.as_deref()),
        name: optional_arc(property.name.as_deref()),
        type_ref: optional_arc(property.type_ref.as_deref()),
        retrieval_expressions: property
            .retrieval_expressions
            .iter()
            .map(|retrieval| BpmnCorrelationPropertyRetrievalIntent {
                retrieval_expression_id: optional_arc(retrieval.retrieval_expression_id.as_deref()),
                message_ref: optional_arc(retrieval.message_ref.as_deref()),
                message_path: optional_arc(retrieval.message_path.as_deref()),
            })
            .collect(),
    }
}

fn push_collaboration_correlation_keys(
    envelope: &mut BpmnCollaborationHostEnvelope,
    collaboration: &BpmnCollaborationSnapshot,
) {
    let collaboration_id = optional_arc(collaboration.collaboration_id.as_deref());
    envelope
        .correlation_keys
        .extend(collaboration.correlation_keys.iter().map(|key| {
            correlation_key_intent(
                BpmnCorrelationKeyScope::Collaboration,
                collaboration_id.clone(),
                key,
            )
        }));
    for node in &collaboration.conversation_nodes {
        push_conversation_correlation_keys(envelope, node);
    }
    for activity in &collaboration.choreography_activities {
        push_choreography_correlation_keys(envelope, activity);
    }
}

fn push_conversation_correlation_keys(
    envelope: &mut BpmnCollaborationHostEnvelope,
    node: &BpmnConversationNodeSnapshot,
) {
    let scope_id = optional_arc(node.node_id.as_deref());
    envelope
        .correlation_keys
        .extend(node.correlation_keys.iter().map(|key| {
            correlation_key_intent(BpmnCorrelationKeyScope::Conversation, scope_id.clone(), key)
        }));
    for child in &node.child_nodes {
        push_conversation_correlation_keys(envelope, child);
    }
}

fn push_choreography_correlation_keys(
    envelope: &mut BpmnCollaborationHostEnvelope,
    activity: &BpmnChoreographyActivitySnapshot,
) {
    let scope_id = optional_arc(activity.activity_id.as_deref());
    envelope
        .correlation_keys
        .extend(activity.correlation_keys.iter().map(|key| {
            correlation_key_intent(BpmnCorrelationKeyScope::Choreography, scope_id.clone(), key)
        }));
    for child in &activity.child_activities {
        push_choreography_correlation_keys(envelope, child);
    }
}

fn correlation_key_intent(
    scope: BpmnCorrelationKeyScope,
    scope_id: Option<Arc<str>>,
    key: &BpmnCorrelationKeySnapshot,
) -> BpmnCorrelationKeyIntent {
    BpmnCorrelationKeyIntent {
        scope,
        scope_id,
        correlation_key_id: optional_arc(key.correlation_key_id.as_deref()),
        name: optional_arc(key.name.as_deref()),
        correlation_property_refs: arc_vec(&key.correlation_property_refs),
    }
}

fn process_correlation_subscription_intent(
    process_id: &str,
    subscription: &BpmnCorrelationSubscriptionSnapshot,
) -> BpmnProcessCorrelationSubscriptionIntent {
    BpmnProcessCorrelationSubscriptionIntent {
        process_id: (Arc::<str>::from(process_id)),
        subscription_id: optional_arc(subscription.subscription_id.as_deref()),
        correlation_key_ref: optional_arc(subscription.correlation_key_ref.as_deref()),
        bindings: subscription
            .bindings
            .iter()
            .map(correlation_property_binding_intent)
            .collect(),
    }
}

fn correlation_property_binding_intent(
    binding: &BpmnCorrelationPropertyBindingSnapshot,
) -> BpmnCorrelationPropertyBindingIntent {
    BpmnCorrelationPropertyBindingIntent {
        binding_id: optional_arc(binding.binding_id.as_deref()),
        correlation_property_ref: optional_arc(binding.correlation_property_ref.as_deref()),
        data_path: optional_arc(binding.data_path.as_deref()),
        data_path_language: optional_arc(binding.data_path_language.as_deref()),
        data_path_evaluates_to_type_ref: optional_arc(
            binding.data_path_evaluates_to_type_ref.as_deref(),
        ),
    }
}

fn optional_arc(value: Option<&str>) -> Option<Arc<str>> {
    value.map(Arc::<str>::from)
}

fn arc_vec(values: &[String]) -> Vec<Arc<str>> {
    values
        .iter()
        .map(|value| Arc::<str>::from(value.as_str()))
        .collect()
}
