use crate::contracts::{FlowInstruction, QianjiMechanism};
use crate::executors::ProbabilisticRouter;
use serde_json::json;

#[tokio::test]
async fn test_router_selects_single_branch() {
    let router = ProbabilisticRouter {
        branches: vec![("alpha".to_string(), 1.0)],
        semantic_guard_route_key: None,
    };
    let output = router
        .execute(&json!({}))
        .await
        .unwrap_or_else(|err| panic!("router should succeed: {err}"));
    assert_eq!(output.data["selected_route"], "alpha");
    match output.instruction {
        FlowInstruction::SelectBranch(branch) => assert_eq!(branch, "alpha"),
        _ => panic!("Expected SelectBranch instruction"),
    }
}

#[tokio::test]
async fn test_router_empty_branches_error() {
    let router = ProbabilisticRouter {
        branches: vec![],
        semantic_guard_route_key: None,
    };
    let Err(err) = router.execute(&json!({})).await else {
        panic!("router should fail when no branches are configured");
    };
    assert!(err.contains("no branches"));
}

#[tokio::test]
async fn test_router_zero_weight_error() {
    let router = ProbabilisticRouter {
        branches: vec![("alpha".to_string(), 0.0)],
        semantic_guard_route_key: None,
    };
    let Err(err) = router.execute(&json!({})).await else {
        panic!("router should fail when no positive weights exist");
    };
    assert!(err.contains("no positive"));
}

#[tokio::test]
async fn test_router_invalid_confidence_error() {
    let router = ProbabilisticRouter {
        branches: vec![("alpha".to_string(), 1.0)],
        semantic_guard_route_key: None,
    };
    let Err(err) = router.execute(&json!({ "omega_confidence": -1.0 })).await else {
        panic!("router should fail for invalid omega_confidence");
    };
    assert!(err.contains("omega_confidence"));
}

#[tokio::test]
async fn test_router_selects_semantic_guard_route_branch_when_enabled() {
    let router = ProbabilisticRouter {
        branches: vec![
            ("continue".to_string(), 1.0),
            ("review_required".to_string(), 1.0),
            ("blocked".to_string(), 1.0),
        ],
        semantic_guard_route_key: Some("semanticScopeGuardRoute".to_string()),
    };
    let output = router
        .execute(&json!({
            "semanticScopeGuardRoute": {
                "recommendedAction": "review_required"
            },
            "omega_confidence": -1.0
        }))
        .await
        .unwrap_or_else(|err| panic!("router should consume semantic guard route: {err}"));
    assert_eq!(output.data["selected_route"], "review_required");
    match output.instruction {
        FlowInstruction::SelectBranch(branch) => assert_eq!(branch, "review_required"),
        _ => panic!("Expected SelectBranch instruction"),
    }
}

#[tokio::test]
async fn test_router_ignores_semantic_guard_route_without_opt_in() {
    let router = ProbabilisticRouter {
        branches: vec![("continue".to_string(), 1.0)],
        semantic_guard_route_key: None,
    };
    let output = router
        .execute(&json!({
            "semanticScopeGuardRoute": {
                "recommendedAction": "review_required"
            }
        }))
        .await
        .unwrap_or_else(|err| panic!("router should keep default behavior: {err}"));
    assert_eq!(output.data["selected_route"], "continue");
}
