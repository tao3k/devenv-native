use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use xiuxian_wendao_parsers::{
    SemanticChangeIntent, SemanticConfidenceSource, SemanticObjectKind,
    SemanticProjectionFreshnessPolicyReport, SemanticProjectionStaleness, SemanticRepository,
    SemanticScopeBundle, SemanticScopeRequest, SemanticStatus, SemanticValidationReport,
    load_semantic_repository, semantic_projection_freshness_policy_report, semantic_scope_bundle,
    semantic_scope_metadata_envelope, semantic_scope_metadata_envelope_to_vec,
};
use xiuxian_wendao_server::transport::{
    AnalysisFlightRouteResponse, CodeAstAnalysisFlightRouteProvider,
    MarkdownAnalysisFlightRouteProvider, SemanticScopeFlightRequest,
    SemanticScopeFlightRouteProvider,
};
use xiuxian_wendao_sql::DataFusionLocalRelationEngine;
use xiuxian_wendao_sql::semantic_read_model::{
    SemanticProjectionFreshnessFinding, SemanticSqlGuardEvidence,
    run_semantic_sql_projection_freshness_guard_with_engine,
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
    pub(crate) fn new(state: &GatewayState) -> Self {
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
        let sql_guard_evidence = semantic_sql_guard_evidence(&repository).await?;
        let projection_policy_evidence = semantic_projection_freshness_policy_report(&repository);
        let batch = semantic_scope_bundle_batch(&bundle)?;
        let metadata = semantic_scope_bundle_metadata(
            &bundle,
            &sql_guard_evidence,
            &projection_policy_evidence,
        )?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

struct SemanticScopeBatchColumns<'a> {
    object_ids: Vec<&'a str>,
    kinds: Vec<&'static str>,
    statuses: Vec<&'static str>,
    titles: Vec<&'a str>,
    confidence_scores: Vec<f64>,
    confidence_sources: Vec<&'static str>,
    source_paths: Vec<String>,
    required_validations_json: Vec<String>,
    relation_targets_json: Vec<String>,
    change_intent_ids_json: Vec<String>,
    projection_revisions: Vec<&'a str>,
    projection_source_revisions: Vec<&'a str>,
    projection_staleness: Vec<&'static str>,
}

fn semantic_scope_bundle_batch(bundle: &SemanticScopeBundle) -> Result<LanceRecordBatch, String> {
    let columns = semantic_scope_batch_columns(bundle)?;
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
            LanceField::new("changeIntentIdsJson", LanceDataType::Utf8, false),
            LanceField::new("projectionRevision", LanceDataType::Utf8, false),
            LanceField::new("projectionSourceRevision", LanceDataType::Utf8, false),
            LanceField::new("projectionStaleness", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(columns.object_ids)),
            Arc::new(LanceStringArray::from(columns.kinds)),
            Arc::new(LanceStringArray::from(columns.statuses)),
            Arc::new(LanceStringArray::from(columns.titles)),
            Arc::new(LanceFloat64Array::from(columns.confidence_scores)),
            Arc::new(LanceStringArray::from(columns.confidence_sources)),
            Arc::new(LanceStringArray::from(columns.source_paths)),
            Arc::new(LanceStringArray::from(columns.required_validations_json)),
            Arc::new(LanceStringArray::from(columns.relation_targets_json)),
            Arc::new(LanceStringArray::from(columns.change_intent_ids_json)),
            Arc::new(LanceStringArray::from(columns.projection_revisions)),
            Arc::new(LanceStringArray::from(columns.projection_source_revisions)),
            Arc::new(LanceStringArray::from(columns.projection_staleness)),
        ],
    )
    .map_err(|error| format!("failed to build semantic-scope Flight batch: {error}"))
}

fn semantic_scope_batch_columns(
    bundle: &SemanticScopeBundle,
) -> Result<SemanticScopeBatchColumns<'_>, String> {
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
    let change_intent_ids_json = bundle
        .objects
        .iter()
        .map(|object| {
            serde_json::to_string(&change_intent_ids_for_object(
                object.id.as_str(),
                bundle.change_intents.as_slice(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to encode semantic change intent id list: {error}"))?;

    Ok(SemanticScopeBatchColumns {
        object_ids: bundle
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect(),
        kinds: bundle
            .objects
            .iter()
            .map(|object| semantic_kind_token(&object.kind))
            .collect(),
        statuses: bundle
            .objects
            .iter()
            .map(|object| semantic_status_token(&object.status))
            .collect(),
        titles: bundle
            .objects
            .iter()
            .map(|object| object.title.as_str())
            .collect(),
        confidence_scores: bundle
            .objects
            .iter()
            .map(|object| object.confidence.score)
            .collect(),
        confidence_sources: bundle
            .objects
            .iter()
            .map(|object| semantic_confidence_source_token(&object.confidence.source))
            .collect(),
        source_paths: bundle
            .objects
            .iter()
            .map(|object| object.source_path.display().to_string())
            .collect(),
        required_validations_json,
        relation_targets_json,
        change_intent_ids_json,
        projection_revisions: bundle
            .objects
            .iter()
            .map(|_| bundle.projection_revision.as_str())
            .collect(),
        projection_source_revisions: bundle
            .objects
            .iter()
            .map(|_| {
                bundle
                    .projection_source_revision
                    .as_deref()
                    .unwrap_or("semantic-ssot-unprojected")
            })
            .collect(),
        projection_staleness: bundle
            .objects
            .iter()
            .map(|_| {
                bundle
                    .projection_staleness
                    .as_ref()
                    .map_or("unprojected", semantic_projection_staleness_token)
            })
            .collect(),
    })
}

async fn semantic_sql_guard_evidence(
    repository: &SemanticRepository,
) -> Result<SemanticSqlGuardEvidence, String> {
    let query_engine = DataFusionLocalRelationEngine::new_with_information_schema();
    run_semantic_sql_projection_freshness_guard_with_engine(repository, &query_engine).await
}

fn semantic_scope_bundle_metadata(
    bundle: &SemanticScopeBundle,
    sql_guard_evidence: &SemanticSqlGuardEvidence,
    projection_policy_evidence: &SemanticProjectionFreshnessPolicyReport,
) -> Result<Vec<u8>, String> {
    let envelope = semantic_scope_metadata_envelope(
        bundle.clone(),
        Some(semantic_sql_guard_evidence_json(sql_guard_evidence)),
        Some(projection_policy_evidence.clone()),
    );
    semantic_scope_metadata_envelope_to_vec(&envelope)
        .map_err(|error| format!("failed to encode semantic-scope metadata: {error}"))
}

fn semantic_sql_guard_evidence_json(evidence: &SemanticSqlGuardEvidence) -> serde_json::Value {
    serde_json::json!({
        "guardId": evidence.guard_id.as_str(),
        "semanticObjectId": evidence.semantic_object_id.as_str(),
        "status": evidence.status.as_str(),
        "queryText": evidence.query_text.as_str(),
        "failingRowCount": evidence.failing_row_count,
        "findings": evidence.findings.iter().map(semantic_sql_guard_finding_json).collect::<Vec<_>>(),
        "message": evidence.message.as_str(),
        "localRelationEngine": evidence.local_relation_engine.as_deref(),
    })
}

fn semantic_sql_guard_finding_json(
    finding: &SemanticProjectionFreshnessFinding,
) -> serde_json::Value {
    serde_json::json!({
        "projection": finding.projection.as_str(),
        "sourceRevision": finding.source_revision.as_str(),
        "currentSourceRevision": finding.current_source_revision.as_str(),
        "projectionRevision": finding.projection_revision.as_str(),
        "staleness": finding.staleness.as_str(),
        "sourcePath": finding.source_path.as_str(),
    })
}

fn change_intent_ids_for_object<'a>(
    object_id: &str,
    change_intents: &'a [SemanticChangeIntent],
) -> Vec<&'a str> {
    change_intents
        .iter()
        .filter(|intent| {
            intent
                .touched_objects
                .iter()
                .any(|touched| touched == object_id)
                || intent
                    .affected_invariants
                    .iter()
                    .any(|invariant| invariant == object_id)
                || intent
                    .changed_relations
                    .iter()
                    .any(|relation| relation.source == object_id || relation.target == object_id)
        })
        .map(|intent| intent.id.as_str())
        .collect()
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
        fs::create_dir_all(semantic_root.join("objects/invariant"))
            .expect("create invariant object directory");
        fs::create_dir_all(semantic_root.join("objects/task")).expect("create task directory");
        fs::create_dir_all(semantic_root.join("change-intents"))
            .expect("create change intent directory");
        fs::create_dir_all(semantic_root.join("projections")).expect("create projections");
        fs::write(
            semantic_root.join("objects/component/demo.md"),
            r"---
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
    - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations: []
---

# Demo Component
",
        )
        .expect("write component object");
        fs::write(
            semantic_root.join("objects/invariant/authority.md"),
            r"---
id: invariant.demo-authority
kind: invariant
title: Demo Authority Invariant
status: active
confidence:
  score: 0.95
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
    - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations: []
---

# Demo Authority Invariant
",
        )
        .expect("write invariant object");
        fs::write(
            semantic_root.join("objects/task/pilot.md"),
            r"---
id: task.semantic-scope-pilot
kind: task
title: Semantic Scope Pilot
status: candidate
confidence:
  score: 0.8
  source: llm_suggested
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
    role: route-adapter
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations:
  - kind: implements
    target: component.demo
---

# Semantic Scope Pilot
",
        )
        .expect("write task object");
        fs::write(
            semantic_root.join("projections/llm-compression.md"),
            r"---
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
",
        )
        .expect("write projection");
        fs::write(
            semantic_root.join("change-intents/semantic-scope-pilot.md"),
            r"---
type: semantic_change_intent
id: change.semantic-scope-pilot
title: Semantic Scope Pilot
status: active
touched_objects:
  - task.semantic-scope-pilot
changed_relations:
  - source: task.semantic-scope-pilot
    kind: implements
    target: component.demo
    action: add
affected_invariants:
  - invariant.demo-authority
required_validations:
  - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
projections_to_refresh:
  - llm_compression
candidate_suggestions:
  - task.semantic-scope-pilot
---

# Semantic Scope Pilot Change
",
        )
        .expect("write change intent");

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
        let change_intents = response
            .batch
            .column_by_name("changeIntentIdsJson")
            .expect("changeIntentIdsJson column")
            .as_any()
            .downcast_ref::<LanceStringArray>()
            .expect("changeIntentIdsJson string column");
        assert_eq!(
            change_intents.value(0),
            r#"["change.semantic-scope-pilot"]"#
        );
        assert_eq!(
            change_intents.value(1),
            r#"["change.semantic-scope-pilot"]"#
        );

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
        assert_eq!(
            metadata["semanticScopeBundle"]["change_intents"]
                .as_array()
                .expect("change intent metadata")
                .len(),
            1
        );
        assert_eq!(
            metadata["semanticSqlGuardEvidence"]["guardId"],
            "semantic_sql.projection_freshness"
        );
        assert_eq!(
            metadata["semanticSqlGuardEvidence"]["status"],
            "review_required"
        );
        assert_eq!(metadata["semanticSqlGuardEvidence"]["failingRowCount"], 1);
        assert_eq!(
            metadata["semanticSqlGuardEvidence"]["findings"]
                .as_array()
                .expect("semantic SQL guard findings metadata")
                .len(),
            1
        );
        assert_eq!(
            metadata["semanticProjectionPolicyEvidence"]["policyId"],
            "semantic_projection.required_refresh_targets"
        );
        assert_eq!(
            metadata["semanticProjectionPolicyEvidence"]["status"],
            "review_required"
        );
        assert_eq!(
            metadata["semanticProjectionPolicyEvidence"]["failingProjectionCount"],
            1
        );
        assert_eq!(
            metadata["semanticProjectionPolicyEvidence"]["projections"]
                .as_array()
                .expect("semantic projection policy finding metadata")
                .len(),
            1
        );
    }
}
