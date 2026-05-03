use std::io::Cursor;

use arrow::array::{Array, StringArray, UInt64Array};
use arrow::ipc::reader::StreamReader;

use crate::studio::router::retrieval_arrow::encode_retrieval_chunks_ipc;
use xiuxian_wendao::search::contracts::{RetrievalChunk, RetrievalChunkSurface};

#[test]
fn retrieval_arrow_roundtrip_preserves_chunk_fields() {
    let chunks = vec![
        RetrievalChunk {
            owner_id: "section:intro".to_string(),
            chunk_id: "md:intro".to_string(),
            semantic_type: "section".to_string(),
            fingerprint: "fp:intro".to_string(),
            token_estimate: 18,
            display_label: Some("Intro".to_string()),
            excerpt: Some("Hello world".to_string()),
            line_start: Some(1),
            line_end: Some(4),
            surface: Some(RetrievalChunkSurface::Section),
            attributes: vec![("heading_path".to_string(), "Intro".to_string())],
        },
        RetrievalChunk {
            owner_id: "block:return:solve".to_string(),
            chunk_id: "ast:return:solve".to_string(),
            semantic_type: "return".to_string(),
            fingerprint: "fp:return".to_string(),
            token_estimate: 9,
            display_label: None,
            excerpt: None,
            line_start: Some(22),
            line_end: Some(24),
            surface: Some(RetrievalChunkSurface::Block),
            attributes: Vec::new(),
        },
    ];

    let encoded = encode_retrieval_chunks_ipc(&chunks)
        .unwrap_or_else(|error| panic!("arrow encoding should succeed: {error}"));
    let reader = StreamReader::try_new(Cursor::new(encoded), None)
        .unwrap_or_else(|error| panic!("stream reader should open: {error}"));
    let batches = reader
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("batches should decode: {error}"));
    assert_eq!(batches.len(), 1);
    let batch = &batches[0];
    assert_eq!(batch.num_rows(), 2);

    let Some(owner_id_column) = batch.column_by_name("ownerId") else {
        panic!("ownerId column");
    };
    let Some(owner_ids) = owner_id_column.as_any().downcast_ref::<StringArray>() else {
        panic!("ownerId should be utf8");
    };
    assert_eq!(owner_ids.value(0), "section:intro");
    assert_eq!(owner_ids.value(1), "block:return:solve");

    let Some(token_estimate_column) = batch.column_by_name("tokenEstimate") else {
        panic!("tokenEstimate column");
    };
    let Some(token_estimates) = token_estimate_column.as_any().downcast_ref::<UInt64Array>() else {
        panic!("tokenEstimate should be u64");
    };
    assert_eq!(token_estimates.value(0), 18);
    assert_eq!(token_estimates.value(1), 9);

    let Some(surface_column) = batch.column_by_name("surface") else {
        panic!("surface column");
    };
    let Some(surfaces) = surface_column.as_any().downcast_ref::<StringArray>() else {
        panic!("surface should be utf8");
    };
    assert_eq!(surfaces.value(0), "section");
    assert_eq!(surfaces.value(1), "block");

    let Some(attributes_column) = batch.column_by_name("attributesJson") else {
        panic!("attributesJson column");
    };
    let Some(attributes) = attributes_column.as_any().downcast_ref::<StringArray>() else {
        panic!("attributesJson should be utf8");
    };
    assert_eq!(attributes.value(0), r#"[["heading_path","Intro"]]"#);
    assert!(attributes.is_null(1));
}
