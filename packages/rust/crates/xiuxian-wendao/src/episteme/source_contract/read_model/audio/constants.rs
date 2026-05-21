//! Audio read-model constants.

pub(super) const AUDIO_EVIDENCE_SOURCE_OBJECT_KIND: &str =
    "episteme_audio_evidence.transcript_source";
pub(super) const AUDIO_EVIDENCE_SEGMENT_OBJECT_KIND: &str =
    "episteme_audio_evidence.transcript_segment";
pub(super) const AUDIO_EVIDENCE_SEGMENT_SOURCE_RELATION_KIND: &str =
    "episteme_audio_evidence.transcript_segment.has_source";
pub(super) const AUDIO_EVIDENCE_REVIEW_PROJECTION_ID: &str =
    "episteme_audio_evidence.review_seed.v1";
pub(super) const AUDIO_EVIDENCE_REVIEW_PROJECTION_REVISION: &str =
    "episteme_audio_evidence.review_seed.v1";
pub(super) const AUDIO_EVIDENCE_CONFIDENCE_SOURCE: &str = "audio_org_evidence_projection";
pub(super) const AUDIO_REVIEWED_CLAIM_OBJECT_KIND: &str =
    "episteme_audio_reviewed_claim.semantic_claim";
pub(super) const AUDIO_REVIEWED_CLAIM_SEGMENT_RELATION_KIND: &str =
    "episteme_audio_reviewed_claim.claim.has_evidence_segment";
pub(super) const AUDIO_REVIEWED_CLAIM_PROJECTION_ID: &str = "episteme_audio_reviewed_claim.seed.v1";
pub(super) const AUDIO_REVIEWED_CLAIM_PROJECTION_REVISION: &str =
    "episteme_audio_reviewed_claim.seed.v1";
pub(super) const AUDIO_REVIEWED_CLAIM_CONFIDENCE_SOURCE: &str = "audio_reviewed_semantic_claim";
pub(super) const STATUS_ACTIVE: &str = "active";
pub(super) const STATUS_REVIEW_REQUIRED: &str = "review-required";
pub(super) const STATUS_PROMOTION_CANDIDATE: &str = "promotion-candidate";
