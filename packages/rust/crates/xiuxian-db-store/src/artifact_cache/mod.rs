//! Artifact cache contracts for attachment and document extraction shards.

mod blob;
mod error;
mod filesystem;
#[cfg(feature = "foyer-artifact-cache")]
mod foyer_backend;
mod identity;

pub use blob::{ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactBlobWriteOutcome};
pub use error::ArtifactCacheError;
pub use filesystem::{
    ContentAddressedFilesystemBlobCache, ContentAddressedFilesystemBlobCacheConfig,
};
#[cfg(feature = "foyer-artifact-cache")]
pub use foyer_backend::{FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig};
pub use identity::{ArtifactKey, ArtifactKeyComponent, ArtifactKeyParts, ArtifactKind};
