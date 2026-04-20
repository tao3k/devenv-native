//! JSON codec helpers for checkpoint envelopes.

use crate::checkpoint::BpmnCheckpointEnvelope;
use crate::error::{BpmnEngineError, Result};

/// Encodes a checkpoint envelope into JSON.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointCodec`] when JSON serialization fails.
pub fn encode_checkpoint_json(checkpoint: &BpmnCheckpointEnvelope) -> Result<String> {
    serde_json::to_string(checkpoint)
        .map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))
}

/// Decodes a checkpoint envelope from JSON.
///
/// # Errors
///
/// Returns [`BpmnEngineError::CheckpointCodec`] when JSON deserialization fails.
pub fn decode_checkpoint_json(json: &str) -> Result<BpmnCheckpointEnvelope> {
    serde_json::from_str(json).map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))
}
