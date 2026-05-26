use std::collections::BTreeSet;
use std::error::Error;

use arrow::datatypes::DataType;
use xiuxian_db_store::{
    RETRIEVAL_ID_COLUMN, RETRIEVAL_LINE_COLUMN, RETRIEVAL_SCORE_COLUMN, RetrievalDocType,
    RetrievalRow, payload_fetch_record_batch, retrieval_result_columns, retrieval_result_schema,
    retrieval_rows_from_record_batch, retrieval_rows_to_record_batch,
};

#[test]
fn retrieval_result_schema_is_built_from_shared_contract() -> Result<(), Box<dyn Error>> {
    let schema = retrieval_result_schema();

    assert_eq!(schema.fields().len(), retrieval_result_columns().len());
    assert_eq!(schema.fields()[0].name(), RETRIEVAL_ID_COLUMN);
    assert!(!schema.fields()[0].is_nullable());

    let score = schema.field_with_name(RETRIEVAL_SCORE_COLUMN)?;
    assert_eq!(score.data_type(), &DataType::Float64);
    assert!(score.is_nullable());

    let line = schema.field_with_name(RETRIEVAL_LINE_COLUMN)?;
    assert_eq!(line.data_type(), &DataType::UInt64);
    assert!(line.is_nullable());

    Ok(())
}

#[test]
fn projected_retrieval_payload_schema_preserves_contract_columns() -> Result<(), Box<dyn Error>> {
    let batch = retrieval_rows_to_record_batch(&sample_rows())?;
    let projection = payload_fetch_record_batch(
        &batch,
        &[
            RETRIEVAL_ID_COLUMN.to_owned(),
            RETRIEVAL_LINE_COLUMN.to_owned(),
            RETRIEVAL_SCORE_COLUMN.to_owned(),
        ],
        None,
    )?;

    assert_eq!(projection.schema().fields().len(), 3);
    assert_eq!(projection.schema().fields()[0].name(), RETRIEVAL_ID_COLUMN);
    assert_eq!(
        projection.schema().fields()[1].data_type(),
        &DataType::UInt64
    );
    assert_eq!(
        projection.schema().fields()[2].data_type(),
        &DataType::Float64
    );

    Ok(())
}

#[test]
fn retrieval_batch_round_trip_keeps_rows_and_filter_order() -> Result<(), Box<dyn Error>> {
    let batch = retrieval_rows_to_record_batch(&sample_rows())?;
    let decoded = retrieval_rows_from_record_batch(&batch)?;
    assert_eq!(decoded, sample_rows());

    let ids = BTreeSet::from(["candidate-2".to_owned()]);
    let projection =
        payload_fetch_record_batch(&batch, &[RETRIEVAL_ID_COLUMN.to_owned()], Some(&ids))?;
    assert_eq!(projection.num_rows(), 1);
    assert_eq!(projection.num_columns(), 1);

    Ok(())
}

#[test]
fn retrieval_payload_projection_rejects_unknown_columns() -> Result<(), Box<dyn Error>> {
    let batch = retrieval_rows_to_record_batch(&sample_rows())?;

    let error = payload_fetch_record_batch(&batch, &["unknown".to_owned()], None)
        .err()
        .ok_or("unknown projection column should fail")?;

    assert!(
        error
            .to_string()
            .contains("unsupported retrieval payload column `unknown`")
    );
    Ok(())
}

fn sample_rows() -> Vec<RetrievalRow> {
    vec![
        RetrievalRow {
            id: "candidate-1".to_owned(),
            path: "docs/a.md".to_owned(),
            repo: Some("main".to_owned()),
            title: Some("A".to_owned()),
            score: Some(0.91),
            source: "hybrid".to_owned(),
            snippet: Some("alpha".to_owned()),
            doc_type: Some(RetrievalDocType::from("markdown")),
            match_reason: Some("title".to_owned()),
            best_section: Some("overview".to_owned()),
            language: Some("markdown".to_owned()),
            line: Some(12),
        },
        RetrievalRow {
            id: "candidate-2".to_owned(),
            path: "docs/b.md".to_owned(),
            repo: None,
            title: None,
            score: None,
            source: "semantic".to_owned(),
            snippet: None,
            doc_type: None,
            match_reason: None,
            best_section: None,
            language: None,
            line: None,
        },
    ]
}
