//! Audio transcript evidence review-seed materialization.

use sha2::{Digest, Sha256};

use super::constants::{
    AUDIO_EVIDENCE_CONFIDENCE_SOURCE, AUDIO_EVIDENCE_REVIEW_PROJECTION_ID,
    AUDIO_EVIDENCE_REVIEW_PROJECTION_REVISION, AUDIO_EVIDENCE_SEGMENT_OBJECT_KIND,
    AUDIO_EVIDENCE_SEGMENT_SOURCE_RELATION_KIND, AUDIO_EVIDENCE_SOURCE_OBJECT_KIND, STATUS_ACTIVE,
    STATUS_REVIEW_REQUIRED,
};
use super::rows::{EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow};
use super::validation::validate_audio_evidence_review_request;
use crate::episteme::source_contract::facade::read_model::facade::tables::{
    semantic_objects_batch, semantic_projection_state_batch, semantic_relations_batch,
};
use crate::episteme::source_contract::facade::read_model::facade::{
    EpistemeError, EpistemeReadModelMaterialization, EpistemeReadModelTable, OBJECTS_TABLE,
    PROJECTION_STATE_TABLE, RECORDED_AT, RECORDED_BY, RELATIONS_TABLE, STALENESS_FRESH,
    SemanticObjectRow, SemanticProjectionStateRow, SemanticRelationRow, json_array, owners_json,
    semantic_relation_counts,
};

/// # Errors
///
/// Returns an error when evidence rows are incomplete, duplicate, detached from
/// the source row, or cannot be encoded into the stable Arrow read-model table
/// schemas.
pub fn materialize_episteme_audio_evidence_review_seed(
    request: &EpistemeAudioEvidenceReadModelRequest,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    validate_audio_evidence_review_request(request)?;
    let source_revision = audio_evidence_review_revision(request);
    let relation_rows = audio_evidence_relation_rows(request, source_revision.as_str());
    let object_rows =
        audio_evidence_object_rows(request, &relation_rows, source_revision.as_str())?;
    let projection_rows = audio_evidence_projection_rows(
        &object_rows,
        request.source.source_path.as_str(),
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
pub(super) fn audio_evidence_object_rows(
    request: &EpistemeAudioEvidenceReadModelRequest,
    relations: &[SemanticRelationRow],
    source_revision: &str,
) -> Result<Vec<SemanticObjectRow>, EpistemeError> {
    let relation_counts = semantic_relation_counts(relations);
    let source = &request.source;
    let mut rows = Vec::with_capacity(request.segments.len() + 1);
    rows.push(SemanticObjectRow {
        id: source.evidence_source_id.clone(),
        kind: AUDIO_EVIDENCE_SOURCE_OBJECT_KIND,
        title: source.source_path.clone(),
        status: STATUS_REVIEW_REQUIRED,
        confidence_score: 1.0,
        confidence_source: AUDIO_EVIDENCE_CONFIDENCE_SOURCE,
        owner_count: 1,
        owners_json: owners_json(request.owner_scope.as_str())?,
        provenance_source: source.source_path.clone(),
        provenance_recorded_by: RECORDED_BY,
        provenance_recorded_at: RECORDED_AT,
        verification_required_json: json_array([
            "audio_evidence_review",
            "source_fingerprint",
            "ledger_fingerprint",
        ])?,
        verification_evidence_json: json_array([
            format!("contract_version:{}", source.contract_version),
            format!("source_sha256:{}", source.source_sha256),
            format!("ledger_sha256:{}", source.ledger_sha256),
            format!("shard_profile:{}", source.shard_profile),
            format!("task_profile:{}", source.task_profile),
            format!("backend_profile:{}", source.backend_profile),
            format!("segment_count:{}", source.segment_count),
        ])?,
        relation_count: i64::try_from(
            relation_counts
                .get(source.evidence_source_id.as_str())
                .copied()
                .unwrap_or_default(),
        )
        .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
        source_path: source.source_path.clone(),
        read_model_source_revision: source_revision.to_string(),
        read_model_projection_revision: AUDIO_EVIDENCE_REVIEW_PROJECTION_REVISION,
        read_model_projection_staleness: STALENESS_FRESH,
    });

    for segment in &request.segments {
        rows.push(SemanticObjectRow {
            id: segment.evidence_segment_id.clone(),
            kind: AUDIO_EVIDENCE_SEGMENT_OBJECT_KIND,
            title: audio_segment_title(segment),
            status: STATUS_REVIEW_REQUIRED,
            confidence_score: segment.confidence.unwrap_or(0.0),
            confidence_source: AUDIO_EVIDENCE_CONFIDENCE_SOURCE,
            owner_count: 1,
            owners_json: owners_json(request.owner_scope.as_str())?,
            provenance_source: source.source_path.clone(),
            provenance_recorded_by: RECORDED_BY,
            provenance_recorded_at: RECORDED_AT,
            verification_required_json: json_array([
                "audio_evidence_review",
                "human_transcript_review",
                "segment_fingerprint",
            ])?,
            verification_evidence_json: json_array([
                format!("contract_version:{}", segment.contract_version),
                format!("shard_element_id:{}", segment.shard_element_id),
                format!("result_element_id:{}", segment.result_element_id),
                format!("source_sha256:{}", segment.source_sha256),
                format!("shard_sha256:{}", segment.shard_sha256),
                format!("transcript_sha256:{}", segment.transcript_sha256),
                format!("reading_order_key:{}", segment.reading_order_key),
                format!("start_ms:{}", segment.start_ms),
                format!("duration_ms:{}", segment.duration_ms),
                format!("end_ms:{}", segment.end_ms),
                format!("backend_profile:{}", source.backend_profile),
            ])?,
            relation_count: i64::try_from(
                relation_counts
                    .get(segment.evidence_segment_id.as_str())
                    .copied()
                    .unwrap_or_default(),
            )
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
            source_path: format!("{}#{}", source.source_path, segment.reading_order_key),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: AUDIO_EVIDENCE_REVIEW_PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        });
    }
    Ok(rows)
}

pub(super) fn audio_evidence_relation_rows(
    request: &EpistemeAudioEvidenceReadModelRequest,
    source_revision: &str,
) -> Vec<SemanticRelationRow> {
    request
        .segments
        .iter()
        .map(|segment| SemanticRelationRow {
            source: segment.evidence_segment_id.clone(),
            kind: AUDIO_EVIDENCE_SEGMENT_SOURCE_RELATION_KIND,
            target: request.source.evidence_source_id.clone(),
            source_path: format!(
                "{}#{}",
                request.source.source_path, segment.reading_order_key
            ),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: AUDIO_EVIDENCE_REVIEW_PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        })
        .collect()
}

fn audio_evidence_projection_rows(
    object_rows: &[SemanticObjectRow],
    source_path: &str,
    source_revision: &str,
) -> Result<Vec<SemanticProjectionStateRow>, EpistemeError> {
    Ok(vec![SemanticProjectionStateRow {
        projection: AUDIO_EVIDENCE_REVIEW_PROJECTION_ID,
        status: STATUS_ACTIVE,
        source_revision: source_revision.to_string(),
        current_source_revision: source_revision.to_string(),
        projection_revision: AUDIO_EVIDENCE_REVIEW_PROJECTION_REVISION,
        staleness: STALENESS_FRESH,
        source_object_count: i64::try_from(object_rows.len())
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
        source_objects_json: json_array(object_rows.iter().map(|row| row.id.as_str()))?,
        source_path: source_path.to_string(),
    }])
}

pub(super) fn audio_evidence_review_revision(
    request: &EpistemeAudioEvidenceReadModelRequest,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(request.owner_scope.as_bytes());
    hasher.update(request.source.evidence_source_id.as_bytes());
    hasher.update(request.source.source_path.as_bytes());
    hasher.update(request.source.source_sha256.as_bytes());
    hasher.update(request.source.ledger_sha256.as_bytes());
    for segment in &request.segments {
        hasher.update(segment.evidence_segment_id.as_bytes());
        hasher.update(segment.shard_element_id.as_bytes());
        hasher.update(segment.result_element_id.as_bytes());
        hasher.update(segment.source_sha256.as_bytes());
        hasher.update(segment.shard_sha256.as_bytes());
        hasher.update(segment.reading_order_key.as_bytes());
        hasher.update(segment.transcript_sha256.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn audio_segment_title(segment: &EpistemeAudioEvidenceSegmentRow) -> String {
    format!(
        "{} chunk {} {}-{}ms",
        segment.source_name, segment.chunk_index, segment.start_ms, segment.end_ms
    )
}
