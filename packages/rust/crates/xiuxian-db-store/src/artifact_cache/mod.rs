//! Artifact cache contracts for attachment and document extraction shards.

mod agent_artifact;
mod attachment_artifact;
mod backend;
mod blob;
mod directory_bundle;
mod error;
mod filesystem;
#[cfg(feature = "foyer-artifact-cache")]
mod foyer_backend;
mod identity;
mod ontology_artifact;
mod readthrough;

pub use agent_artifact::{AgentArtifactKeyParts, agent_artifact_key};
pub use attachment_artifact::{AttachmentArtifactKeyParts, attachment_artifact_key};
pub use backend::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV, ARTIFACT_CACHE_FLUSHERS_ENV,
    ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
    ARTIFACT_CACHE_RECLAIMERS_ENV, ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_RUNTIME_WORKERS_ENV, ARTIFACT_CACHE_STORAGE_BYTES_ENV, ArtifactBlobCacheBackend,
    ArtifactBlobCacheBackendConfig, ArtifactCacheBackendKind,
};
pub use blob::{ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactBlobWriteOutcome};
pub use directory_bundle::{pack_artifact_directory, unpack_artifact_directory};
pub use error::ArtifactCacheError;
pub use filesystem::{
    ContentAddressedFilesystemBlobCache, ContentAddressedFilesystemBlobCacheConfig,
};
#[cfg(feature = "foyer-artifact-cache")]
pub use foyer_backend::{
    FOYER_ARTIFACT_BLOCK_SIZE_BYTES, FOYER_ARTIFACT_CACHE_POLICY, FOYER_ARTIFACT_MEMORY_WEIGHTER,
    FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig, FoyerArtifactBlobCacheEventStats,
};
pub use identity::{ArtifactKey, ArtifactKeyComponent, ArtifactKeyParts, ArtifactKind};
pub use ontology_artifact::{OntologyArtifactKeyParts, ontology_artifact_key};
pub use readthrough::{ArtifactReadThrough, read_through_artifact_bytes};
