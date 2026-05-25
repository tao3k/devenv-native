use super::{Value, json};

pub(in crate::lint::bpmn::document_surface) fn routing_boundary_evidence() -> Value {
    json!({
        "status": "metadata_only",
        "execution_policy": "deferred",
        "runtime_scope": "single_process_graph",
        "preserved_metadata": [
            "participants",
            "partner_catalogs",
            "endpoints",
            "messages",
            "message_flows",
            "conversations",
            "choreography",
            "correlation_properties",
            "correlation_retrieval_expressions"
        ],
        "deferred_semantics": [
            "participant_dispatch",
            "endpoint_invocation",
            "message_flow_routing",
            "conversation_routing",
            "choreography_execution",
            "correlation_matching",
            "correlation_subscription_matching",
            "correlation_key_evaluation",
            "retrieval_expression_evaluation"
        ],
        "supported_execution_models": [
            "supported_process_graph",
            "host_dispatched_task",
            "supported_event_wait"
        ],
        "repair_guidance": "Model executable behavior through supported process flow, host-dispatched work, or supported waits."
    })
}
