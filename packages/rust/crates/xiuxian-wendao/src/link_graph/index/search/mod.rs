//! `link_graph::index::search` owns Wendao link graph index search behavior.
/// Public Wendao boundary.

#[path = "context.rs"]
pub mod context;
#[path = "emit.rs"]
mod emit;
#[path = "graph_state_filters.rs"]
mod graph_state_filters;
#[path = "path_tag_filters.rs"]
mod path_tag_filters;
/// Public Wendao boundary.
#[path = "pipeline/mod.rs"]
pub mod pipeline;
/// Public Wendao boundary.
#[path = "plan/mod.rs"]
pub mod plan;
/// Public Wendao boundary.
#[cfg(feature = "vector-store")]
#[path = "quantum_fusion/mod.rs"]
pub mod quantum_fusion;
#[path = "row_evaluator/mod.rs"]
mod row_evaluator;
#[path = "score/mod.rs"]
mod score;
#[path = "semantic_gate.rs"]
mod semantic_gate;
#[path = "strategy.rs"]
mod strategy;
#[path = "structured_filters/mod.rs"]
mod structured_filters;
#[path = "traversal_candidates/mod.rs"]
mod traversal_candidates;

pub(super) use super::{ScoredSearchRow, deterministic_random_key, sort_hits};
pub use crate::link_graph::{LinkGraphHit, LinkGraphIndex, LinkGraphSearchOptions};
