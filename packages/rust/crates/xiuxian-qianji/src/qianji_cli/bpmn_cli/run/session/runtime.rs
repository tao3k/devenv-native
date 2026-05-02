use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnPreparedWorkflowStart, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowResumeRequest, SchedulerAgentIdentity,
};
use crate::qianji_cli::bpmn_cli::host;
use crate::qianji_cli::bpmn_cli::render;
use crate::qianji_cli::bpmn_cli::run::execution::{
    build_bpmn_workflow_start_request, build_bpmn_workflow_task_complete_request,
};
use crate::qianji_cli::bpmn_cli::run::shared::workflow_control_service;
use crate::qianji_cli::bpmn_cli::types::{BpmnCliHostBridgeContext, BpmnHostSessionCliCommand};

use super::prepared::{run_prepared_session_start, run_prepared_session_task_complete};
use super::request::{BpmnHostSessionTaskCompleteRequest, build_task_complete_command};
use super::result::BpmnHostSessionStepResult;

pub(super) struct BpmnHostSessionRuntime {
    control_service: QianjiBpmnWorkflowControlService,
    prepared_source: QianjiBpmnPreparedWorkflowStart,
    host_context: BpmnCliHostBridgeContext,
    pub(super) start_result: BpmnHostSessionStepResult,
}

impl BpmnHostSessionRuntime {
    pub(super) async fn start(
        command: &BpmnHostSessionCliCommand,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler_identity = SchedulerAgentIdentity::from_env();
        let control_service = workflow_control_service(None, Some(&scheduler_identity));
        let start_request = build_bpmn_workflow_start_request(&command.start)?;
        let prepared = control_service.prepare_start_workflow(&start_request)?;
        let prepared_source = prepared.clone();
        let host_context = host::build_bpmn_cli_host_bridge(
            &prepared.package,
            command.start.process_id.as_str(),
            command.start.host_fixture_path.as_deref(),
            command.start.event_fixture_path.as_deref(),
        )?;
        let start_result =
            run_prepared_session_start(&command.start, &control_service, prepared, &host_context)
                .await?;

        Ok(Self {
            control_service,
            prepared_source,
            host_context,
            start_result,
        })
    }

    pub(super) async fn complete_task(
        &self,
        session_command: &BpmnHostSessionCliCommand,
        request: BpmnHostSessionTaskCompleteRequest,
    ) -> Result<BpmnHostSessionStepResult, Box<dyn std::error::Error>> {
        let task_command = build_task_complete_command(session_command, request)?;
        let task_request = build_bpmn_workflow_task_complete_request(&task_command)?;
        let resume_request = QianjiBpmnWorkflowResumeRequest {
            bpmn_path: task_command.bpmn_path.clone(),
            dmn_paths: task_command.dmn_paths.clone(),
            instance_id: task_command.instance_id.clone(),
            checkpoint_backend: task_command.checkpoint_backend.clone(),
        };
        match self
            .control_service
            .prepare_resume_workflow_from_prepared_start(&resume_request, &self.prepared_source)
            .await
        {
            Ok(prepared) => {
                run_prepared_session_task_complete(
                    &task_command,
                    &task_request,
                    &self.control_service,
                    prepared,
                    &self.host_context,
                )
                .await
            }
            Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
                Ok(BpmnHostSessionStepResult {
                    output: render::render_bpmn_task_complete_missing_output(&task_command),
                    summary: None,
                })
            }
            Err(error) => Err(error.into()),
        }
    }
}
