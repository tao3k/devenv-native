//! `link_graph::models::attachments` owns Wendao link graph models attachments behavior.

use serde::{Deserialize, Serialize};

/// Vision annotation payload from multimodal analysis.
///
/// Contains OCR-extracted text, LLM-generated descriptions,
/// and recognized entities from image analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisionAnnotation {
    /// OCR/LLM-generated text description of image content.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Confidence score (0.0-1.0) from multimodal analysis.
    #[serde(default)]
    pub confidence: f64,
    /// Extracted text entities (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<String>,
    /// Timestamp of annotation creation (Unix seconds since epoch).
    #[serde(default)]
    pub annotated_at: i64,
}

impl VisionAnnotation {
    /// Create a new vision annotation with description.
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            confidence: 0.8,
            entities: Vec::new(),
            annotated_at: 0,
        }
    }

    /// Create empty annotation.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Attachment kind classification inferred from file extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphAttachmentKind {
    /// Image attachments (`png`, `jpg`, `svg`, ...).
    Image,
    /// PDF attachments.
    Pdf,
    /// GPG/PGP/key material attachments.
    Gpg,
    /// Generic document attachments (`docx`, `txt`, `epub`, ...).
    Document,
    /// Compressed/archive attachments.
    Archive,
    /// Audio attachments.
    Audio,
    /// Video attachments.
    Video,
    /// Unclassified attachment type.
    Other,
}

impl LinkGraphAttachmentKind {
    /// Parse kind alias from CLI/runtime input.
    #[must_use]
    pub fn from_alias(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "image" | "img" => Self::Image,
            "pdf" => Self::Pdf,
            "gpg" | "pgp" | "key" => Self::Gpg,
            "doc" | "document" => Self::Document,
            "archive" | "compressed" => Self::Archive,
            "audio" | "sound" => Self::Audio,
            "video" => Self::Video,
            _ => Self::Other,
        }
    }

    /// Infer attachment kind from extension (without leading dot).
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        let normalized = ext.trim().trim_start_matches('.').to_lowercase();
        match normalized.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "avif" | "tif"
            | "tiff" | "heic" | "heif" => Self::Image,
            "pdf" => Self::Pdf,
            "gpg" | "pgp" | "asc" | "sig" | "key" => Self::Gpg,
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => Self::Archive,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" => Self::Audio,
            "mp4" | "mkv" | "mov" | "avi" | "webm" | "m4v" => Self::Video,
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt" | "ods"
            | "odp" | "epub" => Self::Document,
            _ => Self::Other,
        }
    }
}

/// One normalized attachment reference extracted from a markdown note.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct LinkGraphAttachment {
    /// Source document id (relative path without extension).
    pub source_id: String,
    /// Source note stem.
    pub source_stem: String,
    /// Source note path with extension.
    pub source_path: String,
    /// Source note title.
    pub source_title: String,
    /// Normalized attachment path/target.
    pub attachment_path: String,
    /// Attachment basename.
    pub attachment_name: String,
    /// Lowercased extension without leading dot.
    pub attachment_ext: String,
    /// Attachment kind inferred from extension.
    pub kind: LinkGraphAttachmentKind,
    /// Vision annotation (optional, injected by `VisionIngress`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_annotation: Option<VisionAnnotation>,
}

/// Attachment search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct LinkGraphAttachmentHit {
    /// Source document id (relative path without extension).
    pub source_id: String,
    /// Source note stem.
    pub source_stem: String,
    /// Source note title.
    pub source_title: String,
    /// Source note path with extension.
    pub source_path: String,
    /// Normalized attachment path/target.
    pub attachment_path: String,
    /// Attachment basename.
    pub attachment_name: String,
    /// Lowercased extension without leading dot.
    pub attachment_ext: String,
    /// Attachment kind inferred from extension.
    pub kind: LinkGraphAttachmentKind,
    /// Search relevance score (0-1).
    pub score: f64,
    /// Vision annotation snippet for search display (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_snippet: Option<String>,
}
