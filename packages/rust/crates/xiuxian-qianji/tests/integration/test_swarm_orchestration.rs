//! Integration tests for inner-rust swarm orchestration.

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};
use xiuxian_qianji::engine::NodeExecutionAffinity;
use xiuxian_qianji::{
    FlowInstruction, NodeTransitionPhase, PulseEmitter, QianjiEngine, QianjiMechanism,
    QianjiOutput, SwarmAgentConfig, SwarmEngine, SwarmEvent, SwarmExecutionOptions,
};

struct StaticOutputMechanism {
    key: String,
    value: String,
}

#[async_trait]
impl QianjiMechanism for StaticOutputMechanism {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        Ok(QianjiOutput {
            data: json!({ self.key.clone(): self.value.clone() }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

struct FailingMechanism;

#[async_trait]
impl QianjiMechanism for FailingMechanism {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        Err("intentional failure for swarm cancellation test".to_string())
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

struct SlowMechanism;

#[async_trait]
impl QianjiMechanism for SlowMechanism {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(QianjiOutput {
            data: json!({ "slow": "done" }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

#[derive(Debug, Clone)]
struct CapturingEmitter {
    events: Arc<Mutex<Vec<SwarmEvent>>>,
}

impl CapturingEmitter {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl PulseEmitter for CapturingEmitter {
    async fn emit_pulse(&self, event: SwarmEvent) -> Result<(), String> {
        self.events.lock().await.push(event);
        Ok(())
    }
}

fn unique_session_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{prefix}_{millis}")
}

#[tokio::test]
async fn swarm_engine_executes_workers_concurrently_with_isolated_windows()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = QianjiEngine::new();
    let first = engine.add_mechanism(
        "Draft",
        std::sync::Arc::new(StaticOutputMechanism {
            key: "draft".to_string(),
            value: "ok".to_string(),
        }),
    );
    let second = engine.add_mechanism(
        "Audit",
        std::sync::Arc::new(StaticOutputMechanism {
            key: "audit".to_string(),
            value: "ok".to_string(),
        }),
    );
    engine.add_link(first, second, None, 1.0);

    let swarm = SwarmEngine::new(engine);
    let identities = vec![
        SwarmAgentConfig::new("student"),
        SwarmAgentConfig::new("steward"),
        SwarmAgentConfig::new("professor"),
    ];

    let report = swarm
        .execute_swarm(json!({}), identities, SwarmExecutionOptions::default())
        .await?;

    assert_eq!(report.workers.len(), 3);
    assert!(report.final_context["draft"] == "ok");
    assert!(report.final_context["audit"] == "ok");
    for worker in &report.workers {
        assert!(worker.success, "worker failed: {:?}", worker.error);
        assert!(worker.window_turns >= 2);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires live valkey server for distributed checkpoint synchronization"]
async fn swarm_engine_routes_role_owned_nodes_via_valkey() -> Result<(), Box<dyn std::error::Error>>
{
    let redis_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());

    let mut engine = QianjiEngine::new();
    let student_idx = engine.add_mechanism_with_affinity(
        "StudentNode",
        std::sync::Arc::new(StaticOutputMechanism {
            key: "student_done".to_string(),
            value: "yes".to_string(),
        }),
        None,
        NodeExecutionAffinity {
            agent_id: None,
            role_class: Some("student".to_string()),
        },
    );
    let steward_idx = engine.add_mechanism_with_affinity(
        "StewardNode",
        std::sync::Arc::new(StaticOutputMechanism {
            key: "steward_done".to_string(),
            value: "yes".to_string(),
        }),
        None,
        NodeExecutionAffinity {
            agent_id: None,
            role_class: Some("steward".to_string()),
        },
    );
    engine.add_link(student_idx, steward_idx, None, 1.0);

    let swarm = SwarmEngine::new(engine);
    let student = {
        let mut profile = SwarmAgentConfig::new("agent_student");
        profile.role_class = Some("student".to_string());
        profile
    };
    let steward = {
        let mut profile = SwarmAgentConfig::new("agent_steward");
        profile.role_class = Some("steward".to_string());
        profile
    };
    let session_id = unique_session_id("swarm_role_routing");

    let report = swarm
        .execute_swarm(
            json!({}),
            vec![student, steward],
            SwarmExecutionOptions {
                session_id: Some(session_id),
                redis_url: Some(redis_url),
                ..Default::default()
            },
        )
        .await?;

    assert_eq!(report.workers.len(), 2);
    assert_eq!(report.final_context["student_done"], "yes");
    assert_eq!(report.final_context["steward_done"], "yes");
    assert!(report.workers.iter().all(|worker| worker.success));
    Ok(())
}

#[tokio::test]
async fn swarm_engine_cancels_all_workers_on_first_failure()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = QianjiEngine::new();
    engine.add_mechanism_with_affinity(
        "FailFastStudent",
        std::sync::Arc::new(FailingMechanism),
        None,
        NodeExecutionAffinity {
            agent_id: None,
            role_class: Some("student".to_string()),
        },
    );
    engine.add_mechanism_with_affinity(
        "SlowSteward",
        std::sync::Arc::new(SlowMechanism),
        None,
        NodeExecutionAffinity {
            agent_id: None,
            role_class: Some("steward".to_string()),
        },
    );

    let swarm = SwarmEngine::new(engine);
    let student = {
        let mut profile = SwarmAgentConfig::new("agent_student");
        profile.role_class = Some("student".to_string());
        profile
    };
    let steward = {
        let mut profile = SwarmAgentConfig::new("agent_steward");
        profile.role_class = Some("steward".to_string());
        profile
    };

    let result = timeout(
        Duration::from_secs(2),
        swarm.execute_swarm(
            json!({}),
            vec![student, steward],
            SwarmExecutionOptions::default(),
        ),
    )
    .await;

    assert!(
        result.is_ok(),
        "swarm did not abort within cancellation budget"
    );
    let Ok(execution_result) = result else {
        return Err("timeout wrapper should not fail".into());
    };
    assert!(
        execution_result.is_err(),
        "swarm should fail fast when one worker fails"
    );
    Ok(())
}

#[tokio::test]
async fn swarm_engine_emits_pulse_telemetry_events_non_blocking()
-> Result<(), Box<dyn std::error::Error>> {
    let mut engine = QianjiEngine::new();
    let _idx = engine.add_mechanism(
        "Draft",
        std::sync::Arc::new(StaticOutputMechanism {
            key: "draft".to_string(),
            value: "ok".to_string(),
        }),
    );
    let swarm = SwarmEngine::new(engine);
    let emitter = Arc::new(CapturingEmitter::new());

    let report = swarm
        .execute_swarm(
            json!({}),
            vec![SwarmAgentConfig::new("telemetry_agent")],
            SwarmExecutionOptions {
                pulse_emitter: Some(emitter.clone()),
                ..SwarmExecutionOptions::default()
            },
        )
        .await?;
    assert_eq!(report.final_context["draft"], "ok");

    tokio::time::sleep(Duration::from_millis(40)).await;
    let events = emitter.events.lock().await;
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SwarmEvent::SwarmHeartbeat { .. })),
        "missing SwarmHeartbeat event"
    );
    assert!(
        events.iter().any(|event| {
            matches!(
                event,
                SwarmEvent::NodeTransition {
                    phase: NodeTransitionPhase::Entering,
                    node_id,
                    ..
                } if node_id == "Draft"
            )
        }),
        "missing NodeTransition::Entering event for Draft"
    );
    assert!(
        events.iter().any(|event| {
            matches!(
                event,
                SwarmEvent::NodeTransition {
                    phase: NodeTransitionPhase::Exiting,
                    node_id,
                    ..
                } if node_id == "Draft"
            )
        }),
        "missing NodeTransition::Exiting event for Draft"
    );
    Ok(())
}
