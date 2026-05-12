//! `link_graph::index::search::quantum_fusion` owns Wendao index search quantum fusion behavior.

#[path = "anchor_batch.rs"]
mod anchor_batch;
/// Public Wendao boundary.
#[cfg(feature = "vector-store")]
#[path = "openai_ignition.rs"]
pub mod openai_ignition;
/// Public Wendao boundary.

#[path = "orchestrate/mod.rs"]
pub mod orchestrate;
#[path = "scored_context.rs"]
mod scored_context;
/// Public Wendao boundary.
#[path = "scoring.rs"]
pub mod scoring;
#[path = "semantic_anchor.rs"]
mod semantic_anchor;
/// Public Wendao boundary.
#[path = "semantic_ignition.rs"]
pub mod semantic_ignition;
#[path = "topology_expansion.rs"]
mod topology_expansion;
/// Public Wendao boundary.
#[cfg(feature = "vector-store")]
#[path = "vector_ignition.rs"]
pub mod vector_ignition;
