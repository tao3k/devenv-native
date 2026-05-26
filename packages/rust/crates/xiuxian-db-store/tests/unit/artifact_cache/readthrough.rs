use super::{
    AgentArtifactKeyParts, ArtifactBlobCache, ArtifactBlobRead, ArtifactBlobReadStatus,
    ArtifactBlobWrite, ArtifactBlobWriteOutcome, ArtifactCacheError, ArtifactKey, ArtifactKind,
    ArtifactReadThroughStatus, ContentAddressedFilesystemBlobCache, agent_artifact_key,
    fetch_through_artifact_bytes, read_through_artifact_bytes,
};
use std::cell::Cell;

#[test]
fn agent_artifact_readthrough_builds_once_then_hits() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let key = agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::AgentEvidencePack,
        source_digest: "org-md-json-source".to_owned(),
        profile_digest: "prompt-pack-v1".to_owned(),
        shard_digest: "frontier-0001".to_owned(),
    })?;
    let evidence_pack = br##"{"schema":"xiuxian_wendao.agent_evidence_pack.v1","org":"* Task\n","markdown":"# Note\n","json":{"rows":3}}"##;
    let mut build_count = 0;

    let first = read_through_artifact_bytes(&cache, &key, || {
        build_count += 1;
        Ok(evidence_pack.to_vec())
    })?;
    assert!(first.cache_miss());
    assert!(!first.cache_hit());
    assert_eq!(first.status(), ArtifactReadThroughStatus::Miss);
    assert_eq!(first.backend_name(), "filesystem");
    assert_eq!(first.artifact_key(), Some(&key));
    assert_eq!(first.bytes(), evidence_pack);
    assert_eq!(
        first.write_outcome().map(|write| write.byte_len()),
        Some(evidence_pack.len())
    );

    let second = read_through_artifact_bytes(&cache, &key, || {
        build_count += 1;
        Ok(b"unexpected rebuild".to_vec())
    })?;
    assert!(second.cache_hit());
    assert!(!second.cache_miss());
    assert_eq!(second.status(), ArtifactReadThroughStatus::Hit);
    assert_eq!(second.backend_name(), "filesystem");
    assert_eq!(second.artifact_key(), Some(&key));
    assert_eq!(second.bytes(), evidence_pack);
    assert_eq!(second.write_outcome(), None);
    assert_eq!(build_count, 1);
    Ok(())
}

#[test]
fn fetchthrough_preserves_filesystem_receipts() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let key = agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::AgentEvidencePack,
        source_digest: "org-md-json-source".to_owned(),
        profile_digest: "prompt-pack-v2".to_owned(),
        shard_digest: "frontier-0003".to_owned(),
    })?;

    let first = fetch_through_artifact_bytes(&cache, &key, || Ok(b"evidence".to_vec()))?;
    assert_eq!(first.status(), ArtifactReadThroughStatus::Miss);
    assert_eq!(first.backend_name(), "filesystem");
    assert_eq!(first.artifact_key(), Some(&key));
    assert_eq!(first.write_outcome().map(|write| write.byte_len()), Some(8));

    let second = fetch_through_artifact_bytes(&cache, &key, || Ok(b"unexpected".to_vec()))?;
    assert_eq!(second.status(), ArtifactReadThroughStatus::Hit);
    assert_eq!(second.bytes(), b"evidence");
    assert_eq!(second.write_outcome(), None);
    Ok(())
}

#[test]
fn readthrough_reports_pressure_and_skips_write() -> Result<(), Box<dyn std::error::Error>> {
    let cache = ThrottledCache::default();
    let key = agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::AgentEvidencePack,
        source_digest: "org-source".to_owned(),
        profile_digest: "prompt-pack-v1".to_owned(),
        shard_digest: "frontier-0002".to_owned(),
    })?;

    let artifact = read_through_artifact_bytes(&cache, &key, || Ok(b"rebuilt".to_vec()))?;

    assert_eq!(artifact.status(), ArtifactReadThroughStatus::Throttled);
    assert!(artifact.cache_miss());
    assert!(artifact.cache_throttled());
    assert_eq!(artifact.backend_name(), "pressure-test");
    assert_eq!(artifact.artifact_key(), Some(&key));
    assert_eq!(artifact.bytes(), b"rebuilt");
    assert_eq!(artifact.write_outcome(), None);
    assert_eq!(cache.write_calls.get(), 0);
    Ok(())
}

#[derive(Default)]
struct ThrottledCache {
    write_calls: Cell<u32>,
}

impl ArtifactBlobCache for ThrottledCache {
    fn backend_name(&self) -> &'static str {
        "pressure-test"
    }

    fn contains(&self, _key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        Ok(false)
    }

    fn read(&self, _key: &ArtifactKey) -> Result<Option<ArtifactBlobRead>, ArtifactCacheError> {
        Ok(None)
    }

    fn read_with_status(
        &self,
        _key: &ArtifactKey,
    ) -> Result<ArtifactBlobReadStatus, ArtifactCacheError> {
        Ok(ArtifactBlobReadStatus::Throttled)
    }

    fn write(
        &self,
        _key: &ArtifactKey,
        value: ArtifactBlobWrite<'_>,
    ) -> Result<ArtifactBlobWriteOutcome, ArtifactCacheError> {
        self.write_calls.set(self.write_calls.get() + 1);
        Ok(ArtifactBlobWriteOutcome::new(value.byte_len(), false))
    }

    fn remove(&self, _key: &ArtifactKey) -> Result<bool, ArtifactCacheError> {
        Ok(false)
    }
}
