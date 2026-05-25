use xiuxian_db_store::artifact_cache::ContentAddressedFilesystemBlobCache;
use xiuxian_wendao_episteme::{
    EpistemeOntologyArtifactBundleIdentity, EpistemeOntologyArtifactBundleKind,
    restore_episteme_ontology_artifact_bundle, write_episteme_ontology_artifact_bundle,
};

#[test]
fn ontology_artifact_bundle_roundtrips_run_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache_root = temp.path().join("cache");
    let source = temp.path().join("source-run");
    let target = temp.path().join("restored-run");
    std::fs::create_dir_all(source.join("review"))?;
    std::fs::write(source.join("reasoning_packet.json"), br#"{"rows":2}"#)?;
    std::fs::write(source.join("review").join("ledger.org"), b"* Review\n")?;
    let cache = ContentAddressedFilesystemBlobCache::new(cache_root.as_path());
    let identity = EpistemeOntologyArtifactBundleIdentity {
        kind: EpistemeOntologyArtifactBundleKind::ReasoningProjection,
        source_digest: "source-contract".to_owned(),
        profile_digest: "bootstrap-v1".to_owned(),
        run_digest: "run-20260524".to_owned(),
    };

    let write = write_episteme_ontology_artifact_bundle(&cache, &identity, source.as_path())?;
    assert_eq!(write.artifact_key.namespace().as_str(), "ontology");
    assert_eq!(
        write.artifact_key.kind().as_storage_component(),
        "ontology-reasoning-projection"
    );
    assert!(!write.replaced);
    assert!(write.byte_len > 0);

    let restore = restore_episteme_ontology_artifact_bundle(&cache, &identity, target.as_path())?
        .ok_or_else(|| std::io::Error::other("bundle should be cached"))?;
    assert_eq!(restore.artifact_key, write.artifact_key);
    assert_eq!(restore.byte_len, write.byte_len);
    assert_eq!(
        std::fs::read(target.join("reasoning_packet.json"))?,
        br#"{"rows":2}"#
    );
    assert_eq!(
        std::fs::read(target.join("review").join("ledger.org"))?,
        b"* Review\n"
    );

    Ok(())
}

#[test]
fn ontology_artifact_bundle_restore_reports_cache_miss() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path().join("cache"));
    let identity = EpistemeOntologyArtifactBundleIdentity {
        kind: EpistemeOntologyArtifactBundleKind::CandidateReadModel,
        source_digest: "source-contract".to_owned(),
        profile_digest: "candidate-v1".to_owned(),
        run_digest: "missing-run".to_owned(),
    };

    let restore =
        restore_episteme_ontology_artifact_bundle(&cache, &identity, temp.path().join("target"))?;

    assert!(restore.is_none());
    Ok(())
}
