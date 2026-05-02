//! Scheduler preflight tests for `$wendao://...` late binding.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;
use xiuxian_qianji::executors::ShellMechanism;
use xiuxian_qianji::{
    FlowInstruction, QianjiEngine, QianjiMechanism, QianjiOutput, QianjiScheduler,
};

#[derive(Debug, Default)]
struct EchoAssetMechanism;

#[async_trait]
impl QianjiMechanism for EchoAssetMechanism {
    async fn execute(&self, context: &Value) -> Result<QianjiOutput, String> {
        Ok(QianjiOutput {
            data: json!({
                "resolved": context.get("asset").cloned().unwrap_or(Value::Null)
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

#[derive(Debug, Default)]
struct ProduceAgendaMechanism;

#[async_trait]
impl QianjiMechanism for ProduceAgendaMechanism {
    async fn execute(&self, _context: &Value) -> Result<QianjiOutput, String> {
        Ok(QianjiOutput {
            data: json!({
                "agenda_steward_propose": {
                    "output": "structured agenda draft"
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

#[tokio::test]
async fn scheduler_preflight_resolves_wendao_placeholder_before_node_execution() {
    let mut engine = QianjiEngine::new();
    let _ = engine.add_mechanism("echo", Arc::new(EchoAssetMechanism));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "asset": "$wendao://skills/agenda-management/references/prompts/classifier.md"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler run should succeed: {error}"));

    let Some(resolved) = output.get("resolved").and_then(Value::as_str) else {
        panic!("expected resolved context payload in scheduler output");
    };
    assert!(
        resolved.contains("agenda-validation preflight classifier"),
        "preflight should resolve semantic URI before mechanism execution"
    );
}

#[tokio::test]
async fn scheduler_preflight_returns_error_when_wendao_placeholder_is_unresolvable() {
    let mut engine = QianjiEngine::new();
    let _ = engine.add_mechanism("echo", Arc::new(EchoAssetMechanism));
    let scheduler = QianjiScheduler::new(engine);

    let error = scheduler
        .run(json!({
            "asset": "$wendao://skills/agenda-management/references/prompts/does_not_exist.md"
        }))
        .await;
    let rendered = match error {
        Ok(output) => {
            panic!("scheduler run should fail on invalid semantic URI, got output: {output:?}")
        }
        Err(error) => error.to_string(),
    };
    assert!(
        rendered.contains("semantic resource URI"),
        "unexpected error payload: {rendered}"
    );
}

#[tokio::test]
async fn scheduler_preflight_resolves_context_path_placeholder_after_upstream_merge() {
    let mut engine = QianjiEngine::new();
    let producer = engine.add_mechanism("producer", Arc::new(ProduceAgendaMechanism));
    let consumer = engine.add_mechanism("consumer", Arc::new(EchoAssetMechanism));
    engine.add_link(producer, consumer, None, 1.0);
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "asset": "$agenda_steward_propose.output"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler run should succeed: {error}"));

    let Some(resolved) = output.get("resolved").and_then(Value::as_str) else {
        panic!("expected resolved context payload in scheduler output");
    };
    assert_eq!(resolved, "structured agenda draft");
}

#[tokio::test]
async fn scheduler_preflight_expands_dynamic_query_into_xml_lite_bundle() {
    let mut engine = QianjiEngine::new();
    let _ = engine.add_mechanism("echo", Arc::new(EchoAssetMechanism));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "asset": "$carryover:>=1"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler run should succeed: {error}"));

    let Some(resolved) = output.get("resolved").and_then(Value::as_str) else {
        panic!("expected resolved context payload in scheduler output");
    };
    assert!(
        resolved.contains("<wendao_query_result>"),
        "dynamic query should expand into XML-Lite result block"
    );
    assert!(
        resolved.contains("wendao://skills/agenda-management/references/rules.md"),
        "dynamic query should include canonical semantic URI hits"
    );
}

#[tokio::test]
async fn shell_mechanism_resolves_semantic_placeholder_in_command_field() {
    let mechanism = ShellMechanism {
        cmd: "$command_payload".to_string(),
        allow_fail: false,
        stop_on_empty_stdout: false,
        empty_reason: None,
        output_key: "stdout".to_string(),
    };

    let output = mechanism
        .execute(&json!({
            "command_payload": "echo semantic-cmd-ok"
        }))
        .await
        .unwrap_or_else(|error| panic!("shell mechanism should resolve semantic command: {error}"));

    assert_eq!(output.data["stdout"], "semantic-cmd-ok");
}
