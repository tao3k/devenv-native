//! Foyer-backed `ArtifactBlobCache` implementation.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use foyer::{
    BlockEngineConfig, DeviceBuilder, FsDeviceBuilder, HybridCache, HybridCacheBuilder,
    HybridCachePolicy, Load,
};
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime};

use crate::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactBlobWriteOutcome,
    ArtifactCacheError, ArtifactKey,
};

const FOYER_BACKEND_NAME: &str = "foyer";
const DEFAULT_MEMORY_CAPACITY_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_STORAGE_CAPACITY_BYTES: usize = 512 * 1024 * 1024;

type FoyerBlobCache = HybridCache<String, Vec<u8>>;

/// Configuration for the Foyer artifact blob cache backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoyerArtifactBlobCacheConfig {
    root: PathBuf,
    memory_capacity_bytes: usize,
    storage_capacity_bytes: usize,
}

impl FoyerArtifactBlobCacheConfig {
    /// Create a Foyer backend configuration with explicit capacities.
    #[must_use]
    pub fn new(
        root: impl Into<PathBuf>,
        memory_capacity_bytes: usize,
        storage_capacity_bytes: usize,
    ) -> Self {
        Self {
            root: root.into(),
            memory_capacity_bytes,
            storage_capacity_bytes,
        }
    }

    /// Create a Foyer backend configuration using bounded default capacities.
    #[must_use]
    pub fn with_default_capacities(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            memory_capacity_bytes: DEFAULT_MEMORY_CAPACITY_BYTES,
            storage_capacity_bytes: DEFAULT_STORAGE_CAPACITY_BYTES,
        }
    }

    /// Root directory used by Foyer's filesystem device.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    /// In-memory cache capacity in bytes.
    #[must_use]
    pub const fn memory_capacity_bytes(&self) -> usize {
        self.memory_capacity_bytes
    }

    /// Disk cache capacity in bytes.
    #[must_use]
    pub const fn storage_capacity_bytes(&self) -> usize {
        self.storage_capacity_bytes
    }
}

/// Optional Foyer backend for artifact blob bytes.
///
/// This type stores bytes only. Artifact truth, precision status, lineage, and
/// schema acceptance remain owned by the caller's catalog and gates.
pub struct FoyerArtifactBlobCache {
    inner: Option<FoyerArtifactBlobCacheInner>,
    pending_disk_work: AtomicBool,
}

struct FoyerArtifactBlobCacheInner {
    cache: FoyerBlobCache,
    runtime: Runtime,
}

impl FoyerArtifactBlobCache {
    /// Build a Foyer-backed artifact blob cache.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the internal runtime or Foyer
    /// hybrid cache cannot be created.
    pub fn from_config(config: FoyerArtifactBlobCacheConfig) -> Result<Self, ArtifactCacheError> {
        let runtime = TokioRuntimeBuilder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("xiuxian-db-store-foyer-artifact-cache")
            .build()
            .map_err(|error| {
                ArtifactCacheError::backend(
                    FOYER_BACKEND_NAME,
                    "building runtime",
                    error.to_string(),
                )
            })?;

        let cache = build_foyer_cache_on_runtime(&runtime, config)?;
        Ok(Self {
            inner: Some(FoyerArtifactBlobCacheInner { cache, runtime }),
            pending_disk_work: AtomicBool::new(false),
        })
    }

    /// Drain pending Foyer disk work before the cache handle is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] if the runtime driver panics while
    /// waiting for pending disk work.
    pub fn close(&self) -> Result<(), ArtifactCacheError> {
        if !self.pending_disk_work.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        let inner = self.inner()?;
        run_on_foyer_runtime(&inner.runtime, {
            let cache = inner.cache.clone();
            async move {
                cache.storage().wait().await;
                Ok(())
            }
        })
    }

    fn inner(&self) -> Result<&FoyerArtifactBlobCacheInner, ArtifactCacheError> {
        self.inner.as_ref().ok_or_else(|| {
            ArtifactCacheError::backend(FOYER_BACKEND_NAME, "accessing cache", "cache is closed")
        })
    }
}

impl ArtifactBlobCache for FoyerArtifactBlobCache {
    fn contains(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        Ok(self.inner()?.cache.contains(&foyer_storage_key(key)))
    }

    fn read(&self, key: &ArtifactKey) -> Result<Option<ArtifactBlobRead>, ArtifactCacheError> {
        let inner = self.inner()?;
        let cache_key = foyer_storage_key(key);
        if let Some(entry) = inner.cache.memory().get(&cache_key) {
            return Ok(Some(ArtifactBlobRead::new(entry.value().clone())));
        }
        run_on_foyer_runtime(&inner.runtime, {
            let cache = inner.cache.clone();
            async move {
                cache
                    .storage()
                    .load(&cache_key)
                    .await
                    .map(|load| match load {
                        Load::Entry { value, .. } => Some(ArtifactBlobRead::new(value)),
                        Load::Piece { piece, .. } => {
                            Some(ArtifactBlobRead::new(piece.value().clone()))
                        }
                        Load::Miss | Load::Throttled => None,
                    })
                    .map_err(|error| {
                        ArtifactCacheError::backend(
                            FOYER_BACKEND_NAME,
                            "reading bytes",
                            error.to_string(),
                        )
                    })
            }
        })
    }

    fn write(
        &self,
        key: &ArtifactKey,
        value: ArtifactBlobWrite<'_>,
    ) -> Result<ArtifactBlobWriteOutcome, ArtifactCacheError> {
        let replaced = self.contains(key)?;
        self.inner()?
            .cache
            .insert(foyer_storage_key(key), value.bytes().to_vec());
        self.pending_disk_work.store(true, Ordering::Release);
        Ok(ArtifactBlobWriteOutcome::new(value.byte_len(), replaced))
    }

    fn remove(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        let existed = self.contains(key)?;
        self.inner()?.cache.remove(&foyer_storage_key(key));
        if existed {
            self.pending_disk_work.store(true, Ordering::Release);
            self.close()?;
        }
        Ok(existed)
    }
}

impl Drop for FoyerArtifactBlobCache {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            if self.pending_disk_work.swap(false, Ordering::AcqRel) {
                let _ = run_on_foyer_runtime(&inner.runtime, {
                    let cache = inner.cache.clone();
                    async move {
                        cache.storage().wait().await;
                        Ok(())
                    }
                });
            }
            drop_foyer_inner(inner);
        }
    }
}

fn build_foyer_cache_on_runtime(
    runtime: &Runtime,
    config: FoyerArtifactBlobCacheConfig,
) -> Result<FoyerBlobCache, ArtifactCacheError> {
    run_on_foyer_runtime(runtime, async move {
        let device = FsDeviceBuilder::new(config.root())
            .with_capacity(config.storage_capacity_bytes())
            .build()
            .map_err(|error| {
                ArtifactCacheError::backend(
                    FOYER_BACKEND_NAME,
                    "building filesystem device",
                    error.to_string(),
                )
            })?;

        HybridCacheBuilder::new()
            .with_name("xiuxian-db-store-artifact-cache")
            .with_policy(HybridCachePolicy::WriteOnInsertion)
            .with_flush_on_close(false)
            .memory(config.memory_capacity_bytes())
            .storage()
            .with_engine_config(BlockEngineConfig::new(device))
            .build()
            .await
            .map_err(|error| {
                ArtifactCacheError::backend(
                    FOYER_BACKEND_NAME,
                    "building hybrid cache",
                    error.to_string(),
                )
            })
    })
}

fn run_on_foyer_runtime<T, F>(runtime: &Runtime, future: F) -> Result<T, ArtifactCacheError>
where
    T: Send,
    F: Future<Output = Result<T, ArtifactCacheError>> + Send,
{
    std::thread::scope(|scope| {
        scope
            .spawn(|| runtime.block_on(future))
            .join()
            .map_err(|_| {
                ArtifactCacheError::backend(FOYER_BACKEND_NAME, "joining runtime thread", "panic")
            })?
    })
}

fn drop_foyer_inner(inner: FoyerArtifactBlobCacheInner) {
    if tokio::runtime::Handle::try_current().is_ok() {
        let _ = std::thread::Builder::new()
            .name("xiuxian-db-store-foyer-artifact-cache-drop".to_owned())
            .spawn(move || drop(inner))
            .and_then(|handle| {
                handle
                    .join()
                    .map_err(|_| std::io::Error::other("foyer cache drop panicked"))
            });
    } else {
        drop(inner);
    }
}

fn foyer_storage_key(key: &ArtifactKey) -> String {
    format!(
        "{}/{}/{}/{}/{}",
        key.namespace().as_str(),
        key.kind().as_storage_component(),
        key.source_digest().as_str(),
        key.profile_digest().as_str(),
        key.shard_digest().as_str()
    )
}
