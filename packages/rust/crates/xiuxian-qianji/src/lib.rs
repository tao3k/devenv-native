//! xiuxian-qianji: The Thousand Mechanisms Engine.
//!
//! A high-performance, probabilistic DAG executor based on petgraph.
//! Follows Rust 2024 Edition standards.

/// Application-layer scheduler factories and built-in pipeline presets.
#[cfg(feature = "qianji-full")]
pub mod app;
/// High-level laboratory API for end-to-end workflow execution.
#[cfg(feature = "qianji-full")]
pub mod bootcamp;
/// Thin BPMN host adapter helpers backed by `qianji-bpmn-engine`.
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
mod qianji_server_cli;
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
#[cfg(feature = "qianji-full")]
pub mod sovereign;
/// Multi-agent swarm orchestration runtime.
#[cfg(feature = "qianji-full")]
pub mod swarm;
/// Real-time swarm telemetry contracts and Valkey emitter.
pub mod telemetry;
/// Bounded work-surface parsing, validation, and CLI support helpers.
#[cfg(feature = "qianji-full")]
pub mod workdir;

#[cfg(feature = "qianji-full")]
pub use app::{
    MEMORY_PROMOTION_PIPELINE_TOML, QianjiApp, RESEARCH_TRINITY_TOML, WENDAO_SQL_AUTHORING_V1_TOML,
};
#[cfg(feature = "qianji-full")]
pub use bootcamp::{
    BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, WorkflowReport, run_scenario,
    run_workflow, run_workflow_from_manifest_toml, run_workflow_with_mounts,
};
pub use bpmn::{
    BpmnAdapterError, BpmnOrchestrationError, DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    QianjiBpmnCheckpointStore, QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade,
    QianjiBpmnExecutionMode, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnExecutionScheduler, QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder,
    QianjiBpmnPendingHostCompletion, QianjiBpmnPendingHostWorkHttpResponse,
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnSchedulerLeaseConfig, QianjiBpmnSession, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowCancelHttpResponse, QianjiBpmnWorkflowCancelReport,
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowEventPollRequest,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpErrorBody,
    QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowInstanceSummary,
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
    QianjiBpmnWorkflowTaskClaimRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionHttpPayload,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiBpmnWorkflowTaskReleaseHttpPayload, QianjiBpmnWorkflowTaskReleaseHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseReport, QianjiBpmnWorkflowTaskReleaseRequest,
    QianjiBpmnWorkflowWorklistItem, QianjiBpmnWorkflowWorklistReport,
    QianjiBpmnWorkflowWorklistRequest, QianjiBpmnWorkflowWorklistRoutingFilter,
    dispatch_pending_host_work_request, dispatch_pending_host_work_requests,
    load_bpmn_package_from_files, load_bpmn_package_from_files_with_options,
    qianji_bpmn_workflow_router, resolve_pending_host_work, resolve_waiting_external_event,
};
#[cfg(feature = "duckdb")]
pub use bpmn::{
    DEFAULT_QIANJI_BPMN_DUCKDB_THREADS, QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
    QianjiBpmnDataRecord, QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStore,
    QianjiBpmnDuckDbDataStoreConfig,
};
pub use construct_cards::{
    ConstructCard, ConstructIndexEntry, ConstructLintMapping, ConstructStatus, construct_cards,
    construct_index_entries, find_construct_card, render_construct_card,
    render_construct_card_json, render_construct_index, render_construct_index_json,
};
pub use construct_plan::{
    WorkflowPlan, WorkflowPlanDiagnostic, WorkflowPlanDiagnosticSeverity, WorkflowPlanEdge,
    WorkflowPlanEmitError, WorkflowPlanTask, WorkflowPlanValidationReport, emit_workflow_plan_bpmn,
    render_workflow_plan_validation_report, render_workflow_plan_validation_report_json,
    validate_workflow_plan,
};
#[cfg(feature = "qianji-full")]
pub use contract_feedback::{QianjiContractFeedbackRun, run_contract_feedback_flow};
#[cfg(all(feature = "llm", feature = "qianji-full"))]
pub use contract_feedback::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory,
};
#[cfg(feature = "qianji-full")]
pub use contract_feedback::{
    QianjiPersistedContractFeedbackRun, persist_contract_feedback_run,
    run_and_persist_contract_feedback_flow,
};
#[cfg(feature = "qianji-full")]
pub use contracts::{
    FlowInstruction, FlowhubGraphTopology, NodeQianhuanExecutionMode, NodeStatus, QianjiManifest,
    QianjiMechanism, QianjiOutput, WendaoDocsContractShow, render_wendao_docs_contract_show,
    show_wendao_docs_contract,
};
#[cfg(feature = "qianji-full")]
pub use engine::{QianjiCompiler, QianjiEngine};
#[cfg(feature = "qianji-full")]
pub use flowhub::{
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
pub use llm_client::QianjiLlmClient;
#[cfg(feature = "qianji-full")]
pub use manifest::{manifest_declares_qianhuan_bindings, manifest_requires_llm};
#[cfg(feature = "qianji-full")]
pub use qianji_cli::run_qianji_cli;
pub use qianji_server_cli::run_qianji_server_cli;
#[cfg(feature = "qianji-full")]
pub use safety::QianjiSafetyGuard;
#[cfg(feature = "qianji-full")]
pub use scheduler::QianjiScheduler;
#[cfg(feature = "qianji-full")]
pub use scheduler::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
pub use scheduler_identity::SchedulerAgentIdentity;
#[cfg(feature = "qianji-full")]
pub use swarm::{
    ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry, RemoteNodeRequest,
    RemoteNodeResponse, RemotePossessionBus, SwarmAgentConfig, SwarmAgentReport, SwarmEngine,
    SwarmExecutionOptions, SwarmExecutionReport, map_execution_error_to_response,
};
pub use telemetry::{
    ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase, NoopPulseEmitter, PulseEmitter,
    SwarmEvent, ValkeyPulseEmitter, unix_millis_now,
};
#[cfg(feature = "qianji-full")]
pub use workdir::{
    WorkdirAdvance, WorkdirCheckFollowUpQuery, WorkdirCheckReport, WorkdirDiagnostic,
    WorkdirMarkdownSurface, WorkdirShow, WorkdirVisibleSurface, WorkdirVisibleSurfaceKind,
    advance_workdir_step, build_workdir_check_follow_up_query, check_workdir,
    load_workdir_manifest, looks_like_workdir_dir, parse_workdir_manifest,
    query_workdir_check_follow_up_payload, query_workdir_markdown_payload, render_workdir_advance,
    render_workdir_check_markdown, render_workdir_show, show_workdir,
};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();
