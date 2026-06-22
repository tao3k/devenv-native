//! Server-owned BPMN host-work admission into LLM activity tasks.
//!
//! This module keeps provider/model resolution inside qianji-server. HTTP
//! clients only start workflows; workflow-task TOML and runtime TOML decide
//! the admitted LLM activity contract.

use super::activity_evidence::now_unix_ms;
use super::error_api::QianjiBpmnWorkflowHttpError;
use super::llm_task_documentation::bpmn_task_documentation;
use super::response_api::{
    QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
};
use super::workflow_source_admission::is_server_owned_repair_deterministic_work_id;
use crate::bpmn::identity::{QianjiBpmnActivityId, QianjiBpmnProcessId};
use crate::bpmn::llm_activity_adapter::{
    BpmnHostWorkLlmActivityRouteInput, build_bpmn_host_work_llm_activity_route,
};
use crate::bpmn::session::QianjiBpmnSession;
use crate::runtime_config::{
    QianjiRuntimeEnv, QianjiRuntimeLlmConfig, resolve_qianji_runtime_llm_config,
    resolve_qianji_runtime_llm_config_with_env,
};
use crate::workflow_config::{
    DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE, QianjiWorkflowLlmTaskConfig,
    resolve_qianji_workflow_llm_task_config, resolve_qianji_workflow_llm_task_config_with_env,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_qianji_bpmn_engine::PendingHostWorkKind;
use xiuxian_qianji_control::{
    AdmittedLlmActivityScheduleRecord, ArtifactId, ArtifactKind, ArtifactRef, ControlEventKind,
    ControlLedger, LlmActivityAdmission, RunId, StepCreatedJournalRecord, StepId,
    record_admitted_llm_activity_schedule_idempotent, record_control_event_batch,
};

const PROMPT_ARTIFACT_SCHEMA: &str = "qianji.bpmn.host_work.llm_prompt.v1";
const DEFAULT_PROMPT_ARTIFACT_KIND: &str = "qianji.bpmn.host_work.prompt";

pub(super) fn record_bpmn_llm_host_work_schedules(
    ledger: Option<&dyn ControlLedger>,
    runtime_env: Option<&QianjiRuntimeEnv>,
    session: &QianjiBpmnSession,
    bpmn_source: Option<&Path>,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    let Some(ledger) = ledger else {
        return Ok(());
    };
    let pending_work = selectable_llm_host_work(session);
    if pending_work.is_empty() {
        return Ok(());
    }
    let run_id = bpmn_control_run_id(session)?;
    let runtime_llm = resolve_runtime_llm_config(runtime_env)?;
    let workflow_config = resolve_workflow_llm_task_config(runtime_env)?;
    let artifact_dir = prompt_artifact_dir(runtime_env)?;
    fs::create_dir_all(&artifact_dir).map_err(schedule_error)?;
    let source_ref = bpmn_source.map(|path| path.display().to_string());
    let now_ms = now_unix_ms();
    ensure_llm_host_work_steps(ledger, &run_id, &pending_work, now_ms)?;
    let mut scheduled_activity_ids = existing_step_activity_ids(ledger, &run_id)?;

    for (index, work) in pending_work.iter().enumerate() {
        let prompt_ref = write_prompt_artifact(
            &artifact_dir,
            &run_id,
            work,
            source_ref.as_deref(),
            &workflow_config,
        )?;
        let route = build_bpmn_host_work_llm_activity_route(BpmnHostWorkLlmActivityRouteInput {
            instance_id: session.instance().instance_id.as_ref(),
            bpmn_source_ref: source_ref.as_deref(),
            profile: DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE,
            pending_work: work,
            workflow_config: &workflow_config,
            runtime_llm: &runtime_llm,
            prompt_ref: &prompt_ref,
            context_ref: None,
            response_schema_ref: None,
        })
        .map_err(schedule_error)?;
        let admission =
            LlmActivityAdmission::from_activity(route.llm_activity).map_err(schedule_error)?;
        let step_id = StepId::new(route.activity_id.clone()).map_err(schedule_error)?;
        let activity_id = admission.activity_task().activity_id.as_str().to_owned();
        if !scheduled_activity_ids.insert((step_id.as_str().to_owned(), activity_id)) {
            continue;
        }
        let occurred_at_ms =
            now_ms.saturating_add(u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1));
        record_admitted_llm_activity_schedule_idempotent(
            ledger,
            AdmittedLlmActivityScheduleRecord::step(
                run_id.clone(),
                step_id,
                occurred_at_ms,
                admission,
            ),
        )
        .map_err(schedule_error)?;
    }
    Ok(())
}

fn existing_step_activity_ids(
    ledger: &dyn ControlLedger,
    run_id: &RunId,
) -> Result<BTreeSet<(String, String)>, QianjiBpmnWorkflowHttpError> {
    let view = ledger.load_run_view(run_id).map_err(schedule_error)?;
    Ok(view
        .steps
        .iter()
        .flat_map(|(step_id, step)| {
            step.activities
                .keys()
                .map(|activity_id| (step_id.as_str().to_owned(), activity_id.as_str().to_owned()))
                .collect::<Vec<_>>()
        })
        .collect())
}

fn ensure_llm_host_work_steps(
    ledger: &dyn ControlLedger,
    run_id: &RunId,
    pending_work: &[QianjiBpmnPendingHostWorkHttpResponse],
    now_ms: u64,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    let existing = ledger.load_events(run_id).map_err(schedule_error)?;
    let mut declared_steps = existing
        .iter()
        .filter_map(|record| {
            let step_id = record.event.step_id.as_ref()?;
            matches!(record.event.kind, ControlEventKind::StepCreated { .. })
                .then(|| step_id.as_str().to_owned())
        })
        .collect::<BTreeSet<_>>();
    let mut events = Vec::new();

    for (index, work) in pending_work.iter().enumerate() {
        let Some(activity_id) = work.activity_id.as_ref() else {
            continue;
        };
        if !declared_steps.insert(activity_id.as_str().to_owned()) {
            continue;
        }
        let occurred_at_ms = now_ms.saturating_add(u64::try_from(index).unwrap_or(u64::MAX));
        events.push(
            StepCreatedJournalRecord::new(
                run_id.clone(),
                StepId::new(activity_id.as_str()).map_err(schedule_error)?,
                llm_host_work_step_title(work),
                occurred_at_ms,
            )
            .into_event(),
        );
    }

    if events.is_empty() {
        return Ok(());
    }
    record_control_event_batch(ledger, events)
        .map(|_| ())
        .map_err(schedule_error)
}

fn llm_host_work_step_title(work: &QianjiBpmnPendingHostWorkHttpResponse) -> String {
    work.activity_id
        .as_ref()
        .map(QianjiBpmnActivityId::as_str)
        .or(work.node_id.as_deref())
        .unwrap_or("BPMN LLM host work")
        .to_owned()
}

fn selectable_llm_host_work(
    session: &QianjiBpmnSession,
) -> Vec<QianjiBpmnPendingHostWorkHttpResponse> {
    let snapshot = QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(session.instance());
    snapshot
        .pending_host_work
        .into_iter()
        .filter(is_llm_routable_host_work)
        .collect()
}

fn is_llm_routable_host_work(work: &QianjiBpmnPendingHostWorkHttpResponse) -> bool {
    if is_server_owned_repair_deterministic_work_id(
        work.process_id.as_ref().map(QianjiBpmnProcessId::as_str),
        work.activity_id.as_ref().map(QianjiBpmnActivityId::as_str),
    ) {
        return false;
    }
    matches!(
        work.kind,
        PendingHostWorkKind::Service | PendingHostWorkKind::Task | PendingHostWorkKind::Script
    )
}

fn resolve_runtime_llm_config(
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<QianjiRuntimeLlmConfig, QianjiBpmnWorkflowHttpError> {
    let resolved = match runtime_env {
        Some(runtime_env) => resolve_qianji_runtime_llm_config_with_env(runtime_env),
        None => resolve_qianji_runtime_llm_config(),
    };
    resolved.map_err(schedule_error)
}

fn resolve_workflow_llm_task_config(
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<QianjiWorkflowLlmTaskConfig, QianjiBpmnWorkflowHttpError> {
    let resolved = match runtime_env {
        Some(runtime_env) => resolve_qianji_workflow_llm_task_config_with_env(
            DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE,
            runtime_env,
        ),
        None => resolve_qianji_workflow_llm_task_config(),
    };
    resolved.map_err(schedule_error)
}

fn write_prompt_artifact(
    artifact_dir: &Path,
    run_id: &RunId,
    work: &QianjiBpmnPendingHostWorkHttpResponse,
    bpmn_source_ref: Option<&str>,
    workflow_config: &QianjiWorkflowLlmTaskConfig,
) -> Result<ArtifactRef, QianjiBpmnWorkflowHttpError> {
    let artifact_id = prompt_artifact_id(run_id, work);
    let prompt = prompt_text(work, bpmn_source_ref);
    let path = artifact_dir.join(format!("{artifact_id}.md"));
    fs::write(&path, prompt.as_bytes()).map_err(schedule_error)?;
    let digest = sha256_digest(prompt.as_bytes());
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(artifact_id).map_err(schedule_error)?,
        artifact_kind: ArtifactKind::new(
            workflow_config
                .task
                .prompt_artifact_kind
                .as_deref()
                .unwrap_or(DEFAULT_PROMPT_ARTIFACT_KIND),
        )
        .map_err(schedule_error)?,
        uri: format!("file://{}", path.display()),
        content_digest: Some(digest),
        metadata: json!({
            "schema": PROMPT_ARTIFACT_SCHEMA,
            "source": "qianji-server",
            "pendingWorkKind": format!("{:?}", work.kind),
            "activityId": work.activity_id.as_ref().map(QianjiBpmnActivityId::as_str),
            "tokenId": work.token_id,
        }),
    })
}

fn prompt_text(
    work: &QianjiBpmnPendingHostWorkHttpResponse,
    bpmn_source_ref: Option<&str>,
) -> String {
    let task_documentation = bpmn_task_documentation(
        bpmn_source_ref,
        work.activity_id.as_ref().map(QianjiBpmnActivityId::as_str),
    );
    let payload = json!({
        "schema": PROMPT_ARTIFACT_SCHEMA,
        "bpmnSourceRef": bpmn_source_ref,
        "processId": work.process_id.as_ref().map(QianjiBpmnProcessId::as_str),
        "activityId": work.activity_id.as_ref().map(QianjiBpmnActivityId::as_str),
        "nodeId": work.node_id,
        "tokenId": work.token_id,
        "workId": work.work_id,
        "kind": format!("{:?}", work.kind),
        "variables": work.variables,
        "inputs": work.inputs,
        "taskDocumentation": task_documentation,
        "outputBindings": work.output_bindings,
    });
    format!(
        "You are the Qianji server-owned LLM worker for one BPMN host-work item.\n\
         Follow the BPMN task identity exactly. Use only the supplied variables, inputs, and taskDocumentation.\n\
         If taskDocumentation is present, treat it as the executable task instruction for this BPMN node.\n\
         If outputBindings is non-empty, return JSON only, with no Markdown fences or prose, and include every declared output binding name.\n\
         If outputBindings is empty, return concise raw text suitable for an operator-facing workflow trace.\n\n\
         <qianji_bpmn_host_work>\n{}\n</qianji_bpmn_host_work>\n",
        pretty_json(&payload),
    )
}

fn prompt_artifact_dir(
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<PathBuf, QianjiBpmnWorkflowHttpError> {
    if let Some(path) = std::env::var_os("PRJ_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(path.join("qianji/llm-prompts"));
    }
    if let Some(project_root) = runtime_env.and_then(|runtime_env| runtime_env.prj_root.as_ref()) {
        return Ok(project_root.join(".cache/qianji/llm-prompts"));
    }
    Ok(std::env::current_dir()
        .map_err(schedule_error)?
        .join(".cache/qianji/llm-prompts"))
}

fn bpmn_control_run_id(session: &QianjiBpmnSession) -> Result<RunId, QianjiBpmnWorkflowHttpError> {
    RunId::new(format!(
        "bpmn.workflow.{}",
        session.instance().instance_id.as_ref()
    ))
    .map_err(schedule_error)
}

fn prompt_artifact_id(run_id: &RunId, work: &QianjiBpmnPendingHostWorkHttpResponse) -> String {
    format!(
        "qianji-bpmn-host-work-prompt-{}-{}-{}",
        sanitize_id_fragment(run_id.as_str()),
        sanitize_id_fragment(
            work.activity_id
                .as_ref()
                .map_or("unknown", QianjiBpmnActivityId::as_str)
        ),
        work.token_id,
    )
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

fn sha256_digest(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    format!("sha256:{digest:x}")
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn schedule_error(error: impl std::fmt::Display) -> QianjiBpmnWorkflowHttpError {
    QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string())
}
