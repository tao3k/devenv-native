use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::{Value, json};
use xiuxian_qianji_bpmn_engine::{
    BpmnTaskIoSpec, BpmnTaskOutputBinding, BusinessRuleTaskOutcome, BusinessRuleTaskRequest,
    EventPollOutcome, EventPollRequest, HostBridgeError, ManualTaskOutcome, ManualTaskRequest,
    PendingHostWork, PendingHostWorkKind, ScriptTaskOutcome, ScriptTaskRequest, SendTaskOutcome,
    SendTaskRequest, ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome, UserTaskRequest,
};
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, InMemoryControlLedger, InMemoryHotStateStore, RunId,
};
use xiuxian_qianji_runtime::{
    FLOWHUB_SERVICE_WORKER_RUN_SCHEMA, FlowhubServiceWorkerLoopRequest,
    QianjiRuntimeWorkflowControlPort, QianjiRuntimeWorkflowResumeRequest,
    QianjiRuntimeWorkflowStatusRequest, QianjiRuntimeWorkflowStatusView,
    QianjiRuntimeWorkflowTaskCompleteRequest, run_flowhub_service_worker_completion_loop,
};

#[tokio::test(flavor = "current_thread")]
async fn flowhub_worker_loop_completes_service_work_through_runtime_port()
-> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding-runtime-loop")?;
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let port = FakeWorkflowControlPort::new(vec![
        flowhub_service_work(7, "resolve_project", "projectResolved"),
        flowhub_service_work(8, "validate_contract", "contractValidated"),
    ]);
    let host = NoopHost;

    let output = run_flowhub_service_worker_completion_loop(
        &port,
        &host,
        &ledger,
        &hot_state,
        &FlowhubServiceWorkerLoopRequest {
            run_id: &run_id,
            scenario_id: "agent-coding",
            instance_id: "flowhub_agent_coding_runtime_loop",
            bpmn_source: Path::new("qianji-flowhub/plan/agent-coding.bpmn"),
            worker_id: "flowhub-service-worker",
            checkpoint_backend: FakeCheckpointBackend::Memory,
            now_ms: 42,
            lease_ttl_ms: 1_000,
            settled_at_ms: 84,
            max_steps: 8,
        },
    )
    .await?;

    let completed: Vec<_> = output
        .completed_steps
        .iter()
        .map(|step| step.activity_id.as_str())
        .collect();
    assert_eq!(completed, vec!["resolve_project", "validate_contract"]);
    assert!(output.completed_steps.iter().all(|step| step.released));
    assert_eq!(output.final_pending_host_work_count, 0);
    assert_eq!(
        output
            .final_report
            .as_ref()
            .map(|report| report.activity_id.as_str()),
        Some("validate_contract")
    );
    assert_flowhub_worker_run_was_admitted(&ledger, &run_id)?;
    assert_eq!(
        port.completed_activity_ids()?,
        vec!["resolve_project", "validate_contract"]
    );
    assert_workflow_requests_preserve_loop_boundary(&port)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FakeCheckpointBackend {
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakePreparedResume;

#[derive(Debug, Clone, PartialEq)]
struct FakeTaskCompleteReport {
    activity_id: String,
    data: Value,
}

#[derive(Debug)]
struct FakeWorkflowControlPort {
    pending: Mutex<Vec<PendingHostWork>>,
    completed: Mutex<Vec<String>>,
    status_requests: Mutex<Vec<QianjiRuntimeWorkflowStatusRequest<FakeCheckpointBackend>>>,
    resume_requests: Mutex<Vec<QianjiRuntimeWorkflowResumeRequest<FakeCheckpointBackend>>>,
    complete_requests: Mutex<Vec<QianjiRuntimeWorkflowTaskCompleteRequest<FakeCheckpointBackend>>>,
}

impl FakeWorkflowControlPort {
    fn new(pending: Vec<PendingHostWork>) -> Self {
        Self {
            pending: Mutex::new(pending),
            completed: Mutex::new(Vec::new()),
            status_requests: Mutex::new(Vec::new()),
            resume_requests: Mutex::new(Vec::new()),
            complete_requests: Mutex::new(Vec::new()),
        }
    }

    fn completed_activity_ids(&self) -> Result<Vec<String>, FakeWorkflowError> {
        Ok(self
            .completed
            .lock()
            .map_err(|_| FakeWorkflowError("completed mutex poisoned".to_owned()))?
            .clone())
    }

    fn status_requests(
        &self,
    ) -> Result<Vec<QianjiRuntimeWorkflowStatusRequest<FakeCheckpointBackend>>, FakeWorkflowError>
    {
        Ok(self
            .status_requests
            .lock()
            .map_err(|_| FakeWorkflowError("status request mutex poisoned".to_owned()))?
            .clone())
    }

    fn resume_requests(
        &self,
    ) -> Result<Vec<QianjiRuntimeWorkflowResumeRequest<FakeCheckpointBackend>>, FakeWorkflowError>
    {
        Ok(self
            .resume_requests
            .lock()
            .map_err(|_| FakeWorkflowError("resume request mutex poisoned".to_owned()))?
            .clone())
    }

    fn complete_requests(
        &self,
    ) -> Result<
        Vec<QianjiRuntimeWorkflowTaskCompleteRequest<FakeCheckpointBackend>>,
        FakeWorkflowError,
    > {
        Ok(self
            .complete_requests
            .lock()
            .map_err(|_| FakeWorkflowError("complete request mutex poisoned".to_owned()))?
            .clone())
    }
}

#[async_trait]
impl QianjiRuntimeWorkflowControlPort<NoopHost> for FakeWorkflowControlPort {
    type CheckpointBackend = FakeCheckpointBackend;
    type PreparedResume = FakePreparedResume;
    type TaskCompleteReport = FakeTaskCompleteReport;
    type Error = FakeWorkflowError;

    async fn load_workflow_status_view(
        &self,
        request: QianjiRuntimeWorkflowStatusRequest<Self::CheckpointBackend>,
    ) -> Result<QianjiRuntimeWorkflowStatusView, Self::Error> {
        self.status_requests
            .lock()
            .map_err(|_| FakeWorkflowError("status request mutex poisoned".to_owned()))?
            .push(request);
        Ok(QianjiRuntimeWorkflowStatusView::new(
            self.pending
                .lock()
                .map_err(|_| FakeWorkflowError("pending mutex poisoned".to_owned()))?
                .clone(),
        ))
    }

    async fn prepare_resume_workflow(
        &self,
        request: QianjiRuntimeWorkflowResumeRequest<Self::CheckpointBackend>,
    ) -> Result<Self::PreparedResume, Self::Error> {
        self.resume_requests
            .lock()
            .map_err(|_| FakeWorkflowError("resume request mutex poisoned".to_owned()))?
            .push(request);
        Ok(FakePreparedResume)
    }

    async fn complete_prepared_workflow_task_until_host_boundary(
        &self,
        _prepared: Self::PreparedResume,
        request: QianjiRuntimeWorkflowTaskCompleteRequest<Self::CheckpointBackend>,
        _host: &NoopHost,
    ) -> Result<Self::TaskCompleteReport, Self::Error> {
        self.complete_requests
            .lock()
            .map_err(|_| FakeWorkflowError("complete request mutex poisoned".to_owned()))?
            .push(request.clone());
        let activity_id = request.completion.activity_id.as_str().to_owned();
        let data = request.completion.data;
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| FakeWorkflowError("pending mutex poisoned".to_owned()))?;
        let index = pending
            .iter()
            .position(|work| {
                work.activity_id
                    .as_ref()
                    .is_some_and(|pending_id| pending_id.as_str() == activity_id)
            })
            .ok_or_else(|| FakeWorkflowError(format!("missing pending work `{activity_id}`")))?;
        pending.remove(index);
        drop(pending);
        self.completed
            .lock()
            .map_err(|_| FakeWorkflowError("completed mutex poisoned".to_owned()))?
            .push(activity_id.clone());
        Ok(FakeTaskCompleteReport { activity_id, data })
    }
}

fn assert_workflow_requests_preserve_loop_boundary(
    port: &FakeWorkflowControlPort,
) -> Result<(), Box<dyn Error>> {
    let status_requests = port.status_requests()?;
    assert_eq!(status_requests.len(), 4);
    assert!(status_requests.iter().all(|request| {
        request.instance_id.as_str() == "flowhub_agent_coding_runtime_loop"
            && request.checkpoint_backend == FakeCheckpointBackend::Memory
    }));

    let resume_requests = port.resume_requests()?;
    assert_eq!(resume_requests.len(), 2);
    assert!(resume_requests.iter().all(|request| {
        request.bpmn_source.as_path() == Path::new("qianji-flowhub/plan/agent-coding.bpmn")
            && request.dmn_sources.as_slice().is_empty()
            && request.instance_id.as_str() == "flowhub_agent_coding_runtime_loop"
            && request.checkpoint_backend == FakeCheckpointBackend::Memory
    }));

    let complete_requests = port.complete_requests()?;
    assert_eq!(complete_requests.len(), 2);
    let activity_ids = complete_requests
        .iter()
        .map(|request| request.completion.activity_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(activity_ids, vec!["resolve_project", "validate_contract"]);
    assert!(complete_requests.iter().all(|request| {
        request.bpmn_source.as_path() == Path::new("qianji-flowhub/plan/agent-coding.bpmn")
            && request.dmn_sources.as_slice().is_empty()
            && request.instance_id.as_str() == "flowhub_agent_coding_runtime_loop"
            && request.checkpoint_backend == FakeCheckpointBackend::Memory
            && !request.continue_until_human_boundary.as_bool()
    }));
    assert_eq!(
        complete_requests[0].completion.data,
        json!({"projectResolved": true})
    );
    assert_eq!(
        complete_requests[1].completion.data,
        json!({"contractValidated": true})
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct FakeWorkflowError(String);

impl fmt::Display for FakeWorkflowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for FakeWorkflowError {}

struct NoopHost;

#[async_trait]
impl xiuxian_qianji_bpmn_engine::BpmnHostBridge for NoopHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> Result<SendTaskOutcome, HostBridgeError> {
        Err(unsupported("dispatch_send_task"))
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> Result<ServiceTaskOutcome, HostBridgeError> {
        Err(unsupported("dispatch_service_task"))
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> Result<ScriptTaskOutcome, HostBridgeError> {
        Err(unsupported("dispatch_script_task"))
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> Result<UserTaskOutcome, HostBridgeError> {
        Err(unsupported("dispatch_user_task"))
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> Result<ManualTaskOutcome, HostBridgeError> {
        Err(unsupported("dispatch_manual_task"))
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> Result<BusinessRuleTaskOutcome, HostBridgeError> {
        Err(unsupported("dispatch_business_rule_task"))
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> Result<EventPollOutcome, HostBridgeError> {
        Err(unsupported("poll_external_event"))
    }

    fn now_unix_ms(&self) -> u64 {
        0
    }
}

fn unsupported(operation: &'static str) -> HostBridgeError {
    HostBridgeError::UnsupportedOperation { operation }
}

fn assert_flowhub_worker_run_was_admitted(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
) -> Result<(), Box<dyn Error>> {
    let records = ledger.load_events(run_id)?;
    let Some(first) = records.first() else {
        return Err("Flowhub worker loop should create its control run".into());
    };
    let ControlEventKind::RunCreated {
        intent, metadata, ..
    } = &first.event.kind
    else {
        return Err("first Flowhub worker-loop event should create the run".into());
    };
    assert_eq!(
        intent,
        "Flowhub service worker for scenario agent-coding instance flowhub_agent_coding_runtime_loop"
    );
    assert_eq!(metadata["schema"], FLOWHUB_SERVICE_WORKER_RUN_SCHEMA);
    assert_eq!(metadata["scenarioId"], "agent-coding");
    assert_eq!(metadata["instanceId"], "flowhub_agent_coding_runtime_loop");
    Ok(())
}

fn flowhub_service_work(token_id: u64, activity_id: &str, output_name: &str) -> PendingHostWork {
    PendingHostWork {
        token_id,
        process_id: Some("agent_coding".into()),
        node_index: u32::try_from(token_id).unwrap_or(u32::MAX),
        activity_id: Some(activity_id.into()),
        kind: PendingHostWorkKind::Service,
        decision: None,
        lane: None,
        script_format: None,
        script_body: None,
        human_task_form: None,
        human_task_assignment: None,
        task_io: Some(BpmnTaskIoSpec {
            inputs: Vec::new(),
            outputs: vec![BpmnTaskOutputBinding {
                name: output_name.into(),
                target_ref: format!("flowhub.{output_name}").into(),
                required: true,
            }],
        }),
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: Some(format!("work.{activity_id}.{token_id}").into()),
    }
}
