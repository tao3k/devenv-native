use serde::{Deserialize, Serialize};
use specta::Type;

use super::StudioNavigationTarget;

/// A hit representing an attachment or external resource.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSearchHit {
    /// Attachment filename.
    pub name: String,
    /// Relative path.
    pub path: String,
    /// Stable source document identifier.
    pub source_id: String,
    /// Source document stem.
    pub source_stem: String,
    /// Source document title.
    pub source_title: String,
    /// Source document path.
    pub source_path: String,
    /// Stable attachment identifier.
    pub attachment_id: String,
    /// Relative attachment path.
    pub attachment_path: String,
    /// Attachment display name.
    pub attachment_name: String,
    /// Lowercased attachment extension without leading dot.
    pub attachment_ext: String,
    /// Attachment kind label.
    pub kind: String,
    /// Navigation target.
    pub navigation_target: StudioNavigationTarget,
    /// Relevance score.
    pub score: f64,
    /// Optional OCR or vision snippet for the attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_snippet: Option<String>,
}

/// Response for Studio attachment search queries.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSearchResponse {
    /// Original query string.
    pub query: String,
    /// Matching attachment hits.
    pub hits: Vec<AttachmentSearchHit>,
    /// Total number of hits returned.
    pub hit_count: usize,
    /// Selected attachment scope label.
    pub selected_scope: String,
    /// Whether the response is partial because the attachment index is still warming.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
    /// Current attachment-index lifecycle state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing_state: Option<String>,
    /// Optional attachment-index error surfaced without blocking the request path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_error: Option<String>,
}
