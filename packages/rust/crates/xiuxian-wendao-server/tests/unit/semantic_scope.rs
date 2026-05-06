use std::sync::{Arc, Mutex};

use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_flight::FlightDescriptor;
use arrow_flight::flight_service_server::FlightService;
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use tonic::Request;
use xiuxian_wendao_server::transport::{
    ANALYSIS_SEMANTIC_SCOPE_ROUTE, AnalysisFlightRouteResponse, RepoSearchFlightRequest,
    RepoSearchFlightRouteProvider, RerankScoreWeights, SemanticScopeFlightRequest,
    SemanticScopeFlightRouteProvider, WENDAO_SCHEMA_VERSION_HEADER,
    WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER, WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER,
    WendaoFlightRouteProviders, WendaoFlightService, flight_descriptor_path,
    validate_semantic_scope_request,
};

const TEST_SCHEMA_VERSION: &str = "semantic-scope-smoke";

#[test]
fn semantic_scope_route_contract_exposes_stable_headers() {
    assert_eq!(ANALYSIS_SEMANTIC_SCOPE_ROUTE, "/analysis/semantic-scope");
    assert_eq!(
        WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER,
        "x-wendao-semantic-task-id"
    );
    assert_eq!(
        WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
        "x-wendao-semantic-object-ids"
    );
}

#[test]
fn semantic_scope_request_accepts_task_and_object_ids() {
    let request = validate_semantic_scope_request(
        Some(" task.semantic-ssot.object-schema-pilot "),
        &[
            "component.wendao.query-substrate".to_string(),
            " invariant.llm-output-is-not-authority ".to_string(),
        ],
    )
    .unwrap_or_else(|error| panic!("semantic scope request should validate: {error}"));

    assert_eq!(
        request.task_id.as_deref(),
        Some("task.semantic-ssot.object-schema-pilot")
    );
    assert_eq!(
        request.object_ids,
        vec![
            "component.wendao.query-substrate",
            "invariant.llm-output-is-not-authority"
        ]
    );
}

#[test]
fn semantic_scope_request_allows_default_active_scope() {
    let request = validate_semantic_scope_request(None, &[])
        .unwrap_or_else(|error| panic!("default semantic scope should validate: {error}"));

    assert!(request.task_id.is_none());
    assert!(request.object_ids.is_empty());
}

#[test]
fn semantic_scope_request_rejects_blank_object_ids() {
    let result = validate_semantic_scope_request(None, &[String::new()]);

    assert!(result.is_err());
}

#[tokio::test]
async fn semantic_scope_flight_info_routes_to_provider_and_preserves_metadata() {
    let observed_request = Arc::new(Mutex::new(None));
    let mut providers = WendaoFlightRouteProviders::new(Arc::new(FakeRepoSearchProvider {
        batch: empty_batch(),
    }));
    providers.semantic_scope = Some(Arc::new(FakeSemanticScopeProvider {
        observed_request: Arc::clone(&observed_request),
    }));
    let service = WendaoFlightService::new_with_route_providers(
        TEST_SCHEMA_VERSION,
        providers,
        1,
        RerankScoreWeights::default(),
    )
    .unwrap_or_else(|error| panic!("build semantic-scope Flight service: {error}"));

    let descriptor = FlightDescriptor::new_path(
        flight_descriptor_path(ANALYSIS_SEMANTIC_SCOPE_ROUTE)
            .unwrap_or_else(|error| panic!("semantic-scope descriptor path: {error}")),
    );
    let mut request = Request::new(descriptor);
    request.metadata_mut().insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        TEST_SCHEMA_VERSION
            .parse()
            .unwrap_or_else(|error| panic!("schema metadata value: {error}")),
    );
    request.metadata_mut().insert(
        WENDAO_SEMANTIC_SCOPE_TASK_ID_HEADER,
        " task.semantic-ssot.object-schema-pilot "
            .parse()
            .unwrap_or_else(|error| panic!("task metadata value: {error}")),
    );
    request.metadata_mut().insert(
        WENDAO_SEMANTIC_SCOPE_OBJECT_IDS_HEADER,
        "component.wendao.query-substrate, invariant.llm-output-is-not-authority"
            .parse()
            .unwrap_or_else(|error| panic!("object ids metadata value: {error}")),
    );

    let flight_info = service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| panic!("semantic-scope get_flight_info: {error}"))
        .into_inner();

    assert_eq!(flight_info.total_records, 1);
    assert_eq!(
        flight_info.app_metadata.as_ref(),
        br#"{"semanticScopeBundle":{"status":"ready"}}"#
    );
    let observed = observed_request
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_else(|| panic!("semantic-scope provider should receive request"));
    assert_eq!(
        observed.task_id.as_deref(),
        Some("task.semantic-ssot.object-schema-pilot")
    );
    assert_eq!(
        observed.object_ids,
        vec![
            "component.wendao.query-substrate",
            "invariant.llm-output-is-not-authority"
        ]
    );
}

#[derive(Debug)]
struct FakeRepoSearchProvider {
    batch: RecordBatch,
}

#[async_trait]
impl RepoSearchFlightRouteProvider for FakeRepoSearchProvider {
    async fn repo_search_batch(
        &self,
        _request: &RepoSearchFlightRequest,
    ) -> Result<RecordBatch, String> {
        Ok(self.batch.clone())
    }
}

#[derive(Debug)]
struct FakeSemanticScopeProvider {
    observed_request: Arc<Mutex<Option<SemanticScopeFlightRequest>>>,
}

#[async_trait]
impl SemanticScopeFlightRouteProvider for FakeSemanticScopeProvider {
    async fn semantic_scope_batch(
        &self,
        request: &SemanticScopeFlightRequest,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *self
            .observed_request
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(request.clone());
        Ok(AnalysisFlightRouteResponse::new(semantic_scope_batch())
            .with_app_metadata(br#"{"semanticScopeBundle":{"status":"ready"}}"#.as_slice()))
    }
}

fn empty_batch() -> RecordBatch {
    RecordBatch::new_empty(Arc::new(Schema::empty()))
}

fn semantic_scope_batch() -> RecordBatch {
    let object_ids: ArrayRef = Arc::new(StringArray::from(vec![
        "task.semantic-ssot.object-schema-pilot",
    ]));
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "objectId",
            DataType::Utf8,
            false,
        )])),
        vec![object_ids],
    )
    .unwrap_or_else(|error| panic!("semantic-scope test batch: {error}"))
}
