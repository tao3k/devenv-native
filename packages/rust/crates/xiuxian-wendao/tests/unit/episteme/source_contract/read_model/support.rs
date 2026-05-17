use xiuxian_wendao::episteme::{
    EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow,
    EpistemeAudioEvidenceSourceRow, EpistemeAudioReviewedClaimObjectKind,
    EpistemeAudioReviewedClaimReadModelRequest, EpistemeAudioReviewedClaimRow,
};

pub(super) fn sample_audio_evidence_request() -> EpistemeAudioEvidenceReadModelRequest {
    let source = EpistemeAudioEvidenceSourceRow {
        contract_version: "xiuxian_wendao.audio_org_evidence_source.v1".to_string(),
        evidence_source_id: "audio-org-source:001".to_string(),
        source_path: "audio/full-source.mp3".to_string(),
        source_sha256: "sha256:source".to_string(),
        shard_profile: "audio-transcript-shard-v1".to_string(),
        task_profile: "audio-transcript".to_string(),
        backend_profile: "qwen3-asr-1.7b-mlx".to_string(),
        ledger_sha256: "sha256:ledger".to_string(),
        segment_count: 2,
    };
    let segments = vec![
        sample_audio_evidence_segment("001", 0, 0, 30_000, "0000000000000000"),
        sample_audio_evidence_segment("002", 1, 30_000, 30_000, "0000000003000000"),
    ];
    EpistemeAudioEvidenceReadModelRequest::new(
        "episteme://synthetic/audio-review",
        source,
        segments,
    )
}

pub(super) fn sample_audio_reviewed_claim_request() -> EpistemeAudioReviewedClaimReadModelRequest {
    EpistemeAudioReviewedClaimReadModelRequest::new(
        sample_audio_evidence_request(),
        vec![EpistemeAudioReviewedClaimRow {
            claim_id: "audio-reviewed-claim:001".to_string(),
            evidence_segment_id: "audio-org-segment:001".to_string(),
            ontology_subject: "episteme://synthetic/entity/a".to_string(),
            ontology_predicate: "episteme://synthetic/relation/mentions".to_string(),
            ontology_object: "episteme://synthetic/entity/b".to_string(),
            object_kind: EpistemeAudioReviewedClaimObjectKind::Entity,
            reviewer_id: "review-gate:synthetic".to_string(),
            reviewed_at: "2026-05-16".to_string(),
            evidence_quote_sha256: "sha256:evidence-quote".to_string(),
            review_note_sha256: Some("sha256:review-note".to_string()),
            confidence: 0.94,
        }],
    )
}

fn sample_audio_evidence_segment(
    suffix: &str,
    chunk_index: u32,
    start_ms: u64,
    duration_ms: u64,
    reading_order_key: &str,
) -> EpistemeAudioEvidenceSegmentRow {
    EpistemeAudioEvidenceSegmentRow {
        contract_version: "xiuxian_wendao.audio_org_evidence_segment.v1".to_string(),
        evidence_source_id: "audio-org-source:001".to_string(),
        evidence_segment_id: format!("audio-org-segment:{suffix}"),
        shard_element_id: format!("audio-shard-{suffix}"),
        result_element_id: format!("audio-result-{suffix}"),
        source_name: "synthetic-audio.mp3".to_string(),
        chunk_index,
        start_ms,
        duration_ms,
        end_ms: start_ms + duration_ms,
        source_sha256: "sha256:source".to_string(),
        shard_sha256: format!("sha256:shard-{suffix}"),
        reading_order_key: reading_order_key.to_string(),
        confidence: Some(0.92),
        transcript_sha256: format!("sha256:transcript-{suffix}"),
        transcript_text: format!("neutral synthetic transcript segment {suffix}"),
    }
}
