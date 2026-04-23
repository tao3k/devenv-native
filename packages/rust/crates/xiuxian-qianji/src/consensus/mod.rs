//! Distributed consensus module.
//!
//! `mod.rs` intentionally contains declarations and re-exports only.

#[path = "../consensus_manager.rs"]
mod manager;
#[path = "../consensus_models.rs"]
mod models;
mod thresholds;

pub use manager::ConsensusManager;
pub use models::{AgentIdentity, AgentVote, ConsensusMode, ConsensusPolicy, ConsensusResult};
