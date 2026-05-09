use crate::studio::arrow_types::{LanceArray, LanceStringArray, LanceUInt64Array};

use super::{
    repo_projected_retrieval_context_batch,
    repo_projected_retrieval_context_batch_with_requested_node,
    repo_projected_retrieval_context_metadata,
    repo_projected_retrieval_context_metadata_with_requested_node,
};
use xiuxian_wendao::analyzers::{
    ProjectedPageRecord, ProjectedRetrievalHit, ProjectedRetrievalHitKind, ProjectionPageKind,
    RepoProjectedRetrievalContextResult,
};

fn demo_page(page_id: &str, title: &str) -> ProjectedPageRecord {
    ProjectedPageRecord {
        repo_id: "gateway-sync".to_string(),
        page_id: page_id.to_string(),
        kind: ProjectionPageKind::Reference,
        title: title.to_string(),
        module_ids: Vec::new(),
        symbol_ids: Vec::new(),
        example_ids: Vec::new(),
        doc_ids: vec![format!("repo:gateway-sync:doc:{page_id}")],
        paths: vec!["docs/solve.md".to_string()],
        format_hints: vec!["markdown".to_string()],
        sections: Vec::new(),
        doc_id: format!("repo:gateway-sync:doc:{page_id}"),
        path: "docs/solve.md".to_string(),
        keywords: vec![title.to_string()],
    }
}

fn demo_context() -> RepoProjectedRetrievalContextResult {
    let center_page = demo_page(
        "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md",
        "solve",
    );
    RepoProjectedRetrievalContextResult {
        repo_id: "gateway-sync".to_string(),
        center: ProjectedRetrievalHit {
            kind: ProjectedRetrievalHitKind::Page,
            page: center_page,
            node: None,
        },
        related_pages: vec![demo_page(
            "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/related.md",
            "related",
        )],
        node_context: None,
    }
}

fn demo_context_with_node_context() -> RepoProjectedRetrievalContextResult {
    RepoProjectedRetrievalContextResult {
        node_context: Some(Default::default()),
        ..demo_context()
    }
}

#[test]
fn projected_retrieval_context_batch_preserves_json_payload() {
    let batch = repo_projected_retrieval_context_batch(&demo_context())
        .unwrap_or_else(|error| panic!("batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(page_id_column) = batch.column_by_name("pageId") else {
        panic!("pageId column");
    };
    let Some(page_ids) = page_id_column.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("pageId column type");
    };
    assert_eq!(
        page_ids.value(0),
        "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
    );

    let Some(related_count_column) = batch.column_by_name("relatedCount") else {
        panic!("relatedCount column");
    };
    let Some(related_counts) = related_count_column
        .as_any()
        .downcast_ref::<LanceUInt64Array>()
    else {
        panic!("relatedCount column type");
    };
    assert_eq!(related_counts.value(0), 1);

    let Some(center_json_column) = batch.column_by_name("centerJson") else {
        panic!("centerJson column");
    };
    let Some(center_json) = center_json_column
        .as_any()
        .downcast_ref::<LanceStringArray>()
    else {
        panic!("centerJson column type");
    };
    let center: ProjectedRetrievalHit = serde_json::from_str(center_json.value(0))
        .unwrap_or_else(|error| panic!("centerJson should decode: {error}"));
    assert_eq!(center.page.title, "solve");

    let Some(node_context_column) = batch.column_by_name("nodeContextJson") else {
        panic!("nodeContextJson column");
    };
    assert!(node_context_column.is_null(0));
}

#[test]
fn projected_retrieval_context_metadata_preserves_summary_fields() {
    let metadata = repo_projected_retrieval_context_metadata(&demo_context())
        .unwrap_or_else(|error| panic!("metadata should encode: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["repoId"], "gateway-sync");
    assert_eq!(
        payload["pageId"],
        "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
    );
    assert!(payload["nodeId"].is_null());
    assert_eq!(payload["relatedCount"], 1);
    assert_eq!(payload["hasNodeContext"], false);
}

#[test]
fn projected_retrieval_context_batch_materializes_requested_node_id() {
    let batch = repo_projected_retrieval_context_batch_with_requested_node(
        &demo_context_with_node_context(),
        Some("reference/solve#anchors"),
    )
    .unwrap_or_else(|error| panic!("batch should build: {error}"));

    let Some(node_id_column) = batch.column_by_name("nodeId") else {
        panic!("nodeId column");
    };
    let Some(node_ids) = node_id_column.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("nodeId column type");
    };
    assert_eq!(node_ids.value(0), "reference/solve#anchors");
}

#[test]
fn projected_retrieval_context_metadata_materializes_requested_node_id() {
    let metadata = repo_projected_retrieval_context_metadata_with_requested_node(
        &demo_context_with_node_context(),
        Some("reference/solve#anchors"),
    )
    .unwrap_or_else(|error| panic!("metadata should encode: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["nodeId"], "reference/solve#anchors");
}
