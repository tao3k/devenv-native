//! PDF render artifact cache backend discovery and sharing.

use std::env;

#[cfg(feature = "foyer-artifact-cache")]
use std::collections::BTreeMap;
#[cfg(feature = "foyer-artifact-cache")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV, ARTIFACT_CACHE_FLUSHERS_ENV,
    ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
    ARTIFACT_CACHE_RECLAIMERS_ENV, ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_RUNTIME_WORKERS_ENV, ARTIFACT_CACHE_STORAGE_BYTES_ENV, ArtifactBlobCacheBackend,
    ArtifactBlobCacheBackendConfig,
};

use super::model::PdfRenderArtifactCache;

#[cfg(feature = "foyer-artifact-cache")]
static PDF_RENDER_ARTIFACT_CACHE_BACKENDS: OnceLock<
    Mutex<BTreeMap<String, Arc<ArtifactBlobCacheBackend>>>,
> = OnceLock::new();

pub(crate) fn pdf_render_artifact_cache_from_environment()
-> Result<Option<PdfRenderArtifactCache>, String> {
    #[cfg(feature = "foyer-artifact-cache")]
    {
        if !artifact_cache_env_present() {
            return Ok(None);
        }
        let config = ArtifactBlobCacheBackendConfig::from_env()
            .map_err(|error| format!("resolve PDF render ArtifactBlobCache backend: {error}"))?;
        let backend = shared_pdf_render_artifact_cache_backend(&config)?;
        Ok(Some(PdfRenderArtifactCache::new(backend)))
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        if env::var("WENDAO_ARTIFACT_CACHE_BACKEND").is_ok()
            || env::var("WENDAO_ARTIFACT_CACHE_ROOT").is_ok()
        {
            return Err(
                "PDF render artifact cache is configured but foyer-artifact-cache is not enabled"
                    .to_string(),
            );
        }
        Ok(None)
    }
}

#[cfg(feature = "foyer-artifact-cache")]
fn shared_pdf_render_artifact_cache_backend(
    config: &ArtifactBlobCacheBackendConfig,
) -> Result<Arc<ArtifactBlobCacheBackend>, String> {
    let key = artifact_cache_backend_config_key(config);
    let backends = PDF_RENDER_ARTIFACT_CACHE_BACKENDS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut backends = backends
        .lock()
        .map_err(|_| "PDF render ArtifactBlobCache backend registry lock poisoned".to_string())?;
    if let Some(backend) = backends.get(&key) {
        return Ok(Arc::clone(backend));
    }
    let backend = Arc::new(
        config
            .build()
            .map_err(|error| format!("build PDF render ArtifactBlobCache backend: {error}"))?,
    );
    backends.insert(key, Arc::clone(&backend));
    Ok(backend)
}

#[cfg(feature = "foyer-artifact-cache")]
fn artifact_cache_backend_config_key(config: &ArtifactBlobCacheBackendConfig) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        config.kind().as_str(),
        config.root().display(),
        config.memory_capacity_bytes(),
        config.storage_capacity_bytes(),
        config.runtime_worker_threads(),
        config.memory_shards(),
        config.block_size_bytes(),
        config.recover_concurrency(),
        config.flushers(),
        config.reclaimers()
    )
}

#[cfg(feature = "foyer-artifact-cache")]
fn artifact_cache_env_present() -> bool {
    [
        ARTIFACT_CACHE_BACKEND_ENV,
        ARTIFACT_CACHE_ROOT_ENV,
        ARTIFACT_CACHE_MEMORY_BYTES_ENV,
        ARTIFACT_CACHE_STORAGE_BYTES_ENV,
        ARTIFACT_CACHE_RUNTIME_WORKERS_ENV,
        ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
        ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV,
        ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV,
        ARTIFACT_CACHE_FLUSHERS_ENV,
        ARTIFACT_CACHE_RECLAIMERS_ENV,
        "PRJ_CACHE_HOME",
    ]
    .iter()
    .any(|key| env::var(key).is_ok_and(|value| !value.trim().is_empty()))
}
