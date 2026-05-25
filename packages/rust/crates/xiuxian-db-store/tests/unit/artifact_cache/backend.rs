use super::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV, ARTIFACT_CACHE_FLUSHERS_ENV,
    ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
    ARTIFACT_CACHE_RECLAIMERS_ENV, ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_RUNTIME_WORKERS_ENV, ARTIFACT_CACHE_STORAGE_BYTES_ENV,
    ArtifactBlobCacheBackendConfig, ArtifactCacheBackendKind,
};

#[test]
#[cfg(not(feature = "foyer-artifact-cache"))]
fn artifact_cache_backend_config_defaults_to_filesystem_without_foyer_feature()
-> Result<(), Box<dyn std::error::Error>> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        "PRJ_CACHE_HOME" => Some("/tmp/wendao-cache".to_owned()),
        _ => None,
    })?;

    assert_eq!(config.kind(), ArtifactCacheBackendKind::Filesystem);
    assert!(config.root().ends_with("wendao/artifacts"));
    Ok(())
}

#[test]
#[cfg(feature = "foyer-artifact-cache")]
fn artifact_cache_backend_config_defaults_to_foyer_when_foyer_feature_is_enabled()
-> Result<(), Box<dyn std::error::Error>> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        "PRJ_CACHE_HOME" => Some("/tmp/wendao-cache".to_owned()),
        _ => None,
    })?;

    assert_eq!(config.kind(), ArtifactCacheBackendKind::Foyer);
    assert!(config.root().ends_with("wendao/artifacts"));
    assert!(config.runtime_worker_threads() > 0);
    Ok(())
}

#[test]
fn artifact_cache_backend_config_accepts_foyer_and_capacity_env()
-> Result<(), Box<dyn std::error::Error>> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        ARTIFACT_CACHE_BACKEND_ENV => Some("foyer".to_owned()),
        ARTIFACT_CACHE_ROOT_ENV => Some("/tmp/wendao-artifacts".to_owned()),
        ARTIFACT_CACHE_MEMORY_BYTES_ENV => Some("1048576".to_owned()),
        ARTIFACT_CACHE_STORAGE_BYTES_ENV => Some("67108864".to_owned()),
        ARTIFACT_CACHE_RUNTIME_WORKERS_ENV => Some("7".to_owned()),
        ARTIFACT_CACHE_MEMORY_SHARDS_ENV => Some("11".to_owned()),
        ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV => Some("4194304".to_owned()),
        ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV => Some("13".to_owned()),
        ARTIFACT_CACHE_FLUSHERS_ENV => Some("3".to_owned()),
        ARTIFACT_CACHE_RECLAIMERS_ENV => Some("5".to_owned()),
        _ => None,
    })?;

    assert_eq!(config.kind(), ArtifactCacheBackendKind::Foyer);
    assert_eq!(config.root(), std::path::Path::new("/tmp/wendao-artifacts"));
    assert_eq!(config.memory_capacity_bytes(), 1_048_576);
    assert_eq!(config.storage_capacity_bytes(), 67_108_864);
    assert_eq!(config.runtime_worker_threads(), 7);
    assert_eq!(config.memory_shards(), 11);
    assert_eq!(config.block_size_bytes(), 4_194_304);
    assert_eq!(config.recover_concurrency(), 13);
    assert_eq!(config.flushers(), 3);
    assert_eq!(config.reclaimers(), 5);
    assert_eq!(config.foyer_memory_weighter_name(), Some("bytes"));
    assert_eq!(config.foyer_cache_policy_name(), Some("write-on-insertion"));
    assert_eq!(config.foyer_block_size_bytes(), Some(4_194_304));
    Ok(())
}

#[test]
fn artifact_cache_backend_config_accepts_auto_runtime_workers()
-> Result<(), Box<dyn std::error::Error>> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        ARTIFACT_CACHE_BACKEND_ENV => Some("foyer".to_owned()),
        ARTIFACT_CACHE_ROOT_ENV => Some("/tmp/wendao-artifacts".to_owned()),
        ARTIFACT_CACHE_RUNTIME_WORKERS_ENV
        | ARTIFACT_CACHE_MEMORY_SHARDS_ENV
        | ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV
        | ARTIFACT_CACHE_FLUSHERS_ENV
        | ARTIFACT_CACHE_RECLAIMERS_ENV => Some("auto".to_owned()),
        _ => None,
    })?;

    assert!(config.runtime_worker_threads() > 0);
    assert!(config.memory_shards() > 0);
    assert!(config.recover_concurrency() > 0);
    assert!(config.flushers() > 0);
    assert!(config.reclaimers() > 0);
    Ok(())
}

#[test]
fn artifact_cache_backend_config_adapts_io_lanes_to_block_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        ARTIFACT_CACHE_BACKEND_ENV => Some("foyer".to_owned()),
        ARTIFACT_CACHE_ROOT_ENV => Some("/tmp/wendao-artifacts".to_owned()),
        ARTIFACT_CACHE_STORAGE_BYTES_ENV | ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV => {
            Some("16777216".to_owned())
        }
        ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV
        | ARTIFACT_CACHE_FLUSHERS_ENV
        | ARTIFACT_CACHE_RECLAIMERS_ENV => Some("auto".to_owned()),
        _ => None,
    })?;

    assert_eq!(config.block_size_bytes(), 16_777_216);
    assert_eq!(config.recover_concurrency(), 1);
    assert_eq!(config.flushers(), 1);
    assert_eq!(config.reclaimers(), 1);
    Ok(())
}

#[test]
fn artifact_cache_backend_config_rejects_zero_adaptive_concurrency() {
    let Err(error) = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        ARTIFACT_CACHE_BACKEND_ENV => Some("foyer".to_owned()),
        ARTIFACT_CACHE_ROOT_ENV => Some("/tmp/wendao-artifacts".to_owned()),
        ARTIFACT_CACHE_MEMORY_SHARDS_ENV => Some("0".to_owned()),
        _ => None,
    }) else {
        panic!("zero adaptive concurrency should be rejected");
    };

    assert!(error.to_string().contains(ARTIFACT_CACHE_MEMORY_SHARDS_ENV));
}
