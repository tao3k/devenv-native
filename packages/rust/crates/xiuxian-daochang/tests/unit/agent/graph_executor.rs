use std::sync::Arc;

use crate::unit::tool_runtime_mock::{
    MockCallToolReply, MockToolRuntimeConfig, permissive_tool_definition, reserve_local_addr,
    spawn_mock_tool_runtime, text_result,
};
use anyhow::Result;

use xiuxian_daochang::{
    AgentConfig, GraphExecutionPlan, GraphPlanStep, GraphPlanStepKind, GraphWorkflowMode,
    OmegaDecision, OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass,
    ToolServerEntry, WorkflowBridgeMode,
};

use super::{GraphPlanExecutionInput, GraphPlanExecutionOutcome};

async fn spawn_mock_bridge_server(
    addr: std::net::SocketAddr,
) -> (
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
) {
    let recorded_arguments = Arc::new(std::sync::Mutex::new(Vec::new()));
    let server = spawn_mock_tool_runtime(
        addr,
        MockToolRuntimeConfig::with_static_tools(
            vec![permissive_tool_definition(
                "bridge.flaky",
                "Rejects metadata-rich calls",
            )],
            crate::unit::tool_runtime_mock::call_handler({
                let recorded_arguments = Arc::clone(&recorded_arguments);
                move |request| {
                    let recorded_arguments = Arc::clone(&recorded_arguments);
                    async move {
                        let args_json = request
                            .arguments
                            .clone()
                            .map(serde_json::Value::Object)
                            .unwrap_or_else(|| serde_json::json!({}));
                        recorded_arguments
                            .lock()
                            .expect("recorded arguments lock poisoned")
                            .push(args_json);
                        if request
                            .arguments
                            .as_ref()
                            .and_then(|arguments| arguments.get("_omni"))
                            .is_some()
                        {
                            return MockCallToolReply::RpcError {
                                code: -32_603,
                                message: "metadata not accepted".to_string(),
                                data: None,
                            };
                        }
                        MockCallToolReply::Result(text_result("fallback-ok"))
                    }
                }
            }),
        ),
    )
    .await;
    (server, recorded_arguments)
}

fn base_config(tool_url: String) -> AgentConfig {
    AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        model: "test-model".to_string(),
        tool_servers: vec![ToolServerEntry {
            name: "mock".to_string(),
            url: Some(tool_url),
            command: None,
            args: None,
        }],
        tool_handshake_timeout_secs: 2,
        tool_connect_retries: 2,
        tool_connect_retry_backoff_ms: 50,
        tool_timeout_secs: 15,
        tool_list_cache_ttl_ms: 100,
        max_tool_rounds: 3,
        ..AgentConfig::default()
    }
}

fn build_decision(fallback_policy: OmegaFallbackPolicy) -> OmegaDecision {
    OmegaDecision {
        route: OmegaRoute::Graph,
        confidence: 0.9,
        risk_level: OmegaRiskLevel::Low,
        fallback_policy,
        tool_trust_class: OmegaToolTrustClass::Verification,
        reason: "unit-test".to_string(),
        policy_id: Some("omega.unit.graph_plan_executor.v1".to_string()),
        drift_tolerance: None,
        next_audit_turn: None,
    }
}

fn build_plan(fallback_policy: OmegaFallbackPolicy, fallback_action: &str) -> GraphExecutionPlan {
    GraphExecutionPlan {
        plan_id: format!("graph-plan:test:{fallback_action}"),
        plan_version: "v1".to_string(),
        route: OmegaRoute::Graph,
        workflow_mode: GraphWorkflowMode::Omega,
        tool_name: "bridge.flaky".to_string(),
        fallback_policy,
        steps: vec![
            GraphPlanStep {
                index: 1,
                id: "prepare_injection_context".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                description: "prepare snapshot".to_string(),
                tool_name: None,
                fallback_action: None,
            },
            GraphPlanStep {
                index: 2,
                id: "invoke_graph_tool".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                description: "invoke tool".to_string(),
                tool_name: Some("bridge.flaky".to_string()),
                fallback_action: None,
            },
            GraphPlanStep {
                index: 3,
                id: "evaluate_fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                description: "apply fallback".to_string(),
                tool_name: None,
                fallback_action: Some(fallback_action.to_string()),
            },
        ],
    }
}

#[tokio::test]
async fn execute_graph_shortcut_plan_uses_plan_route_to_react_even_when_policy_is_retry()
-> Result<()> {
    let addr = reserve_local_addr().await;
    let (server_handle, recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let tool_url = format!("http://{addr}/sse");

    let agent = xiuxian_daochang::Agent::from_config(base_config(tool_url)).await?;
    let decision = build_decision(OmegaFallbackPolicy::SwitchToGraph);
    let plan = build_plan(OmegaFallbackPolicy::SwitchToGraph, "route_to_react");

    let outcome = agent
        .execute_graph_shortcut_plan(
            "telegram:test:graph-executor-react",
            &decision,
            &plan,
            GraphPlanExecutionInput {
                workflow_mode: WorkflowBridgeMode::Omega,
                turn_id: 1771578576,
                shortcut_user_message: r#"omega bridge.flaky {\"task\":\"route\"}"#.to_string(),
                bridge_arguments_with_metadata: Some(serde_json::json!({
                    "task": "route",
                    "_omni": {"trace": "x"}
                })),
                bridge_arguments_without_metadata: Some(serde_json::json!({"task": "route"})),
                injection: None,
            },
        )
        .await
        .expect("plan should route to react instead of retrying bridge");

    match outcome {
        GraphPlanExecutionOutcome::RouteToReact {
            rewritten_user_message,
            tool_summary,
        } => {
            assert!(rewritten_user_message.contains("Execute this task with ReAct"));
            assert_eq!(tool_summary.attempted, 1);
            assert_eq!(tool_summary.failed, 1);
            assert_eq!(tool_summary.succeeded, 0);
        }
        other => panic!("expected RouteToReact, got {other:?}"),
    }

    let captured = recorded_arguments
        .lock()
        .expect("recorded arguments lock poisoned")
        .clone();
    assert_eq!(captured.len(), 1, "plan route_to_react must not retry tool");
    assert!(captured[0].get("_omni").is_some());

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn execute_graph_shortcut_plan_uses_plan_retry_even_when_policy_is_abort() -> Result<()> {
    let addr = reserve_local_addr().await;
    let (server_handle, recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let tool_url = format!("http://{addr}/sse");

    let agent = xiuxian_daochang::Agent::from_config(base_config(tool_url)).await?;
    let decision = build_decision(OmegaFallbackPolicy::Abort);
    let plan = build_plan(OmegaFallbackPolicy::Abort, "retry_bridge_without_metadata");

    let outcome = agent
        .execute_graph_shortcut_plan(
            "telegram:test:graph-executor-retry",
            &decision,
            &plan,
            GraphPlanExecutionInput {
                workflow_mode: WorkflowBridgeMode::Omega,
                turn_id: 1771578577,
                shortcut_user_message: r#"omega bridge.flaky {\"task\":\"retry\"}"#.to_string(),
                bridge_arguments_with_metadata: Some(serde_json::json!({
                    "task": "retry",
                    "_omni": {"trace": "x"}
                })),
                bridge_arguments_without_metadata: Some(serde_json::json!({"task": "retry"})),
                injection: None,
            },
        )
        .await
        .expect("plan retry should succeed on metadata-free second attempt");

    match outcome {
        GraphPlanExecutionOutcome::Completed {
            output,
            tool_summary,
        } => {
            assert_eq!(output, "fallback-ok");
            assert_eq!(tool_summary.attempted, 2);
            assert_eq!(tool_summary.failed, 1);
            assert_eq!(tool_summary.succeeded, 1);
        }
        other => panic!("expected Completed, got {other:?}"),
    }

    let captured = recorded_arguments
        .lock()
        .expect("recorded arguments lock poisoned")
        .clone();
    assert_eq!(
        captured.len(),
        2,
        "plan retry_bridge_without_metadata must execute two attempts"
    );
    assert!(captured[0].get("_omni").is_some());
    assert!(captured[1].get("_omni").is_none());

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[test]
fn ordered_steps_rejects_non_consecutive_indices() {
    let plan = GraphExecutionPlan {
        plan_id: "graph-plan:test:bad-order".to_string(),
        plan_version: "v1".to_string(),
        route: OmegaRoute::Graph,
        workflow_mode: GraphWorkflowMode::Graph,
        tool_name: "bridge.flaky".to_string(),
        fallback_policy: OmegaFallbackPolicy::Abort,
        steps: vec![
            GraphPlanStep {
                index: 1,
                id: "prepare".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                description: "x".to_string(),
                tool_name: None,
                fallback_action: None,
            },
            GraphPlanStep {
                index: 3,
                id: "invoke".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                description: "x".to_string(),
                tool_name: Some("bridge.flaky".to_string()),
                fallback_action: None,
            },
            GraphPlanStep {
                index: 4,
                id: "fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                description: "x".to_string(),
                tool_name: None,
                fallback_action: Some("abort".to_string()),
            },
        ],
    };

    let error = super::ordered_steps(&plan).expect_err("step index gap should fail validation");
    assert!(error.to_string().contains("step ordering is invalid"));
}

#[test]
fn ordered_steps_rejects_unsupported_fallback_action() {
    let plan = GraphExecutionPlan {
        plan_id: "graph-plan:test:bad-fallback".to_string(),
        plan_version: "v1".to_string(),
        route: OmegaRoute::Graph,
        workflow_mode: GraphWorkflowMode::Graph,
        tool_name: "bridge.flaky".to_string(),
        fallback_policy: OmegaFallbackPolicy::Abort,
        steps: vec![
            GraphPlanStep {
                index: 1,
                id: "prepare".to_string(),
                kind: GraphPlanStepKind::PrepareInjectionContext,
                description: "x".to_string(),
                tool_name: None,
                fallback_action: None,
            },
            GraphPlanStep {
                index: 2,
                id: "invoke".to_string(),
                kind: GraphPlanStepKind::InvokeGraphTool,
                description: "x".to_string(),
                tool_name: Some("bridge.flaky".to_string()),
                fallback_action: None,
            },
            GraphPlanStep {
                index: 3,
                id: "fallback".to_string(),
                kind: GraphPlanStepKind::EvaluateFallback,
                description: "x".to_string(),
                tool_name: None,
                fallback_action: Some("legacy_retry_mode".to_string()),
            },
        ],
    };

    let error = super::ordered_steps(&plan)
        .expect_err("unsupported fallback action should fail validation");
    assert!(error.to_string().contains("unsupported fallback_action"));
}
