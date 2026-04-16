use super::test_prelude::*;
use super::*;
use crate::analyzers::{
    ExampleRecord, ModuleRecord, RepoSymbolKind, RepositoryAnalysisOutput, SymbolRecord,
};
use crate::gateway::studio::build_ast_index;
use crate::gateway::studio::{GatewayState, StudioState};
use crate::gateway::studio::search::handlers::knowledge::ensure_intent_indices;
use crate::gateway::studio::search::handlers::status::search_index_status;
use crate::gateway::studio::search::strip_option;
use crate::gateway::studio::test_support::{assert_studio_json_snapshot, round_f64};
use crate::gateway::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
    RepoIndexStatusResponse,
};
use crate::search::SearchPlaneService;
use serde_json::json;
use tempfile::tempdir;

mod ast;
mod attachments;
mod autocomplete;
mod code_search_intent;
mod content;
mod definition_api;
mod intent;
mod knowledge;
mod references_symbols;
mod status;

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
    telemetry: &'a crate::gateway::studio::StudioSearchColdStartTelemetry,
    corpus: &str,
) -> &'a crate::gateway::studio::StudioSearchColdStartCorpusTelemetry {
    telemetry
        .corpora
        .iter()
        .find(|entry| entry.corpus == corpus)
        .unwrap_or_else(|| panic!("missing cold-start telemetry corpus `{corpus}`"))
}

fn status_corpus_entry<'a>(payload: &'a serde_json::Value, corpus: &str) -> &'a serde_json::Value {
    payload
        .get("corpora")
        .and_then(serde_json::Value::as_array)
        .and_then(|corpora| {
            corpora.iter().find(|entry| {
                entry
                    .get("corpus")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value == corpus)
            })
        })
        .unwrap_or_else(|| panic!("status payload should include `{corpus}` corpus row"))
}

fn parse_payload_time(
    value: &serde_json::Value,
    field: &str,
) -> chrono::DateTime<chrono::FixedOffset> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .and_then(|entry| chrono::DateTime::parse_from_rfc3339(entry).ok())
        .unwrap_or_else(|| panic!("payload field `{field}` should be RFC3339"))
}

fn assert_search_status_knowledge_cold_start(payload: &serde_json::Value) {
    let cold_start = payload
        .get("coldStartTelemetry")
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("status payload should include coldStartTelemetry"));
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
    let status_knowledge = status_corpus_entry(payload, "knowledge_section");

    assert_eq!(
        cold_start
            .get("coldStartWindowMs")
            .and_then(serde_json::Value::as_u64),
        Some(60_000)
    );
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

    let observed_ready_at = knowledge.get("firstReadyObserved").map_or_else(
        || panic!("firstReadyObserved should be present"),
        |value| parse_payload_time(value, "recordedAt"),
    );
    let current_build_finished_at = parse_payload_time(status_knowledge, "buildFinishedAt");
    assert!(observed_ready_at <= current_build_finished_at);
}

fn assert_search_status_repeat_work(payload: &serde_json::Value) {
    let repeat_work = payload
        .get("coldStartTelemetry")
        .and_then(|value| value.get("diagnostics"))
        .and_then(|value| value.get("repeatWork"))
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("coldStartTelemetry should include diagnostics.repeatWork"));
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
