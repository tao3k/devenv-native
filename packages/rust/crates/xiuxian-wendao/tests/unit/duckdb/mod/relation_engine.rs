use super::{
    Arc, DataFusionLocalRelationEngine, DataType, DuckDbLocalRelationEngine,
    DuckDbRegistrationStrategy, Field, Int64Array, LocalRelationEngine, LocalRelationEngineKind,
    LocalRelationRegistrationHint, RecordBatch, Schema, StringArray, TestResult,
    in_memory_search_duckdb_runtime,
};

#[tokio::test]
async fn datafusion_local_relation_engine_registers_and_queries_batches() -> TestResult {
    let engine = DataFusionLocalRelationEngine::new_with_information_schema();
    assert_eq!(engine.kind(), LocalRelationEngineKind::DataFusion);

    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["alpha", "beta"]))],
    )?;

    engine.register_record_batches("ducks", schema, vec![batch])?;
    let result = engine
        .query_batches("select name from ducks order by name")
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].num_rows(), 2);
    assert_eq!(result[0].schema().field(0).name(), "name");
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn duckdb_local_relation_engine_registers_and_queries_batches() -> TestResult {
    let temp = tempfile::tempdir()?;
    let engine =
        DuckDbLocalRelationEngine::from_runtime(in_memory_search_duckdb_runtime(temp.path()))
            .map_err(std::io::Error::other)?;
    assert_eq!(engine.kind(), LocalRelationEngineKind::DuckDb);

    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("line_count", DataType::Int64, false),
    ]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(vec!["beta", "alpha"])),
            Arc::new(Int64Array::from(vec![2_i64, 1_i64])),
        ],
    )?;

    engine.register_record_batches("ducks", schema, vec![batch])?;
    assert_eq!(
        engine.registered_strategy("ducks")?,
        Some(DuckDbRegistrationStrategy::VirtualArrow)
    );
    let result = engine
        .query_batches("select name, line_count from ducks order by line_count")
        .await
        .map_err(std::io::Error::other)?;

    assert!(engine.last_query_temp_storage_peak_bytes().is_some());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].num_rows(), 2);
    let names = result[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| std::io::Error::other("missing Utf8 name column"))?;
    let counts = result[0]
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| std::io::Error::other("missing Int64 line_count column"))?;
    assert_eq!(names.value(0), "alpha");
    assert_eq!(names.value(1), "beta");
    assert_eq!(counts.value(0), 1);
    assert_eq!(counts.value(1), 2);
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn duckdb_local_relation_engine_materializes_when_threshold_is_reached() -> TestResult {
    let temp = tempfile::tempdir()?;
    let mut runtime = in_memory_search_duckdb_runtime(temp.path());
    runtime.materialize_threshold_rows = 2;
    let engine = DuckDbLocalRelationEngine::from_runtime(runtime).map_err(std::io::Error::other)?;

    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["beta", "alpha"]))],
    )?;

    engine.register_record_batches("ducks", schema, vec![batch])?;
    assert_eq!(
        engine.registered_strategy("ducks")?,
        Some(DuckDbRegistrationStrategy::MaterializedAppender)
    );
    let result = engine
        .query_batches("select name from ducks order by name")
        .await
        .map_err(std::io::Error::other)?;

    assert!(engine.last_query_temp_storage_peak_bytes().is_some());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].num_rows(), 2);
    Ok(())
}

#[cfg(feature = "duckdb")]
#[tokio::test]
async fn duckdb_local_relation_engine_materializes_when_repeated_use_is_hint() -> TestResult {
    let temp = tempfile::tempdir()?;
    let mut runtime = in_memory_search_duckdb_runtime(temp.path());
    runtime.materialize_threshold_rows = 100;
    let engine = DuckDbLocalRelationEngine::from_runtime(runtime).map_err(std::io::Error::other)?;

    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["beta", "alpha"]))],
    )?;

    engine.register_record_batches_with_hint(
        "ducks",
        schema,
        vec![batch],
        LocalRelationRegistrationHint::RepeatedUse,
    )?;
    assert_eq!(
        engine.registered_strategy("ducks")?,
        Some(DuckDbRegistrationStrategy::MaterializedAppender)
    );
    assert_eq!(
        engine.relation_materialization_state("ducks"),
        Some(crate::duckdb::LocalRelationMaterializationState::Materialized)
    );
    Ok(())
}
