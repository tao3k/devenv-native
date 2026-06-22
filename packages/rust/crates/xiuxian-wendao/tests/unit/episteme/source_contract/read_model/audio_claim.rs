use super::support::sample_audio_reviewed_claim_request;
#[cfg(feature = "julia")]
use crate::episteme::source_contract::support::decode_single_arrow_batch;
use crate::episteme::source_contract::support::{string_column, table};
#[cfg(feature = "julia")]
use xiuxian_julia_core::integration_support::build_wendaograph_ontology_read_model_quality_arrow_request;
#[cfg(feature = "julia")]
use xiuxian_wendao::episteme::build_episteme_wendaograph_quality_request_batches;
use xiuxian_wendao::episteme::{
    materialize_episteme_audio_reviewed_claim_seed, validate_episteme_read_model_relation_endpoints,
};

#[test]
fn episteme_audio_reviewed_claim_seed_materializes_promotion_candidate_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let request = sample_audio_reviewed_claim_request();

    let materialization = materialize_episteme_audio_reviewed_claim_seed(&request)?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [4, 3, 1]);

    let objects = table(&materialization, "semantic_objects");
    assert_eq!(
        string_column(objects, "kind").value(3),
        "episteme_audio_reviewed_claim.semantic_claim"
    );
    assert_eq!(
        string_column(objects, "status").value(3),
        "promotion-candidate"
    );
    assert!(
        string_column(objects, "verification_required_json")
            .value(3)
            .contains("rdf_promotion_gate")
    );
    assert!(
        string_column(objects, "verification_evidence_json")
            .value(3)
            .contains("ontology_subject:episteme://synthetic/entity/a")
    );

    let raw_text = "neutral synthetic transcript segment";
    for column_name in ["verification_evidence_json", "source_path"] {
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
        string_column(relations, "kind").value(2),
        "episteme_audio_reviewed_claim.claim.has_evidence_segment"
    );
    assert_eq!(
        string_column(relations, "source").value(2),
        "audio-reviewed-claim:001"
    );
    assert_eq!(
        string_column(relations, "target").value(2),
        "audio-org-segment:001"
    );

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_audio_reviewed_claim.seed.v1"
    );

    Ok(())
}

#[test]
#[cfg(feature = "julia")]
fn episteme_audio_reviewed_claim_seed_builds_wendaograph_quality_request()
-> Result<(), Box<dyn std::error::Error>> {
    let materialization =
        materialize_episteme_audio_reviewed_claim_seed(&sample_audio_reviewed_claim_request())?;
    let quality_batches = build_episteme_wendaograph_quality_request_batches(&materialization)?;

    assert_eq!(quality_batches.row_counts(), [4, 3, 1]);

    let request = build_wendaograph_ontology_read_model_quality_arrow_request(&quality_batches)?;
    let objects = decode_single_arrow_batch(request.semantic_objects_payload.as_slice())?;
    assert_eq!(
        string_column(&objects, "kind").value(3),
        "episteme_audio_reviewed_claim.semantic_claim"
    );
    assert_eq!(
        string_column(&objects, "status").value(3),
        "promotion-candidate"
    );
    assert!(
        !string_column(&objects, "verification_evidence_json")
            .value(3)
            .contains("neutral synthetic transcript segment"),
        "WendaoGraph quality request must not carry raw transcript text"
    );

    Ok(())
}

#[test]
fn episteme_audio_reviewed_claim_seed_rejects_unknown_segment()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = sample_audio_reviewed_claim_request();
    request.claims[0].evidence_segment_id = "audio-org-segment:missing".to_string();

    let Err(error) = materialize_episteme_audio_reviewed_claim_seed(&request) else {
        return Err("unknown evidence segment should fail".into());
    };
    assert!(
        error
            .to_string()
            .contains("references unknown evidence segment")
    );

    Ok(())
}

#[test]
fn episteme_audio_reviewed_claim_seed_rejects_duplicate_claims()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = sample_audio_reviewed_claim_request();
    request.claims.push(request.claims[0].clone());

    let Err(error) = materialize_episteme_audio_reviewed_claim_seed(&request) else {
        return Err("duplicate claim ids should fail".into());
    };
    assert!(
        error
            .to_string()
            .contains("duplicate audio reviewed claim id")
    );

    Ok(())
}

#[test]
fn episteme_audio_reviewed_claim_seed_rejects_empty_reviewer()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = sample_audio_reviewed_claim_request();
    request.claims[0].reviewer_id.clear();

    let Err(error) = materialize_episteme_audio_reviewed_claim_seed(&request) else {
        return Err("empty reviewer should fail".into());
    };
    assert!(error.to_string().contains("reviewer_id is empty"));

    Ok(())
}
