//! JSON codec helpers for checkpoint envelopes.

use crate::checkpoint_api::BpmnCheckpointEnvelope;
use crate::error::{BpmnEngineError, Result};

/// Encodes a checkpoint envelope into JSON.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointCodec`] when JSON serialization fails.
pub(in crate::checkpoint) fn encode_checkpoint_json_impl(
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<String> {
    serde_json::to_string(checkpoint)
        .map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))
}

/// Decodes a checkpoint envelope from JSON.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointCodec`] when JSON deserialization fails.
pub(in crate::checkpoint) fn decode_checkpoint_json_impl(
    json: &str,
) -> Result<BpmnCheckpointEnvelope> {
    serde_json::from_str(json).map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))
}
