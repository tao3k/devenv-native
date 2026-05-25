use super::{
    AgentArtifactKeyParts, ArtifactKeyComponent, ArtifactKind, AttachmentArtifactKeyParts,
    OntologyArtifactKeyParts, agent_artifact_key, attachment_artifact_key, ontology_artifact_key,
};

#[test]
fn artifact_key_rejects_unsafe_path_components() {
    assert!(ArtifactKeyComponent::new("source_digest", "../source").is_err());
    assert!(ArtifactKeyComponent::new("source_digest", "source/path").is_err());
    assert!(ArtifactKeyComponent::new("source_digest", "..").is_err());
    assert!(ArtifactKeyComponent::new("source_digest", "").is_err());
}

#[test]
fn custom_artifact_kind_uses_validated_storage_component() -> Result<(), Box<dyn std::error::Error>>
{
    let kind = ArtifactKind::custom("medical-audio-window")?;

    assert_eq!(kind.as_storage_component(), "medical-audio-window");
    assert!(ArtifactKind::custom("medical/audio/window").is_err());

    Ok(())
}

#[test]
fn agent_artifact_kinds_use_stable_storage_components() {
    assert_eq!(
        ArtifactKind::AgentEvidencePack.as_storage_component(),
        "agent-evidence-pack"
    );
    assert_eq!(
        ArtifactKind::OrgProjection.as_storage_component(),
        "org-projection"
    );
    assert_eq!(
        ArtifactKind::JsonProjection.as_storage_component(),
        "json-projection"
    );
    assert_eq!(
        ArtifactKind::TabularProjection.as_storage_component(),
        "tabular-projection"
    );
    assert_eq!(
        ArtifactKind::PromptContextPack.as_storage_component(),
        "prompt-context-pack"
    );
}

#[test]
fn attachment_artifact_kinds_use_stable_storage_components() {
    assert_eq!(
        ArtifactKind::AudioChunk.as_storage_component(),
        "audio-chunk"
    );
    assert_eq!(
        ArtifactKind::AttachmentSourcePayload.as_storage_component(),
        "attachment-source-payload"
    );
    assert_eq!(
        ArtifactKind::PdfPageRaster.as_storage_component(),
        "pdf-page-raster"
    );
    assert_eq!(
        ArtifactKind::OcrRegionCrop.as_storage_component(),
        "ocr-region-crop"
    );
    assert_eq!(ArtifactKind::VlmAtlas.as_storage_component(), "vlm-atlas");
    assert_eq!(
        ArtifactKind::ArrowIpcBatch.as_storage_component(),
        "arrow-ipc-batch"
    );
}

#[test]
fn ontology_artifact_kinds_use_stable_storage_components() {
    assert_eq!(
        ArtifactKind::OntologyRegistrySnapshot.as_storage_component(),
        "ontology-registry-snapshot"
    );
    assert_eq!(
        ArtifactKind::OntologyCandidatePacket.as_storage_component(),
        "ontology-candidate-packet"
    );
    assert_eq!(
        ArtifactKind::OntologyCandidateReadModel.as_storage_component(),
        "ontology-candidate-read-model"
    );
    assert_eq!(
        ArtifactKind::OntologyRdfDraft.as_storage_component(),
        "ontology-rdf-draft"
    );
    assert_eq!(
        ArtifactKind::OntologyPromotionReviewPacket.as_storage_component(),
        "ontology-promotion-review-packet"
    );
    assert_eq!(
        ArtifactKind::OntologyReasoningProjection.as_storage_component(),
        "ontology-reasoning-projection"
    );
}

#[test]
fn agent_artifact_key_uses_agent_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let key = agent_artifact_key(AgentArtifactKeyParts {
        kind: ArtifactKind::AgentEvidencePack,
        source_digest: "org-md-json-source".to_owned(),
        profile_digest: "prompt-pack-v1".to_owned(),
        shard_digest: "frontier-0001".to_owned(),
    })?;

    assert_eq!(key.namespace().as_str(), "agent");
    assert_eq!(key.kind().as_storage_component(), "agent-evidence-pack");
    assert_eq!(key.source_digest().as_str(), "org-md-json-source");
    assert_eq!(key.profile_digest().as_str(), "prompt-pack-v1");
    assert_eq!(key.shard_digest().as_str(), "frontier-0001");
    Ok(())
}

#[test]
fn attachment_artifact_key_uses_attachment_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let key = attachment_artifact_key(AttachmentArtifactKeyParts {
        kind: ArtifactKind::PdfPageRaster,
        source_digest: "source-pdf".to_owned(),
        profile_digest: "dpi-300-profile-v1".to_owned(),
        shard_digest: "page-0001".to_owned(),
    })?;

    assert_eq!(key.namespace().as_str(), "attachment");
    assert_eq!(key.kind().as_storage_component(), "pdf-page-raster");
    assert_eq!(key.source_digest().as_str(), "source-pdf");
    assert_eq!(key.profile_digest().as_str(), "dpi-300-profile-v1");
    assert_eq!(key.shard_digest().as_str(), "page-0001");
    Ok(())
}

#[test]
fn ontology_artifact_key_uses_ontology_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let key = ontology_artifact_key(OntologyArtifactKeyParts {
        kind: ArtifactKind::OntologyCandidateReadModel,
        source_digest: "source-contract".to_owned(),
        profile_digest: "candidate-compiler-v1".to_owned(),
        shard_digest: "run-20260524".to_owned(),
    })?;

    assert_eq!(key.namespace().as_str(), "ontology");
    assert_eq!(
        key.kind().as_storage_component(),
        "ontology-candidate-read-model"
    );
    assert_eq!(key.source_digest().as_str(), "source-contract");
    assert_eq!(key.profile_digest().as_str(), "candidate-compiler-v1");
    assert_eq!(key.shard_digest().as_str(), "run-20260524");
    Ok(())
}
