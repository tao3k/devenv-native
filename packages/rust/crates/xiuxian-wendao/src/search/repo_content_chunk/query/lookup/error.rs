use xiuxian_vector_store::VectorStoreError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum RepoContentChunkSearchError {
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
    #[error("{0}")]
    Decode(String),
}
