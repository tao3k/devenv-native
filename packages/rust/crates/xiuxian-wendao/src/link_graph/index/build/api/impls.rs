//! Coordinates link-graph index builds across cache, Valkey, graph memory, and runtime config.

use crate::link_graph::index::build::cache::{
    CacheLookupOutcome, LINK_GRAPH_CACHE_SCHEMA_VERSION, cache_schema_fingerprint,
    default_local_duckdb_cache_path, load_cached_index_from_duckdb, load_cached_index_from_valkey,
    save_cached_index_to_duckdb, save_cached_index_to_valkey,
};
use crate::link_graph::index::build::graphmem::{
    sync_graphmem_state_best_effort, sync_graphmem_state_to_valkey,
};
use crate::link_graph::index::{LinkGraphCacheBuildMeta, LinkGraphIndex};
use crate::link_graph::runtime_config::{
    LinkGraphCacheRuntimeConfig, resolve_link_graph_cache_runtime,
};

use super::build_context::{
    BuildCacheContext, BuildCacheSlotContext, prepare_build_cache_context,
    prepare_build_cache_slot_context,
};
use super::meta::build_cache_meta;
use crate::link_graph::index::build::fingerprint::LinkGraphFingerprint;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ResidentLocalCacheKey {
    cache_path: PathBuf,
    slot_key: String,
}

#[derive(Debug, Clone)]
struct ResidentLocalCacheEntry {
    fingerprint: LinkGraphFingerprint,
    index: Arc<LinkGraphIndex>,
}

type ResidentLocalCacheMap = HashMap<ResidentLocalCacheKey, ResidentLocalCacheEntry>;

static RESIDENT_LOCAL_LINK_GRAPH_CACHE: OnceLock<Mutex<ResidentLocalCacheMap>> = OnceLock::new();

fn resident_local_cache() -> &'static Mutex<ResidentLocalCacheMap> {
    RESIDENT_LOCAL_LINK_GRAPH_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lookup_resident_local_cache(
    key: &ResidentLocalCacheKey,
    fingerprint: &LinkGraphFingerprint,
) -> Result<Option<Arc<LinkGraphIndex>>, String> {
    let cache = resident_local_cache()
        .lock()
        .map_err(|error| format!("link-graph resident cache lock poisoned: {error}"))?;
    Ok(cache
        .get(key)
        .filter(|entry| entry.fingerprint == *fingerprint)
        .map(|entry| Arc::clone(&entry.index)))
}

fn lookup_prewarmed_resident_local_cache(
    key: &ResidentLocalCacheKey,
) -> Result<Option<Arc<LinkGraphIndex>>, String> {
    let cache = resident_local_cache()
        .lock()
        .map_err(|error| format!("link-graph resident cache lock poisoned: {error}"))?;
    Ok(cache.get(key).map(|entry| Arc::clone(&entry.index)))
}

fn store_resident_local_cache(
    key: ResidentLocalCacheKey,
    fingerprint: LinkGraphFingerprint,
    index: Arc<LinkGraphIndex>,
) -> Result<(), String> {
    let mut cache = resident_local_cache()
        .lock()
        .map_err(|error| format!("link-graph resident cache lock poisoned: {error}"))?;
    cache.insert(key, ResidentLocalCacheEntry { fingerprint, index });
    Ok(())
}

fn invalidate_resident_local_cache(key: &ResidentLocalCacheKey) -> Result<bool, String> {
    let mut cache = resident_local_cache()
        .lock()
        .map_err(|error| format!("link-graph resident cache lock poisoned: {error}"))?;
    Ok(cache.remove(key).is_some())
}

fn resident_local_cache_key(
    slot: &BuildCacheSlotContext,
    cache_path: &Path,
) -> ResidentLocalCacheKey {
    ResidentLocalCacheKey {
        cache_path: cache_path.to_path_buf(),
        slot_key: slot.slot_key.clone(),
    }
}

impl LinkGraphIndex {
    /// Build index from notebook root directory.
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails.
    pub fn build(root_dir: &Path) -> Result<Self, String> {
        let index = Self::build_with_filters(root_dir, &[], &[])?;
        sync_graphmem_state_best_effort(&index);
        Ok(index)
    }

    fn build_with_cache_runtime_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        runtime: &LinkGraphCacheRuntimeConfig,
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        let context = prepare_build_cache_context(root_dir, include_dirs, excluded_dirs)?;
        let cache_lookup = load_cached_index_from_valkey(
            runtime,
            &context.slot.slot_key,
            &context.slot.root,
            &context.slot.normalized_include_dirs,
            &context.slot.normalized_excluded_dirs,
            &context.fingerprint,
        )?;
        let miss_reason = match cache_lookup {
            CacheLookupOutcome::Hit(index) => {
                let _ = sync_graphmem_state_to_valkey(&index, runtime);
                let meta = build_cache_meta("valkey", "hit", None);
                return Ok((*index, meta));
            }
            CacheLookupOutcome::Miss(reason) => Some(reason.to_string()),
        };

        let index = Self::build_with_filters(
            &context.slot.root,
            &context.slot.normalized_include_dirs,
            &context.slot.normalized_excluded_dirs,
        )?;
        let _ = sync_graphmem_state_to_valkey(&index, runtime);
        save_cached_index_to_valkey(&index, runtime, &context.slot.slot_key, context.fingerprint)?;
        let meta = build_cache_meta("valkey", "miss", miss_reason);
        Ok((index, meta))
    }

    fn build_with_local_cache_path_with_meta_impl(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        let context = prepare_build_cache_context(root_dir, include_dirs, excluded_dirs)?;
        Self::build_with_local_cache_context_with_meta_impl(&context, cache_path)
    }

    fn build_with_local_cache_context_with_meta_impl(
        context: &BuildCacheContext,
        cache_path: &Path,
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        #[cfg(feature = "duckdb")]
        let mut miss_reason = match load_cached_index_from_duckdb(
            cache_path,
            &context.slot.slot_key,
            &context.slot.root,
            &context.slot.normalized_include_dirs,
            &context.slot.normalized_excluded_dirs,
            &context.fingerprint,
        ) {
            Ok(CacheLookupOutcome::Hit(index)) => {
                let meta = build_cache_meta("duckdb", "hit", None);
                return Ok((*index, meta));
            }
            Ok(CacheLookupOutcome::Miss(reason)) => Some(reason.to_string()),
            Err(error) => Some(format!("duckdb_cache_unavailable: {error}")),
        };
        #[cfg(not(feature = "duckdb"))]
        let miss_reason = match load_cached_index_from_duckdb(
            cache_path,
            &context.slot.slot_key,
            &context.slot.root,
            &context.slot.normalized_include_dirs,
            &context.slot.normalized_excluded_dirs,
            &context.fingerprint,
        ) {
            CacheLookupOutcome::Hit(index) => {
                let meta = build_cache_meta("duckdb", "hit", None);
                return Ok((*index, meta));
            }
            CacheLookupOutcome::Miss(reason) => Some(reason.to_string()),
        };

        let index = Self::build_with_filters(
            &context.slot.root,
            &context.slot.normalized_include_dirs,
            &context.slot.normalized_excluded_dirs,
        )?;
        #[cfg(feature = "duckdb")]
        if let Err(error) = save_cached_index_to_duckdb(
            &index,
            cache_path,
            &context.slot.slot_key,
            &context.fingerprint,
        ) && miss_reason.is_none()
        {
            miss_reason = Some(format!("duckdb_cache_save_failed: {error}"));
        }
        #[cfg(not(feature = "duckdb"))]
        save_cached_index_to_duckdb(
            &index,
            cache_path,
            &context.slot.slot_key,
            &context.fingerprint,
        );
        let meta = build_cache_meta("duckdb", "miss", miss_reason);
        Ok((index, meta))
    }

    fn build_with_resident_local_cache_path_with_meta_impl(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<(Arc<Self>, LinkGraphCacheBuildMeta), String> {
        let context = prepare_build_cache_context(root_dir, include_dirs, excluded_dirs)?;
        let key = resident_local_cache_key(&context.slot, cache_path);
        if let Some(index) = lookup_resident_local_cache(&key, &context.fingerprint)? {
            let meta = build_cache_meta("resident", "hit", None);
            return Ok((index, meta));
        }

        let fingerprint = context.fingerprint.clone();
        let (index, meta) =
            Self::build_with_local_cache_context_with_meta_impl(&context, cache_path)?;
        let index = Arc::new(index);
        store_resident_local_cache(key, fingerprint, Arc::clone(&index))?;
        Ok((index, meta))
    }

    /// Build index with cache fast-path.
    ///
    /// Uses a fingerprint-validated snapshot in `Valkey`.
    /// Rebuilds when cache key is missing/stale, then writes snapshot back to `Valkey`.
    ///
    /// # Errors
    ///
    /// Returns an error when runtime config resolution, cache I/O, or index build fails.
    pub fn build_with_cache(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
    ) -> Result<Self, String> {
        let runtime = resolve_link_graph_cache_runtime()?;
        let (index, _) = Self::build_with_cache_runtime_with_meta(
            root_dir,
            include_dirs,
            excluded_dirs,
            &runtime,
        )?;
        Ok(index)
    }

    /// Build index with cache fast-path and return cache build metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when runtime config resolution, cache I/O, or index build fails.
    pub fn build_with_cache_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        let runtime = resolve_link_graph_cache_runtime()?;
        Self::build_with_cache_runtime_with_meta(root_dir, include_dirs, excluded_dirs, &runtime)
    }

    /// Build index with the default local `DuckDB` cache fast-path.
    ///
    /// Uses a fingerprint-validated snapshot in a project-local `DuckDB` file
    /// and does not resolve Valkey runtime config.
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails.
    pub fn build_with_local_cache(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
    ) -> Result<Self, String> {
        let (index, _) =
            Self::build_with_local_cache_with_meta(root_dir, include_dirs, excluded_dirs)?;
        Ok(index)
    }

    /// Build index with the default local `DuckDB` cache fast-path and return
    /// cache build metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails.
    pub fn build_with_local_cache_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        let root = root_dir
            .canonicalize()
            .map_err(|e| format!("invalid notebook root '{}': {e}", root_dir.display()))?;
        let cache_path = default_local_duckdb_cache_path(&root);
        Self::build_with_local_cache_path_with_meta_impl(
            &root,
            include_dirs,
            excluded_dirs,
            &cache_path,
        )
    }

    /// Build index with an explicit local `DuckDB` cache file and return cache
    /// build metadata.
    ///
    /// Intended for tests and controlled local runners that need an isolated
    /// cache file.
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails.
    pub fn build_with_local_cache_path_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        Self::build_with_local_cache_path_with_meta_impl(
            root_dir,
            include_dirs,
            excluded_dirs,
            cache_path,
        )
    }

    /// Build index with an explicit local `DuckDB` cache file and a
    /// fingerprint-gated in-process resident fast-path.
    ///
    /// Intended for long-lived runners that need to avoid repeated
    /// `DuckDB`/Arrow snapshot loads while preserving fingerprint validation.
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails or the resident cache
    /// lock is poisoned.
    pub fn build_with_resident_local_cache_path_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<(Arc<Self>, LinkGraphCacheBuildMeta), String> {
        Self::build_with_resident_local_cache_path_with_meta_impl(
            root_dir,
            include_dirs,
            excluded_dirs,
            cache_path,
        )
    }

    /// Prewarm the resident `LinkGraph` index for an explicit local `DuckDB`
    /// cache file.
    ///
    /// This validates the current fingerprint, loads from resident/DuckDB or
    /// rebuilds as needed, and stores the resulting index in the in-process
    /// resident cache.
    ///
    /// # Errors
    ///
    /// Returns an error when index construction fails or the resident cache
    /// lock is poisoned.
    pub fn prewarm_resident_local_cache_path_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<(Arc<Self>, LinkGraphCacheBuildMeta), String> {
        Self::build_with_resident_local_cache_path_with_meta_impl(
            root_dir,
            include_dirs,
            excluded_dirs,
            cache_path,
        )
    }

    /// Lookup a prewarmed resident `LinkGraph` index without revalidating the
    /// filesystem fingerprint.
    ///
    /// This is intended for request paths where a surrounding lifecycle has
    /// already run prewarm or invalidation. It returns a miss error when no
    /// resident index is loaded for the cache slot.
    ///
    /// # Errors
    ///
    /// Returns an error when slot normalization fails, no resident index is
    /// loaded, or the resident cache lock is poisoned.
    pub fn lookup_prewarmed_resident_local_cache_path_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<(Arc<Self>, LinkGraphCacheBuildMeta), String> {
        let slot = prepare_build_cache_slot_context(root_dir, include_dirs, excluded_dirs)?;
        let key = resident_local_cache_key(&slot, cache_path);
        let index = lookup_prewarmed_resident_local_cache(&key)?.ok_or_else(|| {
            format!(
                "link-graph resident cache miss for slot `{}`",
                slot.slot_key
            )
        })?;
        let meta = build_cache_meta("resident-prewarmed", "hit", None);
        Ok((index, meta))
    }

    /// Invalidate a prewarmed resident `LinkGraph` index for an explicit local
    /// `DuckDB` cache file.
    ///
    /// # Errors
    ///
    /// Returns an error when slot normalization fails or the resident cache
    /// lock is poisoned.
    pub fn invalidate_resident_local_cache_path(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        cache_path: &Path,
    ) -> Result<bool, String> {
        let slot = prepare_build_cache_slot_context(root_dir, include_dirs, excluded_dirs)?;
        let key = resident_local_cache_key(&slot, cache_path);
        invalidate_resident_local_cache(&key)
    }

    /// Build index with an explicit `Valkey` cache runtime.
    ///
    /// Intended for tests and controlled runners that pass cache config directly.
    ///
    /// # Errors
    ///
    /// Returns an error when `valkey_url` is invalid, cache I/O fails, or index build fails.
    pub fn build_with_cache_with_valkey(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        valkey_url: &str,
        key_prefix: Option<&str>,
        ttl_seconds: Option<u64>,
    ) -> Result<Self, String> {
        if valkey_url.trim().is_empty() {
            return Err("link_graph cache valkey_url must be non-empty".to_string());
        }
        let runtime = LinkGraphCacheRuntimeConfig::from_parts(valkey_url, key_prefix, ttl_seconds);
        let (index, _) = Self::build_with_cache_runtime_with_meta(
            root_dir,
            include_dirs,
            excluded_dirs,
            &runtime,
        )?;
        Ok(index)
    }

    /// Build index with explicit `Valkey` runtime and return cache build metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when `valkey_url` is invalid, cache I/O fails, or index build fails.
    pub fn build_with_cache_with_valkey_with_meta(
        root_dir: &Path,
        include_dirs: &[String],
        excluded_dirs: &[String],
        valkey_url: &str,
        key_prefix: Option<&str>,
        ttl_seconds: Option<u64>,
    ) -> Result<(Self, LinkGraphCacheBuildMeta), String> {
        if valkey_url.trim().is_empty() {
            return Err("link_graph cache valkey_url must be non-empty".to_string());
        }
        let runtime = LinkGraphCacheRuntimeConfig::from_parts(valkey_url, key_prefix, ttl_seconds);
        Self::build_with_cache_runtime_with_meta(root_dir, include_dirs, excluded_dirs, &runtime)
    }

    /// Return the schema version used by `LinkGraph` cache snapshots.
    #[must_use]
    pub fn cache_schema_version() -> &'static str {
        LINK_GRAPH_CACHE_SCHEMA_VERSION
    }

    /// Return the schema version used by `LinkGraph` `Valkey` cache snapshots.
    #[must_use]
    pub fn valkey_cache_schema_version() -> &'static str {
        Self::cache_schema_version()
    }

    /// Return the schema fingerprint used by `LinkGraph` cache snapshots.
    ///
    /// Fingerprint changes whenever the shared schema JSON changes.
    #[must_use]
    pub fn cache_schema_fingerprint() -> &'static str {
        cache_schema_fingerprint()
    }

    /// Return the schema fingerprint used by `LinkGraph` `Valkey` cache snapshots.
    ///
    /// Fingerprint changes whenever the shared schema JSON changes.
    #[must_use]
    pub fn valkey_cache_schema_fingerprint() -> &'static str {
        Self::cache_schema_fingerprint()
    }
}
