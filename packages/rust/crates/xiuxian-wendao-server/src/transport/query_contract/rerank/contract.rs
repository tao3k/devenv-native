//! Rerank route metadata and schema constant contract.

/// Canonical rerank-embedding dimension metadata header for Wendao Flight exchange requests.
pub const WENDAO_RERANK_DIMENSION_HEADER: &str = "x-wendao-rerank-embedding-dimension";
/// Canonical rerank top-k metadata header for Wendao Flight exchange requests.
#[cfg(feature = "transport")]
pub const WENDAO_RERANK_TOP_K_HEADER: &str = "x-wendao-rerank-top-k";
/// Canonical rerank minimum-final-score metadata header for Wendao Flight exchange requests.
#[cfg(feature = "transport")]
pub const WENDAO_RERANK_MIN_FINAL_SCORE_HEADER: &str = "x-wendao-rerank-min-final-score";
/// Stable route for the rerank contract.
pub const RERANK_ROUTE: &str = "/rerank";
