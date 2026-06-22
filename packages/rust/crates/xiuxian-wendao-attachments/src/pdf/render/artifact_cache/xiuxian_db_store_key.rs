//! Feature-gated `ArtifactKey` type bridge for PDF render artifact cache.

#[cfg(feature = "foyer-artifact-cache")]
pub(super) use xiuxian_db_store::artifact_cache::ArtifactKey;

#[cfg(not(feature = "foyer-artifact-cache"))]
pub(super) struct ArtifactKey;
