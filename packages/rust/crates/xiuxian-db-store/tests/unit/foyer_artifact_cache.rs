use super::{
    AgentArtifactKeyParts, ArtifactBlobCache, ArtifactBlobCacheBackend,
    ArtifactBlobCacheBackendConfig, ArtifactBlobRead, ArtifactBlobWrite, ArtifactCacheBackendKind,
    ArtifactKey, ArtifactKeyParts, ArtifactKind, FoyerArtifactBlobCache,
    FoyerArtifactBlobCacheConfig, agent_artifact_key, read_through_artifact_bytes,
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

fn agent_evidence_pack_key() -> Result<ArtifactKey, Box<dyn std::error::Error>> {
    Ok(agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::AgentEvidencePack,
        source_digest: "org-md-json-source".to_owned(),
        profile_digest: "prompt-pack-v1".to_owned(),
        shard_digest: "frontier-0001".to_owned(),
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

#[test]
fn foyer_agent_evidence_pack_reopens_cached_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let key = agent_evidence_pack_key()?;
    let evidence_pack = br##"{"schema":"xiuxian_wendao.agent_evidence_pack.v1","org":"* Task\n","markdown":"# Note\n","json":{"rows":3}}"##;
    let mut hit_count = 0;
    let mut read_bytes = 0;
    let mut build_count = 0;

    {
        let cache = FoyerArtifactBlobCache::from_config(test_config(temp.path()))?;
        let first = read_through_artifact_bytes(&cache, &key, || {
            build_count += 1;
            Ok(evidence_pack.to_vec())
        })?;
        assert!(first.cache_miss());
        assert_eq!(first.bytes(), evidence_pack);
        assert_eq!(
            first.write_outcome().map(|write| write.byte_len()),
            Some(evidence_pack.len())
        );
        let read = read_through_artifact_bytes(&cache, &key, || {
            build_count += 1;
            Ok(b"unexpected hot rebuild".to_vec())
        })?;
        assert!(read.cache_hit());
        hit_count += 1;
        read_bytes += read.byte_len();
        assert_eq!(read.bytes(), evidence_pack);
        cache.close()?;
    }

    let reopened = FoyerArtifactBlobCache::from_config(test_config(temp.path()))?;
    let read = read_through_artifact_bytes(&reopened, &key, || {
        build_count += 1;
        Ok(b"unexpected restart rebuild".to_vec())
    })?;
    assert!(read.cache_hit());
    hit_count += 1;
    read_bytes += read.byte_len();
    assert_eq!(read.bytes(), evidence_pack);
    reopened.close()?;

    assert_eq!(build_count, 1);
    assert_eq!(hit_count, 2);
    assert_eq!(read_bytes, evidence_pack.len() * 2);
    Ok(())
}

#[test]
fn foyer_memory_capacity_uses_artifact_byte_weight() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = FoyerArtifactBlobCache::from_config(
        FoyerArtifactBlobCacheConfig::new_with_runtime_workers(temp.path(), 512, 64 * 1024, 1)
            .with_memory_shards(1)
            .with_block_size_bytes(16 * 1024)
            .with_recover_concurrency(1)
            .with_flushers(1)
            .with_reclaimers(1),
    )?;

    for chunk_index in 0..8 {
        let key = ArtifactKey::from_parts(ArtifactKeyParts {
            namespace: "attachment".to_owned(),
            kind: ArtifactKind::AudioChunk,
            source_digest: "source-byte-weight".to_owned(),
            profile_digest: "profile-byte-weight".to_owned(),
            shard_digest: format!("shard-{chunk_index:04}"),
        })?;
        let payload = vec![chunk_index; 256];
        cache.write(&key, ArtifactBlobWrite::new(payload.as_slice()))?;
    }
    cache.close()?;

    assert!(
        cache.event_stats().evicted_entries() > 0,
        "byte-weighted memory capacity should evict oversized artifact payloads"
    );
    Ok(())
}

#[test]
fn foyer_blob_cache_drops_inside_tokio_runtime_worker() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let root = temp.path().to_path_buf();
    let key = sample_key()?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()?;

    runtime.block_on(async move {
        let cache = FoyerArtifactBlobCache::from_config(test_config(root.as_path()))?;
        cache.write(&key, ArtifactBlobWrite::new(b"async-context-drop"))?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    Ok(())
}

#[test]
fn foyer_backend_config_builds_artifact_cache_backend() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let config =
        ArtifactBlobCacheBackendConfig::from_root_and_lookup(temp.path(), &|key| match key {
            "WENDAO_ARTIFACT_CACHE_BACKEND" => Some("foyer".to_owned()),
            "WENDAO_ARTIFACT_CACHE_MEMORY_BYTES" => Some((4 * 1024 * 1024).to_string()),
            "WENDAO_ARTIFACT_CACHE_STORAGE_BYTES" => Some((16 * 1024 * 1024).to_string()),
            _ => None,
        })?;
    assert_eq!(config.kind(), ArtifactCacheBackendKind::Foyer);

    let backend = config.build()?;
    assert_eq!(backend.backend_name(), "foyer");
    let ArtifactBlobCacheBackend::Foyer(_) = &backend else {
        return Err("expected Foyer artifact backend".into());
    };
    let key = sample_key()?;
    backend.write(&key, ArtifactBlobWrite::new(b"backend selected"))?;
    assert_eq!(
        backend.read(&key)?.map(ArtifactBlobRead::into_bytes),
        Some(b"backend selected".to_vec())
    );
    backend.close()?;
    Ok(())
}
