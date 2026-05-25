use super::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_STORAGE_BYTES_ENV, ArtifactBlobCacheBackendConfig, ArtifactCacheBackendKind,
};

#[test]
fn artifact_cache_backend_config_defaults_to_filesystem_under_project_cache()
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
fn artifact_cache_backend_config_accepts_foyer_and_capacity_env()
-> Result<(), Box<dyn std::error::Error>> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(&|key| match key {
        ARTIFACT_CACHE_BACKEND_ENV => Some("foyer".to_owned()),
        ARTIFACT_CACHE_ROOT_ENV => Some("/tmp/wendao-artifacts".to_owned()),
        ARTIFACT_CACHE_MEMORY_BYTES_ENV => Some("1048576".to_owned()),
        ARTIFACT_CACHE_STORAGE_BYTES_ENV => Some("8388608".to_owned()),
        _ => None,
    })?;

    assert_eq!(config.kind(), ArtifactCacheBackendKind::Foyer);
    assert_eq!(config.root(), std::path::Path::new("/tmp/wendao-artifacts"));
    assert_eq!(config.memory_capacity_bytes(), 1_048_576);
    assert_eq!(config.storage_capacity_bytes(), 8_388_608);
    Ok(())
}
