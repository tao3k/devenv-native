//! Local `LinkGraphIndex` adapter for the `WendaoGraph` evidence request contract.

mod projection;
mod types;

pub use projection::{
    build_wendao_graph_evidence_request_bundle,
    build_wendao_graph_evidence_request_bundle_with_options,
};
pub use types::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphEvidenceRequestBundle,
    WendaoGraphEvidenceRequestOptions, WendaoGraphEvidenceSeed,
};

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/wendao_graph_evidence/mod.rs"]
mod tests;
