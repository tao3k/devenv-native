use super::{Value, json};

pub(in crate::lint::bpmn::document_surface) fn correlation_boundary_evidence() -> Value {
    json!({
        "status": "metadata_only",
        "execution_policy": "deferred",
        "runtime_scope": "explicit_event_reference_waits",
        "preserved_metadata": [
            "process_correlation_subscriptions",
            "correlation_property_bindings",
            "correlation_keys",
            "correlation_properties",
            "binding_data_paths"
        ],
        "bounded_executable_surface": [
            "message_event_reference_wait",
            "signal_event_reference_wait",
            "timer_wait",
            "conditional_wait"
        ],
        "deferred_semantics": [
            "correlation_subscription_matching",
            "correlation_key_evaluation",
            "correlation_property_binding_evaluation",
            "binding_data_path_evaluation",
            "collaboration_message_correlation"
        ],
        "repair_guidance": "Use explicit supported event waits or host-dispatched work for executable behavior; preserve BPMN correlation subscriptions as metadata until runtime matching is implemented."
    })
}
