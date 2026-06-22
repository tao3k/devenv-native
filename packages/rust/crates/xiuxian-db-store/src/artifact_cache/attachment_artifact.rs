//! Attachment artifact key helpers for materialized media/document bytes.

use crate::artifact_cache::{ArtifactCacheError, ArtifactKey, ArtifactKeyParts, ArtifactKind};

const ATTACHMENT_ARTIFACT_NAMESPACE: &str = "attachment";

/// Named request used to build an attachment artifact key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentArtifactKeyParts {
    /// Attachment artifact kind.
    pub kind: ArtifactKind,
    /// Source content digest component.
    pub source_digest: String,
    /// Profile, parser, or planner digest component.
    pub profile_digest: String,
    /// Shard, page, region, or derived payload digest component.
    pub shard_digest: String,
}

/// Build a stable attachment artifact key.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when any component is not safe for
/// content-addressed storage.
pub fn attachment_artifact_key(
    parts: AttachmentArtifactKeyParts,
) -> Result<ArtifactKey, ArtifactCacheError> {
    ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: ATTACHMENT_ARTIFACT_NAMESPACE.to_owned(),
        kind: parts.kind,
        source_digest: parts.source_digest,
        profile_digest: parts.profile_digest,
        shard_digest: parts.shard_digest,
    })
}
