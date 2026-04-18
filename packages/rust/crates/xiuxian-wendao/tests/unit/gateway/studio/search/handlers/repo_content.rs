use crate::gateway::studio::search::handlers::code_search::build_repo_content_search_hits;
use crate::gateway::studio::search::handlers::tests::{
    publish_repo_content_chunk_index, test_studio_state,
};
use crate::repo_index::RepoCodeDocument;
use crate::search::{SearchCorpusKind, SearchQueryTelemetrySource};
use std::sync::Arc;

#[tokio::test]
async fn repo_content_search_hits_find_matching_julia_source_lines() {
    let studio = test_studio_state();
    publish_repo_content_chunk_index(
        &studio,
        "sciml",
        vec![RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 67,
            modified_unix_ms: 0,
        }],
    )
    .await;

    let hits = build_repo_content_search_hits(&studio, "sciml", "lang:julia reexport", 10)
        .await
        .unwrap_or_else(|error| panic!("repo content search hits: {error:?}"));

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].doc_type.as_deref(), Some("file"));
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
    assert_eq!(hits[0].title.as_deref(), Some("src/BaseModelica.jl"));
    assert_eq!(hits[0].match_reason.as_deref(), Some("repo_content_search"));
    assert!(
        hits[0].tags.iter().any(|tag| tag == "kind:file"),
        "expected flight-backed repo-content bridge to preserve `kind:file`: {:?}",
        hits[0].tags
    );
    assert!(
        hits[0].tags.iter().any(|tag| tag == "lang:julia"),
        "expected flight-backed repo-content bridge to preserve `lang:julia`: {:?}",
        hits[0].tags
    );
    assert_eq!(
        hits[0]
            .navigation_target
            .as_ref()
            .map(|target| target.path.as_str()),
        Some("sciml/src/BaseModelica.jl")
    );
    assert_eq!(
        hits[0]
            .navigation_target
            .as_ref()
            .map(|target| target.category.as_str()),
        Some("repo_code")
    );
    assert_eq!(
        hits[0]
            .navigation_target
            .as_ref()
            .and_then(|target| target.line),
        Some(3)
    );
}

#[tokio::test]
async fn repo_content_search_hits_find_matching_code_punctuation_queries() {
    let studio = test_studio_state();
    publish_repo_content_chunk_index(
        &studio,
        "sciml",
        vec![RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 67,
            modified_unix_ms: 0,
        }],
    )
    .await;

    let hits = build_repo_content_search_hits(&studio, "sciml", "@reexport", 10)
        .await
        .unwrap_or_else(|error| panic!("repo content punctuation search hits: {error:?}"));

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "src/BaseModelica.jl");
    assert_eq!(
        hits[0]
            .navigation_target
            .as_ref()
            .and_then(|target| target.line),
        Some(3)
    );
}

#[tokio::test]
async fn repo_content_search_hits_record_query_core_telemetry_into_search_plane_status() {
    let studio = test_studio_state();
    publish_repo_content_chunk_index(
        &studio,
        "sciml",
        vec![RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 67,
            modified_unix_ms: 0,
        }],
    )
    .await;

    let hits = build_repo_content_search_hits(&studio, "sciml", "lang:julia reexport", 10)
        .await
        .unwrap_or_else(|error| panic!("repo content search hits: {error:?}"));
    assert_eq!(hits.len(), 1);

    let snapshot = studio.search_plane.status();
    let repo_content = snapshot
        .corpora
        .iter()
        .find(|status| status.corpus == SearchCorpusKind::RepoContentChunk)
        .unwrap_or_else(|| panic!("repo content status should be present"));
    let telemetry = repo_content
        .last_query_telemetry
        .as_ref()
        .unwrap_or_else(|| panic!("repo content telemetry should be present"));

    assert_eq!(telemetry.scope.as_deref(), Some("sciml"));
    assert_eq!(telemetry.source, SearchQueryTelemetrySource::Scan);
    assert_eq!(telemetry.result_count, 1);
    assert_eq!(telemetry.matched_rows, 1);
}
