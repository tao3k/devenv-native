#[derive(Debug, thiserror::Error)]
pub(crate) enum KnowledgeSectionSearchError {
    #[error("knowledge section index has no published epoch")]
    NotReady,
    #[error(transparent)]
    Storage(#[from] xiuxian_vector_store::VectorStoreError),
    #[error("{0}")]
    Decode(String),
}
