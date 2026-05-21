use crate::duckdb::{
    DatasetOntologyDuckDbMaterializer, DatasetOntologyRuntimeMaterializationRequest,
    DuckDbLocalRelationEngine,
};
use xiuxian_wendao_sql::dataset_ontology::{
    DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME, DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME, materialize_dataset_ontology_with_engine,
};

use super::support::{
    healthcare_arrow_ipc_specs, healthcare_contract_mapping_sql, healthcare_source_tables,
};
use super::{TestResult, in_memory_search_duckdb_runtime};

#[tokio::test]
async fn duckdb_materializes_healthcare_dataset_ontology_mapping_sql() -> TestResult {
    let temp = tempfile::tempdir()?;
    let engine =
        DuckDbLocalRelationEngine::from_runtime(in_memory_search_duckdb_runtime(temp.path()))
            .map_err(std::io::Error::other)?;

    let report = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &healthcare_contract_mapping_sql()?,
    )
    .await
    .map_err(std::io::Error::other)?;

    assert_eq!(report.execution_engine, "duckdb");
    assert!(report.passed(), "{:?}", report.validation_failures);
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME),
        Some(6)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME),
        Some(1)
    );
    assert_eq!(
        xiuxian_wendao_sql::LocalRelationEngine::relation_materialization_state(
            &engine,
            "raw_patients"
        )
        .map(xiuxian_wendao_sql::LocalRelationMaterializationState::as_str),
        Some("materialized")
    );
    Ok(())
}

#[tokio::test]
async fn duckdb_runtime_materializer_wraps_contract_metadata() -> TestResult {
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

    assert_eq!(
        request.contract_id(),
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(
        request.mapping_id(),
        "healthcare.synthetic_care_delivery.v1"
    );
    assert_eq!(request.source_table_count(), 4);

    let report = materializer
        .materialize(request)
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(
        report.contract_id,
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(report.mapping_id, "healthcare.synthetic_care_delivery.v1");
    assert_eq!(report.source_table_count, 4);
    assert!(report.passed(), "{:?}", report.materialization);
    assert_eq!(
        report
            .materialization
            .row_count_for(DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME),
        Some(8)
    );
    Ok(())
}

#[tokio::test]
async fn duckdb_runtime_materializer_reads_arrow_ipc_source_tables() -> TestResult {
    let temp = tempfile::tempdir()?;
    let materializer = DatasetOntologyDuckDbMaterializer::from_runtime(
        in_memory_search_duckdb_runtime(temp.path()),
    )
    .map_err(std::io::Error::other)?;
    let specs = healthcare_arrow_ipc_specs(temp.path())?;

    let request = DatasetOntologyRuntimeMaterializationRequest::from_arrow_ipc_streams(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        &specs,
        healthcare_contract_mapping_sql()?,
    )
    .map_err(std::io::Error::other)?;

    assert_eq!(request.source_table_count(), 4);

    let report = materializer
        .materialize(request)
        .await
        .map_err(std::io::Error::other)?;

    assert!(report.passed(), "{:?}", report.materialization);
    assert_eq!(
        report
            .materialization
            .row_count_for(DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report
            .materialization
            .row_count_for(DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME),
        Some(6)
    );
    Ok(())
}

#[tokio::test]
async fn duckdb_runtime_materializer_returns_semantic_read_model_batches() -> TestResult {
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

    assert!(materialization.report.passed());
    assert_eq!(materialization.read_model_tables.len(), 3);
    assert_eq!(
        materialization.read_model_tables[0].table_name(),
        DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME
    );
    assert_eq!(materialization.read_model_tables[0].row_count(), 8);
    assert_eq!(
        materialization.read_model_tables[1].table_name(),
        DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME
    );
    assert_eq!(materialization.read_model_tables[1].row_count(), 6);
    assert_eq!(
        materialization.read_model_tables[2].table_name(),
        DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME
    );
    assert_eq!(materialization.read_model_tables[2].row_count(), 1);
    Ok(())
}

#[test]
fn dataset_ontology_runtime_request_rejects_empty_identifiers() -> TestResult {
    let mapping_sql = healthcare_contract_mapping_sql()?;
    let source_tables = healthcare_source_tables()?;

    assert!(
        DatasetOntologyRuntimeMaterializationRequest::new(
            " ",
            "healthcare.synthetic_care_delivery.v1",
            source_tables.clone(),
            mapping_sql.clone(),
        )
        .is_err()
    );
    assert!(
        DatasetOntologyRuntimeMaterializationRequest::new(
            "healthcare.synthetic_care_delivery.contract.v1",
            "",
            source_tables,
            mapping_sql,
        )
        .is_err()
    );
    Ok(())
}
