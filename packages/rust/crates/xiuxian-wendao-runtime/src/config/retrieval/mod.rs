//! Retrieval runtime configuration records for Wendao search behavior.

mod base;
mod semantic_ignition;

pub use base::{
    LinkGraphRetrievalBaseRuntimeConfig, resolve_link_graph_retrieval_base_runtime_with_settings,
};
pub use semantic_ignition::{
    LinkGraphSemanticIgnitionBackend, LinkGraphSemanticIgnitionRuntimeConfig,
    apply_semantic_ignition_runtime_config,
};
