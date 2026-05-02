use super::{
    Arc, AstSearchQuery, Query, State, UiConfig, UiProjectConfig, assert_studio_json_snapshot,
    json, make_state_with_docs, publish_local_symbol_index, round_f64, search_ast,
};

#[tokio::test]
async fn search_ast_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = search_ast(
        State(Arc::clone(&fixture.state)),
        Query(AstSearchQuery {
            q: Some("   ".to_string()),
            limit: None,
        }),
    )
    .await;

    let Err(error) = result else {
        panic!("expected missing-query AST search to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn search_ast_returns_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n\npub fn alpha_handler() {}\n",
        ),
        (
            "packages/python/demo/tool.py",
            "class AlphaClient:\n    pass\n\ndef alpha_helper():\n    return None\n",
        ),
        (
            "notes/ignored.txt",
            "alpha should stay outside AST search fixtures.\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = search_ast(
        State(fixture.state),
        Query(AstSearchQuery {
            q: Some("alpha".to_string()),
            limit: Some(10),
        }),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected AST search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_ast_payload",
        json!({
            "query": response.0.query,
            "hitCount": response.0.hit_count,
            "selectedScope": response.0.selected_scope,
            "hits": response.0.hits.into_iter().map(|hit| {
                json!({
                    "name": hit.name,
                    "signature": hit.signature,
                    "path": hit.path,
                    "language": hit.language,
                    "crateName": hit.crate_name,
                    "projectName": hit.project_name,
                    "rootLabel": hit.root_label,
                    "nodeKind": hit.node_kind,
                    "ownerTitle": hit.owner_title,
                    "navigationTarget": {
                        "path": hit.navigation_target.path,
                        "category": hit.navigation_target.category,
                        "projectName": hit.navigation_target.project_name,
                        "rootLabel": hit.navigation_target.root_label,
                        "line": hit.navigation_target.line,
                        "lineEnd": hit.navigation_target.line_end,
                        "column": hit.navigation_target.column,
                    },
                    "lineStart": hit.line_start,
                    "lineEnd": hit.line_end,
                    "score": round_f64(hit.score),
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_ast_includes_markdown_outline_hits() {
    let fixture = make_state_with_docs(vec![(
        "docs/03_features/204_gateway_api_contracts.md",
        "# Gateway API Contracts\n\n## AST Search\n\n- [ ] Verify docs AST alignment.\n",
    )]);
    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: vec![UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            }],
            repo_projects: Vec::new(),
        });
    publish_local_symbol_index(&fixture.state).await;

    let result = search_ast(
        State(Arc::clone(&fixture.state)),
        Query(AstSearchQuery {
            q: Some("ast".to_string()),
            limit: Some(10),
        }),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected markdown AST search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_ast_markdown_payload",
        json!({
            "query": response.0.query,
            "hitCount": response.0.hit_count,
            "selectedScope": response.0.selected_scope,
            "hits": response.0.hits.into_iter().map(|hit| {
                json!({
                    "name": hit.name,
                    "signature": hit.signature,
                    "path": hit.path,
                    "language": hit.language,
                    "crateName": hit.crate_name,
                    "projectName": hit.project_name,
                    "rootLabel": hit.root_label,
                    "nodeKind": hit.node_kind,
                    "ownerTitle": hit.owner_title,
                    "navigationTarget": {
                        "path": hit.navigation_target.path,
                        "category": hit.navigation_target.category,
                        "projectName": hit.navigation_target.project_name,
                        "rootLabel": hit.navigation_target.root_label,
                        "line": hit.navigation_target.line,
                        "lineEnd": hit.navigation_target.line_end,
                        "column": hit.navigation_target.column,
                    },
                    "lineStart": hit.line_start,
                    "lineEnd": hit.line_end,
                    "score": round_f64(hit.score),
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_ast_includes_markdown_property_drawer_hits() {
    let fixture = make_state_with_docs(vec![(
        "docs/index.md",
        "# Studio Functional Ledger\n:PROPERTIES:\n:ID: SearchBarProtocol\n:OBSERVE: lang:typescript scope:\"src/components/SearchBar/**\" \"export const SearchBar: React.FC<SearchBarProps> = ({ $$$ })\"\n:END:\n\n## Runtime Contract\n",
    )]);
    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: vec![UiProjectConfig {
                name: "main".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            }],
            repo_projects: Vec::new(),
        });
    publish_local_symbol_index(&fixture.state).await;

    let result = search_ast(
        State(Arc::clone(&fixture.state)),
        Query(AstSearchQuery {
            q: Some("SearchBar".to_string()),
            limit: Some(10),
        }),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected markdown property AST search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_ast_markdown_property_payload",
        json!({
            "query": response.0.query,
            "hitCount": response.0.hit_count,
            "selectedScope": response.0.selected_scope,
            "hits": response.0.hits.into_iter().map(|hit| {
                json!({
                    "name": hit.name,
                    "signature": hit.signature,
                    "path": hit.path,
                    "language": hit.language,
                    "crateName": hit.crate_name,
                    "projectName": hit.project_name,
                    "rootLabel": hit.root_label,
                    "nodeKind": hit.node_kind,
                    "ownerTitle": hit.owner_title,
                    "navigationTarget": {
                        "path": hit.navigation_target.path,
                        "category": hit.navigation_target.category,
                        "projectName": hit.navigation_target.project_name,
                        "rootLabel": hit.navigation_target.root_label,
                        "line": hit.navigation_target.line,
                        "lineEnd": hit.navigation_target.line_end,
                        "column": hit.navigation_target.column,
                    },
                    "lineStart": hit.line_start,
                    "lineEnd": hit.line_end,
                    "score": round_f64(hit.score),
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_ast_returns_partial_response_before_initial_index_publication() {
    let fixture = make_state_with_docs(vec![(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct AlphaService {\n    ready: bool,\n}\n\npub fn alpha_handler() {}\n",
    )]);

    let result = search_ast(
        State(fixture.state),
        Query(AstSearchQuery {
            q: Some("alpha".to_string()),
            limit: Some(10),
        }),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected cold-start AST search request to succeed");
    };

    assert_eq!(response.0.hit_count, 0);
    assert!(response.0.partial);
    assert_eq!(response.0.indexing_state.as_deref(), Some("indexing"));
    assert!(response.0.hits.is_empty());
}
