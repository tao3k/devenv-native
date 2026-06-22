//! Foyer-backed `ArtifactBlobCache` implementation.

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use foyer::{
    BlockEngineConfig, Code, DeviceBuilder, Error as FoyerError, Event, EventListener,
    FsDeviceBuilder, HybridCache, HybridCacheBuilder, HybridCachePolicy, Source,
};
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime};

use crate::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobFetch, ArtifactBlobFetchBuilder, ArtifactBlobFetchParts,
    ArtifactBlobFetchStatus, ArtifactBlobRead, ArtifactBlobReadStatus, ArtifactBlobWrite,
    ArtifactBlobWriteOutcome, ArtifactBytes, ArtifactCacheError, ArtifactKey,
};

const FOYER_BACKEND_NAME: &str = "foyer";
const DEFAULT_MEMORY_CAPACITY_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_STORAGE_CAPACITY_BYTES: usize = 512 * 1024 * 1024;
const MIN_FOYER_BLOCK_SIZE_BYTES: usize = 4 * 1024;
/// Foyer in-memory admission uses byte weight for artifact keys and payloads.
pub const FOYER_ARTIFACT_MEMORY_WEIGHTER: &str = "bytes";
/// Artifact persistence writes through to disk on insertion for restart reuse.
pub const FOYER_ARTIFACT_CACHE_POLICY: &str = "write-on-insertion";
/// Default Foyer block size used by the artifact backend.
pub const FOYER_ARTIFACT_BLOCK_SIZE_BYTES: usize = 16 * 1024 * 1024;

type FoyerBlobCache = HybridCache<String, ArtifactBytes>;

impl Code for ArtifactBytes {
    fn encode(&self, writer: &mut impl Write) -> foyer::Result<()> {
        self.len().encode(writer)?;
        writer
            .write_all(self.as_slice())
            .map_err(FoyerError::io_error)
    }

    fn decode(reader: &mut impl Read) -> foyer::Result<Self>
    where
        Self: Sized,
    {
        Vec::<u8>::decode(reader).map(ArtifactBytes::from_vec)
    }

    fn estimated_size(&self) -> usize {
        std::mem::size_of::<usize>() + self.len()
    }
}

/// Configuration for the Foyer artifact blob cache backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoyerArtifactBlobCacheConfig {
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
            runtime_worker_threads: default_runtime_worker_threads(),
            memory_shards: default_memory_shards(),
            block_size_bytes: FOYER_ARTIFACT_BLOCK_SIZE_BYTES,
            recover_concurrency: default_recover_concurrency(),
            flushers: default_io_lanes(),
            reclaimers: default_io_lanes(),
        }
    }

    /// Create a Foyer backend configuration with explicit capacities and
    /// runtime worker count.
    #[must_use]
    pub fn new_with_runtime_workers(
        root: impl Into<PathBuf>,
        memory_capacity_bytes: usize,
        storage_capacity_bytes: usize,
        runtime_worker_threads: usize,
    ) -> Self {
        Self {
            root: root.into(),
            memory_capacity_bytes,
            storage_capacity_bytes,
            runtime_worker_threads,
            memory_shards: default_memory_shards(),
            block_size_bytes: FOYER_ARTIFACT_BLOCK_SIZE_BYTES,
            recover_concurrency: default_recover_concurrency(),
            flushers: default_io_lanes(),
            reclaimers: default_io_lanes(),
        }
    }

    /// Create a Foyer backend configuration using bounded default capacities.
    #[must_use]
    pub fn with_default_capacities(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            memory_capacity_bytes: DEFAULT_MEMORY_CAPACITY_BYTES,
            storage_capacity_bytes: DEFAULT_STORAGE_CAPACITY_BYTES,
            runtime_worker_threads: default_runtime_worker_threads(),
            memory_shards: default_memory_shards(),
            block_size_bytes: FOYER_ARTIFACT_BLOCK_SIZE_BYTES,
            recover_concurrency: default_recover_concurrency(),
            flushers: default_io_lanes(),
            reclaimers: default_io_lanes(),
        }
    }

    /// Set Foyer memory shard count.
    #[must_use]
    pub fn with_memory_shards(mut self, memory_shards: usize) -> Self {
        self.memory_shards = memory_shards;
        self
    }

    /// Set the Foyer block-engine block size in bytes.
    #[must_use]
    pub fn with_block_size_bytes(mut self, block_size_bytes: usize) -> Self {
        self.block_size_bytes = block_size_bytes;
        self
    }

    /// Set the Foyer disk recover concurrency.
    #[must_use]
    pub fn with_recover_concurrency(mut self, recover_concurrency: usize) -> Self {
        self.recover_concurrency = recover_concurrency;
        self
    }

    /// Set Foyer disk flusher count.
    #[must_use]
    pub fn with_flushers(mut self, flushers: usize) -> Self {
        self.flushers = flushers;
        self
    }

    /// Set Foyer disk reclaimer count.
    #[must_use]
    pub fn with_reclaimers(mut self, reclaimers: usize) -> Self {
        self.reclaimers = reclaimers;
        self
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

    /// Tokio runtime worker threads used by Foyer disk operations.
    #[must_use]
    pub const fn runtime_worker_threads(&self) -> usize {
        self.runtime_worker_threads
    }

    /// Foyer memory shard count.
    #[must_use]
    pub const fn memory_shards(&self) -> usize {
        self.memory_shards
    }

    /// Foyer block-engine block size in bytes.
    #[must_use]
    pub const fn block_size_bytes(&self) -> usize {
        self.block_size_bytes
    }

    /// Foyer disk recover concurrency.
    #[must_use]
    pub const fn recover_concurrency(&self) -> usize {
        self.recover_concurrency
    }

    /// Foyer disk flusher count.
    #[must_use]
    pub const fn flushers(&self) -> usize {
        self.flushers
    }

    /// Foyer disk reclaimer count.
    #[must_use]
    pub const fn reclaimers(&self) -> usize {
        self.reclaimers
    }
}

/// Snapshot of Foyer in-memory cache leave events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FoyerArtifactBlobCacheEventStats {
    evicted: u64,
    replaced: u64,
    removed: u64,
    cleared: u64,
}

impl FoyerArtifactBlobCacheEventStats {
    /// Entries evicted from the in-memory tier.
    #[must_use]
    pub const fn evicted_entries(self) -> u64 {
        self.evicted
    }

    /// Entries replaced by a newer value.
    #[must_use]
    pub const fn replaced_entries(self) -> u64 {
        self.replaced
    }

    /// Entries removed explicitly.
    #[must_use]
    pub const fn removed_entries(self) -> u64 {
        self.removed
    }

    /// Entries removed by a cache clear.
    #[must_use]
    pub const fn cleared_entries(self) -> u64 {
        self.cleared
    }
}

#[derive(Debug, Default)]
struct FoyerArtifactBlobCacheEvents {
    evicted: AtomicU64,
    replaced: AtomicU64,
    removed: AtomicU64,
    cleared: AtomicU64,
}

impl FoyerArtifactBlobCacheEvents {
    fn snapshot(&self) -> FoyerArtifactBlobCacheEventStats {
        FoyerArtifactBlobCacheEventStats {
            evicted: self.evicted.load(Ordering::Relaxed),
            replaced: self.replaced.load(Ordering::Relaxed),
            removed: self.removed.load(Ordering::Relaxed),
            cleared: self.cleared.load(Ordering::Relaxed),
        }
    }
}

impl EventListener for FoyerArtifactBlobCacheEvents {
    type Key = String;
    type Value = ArtifactBytes;

    fn on_leave(&self, reason: Event, _key: &Self::Key, _value: &Self::Value) {
        let counter = match reason {
            Event::Evict => &self.evicted,
            Event::Replace => &self.replaced,
            Event::Remove => &self.removed,
            Event::Clear => &self.cleared,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }
}

/// Optional Foyer backend for artifact blob bytes.
///
/// This type stores bytes only. Artifact truth, precision status, lineage, and
/// schema acceptance remain owned by the caller's catalog and gates.
pub struct FoyerArtifactBlobCache {
    inner: Option<FoyerArtifactBlobCacheInner>,
    pending_disk_work: AtomicBool,
    events: Arc<FoyerArtifactBlobCacheEvents>,
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
        let events = Arc::new(FoyerArtifactBlobCacheEvents::default());
        let runtime = TokioRuntimeBuilder::new_multi_thread()
            .worker_threads(config.runtime_worker_threads())
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

        let cache = build_foyer_cache_on_runtime(&runtime, config, Arc::clone(&events))?;
        Ok(Self {
            inner: Some(FoyerArtifactBlobCacheInner { cache, runtime }),
            pending_disk_work: AtomicBool::new(false),
            events,
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
                cache.close().await.map_err(|error| {
                    ArtifactCacheError::backend(
                        FOYER_BACKEND_NAME,
                        "closing hybrid cache",
                        error.to_string(),
                    )
                })
            }
        })
    }

    /// Return in-memory cache leave-event counters.
    #[must_use]
    pub fn event_stats(&self) -> FoyerArtifactBlobCacheEventStats {
        self.events.snapshot()
    }

    fn inner(&self) -> Result<&FoyerArtifactBlobCacheInner, ArtifactCacheError> {
        self.inner.as_ref().ok_or_else(|| {
            ArtifactCacheError::backend(FOYER_BACKEND_NAME, "accessing cache", "cache is closed")
        })
    }
}

impl ArtifactBlobCache for FoyerArtifactBlobCache {
    fn backend_name(&self) -> &'static str {
        FOYER_BACKEND_NAME
    }

    fn contains(&self, key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        Ok(self.inner()?.cache.contains(&foyer_storage_key(key)))
    }

    fn read(&self, key: &ArtifactKey) -> Result<Option<ArtifactBlobRead>, ArtifactCacheError> {
        Ok(self.read_with_status(key)?.into_read())
    }

    fn read_with_status(
        &self,
        key: &ArtifactKey,
    ) -> Result<ArtifactBlobReadStatus, ArtifactCacheError> {
        let inner = self.inner()?;
        let cache_key = foyer_storage_key(key);
        run_on_foyer_runtime(&inner.runtime, {
            let cache = inner.cache.clone();
            async move {
                cache
                    .get(&cache_key)
                    .await
                    .map(|entry| match entry {
                        Some(entry) => ArtifactBlobReadStatus::Hit(ArtifactBlobRead::from_shared(
                            entry.value().clone(),
                        )),
                        None => ArtifactBlobReadStatus::Miss,
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

    fn fetch_through(
        &self,
        key: &ArtifactKey,
        build: ArtifactBlobFetchBuilder,
    ) -> Result<ArtifactBlobFetch, ArtifactCacheError> {
        let inner = self.inner()?;
        let cache_key = foyer_storage_key(key);
        let started = std::time::Instant::now();
        let entry = run_on_foyer_runtime(&inner.runtime, {
            let cache = inner.cache.clone();
            let cache_key = cache_key.clone();
            async move {
                cache
                    .get_or_fetch(&cache_key, || async move {
                        build().map(ArtifactBytes::from_vec)
                    })
                    .await
                    .map_err(|error| {
                        ArtifactCacheError::backend(
                            FOYER_BACKEND_NAME,
                            "fetching bytes",
                            error.to_string(),
                        )
                    })
            }
        })?;
        let elapsed = started.elapsed();
        let status = match entry.source() {
            Source::Memory | Source::Disk => ArtifactBlobFetchStatus::Hit,
            Source::Outer => ArtifactBlobFetchStatus::Miss,
        };
        let write = if status == ArtifactBlobFetchStatus::Miss {
            self.pending_disk_work.store(true, Ordering::Release);
            Some(ArtifactBlobWriteOutcome::new(entry.value().len(), false))
        } else {
            None
        };
        let build_elapsed = if status.is_hit() {
            std::time::Duration::ZERO
        } else {
            elapsed
        };
        Ok(ArtifactBlobFetch::from_parts(
            ArtifactBlobFetchParts::from_shared_bytes(entry.value().clone(), status)
                .with_write(write)
                .with_read_elapsed(elapsed)
                .with_build_elapsed(build_elapsed),
        ))
    }

    fn write(
        &self,
        key: &ArtifactKey,
        value: ArtifactBlobWrite<'_>,
    ) -> Result<ArtifactBlobWriteOutcome, ArtifactCacheError> {
        let replaced = self.contains(key)?;
        self.inner()?.cache.insert(
            foyer_storage_key(key),
            ArtifactBytes::from_slice(value.bytes()),
        );
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
                        cache.close().await.map_err(|error| {
                            ArtifactCacheError::backend(
                                FOYER_BACKEND_NAME,
                                "closing hybrid cache",
                                error.to_string(),
                            )
                        })
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
    events: Arc<FoyerArtifactBlobCacheEvents>,
) -> Result<FoyerBlobCache, ArtifactCacheError> {
    run_on_foyer_runtime(runtime, async move {
        let block_size_bytes = normalized_block_size_bytes(config.block_size_bytes());
        let memory_shards =
            effective_memory_shards(config.memory_shards(), config.memory_capacity_bytes());
        let recover_concurrency = effective_recover_concurrency(
            config.recover_concurrency(),
            config.storage_capacity_bytes(),
            block_size_bytes,
        );
        let flushers = effective_io_lanes(
            config.flushers(),
            config.storage_capacity_bytes(),
            block_size_bytes,
        );
        let reclaimers = effective_io_lanes(
            config.reclaimers(),
            config.storage_capacity_bytes(),
            block_size_bytes,
        );
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
            .with_flush_on_close(true)
            .with_event_listener(events)
            .memory(config.memory_capacity_bytes())
            .with_shards(memory_shards)
            .with_weighter(|key, value| key.len().saturating_add(value.len()))
            .storage()
            .with_engine_config(
                BlockEngineConfig::new(device)
                    .with_block_size(block_size_bytes)
                    .with_recover_concurrency(recover_concurrency)
                    .with_flushers(flushers)
                    .with_reclaimers(reclaimers),
            )
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
