//! `search::repo_content_chunk::query::lookup::error` owns Wendao query lookup error behavior.

use xiuxian_db_store::VectorStoreError;
/// `RepoContentChunkSearchError` public enum boundary for Wendao.

#[derive(Debug, thiserror::Error)]
pub enum RepoContentChunkSearchError {
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
    #[error("{0}")]
    Decode(String),
}
