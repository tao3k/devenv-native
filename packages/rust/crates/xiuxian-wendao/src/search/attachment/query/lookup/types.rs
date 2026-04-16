use std::collections::HashSet;

use crate::search::ranking::{RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry};
use xiuxian_vector_store::VectorStoreError;

pub(crate) const MIN_RETAINED_ATTACHMENTS: usize = 32;
pub(crate) const RETAINED_ATTACHMENT_MULTIPLIER: usize = 2;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AttachmentSearchError {
    #[error("attachment index has no published epoch")]
    NotReady,
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
    #[error("{0}")]
    Decode(String),
}

#[derive(Debug, Clone)]
pub(crate) struct AttachmentCandidate {
    pub(crate) id: String,
    pub(crate) score: f64,
    pub(crate) source_path: String,
    pub(crate) attachment_path: String,
}

pub(crate) struct AttachmentCandidateQuery<'a> {
    pub(crate) case_sensitive: bool,
    pub(crate) normalized_query: &'a str,
    pub(crate) query_tokens: &'a [String],
    pub(crate) extensions: &'a HashSet<String>,
    pub(crate) kinds: &'a HashSet<String>,
    pub(crate) window: RetainedWindow,
}

pub(crate) struct AttachmentSearchExecution {
    pub(crate) candidates: Vec<AttachmentCandidate>,
    pub(crate) telemetry: StreamingRerankTelemetry,
    pub(crate) source: StreamingRerankSource,
}
