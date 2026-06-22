use super::support::sample_audio_evidence_request;
#[cfg(feature = "julia")]
use crate::episteme::source_contract::support::decode_single_arrow_batch;
use crate::episteme::source_contract::support::{i64_column, string_column, table};
#[cfg(feature = "julia")]
use xiuxian_julia_core::integration_support::{
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
};
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::build_episteme_wendaograph_quality_request_batches;
use xiuxian_wendao::episteme::{
    materialize_episteme_audio_evidence_review_seed,
    validate_episteme_read_model_relation_endpoints,
};

#[test]
fn episteme_audio_evidence_review_seed_materializes_review_required_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let request = sample_audio_evidence_request();

    let materialization = materialize_episteme_audio_evidence_review_seed(&request)?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [3, 2, 1]);

    let objects = table(&materialization, "semantic_objects");
    assert_eq!(
        string_column(objects, "kind").value(0),
        "episteme_audio_evidence.transcript_source"
    );
    assert_eq!(
        string_column(objects, "kind").value(1),
        "episteme_audio_evidence.transcript_segment"
    );
    assert_eq!(string_column(objects, "status").value(1), "review-required");
    assert!(
        string_column(objects, "verification_required_json")
            .value(1)
            .contains("human_transcript_review")
    );

    let raw_text = "neutral synthetic transcript segment";
    for column_name in ["title", "verification_evidence_json", "source_path"] {
        let column = string_column(objects, column_name);
        for row_index in 0..objects.num_rows() {
            assert!(
                !column.value(row_index).contains(raw_text),
                "{column_name} must not embed raw transcript text"
            );
        }
    }

    let relations = table(&materialization, "semantic_relations");
    assert_eq!(
        string_column(relations, "kind").value(0),
        "episteme_audio_evidence.transcript_segment.has_source"
    );
    assert_eq!(
        string_column(relations, "source").value(0),
        "audio-org-segment:001"
    );
    assert_eq!(
        string_column(relations, "target").value(0),
        "audio-org-source:001"
    );

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_audio_evidence.review_seed.v1"
    );
    assert_eq!(i64_column(projection, "source_object_count").value(0), 3);

    Ok(())
}

#[test]
#[cfg(feature = "julia")]
fn episteme_audio_evidence_review_seed_builds_wendaograph_quality_request()
-> Result<(), Box<dyn std::error::Error>> {
    let materialization =
        materialize_episteme_audio_evidence_review_seed(&sample_audio_evidence_request())?;
    let quality_batches = build_episteme_wendaograph_quality_request_batches(&materialization)?;

    assert_eq!(quality_batches.row_counts(), [3, 2, 1]);

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&quality_batches)?;
    let bundle = build_wendaograph_ontology_read_model_quality_flight_request_batch(&request)?;
    assert_eq!(bundle.num_rows(), 1);
    assert!(
        request
            .payload_byte_sizes()
            .into_iter()
            .all(|size| size > 0)
    );

    let objects = decode_single_arrow_batch(request.semantic_objects_payload.as_slice())?;
    assert_eq!(
        string_column(&objects, "kind").value(1),
        "episteme_audio_evidence.transcript_segment"
    );
    assert_eq!(
        string_column(&objects, "status").value(1),
        "review-required"
    );
    assert!(
        !string_column(&objects, "verification_evidence_json")
            .value(1)
            .contains("neutral synthetic transcript segment"),
        "WendaoGraph quality request must not carry raw transcript text"
    );

    let relations = decode_single_arrow_batch(request.semantic_relations_payload.as_slice())?;
    assert_eq!(
        string_column(&relations, "kind").value(0),
        "episteme_audio_evidence.transcript_segment.has_source"
    );

    Ok(())
}
#[test]
fn episteme_audio_evidence_review_seed_rejects_duplicate_segments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = sample_audio_evidence_request();
    request.segments[1].evidence_segment_id = request.segments[0].evidence_segment_id.clone();

    let Err(error) = materialize_episteme_audio_evidence_review_seed(&request) else {
        return Err("duplicate audio segment ids should fail".into());
    };
    assert!(
        error
            .to_string()
            .contains("duplicate audio evidence segment id")
    );

    Ok(())
}

#[test]
fn episteme_audio_evidence_review_seed_rejects_source_hash_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = sample_audio_evidence_request();
    request.segments[0].source_sha256 = "sha256:mismatch".to_string();

    let Err(error) = materialize_episteme_audio_evidence_review_seed(&request) else {
        return Err("source hash mismatch should fail".into());
    };
    assert!(error.to_string().contains("source hash does not match"));

    Ok(())
}

#[test]
fn episteme_audio_evidence_review_seed_rejects_empty_transcript()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = sample_audio_evidence_request();
    request.segments[0].transcript_text.clear();

    let Err(error) = materialize_episteme_audio_evidence_review_seed(&request) else {
        return Err("empty transcript evidence should fail".into());
    };
    assert!(error.to_string().contains("transcript_text is empty"));

    Ok(())
}
