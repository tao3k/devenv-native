use super::super::{LinkGraphCacheBuildMeta, LinkGraphIndex};
use super::cache::{
    CacheLookupOutcome, LINK_GRAPH_VALKEY_CACHE_SCHEMA_VERSION, cache_schema_fingerprint,
    load_cached_index_from_valkey, save_cached_index_to_valkey,
};
use super::graphmem::{sync_graphmem_state_best_effort, sync_graphmem_state_to_valkey};
use crate::link_graph::runtime_config::{
    LinkGraphCacheRuntimeConfig, resolve_link_graph_cache_runtime,
};
#[path = "api/build_context.rs"]
mod build_context;
#[path = "api/meta.rs"]
mod meta;

use build_context::prepare_build_cache_context;
use meta::build_cache_meta;
use std::path::Path;

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
            &context.slot_key,
            &context.root,
            &context.normalized_include_dirs,
            &context.normalized_excluded_dirs,
            &context.fingerprint,
        )?;
        let miss_reason = match cache_lookup {
            CacheLookupOutcome::Hit(index) => {
                let _ = sync_graphmem_state_to_valkey(&index, runtime);
                let meta = build_cache_meta("hit", None);
                return Ok((*index, meta));
            }
            CacheLookupOutcome::Miss(reason) => Some(reason.to_string()),
        };

        let index = Self::build_with_filters(
            &context.root,
            &context.normalized_include_dirs,
            &context.normalized_excluded_dirs,
        )?;
        let _ = sync_graphmem_state_to_valkey(&index, runtime);
        save_cached_index_to_valkey(&index, runtime, &context.slot_key, context.fingerprint)?;
        let meta = build_cache_meta("miss", miss_reason);
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

    /// Return the schema version used by `LinkGraph` `Valkey` cache snapshots.
    #[must_use]
    pub fn valkey_cache_schema_version() -> &'static str {
        LINK_GRAPH_VALKEY_CACHE_SCHEMA_VERSION
    }

    /// Return the schema fingerprint used by `LinkGraph` `Valkey` cache snapshots.
    ///
    /// Fingerprint changes whenever the shared schema JSON changes.
    #[must_use]
    pub fn valkey_cache_schema_fingerprint() -> &'static str {
        cache_schema_fingerprint()
    }
}
