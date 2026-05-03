use crate::studio::arrow_types::{LanceArray, LanceStringArray};

use crate::studio::router::handlers::repo::analysis::projected_page_index_tree_flight::{
    repo_projected_page_index_tree_batch, repo_projected_page_index_tree_metadata,
};
use xiuxian_wendao::analyzers::{
    ProjectedPageIndexNode, ProjectedPageIndexTree, ProjectionPageKind,
    RepoProjectedPageIndexTreeResult,
};

fn demo_tree() -> ProjectedPageIndexTree {
    ProjectedPageIndexTree {
        repo_id: "gateway-sync".to_string(),
        page_id: "repo:gateway-sync:projection:reference:doc:repo:gateway-sync:doc:docs/solve.md"
            .to_string(),
        kind: ProjectionPageKind::Reference,
        path: "docs/solve.md".to_string(),
        doc_id: "repo:gateway-sync:doc:docs/solve.md".to_string(),
        title: "solve".to_string(),
        root_count: 1,
        roots: vec![ProjectedPageIndexNode {
            node_id: "repo:gateway-sync:doc:docs/solve.md#root".to_string(),
            title: "solve".to_string(),
            level: 1,
            structural_path: vec!["solve".to_string()],
            line_range: (1, 3),
            token_count: 4,
            is_thinned: false,
            text: "solve docs".to_string(),
            summary: None,
            children: Vec::new(),
        }],
    }
}

#[test]
fn projected_page_index_tree_batch_preserves_tree_payload() {
    let batch = repo_projected_page_index_tree_batch(&RepoProjectedPageIndexTreeResult {
        repo_id: "gateway-sync".to_string(),
        tree: Some(demo_tree()),
    })
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
    let Some(kind_column) = batch.column_by_name("kind") else {
        panic!("kind column");
    };
    let Some(kinds) = kind_column.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("kind column type");
    };
    assert_eq!(kinds.value(0), "reference");

    let Some(roots_json_column) = batch.column_by_name("rootsJson") else {
        panic!("rootsJson column");
    };
    let Some(roots_json) = roots_json_column
        .as_any()
        .downcast_ref::<LanceStringArray>()
    else {
        panic!("rootsJson column type");
    };
    let roots: Vec<ProjectedPageIndexNode> = serde_json::from_str(roots_json.value(0))
        .unwrap_or_else(|error| panic!("rootsJson should decode: {error}"));
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].title, "solve");
}

#[test]
fn projected_page_index_tree_metadata_preserves_summary_fields() {
    let metadata = repo_projected_page_index_tree_metadata(&RepoProjectedPageIndexTreeResult {
        repo_id: "gateway-sync".to_string(),
        tree: Some(demo_tree()),
    })
    .unwrap_or_else(|error| panic!("metadata should encode: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["repoId"], "gateway-sync");
    assert_eq!(payload["kind"], "reference");
    assert_eq!(payload["path"], "docs/solve.md");
    assert_eq!(payload["rootCount"], 1);
}
