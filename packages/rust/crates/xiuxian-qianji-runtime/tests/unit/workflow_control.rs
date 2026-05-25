use std::{error::Error, io, path::Path};

use async_trait::async_trait;
use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    BpmnHostBridge, BusinessRuleTaskOutcome, BusinessRuleTaskRequest, EventPollOutcome,
    EventPollRequest, HostBridgeError, ManualTaskOutcome, ManualTaskRequest, PendingHostWork,
    PendingHostWorkKind, ScriptTaskOutcome, ScriptTaskRequest, SendTaskOutcome, SendTaskRequest,
    ServiceTaskOutcome, ServiceTaskRequest, UserTaskOutcome, UserTaskRequest,
};
use xiuxian_qianji_runtime::{
    QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnInstanceId, QianjiRuntimeBpmnProcessId,
    QianjiRuntimeBpmnSourcePath, QianjiRuntimeBpmnTokenId, QianjiRuntimeContinueUntilHumanBoundary,
    QianjiRuntimeDmnSourcePaths, QianjiRuntimeWorkflowControlPort,
    QianjiRuntimeWorkflowResumeRequest, QianjiRuntimeWorkflowStatusRequest,
    QianjiRuntimeWorkflowStatusView, QianjiRuntimeWorkflowTaskCompleteRequest,
    QianjiRuntimeWorkflowTaskCompletionKind, QianjiRuntimeWorkflowTaskCompletionPayload,
};

#[tokio::test]
async fn workflow_control_port_round_trips_runtime_requests() -> Result<(), Box<dyn Error>> {
    let port = FakePort;
    let host = FakeHost;

    let status = port
        .load_workflow_status_view(QianjiRuntimeWorkflowStatusRequest {
            instance_id: QianjiRuntimeBpmnInstanceId::new("workflow-1"),
            checkpoint_backend: FakeCheckpointBackend::new("status"),
        })
        .await?;

    assert_eq!(status.pending_host_work.len(), 1);
    assert_eq!(
        status.pending_host_work[0].kind,
        PendingHostWorkKind::Service
    );

    let prepared = port
        .prepare_resume_workflow(QianjiRuntimeWorkflowResumeRequest {
            bpmn_source: QianjiRuntimeBpmnSourcePath::new("flows/agent-coding.bpmn"),
            dmn_sources: QianjiRuntimeDmnSourcePaths::empty(),
            instance_id: QianjiRuntimeBpmnInstanceId::new("workflow-1"),
            checkpoint_backend: FakeCheckpointBackend::new("resume"),
        })
        .await?;

    let report = port
        .complete_prepared_workflow_task_until_host_boundary(
            prepared,
            QianjiRuntimeWorkflowTaskCompleteRequest {
                bpmn_source: QianjiRuntimeBpmnSourcePath::new("flows/agent-coding.bpmn"),
                dmn_sources: QianjiRuntimeDmnSourcePaths::new(Vec::new()),
                instance_id: QianjiRuntimeBpmnInstanceId::new("workflow-1"),
                checkpoint_backend: FakeCheckpointBackend::new("complete"),
                completion: QianjiRuntimeWorkflowTaskCompletionPayload {
                    token_id: QianjiRuntimeBpmnTokenId::new(7),
                    process_id: QianjiRuntimeBpmnProcessId::new("agent_coding"),
                    activity_id: QianjiRuntimeBpmnActivityId::new("resolve_project"),
                    kind: QianjiRuntimeWorkflowTaskCompletionKind::Service,
                    data: json!({"projectResolved": true}),
                    claimant: None,
                },
                continue_until_human_boundary: QianjiRuntimeContinueUntilHumanBoundary::new(false),
            },
            &host,
        )
        .await?;

    assert_eq!(report.completed_token_id, 7);
    Ok(())
}

struct FakePort;

#[derive(Clone)]
struct FakeCheckpointBackend {
    name: &'static str,
}

impl FakeCheckpointBackend {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

struct FakePrepared {
    instance_id: String,
}

#[derive(Clone)]
struct FakeReport {
    completed_token_id: u64,
}

struct FakeHost;

#[async_trait]
impl BpmnHostBridge for FakeHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> Result<SendTaskOutcome, HostBridgeError> {
        Err(unsupported("send"))
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> Result<ServiceTaskOutcome, HostBridgeError> {
        Err(unsupported("service"))
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> Result<ScriptTaskOutcome, HostBridgeError> {
        Err(unsupported("script"))
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> Result<UserTaskOutcome, HostBridgeError> {
        Err(unsupported("user"))
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> Result<ManualTaskOutcome, HostBridgeError> {
        Err(unsupported("manual"))
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> Result<BusinessRuleTaskOutcome, HostBridgeError> {
        Err(unsupported("business_rule"))
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> Result<EventPollOutcome, HostBridgeError> {
        Err(unsupported("event_poll"))
    }

    fn now_unix_ms(&self) -> u64 {
        42
    }
}

fn unsupported(operation: &'static str) -> HostBridgeError {
    HostBridgeError::UnsupportedOperation { operation }
}

#[async_trait]
impl QianjiRuntimeWorkflowControlPort<FakeHost> for FakePort {
    type CheckpointBackend = FakeCheckpointBackend;
    type PreparedResume = FakePrepared;
    type TaskCompleteReport = FakeReport;
    type Error = io::Error;

    async fn load_workflow_status_view(
        &self,
        request: QianjiRuntimeWorkflowStatusRequest<Self::CheckpointBackend>,
    ) -> Result<QianjiRuntimeWorkflowStatusView, Self::Error> {
        assert_eq!(request.instance_id.as_str(), "workflow-1");
        assert_eq!(request.checkpoint_backend.name, "status");
        Ok(QianjiRuntimeWorkflowStatusView::new(vec![service_work()]))
    }

    async fn prepare_resume_workflow(
        &self,
        request: QianjiRuntimeWorkflowResumeRequest<Self::CheckpointBackend>,
    ) -> Result<Self::PreparedResume, Self::Error> {
        assert_eq!(
            request.bpmn_source.as_path(),
            Path::new("flows/agent-coding.bpmn")
        );
        assert!(request.dmn_sources.as_slice().is_empty());
        assert_eq!(request.checkpoint_backend.name, "resume");
        Ok(FakePrepared {
            instance_id: request.instance_id.into_string(),
        })
    }

    async fn complete_prepared_workflow_task_until_host_boundary(
        &self,
        prepared: Self::PreparedResume,
        request: QianjiRuntimeWorkflowTaskCompleteRequest<Self::CheckpointBackend>,
        _host: &FakeHost,
    ) -> Result<Self::TaskCompleteReport, Self::Error> {
        assert_eq!(prepared.instance_id, "workflow-1");
        assert_eq!(request.checkpoint_backend.name, "complete");
        assert_eq!(request.completion.token_id.as_u64(), 7);
        assert_eq!(
            request.completion.kind,
            QianjiRuntimeWorkflowTaskCompletionKind::Service
        );
        assert!(!request.continue_until_human_boundary.as_bool());
        Ok(FakeReport {
            completed_token_id: request.completion.token_id.as_u64(),
        })
    }
}

fn service_work() -> PendingHostWork {
    PendingHostWork {
        token_id: 7,
        process_id: Some("agent_coding".into()),
        node_index: 11,
        activity_id: Some("resolve_project".into()),
        kind: PendingHostWorkKind::Service,
        decision: None,
        lane: None,
        script_format: None,
        script_body: None,
        human_task_form: None,
        human_task_assignment: None,
        task_io: None,
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: Some("work.resolve_project.7".into()),
    }
}
