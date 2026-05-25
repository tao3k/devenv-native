use super::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_STORAGE_BYTES_ENV, AgentArtifactKeyParts, ArtifactBlobCache,
    ArtifactBlobCacheBackendConfig, ArtifactBlobRead, ArtifactBlobWrite, ArtifactCacheBackendKind,
    ArtifactKey, ArtifactKeyComponent, ArtifactKeyParts, ArtifactKind, AttachmentArtifactKeyParts,
    ContentAddressedFilesystemBlobCache, OntologyArtifactKeyParts, agent_artifact_key,
    attachment_artifact_key, ontology_artifact_key, pack_artifact_directory,
    read_through_artifact_bytes, unpack_artifact_directory,
};

mod backend;
mod directory_bundle;
mod filesystem;
mod identity;
mod readthrough;

fn sample_key() -> Result<ArtifactKey, Box<dyn std::error::Error>> {
    Ok(ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: "attachment".to_owned(),
        kind: ArtifactKind::AudioChunk,
        source_digest: "source-abc".to_owned(),
        profile_digest: "profile-qwen3".to_owned(),
        shard_digest: "shard-0001".to_owned(),
    })?)
}
