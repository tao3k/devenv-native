use super::{
    DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME, DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME, DataFusionLocalRelationEngine, TestResult,
    healthcare_mapping_sql, healthcare_source_tables, materialize_dataset_ontology_with_engine,
};

#[tokio::test]
async fn dataset_ontology_materializes_healthcare_counts() -> TestResult {
    let engine = DataFusionLocalRelationEngine::new_with_information_schema();
    let report = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &healthcare_mapping_sql(true),
    )
    .await
    .map_err(std::io::Error::other)?;

    assert_eq!(report.execution_engine, "datafusion");
    assert!(report.passed(), "{:?}", report.validation_failures);
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME),
        Some(8)
    );
    assert_eq!(
        report.row_count_for(DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME),
        Some(6)
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
    Ok(())
}

#[tokio::test]
async fn dataset_ontology_validation_reports_missing_provider_links() -> TestResult {
    let engine = DataFusionLocalRelationEngine::new_with_information_schema();
    let report = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &healthcare_mapping_sql(false),
    )
    .await
    .map_err(std::io::Error::other)?;

    assert!(!report.passed());
    assert_eq!(report.validation_failures.len(), 1);
    assert_eq!(
        report.validation_failures[0].rule_id,
        "HEALTHCARE_ENCOUNTER_MISSING_CONTEXT"
    );
    assert_eq!(report.validation_failures[0].row_count, 2);
    Ok(())
}
