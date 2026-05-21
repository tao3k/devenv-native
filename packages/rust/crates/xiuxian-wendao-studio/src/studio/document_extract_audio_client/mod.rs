//! Owns the Studio document extract audio shard Flight client surface.

mod client;
mod recovery;
mod workflow;

pub use client::{AudioShardFlightClient, AudioShardFlightResponse, AudioShardWorkflowExecution};
pub use recovery::{AudioShardRecoveryPlanRequest, AudioShardRecoveryPlanning};
pub use workflow::{AudioShardRecoveryWorkflowExecution, AudioShardRecoveryWorkflowRequest};

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/document_extract_audio_client/mod.rs"]
pub(crate) mod tests;
