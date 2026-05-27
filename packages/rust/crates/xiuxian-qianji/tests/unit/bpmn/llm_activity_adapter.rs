use std::error::Error;

use crate::bpmn::{
    BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA, BpmnHostWorkLlmActivityRouteInput,
    QianjiBpmnActivityId, QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnProcessId,
    build_bpmn_host_work_llm_activity_route,
};
use crate::runtime_config::QianjiRuntimeLlmConfig;
use crate::workflow_config::QianjiWorkflowLlmTaskConfig;
use xiuxian_qianji_bpmn_engine::PendingHostWorkKind;
use xiuxian_qianji_control::{ArtifactId, ArtifactKind, ArtifactRef};

#[test]
fn bpmn_llm_route_uses_workflow_task_config_and_runtime_endpoint() -> Result<(), Box<dyn Error>> {
    let work = pending_work();
    let workflow_config = workflow_task_config(
        r#"
schema = "qianji.workflow.llm_task.v1"

[llm]
provider = "openrouter"
model = "deepseek/test"

[task]
activity_type = "llm.plan"
task_queue = "llm.openrouter"
idempotency_key_prefix = "qianji:test"
max_tokens = 2048
temperature_millis = 100
timeout_ms = 60000

[task.retry]
max_attempts = 2
initial_interval_ms = 500
non_retryable_error_codes = ["SchemaInvalid"]
"#,
    )?;
    let runtime = runtime_llm();
    let prompt_ref = artifact_ref("prompt-1", "qianji.bpmn.prompt", "file:///tmp/prompt.md")?;

    let decision = build_bpmn_host_work_llm_activity_route(BpmnHostWorkLlmActivityRouteInput {
        instance_id: "instance-1",
        bpmn_source_ref: Some("file:///tmp/workflow.bpmn"),
        profile: "bpmn-host-work-llm",
        pending_work: &work,
        workflow_config: &workflow_config,
        runtime_llm: &runtime,
        prompt_ref: &prompt_ref,
        context_ref: None,
        response_schema_ref: None,
    })?;

    assert_eq!(decision.schema, BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA);
    assert_eq!(decision.profile, "bpmn-host-work-llm");
    assert_eq!(decision.process_id, "process-main");
    assert_eq!(decision.activity_id, "step-1");
    assert_eq!(decision.endpoint.provider, "openrouter");
    assert_eq!(decision.endpoint.model, "deepseek/test");
    assert_eq!(decision.endpoint.base_url, "http://runtime.local/v1");
    assert_eq!(decision.endpoint.api_key_env, "RUNTIME_API_KEY");
    assert_eq!(
        decision.llm_activity.task.activity_type.as_str(),
        "llm.plan"
    );
    assert_eq!(
        decision.llm_activity.task.task_queue.as_str(),
        "llm.openrouter"
    );
    assert_eq!(
        decision.llm_activity.task.input_ref,
        Some(prompt_ref.clone())
    );
    assert_eq!(
        decision.llm_activity.request.prompt_ref, prompt_ref,
        "LLM prompt claim-check must match task input_ref"
    );
    assert_eq!(
        decision.llm_activity.request.max_tokens,
        Some(2048),
        "workflow task route should own request token limit"
    );
    assert!(
        decision
            .llm_activity
            .task
            .idempotency_key
            .as_str()
            .starts_with("qianji:test:instance-1:process-main:step-1:7")
    );
    let metadata = &decision.llm_activity.task.metadata;
    assert_eq!(metadata["process_id"], "process-main");
    assert_eq!(metadata["activity_id"], "step-1");
    Ok(())
}

#[test]
fn bpmn_llm_route_rejects_missing_identity() -> Result<(), Box<dyn Error>> {
    let mut work = pending_work();
    work.activity_id = None;
    let prompt_ref = artifact_ref("prompt-1", "qianji.bpmn.prompt", "file:///tmp/prompt.md")?;
    let result = build_bpmn_host_work_llm_activity_route(BpmnHostWorkLlmActivityRouteInput {
        instance_id: "instance-1",
        bpmn_source_ref: None,
        profile: "bpmn-host-work-llm",
        pending_work: &work,
        workflow_config: &QianjiWorkflowLlmTaskConfig::default(),
        runtime_llm: &runtime_llm(),
        prompt_ref: &prompt_ref,
        context_ref: None,
        response_schema_ref: None,
    });
    let Err(error) = result else {
        return Err("missing activity id must fail".into());
    };

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains("activity_id"));
    Ok(())
}

fn pending_work() -> QianjiBpmnPendingHostWorkHttpResponse {
    QianjiBpmnPendingHostWorkHttpResponse {
        token_id: 7,
        process_id: Some(QianjiBpmnProcessId::from("process-main")),
        node_index: 1,
        node_id: Some("step-1".to_string()),
        activity_id: Some(QianjiBpmnActivityId::from("step-1")),
        kind: PendingHostWorkKind::Service,
        work_id: Some("work-1".to_string()),
        form: None,
        assignment: None,
        lane: None,
        task_io: None,
        claim: None,
        variables: serde_json::json!({"topic": "workflow"}),
        inputs: serde_json::json!({"brief": "write plan"}),
        output_bindings: Vec::new(),
        repeat: None,
    }
}

fn workflow_task_config(source: &str) -> Result<QianjiWorkflowLlmTaskConfig, toml::de::Error> {
    toml::from_str(source)
}

fn runtime_llm() -> QianjiRuntimeLlmConfig {
    QianjiRuntimeLlmConfig {
        model: "runtime-model".to_string(),
        base_url: "http://runtime.local/v1".to_string(),
        api_key_env: "RUNTIME_API_KEY".to_string(),
        wire_api: "chat_completions".to_string(),
        api_key: "secret-not-serialized".to_string(),
    }
}

fn artifact_ref(id: &str, kind: &str, uri: &str) -> Result<ArtifactRef, Box<dyn Error>> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(id)?,
        artifact_kind: ArtifactKind::new(kind)?,
        uri: uri.to_string(),
        content_digest: None,
        metadata: serde_json::Value::Null,
    })
}
