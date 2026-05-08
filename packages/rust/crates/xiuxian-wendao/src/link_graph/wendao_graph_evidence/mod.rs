//! Local `LinkGraphIndex` adapter for the `WendaoGraph` evidence request contract.

mod page_index;
mod projection;
mod semantic_reasoning;
mod types;

pub use page_index::{
    build_wendao_graph_page_index_reasoning_request_bundle,
    build_wendao_graph_page_index_reasoning_request_bundle_with_options,
};
pub use projection::{
    build_wendao_graph_evidence_request_bundle,
    build_wendao_graph_evidence_request_bundle_with_options,
};
pub use semantic_reasoning::{
    build_semantic_scope_page_index_reasoning_request_bundle,
    build_semantic_scope_page_index_reasoning_request_bundle_with_options,
    semantic_scope_page_index_reasoning_default_options,
};
pub use types::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphEvidenceRequestBundle,
    WendaoGraphEvidenceRequestOptions, WendaoGraphEvidenceSeed,
    WendaoGraphPageIndexReasoningRequestBundle, WendaoGraphPageIndexReasoningRequestOptions,
    WendaoGraphPageIndexReasoningSeed, WendaoGraphSemanticOverlayEdge,
};

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/wendao_graph_evidence/mod.rs"]
mod tests;
