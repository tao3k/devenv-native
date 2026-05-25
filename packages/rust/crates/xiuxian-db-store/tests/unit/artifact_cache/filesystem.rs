use super::{
    ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobWrite, ContentAddressedFilesystemBlobCache,
    sample_key,
};

#[test]
fn filesystem_blob_cache_roundtrips_and_replaces_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let key = sample_key()?;

    assert!(!cache.contains(&key)?);
    assert!(cache.read(&key)?.is_none());

    let first = cache.write(&key, ArtifactBlobWrite::new(b"first transcript chunk"))?;
    assert_eq!(first.byte_len(), 22);
    assert!(!first.replaced());
    assert!(cache.contains(&key)?);
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
    assert!(!cache.remove(&key)?);
    assert!(cache.read(&key)?.is_none());

    Ok(())
}

#[test]
fn filesystem_blob_cache_path_is_content_addressed() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let key = sample_key()?;
    let path = cache.artifact_path(&key);

    assert!(
        path.ends_with("attachment/audio-chunk/source-abc/profile-qwen3/shard-0001/payload.bin")
    );

    Ok(())
}
