use super::{
    AgentArtifactKeyParts, ArtifactKind, ContentAddressedFilesystemBlobCache, agent_artifact_key,
    read_through_artifact_bytes,
};

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
    assert_eq!(second.bytes(), evidence_pack);
    assert_eq!(second.write_outcome(), None);
    assert_eq!(build_count, 1);
    Ok(())
}
