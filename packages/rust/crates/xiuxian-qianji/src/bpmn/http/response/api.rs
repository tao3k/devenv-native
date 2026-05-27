//! HTTP response DTOs for BPMN workflow routes.

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
use crate::QianjiServerOpenAiCompatibleLlmWorkerLoopOutput;
use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStatusReport,
    QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowTaskReleaseReport,
};
use crate::bpmn::driver::QianjiBpmnExecutionReport;
use crate::bpmn::identity::{
    QianjiBpmnActivityId, QianjiBpmnProcessId, QianjiBpmnWorkflowInstanceId,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec,
    BpmnHumanTaskLifecycleEvent, BpmnInstanceState, BpmnLaneMembershipSpec, BpmnTaskIoSpec,
    BpmnTaskOutputBinding, InstanceLifecycle, PendingHostWork, PendingHostWorkClaim,
    PendingHostWorkKind, PendingHostWorkRequest, RepeatExecutionContext,
    build_pending_host_work_requests,
};
use xiuxian_qianji_control::{
    ControlEventRecord, RecoveryLoopApplication, RunOperatorDiagnostics, RunOperatorSummary,
    RunRecoverySnapshot,
};

/// Raw DTO boundary: pending host-work item embedded in HTTP workflow
/// snapshots.
///
/// Primitive fields mirror the public JSON wire contract; typed identity is
/// restored before durable control-ledger scheduling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnPendingHostWorkHttpResponse {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier for the pending host work.
    pub process_id: Option<QianjiBpmnProcessId>,
    /// BPMN node index.
    pub node_index: u32,
    /// Stable BPMN node identifier, when the snapshot still has the process
    /// graph needed to resolve the node index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Stable BPMN activity identifier for the blocked node.
    pub activity_id: Option<QianjiBpmnActivityId>,
    /// Host work category.
    pub kind: PendingHostWorkKind,
    /// Optional host-generated work identifier.
    pub work_id: Option<String>,
    /// Optional human-task form metadata preserved for host rendering.
    pub form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN assignment metadata preserved for host routing.
    pub assignment: Option<BpmnHumanTaskAssignmentSpec>,
    /// Optional BPMN lane membership metadata preserved for host routing.
    pub lane: Option<BpmnLaneMembershipSpec>,
    /// Optional standard BPMN task IO metadata preserved for host routing.
    pub task_io: Option<BpmnTaskIoSpec>,
    /// Optional checkpointed claim metadata.
    pub claim: Option<PendingHostWorkClaim>,
    /// Host-visible workflow variables for this dispatch request.
    #[serde(default)]
    pub variables: Value,
    /// Resolved standard BPMN task inputs for this dispatch request.
    #[serde(default)]
    pub inputs: Value,
    /// Declared standard BPMN task outputs for strict completion mapping.
    #[serde(default)]
    pub output_bindings: Vec<BpmnTaskOutputBinding>,
    /// Optional repeat-execution metadata for this blocked host request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<RepeatExecutionContext>,
}

impl QianjiBpmnPendingHostWorkHttpResponse {
    fn from_instance_pending_host_work(
        instance: &BpmnInstanceState,
        work: &PendingHostWork,
    ) -> Self {
        Self::from_pending_host_work_with_details(
            work,
            PendingHostWorkHttpDispatchDetails::from_instance_pending_work(instance, work),
        )
    }

    fn from_pending_host_work_with_details(
        work: &PendingHostWork,
        details: PendingHostWorkHttpDispatchDetails,
    ) -> Self {
        Self {
            token_id: work.token_id,
            process_id: work
                .process_id
                .as_ref()
                .map(|process_id| process_id.as_str().into()),
            node_index: work.node_index,
            node_id: details.node_id,
            activity_id: work
                .activity_id
                .as_ref()
                .map(|activity_id| activity_id.as_str().into()),
            kind: work.kind.clone(),
            work_id: work
                .work_id
                .as_ref()
                .map(|work_id| work_id.as_str().to_owned()),
            form: work.human_task_form.clone(),
            assignment: work.human_task_assignment.clone(),
            lane: work.lane.clone(),
            task_io: work.task_io.clone(),
            claim: work.claim.clone(),
            variables: details.variables,
            inputs: details.inputs,
            output_bindings: details.output_bindings,
            repeat: details.repeat,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PendingHostWorkHttpDispatchDetails {
    node_id: Option<String>,
    variables: Value,
    inputs: Value,
    output_bindings: Vec<BpmnTaskOutputBinding>,
    repeat: Option<RepeatExecutionContext>,
}

impl Default for PendingHostWorkHttpDispatchDetails {
    fn default() -> Self {
        Self {
            node_id: None,
            variables: Value::Null,
            inputs: Value::Object(Map::default()),
            output_bindings: Vec::new(),
            repeat: None,
        }
    }
}

impl PendingHostWorkHttpDispatchDetails {
    fn from_instance_pending_work(instance: &BpmnInstanceState, work: &PendingHostWork) -> Self {
        let mut details = Self {
            node_id: work
                .activity_id
                .as_ref()
                .map(|activity_id| activity_id.to_string())
                .or_else(|| Some(format!("node_{}", work.node_index))),
            ..Self::default()
        };
        if let Some(request) =
            build_pending_host_work_requests(instance)
                .ok()
                .and_then(|requests| {
                    requests.into_iter().find(|request| {
                        pending_host_work_request_token_id(request) == work.token_id
                    })
                })
        {
            details.apply_request(request);
        }
        details
    }

    fn apply_request(&mut self, request: PendingHostWorkRequest) {
        match request {
            PendingHostWorkRequest::Task(request) => {
                self.variables = request.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
                self.repeat = request.repeat;
            }
            PendingHostWorkRequest::Send(request) => {
                self.variables = request.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
            }
            PendingHostWorkRequest::Service(request) => {
                self.variables = request.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
                self.repeat = request.repeat;
            }
            PendingHostWorkRequest::Script(request) => {
                self.variables = request.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
                self.repeat = request.repeat;
            }
            PendingHostWorkRequest::User(request) => {
                self.variables = request.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
                self.repeat = request.repeat;
            }
            PendingHostWorkRequest::Manual(request) => {
                self.variables = request.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
                self.repeat = request.repeat;
            }
            PendingHostWorkRequest::BusinessRule(request) => {
                self.variables = request.evaluation.variables;
                self.inputs = request.inputs;
                self.output_bindings = request.output_bindings;
                self.repeat = request.repeat;
            }
        }
    }
}

fn pending_host_work_request_token_id(request: &PendingHostWorkRequest) -> u64 {
    match request {
        PendingHostWorkRequest::Task(request) => request.token_id.get(),
        PendingHostWorkRequest::Send(request) => request.token_id,
        PendingHostWorkRequest::Service(request) => request.token_id,
        PendingHostWorkRequest::Script(request) => request.token_id,
        PendingHostWorkRequest::User(request) => request.token_id.get(),
        PendingHostWorkRequest::Manual(request) => request.token_id.get(),
        PendingHostWorkRequest::BusinessRule(request) => request.token_id,
    }
}

/// HTTP response for one control-ledger run history query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlHistoryHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Number of control event records returned.
    pub event_count: usize,
    /// Append-only control event records for the run.
    #[serde(default)]
    pub events: Vec<ControlEventRecord>,
}

impl QianjiControlHistoryHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(
        run_id: String,
        events: Vec<ControlEventRecord>,
    ) -> Self {
        Self {
            run_id,
            event_count: events.len(),
            events,
        }
    }
}

/// HTTP response carrying the server-recorded BPMN source for one control run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiControlBpmnSourceHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Server-recorded BPMN source reference.
    pub source_ref: String,
    /// Media type of the returned source payload.
    pub media_type: QianjiControlBpmnSourceMediaType,
    /// Raw BPMN XML read by qianji-server from the recorded source reference.
    pub bpmn_xml: String,
}

/// Media type for a server-recorded BPMN source payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QianjiControlBpmnSourceMediaType {
    /// BPMN 2.0 XML.
    #[serde(rename = "application/bpmn+xml")]
    ApplicationBpmnXml,
}

impl QianjiControlBpmnSourceHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(
        run_id: String,
        source_ref: String,
        bpmn_xml: String,
    ) -> Self {
        Self {
            run_id,
            source_ref,
            media_type: QianjiControlBpmnSourceMediaType::ApplicationBpmnXml,
            bpmn_xml,
        }
    }
}

/// HTTP response for one control-ledger operator summary query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlRunSummaryHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Replay-derived operator-safe run summary.
    pub summary: RunOperatorSummary,
}

impl QianjiControlRunSummaryHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(summary: RunOperatorSummary) -> Self {
        Self {
            run_id: summary.run_id.as_str().to_owned(),
            summary,
        }
    }
}

/// HTTP response for one control-ledger recovery snapshot query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlRecoveryHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Replay-derived recovery snapshot and ordered recovery actions.
    pub recovery: RunRecoverySnapshot,
}

impl QianjiControlRecoveryHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(recovery: RunRecoverySnapshot) -> Self {
        Self {
            run_id: recovery.run_id.as_str().to_owned(),
            recovery,
        }
    }
}

/// HTTP response for one combined control-ledger diagnostics query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlDiagnosticsHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Replay-derived operator diagnostics package.
    pub diagnostics: RunOperatorDiagnostics,
}

impl QianjiControlDiagnosticsHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(diagnostics: RunOperatorDiagnostics) -> Self {
        Self {
            run_id: diagnostics.run_id.as_str().to_owned(),
            diagnostics,
        }
    }
}

/// HTTP response for explicit recovery-plan application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlRecoveryApplyHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Bounded recovery application trace.
    pub application: RecoveryLoopApplication,
    /// Replay-derived diagnostics after application.
    pub diagnostics: RunOperatorDiagnostics,
}

impl QianjiControlRecoveryApplyHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(
        application: RecoveryLoopApplication,
        diagnostics: RunOperatorDiagnostics,
    ) -> Self {
        Self {
            run_id: diagnostics.run_id.as_str().to_owned(),
            application,
            diagnostics,
        }
    }
}

/// HTTP response for one bounded qianji-server OpenAI-compatible LLM worker run.
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Bounded qianji-server worker-loop trace.
    pub worker: QianjiServerOpenAiCompatibleLlmWorkerLoopOutput,
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
impl QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(
        run_id: String,
        worker: QianjiServerOpenAiCompatibleLlmWorkerLoopOutput,
    ) -> Self {
        Self { run_id, worker }
    }
}

/// HTTP response for one bounded qianji-server OpenAI-compatible LLM worker
/// run that also completes matching BPMN host work server-side.
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
#[derive(Debug, Clone, Serialize)]
pub struct QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Bounded worker-loop traces executed by qianji-server.
    pub worker_runs: Vec<QianjiServerOpenAiCompatibleLlmWorkerLoopOutput>,
    /// Number of BPMN host-work completions applied by qianji-server.
    pub completed_count: usize,
    /// Last workflow response after server-owned BPMN completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_workflow: Option<QianjiBpmnWorkflowRunHttpResponse>,
}

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
impl QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse {
    pub(in crate::bpmn::http_transport) fn new(
        run_id: String,
        worker_runs: Vec<QianjiServerOpenAiCompatibleLlmWorkerLoopOutput>,
        completed_count: usize,
        final_workflow: Option<QianjiBpmnWorkflowRunHttpResponse>,
    ) -> Self {
        Self {
            run_id,
            worker_runs,
            completed_count,
            final_workflow,
        }
    }
}

/// Compact runtime snapshot embedded in HTTP responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowSnapshotHttpResponse {
    /// Stable workflow instance identifier.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Stable BPMN process identifier.
    pub process_id: QianjiBpmnProcessId,
    /// Monotonic runtime sequence.
    pub sequence: u64,
    /// High-level BPMN instance lifecycle.
    pub lifecycle: InstanceLifecycle,
    /// Current workflow variables.
    pub variables: Value,
    /// Number of active host-work items.
    pub pending_host_work_count: usize,
    /// Active host-work items with Rust-owned identity and human-task metadata.
    #[serde(default)]
    pub pending_host_work: Vec<QianjiBpmnPendingHostWorkHttpResponse>,
    /// Durable lifecycle events for BPMN `userTask` and `manualTask`.
    #[serde(default)]
    pub human_task_events: Vec<BpmnHumanTaskLifecycleEvent>,
    /// Number of active external wait registrations.
    pub wait_registration_count: usize,
    /// Number of active runtime tokens.
    pub active_token_count: usize,
}

impl QianjiBpmnWorkflowSnapshotHttpResponse {
    /// Creates one compact HTTP snapshot from an engine instance state.
    #[must_use]
    pub fn from_instance(instance: &BpmnInstanceState) -> Self {
        Self {
            instance_id: instance.instance_id.as_ref().into(),
            process_id: instance.process.process_id.as_ref().into(),
            sequence: instance.sequence,
            lifecycle: instance.lifecycle.clone(),
            variables: instance.variables.clone(),
            pending_host_work_count: instance.pending_host_work.len(),
            pending_host_work: instance
                .pending_host_work
                .iter()
                .map(|work| {
                    QianjiBpmnPendingHostWorkHttpResponse::from_instance_pending_host_work(
                        instance, work,
                    )
                })
                .collect(),
            human_task_events: instance.human_task_events.clone(),
            wait_registration_count: instance.waits.len(),
            active_token_count: instance.active_tokens.len(),
        }
    }
}

/// HTTP response for one BPMN workflow execution action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowRunHttpResponse {
    /// Stable engine outcome emitted by the execution facade.
    pub outcome: BpmnAdvanceOutcome,
    /// Whether the run resumed from a stored checkpoint.
    pub resumed_from_checkpoint: bool,
    /// Whether the driver saved a checkpoint after the run.
    pub checkpoint_saved: bool,
    /// Whether the driver deleted stored checkpoint state after a terminal run.
    pub checkpoint_deleted: bool,
    /// Checkpoint backend used by the action, if any.
    pub checkpoint_backend: Option<String>,
    /// Runtime snapshot after the action.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowRunHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_start_report(
        report: &QianjiBpmnWorkflowStartReport,
    ) -> Self {
        Self::from_execution_report(&report.execution, report.checkpoint_store.as_ref())
    }

    fn from_execution_report(
        execution: &QianjiBpmnExecutionReport,
        checkpoint_store: Option<&crate::bpmn::backend::QianjiBpmnCheckpointStore>,
    ) -> Self {
        Self {
            outcome: execution.outcome.clone(),
            resumed_from_checkpoint: execution.resumed_from_checkpoint,
            checkpoint_saved: execution.checkpoint_saved,
            checkpoint_deleted: execution.checkpoint_deleted,
            checkpoint_backend: checkpoint_store.map(|store| store.backend_name().to_string()),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(
                execution.session.instance(),
            ),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN workflow status load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowStatusHttpResponse {
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Runtime snapshot loaded from storage.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowStatusHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowStatusReport,
    ) -> Self {
        Self {
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN human-task claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskClaimHttpResponse {
    /// Whether the claim mutated checkpointed state.
    pub claimed: bool,
    /// Monotonic checkpoint sequence after claim processing.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Claimed pending host-work item.
    pub claimed_work: QianjiBpmnPendingHostWorkHttpResponse,
    /// Runtime snapshot loaded after claim processing.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowTaskClaimHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowTaskClaimReport,
    ) -> Self {
        Self {
            claimed: report.changed,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            claimed_work: QianjiBpmnPendingHostWorkHttpResponse::from_instance_pending_host_work(
                &report.instance,
                &report.claimed_work,
            ),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN human-task claim release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskReleaseHttpResponse {
    /// Whether the release mutated checkpointed state.
    pub released: bool,
    /// Monotonic checkpoint sequence after release processing.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Pending host-work item after release.
    pub released_work: QianjiBpmnPendingHostWorkHttpResponse,
    /// Runtime snapshot loaded after release processing.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowTaskReleaseHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowTaskReleaseReport,
    ) -> Self {
        Self {
            released: report.changed,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            released_work: QianjiBpmnPendingHostWorkHttpResponse::from_instance_pending_host_work(
                &report.instance,
                &report.released_work,
            ),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN workflow cancellation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowCancelHttpResponse {
    /// Whether a checkpoint was deleted.
    pub cancelled: bool,
    /// Monotonic checkpoint sequence loaded before deletion.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Runtime snapshot loaded before deletion.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowCancelHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowCancelReport,
    ) -> Self {
        Self {
            cancelled: true,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}
