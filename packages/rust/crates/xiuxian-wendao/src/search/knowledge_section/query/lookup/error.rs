#[derive(Debug, thiserror::Error)]
/// Errors returned while querying the knowledge-section search index.
pub enum KnowledgeSectionSearchError {
    /// The knowledge-section index has not published a readable epoch yet.
    #[error("knowledge section index has no published epoch")]
    NotReady,
    /// The vector-store or query-engine layer failed.
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
    /// Stored knowledge-section rows could not be decoded into search hits.
    #[error("{0}")]
    Decode(String),
}
