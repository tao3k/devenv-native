use std::collections::BTreeSet;

use crate::{
    RETRIEVAL_ID_COLUMN, RETRIEVAL_PATH_COLUMN, RetrievalRow, payload_fetch_record_batch,
    retrieval_rows_from_record_batch, retrieval_rows_to_record_batch,
};
use anyhow::Result;

fn sample_rows() -> Vec<RetrievalRow> {
    vec![
        RetrievalRow {
            id: "a".to_string(),
            path: "src/a.rs".to_string(),
            repo: Some("alpha/repo".to_string()),
            title: Some("Alpha".to_string()),
            score: Some(0.8),
            source: "legacy-search-plane".to_string(),
            snippet: Some("fn alpha()".to_string()),
            doc_type: Some("file".to_string()),
            match_reason: Some("repo_content_search".to_string()),
            best_section: Some("3: fn alpha()".to_string()),
            language: Some("rust".to_string()),
            line: Some(3),
        },
        RetrievalRow {
            id: "b".to_string(),
            path: "src/b.rs".to_string(),
            repo: Some("alpha/repo".to_string()),
            title: Some("Beta".to_string()),
            score: Some(0.6),
            source: "legacy-search-plane".to_string(),
            snippet: Some("fn beta()".to_string()),
            doc_type: Some("file".to_string()),
            match_reason: Some("repo_content_search".to_string()),
            best_section: Some("8: fn beta()".to_string()),
            language: Some("rust".to_string()),
            line: Some(8),
        },
    ]
}

#[test]
fn retrieval_rows_to_record_batch_preserves_schema() -> Result<()> {
    let batch = retrieval_rows_to_record_batch(&sample_rows())?;
    let schema = batch.schema();
    let field_names = schema
        .fields()
        .iter()
        .map(|field| field.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        field_names,
        vec![
            "id",
            "path",
            "repo",
            "title",
            "score",
            "source",
            "snippet",
            "doc_type",
            "match_reason",
            "best_section",
            "language",
            "line",
        ]
    );

    Ok(())
}

#[test]
fn payload_fetch_record_batch_projects_requested_columns() -> Result<()> {
    let batch = retrieval_rows_to_record_batch(&sample_rows())?;
    let projected = payload_fetch_record_batch(
        &batch,
        &[
            RETRIEVAL_ID_COLUMN.to_string(),
            RETRIEVAL_PATH_COLUMN.to_string(),
        ],
        None,
    )?;
    let schema = projected.schema();
    let field_names = schema
        .fields()
        .iter()
        .map(|field| field.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(field_names, vec!["id", "path"]);
    assert_eq!(projected.num_rows(), 2);

    Ok(())
}

#[test]
fn payload_fetch_record_batch_filters_by_id() -> Result<()> {
    let batch = retrieval_rows_to_record_batch(&sample_rows())?;
    let ids = BTreeSet::from(["b".to_string()]);
    let projected = payload_fetch_record_batch(&batch, &[], Some(&ids))?;
    let rows = retrieval_rows_from_record_batch(&projected)?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "b");

    Ok(())
}
