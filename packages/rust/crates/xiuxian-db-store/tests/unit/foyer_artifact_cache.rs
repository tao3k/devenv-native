use super::{
    ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ArtifactKey, ArtifactKeyParts,
    ArtifactKind, FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig,
};

fn sample_key() -> Result<ArtifactKey, Box<dyn std::error::Error>> {
    Ok(ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: "attachment".to_owned(),
        kind: ArtifactKind::AudioChunk,
        source_digest: "source-abc".to_owned(),
        profile_digest: "profile-qwen3".to_owned(),
        shard_digest: "shard-0001".to_owned(),
    })?)
}

fn test_config(root: &std::path::Path) -> FoyerArtifactBlobCacheConfig {
    FoyerArtifactBlobCacheConfig::new(root, 4 * 1024 * 1024, 16 * 1024 * 1024)
}

#[test]
fn foyer_blob_cache_roundtrips_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = FoyerArtifactBlobCache::from_config(test_config(temp.path()))?;
    let key = sample_key()?;

    assert!(cache.read(&key)?.is_none());
    let first = cache.write(&key, ArtifactBlobWrite::new(b"first transcript chunk"))?;
    assert_eq!(first.byte_len(), 22);
    assert!(!first.replaced());
    assert_eq!(
        cache.read(&key)?.map(ArtifactBlobRead::into_bytes),
        Some(b"first transcript chunk".to_vec())
    );

    let second = cache.write(&key, ArtifactBlobWrite::new(b"second"))?;
    assert_eq!(second.byte_len(), 6);
    assert!(second.replaced());
    assert_eq!(
        cache.read(&key)?.map(ArtifactBlobRead::into_bytes),
        Some(b"second".to_vec())
    );

    assert!(cache.remove(&key)?);
    assert!(cache.read(&key)?.is_none());
    cache.close()?;

    Ok(())
}

#[test]
#[ignore = "pending Foyer close/flush/reopen lifecycle closure under the synchronous ArtifactBlobCache wrapper"]
fn foyer_blob_cache_reopens_persisted_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let key = sample_key()?;

    {
        let cache = FoyerArtifactBlobCache::from_config(test_config(temp.path()))?;
        cache.write(&key, ArtifactBlobWrite::new(b"restart reusable chunk"))?;
        cache.close()?;
    }

    let reopened = FoyerArtifactBlobCache::from_config(test_config(temp.path()))?;
    assert_eq!(
        reopened.read(&key)?.map(ArtifactBlobRead::into_bytes),
        Some(b"restart reusable chunk".to_vec())
    );
    reopened.close()?;

    Ok(())
}
