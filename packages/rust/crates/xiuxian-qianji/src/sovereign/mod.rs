//! Sovereign Memory Module (Blueprint V6.1).
//!
//! This module implements the "问道归元" (Wendao Guiyuan) architecture for
//! giving agents true sovereign memory - persistent reasoning traces that
//! connect Intent → Reasoning → Outcome.
//! Start in `contract_feedback_sink` for the public persistence seam.
//!
//! ## Architecture
//!
//! ```text
//! Qianji Execution Loop
//!        │
//!        ▼ ZhenfaStreamingEvent
//! ThoughtAggregator.process_event()
//!        │
//!        ▼ CognitiveTraceRecord
//! ArtifactObserver.ingest_artifact()
//!        │
//!        ▼ WendaoIngestionSink (FileWendaoSink)
//!        │
//!        ▼ Markdown file in .cognitive/traces/
//!        │
//!        ▼ Wendao LinkGraphIndex (on next rebuild)
//! ```
//!
//! ## Historical Sovereignty
//!
//! This enables querying the knowledge graph for the reasoning chain that
//! led to any commit or decision: "Query Wendao for the reasoning chain
//! that led to Commit-X".

#[cfg(test)]
#[path = "../sovereign_artifact_observer.rs"]
mod artifact_observer;
#[path = "../sovereign_contract_feedback_sink.rs"]
mod contract_feedback_sink;
#[cfg(test)]
#[path = "../sovereign_thought_aggregator.rs"]
mod thought_aggregator;
#[cfg(test)]
#[path = "../sovereign_wendao_adapter.rs"]
mod wendao_adapter;
#[cfg(test)]
#[path = "../sovereign_wendao_sink.rs"]
mod wendao_sink;

pub use self::contract_feedback_sink::{
    ContractFeedbackKnowledgeSink, InMemoryContractFeedbackSink,
    KnowledgeStorageContractFeedbackSink,
};
