use super::{
    DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME, DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME, DuckDbLocalRelationEngine, TestResult,
    healthcare_mapping_sql, healthcare_source_tables, materialize_dataset_ontology_with_engine,
};

#[tokio::test]
async fn dataset_ontology_materializes_healthcare_counts() -> TestResult {
    let engine = DuckDbLocalRelationEngine::new_in_memory().map_err(std::io::Error::other)?;
    let report = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &healthcare_mapping_sql(true),
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
    let engine = DuckDbLocalRelationEngine::new_in_memory().map_err(std::io::Error::other)?;
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

#[tokio::test]
async fn dataset_ontology_rejects_semantic_read_model_schema_drift() -> TestResult {
    let engine = DuckDbLocalRelationEngine::new_in_memory().map_err(std::io::Error::other)?;
    let mut mapping_sql = healthcare_mapping_sql(true);
    mapping_sql.semantic_objects = "select object_id as id from ontology_object_observation".into();

    let Err(error) = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &mapping_sql,
    )
    .await
    else {
        return Err("semantic read-model schema drift must fail".into());
    };

    assert!(
        error.contains("dataset ontology `semantic_objects` output schema"),
        "{error}"
    );
    assert!(
        error.contains("expected 18 columns but received 1"),
        "{error}"
    );
    Ok(())
}

#[tokio::test]
async fn dataset_ontology_rejects_object_observation_schema_drift() -> TestResult {
    let engine = DuckDbLocalRelationEngine::new_in_memory().map_err(std::io::Error::other)?;
    let mut mapping_sql = healthcare_mapping_sql(true);
    mapping_sql.object_observations =
        "select patient_id as object_id from raw_patients order by object_id".into();

    let Err(error) = materialize_dataset_ontology_with_engine(
        &engine,
        &healthcare_source_tables()?,
        &mapping_sql,
    )
    .await
    else {
        return Err("object observation schema drift must fail".into());
    };

    assert!(
        error.contains("dataset ontology `ontology_object_observation` output schema"),
        "{error}"
    );
    assert!(
        error.contains("expected 9 columns but received 1"),
        "{error}"
    );
    Ok(())
}
