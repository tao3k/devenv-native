//! Backend-neutral `ArtifactBlobCache` read and write contracts.

use crate::artifact_cache::{ArtifactCacheError, ArtifactKey};

/// Bytes loaded from an artifact blob cache backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBlobRead {
    bytes: Vec<u8>,
}

impl ArtifactBlobRead {
    /// Create a read result from owned bytes.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Borrow the cached bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Consume the read result and return owned bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Number of cached bytes.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

/// Borrowed bytes to write into an artifact blob cache backend.
#[derive(Debug, Clone, Copy)]
pub struct ArtifactBlobWrite<'a> {
    bytes: &'a [u8],
}

impl<'a> ArtifactBlobWrite<'a> {
    /// Create a write request from borrowed bytes.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Borrow the bytes to be written.
    #[must_use]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Number of bytes to be written.
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

/// Result of writing artifact bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactBlobWriteOutcome {
    byte_len: usize,
    replaced: bool,
}

impl ArtifactBlobWriteOutcome {
    /// Create a write outcome.
    #[must_use]
    pub const fn new(byte_len: usize, replaced: bool) -> Self {
        Self { byte_len, replaced }
    }

    /// Number of bytes persisted by the write.
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.byte_len
    }

    /// Whether the write replaced existing cached bytes for the same key.
    #[must_use]
    pub const fn replaced(&self) -> bool {
        self.replaced
    }
}

/// Backend-neutral contract for large attachment and document extraction
/// artifact bytes.
pub trait ArtifactBlobCache {
    /// Return whether the cache currently has bytes for the artifact key.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend cannot inspect the key.
    fn contains(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError>;

    /// Read cached bytes for the artifact key.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend cannot read bytes for
    /// the key.
    fn read(&self, key: &ArtifactKey) -> Result<Option<ArtifactBlobRead>, ArtifactCacheError>;

    /// Write cached bytes for the artifact key.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend cannot persist bytes for
    /// the key.
    fn write(
        &self,
        key: &ArtifactKey,
        value: ArtifactBlobWrite<'_>,
    ) -> Result<ArtifactBlobWriteOutcome, ArtifactCacheError>;

    /// Remove cached bytes for the artifact key.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend cannot remove bytes for
    /// the key.
    fn remove(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError>;
}
