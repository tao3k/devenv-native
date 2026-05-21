use std::fs;

use xiuxian_wendao::episteme::{
    EpistemeAudioClaimPromotionProposalRequest, EpistemeAudioEvidenceReadModelRequest,
    EpistemeAudioEvidenceSegmentRow, EpistemeAudioEvidenceSourceRow,
    EpistemeAudioReviewedClaimObjectKind, EpistemeAudioReviewedClaimReadModelRequest,
    EpistemeAudioReviewedClaimRow, write_episteme_audio_claim_promotion_proposal,
};

#[test]
fn episteme_audio_claim_promotion_proposal_writes_review_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let request = EpistemeAudioClaimPromotionProposalRequest::new(
        "audio-proposal-001",
        sample_audio_reviewed_claim_request(),
    );

    let report = write_episteme_audio_claim_promotion_proposal(&request, temp.path())?;

    assert_eq!(report.claim_count, 1);
    assert_eq!(report.evidence_segment_count, 1);
    assert!(!report.rdf_materialization_performed);
    assert!(!report.ontology_source_write_performed);
    assert!(!report.raw_transcript_promotion_allowed);
    assert!(report.claims_path.is_file());
    assert!(report.receipt_path.is_file());

    let claims = fs::read_to_string(&report.claims_path)?;
    assert!(claims.contains("audio-reviewed-claim:001"));
    assert!(claims.contains("episteme://synthetic/entity/a"));
    assert!(claims.contains("promotion-candidate"));
    assert!(!claims.contains("neutral synthetic transcript segment"));

    let receipt = fs::read_to_string(&report.receipt_path)?;
    assert!(receipt.contains("xiuxian_wendao.episteme_audio_claim_promotion_proposal.v1"));
    assert!(receipt.contains("\"rdf_materialization_performed\": false"));
    assert!(receipt.contains("\"ontology_source_write_performed\": false"));
    assert!(!receipt.contains("neutral synthetic transcript segment"));

    Ok(())
}

#[test]
fn episteme_audio_claim_promotion_proposal_rejects_unsafe_id()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let request = EpistemeAudioClaimPromotionProposalRequest::new(
        "../unsafe",
        sample_audio_reviewed_claim_request(),
    );

    let Err(error) = write_episteme_audio_claim_promotion_proposal(&request, temp.path()) else {
        return Err("unsafe proposal id should fail".into());
    };
    assert!(error.to_string().contains("invalid run id"));

    Ok(())
}

#[test]
fn episteme_audio_claim_promotion_proposal_rejects_invalid_claims_before_write()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let mut reviewed_claims = sample_audio_reviewed_claim_request();
    reviewed_claims.claims[0].evidence_segment_id = "audio-org-segment:missing".to_string();
    let request =
        EpistemeAudioClaimPromotionProposalRequest::new("audio-proposal-invalid", reviewed_claims);

    let Err(error) = write_episteme_audio_claim_promotion_proposal(&request, temp.path()) else {
        return Err("invalid claim should fail".into());
    };
    assert!(
        error
            .to_string()
            .contains("references unknown evidence segment")
    );
    assert!(!temp.path().join("audio-proposal-invalid").exists());

    Ok(())
}

fn sample_audio_reviewed_claim_request() -> EpistemeAudioReviewedClaimReadModelRequest {
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

fn sample_audio_evidence_request() -> EpistemeAudioEvidenceReadModelRequest {
    let source = EpistemeAudioEvidenceSourceRow {
        contract_version: "xiuxian_wendao.audio_org_evidence_source.v1".to_string(),
        evidence_source_id: "audio-org-source:001".to_string(),
        source_path: "audio/full-source.mp3".to_string(),
        source_sha256: "sha256:source".to_string(),
        shard_profile: "audio-transcript-shard-v1".to_string(),
        task_profile: "audio-transcript".to_string(),
        backend_profile: "qwen3-asr-1.7b-mlx".to_string(),
        ledger_sha256: "sha256:ledger".to_string(),
        segment_count: 1,
    };
    EpistemeAudioEvidenceReadModelRequest::new(
        "episteme://synthetic/audio-review",
        source,
        vec![sample_audio_evidence_segment()],
    )
}

fn sample_audio_evidence_segment() -> EpistemeAudioEvidenceSegmentRow {
    EpistemeAudioEvidenceSegmentRow {
        contract_version: "xiuxian_wendao.audio_org_evidence_segment.v1".to_string(),
        evidence_source_id: "audio-org-source:001".to_string(),
        evidence_segment_id: "audio-org-segment:001".to_string(),
        shard_element_id: "audio-shard-001".to_string(),
        result_element_id: "audio-result-001".to_string(),
        source_name: "synthetic-audio.mp3".to_string(),
        chunk_index: 0,
        start_ms: 0,
        duration_ms: 30_000,
        end_ms: 30_000,
        source_sha256: "sha256:source".to_string(),
        shard_sha256: "sha256:shard-001".to_string(),
        reading_order_key: "0000000000000000".to_string(),
        confidence: Some(0.92),
        transcript_sha256: "sha256:transcript-001".to_string(),
        transcript_text: "neutral synthetic transcript segment".to_string(),
    }
}
