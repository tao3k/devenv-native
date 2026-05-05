use super::{
    UiConfig, UiProjectConfig, assert_studio_json_snapshot, build_knowledge_search_response,
    cold_start_corpus, json, make_state_with_docs, publish_knowledge_section_index, round_f64,
};

#[tokio::test]
async fn search_knowledge_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "   ",
        10,
        Some("semantic_lookup".to_string()),
    )
    .await;

    let Err(error) = result else {
        panic!("expected missing-query request to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn search_knowledge_returns_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "alpha.md",
            "# Alpha\n\nThis note contains search target keyword: wendao.\n",
        ),
        (
            "beta.md",
            "# Beta\n\nAnother note mentions wendao in text.\n",
        ),
    ]);
    publish_knowledge_section_index(&fixture.state).await;

    let result = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "wendao",
        5,
        Some("semantic_lookup".to_string()),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_payload",
        json!({
            "query": response.query,
            "hitCount": response.hit_count,
            "selectedMode": response.selected_mode,
            "searchMode": response.search_mode,
            "intent": response.intent,
            "intentConfidence": response.intent_confidence.map(round_f64),
            "graphConfidenceScore": response.graph_confidence_score.map(round_f64),
            "hits": response.hits.into_iter().map(|hit| {
                json!({
                    "stem": hit.stem,
                    "title": hit.title,
                    "path": hit.path,
                    "docType": hit.doc_type,
                    "tags": hit.tags,
                    "score": round_f64(hit.score),
                    "bestSection": hit.best_section,
                    "matchReason": hit.match_reason,
                    "hierarchicalUri": hit.hierarchical_uri,
                    "hierarchy": hit.hierarchy,
                    "saliencyScore": hit.saliency_score.map(round_f64),
                    "auditStatus": hit.audit_status,
                    "verificationState": hit.verification_state,
                    "implicitBacklinks": hit.implicit_backlinks,
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_knowledge_returns_partial_response_before_initial_index_publication() {
    let fixture = make_state_with_docs(vec![(
        "alpha.md",
        "# Alpha\n\nThis note contains search target keyword: wendao.\n",
    )]);

    let result = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "wendao",
        5,
        Some("semantic_lookup".to_string()),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected cold-start knowledge search request to succeed");
    };

    assert_eq!(response.hit_count, 0);
    assert!(response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("indexing"));
    assert!(response.hits.is_empty());

    let telemetry = fixture.state.studio.search_cold_start_telemetry();
    let knowledge = cold_start_corpus(&telemetry, "knowledge_section");
    assert_eq!(
        knowledge
            .first_partial_search_response
            .as_ref()
            .and_then(|event| event.source.as_deref()),
        Some("knowledge_search")
    );
    assert!(knowledge.first_ready_search_response.is_none());
}

#[tokio::test]
async fn search_knowledge_uses_studio_display_paths() {
    let fixture = make_state_with_docs(vec![
        (
            "docs/alpha.md",
            "# Alpha\n\nThis note contains search target keyword: wendao.\n",
        ),
        (
            "docs/beta.md",
            "# Beta\n\nAnother note mentions wendao in text.\n",
        ),
    ]);
    publish_knowledge_section_index(&fixture.state).await;

    let result = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "wendao",
        5,
        Some("semantic_lookup".to_string()),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected search request to succeed");
    };
    let hit_paths = response
        .hits
        .iter()
        .map(|hit| hit.path.clone())
        .collect::<Vec<_>>();

    assert_studio_json_snapshot(
        "search_display_paths_payload",
        json!({
            "query": response.query,
            "hitCount": response.hit_count,
            "selectedMode": response.selected_mode,
            "paths": hit_paths.clone(),
        }),
    );

    if hit_paths.is_empty() {
        assert_eq!(response.selected_mode.as_deref(), Some("vector_only"));
        return;
    }

    assert!(
        hit_paths
            .iter()
            .all(|path| !std::path::Path::new(path).is_absolute()),
        "unexpected absolute hit paths: {hit_paths:?}",
    );
    assert!(
        hit_paths.iter().all(|path| !path.contains('\\')),
        "unexpected non-normalized hit paths: {hit_paths:?}",
    );
    assert!(
        hit_paths.iter().any(|path| path.ends_with("alpha.md")),
        "unexpected hit paths: {hit_paths:?}",
    );
}

#[tokio::test]
async fn search_knowledge_uses_project_scoped_display_paths_for_duplicate_roots() {
    let fixture = make_state_with_docs(vec![
        (
            "docs/kernel.md",
            "# Kernel\n\nThis note contains search target keyword: wendao.\n",
        ),
        (
            ".data/wendao-frontend/docs/main.md",
            "# Main\n\nThis note also contains search target keyword: wendao.\n",
        ),
    ]);
    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: vec![
                UiProjectConfig {
                    name: "kernel".to_string(),
                    root: ".".to_string(),
                    dirs: vec!["docs".to_string()],
                },
                UiProjectConfig {
                    name: "main".to_string(),
                    root: ".data/wendao-frontend".to_string(),
                    dirs: vec!["docs".to_string()],
                },
            ],
            repo_projects: Vec::new(),
        });
    publish_knowledge_section_index(&fixture.state).await;

    let result = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "wendao",
        10,
        Some("semantic_lookup".to_string()),
    )
    .await;

    let Ok(response) = result else {
        panic!("expected project-scoped search request to succeed");
    };
    let hit_paths = response
        .hits
        .iter()
        .map(|hit| hit.path.as_str())
        .collect::<Vec<_>>();

    assert!(
        hit_paths.contains(&"kernel/docs/kernel.md"),
        "missing kernel project display path: {hit_paths:?}",
    );
    assert!(
        hit_paths.contains(&"main/docs/main.md"),
        "missing main project display path: {hit_paths:?}",
    );
}
