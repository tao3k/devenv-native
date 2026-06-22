//! Runtime Flight service bridge for `WendaoGraph` `SearchStrategyFlow`.

use xiuxian_polyglot_orchestrator::{
    BenchmarkState, ContractValidationState, JuliaComputeTaskShape, JuliaReadinessEvidence,
    JuliaRuntimeStats, JuliaScheduleAction, JuliaSchedulePlan, JuliaSchedulingInput,
    JuliaTaskComplexityClass, ManifestReadinessState, RouteProfileRef, WarmupState,
};
use xiuxian_wendao_core::PluginProviderSelector;
use xiuxian_wendao_core::capabilities::{ContractVersion, PluginCapabilityBinding};
use xiuxian_wendao_core::ids::{CapabilityId, PluginId};
use xiuxian_wendao_core::transport::{PluginTransportEndpoint, PluginTransportKind};
use xiuxian_wendao_runtime::transport::negotiate_flight_transport_client_from_bindings;

use super::decode::decode_search_strategy_flow_service_response;
use super::types::SearchStrategyFlowServiceRoundtrip;
use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInputBatch;
use crate::integration_support::search_strategy_flow_flight::constants::{
    DEFAULT_TIMEOUT_SECONDS, WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_CAPABILITY_ID,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROFILE_ID, WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROVIDER_ID,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ROUTE, WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION,
};
use crate::integration_support::search_strategy_flow_flight::request::{
    SearchStrategyFlowServiceRequestOptions, build_search_strategy_flow_service_arrow_request,
    build_search_strategy_flow_service_flight_request_batch,
};

/// Runtime binding options for the `WendaoGraph` `SearchStrategyFlow` Flight service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStrategyFlowServiceFlightBindingOptions {
    /// Flight service base URL, for example `http://127.0.0.1:8815`.
    pub base_url: String,
    /// Optional health-check route for the service process.
    pub health_route: Option<String>,
    /// Per-request timeout in seconds.
    pub timeout_seconds: u64,
    /// Optional caller-supplied in-flight cap before orchestrator admission.
    pub max_in_flight_requests: Option<u32>,
}

impl SearchStrategyFlowServiceFlightBindingOptions {
    /// Build service binding options with the default timeout.
    ///
    /// # Errors
    ///
    /// Returns an error when the Flight base URL is blank.
    pub fn new(base_url: impl Into<String>) -> Result<Self, String> {
        let base_url = base_url.into();
        if base_url.trim().is_empty() {
            return Err("SearchStrategyFlow service Flight base URL must not be blank".to_string());
        }
        Ok(Self {
            base_url,
            health_route: None,
            timeout_seconds: DEFAULT_TIMEOUT_SECONDS,
            max_in_flight_requests: None,
        })
    }

    /// Return options with a service health route.
    #[must_use]
    pub fn with_health_route(mut self, health_route: impl Into<String>) -> Self {
        self.health_route = Some(health_route.into());
        self
    }

    /// Return options with a per-request timeout.
    #[must_use]
    pub const fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = if timeout_seconds == 0 {
            1
        } else {
            timeout_seconds
        };
        self
    }

    /// Return options with an explicit in-flight cap.
    #[must_use]
    pub const fn with_max_in_flight_requests(mut self, max_in_flight_requests: u32) -> Self {
        self.max_in_flight_requests = Some(if max_in_flight_requests == 0 {
            1
        } else {
            max_in_flight_requests
        });
        self
    }
}

/// Build the canonical provider selector for `SearchStrategyFlow`.
#[must_use]
pub fn wendaograph_search_strategy_flow_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_CAPABILITY_ID.to_owned()),
        provider: PluginId(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROVIDER_ID.to_owned()),
    }
}

/// Build the polyglot route/profile reference for the `SearchStrategyFlow` service.
#[must_use]
pub fn wendaograph_search_strategy_flow_route_profile_ref() -> RouteProfileRef {
    RouteProfileRef::julia_profile(
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ROUTE,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROFILE_ID,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION,
    )
}

/// Build the orchestrator schedule plan required before enabling the service binding.
#[must_use]
pub fn build_search_strategy_flow_service_orchestrator_schedule_plan(
    options: &SearchStrategyFlowServiceFlightBindingOptions,
    candidate_count: usize,
    input_byte_size: usize,
) -> JuliaSchedulePlan {
    let readiness =
        JuliaReadinessEvidence::graph_search_profile(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROFILE_ID)
            .with_schema_version(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION)
            .with_route_validation(ContractValidationState::Valid)
            .with_schema_validation(ContractValidationState::Valid)
            .with_manifest_readiness(ManifestReadinessState::Ready)
            .with_warmup(WarmupState::Ready)
            .with_benchmark(BenchmarkState::NotRequired)
            .with_admission_window(options.max_in_flight_requests, 0, 0);
    let task_shape = JuliaComputeTaskShape::new()
        .with_rows(saturating_usize_to_u32(candidate_count.max(1)))
        .with_graph_size(
            saturating_usize_to_u32(candidate_count.max(1)),
            saturating_usize_to_u32(candidate_count.saturating_mul(4).max(1)),
        )
        .with_feature_columns(12)
        .with_byte_size(input_byte_size as u64)
        .with_batchability_key(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROFILE_ID)
        .with_complexity(JuliaTaskComplexityClass::Balanced);
    let runtime_stats = JuliaRuntimeStats::new()
        .with_warmup(WarmupState::Ready)
        .with_benchmark(BenchmarkState::NotRequired);
    JuliaSchedulingInput::new(readiness, task_shape, runtime_stats)
        .with_target_latency_ms(Some(saturating_timeout_millis(options.timeout_seconds)))
        .plan()
}

/// Build one negotiated runtime binding for the `SearchStrategyFlow` Flight service.
///
/// # Errors
///
/// Returns an error when the Flight base URL is blank or the orchestrator does
/// not admit the Julia `SearchStrategyFlow` service exchange.
pub fn build_search_strategy_flow_service_flight_binding(
    options: SearchStrategyFlowServiceFlightBindingOptions,
    candidate_batch: &SearchStrategyFlowCandidateInputBatch,
) -> Result<PluginCapabilityBinding, String> {
    build_search_strategy_flow_service_flight_binding_with_input_byte_size(
        options,
        candidate_batch,
        candidate_batch.candidate_input_arrow_ipc_byte_len(),
    )
}

fn build_search_strategy_flow_service_flight_binding_with_input_byte_size(
    options: SearchStrategyFlowServiceFlightBindingOptions,
    candidate_batch: &SearchStrategyFlowCandidateInputBatch,
    input_byte_size: usize,
) -> Result<PluginCapabilityBinding, String> {
    if candidate_batch.row_count == 0 {
        return Err("SearchStrategyFlow service request must include candidate rows".to_string());
    }
    let base_url = options.base_url.trim();
    if base_url.is_empty() {
        return Err("SearchStrategyFlow service Flight base URL must not be blank".to_string());
    }
    let schedule_plan = build_search_strategy_flow_service_orchestrator_schedule_plan(
        &options,
        candidate_batch.row_count,
        input_byte_size,
    );
    if !matches!(
        schedule_plan.action,
        JuliaScheduleAction::Dispatch | JuliaScheduleAction::Queue
    ) {
        return Err(format!(
            "SearchStrategyFlow service exchange was rejected by xiuxian-polyglot-orchestrator: action={:?}, reason={:?}",
            schedule_plan.action, schedule_plan.reason
        ));
    }

    Ok(PluginCapabilityBinding {
        selector: wendaograph_search_strategy_flow_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(base_url.to_owned()),
            route: Some(WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ROUTE.to_owned()),
            health_route: options.health_route,
            timeout_secs: Some(options.timeout_seconds),
            max_in_flight_requests: Some(u64::from(
                schedule_plan.max_in_flight_recommendation.max(1),
            )),
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION.to_owned(),
        ),
    })
}

/// Execute one `SearchStrategyFlow` service roundtrip and decode frontier rows.
///
/// # Errors
///
/// Returns an error when the candidate batch has no Arrow IPC payload,
/// transport negotiation fails, the Flight exchange fails, or the response
/// does not contain valid `strategy_frontier` rows.
pub async fn roundtrip_search_strategy_flow_frontier_with_service(
    candidate_batch: &SearchStrategyFlowCandidateInputBatch,
    options: SearchStrategyFlowServiceFlightBindingOptions,
) -> Result<SearchStrategyFlowServiceRoundtrip, String> {
    roundtrip_search_strategy_flow_frontier_with_service_request(
        candidate_batch,
        SearchStrategyFlowServiceRequestOptions::default(),
        options,
    )
    .await
}

/// Execute one `SearchStrategyFlow` service roundtrip with optional request side tables.
///
/// # Errors
///
/// Returns an error when request bundling fails, transport negotiation fails,
/// the Flight exchange fails, or the response does not contain valid
/// `strategy_frontier` rows.
pub async fn roundtrip_search_strategy_flow_frontier_with_service_request(
    candidate_batch: &SearchStrategyFlowCandidateInputBatch,
    request_options: SearchStrategyFlowServiceRequestOptions,
    options: SearchStrategyFlowServiceFlightBindingOptions,
) -> Result<SearchStrategyFlowServiceRoundtrip, String> {
    let request =
        build_search_strategy_flow_service_arrow_request(candidate_batch, request_options)?;
    let binding = build_search_strategy_flow_service_flight_binding_with_input_byte_size(
        options,
        candidate_batch,
        request.payload_byte_size(),
    )?;
    let transport =
        negotiate_flight_transport_client_from_bindings(std::slice::from_ref(&binding))?
            .ok_or_else(|| {
                "SearchStrategyFlow service Flight binding did not materialize a client".to_string()
            })?;
    let request_batch = build_search_strategy_flow_service_flight_request_batch(&request)?;
    let response_batches = transport.process_batch(&request_batch).await?;
    let response = decode_search_strategy_flow_service_response(&response_batches)?;
    let rows = response.frontier.clone();
    Ok(SearchStrategyFlowServiceRoundtrip {
        flight_route: transport.flight_route().to_owned(),
        response,
        rows,
    })
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn saturating_timeout_millis(timeout_seconds: u64) -> u32 {
    let timeout_millis = timeout_seconds.saturating_mul(1_000);
    u32::try_from(timeout_millis).unwrap_or(u32::MAX)
}
