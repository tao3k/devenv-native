use std::fs;
use std::sync::Arc;

use axum::extract::State;

use crate::contracts::{UiConfig, UiProjectConfig};
use crate::studio::router::handlers::graph::neighbors::graph_neighbors;
use crate::studio::router::handlers::graph::shared::GraphNeighborsQuery;
use crate::studio::router::{GatewayState, StudioState};

#[tokio::test]
async fn graph_index_refreshes_after_document_title_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let docs_dir = temp_dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap_or_else(|error| panic!("create docs dir: {error}"));
    fs::write(
        docs_dir.join("index.md"),
        concat!(
            "---\n",
            "title: Documentation Index\n",
            "---\n\n",
            "# Documentation Index\n\n",
            "Body.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write docs index: {error}"));
    fs::write(docs_dir.join("chapter.md"), "# Chapter\n\nBody.\n")
        .unwrap_or_else(|error| panic!("write docs chapter: {error}"));

    let mut studio = StudioState::new();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = temp_dir.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let first_response = graph_neighbors(
        State(Arc::clone(&state)),
        axum::extract::Path("kernel/docs/index.md".to_string()),
        axum::extract::Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(1),
            limit: Some(20),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("initial graph neighbors should build: {error:?}"))
    .0;
    assert_eq!(first_response.center.label, "Documentation Index");

    fs::write(
        docs_dir.join("index.md"),
        concat!(
            "---\n",
            "title: Qianji Studio DocOS Kernel: Map of Content\n",
            "---\n\n",
            "# Qianji Studio DocOS Kernel: Map of Content\n\n",
            "- [[chapter]]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("rewrite docs index: {error}"));

    let refreshed_response = graph_neighbors(
        State(Arc::clone(&state)),
        axum::extract::Path("kernel/docs/index.md".to_string()),
        axum::extract::Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(1),
            limit: Some(20),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("refreshed graph neighbors should rebuild: {error:?}"))
    .0;
    assert_eq!(
        refreshed_response.center.label,
        "Qianji Studio DocOS Kernel: Map of Content"
    );
}

#[tokio::test]
async fn graph_neighbors_prefers_kernel_project_docs_over_repo_root_docs() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let config_root = temp_dir.path().join(".data/wendao-frontend");
    let kernel_docs_dir = config_root.join("docs");
    let main_docs_dir = temp_dir.path().join("docs");

    fs::create_dir_all(&kernel_docs_dir)
        .unwrap_or_else(|error| panic!("create kernel docs dir: {error}"));
    fs::create_dir_all(&main_docs_dir)
        .unwrap_or_else(|error| panic!("create main docs dir: {error}"));

    fs::write(
        kernel_docs_dir.join("index.md"),
        concat!(
            "---\n",
            "title: Qianji Studio DocOS Kernel: Map of Content\n",
            "---\n\n",
            "# Qianji Studio DocOS Kernel: Map of Content\n\n",
            "- [[chapter]]\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write kernel docs index: {error}"));
    fs::write(
        kernel_docs_dir.join("chapter.md"),
        "# Kernel Chapter\n\nBody.\n",
    )
    .unwrap_or_else(|error| panic!("write kernel chapter: {error}"));
    fs::write(
        main_docs_dir.join("index.md"),
        concat!(
            "---\n",
            "title: Documentation Index\n",
            "---\n\n",
            "# Documentation Index\n\n",
            "Body.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write main docs index: {error}"));

    let mut studio = StudioState::new();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = config_root.clone();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![
            UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            },
            UiProjectConfig {
                name: "main".to_string(),
                root: temp_dir.path().to_string_lossy().to_string(),
                dirs: vec!["docs".to_string()],
            },
        ],
        repo_projects: Vec::new(),
    });

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let kernel_response = graph_neighbors(
        State(Arc::clone(&state)),
        axum::extract::Path("kernel/docs/index.md".to_string()),
        axum::extract::Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(1),
            limit: Some(20),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("kernel graph neighbors should resolve: {error:?}"))
    .0;
    assert_eq!(
        kernel_response.center.label,
        "Qianji Studio DocOS Kernel: Map of Content"
    );
    assert!(
        kernel_response
            .nodes
            .iter()
            .any(|node| node.id == "kernel/docs/chapter.md")
    );

    let main_response = graph_neighbors(
        State(Arc::clone(&state)),
        axum::extract::Path("main/docs/index.md".to_string()),
        axum::extract::Query(GraphNeighborsQuery {
            direction: Some("both".to_string()),
            hops: Some(1),
            limit: Some(20),
        }),
    )
    .await
    .unwrap_or_else(|error| panic!("main graph neighbors should resolve: {error:?}"))
    .0;
    assert_eq!(main_response.center.label, "Documentation Index");
    assert!(
        main_response
            .nodes
            .iter()
            .any(|node| node.id == "main/docs/index.md")
    );
}
