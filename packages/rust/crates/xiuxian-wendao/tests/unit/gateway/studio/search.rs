use super::test_prelude::*;
use super::*;
use crate::analyzers::{
    ExampleRecord, ModuleRecord, RepoSymbolKind, RepositoryAnalysisOutput, SymbolRecord,
};
use crate::gateway::studio::build_ast_index;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::search::handlers::knowledge::intent::ensure_intent_indices;
use crate::gateway::studio::search::handlers::status::search_index_status;
use crate::gateway::studio::search::support::strip_option;
use crate::gateway::studio::test_support::{assert_studio_json_snapshot, round_f64};
use crate::gateway::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
    RepoIndexStatusResponse,
};
use crate::search::SearchPlaneService;
use chrono::DateTime;
use serde_json::json;
use tempfile::tempdir;

#[path = "search/code_search_intent.rs"]
mod code_search_intent;

struct StudioStateFixture {
    state: Arc<GatewayState>,
    temp_dir: tempfile::TempDir,
}

fn create_temp_dir() -> tempfile::TempDir {
    match tempdir() {
        Ok(temp_dir) => temp_dir,
        Err(err) => panic!("failed to create temp dir fixture: {err}"),
    }
}

fn ok_or_panic<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn write_doc(root: &std::path::Path, name: &str, content: &str) {
    let path = root.join(name);
    if let Some(parent) = path.parent()
        && let Err(err) = std::fs::create_dir_all(parent)
    {
        panic!("failed to create fixture parent dirs for {name}: {err}");
    }
    if let Err(err) = std::fs::write(path, content) {
        panic!("failed to write fixture doc {name}: {err}");
    }
}

fn make_state_with_docs(docs: Vec<(&str, &str)>) -> StudioStateFixture {
    let temp_dir = create_temp_dir();
    for (name, content) in docs {
        write_doc(temp_dir.path(), name, content);
    }

    let mut studio_state = StudioState::new_with_bootstrap_ui_config(Arc::new(
        crate::analyzers::bootstrap_builtin_registry()
            .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
    ));
    studio_state.project_root = temp_dir.path().to_path_buf();
    studio_state.config_root = temp_dir.path().to_path_buf();
    studio_state.search_plane = SearchPlaneService::new(temp_dir.path().to_path_buf());
    studio_state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec![
                ".".to_string(),
                "packages".to_string(),
                ".data".to_string(),
                "internal_skills".to_string(),
            ],
        }],
        repo_projects: Vec::new(),
    });

    StudioStateFixture {
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio_state),
        }),
        temp_dir,
    }
}

fn cold_start_corpus<'a>(
    telemetry: &'a crate::gateway::studio::router::StudioSearchColdStartTelemetry,
    corpus: &str,
) -> &'a crate::gateway::studio::router::StudioSearchColdStartCorpusTelemetry {
    telemetry
        .corpora
        .iter()
        .find(|entry| entry.corpus == corpus)
        .unwrap_or_else(|| panic!("missing cold-start telemetry corpus `{corpus}`"))
}

fn ready_repo_status_rows(repo_ids: &[&str]) -> RepoIndexStatusResponse {
    RepoIndexStatusResponse {
        total: repo_ids.len(),
        active: 0,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: repo_ids.len(),
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: repo_ids
            .iter()
            .map(|repo_id| RepoIndexEntryStatus {
                repo_id: (*repo_id).to_string(),
                phase: RepoIndexPhase::Ready,
                queue_position: None,
                last_error: None,
                last_revision: Some("rev-1".to_string()),
                updated_at: Some("2026-04-14T12:00:00Z".to_string()),
                attempt_count: 1,
            })
            .collect(),
    }
}

async fn publish_repo_bundle_for_search_status(state: &Arc<GatewayState>, repo_id: &str) {
    let documents = vec![RepoCodeDocument {
        path: "src/BaseModelica.jl".to_string(),
        language: Some("julia".to_string()),
        contents: Arc::<str>::from(
            "module BaseModelica\nexport reexport\nreexport() = nothing\nend\n",
        ),
        size_bytes: 61,
        modified_unix_ms: 10,
    }];
    let analysis = RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: "symbol:reexport".to_string(),
            module_id: Some("module:BaseModelica".to_string()),
            name: "reexport".to_string(),
            qualified_name: "BaseModelica.reexport".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string(),
            line_start: Some(2),
            line_end: Some(3),
            signature: Some("reexport()".to_string()),
            audit_status: Some("verified".to_string()),
            verification_state: Some("verified".to_string()),
            attributes: std::collections::BTreeMap::new(),
        }],
        examples: vec![ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: "example:reexport".to_string(),
            title: "Reexport example".to_string(),
            path: "examples/reexport.jl".to_string(),
            summary: Some("Shows how to reexport ModelingToolkit".to_string()),
        }],
        ..RepositoryAnalysisOutput::default()
    };
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_repo_entities_with_revision(repo_id, &analysis, &documents, Some("rev-1"))
            .await,
        "publish repo entity status fixture",
    );
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_repo_content_chunks_with_revision(repo_id, &documents, Some("rev-1"))
            .await,
        "publish repo content status fixture",
    );
}

fn search_index_status_payload_view(payload: &serde_json::Value) -> serde_json::Value {
    json!({
        "total": payload.get("total").cloned().unwrap_or(serde_json::Value::Null),
        "idle": payload.get("idle").cloned().unwrap_or(serde_json::Value::Null),
        "indexing": payload.get("indexing").cloned().unwrap_or(serde_json::Value::Null),
        "ready": payload.get("ready").cloned().unwrap_or(serde_json::Value::Null),
        "degraded": payload.get("degraded").cloned().unwrap_or(serde_json::Value::Null),
        "failed": payload.get("failed").cloned().unwrap_or(serde_json::Value::Null),
        "compactionPending": payload
            .get("compactionPending")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "statusReason": payload
            .get("statusReason")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "maintenanceSummary": payload
            .get("maintenanceSummary")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "queryTelemetrySummary": payload
            .get("queryTelemetrySummary")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "repoReadPressure": payload
            .get("repoReadPressure")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "corpora": payload.get("corpora").cloned().unwrap_or(serde_json::Value::Null),
    })
}

async fn publish_local_symbol_index(state: &Arc<GatewayState>) {
    let projects = state.studio.configured_projects();
    let hits = build_ast_index(
        state.studio.project_root.as_path(),
        state.studio.config_root.as_path(),
        &projects,
    );
    let fingerprint = format!(
        "test:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                state.studio.project_root.display(),
                state.studio.config_root.display(),
                hits.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_local_symbol_hits(fingerprint.as_str(), &hits)
            .await,
        "publish local symbol epoch",
    );
}

async fn publish_reference_occurrence_index(state: &Arc<GatewayState>) {
    let projects = state.studio.configured_projects();
    let fingerprint = format!(
        "test:reference:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                state.studio.project_root.display(),
                state.studio.config_root.display(),
                projects.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_reference_occurrences_from_projects(
                state.studio.project_root.as_path(),
                state.studio.config_root.as_path(),
                &projects,
                fingerprint.as_str(),
            )
            .await,
        "publish reference occurrence epoch",
    );
}

async fn publish_attachment_index(state: &Arc<GatewayState>) {
    let projects = state.studio.configured_projects();
    let fingerprint = format!(
        "test:attachment:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                state.studio.project_root.display(),
                state.studio.config_root.display(),
                projects.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_attachments_from_projects(
                state.studio.project_root.as_path(),
                state.studio.config_root.as_path(),
                &projects,
                fingerprint.as_str(),
            )
            .await,
        "publish attachment epoch",
    );
}

async fn publish_knowledge_section_index(state: &Arc<GatewayState>) {
    let projects = state.studio.configured_projects();
    let fingerprint = format!(
        "test:knowledge:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                state.studio.project_root.display(),
                state.studio.config_root.display(),
                projects.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_knowledge_sections_from_projects(
                state.studio.project_root.as_path(),
                state.studio.config_root.as_path(),
                &projects,
                fingerprint.as_str(),
            )
            .await,
        "publish knowledge section epoch",
    );
}

async fn publish_repo_content_chunk_index(
    state: &Arc<GatewayState>,
    repo_id: &str,
    documents: Vec<crate::repo_index::RepoCodeDocument>,
) {
    ok_or_panic(
        state
            .studio
            .search_plane
            .publish_repo_content_chunks_with_revision(repo_id, &documents, None)
            .await,
        "publish repo content chunks",
    );
}

#[test]
fn test_strip_option() {
    assert_eq!(strip_option(""), None);
    assert_eq!(strip_option("value"), Some("value".to_string()));
    assert_eq!(strip_option(" value "), Some("value".to_string()));
}

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

    let cold_start = payload
        .get("coldStartTelemetry")
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("status payload should include coldStartTelemetry"));
    let repeat_work = cold_start
        .get("diagnostics")
        .and_then(|value| value.get("repeatWork"))
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("coldStartTelemetry should include diagnostics.repeatWork"));
    assert_eq!(
        cold_start
            .get("coldStartWindowMs")
            .and_then(serde_json::Value::as_u64),
        Some(60_000)
    );
    let corpora = cold_start
        .get("corpora")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("coldStartTelemetry should include corpora"));
    let knowledge = corpora
        .iter()
        .find(|entry| {
            entry
                .get("corpus")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|corpus| corpus == "knowledge_section")
        })
        .unwrap_or_else(|| panic!("knowledge_section telemetry should be present"));
    let status_knowledge = payload
        .get("corpora")
        .and_then(serde_json::Value::as_array)
        .and_then(|corpora| {
            corpora.iter().find(|entry| {
                entry
                    .get("corpus")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|corpus| corpus == "knowledge_section")
            })
        })
        .unwrap_or_else(|| panic!("status payload should include knowledge_section corpus row"));
    assert_eq!(
        knowledge
            .get("firstPartialSearchResponse")
            .and_then(|value| value.get("source"))
            .and_then(serde_json::Value::as_str),
        Some("knowledge_search")
    );
    assert_eq!(
        knowledge
            .get("firstReadySearchResponse")
            .and_then(|value| value.get("source"))
            .and_then(serde_json::Value::as_str),
        Some("knowledge_search")
    );
    let first_ready_observed_source = knowledge
        .get("firstReadyObserved")
        .and_then(|value| value.get("source"))
        .and_then(serde_json::Value::as_str);
    assert!(
        first_ready_observed_source.is_some_and(|source| {
            source == "knowledge_search"
                || source == "search_index_status"
                || source == "search_plane_bootstrap"
        }),
        "unexpected firstReadyObserved source: {first_ready_observed_source:?}, entry={knowledge:#?}"
    );
    let observed_ready_at = knowledge
        .get("firstReadyObserved")
        .and_then(|value| value.get("recordedAt"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .unwrap_or_else(|| panic!("firstReadyObserved.recordedAt should be RFC3339"));
    let current_build_finished_at = status_knowledge
        .get("buildFinishedAt")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .unwrap_or_else(|| panic!("status payload buildFinishedAt should be RFC3339"));
    assert!(observed_ready_at <= current_build_finished_at);
    let source_operations = repeat_work
        .get("sourceOperations")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("repeatWork should include sourceOperations"));
    assert!(
        source_operations.iter().any(|entry| {
            entry
                .get("source")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|source| source == "test_configured_owner_seed")
                && entry
                    .get("operation")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|operation| operation == "scan_supported_project_files")
        }),
        "repeat-work telemetry should capture the eager configured-owner seed shared scan"
    );
    assert_eq!(
        repeat_work
            .get("summary")
            .and_then(|value| value.get("findingCount"))
            .and_then(serde_json::Value::as_u64),
        repeat_work
            .get("findings")
            .and_then(serde_json::Value::as_array)
            .map(|findings| findings.len() as u64)
    );
    assert!(
        repeat_work
            .get("hotPaths")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|paths| !paths.is_empty()),
        "repeat-work telemetry should surface repeated hot paths after one cold-start build"
    );
    assert!(
        repeat_work
            .get("findings")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|findings| {
                findings.iter().any(|entry| {
                    entry
                        .get("kind")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|kind| kind == "cross_operation_hot_path")
                })
            }),
        "repeat-work telemetry should surface detector findings for repeated hot paths"
    );
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

    assert_studio_json_snapshot(
        "search_intent_payload",
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
                    "score": round_f64(hit.score),
                    "bestSection": hit.best_section,
                    "matchReason": hit.match_reason,
                })
            }).collect::<Vec<_>>(),
        }),
    );
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

#[tokio::test]
async fn autocomplete_limits_and_filters_prefix() {
    let fixture = make_state_with_docs(vec![
        (
            "doc.md",
            "# Search Design\n\nThis doc starts with Search and discusses Search.\n",
        ),
        ("note.md", "# Search Notes\n\nTaggable text.\n"),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_autocomplete_response(fixture.state.studio.as_ref(), "se", 2).await;

    let Ok(response) = result else {
        panic!("expected autocomplete request to succeed");
    };

    assert_studio_json_snapshot(
        "search_autocomplete_payload",
        json!({
            "prefix": response.prefix,
            "suggestions": response.suggestions.into_iter().map(|suggestion| {
                json!({
                    "text": suggestion.text,
                    "suggestionType": suggestion.suggestion_type,
                })
            }).collect::<Vec<_>>(),
        }),
    );
}

#[tokio::test]
async fn autocomplete_includes_code_symbols() {
    let fixture = make_state_with_docs(vec![
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub struct AlphaService;\npub fn alpha_handler() {}\n",
        ),
        (
            "packages/python/demo/tool.py",
            "class AlphaClient:\n    pass\n\ndef alpha_helper():\n    return None\n",
        ),
    ]);
    publish_local_symbol_index(&fixture.state).await;

    let result = build_autocomplete_response(fixture.state.studio.as_ref(), "al", 10).await;

    let Ok(response) = result else {
        panic!("expected code-symbol autocomplete request to succeed");
    };

    let suggestions = response
        .suggestions
        .into_iter()
        .map(|suggestion| (suggestion.text, suggestion.suggestion_type))
        .collect::<Vec<_>>();

    assert_eq!(
        suggestions,
        vec![
            ("AlphaClient".to_string(), "symbol".to_string()),
            ("AlphaService".to_string(), "symbol".to_string()),
            ("alpha_handler".to_string(), "symbol".to_string()),
            ("alpha_helper".to_string(), "symbol".to_string()),
        ]
    );
}

#[tokio::test]
async fn autocomplete_waits_for_initial_index_publication() {
    let fixture = make_state_with_docs(vec![(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
    )]);

    let result = build_autocomplete_response(fixture.state.studio.as_ref(), "al", 10).await;

    let Ok(response) = result else {
        panic!("expected cold-start autocomplete request to succeed");
    };

    assert!(
        response
            .suggestions
            .iter()
            .any(|suggestion| suggestion.text == "AlphaService")
    );
}

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

#[test]
fn repo_navigation_target_prefixes_repo_root_for_relative_paths() {
    let target = repo_navigation_target("mcl", "Modelica/package.mo", None, None, None);
    assert_eq!(target.path, "mcl/Modelica/package.mo");
    assert_eq!(target.category, "repo_code");
    assert_eq!(target.project_name.as_deref(), Some("mcl"));
    assert_eq!(target.root_label.as_deref(), Some("mcl"));
}

#[test]
fn repo_navigation_target_does_not_duplicate_existing_repo_root_prefix() {
    let target = repo_navigation_target("mcl", "mcl/Modelica/package.mo", None, None, None);
    assert_eq!(target.path, "mcl/Modelica/package.mo");
}

#[test]
fn parse_content_search_line_parses_ripgrep_output() {
    let parsed = parse_content_search_line(
        "/tmp/repo/src/DifferentialEquations.jl:42:@reexport using SciMLBase",
    );
    let Some((path, line_number, snippet)) = parsed else {
        panic!("expected ripgrep output to parse");
    };

    assert_eq!(path, "/tmp/repo/src/DifferentialEquations.jl");
    assert_eq!(line_number, 42);
    assert_eq!(snippet, "@reexport using SciMLBase");
}

#[test]
fn supported_code_extension_includes_julia_and_modelica() {
    assert!(is_supported_code_extension("src/Foo.jl"));
    assert!(is_supported_code_extension("Modelica/package.mo"));
    assert!(!is_supported_code_extension("docs/readme.md"));
}

#[test]
fn truncate_content_search_snippet_limits_output_length() {
    let value = "abcdefghijklmnopqrstuvwxyz";
    let truncated = truncate_content_search_snippet(value, 8);
    assert_eq!(truncated, "abcdefgh...");
}

#[test]
fn code_content_globs_do_not_exclude_cache_root() {
    assert!(!CODE_CONTENT_EXCLUDE_GLOBS.contains(&"!.cache/**"));
}

#[test]
fn language_filter_matches_julia_path_extensions() {
    let mut filters = std::collections::HashSet::new();
    filters.insert("julia".to_string());

    assert!(path_matches_language_filters(
        "src/BaseModelica.jl",
        &filters
    ));
    assert!(path_matches_language_filters(
        "src/generated/parser.julia",
        &filters
    ));
    assert!(!path_matches_language_filters("docs/index.md", &filters));
}
