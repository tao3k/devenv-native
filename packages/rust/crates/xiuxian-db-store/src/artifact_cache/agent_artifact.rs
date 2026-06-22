//! Agent artifact key helpers for evidence and prompt context bytes.

use crate::artifact_cache::{ArtifactCacheError, ArtifactKey, ArtifactKeyParts, ArtifactKind};

const AGENT_ARTIFACT_NAMESPACE: &str = "agent";

/// Named request used to build an agent artifact key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentArtifactKeyParts {
    /// Agent artifact kind.
    pub kind: ArtifactKind,
    /// Source content digest component.
    pub source_digest: String,
    /// Profile or prompt-building digest component.
    pub profile_digest: String,
    /// Shard, query, or context-pack digest component.
    pub shard_digest: String,
}

/// Build a stable agent artifact key.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when any component is not safe for
/// content-addressed storage.
pub fn agent_artifact_key(parts: AgentArtifactKeyParts) -> Result<ArtifactKey, ArtifactCacheError> {
    ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: AGENT_ARTIFACT_NAMESPACE.to_owned(),
        kind: parts.kind,
        source_digest: parts.source_digest,
        profile_digest: parts.profile_digest,
        shard_digest: parts.shard_digest,
    })
}
