use super::{
    assert_studio_json_snapshot, build_definition_response, json, make_state_with_docs,
    publish_local_symbol_index, round_f64,
};

#[tokio::test]
async fn search_definition_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = build_definition_response(fixture.state.studio.as_ref(), "   ", None, None).await;

    let Err(error) = result else {
        panic!("expected missing-query definition resolve to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn search_definition_returns_best_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn build_service() {\n    let _service = AlphaService::new();\n}\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n",
        ),
        (
            "packages/rust/crates/other/src/service.rs",
            "pub struct AlphaService;\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_definition_response(
        fixture.state.studio.as_ref(),
        "AlphaService",
        Some("packages/rust/crates/demo/src/lib.rs"),
        Some(2),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected definition resolve request to succeed");
    };

    assert_studio_json_snapshot(
        "search_definition_payload",
        json!({
            "query": response.query,
            "sourcePath": response.source_path,
            "sourceLine": response.source_line,
            "candidateCount": response.candidate_count,
            "selectedScope": response.selected_scope,
            "navigationTarget": {
                "path": response.navigation_target.path,
                "category": response.navigation_target.category,
                "projectName": response.navigation_target.project_name,
                "rootLabel": response.navigation_target.root_label,
                "line": response.navigation_target.line,
                "lineEnd": response.navigation_target.line_end,
                "column": response.navigation_target.column,
            },
            "definition": {
                "name": response.definition.name,
                "signature": response.definition.signature,
                "path": response.definition.path,
                "language": response.definition.language,
                "crateName": response.definition.crate_name,
                "projectName": response.definition.project_name,
                "rootLabel": response.definition.root_label,
                "lineStart": response.definition.line_start,
                "lineEnd": response.definition.line_end,
                "score": round_f64(response.definition.score),
            },
        }),
    );
}

#[tokio::test]
async fn search_definition_waits_for_initial_index_publication() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn build_service() {\n    let _service = AlphaService::new();\n}\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n",
        ),
    ]);

    let result = build_definition_response(
        fixture.state.studio.as_ref(),
        "AlphaService",
        Some("packages/rust/crates/demo/src/lib.rs"),
        Some(2),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected cold-start definition resolve request to succeed");
    };

    assert_eq!(response.definition.name, "AlphaService");
}

#[tokio::test]
async fn search_definition_accepts_absolute_source_paths() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn build_service() {\n    let _service = AlphaService::new();\n}\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n",
        ),
        (
            "packages/rust/crates/other/src/service.rs",
            "pub struct AlphaService;\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;
    let absolute_source_path = fixture
        .state
        .studio
        .project_root
        .join("packages/rust/crates/demo/src/lib.rs")
        .to_string_lossy()
        .to_string();

    let result = build_definition_response(
        fixture.state.studio.as_ref(),
        "AlphaService",
        Some(absolute_source_path.as_str()),
        Some(2),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected definition resolve request to succeed");
    };

    assert_eq!(
        response.definition.path,
        "packages/rust/crates/demo/src/service.rs"
    );
}

#[tokio::test]
async fn search_definition_uses_markdown_observe_hints() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/notes/index.md",
            "# Index\n\n:PROPERTIES:\n:OBSERVE: lang:python scope:\"packages/python/demo/**\" \"AlphaService\"\n:END:\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService;\n",
        ),
        (
            "packages/python/demo/service.py",
            "class AlphaService:\n    pass\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_definition_response(
        fixture.state.studio.as_ref(),
        "AlphaService",
        Some("packages/notes/index.md"),
        Some(4),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected markdown-observe definition resolve request to succeed");
    };

    assert_studio_json_snapshot(
        "search_definition_markdown_observe_hint_payload",
        json!({
            "query": response.query,
            "sourcePath": response.source_path,
            "sourceLine": response.source_line,
            "candidateCount": response.candidate_count,
            "selectedScope": response.selected_scope,
            "navigationTarget": {
                "path": response.navigation_target.path,
                "category": response.navigation_target.category,
                "projectName": response.navigation_target.project_name,
                "rootLabel": response.navigation_target.root_label,
                "line": response.navigation_target.line,
                "lineEnd": response.navigation_target.line_end,
                "column": response.navigation_target.column,
            },
            "definition": {
                "name": response.definition.name,
                "signature": response.definition.signature,
                "path": response.definition.path,
                "language": response.definition.language,
                "crateName": response.definition.crate_name,
                "projectName": response.definition.project_name,
                "rootLabel": response.definition.root_label,
                "lineStart": response.definition.line_start,
                "lineEnd": response.definition.line_end,
                "score": round_f64(response.definition.score),
            },
        }),
    );
}

#[tokio::test]
async fn search_definition_falls_back_to_fuzzy_symbol_match() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn build_service() {\n    let _service = AlphaService::new();\n}\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_definition_response(
        fixture.state.studio.as_ref(),
        "AlphaServic",
        Some("packages/rust/crates/demo/src/lib.rs"),
        Some(2),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected fuzzy definition resolve request to succeed");
    };

    assert_eq!(
        response.definition.path,
        "packages/rust/crates/demo/src/service.rs"
    );
    assert!(response.candidate_count >= 1);
}

#[tokio::test]
async fn search_definition_can_resolve_markdown_heading_hits() {
    let fixture = make_state_with_docs(vec![(
        "packages/notes/guide.md",
        "# AlphaService Guide\n\nThis note explains the service.\n",
    )]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_definition_response(
        fixture.state.studio.as_ref(),
        "AlphaService Guide",
        Some("packages/notes/guide.md"),
        Some(1),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected markdown-backed definition resolve request to succeed");
    };

    assert_eq!(response.definition.language, "markdown");
    assert_eq!(response.definition.path, "packages/notes/guide.md");
}
