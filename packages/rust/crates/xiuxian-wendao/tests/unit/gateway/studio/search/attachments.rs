use super::{
    AttachmentSearchQuery, UiConfig, UiProjectConfig, assert_studio_json_snapshot, json,
    load_attachment_search_response_from_studio, make_state_with_docs, publish_attachment_index,
    round_f64,
};

#[tokio::test]
async fn search_attachments_requires_query() {
    let fixture = make_state_with_docs(Vec::new());

    let result = load_attachment_search_response_from_studio(
        fixture.state.studio.as_ref(),
        AttachmentSearchQuery {
            q: Some("   ".to_string()),
            limit: None,
            ext: Vec::new(),
            kind: Vec::new(),
            case_sensitive: false,
        },
    )
    .await;

    let Err(error) = result else {
        panic!("expected missing-query attachment search to fail");
    };

    assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_QUERY");
}

#[tokio::test]
async fn search_attachments_returns_payload() {
    let fixture = make_state_with_docs(vec![
        (
            "docs/alpha.md",
            "# Alpha\n\n![Topology](assets/topology.png)\n\n[Spec](files/spec.pdf)\n",
        ),
        ("docs/beta.md", "# Beta\n\n![Avatar](images/avatar.jpg)\n"),
    ]);
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
    publish_attachment_index(&fixture.state).await;

    let result = load_attachment_search_response_from_studio(
        fixture.state.studio.as_ref(),
        AttachmentSearchQuery {
            q: Some("topology".to_string()),
            limit: Some(10),
            ext: Vec::new(),
            kind: Vec::new(),
            case_sensitive: false,
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected attachment search request to succeed");
    };

    assert_studio_json_snapshot(
        "search_attachments_payload",
        json!({
            "query": response.query,
            "hitCount": response.hit_count,
            "selectedScope": response.selected_scope,
            "hits": response.hits.into_iter().map(|hit| {
                json!({
                    "path": hit.path,
                    "sourceId": hit.source_id,
                    "sourceStem": hit.source_stem,
                    "sourceTitle": hit.source_title,
                    "sourcePath": hit.source_path,
                    "attachmentId": hit.attachment_id,
                    "attachmentPath": hit.attachment_path,
                    "attachmentName": hit.attachment_name,
                    "attachmentExt": hit.attachment_ext,
                    "kind": hit.kind,
                    "score": round_f64(hit.score),
                    "visionSnippet": hit.vision_snippet,
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn search_attachments_returns_partial_response_before_initial_index_publication() {
    let fixture = make_state_with_docs(vec![(
        "docs/alpha.md",
        "# Alpha\n\n![Topology](assets/topology.png)\n\n[Spec](files/spec.pdf)\n",
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

    let result = load_attachment_search_response_from_studio(
        fixture.state.studio.as_ref(),
        AttachmentSearchQuery {
            q: Some("topology".to_string()),
            limit: Some(10),
            ext: Vec::new(),
            kind: Vec::new(),
            case_sensitive: false,
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected cold-start attachment search request to succeed");
    };

    assert_eq!(response.hit_count, 0);
    assert!(response.partial);
    assert_eq!(response.indexing_state.as_deref(), Some("indexing"));
    assert!(response.hits.is_empty());
}

#[tokio::test]
async fn search_attachments_respects_extension_and_kind_filters() {
    let fixture = make_state_with_docs(vec![(
        "docs/alpha.md",
        "# Alpha\n\n![Topology](assets/topology.png)\n\n[Spec](files/spec.pdf)\n",
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
    publish_attachment_index(&fixture.state).await;

    let result = load_attachment_search_response_from_studio(
        fixture.state.studio.as_ref(),
        AttachmentSearchQuery {
            q: Some("spec".to_string()),
            limit: Some(10),
            ext: vec!["pdf".to_string()],
            kind: vec!["pdf".to_string()],
            case_sensitive: false,
        },
    )
    .await;

    let Ok(response) = result else {
        panic!("expected filtered attachment search request to succeed");
    };

    assert_eq!(response.hit_count, 1);
    assert_eq!(response.hits[0].attachment_name, "spec.pdf");
    assert_eq!(response.hits[0].attachment_ext, "pdf");
    assert_eq!(response.hits[0].kind, "pdf");
}
