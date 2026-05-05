use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use xiuxian_wendao_parsers::{
    SemanticConfidenceSource, SemanticObjectKind, SemanticProjectionStaleness, SemanticScopeBundle,
    SemanticScopeRequest, SemanticStatus, SemanticValidationReport, load_semantic_repository,
    semantic_scope_bundle,
};
use xiuxian_wendao_server::transport::{
    AnalysisFlightRouteResponse, CodeAstAnalysisFlightRouteProvider,
    MarkdownAnalysisFlightRouteProvider, SemanticScopeFlightRequest,
    SemanticScopeFlightRouteProvider,
};

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
};
use crate::studio::router::GatewayState;
use crate::studio::router::StudioApiError;
use crate::studio::router::retrieval_arrow::build_retrieval_chunks_flight_batch;

use super::{load_code_ast_analysis_response, load_markdown_analysis_response};

#[derive(Clone)]
pub(crate) struct StudioMarkdownAnalysisFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioMarkdownAnalysisFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioMarkdownAnalysisFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioMarkdownAnalysisFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl MarkdownAnalysisFlightRouteProvider for StudioMarkdownAnalysisFlightRouteProvider {
    async fn markdown_analysis_batch(
        &self,
        path: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let response = load_markdown_analysis_response(self.state.as_ref(), path)
            .await
            .map_err(|error| map_studio_api_error(&error))?;
        let batch = build_retrieval_chunks_flight_batch(response.retrieval_atoms.as_slice())?;
        let metadata = serde_json::to_vec(&serde_json::json!({
            "path": response.path,
            "documentHash": response.document_hash,
            "nodeCount": response.node_count,
            "edgeCount": response.edge_count,
            "nodes": response.nodes,
            "edges": response.edges,
            "projections": response.projections,
            "diagnostics": response.diagnostics,
        }))
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

#[derive(Clone)]
pub(crate) struct StudioCodeAstAnalysisFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioCodeAstAnalysisFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioCodeAstAnalysisFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioCodeAstAnalysisFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl CodeAstAnalysisFlightRouteProvider for StudioCodeAstAnalysisFlightRouteProvider {
    async fn code_ast_analysis_batch(
        &self,
        path: &str,
        repo_id: &str,
        line_hint: Option<usize>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let response =
            load_code_ast_analysis_response(self.state.as_ref(), path, repo_id, line_hint)
                .await
                .map_err(|error| map_studio_api_error(&error))?;
        let batch = build_retrieval_chunks_flight_batch(response.retrieval_atoms.as_slice())?;
        let metadata = serde_json::to_vec(&serde_json::json!({
            "repoId": response.repo_id,
            "path": response.path,
            "language": response.language,
            "nodeCount": response.nodes.len(),
            "edgeCount": response.edges.len(),
            "nodes": response.nodes,
            "edges": response.edges,
            "projections": response.projections,
            "focusNodeId": response.focus_node_id,
            "diagnostics": response.diagnostics,
        }))
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

#[derive(Clone)]
pub(crate) struct StudioSemanticScopeFlightRouteProvider {
    semantic_root: PathBuf,
}

impl StudioSemanticScopeFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self {
            semantic_root: state.studio.project_root.join("semantic"),
        }
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_semantic_root(semantic_root: PathBuf) -> Self {
        Self { semantic_root }
    }
}

impl std::fmt::Debug for StudioSemanticScopeFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioSemanticScopeFlightRouteProvider")
            .field("semantic_root", &self.semantic_root)
            .finish()
    }
}

#[async_trait]
impl SemanticScopeFlightRouteProvider for StudioSemanticScopeFlightRouteProvider {
    async fn semantic_scope_batch(
        &self,
        request: &SemanticScopeFlightRequest,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let repository = load_semantic_repository(&self.semantic_root);
        if !repository.report.is_success() {
            return Err(format!(
                "semantic repository validation failed: {}",
                semantic_report_summary(&repository.report)
            ));
        }

        let bundle = semantic_scope_bundle(
            &repository,
            &SemanticScopeRequest {
                task_id: request.task_id.clone(),
                object_ids: request.object_ids.clone(),
            },
        );
        let batch = semantic_scope_bundle_batch(&bundle)?;
        let metadata = semantic_scope_bundle_metadata(&bundle)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

fn semantic_scope_bundle_batch(bundle: &SemanticScopeBundle) -> Result<LanceRecordBatch, String> {
    let object_ids = bundle
        .objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<Vec<_>>();
    let kinds = bundle
        .objects
        .iter()
        .map(|object| semantic_kind_token(&object.kind))
        .collect::<Vec<_>>();
    let statuses = bundle
        .objects
        .iter()
        .map(|object| semantic_status_token(&object.status))
        .collect::<Vec<_>>();
    let titles = bundle
        .objects
        .iter()
        .map(|object| object.title.as_str())
        .collect::<Vec<_>>();
    let confidence_scores = bundle
        .objects
        .iter()
        .map(|object| object.confidence.score)
        .collect::<Vec<_>>();
    let confidence_sources = bundle
        .objects
        .iter()
        .map(|object| semantic_confidence_source_token(&object.confidence.source))
        .collect::<Vec<_>>();
    let source_paths = bundle
        .objects
        .iter()
        .map(|object| object.source_path.display().to_string())
        .collect::<Vec<_>>();
    let required_validations_json = bundle
        .objects
        .iter()
        .map(|object| serde_json::to_string(&object.verification.required))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to encode semantic validation list: {error}"))?;
    let relation_targets_json = bundle
        .objects
        .iter()
        .map(|object| {
            serde_json::to_string(
                &object
                    .relations
                    .iter()
                    .map(|relation| relation.target.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to encode semantic relation target list: {error}"))?;
    let projection_revisions = bundle
        .objects
        .iter()
        .map(|_| bundle.projection_revision.as_str())
        .collect::<Vec<_>>();
    let projection_source_revisions = bundle
        .objects
        .iter()
        .map(|_| {
            bundle
                .projection_source_revision
                .as_deref()
                .unwrap_or("semantic-ssot-unprojected")
        })
        .collect::<Vec<_>>();
    let projection_staleness = bundle
        .objects
        .iter()
        .map(|_| {
            bundle
                .projection_staleness
                .as_ref()
                .map_or("unprojected", semantic_projection_staleness_token)
        })
        .collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("objectId", LanceDataType::Utf8, false),
            LanceField::new("kind", LanceDataType::Utf8, false),
            LanceField::new("status", LanceDataType::Utf8, false),
            LanceField::new("title", LanceDataType::Utf8, false),
            LanceField::new("confidenceScore", LanceDataType::Float64, false),
            LanceField::new("confidenceSource", LanceDataType::Utf8, false),
            LanceField::new("sourcePath", LanceDataType::Utf8, false),
            LanceField::new("requiredValidationsJson", LanceDataType::Utf8, false),
            LanceField::new("relationTargetsJson", LanceDataType::Utf8, false),
            LanceField::new("projectionRevision", LanceDataType::Utf8, false),
            LanceField::new("projectionSourceRevision", LanceDataType::Utf8, false),
            LanceField::new("projectionStaleness", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(object_ids)),
            Arc::new(LanceStringArray::from(kinds)),
            Arc::new(LanceStringArray::from(statuses)),
            Arc::new(LanceStringArray::from(titles)),
            Arc::new(LanceFloat64Array::from(confidence_scores)),
            Arc::new(LanceStringArray::from(confidence_sources)),
            Arc::new(LanceStringArray::from(source_paths)),
            Arc::new(LanceStringArray::from(required_validations_json)),
            Arc::new(LanceStringArray::from(relation_targets_json)),
            Arc::new(LanceStringArray::from(projection_revisions)),
            Arc::new(LanceStringArray::from(projection_source_revisions)),
            Arc::new(LanceStringArray::from(projection_staleness)),
        ],
    )
    .map_err(|error| format!("failed to build semantic-scope Flight batch: {error}"))
}

fn semantic_scope_bundle_metadata(bundle: &SemanticScopeBundle) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "semanticScopeBundle": bundle,
    }))
    .map_err(|error| format!("failed to encode semantic-scope metadata: {error}"))
}

fn semantic_report_summary(report: &SemanticValidationReport) -> String {
    report
        .issues
        .iter()
        .map(|issue| match &issue.path {
            Some(path) => format!("{}: {}", path.display(), issue.message),
            None => issue.message.clone(),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn semantic_kind_token(kind: &SemanticObjectKind) -> &'static str {
    kind.id_prefix()
}

fn semantic_status_token(status: &SemanticStatus) -> &'static str {
    match status {
        SemanticStatus::Draft => "draft",
        SemanticStatus::Candidate => "candidate",
        SemanticStatus::Active => "active",
        SemanticStatus::Superseded => "superseded",
        SemanticStatus::Deprecated => "deprecated",
        SemanticStatus::Retired => "retired",
    }
}

fn semantic_confidence_source_token(source: &SemanticConfidenceSource) -> &'static str {
    match source {
        SemanticConfidenceSource::HumanSigned => "human_signed",
        SemanticConfidenceSource::Verified => "verified",
        SemanticConfidenceSource::LlmSuggested => "llm_suggested",
    }
}

fn semantic_projection_staleness_token(staleness: &SemanticProjectionStaleness) -> &'static str {
    match staleness {
        SemanticProjectionStaleness::Fresh => "fresh",
        SemanticProjectionStaleness::Stale => "stale",
    }
}

fn map_studio_api_error(error: &StudioApiError) -> String {
    error
        .error
        .details
        .clone()
        .unwrap_or_else(|| format!("{}: {}", error.code(), error.error.message))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::studio::arrow_types::{LanceArray, LanceStringArray};
    use xiuxian_wendao_server::transport::{
        SemanticScopeFlightRequest, SemanticScopeFlightRouteProvider,
    };

    use super::StudioSemanticScopeFlightRouteProvider;

    #[tokio::test]
    async fn semantic_scope_provider_serves_candidate_request_with_full_metadata() {
        let tempdir = tempfile::tempdir().expect("create temp semantic root");
        let semantic_root = tempdir.path().join("semantic");
        fs::create_dir_all(semantic_root.join("objects/component"))
            .expect("create component object directory");
        fs::create_dir_all(semantic_root.join("objects/task")).expect("create task directory");
        fs::create_dir_all(semantic_root.join("projections")).expect("create projections");
        fs::write(
            semantic_root.join("objects/component/demo.md"),
            r#"---
id: component.demo
kind: component
title: Demo Component
status: active
confidence:
  score: 0.92
  source: verified
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: runtime-provider
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-studio semantic_scope
relations: []
---

# Demo Component
"#,
        )
        .expect("write component object");
        fs::write(
            semantic_root.join("objects/task/pilot.md"),
            r#"---
id: task.semantic-scope-pilot
kind: task
title: Semantic Scope Pilot
status: candidate
confidence:
  score: 0.8
  source: human_signed
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: route-adapter
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-studio semantic_scope
relations:
  - kind: implements
    target: component.demo
---

# Semantic Scope Pilot
"#,
        )
        .expect("write task object");
        fs::write(
            semantic_root.join("projections/llm-compression.md"),
            r#"---
type: semantic_projection
projection: llm_compression
source_objects:
  - component.demo
  - task.semantic-scope-pilot
source_revision: stale-fixture
projection_revision: semantic-scope-test
staleness: stale
status: active
---

# LLM Compression
"#,
        )
        .expect("write projection");

        let provider = StudioSemanticScopeFlightRouteProvider::from_semantic_root(semantic_root);
        let response = provider
            .semantic_scope_batch(&SemanticScopeFlightRequest {
                task_id: Some("task.semantic-scope-pilot".to_string()),
                object_ids: Vec::new(),
            })
            .await
            .expect("semantic scope response");

        assert_eq!(response.batch.num_rows(), 2);
        let object_ids = response
            .batch
            .column_by_name("objectId")
            .expect("objectId column")
            .as_any()
            .downcast_ref::<LanceStringArray>()
            .expect("objectId string column");
        assert_eq!(object_ids.value(0), "component.demo");
        assert_eq!(object_ids.value(1), "task.semantic-scope-pilot");

        let metadata: serde_json::Value =
            serde_json::from_slice(&response.app_metadata).expect("semantic metadata json");
        assert_eq!(
            metadata["semanticScopeBundle"]["projection_revision"],
            "semantic-scope-test"
        );
        assert_eq!(
            metadata["semanticScopeBundle"]["projection_staleness"],
            "stale"
        );
        assert_eq!(
            metadata["semanticScopeBundle"]["objects"]
                .as_array()
                .expect("objects metadata")
                .len(),
            2
        );
    }
}
