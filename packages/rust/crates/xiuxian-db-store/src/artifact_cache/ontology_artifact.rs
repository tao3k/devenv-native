//! Ontology artifact key helpers for review and read-model payload bytes.

use crate::artifact_cache::{ArtifactCacheError, ArtifactKey, ArtifactKeyParts, ArtifactKind};

const ONTOLOGY_ARTIFACT_NAMESPACE: &str = "ontology";

/// Named request used to build an ontology artifact key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyArtifactKeyParts {
    /// Ontology artifact kind.
    pub kind: ArtifactKind,
    /// Source contract, registry, or corpus digest component.
    pub source_digest: String,
    /// Ontology profile, compiler, or validation digest component.
    pub profile_digest: String,
    /// Run, packet, projection, or shard digest component.
    pub shard_digest: String,
}

/// Build a stable ontology artifact key.
///
/// # Errors
///
/// Returns [`ArtifactCacheError`] when any component is not safe for
/// content-addressed storage.
pub fn ontology_artifact_key(
    parts: OntologyArtifactKeyParts,
) -> Result<ArtifactKey, ArtifactCacheError> {
    ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: ONTOLOGY_ARTIFACT_NAMESPACE.to_owned(),
        kind: parts.kind,
        source_digest: parts.source_digest,
        profile_digest: parts.profile_digest,
        shard_digest: parts.shard_digest,
    })
}
