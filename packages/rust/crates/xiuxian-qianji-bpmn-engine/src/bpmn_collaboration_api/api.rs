//! Public bpmn collaboration api contracts for BPMN/DMN engine integration.

use std::sync::Arc;

/// Package-owned host envelope for BPMN collaboration metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCollaborationHostEnvelope {
    /// Source document that contributed this envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<Arc<str>>,
    /// Explicit execution boundary for collaboration metadata.
    pub boundary: BpmnCollaborationHostBoundary,
    /// Collaboration shells preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collaborations: Vec<BpmnCollaborationIntent>,
    /// Participant routing intent flattened across collaborations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participants: Vec<BpmnParticipantIntent>,
    /// Message-flow routing intent flattened across collaborations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub message_flows: Vec<BpmnMessageFlowIntent>,
    /// Correlation property catalog entries preserved from definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub correlation_properties: Vec<BpmnCorrelationPropertyIntent>,
    /// Collaboration, conversation, and choreography correlation keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub correlation_keys: Vec<BpmnCorrelationKeyIntent>,
    /// Process-level correlation subscriptions preserved as intent only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub process_correlation_subscriptions: Vec<BpmnProcessCorrelationSubscriptionIntent>,
}

impl BpmnCollaborationHostEnvelope {
    /// Returns true when no collaboration host metadata has been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.collaborations.is_empty()
            && self.participants.is_empty()
            && self.message_flows.is_empty()
            && self.correlation_properties.is_empty()
            && self.correlation_keys.is_empty()
            && self.process_correlation_subscriptions.is_empty()
    }
}

/// Explicit metadata-only collaboration execution boundary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCollaborationHostBoundary {
    /// Collaboration execution policy exposed to hosts.
    pub execution_policy: BpmnCollaborationExecutionPolicy,
    /// Runtime scope that remains executable in this slice.
    pub runtime_scope: BpmnCollaborationRuntimeScope,
    /// Host event de-duplication policy, distinct from BPMN correlation.
    pub event_deduplication_policy: BpmnEventDeduplicationPolicy,
    /// Deferred BPMN semantics not executed by this envelope.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deferred_semantics: Vec<Arc<str>>,
}

impl Default for BpmnCollaborationHostBoundary {
    fn default() -> Self {
        Self {
            execution_policy: BpmnCollaborationExecutionPolicy::MetadataOnly,
            runtime_scope: BpmnCollaborationRuntimeScope::SingleProcessGraph,
            event_deduplication_policy: BpmnEventDeduplicationPolicy::ExplicitEventReferenceOnly,
            deferred_semantics: vec![
                Arc::<str>::from("participant_dispatch"),
                Arc::<str>::from("endpoint_invocation"),
                Arc::<str>::from("message_flow_routing"),
                Arc::<str>::from("conversation_routing"),
                Arc::<str>::from("choreography_execution"),
                Arc::<str>::from("correlation_matching"),
                Arc::<str>::from("correlation_subscription_matching"),
                Arc::<str>::from("correlation_key_evaluation"),
                Arc::<str>::from("data_path_evaluation"),
            ],
        }
    }
}

/// Execution policy for collaboration metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnCollaborationExecutionPolicy {
    /// Collaboration is preserved for host intent but is not executed.
    MetadataOnly,
}

/// Runtime scope that remains executable while collaboration is metadata-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnCollaborationRuntimeScope {
    /// Existing runtime executes one process graph at a time.
    SingleProcessGraph,
}

/// Host event de-duplication policy exposed alongside collaboration intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnEventDeduplicationPolicy {
    /// Host de-duplication may use explicit event references only.
    ExplicitEventReferenceOnly,
}

/// One BPMN collaboration shell preserved for host inspection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCollaborationIntent {
    /// Optional stable collaboration identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collaboration_id: Option<Arc<str>>,
    /// Optional human-readable collaboration name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Optional BPMN closed-collaboration marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    /// Optional initiating participant reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiating_participant_ref: Option<Arc<str>>,
}

/// One BPMN participant exposed as host routing/display intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnParticipantIntent {
    /// Optional owning collaboration identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collaboration_id: Option<Arc<str>>,
    /// Optional stable participant identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<Arc<str>>,
    /// Optional human-readable participant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Optional referenced process identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_ref: Option<Arc<str>>,
    /// Direct interface references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interface_refs: Vec<Arc<str>>,
    /// Direct endpoint references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub end_point_refs: Vec<Arc<str>>,
    /// Optional participant multiplicity metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_multiplicity: Option<BpmnParticipantMultiplicityIntent>,
}

/// One participant multiplicity declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnParticipantMultiplicityIntent {
    /// Optional stable multiplicity identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplicity_id: Option<Arc<str>>,
    /// Optional minimum expression payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<Arc<str>>,
    /// Optional maximum expression payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<Arc<str>>,
}

/// One BPMN message-flow intent exposed to hosts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnMessageFlowIntent {
    /// Optional owning collaboration identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collaboration_id: Option<Arc<str>>,
    /// Optional stable message-flow identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_flow_id: Option<Arc<str>>,
    /// Optional human-readable message-flow name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Optional BPMN source reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<Arc<str>>,
    /// Optional BPMN target reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<Arc<str>>,
    /// Optional BPMN message reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ref: Option<Arc<str>>,
}

/// One BPMN correlation-property catalog entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationPropertyIntent {
    /// Optional stable correlation-property identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_property_id: Option<Arc<str>>,
    /// Optional human-readable property name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Optional BPMN type reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_ref: Option<Arc<str>>,
    /// Retrieval expressions preserved as metadata only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieval_expressions: Vec<BpmnCorrelationPropertyRetrievalIntent>,
}

/// One BPMN correlation-property retrieval expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationPropertyRetrievalIntent {
    /// Optional stable retrieval-expression identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_expression_id: Option<Arc<str>>,
    /// Optional referenced message identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ref: Option<Arc<str>>,
    /// Optional nested message-path expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_path: Option<Arc<str>>,
}

/// Scope where a BPMN correlation key was declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnCorrelationKeyScope {
    /// Key declared directly under a collaboration.
    Collaboration,
    /// Key declared under a conversation node.
    Conversation,
    /// Key declared under a choreography activity.
    Choreography,
}

/// One BPMN correlation-key declaration preserved as intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationKeyIntent {
    /// Scope of the key declaration.
    pub scope: BpmnCorrelationKeyScope,
    /// Optional owning scope identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<Arc<str>>,
    /// Optional stable correlation-key identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_key_id: Option<Arc<str>>,
    /// Optional human-readable key name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
    /// Direct correlation-property references preserved in source order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub correlation_property_refs: Vec<Arc<str>>,
}

/// One process-level correlation subscription preserved as intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnProcessCorrelationSubscriptionIntent {
    /// Process that declares the subscription.
    pub process_id: Arc<str>,
    /// Optional stable subscription identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<Arc<str>>,
    /// Optional referenced BPMN correlation key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_key_ref: Option<Arc<str>>,
    /// Direct correlation-property bindings preserved as metadata only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BpmnCorrelationPropertyBindingIntent>,
}

/// One process correlation-property binding preserved as intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCorrelationPropertyBindingIntent {
    /// Optional stable binding identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<Arc<str>>,
    /// Optional referenced correlation property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_property_ref: Option<Arc<str>>,
    /// Optional nested data-path payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<Arc<str>>,
    /// Optional data-path expression language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path_language: Option<Arc<str>>,
    /// Optional data-path result type reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path_evaluates_to_type_ref: Option<Arc<str>>,
}
