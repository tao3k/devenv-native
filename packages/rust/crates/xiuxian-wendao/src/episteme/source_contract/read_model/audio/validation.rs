//! Audio read-model request validation.

use std::collections::BTreeSet;

use super::rows::{
    EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow,
    EpistemeAudioEvidenceSourceRow, EpistemeAudioReviewedClaimReadModelRequest,
    EpistemeAudioReviewedClaimRow,
};
use crate::episteme::source_contract::facade::read_model::facade::EpistemeError;

pub(super) fn validate_audio_evidence_review_request(
    request: &EpistemeAudioEvidenceReadModelRequest,
) -> Result<(), EpistemeError> {
    let mut errors = Vec::new();
    if request.owner_scope.trim().is_empty() {
        errors.push("audio evidence review owner_scope is empty".to_string());
    }
    validate_audio_source_row(&request.source, &mut errors);
    if request.segments.is_empty() {
        errors.push("audio evidence review requires at least one segment".to_string());
    }
    if usize::try_from(request.source.segment_count).ok() != Some(request.segments.len()) {
        errors.push(format!(
            "audio evidence source segment_count {} does not match {} segment rows",
            request.source.segment_count,
            request.segments.len()
        ));
    }

    let mut segment_ids = BTreeSet::new();
    let mut shard_ids = BTreeSet::new();
    for segment in &request.segments {
        validate_audio_segment_row(segment, request.source.source_sha256.as_str(), &mut errors);
        if segment.evidence_source_id != request.source.evidence_source_id {
            errors.push(format!(
                "audio evidence segment `{}` references source `{}`, expected `{}`",
                segment.evidence_segment_id,
                segment.evidence_source_id,
                request.source.evidence_source_id
            ));
        }
        if !segment_ids.insert(segment.evidence_segment_id.as_str()) {
            errors.push(format!(
                "duplicate audio evidence segment id `{}`",
                segment.evidence_segment_id
            ));
        }
        if !shard_ids.insert(segment.shard_element_id.as_str()) {
            errors.push(format!(
                "duplicate audio evidence shard id `{}`",
                segment.shard_element_id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::ReadModel(errors.join("; ")))
    }
}

fn validate_audio_source_row(source: &EpistemeAudioEvidenceSourceRow, errors: &mut Vec<String>) {
    require_non_empty(
        source.contract_version.as_str(),
        "audio evidence source contract_version",
        errors,
    );
    require_non_empty(
        source.evidence_source_id.as_str(),
        "audio evidence source evidence_source_id",
        errors,
    );
    require_non_empty(
        source.source_path.as_str(),
        "audio evidence source source_path",
        errors,
    );
    require_non_empty(
        source.source_sha256.as_str(),
        "audio evidence source source_sha256",
        errors,
    );
    require_non_empty(
        source.shard_profile.as_str(),
        "audio evidence source shard_profile",
        errors,
    );
    require_non_empty(
        source.task_profile.as_str(),
        "audio evidence source task_profile",
        errors,
    );
    require_non_empty(
        source.backend_profile.as_str(),
        "audio evidence source backend_profile",
        errors,
    );
    require_non_empty(
        source.ledger_sha256.as_str(),
        "audio evidence source ledger_sha256",
        errors,
    );
}

fn validate_audio_segment_row(
    segment: &EpistemeAudioEvidenceSegmentRow,
    source_sha256: &str,
    errors: &mut Vec<String>,
) {
    require_non_empty(
        segment.contract_version.as_str(),
        "audio evidence segment contract_version",
        errors,
    );
    require_non_empty(
        segment.evidence_source_id.as_str(),
        "audio evidence segment evidence_source_id",
        errors,
    );
    require_non_empty(
        segment.evidence_segment_id.as_str(),
        "audio evidence segment evidence_segment_id",
        errors,
    );
    require_non_empty(
        segment.shard_element_id.as_str(),
        "audio evidence segment shard_element_id",
        errors,
    );
    require_non_empty(
        segment.result_element_id.as_str(),
        "audio evidence segment result_element_id",
        errors,
    );
    require_non_empty(
        segment.source_name.as_str(),
        "audio evidence segment source_name",
        errors,
    );
    require_non_empty(
        segment.shard_sha256.as_str(),
        "audio evidence segment shard_sha256",
        errors,
    );
    require_non_empty(
        segment.reading_order_key.as_str(),
        "audio evidence segment reading_order_key",
        errors,
    );
    require_non_empty(
        segment.transcript_sha256.as_str(),
        "audio evidence segment transcript_sha256",
        errors,
    );
    require_non_empty(
        segment.transcript_text.as_str(),
        "audio evidence segment transcript_text",
        errors,
    );
    if segment.source_sha256 != source_sha256 {
        errors.push(format!(
            "audio evidence segment `{}` source hash does not match source row",
            segment.evidence_segment_id
        ));
    }
    if segment
        .start_ms
        .checked_add(segment.duration_ms)
        .is_none_or(|end_ms| end_ms != segment.end_ms)
    {
        errors.push(format!(
            "audio evidence segment `{}` has inconsistent time range",
            segment.evidence_segment_id
        ));
    }
    if let Some(confidence) = segment.confidence
        && (!confidence.is_finite() || !(0.0..=1.0).contains(&confidence))
    {
        errors.push(format!(
            "audio evidence segment `{}` confidence must be between 0.0 and 1.0",
            segment.evidence_segment_id
        ));
    }
}

fn require_non_empty(value: &str, field: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} is empty"));
    }
}

pub(super) fn validate_audio_reviewed_claim_request(
    request: &EpistemeAudioReviewedClaimReadModelRequest,
) -> Result<(), EpistemeError> {
    let mut errors = Vec::new();
    if request.claims.is_empty() {
        errors.push("audio reviewed claim seed requires at least one claim".to_string());
    }
    let segment_ids = request
        .evidence
        .segments
        .iter()
        .map(|segment| segment.evidence_segment_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut claim_ids = BTreeSet::new();
    for claim in &request.claims {
        validate_audio_reviewed_claim_row(claim, &segment_ids, &mut errors);
        if !claim_ids.insert(claim.claim_id.as_str()) {
            errors.push(format!(
                "duplicate audio reviewed claim id `{}`",
                claim.claim_id
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::ReadModel(errors.join("; ")))
    }
}

fn validate_audio_reviewed_claim_row(
    claim: &EpistemeAudioReviewedClaimRow,
    segment_ids: &BTreeSet<&str>,
    errors: &mut Vec<String>,
) {
    require_non_empty(
        claim.claim_id.as_str(),
        "audio reviewed claim claim_id",
        errors,
    );
    require_non_empty(
        claim.evidence_segment_id.as_str(),
        "audio reviewed claim evidence_segment_id",
        errors,
    );
    require_non_empty(
        claim.ontology_subject.as_str(),
        "audio reviewed claim ontology_subject",
        errors,
    );
    require_non_empty(
        claim.ontology_predicate.as_str(),
        "audio reviewed claim ontology_predicate",
        errors,
    );
    require_non_empty(
        claim.ontology_object.as_str(),
        "audio reviewed claim ontology_object",
        errors,
    );
    require_non_empty(
        claim.reviewer_id.as_str(),
        "audio reviewed claim reviewer_id",
        errors,
    );
    require_non_empty(
        claim.reviewed_at.as_str(),
        "audio reviewed claim reviewed_at",
        errors,
    );
    require_non_empty(
        claim.evidence_quote_sha256.as_str(),
        "audio reviewed claim evidence_quote_sha256",
        errors,
    );
    if !segment_ids.contains(claim.evidence_segment_id.as_str()) {
        errors.push(format!(
            "audio reviewed claim `{}` references unknown evidence segment `{}`",
            claim.claim_id, claim.evidence_segment_id
        ));
    }
    if !claim.confidence.is_finite() || !(0.0..=1.0).contains(&claim.confidence) {
        errors.push(format!(
            "audio reviewed claim `{}` confidence must be between 0.0 and 1.0",
            claim.claim_id
        ));
    }
    if let Some(note_hash) = &claim.review_note_sha256 {
        require_non_empty(
            note_hash.as_str(),
            "audio reviewed claim review_note_sha256",
            errors,
        );
    }
}
