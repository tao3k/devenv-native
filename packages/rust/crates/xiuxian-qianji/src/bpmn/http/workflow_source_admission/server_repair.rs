use crate::bpmn::http_transport::bpmn_source_admission::{
    admit_bpmn_source_request, sha256_digest,
};
use crate::bpmn::http_transport::control_projection::record_bpmn_control_projection;
use crate::bpmn::http_transport::error_api::QianjiBpmnWorkflowHttpError;
use crate::bpmn::http_transport::request_api::{
    QianjiControlBpmnSourceAdmissionHttpRequest, QianjiControlWorkflowSourceAdmissionHttpRequest,
};
use crate::bpmn::http_transport::response_api::QianjiControlWorkflowSourceRepairRunHttpResponse;
use crate::bpmn::http_transport::state::QianjiBpmnWorkflowHttpState;
use crate::bpmn::{
    QianjiBpmnActivityId, QianjiBpmnProcessId, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowInstanceId, QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_qianji_bpmn_engine::{
    BpmnHostBridge, BpmnSourceFile, PendingHostWork, PendingHostWorkKind, lint_bpmn_source,
};

const SERVER_REPAIR_COMPILER: &str = "qianji-server-skill-repair-compiler-v1";
const SERVER_REPAIR_FLOW: &str = "qianji.workflow_source_repair.v1";
const SERVER_REPAIR_ENGINE: &str = "qianji-bpmn-engine";
const SERVER_REPAIR_LINT_EVIDENCE: &str = "qianji-lint-diagnostics-v1";
const SERVER_REPAIR_LINT_JUDGE: &str = "qianji-llm-reasoned-lint-judge-v1";
const SERVER_REPAIR_OUTPUT_CONTRACT: &str = "qianji_workflow_source_repair_result";
const SERVER_REPAIR_PROCESS_ID: &str = "qianji_workflow_source_repair_v1";
const SERVER_REPAIR_BPMN: &str =
    include_str!("../../../../resources/workflows/workflow_source_repair_v1.bpmn");
const SOURCE_INTAKE_ACTIVITY_ID: &str = "source_intake";
const RUN_LINT_ACTIVITY_ID: &str = "run_qianji_lint";
const ADMIT_BPMN_SOURCE_ACTIVITY_ID: &str = "admit_bpmn_source";
const SERVER_REPAIR_DETERMINISTIC_STEP_LIMIT: usize = 8;

pub(super) struct ServerRepairCompilerRequest {
    compiler: &'static str,
    flow: &'static str,
    engine: &'static str,
    lint_evidence: &'static str,
    lint_judge: &'static str,
    source_media_type: String,
    workflow_name: String,
    workflow_description_present: bool,
    process_id: String,
    source_sha256: String,
    output_contract: &'static str,
}

impl ServerRepairCompilerRequest {
    pub(super) fn from_admission_request(
        request: &QianjiControlWorkflowSourceAdmissionHttpRequest,
        source_media_type: &str,
    ) -> Self {
        let source_sha256 = sha256_digest(request.source_text.as_bytes());
        Self {
            compiler: SERVER_REPAIR_COMPILER,
            flow: SERVER_REPAIR_FLOW,
            engine: SERVER_REPAIR_ENGINE,
            lint_evidence: SERVER_REPAIR_LINT_EVIDENCE,
            lint_judge: SERVER_REPAIR_LINT_JUDGE,
            source_media_type: source_media_type.to_owned(),
            workflow_name: request.workflow_name.trim().to_owned(),
            workflow_description_present: !request.workflow_description.trim().is_empty(),
            process_id: request.process_id.as_str().to_owned(),
            source_sha256,
            output_contract: SERVER_REPAIR_OUTPUT_CONTRACT,
        }
    }

    pub(super) fn unavailable_message(&self) -> String {
        format!(
            "server-owned Skill.md/pi-agent repair compiler `{}` is not enabled; BPMN repair flow `{}` must run on `{}` and return `{}` for process `{}` from `{}` source `{}` (workflow `{}`, description_present={}); the flow owns LLM draft/repair, deterministic lint evidence `{}`, LLM reasoning lint judge `{}`, retry, and final qianji-server BPMN admission",
            self.compiler,
            self.flow,
            self.engine,
            self.output_contract,
            self.process_id,
            self.source_media_type,
            self.source_sha256,
            self.workflow_name,
            self.workflow_description_present,
            self.lint_evidence,
            self.lint_judge,
        )
    }

    pub(super) fn compiler(&self) -> &'static str {
        self.compiler
    }

    pub(super) fn output_contract(&self) -> &'static str {
        self.output_contract
    }
}

pub(super) async fn start_server_repair_workflow<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    request: &QianjiControlWorkflowSourceAdmissionHttpRequest,
    repair_request: &ServerRepairCompilerRequest,
    authoring_media_type: &str,
    authoring_source_sha256: &str,
) -> Result<QianjiControlWorkflowSourceRepairRunHttpResponse, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    ensure_repair_runtime_ready(state, repair_request)?;
    let repair_bpmn_path = write_repair_bpmn_resource(state)?;
    let instance_id = repair_instance_id(&request.source_id, authoring_source_sha256);
    let process_id = QianjiBpmnProcessId::new(SERVER_REPAIR_PROCESS_ID);
    let start_request = QianjiBpmnWorkflowStartRequest {
        bpmn_path: repair_bpmn_path.clone(),
        dmn_paths: Vec::new(),
        process_id: process_id.clone(),
        instance_id: instance_id.clone(),
        initial_variables: Some(json!({
            "schema": "xiuxian_qianji.workflow_source_repair.input.v1",
            "sourceId": request.source_id,
            "targetProcessId": request.process_id.as_str(),
            "sourceMediaType": authoring_media_type,
            "sourceText": request.source_text,
            "workflowName": request.workflow_name,
            "workflowDescription": request.workflow_description,
            "authoringSourceSha256": authoring_source_sha256,
            "compiler": repair_request.compiler(),
            "flow": SERVER_REPAIR_FLOW,
            "lintEvidence": SERVER_REPAIR_LINT_EVIDENCE,
            "lintJudge": SERVER_REPAIR_LINT_JUDGE,
            "outputContract": repair_request.output_contract(),
        })),
        start_at_node_id: None,
        checkpoint_backend: Some(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
    };
    let prepared = state.service.prepare_start_workflow(&start_request)?;
    let report = state
        .service
        .start_prepared_workflow_until_host_boundary(prepared, &state.host, false, |_, _| {})
        .await?;
    let report = advance_server_owned_repair_tasks(state, report).await?;
    record_bpmn_control_projection(
        state,
        &report.execution.session,
        Some(report.resolved_bpmn_path.as_path()),
    )?;
    let pending_host_work_count = report.execution.session.instance().pending_host_work.len();
    Ok(QianjiControlWorkflowSourceRepairRunHttpResponse::new(
        format!("bpmn.workflow.{}", instance_id.as_str()),
        instance_id,
        process_id,
        repair_bpmn_path.display().to_string(),
        repair_request.output_contract(),
        pending_host_work_count,
    ))
}

pub(in crate::bpmn::http_transport) async fn advance_server_owned_repair_tasks<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    mut report: QianjiBpmnWorkflowStartReport,
) -> Result<QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    for _ in 0..SERVER_REPAIR_DETERMINISTIC_STEP_LIMIT {
        let Some(completion) = deterministic_repair_completion(state, &report)? else {
            return Ok(report);
        };
        let complete_request = QianjiBpmnWorkflowTaskCompleteRequest {
            bpmn_path: report.resolved_bpmn_path.clone(),
            dmn_paths: report.resolved_dmn_paths.clone(),
            instance_id: QianjiBpmnWorkflowInstanceId::new(
                report
                    .execution
                    .session
                    .instance()
                    .instance_id
                    .as_ref()
                    .to_owned(),
            ),
            checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
            completion,
            continue_until_human_boundary: false,
        };
        let resume_request = complete_request.workflow_resume_request();
        let prepared = state
            .service
            .prepare_resume_workflow(&resume_request)
            .await?;
        report = state
            .service
            .complete_prepared_workflow_task_until_host_boundary(
                prepared,
                &complete_request,
                &state.host,
            )
            .await?;
    }

    Err(QianjiBpmnWorkflowHttpError::internal_server_error(
        "server-owned workflow-source repair exceeded deterministic step limit",
    ))
}

pub(in crate::bpmn::http_transport) fn is_server_owned_repair_deterministic_work_id(
    process_id: Option<&str>,
    activity_id: Option<&str>,
) -> bool {
    process_id == Some(SERVER_REPAIR_PROCESS_ID)
        && matches!(
            activity_id,
            Some(SOURCE_INTAKE_ACTIVITY_ID | RUN_LINT_ACTIVITY_ID | ADMIT_BPMN_SOURCE_ACTIVITY_ID)
        )
}

fn deterministic_repair_completion<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    report: &QianjiBpmnWorkflowStartReport,
) -> Result<Option<QianjiBpmnWorkflowTaskCompletionPayload>, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let instance = report.execution.session.instance();
    if instance.process.process_id.as_ref() != SERVER_REPAIR_PROCESS_ID {
        return Ok(None);
    }
    let Some(pending) = instance
        .pending_host_work
        .iter()
        .find(|work| deterministic_repair_pending_work(work))
    else {
        return Ok(None);
    };
    let activity_id = pending_activity_id(pending)?;
    let data = match activity_id.as_str() {
        SOURCE_INTAKE_ACTIVITY_ID => json!({
            "sourceIntakeAccepted": true,
        }),
        RUN_LINT_ACTIVITY_ID => json!({
            "lintDiagnostics": lint_candidate_bpmn(instance.variables.get("candidateBpmn")),
        }),
        ADMIT_BPMN_SOURCE_ACTIVITY_ID => json!({
            "admittedBpmnSourceRef": admit_candidate_bpmn_source(state, &instance.variables)?,
        }),
        _ => return Ok(None),
    };

    Ok(Some(QianjiBpmnWorkflowTaskCompletionPayload {
        token_id: pending.token_id,
        process_id: QianjiBpmnProcessId::new(SERVER_REPAIR_PROCESS_ID),
        activity_id: QianjiBpmnActivityId::new(activity_id),
        kind: QianjiBpmnWorkflowTaskCompletionKind::Service,
        data,
        claimant: None,
    }))
}

fn deterministic_repair_pending_work(work: &PendingHostWork) -> bool {
    work.kind == PendingHostWorkKind::Service
        && is_server_owned_repair_deterministic_work_id(
            work.process_id
                .as_ref()
                .map(|process_id| process_id.as_ref()),
            work.activity_id
                .as_ref()
                .map(|activity_id| activity_id.as_ref()),
        )
}

fn pending_activity_id(pending: &PendingHostWork) -> Result<String, QianjiBpmnWorkflowHttpError> {
    pending
        .activity_id
        .as_ref()
        .map(|activity_id| {
            let value: &str = activity_id.as_ref();
            value.to_owned()
        })
        .ok_or_else(|| {
            QianjiBpmnWorkflowHttpError::internal_server_error(
                "server-owned workflow-source repair pending host work is missing activity_id",
            )
        })
}

fn lint_candidate_bpmn(candidate_bpmn: Option<&serde_json::Value>) -> serde_json::Value {
    let Some(candidate_bpmn) = candidate_bpmn
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return json!({
            "schema": SERVER_REPAIR_LINT_EVIDENCE,
            "ok": false,
            "issueCount": 1,
            "issues": [{
                "code": "candidate_bpmn_missing",
                "severity": "blocking",
                "title": "Candidate BPMN missing",
                "summary": "The LLM draft or repair step did not emit candidateBpmn.",
            }],
        });
    };
    let source = BpmnSourceFile::new("candidateBpmn.bpmn", candidate_bpmn);
    let report = lint_bpmn_source(&source);
    json!({
        "schema": SERVER_REPAIR_LINT_EVIDENCE,
        "ok": report.ok,
        "issueCount": report.issues.len(),
        "report": report,
    })
}

fn admit_candidate_bpmn_source<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    variables: &serde_json::Value,
) -> Result<String, QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    let candidate_bpmn = required_string_variable(variables, "candidateBpmn")?;
    let source_id = required_string_variable(variables, "sourceId")?;
    let target_process_id = required_string_variable(variables, "targetProcessId")?;
    let admitted = admit_bpmn_source_request(
        state,
        QianjiControlBpmnSourceAdmissionHttpRequest {
            source_id: format!("{source_id}/server-repair"),
            process_id: QianjiBpmnProcessId::new(target_process_id),
            bpmn_xml: candidate_bpmn.to_owned(),
        },
    )?;
    Ok(admitted.source_ref)
}

fn required_string_variable<'a>(
    variables: &'a serde_json::Value,
    name: &str,
) -> Result<&'a str, QianjiBpmnWorkflowHttpError> {
    variables
        .get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            QianjiBpmnWorkflowHttpError::bad_request(
                "workflow_source_repair_missing_variable",
                format!("workflow-source repair requires `{name}` before BPMN admission"),
            )
        })
}

fn ensure_repair_runtime_ready<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    repair_request: &ServerRepairCompilerRequest,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    if state.activity_evidence_ledger.is_none() {
        return Err(QianjiBpmnWorkflowHttpError::service_unavailable(
            "workflow_source_repair_control_ledger_unavailable",
            format!(
                "server_repair requires qianji-server to run with a durable control ledger; {}",
                repair_request.unavailable_message()
            ),
        ));
    }
    if state.recovery_hot_state.is_none() {
        return Err(QianjiBpmnWorkflowHttpError::service_unavailable(
            "workflow_source_repair_hot_state_unavailable",
            format!(
                "server_repair requires qianji-server to run with durable worker hot state; {}",
                repair_request.unavailable_message()
            ),
        ));
    }
    Ok(())
}

fn write_repair_bpmn_resource<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
) -> Result<PathBuf, QianjiBpmnWorkflowHttpError> {
    let path = repair_bpmn_resource_path(state);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string())
        })?;
    }
    fs::write(&path, SERVER_REPAIR_BPMN.as_bytes())
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))?;
    Ok(path)
}

fn repair_bpmn_resource_path<H>(state: &QianjiBpmnWorkflowHttpState<H>) -> PathBuf {
    project_cache_home(state)
        .join("qianji/workflow-source-repair")
        .join("workflow_source_repair_v1.bpmn")
}

fn project_cache_home<H>(state: &QianjiBpmnWorkflowHttpState<H>) -> PathBuf {
    if let Some(path) = std::env::var_os("PRJ_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return path;
    }
    if let Some(project_root) = state
        .runtime_env
        .as_ref()
        .and_then(|runtime_env| runtime_env.prj_root.as_ref())
    {
        return project_root.join(".cache");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| Path::new(".").to_path_buf())
        .join(".cache")
}

fn repair_instance_id(
    source_id: &str,
    authoring_source_sha256: &str,
) -> QianjiBpmnWorkflowInstanceId {
    let suffix = authoring_source_sha256
        .strip_prefix("sha256:")
        .unwrap_or(authoring_source_sha256)
        .chars()
        .take(12)
        .collect::<String>();
    QianjiBpmnWorkflowInstanceId::new(format!(
        "workflow-source-repair-{}-{}",
        sanitize_id_fragment(source_id),
        suffix
    ))
}

fn sanitize_id_fragment(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "unknown".to_owned()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::ServerRepairCompilerRequest;
    use crate::bpmn::QianjiBpmnProcessId;
    use crate::bpmn::http_transport::request_api::{
        QianjiControlWorkflowSourceAdmissionHttpRequest, QianjiControlWorkflowSourceCompilerMode,
    };

    #[test]
    fn server_repair_request_pins_llm_repair_contract() {
        let request = QianjiControlWorkflowSourceAdmissionHttpRequest {
            source_id: "daily-report/freeform".to_owned(),
            process_id: QianjiBpmnProcessId::new("Process_daily_report"),
            source_media_type: "text/markdown".to_owned(),
            source_text: "# Daily Report\n\nWrite a report.".to_owned(),
            workflow_name: "Daily Report".to_owned(),
            workflow_description: "Compile daily facts.".to_owned(),
            compiler_mode: QianjiControlWorkflowSourceCompilerMode::ServerRepair,
        };

        let repair_request =
            ServerRepairCompilerRequest::from_admission_request(&request, "text/markdown");

        assert_eq!(
            repair_request.compiler,
            "qianji-server-skill-repair-compiler-v1"
        );
        assert_eq!(repair_request.flow, "qianji.workflow_source_repair.v1");
        assert_eq!(repair_request.engine, "qianji-bpmn-engine");
        assert_eq!(repair_request.lint_evidence, "qianji-lint-diagnostics-v1");
        assert_eq!(
            repair_request.lint_judge,
            "qianji-llm-reasoned-lint-judge-v1"
        );
        assert_eq!(
            repair_request.output_contract,
            "qianji_workflow_source_repair_result"
        );
        assert_eq!(repair_request.process_id, "Process_daily_report");
        assert_eq!(repair_request.workflow_name, "Daily Report");
        assert!(repair_request.workflow_description_present);
        assert_eq!(
            repair_request.source_sha256,
            "sha256:608ba212948404cccb98d2aeb38f491a1769e35cf566a5e411dbf01e70e2d595"
        );
    }

    #[test]
    fn server_repair_unavailable_message_requires_bpmn_repair_flow() {
        let request = QianjiControlWorkflowSourceAdmissionHttpRequest {
            source_id: "meeting/freeform".to_owned(),
            process_id: QianjiBpmnProcessId::new("Process_meeting"),
            source_media_type: "text/markdown".to_owned(),
            source_text: "# Meeting\n\nSummarize the meeting.".to_owned(),
            workflow_name: "Meeting".to_owned(),
            workflow_description: String::new(),
            compiler_mode: QianjiControlWorkflowSourceCompilerMode::ServerRepair,
        };

        let repair_request =
            ServerRepairCompilerRequest::from_admission_request(&request, "text/markdown");
        let message = repair_request.unavailable_message();

        assert!(message.contains("BPMN repair flow `qianji.workflow_source_repair.v1`"));
        assert!(message.contains("run on `qianji-bpmn-engine`"));
        assert!(message.contains("deterministic lint evidence `qianji-lint-diagnostics-v1`"));
        assert!(message.contains("LLM reasoning lint judge `qianji-llm-reasoned-lint-judge-v1`"));
        assert!(message.contains("final qianji-server BPMN admission"));
        assert!(!message.contains("prompt_schema"));
        assert!(!message.contains("prompt_sha256"));
    }
}
