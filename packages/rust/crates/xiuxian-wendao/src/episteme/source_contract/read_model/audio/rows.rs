//! Audio read-model request and row DTOs.

/// Request for compiling audio transcript evidence rows into review-required
/// semantic read-model seed batches.
///
/// Raw DTO boundary and stringly state boundary for audio evidence review seed
/// requests accepted from source-contract fixtures or promotion tooling.
#[derive(Debug, Clone, PartialEq)]
pub struct EpistemeAudioEvidenceReadModelRequest {
    /// Owner scope that will be recorded on emitted evidence objects.
    pub owner_scope: String,
    /// Source-level audio transcript evidence row.
    pub source: EpistemeAudioEvidenceSourceRow,
    /// Ordered segment-level transcript evidence rows.
    pub segments: Vec<EpistemeAudioEvidenceSegmentRow>,
}

impl EpistemeAudioEvidenceReadModelRequest {
    /// Create a request for audio transcript evidence review-seed
    /// materialization.
    #[must_use]
    pub fn new(
        owner_scope: impl Into<String>,
        source: EpistemeAudioEvidenceSourceRow,
        segments: Vec<EpistemeAudioEvidenceSegmentRow>,
    ) -> Self {
        Self {
            owner_scope: owner_scope.into(),
            source,
            segments,
        }
    }
}

/// Source-level audio transcript evidence row.
///
/// Raw DTO boundary and stringly state boundary for audio source evidence rows.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeAudioEvidenceSourceRow {
    /// Evidence source row contract version.
    pub contract_version: String,
    /// Stable source evidence id.
    pub evidence_source_id: String,
    /// Source path recorded by the audio transcript ledger.
    pub source_path: String,
    /// SHA-256 of the original source audio bytes.
    pub source_sha256: String,
    /// Audio shard profile used to generate the ledger.
    pub shard_profile: String,
    /// Logical task profile used to generate the ledger.
    pub task_profile: String,
    /// Backend profile used to generate transcript text.
    pub backend_profile: String,
    /// SHA-256 of the complete generated Org ledger.
    pub ledger_sha256: String,
    /// Number of projected segment rows expected for this source.
    pub segment_count: u32,
}

/// Segment-level audio transcript evidence row.
///
/// Raw DTO boundary: this struct mirrors rows emitted by the audio Org
/// projection contract, so primitive ids and offsets are kept stable at the
/// contract boundary and validated before read-model materialization.
#[derive(Debug, Clone, PartialEq)]
pub struct EpistemeAudioEvidenceSegmentRow {
    /// Evidence segment row contract version.
    pub contract_version: String,
    /// Parent source evidence id.
    pub evidence_source_id: String,
    /// Stable segment evidence id.
    pub evidence_segment_id: String,
    /// Shard element id from the generated ledger.
    pub shard_element_id: String,
    /// Result element id from the generated ledger.
    pub result_element_id: String,
    /// Source display name from the generated ledger.
    pub source_name: String,
    /// Zero-based chunk index.
    pub chunk_index: u32,
    /// Segment start offset in milliseconds.
    pub start_ms: u64,
    /// Segment duration in milliseconds.
    pub duration_ms: u64,
    /// Segment end offset in milliseconds.
    pub end_ms: u64,
    /// SHA-256 of the original source audio bytes.
    pub source_sha256: String,
    /// SHA-256 of the materialized audio shard.
    pub shard_sha256: String,
    /// Stable reading order key.
    pub reading_order_key: String,
    /// Optional model confidence, when provided by the model path.
    pub confidence: Option<f64>,
    /// SHA-256 of the transcript text.
    pub transcript_sha256: String,
    /// Raw transcript evidence text. This is validated but not embedded into
    /// semantic read-model objects.
    pub transcript_text: String,
}

/// Request for compiling reviewed audio semantic claims into
/// promotion-candidate semantic read-model seed batches.
///
/// Raw DTO boundary and stringly state boundary for reviewed audio claim seed
/// requests before RDF promotion.
#[derive(Debug, Clone, PartialEq)]
pub struct EpistemeAudioReviewedClaimReadModelRequest {
    /// Source and segment evidence that the reviewed claims cite.
    pub evidence: EpistemeAudioEvidenceReadModelRequest,
    /// Reviewed claims accepted by a human or deterministic review gate.
    pub claims: Vec<EpistemeAudioReviewedClaimRow>,
}

impl EpistemeAudioReviewedClaimReadModelRequest {
    /// Create a reviewed audio claim seed request.
    #[must_use]
    pub fn new(
        evidence: EpistemeAudioEvidenceReadModelRequest,
        claims: Vec<EpistemeAudioReviewedClaimRow>,
    ) -> Self {
        Self { evidence, claims }
    }
}

/// Reviewed semantic claim anchored to one audio evidence segment.
///
/// Raw DTO boundary and stringly state boundary: this struct mirrors reviewed
/// audio claim rows before RDF promotion, so primitive ids remain explicit at
/// the review contract boundary and are validated before materialization.
#[derive(Debug, Clone, PartialEq)]
pub struct EpistemeAudioReviewedClaimRow {
    /// Stable reviewed claim id.
    pub claim_id: String,
    /// Evidence segment id that supports this claim.
    pub evidence_segment_id: String,
    /// Ontology subject id or compact IRI selected by review.
    pub ontology_subject: String,
    /// Ontology predicate id or compact IRI selected by review.
    pub ontology_predicate: String,
    /// Ontology object value selected by review.
    pub ontology_object: String,
    /// Object value kind.
    pub object_kind: EpistemeAudioReviewedClaimObjectKind,
    /// Reviewer or deterministic review gate id.
    pub reviewer_id: String,
    /// Review timestamp recorded by the review surface.
    pub reviewed_at: String,
    /// SHA-256 of the supporting evidence quote or reviewed span.
    pub evidence_quote_sha256: String,
    /// Optional SHA-256 of the review note.
    pub review_note_sha256: Option<String>,
    /// Reviewer confidence after evidence inspection.
    pub confidence: f64,
}

/// Object value kind for a reviewed audio semantic claim.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EpistemeAudioReviewedClaimObjectKind {
    /// Object is another entity id or compact IRI.
    Entity,
    /// Object is a literal value.
    Literal,
    /// Object is a quantity-like literal.
    Quantity,
}

impl EpistemeAudioReviewedClaimObjectKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Literal => "literal",
            Self::Quantity => "quantity",
        }
    }
}
