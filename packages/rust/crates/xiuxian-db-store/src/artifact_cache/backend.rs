//! Backend selection for artifact blob cache implementations.

use std::path::{Path, PathBuf};

use crate::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactBlobWriteOutcome,
    ArtifactCacheError, ArtifactKey, ContentAddressedFilesystemBlobCache,
};

#[cfg(feature = "foyer-artifact-cache")]
use crate::artifact_cache::{FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig};

/// Environment variable that selects the artifact cache backend.
pub const ARTIFACT_CACHE_BACKEND_ENV: &str = "WENDAO_ARTIFACT_CACHE_BACKEND";
/// Environment variable that selects the artifact cache root.
pub const ARTIFACT_CACHE_ROOT_ENV: &str = "WENDAO_ARTIFACT_CACHE_ROOT";
/// Environment variable that sets the Foyer in-memory tier capacity in bytes.
pub const ARTIFACT_CACHE_MEMORY_BYTES_ENV: &str = "WENDAO_ARTIFACT_CACHE_MEMORY_BYTES";
/// Environment variable that sets the Foyer disk tier capacity in bytes.
pub const ARTIFACT_CACHE_STORAGE_BYTES_ENV: &str = "WENDAO_ARTIFACT_CACHE_STORAGE_BYTES";

const DEFAULT_BACKEND: ArtifactCacheBackendKind = ArtifactCacheBackendKind::Filesystem;
const DEFAULT_MEMORY_CAPACITY_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_STORAGE_CAPACITY_BYTES: usize = 512 * 1024 * 1024;

/// Artifact cache backend kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCacheBackendKind {
    /// Content-addressed filesystem baseline.
    Filesystem,
    /// Foyer hybrid memory and disk cache backend.
    Foyer,
}

impl ArtifactCacheBackendKind {
    /// Parse a backend kind from an environment value.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the value is not a supported backend.
    pub fn parse(value: &str) -> Result<Self, ArtifactCacheError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "filesystem" => Ok(Self::Filesystem),
            "foyer" => Ok(Self::Foyer),
            _ => Err(ArtifactCacheError::invalid_component(
                "artifact_cache_backend",
                value,
                "backend must be `filesystem` or `foyer`",
            )),
        }
    }

    /// Return the stable backend name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::Foyer => "foyer",
        }
    }
}

/// Resolved artifact cache backend configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBlobCacheBackendConfig {
    kind: ArtifactCacheBackendKind,
    root: PathBuf,
    memory_capacity_bytes: usize,
    storage_capacity_bytes: usize,
}

impl ArtifactBlobCacheBackendConfig {
    /// Build a config from an explicit root and process environment lookup.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when backend or capacity values are invalid.
    pub fn from_root_and_env(root: impl Into<PathBuf>) -> Result<Self, ArtifactCacheError> {
        Self::from_root_and_lookup(root, &|key| std::env::var(key).ok())
    }

    /// Build a config from an explicit root and lookup callback.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when backend or capacity values are invalid.
    pub fn from_root_and_lookup(
        root: impl Into<PathBuf>,
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, ArtifactCacheError> {
        Ok(Self {
            kind: backend_kind_value(lookup)?,
            root: root.into(),
            memory_capacity_bytes: usize_value(
                lookup,
                ARTIFACT_CACHE_MEMORY_BYTES_ENV,
                DEFAULT_MEMORY_CAPACITY_BYTES,
            )?,
            storage_capacity_bytes: usize_value(
                lookup,
                ARTIFACT_CACHE_STORAGE_BYTES_ENV,
                DEFAULT_STORAGE_CAPACITY_BYTES,
            )?,
        })
    }

    /// Build a config from process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when no root can be resolved or when a
    /// value is invalid.
    pub fn from_env() -> Result<Self, ArtifactCacheError> {
        Self::from_lookup(&|key| std::env::var(key).ok())
    }

    /// Build a config from a lookup callback.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when no root can be resolved or when a
    /// value is invalid.
    pub fn from_lookup(
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, ArtifactCacheError> {
        let root = artifact_cache_root_value(lookup)?;
        Self::from_root_and_lookup(root, lookup)
    }

    /// Selected backend kind.
    #[must_use]
    pub const fn kind(&self) -> ArtifactCacheBackendKind {
        self.kind
    }

    /// Artifact cache root.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    /// In-memory tier capacity in bytes.
    #[must_use]
    pub const fn memory_capacity_bytes(&self) -> usize {
        self.memory_capacity_bytes
    }

    /// Disk tier capacity in bytes.
    #[must_use]
    pub const fn storage_capacity_bytes(&self) -> usize {
        self.storage_capacity_bytes
    }

    /// Build the configured backend.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend cannot be created.
    pub fn build(&self) -> Result<ArtifactBlobCacheBackend, ArtifactCacheError> {
        match self.kind {
            ArtifactCacheBackendKind::Filesystem => Ok(ArtifactBlobCacheBackend::Filesystem(
                ContentAddressedFilesystemBlobCache::new(self.root.clone()),
            )),
            ArtifactCacheBackendKind::Foyer => build_foyer_backend(self),
        }
    }
}

/// Concrete artifact cache backend selected by configuration.
pub enum ArtifactBlobCacheBackend {
    /// Content-addressed filesystem baseline.
    Filesystem(ContentAddressedFilesystemBlobCache),
    /// Foyer hybrid memory and disk cache backend.
    #[cfg(feature = "foyer-artifact-cache")]
    Foyer(FoyerArtifactBlobCache),
}

impl ArtifactBlobCacheBackend {
    /// Return the stable backend name.
    #[must_use]
    pub const fn backend_name(&self) -> &'static str {
        match self {
            Self::Filesystem(_) => "filesystem",
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(_) => "foyer",
        }
    }

    /// Close the backend if it has an explicit lifecycle.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the backend close operation fails.
    #[cfg_attr(
        not(feature = "foyer-artifact-cache"),
        expect(
            clippy::unnecessary_wraps,
            reason = "close has a fallible Foyer lifecycle when the foyer-artifact-cache feature is enabled"
        )
    )]
    pub fn close(&self) -> Result<(), ArtifactCacheError> {
        match self {
            Self::Filesystem(_) => Ok(()),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.close(),
        }
    }
}

impl ArtifactBlobCache for ArtifactBlobCacheBackend {
    fn contains(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        match self {
            Self::Filesystem(cache) => cache.contains(key),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.contains(key),
        }
    }

    fn read(&self, key: &ArtifactKey) -> Result<Option<ArtifactBlobRead>, ArtifactCacheError> {
        match self {
            Self::Filesystem(cache) => cache.read(key),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.read(key),
        }
    }

    fn write(
        &self,
        key: &ArtifactKey,
        value: ArtifactBlobWrite<'_>,
    ) -> Result<ArtifactBlobWriteOutcome, ArtifactCacheError> {
        match self {
            Self::Filesystem(cache) => cache.write(key, value),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.write(key, value),
        }
    }

    fn remove(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        match self {
            Self::Filesystem(cache) => cache.remove(key),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.remove(key),
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
fn build_foyer_backend(
    config: &ArtifactBlobCacheBackendConfig,
) -> Result<ArtifactBlobCacheBackend, ArtifactCacheError> {
    Ok(ArtifactBlobCacheBackend::Foyer(
        FoyerArtifactBlobCache::from_config(FoyerArtifactBlobCacheConfig::new(
            config.root().to_path_buf(),
            config.memory_capacity_bytes(),
            config.storage_capacity_bytes(),
        ))?,
    ))
}

#[cfg(not(feature = "foyer-artifact-cache"))]
fn build_foyer_backend(
    _config: &ArtifactBlobCacheBackendConfig,
) -> Result<ArtifactBlobCacheBackend, ArtifactCacheError> {
    Err(ArtifactCacheError::backend(
        "foyer",
        "building backend",
        "xiuxian-db-store/foyer-artifact-cache feature is not enabled",
    ))
}

fn backend_kind_value(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ArtifactCacheBackendKind, ArtifactCacheError> {
    lookup(ARTIFACT_CACHE_BACKEND_ENV)
        .as_deref()
        .map_or(Ok(DEFAULT_BACKEND), ArtifactCacheBackendKind::parse)
}

fn artifact_cache_root_value(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<PathBuf, ArtifactCacheError> {
    if let Some(root) = lookup(ARTIFACT_CACHE_ROOT_ENV)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
    {
        return Ok(PathBuf::from(root));
    }
    lookup("PRJ_CACHE_HOME")
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|root| root.join("wendao").join("artifacts"))
        .ok_or_else(|| {
            ArtifactCacheError::backend(
                "artifact-cache",
                "resolving root",
                "WENDAO_ARTIFACT_CACHE_ROOT or PRJ_CACHE_HOME must be set",
            )
        })
}

fn usize_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: usize,
) -> Result<usize, ArtifactCacheError> {
    let Some(value) = lookup(key).map(|value| value.trim().to_owned()) else {
        return Ok(default);
    };
    if value.is_empty() {
        return Ok(default);
    }
    let parsed = value.parse::<usize>().map_err(|_| {
        ArtifactCacheError::invalid_component(
            key,
            value.clone(),
            "value must be a positive integer",
        )
    })?;
    if parsed == 0 {
        return Err(ArtifactCacheError::invalid_component(
            key,
            value,
            "value must be greater than zero",
        ));
    }
    Ok(parsed)
}
