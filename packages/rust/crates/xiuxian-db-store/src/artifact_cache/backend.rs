//! Backend selection for artifact blob cache implementations.

use std::path::{Path, PathBuf};

use crate::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobFetch, ArtifactBlobFetchBuilder, ArtifactBlobRead,
    ArtifactBlobReadStatus, ArtifactBlobWrite, ArtifactBlobWriteOutcome, ArtifactCacheError,
    ArtifactKey, ContentAddressedFilesystemBlobCache,
};

#[cfg(feature = "foyer-artifact-cache")]
use crate::artifact_cache::{
    FOYER_ARTIFACT_BLOCK_SIZE_BYTES, FOYER_ARTIFACT_CACHE_POLICY, FOYER_ARTIFACT_MEMORY_WEIGHTER,
    FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig,
};

/// Environment variable that selects the artifact cache backend.
pub const ARTIFACT_CACHE_BACKEND_ENV: &str = "WENDAO_ARTIFACT_CACHE_BACKEND";
/// Environment variable that selects the artifact cache root.
pub const ARTIFACT_CACHE_ROOT_ENV: &str = "WENDAO_ARTIFACT_CACHE_ROOT";
/// Environment variable that sets the Foyer in-memory tier capacity in bytes.
pub const ARTIFACT_CACHE_MEMORY_BYTES_ENV: &str = "WENDAO_ARTIFACT_CACHE_MEMORY_BYTES";
/// Environment variable that sets the Foyer disk tier capacity in bytes.
pub const ARTIFACT_CACHE_STORAGE_BYTES_ENV: &str = "WENDAO_ARTIFACT_CACHE_STORAGE_BYTES";
/// Environment variable that sets Foyer runtime worker threads.
pub const ARTIFACT_CACHE_RUNTIME_WORKERS_ENV: &str = "WENDAO_ARTIFACT_CACHE_RUNTIME_WORKERS";
/// Environment variable that sets Foyer in-memory shard count.
pub const ARTIFACT_CACHE_MEMORY_SHARDS_ENV: &str = "WENDAO_ARTIFACT_CACHE_MEMORY_SHARDS";
/// Environment variable that sets Foyer block-engine block size in bytes.
pub const ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV: &str = "WENDAO_ARTIFACT_CACHE_BLOCK_SIZE_BYTES";
/// Environment variable that sets Foyer disk recovery concurrency.
pub const ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV: &str =
    "WENDAO_ARTIFACT_CACHE_RECOVER_CONCURRENCY";
/// Environment variable that sets Foyer disk flusher count.
pub const ARTIFACT_CACHE_FLUSHERS_ENV: &str = "WENDAO_ARTIFACT_CACHE_FLUSHERS";
/// Environment variable that sets Foyer disk reclaimer count.
pub const ARTIFACT_CACHE_RECLAIMERS_ENV: &str = "WENDAO_ARTIFACT_CACHE_RECLAIMERS";

#[cfg(feature = "foyer-artifact-cache")]
const DEFAULT_BACKEND: ArtifactCacheBackendKind = ArtifactCacheBackendKind::Foyer;
#[cfg(not(feature = "foyer-artifact-cache"))]
const DEFAULT_BACKEND: ArtifactCacheBackendKind = ArtifactCacheBackendKind::Filesystem;
const DEFAULT_MEMORY_CAPACITY_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_STORAGE_CAPACITY_BYTES: usize = 512 * 1024 * 1024;
const MIN_FOYER_BLOCK_SIZE_BYTES: usize = 4 * 1024;

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
            "" => Ok(DEFAULT_BACKEND),
            "filesystem" => Ok(Self::Filesystem),
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
    runtime_worker_threads: usize,
    memory_shards: usize,
    block_size_bytes: usize,
    recover_concurrency: usize,
    flushers: usize,
    reclaimers: usize,
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
        let memory_capacity_bytes = usize_value(
            lookup,
            ARTIFACT_CACHE_MEMORY_BYTES_ENV,
            DEFAULT_MEMORY_CAPACITY_BYTES,
        )?;
        let storage_capacity_bytes = usize_value(
            lookup,
            ARTIFACT_CACHE_STORAGE_BYTES_ENV,
            DEFAULT_STORAGE_CAPACITY_BYTES,
        )?;
        let block_size_bytes = normalized_block_size_bytes(usize_value(
            lookup,
            ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV,
            foyer_block_size_bytes(),
        )?);
        let memory_shards = effective_memory_shards(
            adaptive_usize_value(
                lookup,
                ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
                default_memory_shards,
                "memory shards must be `auto` or a positive integer",
            )?,
            memory_capacity_bytes,
        );
        let recover_concurrency = effective_recover_concurrency(
            adaptive_usize_value(
                lookup,
                ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV,
                default_recover_concurrency,
                "recover concurrency must be `auto` or a positive integer",
            )?,
            storage_capacity_bytes,
            block_size_bytes,
        );
        let flushers = effective_io_lanes(
            adaptive_usize_value(
                lookup,
                ARTIFACT_CACHE_FLUSHERS_ENV,
                default_io_lanes,
                "flushers must be `auto` or a positive integer",
            )?,
            storage_capacity_bytes,
            block_size_bytes,
        );
        let reclaimers = effective_io_lanes(
            adaptive_usize_value(
                lookup,
                ARTIFACT_CACHE_RECLAIMERS_ENV,
                default_io_lanes,
                "reclaimers must be `auto` or a positive integer",
            )?,
            storage_capacity_bytes,
            block_size_bytes,
        );
        Ok(Self {
            kind: backend_kind_value(lookup)?,
            root: root.into(),
            memory_capacity_bytes,
            storage_capacity_bytes,
            runtime_worker_threads: runtime_worker_threads_value(lookup)?,
            memory_shards,
            block_size_bytes,
            recover_concurrency,
            flushers,
            reclaimers,
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

    /// Foyer runtime worker threads used by disk cache operations.
    #[must_use]
    pub const fn runtime_worker_threads(&self) -> usize {
        self.runtime_worker_threads
    }

    /// Foyer memory shard count selected for the backend.
    #[must_use]
    pub const fn memory_shards(&self) -> usize {
        self.memory_shards
    }

    /// Foyer block size selected for the backend.
    #[must_use]
    pub const fn block_size_bytes(&self) -> usize {
        self.block_size_bytes
    }

    /// Foyer disk recover concurrency selected for the backend.
    #[must_use]
    pub const fn recover_concurrency(&self) -> usize {
        self.recover_concurrency
    }

    /// Foyer disk flusher count selected for the backend.
    #[must_use]
    pub const fn flushers(&self) -> usize {
        self.flushers
    }

    /// Foyer disk reclaimer count selected for the backend.
    #[must_use]
    pub const fn reclaimers(&self) -> usize {
        self.reclaimers
    }

    /// Stable in-memory weight function for Foyer artifact bytes.
    #[must_use]
    pub const fn foyer_memory_weighter_name(&self) -> Option<&'static str> {
        match self.kind {
            ArtifactCacheBackendKind::Filesystem => None,
            ArtifactCacheBackendKind::Foyer => Some(foyer_memory_weighter_name()),
        }
    }

    /// Stable Foyer hybrid cache policy name.
    #[must_use]
    pub const fn foyer_cache_policy_name(&self) -> Option<&'static str> {
        match self.kind {
            ArtifactCacheBackendKind::Filesystem => None,
            ArtifactCacheBackendKind::Foyer => Some(foyer_cache_policy_name()),
        }
    }

    /// Foyer block size when the selected backend is Foyer.
    #[must_use]
    pub const fn foyer_block_size_bytes(&self) -> Option<usize> {
        match self.kind {
            ArtifactCacheBackendKind::Filesystem => None,
            ArtifactCacheBackendKind::Foyer => Some(self.block_size_bytes),
        }
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
    fn backend_name(&self) -> &'static str {
        self.backend_name()
    }

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

    fn read_with_status(
        &self,
        key: &ArtifactKey,
    ) -> Result<ArtifactBlobReadStatus, ArtifactCacheError> {
        match self {
            Self::Filesystem(cache) => cache.read_with_status(key),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.read_with_status(key),
        }
    }

    fn fetch_through(
        &self,
        key: &ArtifactKey,
        build: ArtifactBlobFetchBuilder,
    ) -> Result<ArtifactBlobFetch, ArtifactCacheError> {
        match self {
            Self::Filesystem(cache) => cache.fetch_through(key, build),
            #[cfg(feature = "foyer-artifact-cache")]
            Self::Foyer(cache) => cache.fetch_through(key, build),
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
        FoyerArtifactBlobCache::from_config(
            FoyerArtifactBlobCacheConfig::new_with_runtime_workers(
                config.root().to_path_buf(),
                config.memory_capacity_bytes(),
                config.storage_capacity_bytes(),
                config.runtime_worker_threads(),
            )
            .with_memory_shards(config.memory_shards())
            .with_block_size_bytes(config.block_size_bytes())
            .with_recover_concurrency(config.recover_concurrency())
            .with_flushers(config.flushers())
            .with_reclaimers(config.reclaimers()),
        )?,
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

fn runtime_worker_threads_value(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<usize, ArtifactCacheError> {
    adaptive_usize_value(
        lookup,
        ARTIFACT_CACHE_RUNTIME_WORKERS_ENV,
        default_runtime_worker_threads,
        "runtime workers must be `auto` or a positive integer",
    )
}

fn adaptive_usize_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: fn() -> usize,
    expectation: &'static str,
) -> Result<usize, ArtifactCacheError> {
    let Some(value) = lookup(key).map(|value| value.trim().to_owned()) else {
        return Ok(default());
    };
    if value.is_empty() || value.eq_ignore_ascii_case("auto") {
        return Ok(default());
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| ArtifactCacheError::invalid_component(key, &value, expectation))?;
    if parsed == 0 {
        return Err(ArtifactCacheError::invalid_component(
            key,
            &value,
            "value must be greater than zero",
        ));
    }
    Ok(parsed)
}

fn default_runtime_worker_threads() -> usize {
    system_parallelism()
}

fn default_memory_shards() -> usize {
    system_parallelism()
}

fn default_recover_concurrency() -> usize {
    system_parallelism()
}

fn default_io_lanes() -> usize {
    system_parallelism().div_ceil(4).max(1)
}

fn system_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
}

fn normalized_block_size_bytes(block_size_bytes: usize) -> usize {
    block_size_bytes.max(MIN_FOYER_BLOCK_SIZE_BYTES)
}

fn effective_memory_shards(memory_shards: usize, memory_capacity_bytes: usize) -> usize {
    memory_shards.max(1).min(memory_capacity_bytes.max(1))
}

fn effective_recover_concurrency(
    recover_concurrency: usize,
    storage_capacity_bytes: usize,
    block_size_bytes: usize,
) -> usize {
    recover_concurrency.max(1).min(storage_block_count(
        storage_capacity_bytes,
        block_size_bytes,
    ))
}

fn effective_io_lanes(
    io_lanes: usize,
    storage_capacity_bytes: usize,
    block_size_bytes: usize,
) -> usize {
    io_lanes.max(1).min(
        storage_block_count(storage_capacity_bytes, block_size_bytes)
            .saturating_sub(1)
            .max(1),
    )
}

fn storage_block_count(storage_capacity_bytes: usize, block_size_bytes: usize) -> usize {
    storage_capacity_bytes
        .checked_div(normalized_block_size_bytes(block_size_bytes))
        .unwrap_or(0)
        .max(1)
}

const fn foyer_memory_weighter_name() -> &'static str {
    #[cfg(feature = "foyer-artifact-cache")]
    {
        FOYER_ARTIFACT_MEMORY_WEIGHTER
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        "bytes"
    }
}

const fn foyer_cache_policy_name() -> &'static str {
    #[cfg(feature = "foyer-artifact-cache")]
    {
        FOYER_ARTIFACT_CACHE_POLICY
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        "write-on-insertion"
    }
}

const fn foyer_block_size_bytes() -> usize {
    #[cfg(feature = "foyer-artifact-cache")]
    {
        FOYER_ARTIFACT_BLOCK_SIZE_BYTES
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        16 * 1024 * 1024
    }
}
