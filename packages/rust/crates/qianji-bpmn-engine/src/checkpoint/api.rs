use crate::checkpoint_api::BpmnCheckpointEnvelope;
use crate::error::Result;

pub(crate) fn encode_checkpoint_json_impl(checkpoint: &BpmnCheckpointEnvelope) -> Result<String> {
    super::codec::encode_checkpoint_json_impl(checkpoint)
}

pub(crate) fn decode_checkpoint_json_impl(json: &str) -> Result<BpmnCheckpointEnvelope> {
    super::codec::decode_checkpoint_json_impl(json)
}

pub(crate) fn state_key_impl(instance_id: &str) -> String {
    super::keys::state_key_impl(instance_id)
}

pub(crate) fn lease_key_impl(instance_id: &str) -> String {
    super::keys::lease_key_impl(instance_id)
}

#[cfg(feature = "valkey")]
pub(crate) async fn load_checkpoint_impl(
    instance_id: &str,
    valkey_url: &str,
) -> Result<Option<BpmnCheckpointEnvelope>> {
    super::valkey::load_checkpoint_impl(instance_id, valkey_url).await
}

#[cfg(feature = "valkey")]
pub(crate) async fn save_checkpoint_impl(
    checkpoint: &BpmnCheckpointEnvelope,
    valkey_url: &str,
) -> Result<()> {
    super::valkey::save_checkpoint_impl(checkpoint, valkey_url).await
}

#[cfg(feature = "valkey")]
pub(crate) async fn delete_checkpoint_impl(instance_id: &str, valkey_url: &str) -> Result<()> {
    super::valkey::delete_checkpoint_impl(instance_id, valkey_url).await
}

#[cfg(feature = "valkey")]
pub(crate) async fn delete_checkpoint_as_owner_impl(
    instance_id: &str,
    owner_token: &str,
    valkey_url: &str,
) -> Result<()> {
    super::valkey::delete_checkpoint_as_owner_impl(instance_id, owner_token, valkey_url).await
}

#[cfg(feature = "valkey")]
pub(crate) async fn save_checkpoint_as_owner_impl(
    checkpoint: &BpmnCheckpointEnvelope,
    owner_token: &str,
    valkey_url: &str,
) -> Result<()> {
    super::valkey::save_checkpoint_as_owner_impl(checkpoint, owner_token, valkey_url).await
}

#[cfg(feature = "valkey")]
pub(crate) async fn try_acquire_checkpoint_lease_impl(
    instance_id: &str,
    owner_token: &str,
    lease_ttl_ms: u64,
    valkey_url: &str,
) -> Result<bool> {
    super::lease::try_acquire_checkpoint_lease_impl(
        instance_id,
        owner_token,
        lease_ttl_ms,
        valkey_url,
    )
    .await
}

#[cfg(feature = "valkey")]
pub(crate) async fn renew_checkpoint_lease_impl(
    instance_id: &str,
    owner_token: &str,
    lease_ttl_ms: u64,
    valkey_url: &str,
) -> Result<bool> {
    super::lease::renew_checkpoint_lease_impl(instance_id, owner_token, lease_ttl_ms, valkey_url)
        .await
}

#[cfg(feature = "valkey")]
pub(crate) async fn release_checkpoint_lease_impl(
    instance_id: &str,
    owner_token: &str,
    valkey_url: &str,
) -> Result<bool> {
    super::lease::release_checkpoint_lease_impl(instance_id, owner_token, valkey_url).await
}
