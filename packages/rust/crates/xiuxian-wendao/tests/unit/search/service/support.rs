use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{ExampleRecord, ModuleRecord, RepoSymbolKind, SymbolRecord};
pub(super) use crate::repo_index::{RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase};
pub(super) use crate::search::contracts::{AstSearchHit, StudioNavigationTarget};
pub(super) use crate::search::{
    SearchCorpusIssueSummary, SearchCorpusKind, SearchCorpusStatus, SearchCorpusStatusAction,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchManifestKeyspace,
    SearchPlaneService, SearchPlaneStatusSnapshot, SearchQueryTelemetry,
};

static TEST_KEYSPACE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(super) fn repo_phase(service: &SearchPlaneService, repo_id: &str) -> Option<RepoIndexPhase> {
    service.repo_runtime_state(repo_id).map(|state| state.phase)
}

pub(super) fn unique_test_manifest_keyspace(label: &str) -> SearchManifestKeyspace {
    let suffix = TEST_KEYSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    SearchManifestKeyspace::new(format!("xiuxian:test:search_plane:{label}:{suffix}"))
}

pub(super) fn service_test_manifest_keyspace() -> SearchManifestKeyspace {
    unique_test_manifest_keyspace("service")
}

pub(super) fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

pub(super) fn ok_or_panic<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn some_or_panic<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

pub(super) fn corpus_status<'a>(
    snapshot: &'a SearchPlaneStatusSnapshot,
    corpus: SearchCorpusKind,
    context: &str,
) -> &'a SearchCorpusStatus {
    some_or_panic(
        snapshot.corpora.iter().find(|entry| entry.corpus == corpus),
        context,
    )
}

pub(super) fn issue_summary<'a>(
    status: &'a SearchCorpusStatus,
    context: &str,
) -> &'a SearchCorpusIssueSummary {
    some_or_panic(status.issue_summary.as_ref(), context)
}

pub(super) fn last_query_telemetry<'a>(
    status: &'a SearchCorpusStatus,
    context: &str,
) -> &'a SearchQueryTelemetry {
    some_or_panic(status.last_query_telemetry.as_ref(), context)
}

pub(super) async fn publish_repo_bundle(
    service: &SearchPlaneService,
    repo_id: &str,
    documents: &[RepoCodeDocument],
    revision: Option<&str>,
) {
    ok_or_panic(
        service
            .publish_repo_entities_with_revision(
                repo_id,
                &sample_repo_analysis(),
                documents,
                revision,
            )
            .await,
        "publish repo entities",
    );
    ok_or_panic(
        service
            .publish_repo_content_chunks_with_revision(repo_id, documents, revision)
            .await,
        "publish repo content chunks",
    );
}

pub(super) fn sample_hit() -> AstSearchHit {
    AstSearchHit {
        name: "AlphaSymbol".to_string(),
        signature: "fn AlphaSymbol()".to_string(),
        path: "src/lib.rs".to_string(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        node_kind: None,
        owner_title: None,
        navigation_target: StudioNavigationTarget {
            path: "src/lib.rs".to_string(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(1),
            line_end: Some(1),
            column: Some(1),
        },
        line_start: 1,
        line_end: 1,
        score: 0.0,
    }
}

pub(super) fn sample_repo_analysis() -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: "alpha/repo".to_string().into(),
            module_id: "module:BaseModelica".to_string().into(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string().into(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: "alpha/repo".to_string().into(),
            symbol_id: "symbol:reexport".to_string().into(),
            module_id: Some("module:BaseModelica".to_string().into()),
            name: "reexport".to_string(),
            qualified_name: "BaseModelica.reexport".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string().into(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some("reexport()".to_string()),
            audit_status: Some("verified".to_string().into()),
            verification_state: Some("verified".to_string().into()),
            attributes: std::collections::BTreeMap::new(),
        }],
        examples: vec![ExampleRecord {
            repo_id: "alpha/repo".to_string().into(),
            example_id: "example:reexport".to_string().into(),
            title: "Reexport example".to_string(),
            path: "examples/reexport.jl".to_string().into(),
            summary: Some("Shows how to reexport ModelingToolkit".to_string()),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

pub(super) fn sample_repo_documents() -> Vec<RepoCodeDocument> {
    vec![
        RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nexport reexport\nreexport() = nothing\nend\n",
            ),
            size_bytes: 61,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "examples/reexport.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("using BaseModelica\nreexport()\n"),
            size_bytes: 29,
            modified_unix_ms: 10,
        },
    ]
}

pub(super) fn repo_status_entry(repo_id: &str, phase: RepoIndexPhase) -> RepoIndexEntryStatus {
    RepoIndexEntryStatus {
        repo_id: repo_id.to_string(),
        phase,
        queue_position: None,
        last_error: None,
        last_revision: Some("rev-1".to_string()),
        updated_at: Some("2026-03-22T12:00:00Z".to_string()),
        attempt_count: 1,
    }
}

pub(super) fn assert_status_reason(
    status: &SearchCorpusStatus,
    code: SearchCorpusStatusReasonCode,
    severity: SearchCorpusStatusSeverity,
    action: SearchCorpusStatusAction,
    readable: bool,
) {
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should be present"));
    assert_eq!(reason.code, code);
    assert_eq!(reason.severity, severity);
    assert_eq!(reason.action, action);
    assert_eq!(reason.readable, readable);
}
