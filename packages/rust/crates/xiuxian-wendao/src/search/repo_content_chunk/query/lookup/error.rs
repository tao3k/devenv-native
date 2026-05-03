use xiuxian_db_store::VectorStoreError;

#[derive(Debug, thiserror::Error)]
pub enum RepoContentChunkSearchError {
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
    #[error("{0}")]
    Decode(String),
}
