//! Plugin Arrow exchange boundary for negotiated scoring roundtrips.

mod errors;
mod metadata;
#[cfg(feature = "vector-store")]
mod prepare;
mod request;
mod response;
mod roundtrip;
#[cfg(test)]
#[path = "../../../tests/unit/transport/plugin_arrow_exchange/mod.rs"]
mod tests;

pub use metadata::{attach_plugin_arrow_request_metadata, plugin_arrow_request_trace_id};
#[cfg(feature = "vector-store")]
pub use prepare::{
    PluginArrowVectorStoreRequestBuildError, build_plugin_arrow_request_batch_from_vector_store,
    build_plugin_arrow_request_batch_from_vector_store_with_metadata,
    prepare_plugin_arrow_request_rows_from_vector_store,
};
pub use request::{
    PluginArrowCandidateProjection, PluginArrowRequestBatchBuildError, PluginArrowRequestRow,
    PluginArrowScoredCandidate, build_plugin_arrow_request_batch,
    build_plugin_arrow_request_batch_from_embeddings,
    build_plugin_arrow_request_batch_from_embeddings_with_metadata,
    project_plugin_arrow_scored_candidates,
};
pub use response::{
    PluginArrowScoreRow, decode_plugin_arrow_score_rows, validate_plugin_arrow_response_batches,
};
pub use roundtrip::{
    NegotiatedPluginArrowScoreRows, PluginArrowScoreRoundtripError,
    roundtrip_plugin_arrow_score_rows_with_binding,
};
