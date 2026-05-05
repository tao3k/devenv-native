//! Public checkpoint api contracts for BPMN/DMN engine integration.

use crate::error::Result;
use crate::runtime::BpmnInstanceState;

use crate::checkpoint::{
    decode_checkpoint_json_impl, encode_checkpoint_json_impl, lease_key_impl, state_key_impl,
};
#[cfg(feature = "valkey")]
use crate::checkpoint::{
    delete_checkpoint_as_owner_impl, delete_checkpoint_impl, load_checkpoint_impl,
    release_checkpoint_lease_impl, renew_checkpoint_lease_impl, save_checkpoint_as_owner_impl,
    save_checkpoint_impl, try_acquire_checkpoint_lease_impl,
};

/// Current scaffold checkpoint format version.
pub const BPMN_CHECKPOINT_FORMAT_VERSION: u32 = 1;

/// Versioned checkpoint envelope for Valkey persistence.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnCheckpointEnvelope {
    /// Checkpoint format version.
    pub version: u32,
    /// Monotonic checkpoint sequence.
    pub sequence: u64,
    /// Durable instance state payload.
    pub state: BpmnInstanceState,
}

/// Public workflow-instance identifier for checkpoint storage APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BpmnCheckpointInstanceId(String);

impl BpmnCheckpointInstanceId {
    /// Borrows the serialized workflow-instance identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for BpmnCheckpointInstanceId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<&String> for BpmnCheckpointInstanceId {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

impl From<String> for BpmnCheckpointInstanceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl BpmnCheckpointEnvelope {
    /// Creates a checkpoint envelope from one instance state.
    #[must_use]
    pub fn from_state(state: BpmnInstanceState) -> Self {
        Self {
            version: BPMN_CHECKPOINT_FORMAT_VERSION,
            sequence: state.sequence,
            state,
        }
    }
}

/// Encodes a checkpoint envelope into JSON.
///
/// # Errors
///
/// Returns a typed checkpoint codec error when JSON serialization fails.
pub fn encode_checkpoint_json(checkpoint: &BpmnCheckpointEnvelope) -> Result<String> {
    encode_checkpoint_json_impl(checkpoint)
}

/// Decodes a checkpoint envelope from JSON.
///
/// # Errors
///
/// Returns a typed checkpoint codec error when JSON deserialization fails.
pub fn decode_checkpoint_json(json: &str) -> Result<BpmnCheckpointEnvelope> {
    decode_checkpoint_json_impl(json)
}

/// Returns the durable state-key name for one workflow instance.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because Valkey key
/// callers already own serialized runtime identifiers.
#[must_use]
pub fn state_key(instance_id: impl Into<BpmnCheckpointInstanceId>) -> String {
    let instance_id = instance_id.into();
    state_key_impl(instance_id.as_str())
}

/// Returns the optional lease-key name for one workflow instance.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because Valkey key
/// callers already own serialized runtime identifiers.
#[must_use]
pub fn lease_key(instance_id: impl Into<BpmnCheckpointInstanceId>) -> String {
    let instance_id = instance_id.into();
    lease_key_impl(instance_id.as_str())
}

/// Loads a checkpoint envelope from Valkey.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because Valkey
/// checkpoint storage is keyed by the serialized runtime instance id.
///
/// # Errors
///
/// Returns a typed checkpoint storage or checkpoint codec error when Valkey
/// cannot be reached or the stored payload cannot be decoded.
#[cfg(feature = "valkey")]
pub async fn load_checkpoint(
    instance_id: impl Into<BpmnCheckpointInstanceId>,
    valkey_url: &str,
) -> Result<Option<BpmnCheckpointEnvelope>> {
    let instance_id = instance_id.into();
    load_checkpoint_impl(instance_id.as_str(), valkey_url).await
}

/// Saves a checkpoint envelope to Valkey.
///
/// # Errors
///
/// Returns a typed stale-write, checkpoint storage, or checkpoint codec error
/// when Valkey rejects or cannot persist the checkpoint payload.
#[cfg(feature = "valkey")]
pub async fn save_checkpoint(checkpoint: &BpmnCheckpointEnvelope, valkey_url: &str) -> Result<()> {
    save_checkpoint_impl(checkpoint, valkey_url).await
}

/// Deletes a checkpoint envelope from Valkey.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because Valkey
/// checkpoint storage is keyed by the serialized runtime instance id.
///
/// # Errors
///
/// Returns a typed checkpoint storage error when Valkey cannot delete the
/// checkpoint payload.
#[cfg(feature = "valkey")]
pub async fn delete_checkpoint(
    instance_id: impl Into<BpmnCheckpointInstanceId>,
    valkey_url: &str,
) -> Result<()> {
    let instance_id = instance_id.into();
    delete_checkpoint_impl(instance_id.as_str(), valkey_url).await
}

/// Deletes a checkpoint envelope from Valkey when the caller owns the lease.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because Valkey
/// checkpoint storage is keyed by the serialized runtime instance id.
///
/// # Errors
///
/// Returns a typed lease-ownership or checkpoint storage error when the
/// caller does not own the lease or Valkey cannot complete the delete.
#[cfg(feature = "valkey")]
pub async fn delete_checkpoint_as_owner(
    instance_id: impl Into<BpmnCheckpointInstanceId>,
    owner_token: &str,
    valkey_url: &str,
) -> Result<()> {
    let instance_id = instance_id.into();
    delete_checkpoint_as_owner_impl(instance_id.as_str(), owner_token, valkey_url).await
}

/// Saves a checkpoint envelope to Valkey when the caller owns the lease.
///
/// # Errors
///
/// Returns a typed lease-ownership, stale-write, checkpoint storage, or
/// checkpoint codec error when the guarded save cannot complete.
#[cfg(feature = "valkey")]
pub async fn save_checkpoint_as_owner(
    checkpoint: &BpmnCheckpointEnvelope,
    owner_token: &str,
    valkey_url: &str,
) -> Result<()> {
    save_checkpoint_as_owner_impl(checkpoint, owner_token, valkey_url).await
}

/// Tries to acquire the BPMN checkpoint lease for one workflow instance.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because lease keys use
/// the serialized runtime instance id.
///
/// # Errors
///
/// Returns a typed invalid-TTL or checkpoint storage error when the lease
/// request cannot be validated or persisted in Valkey.
#[cfg(feature = "valkey")]
pub async fn try_acquire_checkpoint_lease(
    instance_id: impl Into<BpmnCheckpointInstanceId>,
    owner_token: &str,
    lease_ttl_ms: u64,
    valkey_url: &str,
) -> Result<bool> {
    let instance_id = instance_id.into();
    try_acquire_checkpoint_lease_impl(instance_id.as_str(), owner_token, lease_ttl_ms, valkey_url)
        .await
}

/// Renews the BPMN checkpoint lease when the caller still owns it.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because lease keys use
/// the serialized runtime instance id.
///
/// # Errors
///
/// Returns a typed invalid-TTL or checkpoint storage error when the lease
/// renewal cannot be validated or persisted in Valkey.
#[cfg(feature = "valkey")]
pub async fn renew_checkpoint_lease(
    instance_id: impl Into<BpmnCheckpointInstanceId>,
    owner_token: &str,
    lease_ttl_ms: u64,
    valkey_url: &str,
) -> Result<bool> {
    let instance_id = instance_id.into();
    renew_checkpoint_lease_impl(instance_id.as_str(), owner_token, lease_ttl_ms, valkey_url).await
}

/// Releases the BPMN checkpoint lease when the caller still owns it.
///
/// # Identifier Boundary
///
/// The `instance_id` primitive is kept at this boundary because lease keys use
/// the serialized runtime instance id.
///
/// # Errors
///
/// Returns a typed checkpoint storage error when Valkey cannot complete the
/// lease release.
#[cfg(feature = "valkey")]
pub async fn release_checkpoint_lease(
    instance_id: impl Into<BpmnCheckpointInstanceId>,
    owner_token: &str,
    valkey_url: &str,
) -> Result<bool> {
    let instance_id = instance_id.into();
    release_checkpoint_lease_impl(instance_id.as_str(), owner_token, valkey_url).await
}
