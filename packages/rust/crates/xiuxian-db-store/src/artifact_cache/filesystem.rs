//! Content-addressed filesystem baseline for artifact blob bytes.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactBlobWriteOutcome,
    ArtifactCacheError, ArtifactKey,
};

const ARTIFACT_FILE_NAME: &str = "payload.bin";

/// Configuration for the content-addressed filesystem artifact cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAddressedFilesystemBlobCacheConfig {
    root: PathBuf,
}

impl ContentAddressedFilesystemBlobCacheConfig {
    /// Create a filesystem cache configuration.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Root directory for cached artifact bytes.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }
}

/// Content-addressed filesystem implementation of [`ArtifactBlobCache`].
#[derive(Debug, Clone)]
pub struct ContentAddressedFilesystemBlobCache {
    config: ContentAddressedFilesystemBlobCacheConfig,
}

impl ContentAddressedFilesystemBlobCache {
    /// Create a filesystem blob cache from a root directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            config: ContentAddressedFilesystemBlobCacheConfig::new(root),
        }
    }

    /// Create a filesystem blob cache from explicit configuration.
    #[must_use]
    pub const fn with_config(config: ContentAddressedFilesystemBlobCacheConfig) -> Self {
        Self { config }
    }

    /// Root directory for cached artifact bytes.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.config.root()
    }

    /// Resolve the storage path for an artifact key.
    #[must_use]
    pub fn artifact_path(&self, key: &ArtifactKey) -> PathBuf {
        self.root()
            .join(key.namespace().as_str())
            .join(key.kind().as_storage_component())
            .join(key.source_digest().as_str())
            .join(key.profile_digest().as_str())
            .join(key.shard_digest().as_str())
            .join(ARTIFACT_FILE_NAME)
    }
}

impl ArtifactBlobCache for ContentAddressedFilesystemBlobCache {
    fn contains(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        Ok(self.artifact_path(key).is_file())
    }

    fn read(&self, key: &ArtifactKey) -> Result<Option<ArtifactBlobRead>, ArtifactCacheError> {
        let path = self.artifact_path(key);
        match fs::read(path.as_path()) {
            Ok(bytes) => Ok(Some(ArtifactBlobRead::new(bytes))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(ArtifactCacheError::io("reading", path, error)),
        }
    }

    fn write(
        &self,
        key: &ArtifactKey,
        value: ArtifactBlobWrite<'_>,
    ) -> Result<ArtifactBlobWriteOutcome, ArtifactCacheError> {
        let path = self.artifact_path(key);
        let parent = path.parent().ok_or_else(|| {
            ArtifactCacheError::invalid_component(
                "artifact_path",
                path.to_string_lossy(),
                "artifact path has no parent",
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| ArtifactCacheError::io("creating parent directory", parent, error))?;

        let replaced = path.exists();
        let temp_path = temporary_payload_path(parent);
        if temp_path.exists() {
            fs::remove_file(temp_path.as_path()).map_err(|error| {
                ArtifactCacheError::io("removing stale temp file", &temp_path, error)
            })?;
        }
        fs::write(temp_path.as_path(), value.bytes())
            .map_err(|error| ArtifactCacheError::io("writing temp file", &temp_path, error))?;
        if replaced {
            fs::remove_file(path.as_path()).map_err(|error| {
                ArtifactCacheError::io("removing previous payload", &path, error)
            })?;
        }
        fs::rename(temp_path.as_path(), path.as_path()).map_err(|error| {
            ArtifactCacheError::io("promoting temp file", path.as_path(), error)
        })?;
        Ok(ArtifactBlobWriteOutcome::new(value.byte_len(), replaced))
    }

    fn remove(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        let path = self.artifact_path(key);
        match fs::remove_file(path.as_path()) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(ArtifactCacheError::io("removing payload", path, error)),
        }
    }
}

fn temporary_payload_path(parent: &Path) -> PathBuf {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!(
        ".payload.{}.{}.tmp",
        std::process::id(),
        timestamp_nanos
    ))
}
