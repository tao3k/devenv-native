//! xiuxian-qianji: The Thousand Mechanisms Engine.
//!
//! A high-performance, probabilistic DAG executor based on petgraph.
//! Follows Rust 2024 Edition standards.

/// Application-layer scheduler factories and built-in pipeline presets.
#[cfg(feature = "wendao-integration")]
pub mod app;
/// High-level laboratory API for end-to-end workflow execution.
#[cfg(feature = "wendao-integration")]
pub mod bootcamp;
/// Thin BPMN host adapter helpers backed by `xiuxian-qianji-bpmn-engine`.
pub mod bpmn;
/// Distributed consensus management for multi-agent synchronization.
#[cfg(feature = "qianji-full")]
pub mod consensus;
/// LLM-facing progressive-disclosure cards for BPMN/DMN constructs.
pub mod construct_cards;
/// Static WorkflowPlan validation for construct-card consumers.
pub mod construct_plan;
/// Contract-feedback execution bridge for contract suite runs and Wendao export.
#[cfg(feature = "qianji-full")]
pub mod contract_feedback;
/// Contract definitions for nodes, instructions, and manifests.
#[cfg(feature = "qianji-full")]
pub mod contracts;
/// Core graph engine based on petgraph.
#[cfg(feature = "qianji-full")]
pub mod engine;
/// Unified error handling.
pub mod error;
/// Built-in node execution mechanisms.
#[cfg(feature = "qianji-full")]
pub mod executors;
/// Flowhub module, scenario, and materialize helpers.
#[cfg(feature = "qianji-full")]
pub mod flowhub;
/// Graphical layout and aesthetic engine (QGS).
#[cfg(feature = "qianji-full")]
pub mod layout;
mod llm_client;
/// Manifest inspection helpers.
#[cfg(feature = "qianji-full")]
pub mod manifest;
/// Shared markdown renderers for `qianji` show/check surfaces.
#[cfg(feature = "qianji-full")]
pub(crate) mod markdown;
#[cfg(feature = "qianji-full")]
mod qianji_cli;
mod qianji_server;
mod qianji_server_cli;
#[cfg(all(
    feature = "llm",
    any(
        all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
        test
    )
))]
mod qianji_worker;
/// Runtime configuration resolver (`resources/config/qianji.toml` + user overrides).
pub mod runtime_config;
/// Formal logic and safety auditing.
#[cfg(feature = "qianji-full")]
pub mod safety;
/// Asynchronous synaptic-flow scheduler.
#[cfg(feature = "qianji-full")]
pub mod scheduler;
#[cfg(feature = "qianji-full")]
mod scheduler_checkpoint;
mod scheduler_identity;
#[cfg(feature = "qianji-full")]
mod scheduler_policy;
#[cfg(feature = "qianji-full")]
mod scheduler_preflight;
#[cfg(feature = "qianji-full")]
mod scheduler_state;
/// Sovereign Memory Module (Blueprint V6.1) - Agent reasoning trace persistence.
#[cfg(feature = "wendao-integration")]
pub mod sovereign;
/// Multi-agent swarm orchestration runtime.
#[cfg(feature = "qianji-full")]
pub mod swarm;
/// Real-time swarm telemetry contracts and Valkey emitter.
pub mod telemetry;
/// Bounded work-surface parsing, validation, and CLI support helpers.
#[cfg(feature = "qianji-full")]
pub mod workdir;
/// Workflow/task-level route configuration (`resources/config/workflows` + user overlays).
pub mod workflow_config;
/// Low-overhead typed workflow execution substrate.
pub mod workflow_kernel;

mod api;
#[cfg(feature = "valkey")]
pub use api::ValkeyPulseEmitter;
#[cfg(feature = "qianji-full")]
pub use api::{
    AnchoredMaterializedWorkdir, ClusterNodeIdentity, ClusterNodeRecord, FlowInstruction,
    FlowhubCheckReport, FlowhubDiagnostic, FlowhubDirKind, FlowhubGraphShow, FlowhubGraphTopology,
    FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow,
    FlowhubScenarioCaseSummary, FlowhubScenarioCheckReport, FlowhubScenarioDiagnostic,
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview, FlowhubShow,
    GlobalSwarmRegistry, MaterializedWorkdir, NodeQianhuanExecutionMode, NodeStatus,
    QianjiCliError, QianjiCompiler, QianjiEngine, QianjiManifest, QianjiMechanism, QianjiOutput,
    QianjiSafetyGuard, QianjiScheduler, QianjiStateSnapshot, RemoteNodeRequest, RemoteNodeResponse,
    RemotePossessionBus, RemotePossessionBusError, ResolvedFlowhubModule, RoleAvailabilityRegistry,
    SchedulerExcludedClusterRef, SchedulerExecutionPolicy, SwarmAgentConfig, SwarmAgentReport,
    SwarmEngine, SwarmExecutionOptions, SwarmExecutionReport, WorkdirAdvance, WorkdirCheckReport,
    WorkdirDiagnostic, WorkdirMarkdownSurface, WorkdirShow, WorkdirVisibleSurface,
    WorkdirVisibleSurfaceKind, advance_workdir_step, check_flowhub, check_flowhub_scenario,
    check_workdir, classify_flowhub_dir, load_flowhub_module_manifest,
    load_flowhub_scenario_manifest, load_workdir_manifest, looks_like_flowhub_scenario_dir,
    looks_like_workdir_dir, manifest_declares_qianhuan_bindings, manifest_requires_llm,
    map_execution_error_to_response, materialize_flowhub_anchored_scenario,
    materialize_flowhub_anchored_scenario_at_node, materialize_flowhub_scenario_workdir,
    parse_flowhub_module_manifest, parse_flowhub_scenario_manifest, parse_workdir_manifest,
    render_anchored_materialized_workdir, render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_scenario_check_markdown, render_flowhub_scenario_show, render_flowhub_show,
    render_workdir_advance, render_workdir_check_markdown, render_workdir_show,
    resolve_flowhub_module_children, resolve_flowhub_scenario_modules, run_qianji_cli,
    show_flowhub, show_flowhub_anchored_scenario, show_flowhub_graph, show_flowhub_scenario,
    show_workdir,
};
pub use api::{
    BPMN_HOST_WORK_ACTIVITY_METADATA_KEY, BPMN_HOST_WORK_ACTIVITY_SCHEMA,
    BPMN_HOST_WORK_ACTIVITY_TYPE, BPMN_HOST_WORK_COMPLETION_METADATA_KEY,
    BPMN_HOST_WORK_COMPLETION_SCHEMA, BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA, BpmnAdapterError,
    BpmnHostWorkActivityScheduleInput, BpmnHostWorkLlmActivityRouteInput,
    BpmnHostWorkLlmEndpointDecision, BpmnHostWorkLlmRouteDecision, BpmnOrchestrationError,
    BpmnUnsupportedStartNodeKind, ConsensusStatus, ConstructCard, ConstructIndexEntry,
    ConstructLintMapping, ConstructStatus, DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE,
    DEFAULT_PULSE_CHANNEL, DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FlowhubScenarioIdRef, FlowhubServiceActivityHttpScheduleInput,
    FlowhubServiceActivityScheduleInput, NodeTransitionPhase, NoopPulseEmitter, PulseEmitter,
    QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE, QIANJI_RUN_CONSOLE_EVENT_ROUTE,
    QIANJI_RUN_CONSOLE_SCHEMA_VERSION, QianjiBpmnActivityId, QianjiBpmnCheckpointStore,
    QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade, QianjiBpmnExecutionMode,
    QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest, QianjiBpmnExecutionScheduler,
    QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder, QianjiBpmnLeaseOwnerToken,
    QianjiBpmnPackageId, QianjiBpmnPendingHostCompletion, QianjiBpmnPendingHostWorkHttpResponse,
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart, QianjiBpmnProcessId,
    QianjiBpmnSchedulerLeaseConfig, QianjiBpmnSession, QianjiBpmnStartAtNodeId,
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowEventPollReport,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowHttpErrorBody, QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowInstanceId,
    QianjiBpmnWorkflowInstanceSummary, QianjiBpmnWorkflowInstancesReport,
    QianjiBpmnWorkflowInstancesRequest, QianjiBpmnWorkflowInterruptReport,
    QianjiBpmnWorkflowInterruptRequest, QianjiBpmnWorkflowResumeReport,
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowSnapshotHttpResponse, QianjiBpmnWorkflowStartHttpRequest,
    QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowStatusHttpQuery, QianjiBpmnWorkflowStatusHttpResponse,
    QianjiBpmnWorkflowStatusReport, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskClaimHttpPayload, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskClaimHttpResponse, QianjiBpmnWorkflowTaskClaimPayload,
    QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteBatchReport,
    QianjiBpmnWorkflowTaskCompleteBatchRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionHttpPayload,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiBpmnWorkflowTaskReleaseHttpPayload, QianjiBpmnWorkflowTaskReleaseHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseReport, QianjiBpmnWorkflowTaskReleaseRequest,
    QianjiBpmnWorkflowWorklistItem, QianjiBpmnWorkflowWorklistReport,
    QianjiBpmnWorkflowWorklistRequest, QianjiBpmnWorkflowWorklistRoutingFilter,
    QianjiControlDiagnosticsHttpResponse, QianjiControlHistoryHttpResponse,
    QianjiControlRecoveryApplyHttpRequest, QianjiControlRecoveryApplyHttpResponse,
    QianjiControlRecoveryHttpResponse, QianjiControlRunStreamSource,
    QianjiControlRunSummaryHttpResponse, QianjiControlWorkflowSourceAdmissionHttpRequest,
    QianjiControlWorkflowSourceAdmissionHttpResponse,
    QianjiControlWorkflowSourceAdmittedHttpResponse, QianjiControlWorkflowSourceAuthoringMediaType,
    QianjiControlWorkflowSourceCompilerMode, QianjiControlWorkflowSourceRepairStartedHttpResponse,
    QianjiError, QianjiLlmClient, QianjiRunConsoleElementState, QianjiRuntimeBpmnInstanceIdRef,
    QianjiRuntimeInstantMs, QianjiRuntimeLeaseTtlMs, QianjiRuntimeWorkerIdRef,
    QianjiServerCliError, QianjiServerFlowhubServiceWorkerLoopOutput,
    QianjiServerFlowhubServiceWorkerLoopRequest, QianjiServerFlowhubServiceWorkerStepOutput,
    QianjiWorkflowLlmEndpointConfig, QianjiWorkflowLlmTaskConfig, QianjiWorkflowLlmTaskRetryConfig,
    QianjiWorkflowLlmTaskRouteConfig, SchedulerAgentIdentity, SwarmEvent, WorkflowCheckpointError,
    WorkflowCheckpointRef, WorkflowCheckpointStorageKind, WorkflowCompletionError,
    WorkflowEdgeKind, WorkflowExecutionError, WorkflowExecutionReport,
    WorkflowMemoryCheckpointStore, WorkflowPlan, WorkflowPlanDiagnostic,
    WorkflowPlanDiagnosticSeverity, WorkflowPlanEdge, WorkflowPlanEmitError, WorkflowPlanTask,
    WorkflowPlanValidationReport, WorkflowRun, WorkflowStage, WorkflowStageBinding,
    WorkflowStageFacts, WorkflowStageStatus, WorkflowStageTrace, WorkflowTopology,
    WorkflowTopologyEdge, WorkflowTopologyError, WorkflowTrace,
    build_bpmn_host_work_activity_result, build_bpmn_host_work_activity_schedule_record,
    build_bpmn_host_work_llm_activity_route, build_flowhub_service_activity_schedule_record,
    build_flowhub_service_activity_schedule_record_from_http_pending_work,
    build_flowhub_service_task_complete_http_request,
    build_flowhub_service_task_completion_payload,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, construct_cards, construct_index_entries,
    dispatch_pending_host_work_request, dispatch_pending_host_work_requests,
    emit_workflow_plan_bpmn, find_construct_card, load_bpmn_package_from_files,
    load_bpmn_package_from_files_with_options, qianji_bpmn_workflow_router,
    qianji_control_run_stream_rows, render_construct_card, render_construct_card_json,
    render_construct_index, render_construct_index_json, render_workflow_plan_validation_report,
    render_workflow_plan_validation_report_json, resolve_pending_host_work,
    resolve_qianji_workflow_llm_task_config, resolve_qianji_workflow_llm_task_config_with_env,
    resolve_waiting_external_event, run_qianji_server_cli,
    run_qianji_server_flowhub_service_worker_completion_loop, unix_millis_now,
    validate_workflow_plan,
};
#[cfg(feature = "wendao-integration")]
pub use api::{
    BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, MEMORY_PROMOTION_PIPELINE_TOML,
    QianjiApp, QianjiContractFeedbackRun, QianjiManifestPipelineRequest,
    QianjiPersistedContractFeedbackRun, QianjiPipelineDependencies, RESEARCH_TRINITY_TOML,
    WENDAO_SQL_AUTHORING_V1_TOML, WendaoDocsContractShow, WorkdirCheckFollowUpQuery,
    WorkdirSemanticEvidenceStatus, WorkdirSemanticProjectionPolicySummary,
    WorkdirSemanticScopeGuardStatus, WorkdirSemanticScopeGuardTrace,
    WorkdirSemanticScopeObjectKind, WorkdirSemanticScopeObjectStatus,
    WorkdirSemanticScopeObjectSummary, WorkdirSemanticSqlGuardSummary, WorkflowReport,
    build_workdir_check_follow_up_query, persist_contract_feedback_run,
    query_workdir_check_follow_up_payload, query_workdir_markdown_payload,
    render_wendao_docs_contract_show, render_workdir_semantic_scope_guard_trace,
    run_and_persist_contract_feedback_flow, run_contract_feedback_flow, run_scenario, run_workflow,
    run_workflow_from_manifest_toml, run_workflow_with_mounts, show_wendao_docs_contract,
    trace_workdir_semantic_scope_bundle, trace_workdir_semantic_scope_bundle_with_evidence,
    trace_workdir_semantic_scope_bundle_with_sql_guard_evidence, trace_workdir_semantic_scope_json,
    workdir_semantic_scope_guard_trace_json,
};
#[cfg(feature = "duckdb")]
pub use api::{
    DEFAULT_QIANJI_BPMN_DUCKDB_THREADS, QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
    QIANJI_RUN_CONSOLE_RUN_ID_HEADER, QianjiBpmnDataRecord, QianjiBpmnDataStoreError,
    QianjiBpmnDuckDbDataStore, QianjiBpmnDuckDbDataStoreConfig, QianjiRunConsoleArrowReadModel,
    QianjiRunConsoleFlightService, qianji_run_console_arrow_read_model,
    qianji_run_console_element_state_arrow_contract, qianji_run_console_element_state_arrow_schema,
    qianji_run_console_event_arrow_contract, qianji_run_console_event_arrow_schema,
};
#[cfg(all(
    feature = "llm",
    any(
        all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
        test
    )
))]
pub use api::{
    QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse,
    QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse,
};
#[cfg(all(feature = "llm", feature = "wendao-integration"))]
pub use api::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory,
};

#[path = "../tests/unit/support/valkey.rs"]
#[cfg(all(test, feature = "valkey"))]
pub(crate) mod qianji_test_valkey_support;
