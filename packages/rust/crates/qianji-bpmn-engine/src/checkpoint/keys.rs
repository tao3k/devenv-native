//! Valkey key helpers for checkpoint storage.

/// Returns the durable state-key name for one workflow instance.
#[must_use]
pub fn state_key(instance_id: &str) -> String {
    format!("xq:bpmn:ckpt:{instance_id}:state")
}

/// Returns the optional lease-key name for one workflow instance.
#[must_use]
pub fn lease_key(instance_id: &str) -> String {
    format!("xq:bpmn:ckpt:{instance_id}:lease")
}
