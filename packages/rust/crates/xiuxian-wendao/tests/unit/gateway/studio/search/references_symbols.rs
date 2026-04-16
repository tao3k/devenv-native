use super::*;

#[tokio::test]
async fn search_references_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = load_reference_search_response(
        fixture.state.as_ref(),
        ReferenceSearchQuery {
            q: Some("   ".to_string()),
            limit: None,
        },
    )
    .await;

    let Err(error) = result else {
        panic!("expected missing-query reference search to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn search_references_returns_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n\npub fn alpha_handler() {\n    let _service = AlphaService { ready: true };\n}\n",
        ),
        (
            "packages/python/demo/tool.py",
            "class AlphaClient:\n    pass\n\ndef alpha_helper(client: AlphaClient):\n    return client\n",
        ),
    ]);
    publish_reference_occurrence_index(&fixture.state).await;

    let result = load_reference_search_response(
        fixture.state.as_ref(),
        ReferenceSearchQuery {
            q: Some("AlphaService".to_string()),
            limit: Some(10),
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected reference search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_references_payload",
        json!({
            "query": response.query,
            "hitCount": response.hit_count,
            "selectedScope": response.selected_scope,
            "hits": response.hits.into_iter().map(|hit| {
                json!({
                    "name": hit.name,
                    "path": hit.path,
                    "language": hit.language,
                    "crateName": hit.crate_name,
                    "projectName": hit.project_name,
                    "rootLabel": hit.root_label,
                    "navigationTarget": {
                        "path": hit.navigation_target.path,
                        "category": hit.navigation_target.category,
                        "projectName": hit.navigation_target.project_name,
                        "rootLabel": hit.navigation_target.root_label,
                        "line": hit.navigation_target.line,
                        "lineEnd": hit.navigation_target.line_end,
                        "column": hit.navigation_target.column,
                    },
                    "line": hit.line,
                    "column": hit.column,
                    "lineText": hit.line_text,
                    "score": round_f64(hit.score),
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_references_returns_partial_response_before_initial_index_publication() {
    let fixture = make_state_with_docs(vec![(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct AlphaService {\n    ready: bool,\n}\n\npub fn alpha_handler() {\n    let _service = AlphaService { ready: true };\n}\n",
    )]);

    let result = load_reference_search_response(
        fixture.state.as_ref(),
        ReferenceSearchQuery {
            q: Some("AlphaService".to_string()),
            limit: Some(10),
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected cold-start reference search request to succeed");
    };

    assert_eq!(response.hit_count, 0);
    assert!(response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("indexing"));
    assert!(response.hits.is_empty());
}

#[tokio::test]
async fn search_symbols_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = load_symbol_search_response(
        fixture.state.as_ref(),
        SymbolSearchQuery {
            q: Some("   ".to_string()),
            limit: None,
        },
    )
    .await;

    let Err(error) = result else {
        panic!("expected missing-query symbol search to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn search_symbols_returns_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub struct AlphaService;\npub fn alpha_handler() {}\n",
        ),
        (
            "packages/python/demo/tool.py",
            "class AlphaClient:\n    pass\n\ndef alpha_helper():\n    return None\n",
        ),
        (
            "notes/ignored.md",
            "# alpha\n\nThis markdown file should not affect symbol search.\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = load_symbol_search_response(
        fixture.state.as_ref(),
        SymbolSearchQuery {
            q: Some("alpha".to_string()),
            limit: Some(10),
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected symbol search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_symbols_payload",
        json!({
            "query": response.query,
            "hitCount": response.hit_count,
            "selectedScope": response.selected_scope,
            "partial": response.partial,
            "indexingState": response.indexing_state,
            "hits": response.hits.into_iter().map(|hit| {
                json!({
                    "name": hit.name,
                    "kind": hit.kind,
                    "path": hit.path,
                    "line": hit.line,
                    "location": hit.location,
                    "language": hit.language,
                    "crateName": hit.crate_name,
                    "projectName": hit.project_name,
                    "rootLabel": hit.root_label,
                    "navigationTarget": {
                        "path": hit.navigation_target.path,
                        "category": hit.navigation_target.category,
                        "projectName": hit.navigation_target.project_name,
                        "rootLabel": hit.navigation_target.root_label,
                        "line": hit.navigation_target.line,
                        "lineEnd": hit.navigation_target.line_end,
                        "column": hit.navigation_target.column,
                    },
                    "source": hit.source,
                    "score": round_f64(hit.score),
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_symbols_returns_pending_payload_while_index_is_warming() {
    let fixture = make_state_with_docs(vec![(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct PendingSymbolIndex;\n",
    )]);
    fixture
        .state
        .studio
        .ensure_local_symbol_index_started()
        .unwrap_or_else(|error| {
            panic!("expected local symbol build start to succeed: {error:?}");
        });

    let result = load_symbol_search_response(
        fixture.state.as_ref(),
        SymbolSearchQuery {
            q: Some("pending".to_string()),
            limit: Some(10),
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected pending symbol search request to succeed");
    };

    assert_eq!(response.hit_count, 0);
    assert!(response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("indexing"));
    assert!(response.hits.is_empty());
}

#[tokio::test]
async fn search_symbols_respects_glob_dir_filters() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/alpha/src/lib.rs",
            "pub struct GlobFilteredSymbol;\npub fn alpha_glob_symbol() {}\n",
        ),
        (
            "packages/beta/src/lib.rs",
            "pub struct GlobFilteredSymbol;\npub fn beta_glob_symbol() {}\n",
        ),
    ]);

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: vec![UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["packages".to_string(), "packages/alpha/**/*.rs".to_string()],
            }],
            repo_projects: Vec::new(),
        });
    publish_local_symbol_index(&fixture.state).await;

    let result = load_symbol_search_response(
        fixture.state.as_ref(),
        SymbolSearchQuery {
            q: Some("GlobFilteredSymbol".to_string()),
            limit: Some(10),
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected glob-filtered symbol search to succeed");
    };

    let hit_paths = response
        .hits
        .iter()
        .map(|hit| hit.path.as_str())
        .collect::<Vec<_>>();
    assert!(!hit_paths.is_empty());
    assert!(
        hit_paths
            .iter()
            .all(|path| path.starts_with("packages/alpha/")),
        "unexpected glob-filtered hit paths: {hit_paths:?}",
    );
}
