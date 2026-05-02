//! Rerank route contract, schema, and scoring boundary for Wendao Flight.

mod contract;
mod request;
mod response;
mod scoring;
mod types;

pub use contract::{RERANK_ROUTE, WENDAO_RERANK_DIMENSION_HEADER};
#[cfg(feature = "transport")]
pub use contract::{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER, WENDAO_RERANK_TOP_K_HEADER};
#[cfg(feature = "transport")]
pub use request::{
    RERANK_REQUEST_DOC_ID_COLUMN, RERANK_REQUEST_EMBEDDING_COLUMN,
    RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, RERANK_REQUEST_VECTOR_SCORE_COLUMN,
    validate_rerank_request_batch, validate_rerank_request_schema,
};
#[cfg(feature = "transport")]
pub use response::{
    RERANK_RESPONSE_DOC_ID_COLUMN, RERANK_RESPONSE_FINAL_SCORE_COLUMN, RERANK_RESPONSE_RANK_COLUMN,
    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
    validate_rerank_response_batch, validate_rerank_response_schema,
};
#[cfg(feature = "transport")]
pub use scoring::{score_rerank_request_batch, score_rerank_request_batch_with_weights};
pub use types::RerankScoreWeights;
#[cfg(feature = "transport")]
pub use types::RerankScoredCandidate;
