use std::sync::Arc;

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use crate::local_relation::{
    DuckDbLocalRelationEngine, LocalRelationEngine, LocalRelationMaterializationState,
};

#[tokio::test]
async fn duckdb_engine_materializes_arrow_batches_and_queries_results() {
    let engine = DuckDbLocalRelationEngine::new_in_memory().expect("open DuckDB");
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(Int64Array::from(vec![2_i64, 1_i64])),
            Arc::new(StringArray::from(vec!["beta", "alpha"])),
        ],
    )
    .expect("build batch");

    engine
        .register_record_batches("items", schema, vec![batch])
        .expect("register table");

    assert_eq!(
        engine.relation_registration_strategy("items"),
        Some("duckdb_materialized_arrow_staging")
    );
    assert_eq!(
        engine.relation_materialization_state("items"),
        Some(LocalRelationMaterializationState::Materialized)
    );
    let batches = engine
        .query_batches("select name from items order by id")
        .await
        .expect("query rows");
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 2);
    assert!(format!("{batches:?}").contains("alpha"));
}
