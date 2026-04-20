//! xiuxian-qianji: The Thousand Mechanisms Engine.
//!
//! A high-performance, probabilistic DAG executor based on petgraph.
//! Follows Rust 2024 Edition standards.

/// Application-layer scheduler factories and built-in pipeline presets.
pub mod app;
/// High-level laboratory API for end-to-end workflow execution.
pub mod bootcamp;
/// Thin BPMN host adapter helpers backed by `qianji-bpmn-engine`.
pub mod bpmn;
/// Distributed consensus management for multi-agent synchronization.
pub mod consensus;
/// Contract-feedback execution bridge for contract suite runs and Wendao export.
pub mod contract_feedback;
/// Contract definitions for nodes, instructions, and manifests.
pub mod contracts;
/// Core graph engine based on petgraph.
pub mod engine;
/// Unified error handling.
pub mod error;
/// Built-in node execution mechanisms.
pub mod executors;
/// Flowhub module, scenario, and materialize helpers.
pub mod flowhub;
/// Graphical layout and aesthetic engine (QGS).
pub mod layout;
/// Manifest inspection helpers.
pub mod manifest;
/// Shared markdown renderers for `qianji` show/check surfaces.
pub(crate) mod markdown;
/// Runtime configuration resolver (`resources/config/qianji.toml` + user overrides).
pub mod runtime_config;
/// Formal logic and safety auditing.
pub mod safety;
/// Asynchronous synaptic-flow scheduler.
pub mod scheduler;
mod scheduler_checkpoint;
mod scheduler_identity;
mod scheduler_policy;
mod scheduler_preflight;
mod scheduler_state;
/// Sovereign Memory Module (Blueprint V6.1) - Agent reasoning trace persistence.
pub mod sovereign;
/// Multi-agent swarm orchestration runtime.
pub mod swarm;
/// Real-time swarm telemetry contracts and Valkey emitter.
pub mod telemetry;
/// Bounded work-surface parsing, validation, and CLI support helpers.
pub mod workdir;

pub use app::{
    MEMORY_PROMOTION_PIPELINE_TOML, QianjiApp, RESEARCH_TRINITY_TOML, WENDAO_SQL_AUTHORING_V1_TOML,
};
pub use bootcamp::{
    BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, WorkflowReport, run_scenario,
    run_workflow, run_workflow_from_manifest_toml, run_workflow_with_mounts,
};
pub use bpmn::{
    BpmnAdapterError, BpmnOrchestrationError, DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    QianjiBpmnCheckpointStore, QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade,
    QianjiBpmnExecutionMode, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnExecutionScheduler, QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder,
    QianjiBpmnSchedulerLeaseConfig, QianjiBpmnSession, dispatch_pending_host_work_request,
    dispatch_pending_host_work_requests, load_bpmn_package_from_files,
    load_bpmn_package_from_files_with_options, resolve_pending_host_work,
    resolve_waiting_external_event,
};
pub use contract_feedback::{QianjiContractFeedbackRun, run_contract_feedback_flow};
#[cfg(feature = "llm")]
pub use contract_feedback::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory,
};
pub use contract_feedback::{
    QianjiPersistedContractFeedbackRun, persist_contract_feedback_run,
    run_and_persist_contract_feedback_flow,
};
pub use contracts::{
    FlowInstruction, FlowhubGraphTopology, NodeQianhuanExecutionMode, NodeStatus, QianjiManifest,
    QianjiMechanism, QianjiOutput, WendaoDocsContractShow, render_wendao_docs_contract_show,
    show_wendao_docs_contract,
};
pub use engine::{QianjiCompiler, QianjiEngine};
pub use flowhub::{
    AnchoredMaterializedWorkdir, FlowhubCheckReport, FlowhubDiagnostic, FlowhubDirKind,
    FlowhubGraphEdgeSummary, FlowhubGraphNodeSummary, FlowhubGraphShow, FlowhubModuleKind,
    FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow, FlowhubScenarioCaseSummary,
    FlowhubScenarioCheckReport, FlowhubScenarioDiagnostic, FlowhubScenarioHiddenAlias,
    FlowhubScenarioShow, FlowhubScenarioSurfacePreview, FlowhubShow, MaterializedWorkdir,
    ResolvedFlowhubModule, check_flowhub, check_flowhub_scenario, classify_flowhub_dir,
    load_flowhub_module_manifest, load_flowhub_scenario_manifest, looks_like_flowhub_scenario_dir,
    materialize_flowhub_anchored_scenario, materialize_flowhub_anchored_scenario_at_node,
    materialize_flowhub_scenario_workdir, parse_flowhub_module_manifest,
    parse_flowhub_scenario_manifest, render_anchored_materialized_workdir,
    render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_scenario_check_markdown, render_flowhub_scenario_show, render_flowhub_show,
    resolve_flowhub_module_children, resolve_flowhub_scenario_modules, show_flowhub,
    show_flowhub_anchored_scenario, show_flowhub_graph, show_flowhub_scenario,
};
pub use manifest::{manifest_declares_qianhuan_bindings, manifest_requires_llm};
pub use safety::QianjiSafetyGuard;
pub use scheduler::QianjiScheduler;
pub use scheduler::SchedulerAgentIdentity;
pub use scheduler::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
pub use swarm::{
    ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry, RemoteNodeRequest,
    RemoteNodeResponse, RemotePossessionBus, SwarmAgentConfig, SwarmAgentReport, SwarmEngine,
    SwarmExecutionOptions, SwarmExecutionReport, map_execution_error_to_response,
};
pub use telemetry::{
    ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase, NoopPulseEmitter, PulseEmitter,
    SwarmEvent, ValkeyPulseEmitter, unix_millis_now,
};
pub use workdir::{
    WorkdirAdvance, WorkdirCheckFollowUpQuery, WorkdirCheckReport, WorkdirDiagnostic,
    WorkdirMarkdownSurface, WorkdirShow, WorkdirVisibleSurface, WorkdirVisibleSurfaceKind,
    advance_workdir_step, build_workdir_check_follow_up_query, check_workdir,
    load_workdir_manifest, looks_like_workdir_dir, parse_workdir_manifest,
    query_workdir_check_follow_up_payload, query_workdir_markdown_payload, render_workdir_advance,
    render_workdir_check_markdown, render_workdir_show, show_workdir,
};

#[cfg(feature = "llm")]
/// Shared LLM client trait object type when `llm` feature is enabled.
pub type QianjiLlmClient = dyn xiuxian_llm::llm::LlmClient;

#[cfg(not(feature = "llm"))]
/// Placeholder trait object type when `llm` feature is disabled.
pub type QianjiLlmClient = dyn std::any::Any + Send + Sync;

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");
