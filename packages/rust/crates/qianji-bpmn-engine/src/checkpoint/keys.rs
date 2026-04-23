//! Valkey key helpers for checkpoint storage.

/// Returns the durable state-key name for one workflow instance.
#[must_use]
pub(in crate::checkpoint) fn state_key_impl(instance_id: &str) -> String {
    format!("xq:bpmn:ckpt:{instance_id}:state")
}

/// Returns the optional lease-key name for one workflow instance.
#[must_use]
pub(in crate::checkpoint) fn lease_key_impl(instance_id: &str) -> String {
    format!("xq:bpmn:ckpt:{instance_id}:lease")
}
