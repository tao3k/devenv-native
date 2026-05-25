//! Runtime workflow-control port implementation for the qianji BPMN service.

use async_trait::async_trait;
use xiuxian_qianji_bpmn_engine::BpmnHostBridge;
use xiuxian_qianji_runtime::{
    QianjiRuntimeWorkflowControlPort, QianjiRuntimeWorkflowResumeRequest,
    QianjiRuntimeWorkflowStatusRequest, QianjiRuntimeWorkflowStatusView,
    QianjiRuntimeWorkflowTaskCompleteRequest, QianjiRuntimeWorkflowTaskCompletionKind,
    QianjiRuntimeWorkflowTaskCompletionPayload,
};

use crate::bpmn::identity::QianjiBpmnWorkflowInstanceId;

use super::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
};

#[async_trait]
impl<H> QianjiRuntimeWorkflowControlPort<H> for QianjiBpmnWorkflowControlService
where
    H: BpmnHostBridge + Send + Sync,
{
    type CheckpointBackend = QianjiBpmnWorkflowCheckpointBackend;
    type PreparedResume = QianjiBpmnPreparedWorkflowResume;
    type TaskCompleteReport = QianjiBpmnWorkflowTaskCompleteReport;
    type Error = QianjiBpmnWorkflowControlError;

    async fn load_workflow_status_view(
        &self,
        request: QianjiRuntimeWorkflowStatusRequest<Self::CheckpointBackend>,
    ) -> Result<QianjiRuntimeWorkflowStatusView, Self::Error> {
        let report = self
            .load_workflow_status(&QianjiBpmnWorkflowStatusRequest {
                instance_id: QianjiBpmnWorkflowInstanceId::from(request.instance_id.into_string()),
                checkpoint_backend: request.checkpoint_backend,
            })
            .await?;
        Ok(QianjiRuntimeWorkflowStatusView::new(
            report.instance.pending_host_work,
        ))
    }

    async fn prepare_resume_workflow(
        &self,
        request: QianjiRuntimeWorkflowResumeRequest<Self::CheckpointBackend>,
    ) -> Result<Self::PreparedResume, Self::Error> {
        self.prepare_resume_workflow(&QianjiBpmnWorkflowResumeRequest {
            bpmn_path: request.bpmn_source.into_path_buf(),
            dmn_paths: request.dmn_sources.into_vec(),
            instance_id: QianjiBpmnWorkflowInstanceId::from(request.instance_id.into_string()),
            checkpoint_backend: request.checkpoint_backend,
        })
        .await
    }

    async fn complete_prepared_workflow_task_until_host_boundary(
        &self,
        prepared: Self::PreparedResume,
        request: QianjiRuntimeWorkflowTaskCompleteRequest<Self::CheckpointBackend>,
        host: &H,
    ) -> Result<Self::TaskCompleteReport, Self::Error> {
        self.complete_prepared_workflow_task_until_host_boundary(
            prepared,
            &QianjiBpmnWorkflowTaskCompleteRequest {
                bpmn_path: request.bpmn_source.into_path_buf(),
                dmn_paths: request.dmn_sources.into_vec(),
                instance_id: QianjiBpmnWorkflowInstanceId::from(request.instance_id.into_string()),
                checkpoint_backend: request.checkpoint_backend,
                completion: runtime_completion_payload(request.completion),
                continue_until_human_boundary: request.continue_until_human_boundary.as_bool(),
            },
            host,
        )
        .await
    }
}

fn runtime_completion_payload(
    completion: QianjiRuntimeWorkflowTaskCompletionPayload,
) -> QianjiBpmnWorkflowTaskCompletionPayload {
    QianjiBpmnWorkflowTaskCompletionPayload {
        token_id: completion.token_id.as_u64(),
        process_id: completion.process_id.into_string().into(),
        activity_id: completion.activity_id.into_string().into(),
        kind: runtime_completion_kind(completion.kind),
        data: completion.data,
        claimant: completion.claimant,
    }
}

fn runtime_completion_kind(
    kind: QianjiRuntimeWorkflowTaskCompletionKind,
) -> QianjiBpmnWorkflowTaskCompletionKind {
    match kind {
        QianjiRuntimeWorkflowTaskCompletionKind::Send => QianjiBpmnWorkflowTaskCompletionKind::Send,
        QianjiRuntimeWorkflowTaskCompletionKind::Service => {
            QianjiBpmnWorkflowTaskCompletionKind::Service
        }
        QianjiRuntimeWorkflowTaskCompletionKind::Script => {
            QianjiBpmnWorkflowTaskCompletionKind::Script
        }
        QianjiRuntimeWorkflowTaskCompletionKind::User => QianjiBpmnWorkflowTaskCompletionKind::User,
        QianjiRuntimeWorkflowTaskCompletionKind::Manual => {
            QianjiBpmnWorkflowTaskCompletionKind::Manual
        }
    }
}
