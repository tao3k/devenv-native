//! Integration tests for scheduler affinity failover and local proxy delegation.

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::engine::NodeExecutionAffinity;
use xiuxian_qianji::error::QianjiError;
use xiuxian_qianji::scheduler::SchedulerRuntimeServices;
use xiuxian_qianji::{
    FlowInstruction, QianjiEngine, QianjiMechanism, QianjiOutput, QianjiScheduler,
    RoleAvailabilityRegistry, SchedulerAgentIdentity, SchedulerExecutionPolicy,
};

struct StaticOutputMechanism {
    key: String,
    value: String,
}

xiuxian_testing::crate_test_policy_harness!();

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

#[derive(Clone)]
struct MockRoleRegistry {
    has_remote_candidate: bool,
}

#[async_trait]
impl RoleAvailabilityRegistry for MockRoleRegistry {
    async fn has_role(&self, _role_class: &str, _exclude_cluster_id: Option<&str>) -> bool {
        self.has_remote_candidate
    }
}

fn build_single_role_engine(required_role: &str) -> QianjiEngine {
    let mut engine = QianjiEngine::new();
    let _idx = engine.add_mechanism_with_affinity(
        "RoleBoundNode",
        Arc::new(StaticOutputMechanism {
            key: "result".to_string(),
            value: "proxied".to_string(),
        }),
        None,
        NodeExecutionAffinity {
            agent_id: None,
            role_class: Some(required_role.to_string()),
        },
    );
    engine
}

#[tokio::test]
async fn scheduler_executes_as_proxy_when_role_missing_globally()
-> Result<(), Box<dyn std::error::Error>> {
    let engine = build_single_role_engine("teacher");
    let services = SchedulerRuntimeServices {
        role_registry: Some(Arc::new(MockRoleRegistry {
            has_remote_candidate: false,
        })),
        execution_policy: SchedulerExecutionPolicy::new().with_local_proxy_delegation(true),
        ..SchedulerRuntimeServices::default()
    };
    let scheduler = QianjiScheduler::with_runtime_services_config(
        engine,
        SchedulerAgentIdentity::new(
            Some("agent_manager".to_string()),
            Some("manager".to_string()),
        ),
        services,
    );

    let final_context = scheduler.run(json!({})).await?;
    assert_eq!(final_context["result"], "proxied");
    Ok(())
}

#[tokio::test]
async fn scheduler_waits_for_remote_when_role_exists_globally()
-> Result<(), Box<dyn std::error::Error>> {
    let engine = build_single_role_engine("teacher");
    let services = SchedulerRuntimeServices {
        role_registry: Some(Arc::new(MockRoleRegistry {
            has_remote_candidate: true,
        })),
        execution_policy: SchedulerExecutionPolicy::new().with_local_proxy_delegation(true),
        ..SchedulerRuntimeServices::default()
    };
    let scheduler = QianjiScheduler::with_runtime_services_config(
        engine,
        SchedulerAgentIdentity::new(
            Some("agent_manager".to_string()),
            Some("manager".to_string()),
        ),
        services,
    );

    let result = scheduler.run(json!({})).await;
    assert!(
        result.is_err(),
        "scheduler should not proxy when a remote role exists"
    );
    match result {
        Err(QianjiError::Execution(message)) => {
            assert!(
                message.contains("checkpoint is disabled"),
                "unexpected scheduler error: {message}"
            );
        }
        Err(other) => panic!("unexpected error variant: {other:?}"),
        Ok(_) => panic!("expected failure when remote role exists"),
    }
    Ok(())
}
