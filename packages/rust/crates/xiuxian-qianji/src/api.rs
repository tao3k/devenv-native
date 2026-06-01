//! Crate facade exports grouped away from the root module surface.

#[cfg(feature = "wendao-integration")]
pub use crate::app::{
    MEMORY_PROMOTION_PIPELINE_TOML, QianjiApp, QianjiManifestPipelineRequest,
    QianjiPipelineDependencies, RESEARCH_TRINITY_TOML, WENDAO_SQL_AUTHORING_V1_TOML,
};
#[cfg(feature = "wendao-integration")]
pub use crate::bootcamp::{
    BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, WorkflowReport, run_scenario,
    run_workflow, run_workflow_from_manifest_toml, run_workflow_with_mounts,
};
pub use crate::bpmn::{
    BPMN_HOST_WORK_ACTIVITY_METADATA_KEY, BPMN_HOST_WORK_ACTIVITY_SCHEMA,
    BPMN_HOST_WORK_ACTIVITY_TYPE, BPMN_HOST_WORK_COMPLETION_METADATA_KEY,
    BPMN_HOST_WORK_COMPLETION_SCHEMA, BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA, BpmnAdapterError,
    BpmnHostWorkActivityScheduleInput, BpmnHostWorkLlmActivityRouteInput,
    BpmnHostWorkLlmEndpointDecision, BpmnHostWorkLlmRouteDecision, BpmnOrchestrationError,
    BpmnUnsupportedStartNodeKind, DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FlowhubScenarioIdRef, FlowhubServiceActivityHttpScheduleInput,
    FlowhubServiceActivityScheduleInput, QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE,
    QIANJI_RUN_CONSOLE_EVENT_ROUTE, QIANJI_RUN_CONSOLE_SCHEMA_VERSION, QianjiBpmnActivityId,
    QianjiBpmnCheckpointStore, QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade,
    QianjiBpmnExecutionMode, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnExecutionScheduler, QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder,
    QianjiBpmnLeaseOwnerToken, QianjiBpmnPackageId, QianjiBpmnPendingHostCompletion,
    QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnPreparedWorkflowResume,
    QianjiBpmnPreparedWorkflowStart, QianjiBpmnProcessId, QianjiBpmnSchedulerLeaseConfig,
    QianjiBpmnSession, QianjiBpmnStartAtNodeId, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowCancelHttpResponse, QianjiBpmnWorkflowCancelReport,
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowEventPollRequest,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpErrorBody,
    QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowInstanceId, QianjiBpmnWorkflowInstanceSummary,
    QianjiBpmnWorkflowInstancesReport, QianjiBpmnWorkflowInstancesRequest,
    QianjiBpmnWorkflowInterruptReport, QianjiBpmnWorkflowInterruptRequest,
    QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStartReport,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowStatusReport,
    QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowTaskClaimHttpPayload,
    QianjiBpmnWorkflowTaskClaimHttpRequest, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskClaimPayload, QianjiBpmnWorkflowTaskClaimReport,
    QianjiBpmnWorkflowTaskClaimRequest, QianjiBpmnWorkflowTaskCompleteBatchHttpRequest,
    QianjiBpmnWorkflowTaskCompleteBatchReport, QianjiBpmnWorkflowTaskCompleteBatchRequest,
    QianjiBpmnWorkflowTaskCompleteHttpRequest, QianjiBpmnWorkflowTaskCompleteReport,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionHttpKind,
    QianjiBpmnWorkflowTaskCompletionHttpPayload, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiBpmnWorkflowTaskReleaseHttpPayload,
    QianjiBpmnWorkflowTaskReleaseHttpRequest, QianjiBpmnWorkflowTaskReleaseHttpResponse,
    QianjiBpmnWorkflowTaskReleasePayload, QianjiBpmnWorkflowTaskReleaseReport,
    QianjiBpmnWorkflowTaskReleaseRequest, QianjiBpmnWorkflowWorklistItem,
    QianjiBpmnWorkflowWorklistReport, QianjiBpmnWorkflowWorklistRequest,
    QianjiBpmnWorkflowWorklistRoutingFilter, QianjiControlDiagnosticsHttpResponse,
    QianjiControlHistoryHttpResponse, QianjiControlRecoveryApplyHttpRequest,
    QianjiControlRecoveryApplyHttpResponse, QianjiControlRecoveryHttpResponse,
    QianjiControlRunStreamSource, QianjiControlRunSummaryHttpResponse,
    QianjiControlWorkflowSourceAdmissionHttpRequest,
    QianjiControlWorkflowSourceAdmissionHttpResponse,
    QianjiControlWorkflowSourceAdmittedHttpResponse, QianjiControlWorkflowSourceAuthoringMediaType,
    QianjiControlWorkflowSourceCompilerMode, QianjiControlWorkflowSourceRepairStartedHttpResponse,
    QianjiRunConsoleElementState, QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    QianjiRuntimeLeaseTtlMs, QianjiRuntimeWorkerIdRef, build_bpmn_host_work_activity_result,
    build_bpmn_host_work_activity_schedule_record, build_bpmn_host_work_llm_activity_route,
    build_flowhub_service_activity_schedule_record,
    build_flowhub_service_activity_schedule_record_from_http_pending_work,
    build_flowhub_service_task_complete_http_request,
    build_flowhub_service_task_completion_payload,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, dispatch_pending_host_work_request,
    dispatch_pending_host_work_requests, load_bpmn_package_from_files,
    load_bpmn_package_from_files_with_options, qianji_bpmn_workflow_router,
    qianji_control_run_stream_rows, resolve_pending_host_work, resolve_waiting_external_event,
};
#[cfg(feature = "duckdb")]
pub use crate::bpmn::{
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
pub use crate::bpmn::{
    QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse,
    QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse,
};
pub use crate::construct_cards::{
    ConstructCard, ConstructIndexEntry, ConstructLintMapping, ConstructStatus, construct_cards,
    construct_index_entries, find_construct_card, render_construct_card,
    render_construct_card_json, render_construct_index, render_construct_index_json,
};
pub use crate::construct_plan::{
    WorkflowPlan, WorkflowPlanDiagnostic, WorkflowPlanDiagnosticSeverity, WorkflowPlanEdge,
    WorkflowPlanEmitError, WorkflowPlanTask, WorkflowPlanValidationReport, emit_workflow_plan_bpmn,
    render_workflow_plan_validation_report, render_workflow_plan_validation_report_json,
    validate_workflow_plan,
};
#[cfg(feature = "wendao-integration")]
pub use crate::contract_feedback::{QianjiContractFeedbackRun, run_contract_feedback_flow};
#[cfg(all(feature = "llm", feature = "wendao-integration"))]
pub use crate::contract_feedback::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory,
};
#[cfg(feature = "wendao-integration")]
pub use crate::contract_feedback::{
    QianjiPersistedContractFeedbackRun, persist_contract_feedback_run,
    run_and_persist_contract_feedback_flow,
};
#[cfg(feature = "qianji-full")]
pub use crate::contracts::{
    FlowInstruction, FlowhubGraphTopology, NodeQianhuanExecutionMode, NodeStatus, QianjiManifest,
    QianjiMechanism, QianjiOutput,
};
#[cfg(feature = "wendao-integration")]
pub use crate::contracts::{
    WendaoDocsContractShow, render_wendao_docs_contract_show, show_wendao_docs_contract,
};
#[cfg(feature = "qianji-full")]
pub use crate::engine::{QianjiCompiler, QianjiEngine};
#[cfg(feature = "qianji-full")]
pub use crate::flowhub::{
    AnchoredMaterializedWorkdir, FlowhubCheckReport, FlowhubDiagnostic, FlowhubDirKind,
    FlowhubGraphShow, FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow,
    FlowhubScenarioCaseSummary, FlowhubScenarioCheckReport, FlowhubScenarioDiagnostic,
    FlowhubScenarioHiddenAlias, FlowhubScenarioShow, FlowhubScenarioSurfacePreview, FlowhubShow,
    MaterializedWorkdir, ResolvedFlowhubModule, check_flowhub, check_flowhub_scenario,
    classify_flowhub_dir, load_flowhub_module_manifest, load_flowhub_scenario_manifest,
    looks_like_flowhub_scenario_dir, materialize_flowhub_anchored_scenario,
    materialize_flowhub_anchored_scenario_at_node, materialize_flowhub_scenario_workdir,
    parse_flowhub_module_manifest, parse_flowhub_scenario_manifest,
    render_anchored_materialized_workdir, render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_scenario_check_markdown, render_flowhub_scenario_show, render_flowhub_show,
    resolve_flowhub_module_children, resolve_flowhub_scenario_modules, show_flowhub,
    show_flowhub_anchored_scenario, show_flowhub_graph, show_flowhub_scenario,
};
#[cfg(feature = "qianji-full")]
pub use crate::manifest::{manifest_declares_qianhuan_bindings, manifest_requires_llm};
#[cfg(feature = "qianji-full")]
pub use crate::qianji_cli::{QianjiCliError, run_qianji_cli};
#[cfg(feature = "qianji-full")]
pub use crate::safety::QianjiSafetyGuard;
#[cfg(feature = "qianji-full")]
pub use crate::scheduler::QianjiScheduler;
#[cfg(feature = "qianji-full")]
pub use crate::scheduler_checkpoint::QianjiStateSnapshot;
#[cfg(feature = "qianji-full")]
pub use crate::scheduler_policy::{
    RoleAvailabilityRegistry, SchedulerExcludedClusterRef, SchedulerExecutionPolicy,
};
#[cfg(feature = "qianji-full")]
pub use crate::swarm::{
    ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry, RemoteNodeRequest,
    RemoteNodeResponse, RemotePossessionBus, RemotePossessionBusError, SwarmAgentConfig,
    SwarmAgentReport, SwarmEngine, SwarmExecutionOptions, SwarmExecutionReport,
    map_execution_error_to_response,
};
#[cfg(feature = "valkey")]
pub use crate::telemetry::ValkeyPulseEmitter;
pub use crate::telemetry::{
    ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase, NoopPulseEmitter, PulseEmitter,
    SwarmEvent, unix_millis_now,
};
#[cfg(feature = "qianji-full")]
pub use crate::workdir::{
    WorkdirAdvance, WorkdirCheckReport, WorkdirDiagnostic, WorkdirMarkdownSurface, WorkdirShow,
    WorkdirVisibleSurface, WorkdirVisibleSurfaceKind, advance_workdir_step, check_workdir,
    load_workdir_manifest, looks_like_workdir_dir, parse_workdir_manifest, render_workdir_advance,
    render_workdir_check_markdown, render_workdir_show, show_workdir,
};
#[cfg(feature = "wendao-integration")]
pub use crate::workdir::{
    WorkdirCheckFollowUpQuery, WorkdirSemanticEvidenceStatus,
    WorkdirSemanticProjectionPolicySummary, WorkdirSemanticScopeGuardStatus,
    WorkdirSemanticScopeGuardTrace, WorkdirSemanticScopeObjectKind,
    WorkdirSemanticScopeObjectStatus, WorkdirSemanticScopeObjectSummary,
    WorkdirSemanticSqlGuardSummary, build_workdir_check_follow_up_query,
    query_workdir_check_follow_up_payload, query_workdir_markdown_payload,
    render_workdir_semantic_scope_guard_trace, trace_workdir_semantic_scope_bundle,
    trace_workdir_semantic_scope_bundle_with_evidence,
    trace_workdir_semantic_scope_bundle_with_sql_guard_evidence, trace_workdir_semantic_scope_json,
    workdir_semantic_scope_guard_trace_json,
};
pub use crate::workflow_config::{
    DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE, QianjiWorkflowLlmEndpointConfig,
    QianjiWorkflowLlmTaskConfig, QianjiWorkflowLlmTaskRetryConfig,
    QianjiWorkflowLlmTaskRouteConfig, resolve_qianji_workflow_llm_task_config,
    resolve_qianji_workflow_llm_task_config_with_env,
};
pub use crate::workflow_kernel::{
    WorkflowCheckpointError, WorkflowCheckpointRef, WorkflowCheckpointStorageKind,
    WorkflowCompletionError, WorkflowEdgeKind, WorkflowExecutionError, WorkflowExecutionReport,
    WorkflowMemoryCheckpointStore, WorkflowRun, WorkflowStage, WorkflowStageBinding,
    WorkflowStageFacts, WorkflowStageStatus, WorkflowStageTrace, WorkflowTopology,
    WorkflowTopologyEdge, WorkflowTopologyError, WorkflowTrace,
};
pub use crate::{
    error::QianjiError, llm_client::QianjiLlmClient, scheduler_identity::SchedulerAgentIdentity,
};
pub use crate::{
    qianji_server::flowhub_worker::{
        QianjiServerFlowhubServiceWorkerLoopOutput, QianjiServerFlowhubServiceWorkerLoopRequest,
        QianjiServerFlowhubServiceWorkerStepOutput,
        run_qianji_server_flowhub_service_worker_completion_loop,
    },
    qianji_server_cli::{QianjiServerCliError, run_qianji_server_cli},
};
