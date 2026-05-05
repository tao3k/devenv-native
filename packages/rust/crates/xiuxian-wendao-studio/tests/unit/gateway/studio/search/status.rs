use super::{
    Arc, State, assert_search_status_knowledge_cold_start, assert_search_status_repeat_work,
    build_knowledge_search_response, make_state_with_docs, publish_knowledge_section_index,
    publish_repo_bundle_for_search_status, ready_repo_status_rows, search_index_status,
    search_index_status_payload_view,
};

#[tokio::test]
async fn search_index_status_reports_test_configured_owner_seed_repeat_work() {
    let fixture = make_state_with_docs(vec![(
        "alpha.md",
        "# Alpha\n\nThis note contains search target keyword: wendao.\n",
    )]);

    let partial = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "wendao",
        5,
        Some("semantic_lookup".to_string()),
    )
    .await
    .unwrap_or_else(|error| panic!("cold-start knowledge search should succeed: {error:?}"));
    assert!(partial.partial);

    publish_knowledge_section_index(&fixture.state).await;

    let ready = build_knowledge_search_response(
        fixture.state.studio.as_ref(),
        "wendao",
        5,
        Some("semantic_lookup".to_string()),
    )
    .await
    .unwrap_or_else(|error| panic!("ready knowledge search should succeed: {error:?}"));
    assert!(!ready.partial);

    let payload = serde_json::to_value(
        search_index_status(State(Arc::clone(&fixture.state)))
            .await
            .unwrap_or_else(|error| panic!("status handler should resolve: {error:?}"))
            .0,
    )
    .unwrap_or_else(|error| panic!("serialize status payload: {error}"));
    assert_search_status_knowledge_cold_start(&payload);
    assert_search_status_repeat_work(&payload);
}

#[tokio::test]
async fn search_index_status_handler_is_stable_for_reordered_ready_published_repo_rows() {
    let fixture = make_state_with_docs(Vec::new());
    publish_repo_bundle_for_search_status(&fixture.state, "alpha/repo").await;
    publish_repo_bundle_for_search_status(&fixture.state, "beta/repo").await;

    fixture
        .state
        .studio
        .search_plane
        .synchronize_repo_runtime_for_test(&ready_repo_status_rows(&["alpha/repo", "beta/repo"]))
        .await;
    let left_payload = serde_json::to_value(
        search_index_status(State(Arc::clone(&fixture.state)))
            .await
            .unwrap_or_else(|error| panic!("left status handler should resolve: {error:?}"))
            .0,
    )
    .unwrap_or_else(|error| panic!("serialize left status payload: {error}"));

    fixture
        .state
        .studio
        .search_plane
        .clear_all_in_memory_repo_runtime_for_test();
    fixture
        .state
        .studio
        .search_plane
        .synchronize_repo_runtime_for_test(&ready_repo_status_rows(&["beta/repo", "alpha/repo"]))
        .await;
    let right_payload = serde_json::to_value(
        search_index_status(State(Arc::clone(&fixture.state)))
            .await
            .unwrap_or_else(|error| panic!("right status handler should resolve: {error:?}"))
            .0,
    )
    .unwrap_or_else(|error| panic!("serialize right status payload: {error}"));

    assert_eq!(
        search_index_status_payload_view(&left_payload),
        search_index_status_payload_view(&right_payload)
    );
}
