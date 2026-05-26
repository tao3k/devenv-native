//! Backend-neutral `ArtifactBlobCache` read and write contracts.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::artifact_cache::{ArtifactCacheError, ArtifactKey};

/// Shared artifact bytes returned by cache backends.
///
/// The cache contract exposes borrowed byte slices to existing callers while
/// allowing memory-tier backends to return cheap shared clones on cache hits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBytes {
    bytes: Arc<[u8]>,
}

impl ArtifactBytes {
    /// Create shared artifact bytes from an owned buffer.
    #[must_use]
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Arc::from(bytes.into_boxed_slice()),
        }
    }

    /// Create shared artifact bytes from a borrowed slice.
    #[must_use]
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }

    /// Borrow the artifact bytes.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Number of artifact bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Return whether the artifact byte buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Return a copied owned buffer.
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.as_ref().to_vec()
    }

    /// Return whether two handles point at the same shared byte allocation.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.bytes, &other.bytes)
    }
}

impl From<Vec<u8>> for ArtifactBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_vec(bytes)
    }
}

impl From<&[u8]> for ArtifactBytes {
    fn from(bytes: &[u8]) -> Self {
        Self::from_slice(bytes)
    }
}

/// Bytes loaded from an artifact blob cache backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBlobRead {
    bytes: ArtifactBytes,
}

impl ArtifactBlobRead {
    /// Create a read result from owned bytes.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self::from_shared(ArtifactBytes::from_vec(bytes))
    }

    /// Create a read result from shared artifact bytes.
    #[must_use]
    pub fn from_shared(bytes: ArtifactBytes) -> Self {
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
        self.bytes.to_vec()
    }

    /// Consume the read result and return shared artifact bytes.
    #[must_use]
    pub fn into_shared_bytes(self) -> ArtifactBytes {
        self.bytes
    }

    /// Number of cached bytes.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

/// Backend read status for artifact bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactBlobReadStatus {
    /// The artifact bytes were found.
    Hit(ArtifactBlobRead),
    /// The backend did not contain bytes for the artifact key.
    Miss,
    /// The backend could not safely serve the read because it is under pressure.
    Throttled,
}

impl ArtifactBlobReadStatus {
    /// Return whether this status is a cache hit.
    #[must_use]
    pub const fn is_hit(&self) -> bool {
        matches!(self, Self::Hit(_))
    }

    /// Return whether this status is a normal cache miss.
    #[must_use]
    pub const fn is_miss(&self) -> bool {
        matches!(self, Self::Miss)
    }

    /// Return whether this status represents backend pressure.
    #[must_use]
    pub const fn is_throttled(&self) -> bool {
        matches!(self, Self::Throttled)
    }

    /// Consume this status and return cached bytes when present.
    #[must_use]
    pub fn into_read(self) -> Option<ArtifactBlobRead> {
        match self {
            Self::Hit(read) => Some(read),
            Self::Miss | Self::Throttled => None,
        }
    }
}

/// Builder used by backends that can execute read-through internally.
pub type ArtifactBlobFetchBuilder =
    Box<dyn FnOnce() -> Result<Vec<u8>, ArtifactCacheError> + Send + 'static>;

/// Backend fetch-through status for artifact bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactBlobFetchStatus {
    /// Bytes came from the cache backend.
    Hit,
    /// Bytes were generated after a normal cache miss.
    Miss,
    /// Bytes were generated while the cache backend reported pressure.
    Throttled,
}

impl ArtifactBlobFetchStatus {
    /// Return whether this status came from existing cached bytes.
    #[must_use]
    pub const fn is_hit(self) -> bool {
        matches!(self, Self::Hit)
    }

    /// Return whether this status represents backend pressure.
    #[must_use]
    pub const fn is_throttled(self) -> bool {
        matches!(self, Self::Throttled)
    }
}

/// Result of backend-managed artifact fetch-through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBlobFetch {
    bytes: ArtifactBytes,
    status: ArtifactBlobFetchStatus,
    write: Option<ArtifactBlobWriteOutcome>,
    read_elapsed: Duration,
    build_elapsed: Duration,
    write_elapsed: Duration,
}

/// Named fields used to construct a backend-managed artifact fetch result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBlobFetchParts {
    bytes: ArtifactBytes,
    status: ArtifactBlobFetchStatus,
    write: Option<ArtifactBlobWriteOutcome>,
    read_elapsed: Duration,
    build_elapsed: Duration,
    write_elapsed: Duration,
}

impl ArtifactBlobFetchParts {
    /// Create fetch result fields from owned artifact bytes.
    #[must_use]
    pub fn from_owned_bytes(bytes: Vec<u8>, status: ArtifactBlobFetchStatus) -> Self {
        Self::from_shared_bytes(ArtifactBytes::from_vec(bytes), status)
    }

    /// Create fetch result fields from shared artifact bytes.
    #[must_use]
    pub fn from_shared_bytes(bytes: ArtifactBytes, status: ArtifactBlobFetchStatus) -> Self {
        Self {
            bytes,
            status,
            write: None,
            read_elapsed: Duration::ZERO,
            build_elapsed: Duration::ZERO,
            write_elapsed: Duration::ZERO,
        }
    }

    /// Record the cache write outcome for generated bytes.
    #[must_use]
    pub fn with_write(mut self, write: Option<ArtifactBlobWriteOutcome>) -> Self {
        self.write = write;
        self
    }

    /// Record elapsed time spent reading the cache backend.
    #[must_use]
    pub fn with_read_elapsed(mut self, read_elapsed: Duration) -> Self {
        self.read_elapsed = read_elapsed;
        self
    }

    /// Record elapsed time spent building bytes after a miss.
    #[must_use]
    pub fn with_build_elapsed(mut self, build_elapsed: Duration) -> Self {
        self.build_elapsed = build_elapsed;
        self
    }

    /// Record elapsed time spent writing generated bytes.
    #[must_use]
    pub fn with_write_elapsed(mut self, write_elapsed: Duration) -> Self {
        self.write_elapsed = write_elapsed;
        self
    }
}

impl ArtifactBlobFetch {
    /// Create a fetch-through result from named fields.
    #[must_use]
    pub fn from_parts(parts: ArtifactBlobFetchParts) -> Self {
        Self {
            bytes: parts.bytes,
            status: parts.status,
            write: parts.write,
            read_elapsed: parts.read_elapsed,
            build_elapsed: parts.build_elapsed,
            write_elapsed: parts.write_elapsed,
        }
    }

    /// Borrow fetched artifact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Consume this result and return fetched artifact bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    /// Consume this result and return shared artifact bytes.
    #[must_use]
    pub fn into_shared_bytes(self) -> ArtifactBytes {
        self.bytes
    }

    /// Number of artifact bytes returned.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// Fetch-through status.
    #[must_use]
    pub const fn status(&self) -> ArtifactBlobFetchStatus {
        self.status
    }

    /// Write outcome when the backend populated persistent cache bytes.
    #[must_use]
    pub const fn write_outcome(&self) -> Option<ArtifactBlobWriteOutcome> {
        self.write
    }

    /// Elapsed time spent reading from the cache backend.
    #[must_use]
    pub const fn read_elapsed(&self) -> Duration {
        self.read_elapsed
    }

    /// Elapsed time spent building artifact bytes.
    #[must_use]
    pub const fn build_elapsed(&self) -> Duration {
        self.build_elapsed
    }

    /// Elapsed time spent writing artifact bytes.
    #[must_use]
    pub const fn write_elapsed(&self) -> Duration {
        self.write_elapsed
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
    /// Return the stable backend name used in read-through receipts.
    #[must_use]
    fn backend_name(&self) -> &'static str {
        "artifact-cache"
    }

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

    /// Read cached bytes and preserve backend pressure information.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend cannot read bytes for
    /// the key.
    fn read_with_status(
        &self,
        key: &ArtifactKey,
    ) -> Result<ArtifactBlobReadStatus, ArtifactCacheError> {
        Ok(self
            .read(key)?
            .map_or(ArtifactBlobReadStatus::Miss, ArtifactBlobReadStatus::Hit))
    }

    /// Fetch artifact bytes through the backend, allowing specialized backends
    /// to coalesce same-key concurrent misses.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the cache read/write fails or when
    /// the builder fails to produce artifact bytes.
    fn fetch_through(
        &self,
        key: &ArtifactKey,
        build: ArtifactBlobFetchBuilder,
    ) -> Result<ArtifactBlobFetch, ArtifactCacheError> {
        let read_started = Instant::now();
        let read_status = self.read_with_status(key)?;
        let read_elapsed = read_started.elapsed();
        if let ArtifactBlobReadStatus::Hit(read) = read_status {
            return Ok(ArtifactBlobFetch::from_parts(
                ArtifactBlobFetchParts::from_shared_bytes(
                    read.into_shared_bytes(),
                    ArtifactBlobFetchStatus::Hit,
                )
                .with_read_elapsed(read_elapsed),
            ));
        }

        let status = if read_status.is_throttled() {
            ArtifactBlobFetchStatus::Throttled
        } else {
            ArtifactBlobFetchStatus::Miss
        };

        let build_started = Instant::now();
        let bytes = build()?;
        let build_elapsed = build_started.elapsed();

        let write_started = Instant::now();
        let write = if status.is_throttled() {
            None
        } else {
            Some(self.write(key, ArtifactBlobWrite::new(bytes.as_slice()))?)
        };
        let write_elapsed = write_started.elapsed();

        Ok(ArtifactBlobFetch::from_parts(
            ArtifactBlobFetchParts::from_owned_bytes(bytes, status)
                .with_write(write)
                .with_read_elapsed(read_elapsed)
                .with_build_elapsed(build_elapsed)
                .with_write_elapsed(write_elapsed),
        ))
    }

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
