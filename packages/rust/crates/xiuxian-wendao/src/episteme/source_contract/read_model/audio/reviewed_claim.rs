//! Reviewed audio semantic-claim seed materialization.

use sha2::{Digest, Sha256};

use super::constants::{
    AUDIO_REVIEWED_CLAIM_CONFIDENCE_SOURCE, AUDIO_REVIEWED_CLAIM_OBJECT_KIND,
    AUDIO_REVIEWED_CLAIM_PROJECTION_ID, AUDIO_REVIEWED_CLAIM_PROJECTION_REVISION,
    AUDIO_REVIEWED_CLAIM_SEGMENT_RELATION_KIND, STATUS_ACTIVE, STATUS_PROMOTION_CANDIDATE,
};
use super::evidence::{
    audio_evidence_object_rows, audio_evidence_relation_rows, audio_evidence_review_revision,
};
use super::rows::{EpistemeAudioReviewedClaimReadModelRequest, EpistemeAudioReviewedClaimRow};
use super::validation::{
    validate_audio_evidence_review_request, validate_audio_reviewed_claim_request,
};
use crate::episteme::source_contract::facade::read_model::facade::tables::{
    semantic_objects_batch, semantic_projection_state_batch, semantic_relations_batch,
};
use crate::episteme::source_contract::facade::read_model::facade::{
    EpistemeError, EpistemeReadModelMaterialization, EpistemeReadModelTable, OBJECTS_TABLE,
    PROJECTION_STATE_TABLE, RECORDED_AT, RECORDED_BY, RELATIONS_TABLE, STALENESS_FRESH,
    SemanticObjectRow, SemanticProjectionStateRow, SemanticRelationRow, json_array, owners_json,
    semantic_relation_counts,
};

/// Compile reviewed audio semantic claims into graph-readable
/// promotion-candidate semantic read-model seed batches.
///
/// # Errors
///
/// Returns an error when evidence rows are invalid, reviewed claim rows are
/// incomplete, duplicate, detached from evidence segments, or cannot be encoded
/// into the stable Arrow read-model table schemas.
pub fn materialize_episteme_audio_reviewed_claim_seed(
    request: &EpistemeAudioReviewedClaimReadModelRequest,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    validate_audio_evidence_review_request(&request.evidence)?;
    validate_audio_reviewed_claim_request(request)?;
    let source_revision = audio_reviewed_claim_revision(request);
    let relation_rows = audio_reviewed_claim_relation_rows(request, source_revision.as_str());
    let object_rows =
        audio_reviewed_claim_object_rows(request, &relation_rows, source_revision.as_str())?;
    let projection_rows = audio_reviewed_claim_projection_rows(
        &object_rows,
        request.evidence.source.source_path.as_str(),
        source_revision.as_str(),
    )?;

    Ok(EpistemeReadModelMaterialization {
        source_revision,
        tables: vec![
            EpistemeReadModelTable::new(OBJECTS_TABLE, semantic_objects_batch(&object_rows)?),
            EpistemeReadModelTable::new(RELATIONS_TABLE, semantic_relations_batch(&relation_rows)?),
            EpistemeReadModelTable::new(
                PROJECTION_STATE_TABLE,
                semantic_projection_state_batch(&projection_rows)?,
            ),
        ],
    })
}
fn audio_reviewed_claim_object_rows(
    request: &EpistemeAudioReviewedClaimReadModelRequest,
    relations: &[SemanticRelationRow],
    source_revision: &str,
) -> Result<Vec<SemanticObjectRow>, EpistemeError> {
    let mut rows = audio_evidence_object_rows(
        &request.evidence,
        &audio_evidence_relation_rows(&request.evidence, source_revision),
        source_revision,
    )?;
    let relation_counts = semantic_relation_counts(relations);
    for claim in &request.claims {
        rows.push(SemanticObjectRow {
            id: claim.claim_id.clone(),
            kind: AUDIO_REVIEWED_CLAIM_OBJECT_KIND,
            title: audio_reviewed_claim_title(claim),
            status: STATUS_PROMOTION_CANDIDATE,
            confidence_score: claim.confidence,
            confidence_source: AUDIO_REVIEWED_CLAIM_CONFIDENCE_SOURCE,
            owner_count: 1,
            owners_json: owners_json(request.evidence.owner_scope.as_str())?,
            provenance_source: format!(
                "{}#{}",
                request.evidence.source.source_path, claim.evidence_segment_id
            ),
            provenance_recorded_by: RECORDED_BY,
            provenance_recorded_at: RECORDED_AT,
            verification_required_json: json_array([
                "reviewed_semantic_claim",
                "audio_evidence_segment",
                "rdf_promotion_gate",
            ])?,
            verification_evidence_json: audio_reviewed_claim_evidence_json(claim)?,
            relation_count: i64::try_from(
                relation_counts
                    .get(claim.claim_id.as_str())
                    .copied()
                    .unwrap_or_default(),
            )
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
            source_path: format!(
                "{}#{}",
                request.evidence.source.source_path, claim.evidence_segment_id
            ),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: AUDIO_REVIEWED_CLAIM_PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        });
    }
    Ok(rows)
}

fn audio_reviewed_claim_relation_rows(
    request: &EpistemeAudioReviewedClaimReadModelRequest,
    source_revision: &str,
) -> Vec<SemanticRelationRow> {
    let mut rows = audio_evidence_relation_rows(&request.evidence, source_revision);
    rows.extend(request.claims.iter().map(|claim| SemanticRelationRow {
        source: claim.claim_id.clone(),
        kind: AUDIO_REVIEWED_CLAIM_SEGMENT_RELATION_KIND,
        target: claim.evidence_segment_id.clone(),
        source_path: format!(
            "{}#{}",
            request.evidence.source.source_path, claim.evidence_segment_id
        ),
        read_model_source_revision: source_revision.to_string(),
        read_model_projection_revision: AUDIO_REVIEWED_CLAIM_PROJECTION_REVISION,
        read_model_projection_staleness: STALENESS_FRESH,
    }));
    rows
}

fn audio_reviewed_claim_projection_rows(
    object_rows: &[SemanticObjectRow],
    source_path: &str,
    source_revision: &str,
) -> Result<Vec<SemanticProjectionStateRow>, EpistemeError> {
    Ok(vec![SemanticProjectionStateRow {
        projection: AUDIO_REVIEWED_CLAIM_PROJECTION_ID,
        status: STATUS_ACTIVE,
        source_revision: source_revision.to_string(),
        current_source_revision: source_revision.to_string(),
        projection_revision: AUDIO_REVIEWED_CLAIM_PROJECTION_REVISION,
        staleness: STALENESS_FRESH,
        source_object_count: i64::try_from(object_rows.len())
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
        source_objects_json: json_array(object_rows.iter().map(|row| row.id.as_str()))?,
        source_path: source_path.to_string(),
    }])
}

fn audio_reviewed_claim_revision(request: &EpistemeAudioReviewedClaimReadModelRequest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(audio_evidence_review_revision(&request.evidence).as_bytes());
    for claim in &request.claims {
        hasher.update(claim.claim_id.as_bytes());
        hasher.update(claim.evidence_segment_id.as_bytes());
        hasher.update(claim.ontology_subject.as_bytes());
        hasher.update(claim.ontology_predicate.as_bytes());
        hasher.update(claim.ontology_object.as_bytes());
        hasher.update(claim.object_kind.as_str().as_bytes());
        hasher.update(claim.reviewer_id.as_bytes());
        hasher.update(claim.reviewed_at.as_bytes());
        hasher.update(claim.evidence_quote_sha256.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn audio_reviewed_claim_evidence_json(
    claim: &EpistemeAudioReviewedClaimRow,
) -> Result<String, EpistemeError> {
    let mut evidence = vec![
        format!("claim_id:{}", claim.claim_id),
        format!("evidence_segment_id:{}", claim.evidence_segment_id),
        format!("ontology_subject:{}", claim.ontology_subject),
        format!("ontology_predicate:{}", claim.ontology_predicate),
        format!("ontology_object:{}", claim.ontology_object),
        format!("ontology_object_kind:{}", claim.object_kind.as_str()),
        format!("reviewer_id:{}", claim.reviewer_id),
        format!("reviewed_at:{}", claim.reviewed_at),
        format!("evidence_quote_sha256:{}", claim.evidence_quote_sha256),
    ];
    if let Some(note_hash) = &claim.review_note_sha256 {
        evidence.push(format!("review_note_sha256:{note_hash}"));
    }
    json_array(evidence)
}

fn audio_reviewed_claim_title(claim: &EpistemeAudioReviewedClaimRow) -> String {
    format!(
        "{} {} {}",
        claim.ontology_subject, claim.ontology_predicate, claim.ontology_object
    )
}
