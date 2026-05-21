//! `search::contracts::attachments` owns Wendao search contracts attachments behavior.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::StudioNavigationTarget;

/// A hit representing an attachment or external resource.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
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
