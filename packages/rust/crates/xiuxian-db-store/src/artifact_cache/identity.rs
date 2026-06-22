//! Artifact identity types for cache-safe storage paths.

use crate::artifact_cache::ArtifactCacheError;

const MAX_ARTIFACT_COMPONENT_BYTES: usize = 256;

/// One safe storage-path component used by artifact cache keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtifactKeyComponent(String);

impl ArtifactKeyComponent {
    /// Build a validated artifact key component.
    ///
    /// Components are restricted to portable ASCII path-segment characters so
    /// cache implementations can safely map keys onto filesystem paths without
    /// path traversal or platform-specific separators.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the component is empty, too long, is
    /// `.` or `..`, or contains a path separator or unsupported character.
    pub fn new(field: &'static str, value: impl Into<String>) -> Result<Self, ArtifactCacheError> {
        let value = value.into();
        validate_artifact_key_component(field, &value)?;
        Ok(Self(value))
    }

    /// Borrow the validated component as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Artifact categories that can share the same cache contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    /// Encoded or decoded audio shard content.
    AudioChunk,
    /// Source attachment bytes or format-normalized attachment payload.
    AttachmentSourcePayload,
    /// One rendered PDF page raster.
    PdfPageRaster,
    /// One OCR recovery region crop.
    OcrRegionCrop,
    /// A visual-language-model atlas assembled from multiple regions.
    VlmAtlas,
    /// Arrow IPC bytes that represent an extracted resource batch.
    ArrowIpcBatch,
    /// Agent-ready prompt, evidence, and context bytes.
    AgentEvidencePack,
    /// Org source or read-model projection bytes.
    OrgProjection,
    /// JSON source or read-model projection bytes.
    JsonProjection,
    /// CSV or TSV tabular projection bytes.
    TabularProjection,
    /// Prompt context pack bytes prepared for an LLM call.
    PromptContextPack,
    /// Ontology registry snapshot bytes admitted from source contracts.
    OntologyRegistrySnapshot,
    /// Review-gated ontology candidate packet or ledger bytes.
    OntologyCandidatePacket,
    /// Ontology candidate Arrow or Parquet read-model projection bytes.
    OntologyCandidateReadModel,
    /// RDF draft bytes generated for ontology review.
    OntologyRdfDraft,
    /// Promotion review packet bytes for ontology candidates.
    OntologyPromotionReviewPacket,
    /// Structural facts, reasoning packets, or schedule-plan projection bytes.
    OntologyReasoningProjection,
    /// Project-specific artifact kind that still uses safe key components.
    Custom(ArtifactKeyComponent),
}

impl ArtifactKind {
    /// Create a custom artifact kind.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when the custom kind cannot be used as a
    /// safe storage component.
    pub fn custom(value: impl Into<String>) -> Result<Self, ArtifactCacheError> {
        Ok(Self::Custom(ArtifactKeyComponent::new(
            "artifact_kind",
            value,
        )?))
    }

    /// Return the storage component for this artifact kind.
    #[must_use]
    pub fn as_storage_component(&self) -> &str {
        match self {
            Self::AudioChunk => "audio-chunk",
            Self::AttachmentSourcePayload => "attachment-source-payload",
            Self::PdfPageRaster => "pdf-page-raster",
            Self::OcrRegionCrop => "ocr-region-crop",
            Self::VlmAtlas => "vlm-atlas",
            Self::ArrowIpcBatch => "arrow-ipc-batch",
            Self::AgentEvidencePack => "agent-evidence-pack",
            Self::OrgProjection => "org-projection",
            Self::JsonProjection => "json-projection",
            Self::TabularProjection => "tabular-projection",
            Self::PromptContextPack => "prompt-context-pack",
            Self::OntologyRegistrySnapshot => "ontology-registry-snapshot",
            Self::OntologyCandidatePacket => "ontology-candidate-packet",
            Self::OntologyCandidateReadModel => "ontology-candidate-read-model",
            Self::OntologyRdfDraft => "ontology-rdf-draft",
            Self::OntologyPromotionReviewPacket => "ontology-promotion-review-packet",
            Self::OntologyReasoningProjection => "ontology-reasoning-projection",
            Self::Custom(component) => component.as_str(),
        }
    }
}

/// Stable identity for a cached attachment or document extraction artifact.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactKey {
    namespace: ArtifactKeyComponent,
    kind: ArtifactKind,
    source_digest: ArtifactKeyComponent,
    profile_digest: ArtifactKeyComponent,
    shard_digest: ArtifactKeyComponent,
}

/// Named request used to build an [`ArtifactKey`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactKeyParts {
    /// Cache namespace such as an attachment, OCR, or audio shard domain.
    pub namespace: String,
    /// Artifact kind.
    pub kind: ArtifactKind,
    /// Source content digest component.
    pub source_digest: String,
    /// Profile or planner digest component.
    pub profile_digest: String,
    /// Shard or materialized region digest component.
    pub shard_digest: String,
}

impl ArtifactKey {
    /// Build a stable artifact key from validated content and profile facts.
    ///
    /// The key intentionally stores digests as opaque validated components.
    /// Digest computation remains owned by callers that know the source,
    /// profile, and shard materialization contract.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCacheError`] when any caller-provided component is not
    /// safe for content-addressed storage.
    pub fn from_parts(parts: ArtifactKeyParts) -> Result<Self, ArtifactCacheError> {
        Ok(Self {
            namespace: ArtifactKeyComponent::new("namespace", parts.namespace)?,
            kind: parts.kind,
            source_digest: ArtifactKeyComponent::new("source_digest", parts.source_digest)?,
            profile_digest: ArtifactKeyComponent::new("profile_digest", parts.profile_digest)?,
            shard_digest: ArtifactKeyComponent::new("shard_digest", parts.shard_digest)?,
        })
    }

    /// Cache namespace such as an attachment, OCR, or audio shard domain.
    #[must_use]
    pub fn namespace(&self) -> &ArtifactKeyComponent {
        &self.namespace
    }

    /// Artifact kind.
    #[must_use]
    pub fn kind(&self) -> &ArtifactKind {
        &self.kind
    }

    /// Source content digest component.
    #[must_use]
    pub fn source_digest(&self) -> &ArtifactKeyComponent {
        &self.source_digest
    }

    /// Profile or planner digest component.
    #[must_use]
    pub fn profile_digest(&self) -> &ArtifactKeyComponent {
        &self.profile_digest
    }

    /// Shard or materialized region digest component.
    #[must_use]
    pub fn shard_digest(&self) -> &ArtifactKeyComponent {
        &self.shard_digest
    }
}

fn validate_artifact_key_component(
    field: &'static str,
    value: &str,
) -> Result<(), ArtifactCacheError> {
    if value.is_empty() {
        return Err(ArtifactCacheError::invalid_component(
            field,
            value,
            "component must not be empty",
        ));
    }
    if value == "." || value == ".." {
        return Err(ArtifactCacheError::invalid_component(
            field,
            value,
            "component must not be a relative directory marker",
        ));
    }
    if value.len() > MAX_ARTIFACT_COMPONENT_BYTES {
        return Err(ArtifactCacheError::invalid_component(
            field,
            value,
            "component is too long",
        ));
    }
    if value
        .as_bytes()
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Ok(());
    }
    Err(ArtifactCacheError::invalid_component(
        field,
        value,
        "component must contain only ASCII letters, digits, dots, dashes, or underscores",
    ))
}
