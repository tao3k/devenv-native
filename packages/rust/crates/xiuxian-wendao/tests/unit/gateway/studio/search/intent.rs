use super::*;

#[tokio::test]
async fn search_intent_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("   ".to_string()),
            intent: Some("debug_lookup".to_string()),
            limit: None,
            repo: None,
        },
    )
    .await;

    let Err(error) = result else {
        panic!("expected missing-query intent request to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn knowledge_intent_uses_shared_scan_bundle_to_start_indices() {
    let fixture = make_state_with_docs(vec![
        (
            "docs/alpha.md",
            "# Alpha\n\nIntent search should share its startup scan.\n",
        ),
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn intent_shared_scan() {}\n",
        ),
    ]);

    let index_state = ensure_intent_indices(fixture.state.studio.as_ref());
    assert!(!index_state.knowledge_config_missing);
    assert!(!index_state.symbol_config_missing);

    let telemetry = fixture.state.studio.search_plane.repeat_work_telemetry();
    assert!(
        telemetry.source_operations.iter().any(|entry| {
            entry.source == "knowledge_intent"
                && entry.operation == "scan_supported_project_files"
                && entry.file_observation_count >= 2
        }),
        "knowledge intent should record the shared scan bundle"
    );
    assert!(
        telemetry.source_operations.iter().all(|entry| {
            !((entry.source == "knowledge_section.fingerprint"
                && entry.operation == "scan_note_project_files")
                || (entry.source == "local_symbol.fingerprint"
                    && entry.operation == "scan_symbol_project_files"))
        }),
        "knowledge intent should avoid starting its note and symbol corpora with separate scans"
    );
}

#[tokio::test]
async fn search_intent_returns_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "alpha.md",
            "# Alpha\n\nIntent search keyword: alpha_handler.\n",
        ),
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn alpha_handler() {}\n",
        ),
    ]);
    publish_knowledge_section_index(&fixture.state).await;
    publish_local_symbol_index(&fixture.state).await;

    let result = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("alpha_handler".to_string()),
            limit: Some(5),
            intent: Some("debug_lookup".to_string()),
            repo: None,
        },
    )
    .await;

    let Ok((response, _metadata)) = result else {
        panic!("expected intent search request to succeed");
    };
    let mut hits = response.hits;
    hits.sort_by(|left, right| {
        let left_kind = left.doc_type.as_deref().unwrap_or_default();
        let right_kind = right.doc_type.as_deref().unwrap_or_default();
        (
            left_kind != "symbol",
            left.path.as_str(),
            left.stem.as_str(),
        )
            .cmp(&(
                right_kind != "symbol",
                right.path.as_str(),
                right.stem.as_str(),
            ))
    });

    let payload = json!({
        "query": response.query,
        "hitCount": response.hit_count,
        "selectedMode": response.selected_mode,
        "searchMode": response.search_mode,
        "intent": response.intent,
        "intentConfidence": response.intent_confidence.map(round_f64),
        "graphConfidenceScore": response.graph_confidence_score.map(round_f64),
        "hits": hits.into_iter().map(|hit| {
            json!({
                "stem": hit.stem,
                "title": hit.title,
                "path": hit.path,
                "docType": hit.doc_type,
                "score": round_f64(hit.score),
                "bestSection": hit.best_section,
                "matchReason": hit.match_reason,
            })
        }).collect::<Vec<_>>(),
    });

    assert_eq!(payload["query"], json!("alpha_handler"));
    assert_eq!(payload["hitCount"], json!(2));
    assert_eq!(payload["selectedMode"], json!("intent_hybrid"));
    assert_eq!(payload["searchMode"], json!("intent_hybrid"));
    assert_eq!(payload["intent"], json!("debug_lookup"));
    assert_eq!(payload["intentConfidence"], json!(1.0));
    assert_eq!(payload["graphConfidenceScore"], json!(1.0));

    let hits = payload["hits"]
        .as_array()
        .unwrap_or_else(|| panic!("intent payload should include hits array"));
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().any(|hit| {
        hit["stem"] == json!("alpha_handler")
            && hit["title"] == json!("alpha_handler")
            && hit["path"] == json!("packages/rust/crates/demo/src/lib.rs")
            && hit["docType"] == json!("symbol")
            && hit["matchReason"] == json!("local_symbol_search")
    }));
    assert!(hits.iter().any(|hit| {
        hit["stem"] == json!("alpha")
            && hit["title"] == json!("Alpha")
            && hit["path"] == json!("alpha.md")
            && hit["matchReason"] == json!("knowledge_section_search")
    }));
}

#[tokio::test]
async fn search_intent_includes_repo_content_hits_for_code_biased_intent() {
    let fixture = make_state_with_docs(Vec::new());
    let repo_root = fixture.temp_dir.path().join("ValidPkg");
    std::fs::create_dir_all(repo_root.join("src"))
        .unwrap_or_else(|error| panic!("create repo src: {error}"));
    std::fs::write(
        repo_root.join("Project.toml"),
        "name = \"ValidPkg\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
    )
    .unwrap_or_else(|error| panic!("write project file: {error}"));

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects: fixture.state.studio.configured_projects(),
            repo_projects: vec![UiRepoProjectConfig {
                id: "valid".to_string(),
                root: Some(repo_root.display().to_string()),
                url: None,
                git_ref: None,
                refresh: None,
                plugins: vec!["julia".to_string()],
            }],
        });
    let snapshot = Arc::new(RepoIndexSnapshot {
        repo_id: "valid".to_string(),
        analysis: Arc::new(crate::analyzers::RepositoryAnalysisOutput::default()),
    });
    publish_repo_content_chunk_index(
        &fixture.state,
        "valid",
        vec![crate::repo_index::RepoCodeDocument {
            path: "src/ValidPkg.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module ValidPkg\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 62,
            modified_unix_ms: 0,
        }],
    )
    .await;
    fixture
        .state
        .studio
        .repo_index
        .set_snapshot_for_test(&snapshot);
    fixture
        .state
        .studio
        .repo_index
        .set_status_for_test(RepoIndexEntryStatus {
            repo_id: "valid".to_string(),
            phase: RepoIndexPhase::Ready,
            queue_position: None,
            last_error: None,
            last_revision: Some("abc123".to_string()),
            updated_at: Some("2026-03-22T00:00:00Z".to_string()),
            attempt_count: 1,
        });

    let result = load_intent_search_response_with_metadata(
        fixture.state.studio.as_ref(),
        SearchQuery {
            q: Some("lang:julia reexport".to_string()),
            limit: Some(5),
            intent: Some("debug_lookup".to_string()),
            repo: Some("valid".to_string()),
        },
    )
    .await;

    let Ok((response, _metadata)) = result else {
        panic!("expected repo-backed intent search request to succeed");
    };

    assert_eq!(response.selected_mode.as_deref(), Some("intent_hybrid"));
    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("file") && hit.path == "src/ValidPkg.jl"),
        "expected repo content hit in intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
}
