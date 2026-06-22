use crate::duckdb::{
    DatasetOntologyDuckDbMaterializer, DatasetOntologyRuntimeMaterializationRequest,
    build_dataset_ontology_wendaograph_extension_proof_request_batches,
    build_dataset_ontology_wendaograph_quality_request_batches,
    encode_dataset_ontology_materialization_app_metadata,
    summarize_dataset_ontology_wendaograph_extension_proof_response,
    summarize_dataset_ontology_wendaograph_quality_response,
};
use xiuxian_julia_core::integration_support::{
    build_wendaograph_ontology_extension_proof_arrow_request,
    build_wendaograph_ontology_extension_proof_flight_request_batch,
};

use super::support::{
    healthcare_contract_mapping_sql, healthcare_parent_registry_batches, healthcare_source_tables,
    string_record_batch,
};
use super::{TestResult, in_memory_search_duckdb_runtime};

#[tokio::test]
async fn duckdb_runtime_materializer_builds_wendaograph_extension_proof_request() -> TestResult {
    let temp = tempfile::tempdir()?;
    let materializer = DatasetOntologyDuckDbMaterializer::from_runtime(
        in_memory_search_duckdb_runtime(temp.path()),
    )
    .map_err(std::io::Error::other)?;

    let request = DatasetOntologyRuntimeMaterializationRequest::new(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        healthcare_source_tables()?,
        healthcare_contract_mapping_sql()?,
    )
    .map_err(std::io::Error::other)?;

    let materialization = materializer
        .materialize_with_read_model_batches(request)
        .await
        .map_err(std::io::Error::other)?;
    let quality_batches =
        build_dataset_ontology_wendaograph_quality_request_batches(&materialization)
            .map_err(std::io::Error::other)?;

    assert_eq!(quality_batches.row_counts(), [8, 6, 1]);

    let (parent_object_types, parent_link_types) = healthcare_parent_registry_batches()?;
    let extension_batches = build_dataset_ontology_wendaograph_extension_proof_request_batches(
        &materialization,
        parent_object_types,
        parent_link_types,
    )
    .map_err(std::io::Error::other)?;

    assert_eq!(extension_batches.row_counts(), [8, 6, 1, 4, 3]);

    let arrow_request = build_wendaograph_ontology_extension_proof_arrow_request(
        &extension_batches,
        "episteme://30_Healthcare/10_LongTermCare",
        "https://wendao.ai/ontology/ltc#",
    )
    .map_err(std::io::Error::other)?;
    assert!(
        arrow_request
            .payload_byte_sizes()
            .into_iter()
            .all(|size| size > 0)
    );

    let flight_batch =
        build_wendaograph_ontology_extension_proof_flight_request_batch(&arrow_request)
            .map_err(std::io::Error::other)?;
    assert_eq!(flight_batch.num_rows(), 1);
    assert_eq!(flight_batch.num_columns(), 7);

    Ok(())
}

#[test]
fn dataset_ontology_wendaograph_proof_summary_gates_promotion_evidence() -> TestResult {
    let (_, proof_rows) = string_record_batch(
        &[
            "check_id",
            "status",
            "severity",
            "subject",
            "message",
            "source_section",
        ],
        &[
            &[
                "object_graph_component_count",
                "pass",
                "info",
                "semantic_read_model",
                "object graph is connected",
                "semantic_relations",
            ],
            &[
                "extension_read_model_relation_type_consistent",
                "pass",
                "info",
                "ltc.service_item.supports_encounter",
                "extension relation type is consistent",
                "semantic_relations.kind",
            ],
            &[
                "extension_new_link_evidence_anchored",
                "pass",
                "info",
                "ltc.service_item.supports_encounter",
                "new extension term has accepted evidence anchor",
                "semantic_relations.evidence_status",
            ],
        ],
    )?;

    let evidence = summarize_dataset_ontology_wendaograph_extension_proof_response(&proof_rows)
        .map_err(std::io::Error::other)?;

    assert!(evidence.passed());
    assert!(evidence.promotion_candidate);
    assert_eq!(evidence.row_count, 3);
    assert_eq!(evidence.pass_count, 3);
    assert_eq!(evidence.failure_count, 0);
    assert_eq!(evidence.warning_count, 0);
    assert_eq!(evidence.missing_required_checks, Vec::<String>::new());
    assert_eq!(
        evidence.required_checks_present,
        vec![
            "extension_read_model_relation_type_consistent".to_string(),
            "extension_new_link_evidence_anchored".to_string(),
        ]
    );

    Ok(())
}

#[test]
fn dataset_ontology_wendaograph_quality_summary_gates_promotion_evidence() -> TestResult {
    let (_, proof_rows) = string_record_batch(
        &["check_id", "status", "severity"],
        &[
            &["semantic_objects_present", "pass", "info"],
            &["semantic_relation_source_known", "pass", "info"],
            &["semantic_relation_target_known", "pass", "info"],
            &["semantic_projection_state_present", "pass", "info"],
        ],
    )?;

    let evidence = summarize_dataset_ontology_wendaograph_quality_response(&proof_rows)
        .map_err(std::io::Error::other)?;

    assert!(evidence.passed());
    assert_eq!(evidence.row_count, 4);
    assert_eq!(evidence.pass_count, 4);
    assert_eq!(evidence.failure_count, 0);
    assert_eq!(evidence.missing_required_checks, Vec::<String>::new());
    assert_eq!(
        evidence.required_checks_present,
        vec![
            "semantic_objects_present".to_string(),
            "semantic_relation_source_known".to_string(),
            "semantic_relation_target_known".to_string(),
            "semantic_projection_state_present".to_string(),
        ]
    );

    Ok(())
}

#[test]
fn dataset_ontology_wendaograph_proof_summary_rejects_incomplete_or_failed_evidence() -> TestResult
{
    let (_, missing_required_rows) = string_record_batch(
        &["check_id", "status", "severity"],
        &[&["object_graph_component_count", "pass", "info"]],
    )?;
    let missing =
        summarize_dataset_ontology_wendaograph_extension_proof_response(&missing_required_rows)
            .map_err(std::io::Error::other)?;

    assert!(!missing.passed());
    assert_eq!(missing.failure_count, 0);
    assert_eq!(
        missing.missing_required_checks,
        vec![
            "extension_read_model_relation_type_consistent".to_string(),
            "extension_new_link_evidence_anchored".to_string(),
        ]
    );

    let (_, failed_rows) = string_record_batch(
        &["check_id", "status", "severity"],
        &[
            &[
                "extension_read_model_relation_type_consistent",
                "fail",
                "error",
            ],
            &["extension_new_link_evidence_anchored", "pass", "info"],
        ],
    )?;
    let failed = summarize_dataset_ontology_wendaograph_extension_proof_response(&failed_rows)
        .map_err(std::io::Error::other)?;

    assert!(!failed.passed());
    assert_eq!(failed.failure_count, 1);
    assert_eq!(failed.missing_required_checks, Vec::<String>::new());

    Ok(())
}

#[tokio::test]
async fn dataset_ontology_app_metadata_carries_materialization_and_proof_evidence() -> TestResult {
    let temp = tempfile::tempdir()?;
    let materializer = DatasetOntologyDuckDbMaterializer::from_runtime(
        in_memory_search_duckdb_runtime(temp.path()),
    )
    .map_err(std::io::Error::other)?;
    let request = DatasetOntologyRuntimeMaterializationRequest::new(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        healthcare_source_tables()?,
        healthcare_contract_mapping_sql()?,
    )
    .map_err(std::io::Error::other)?;
    let materialization = materializer
        .materialize_with_read_model_batches(request)
        .await
        .map_err(std::io::Error::other)?;
    let (_, proof_rows) = string_record_batch(
        &["check_id", "status", "severity"],
        &[
            &[
                "extension_read_model_relation_type_consistent",
                "pass",
                "info",
            ],
            &["extension_new_link_evidence_anchored", "pass", "info"],
        ],
    )?;
    let proof = summarize_dataset_ontology_wendaograph_extension_proof_response(&proof_rows)
        .map_err(std::io::Error::other)?;

    let metadata =
        encode_dataset_ontology_materialization_app_metadata(&materialization.report, Some(&proof))
            .map_err(std::io::Error::other)?;
    let json: serde_json::Value = serde_json::from_slice(&metadata)?;

    assert_eq!(
        json["schemaVersion"],
        "xiuxian_wendao.dataset_ontology_materialization_app_metadata.v1"
    );
    assert_eq!(
        json["materialization"]["contractId"],
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(
        json["materialization"]["mappingId"],
        "healthcare.synthetic_care_delivery.v1"
    );
    assert_eq!(json["materialization"]["sourceTableCount"], 4);
    assert_eq!(json["wendaographProof"]["promotionCandidate"], true);
    assert_eq!(json["wendaographProof"]["failureCount"], 0);
    assert_eq!(
        json["wendaographProof"]["requiredChecksPresent"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    Ok(())
}
