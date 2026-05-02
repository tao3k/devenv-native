use crate::contracts::{FlowInstruction, QianjiMechanism};
use crate::executors::ProbabilisticRouter;
use serde_json::json;

#[tokio::test]
async fn test_router_selects_single_branch() {
    let router = ProbabilisticRouter {
        branches: vec![("alpha".to_string(), 1.0)],
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
    let router = ProbabilisticRouter { branches: vec![] };
    let Err(err) = router.execute(&json!({})).await else {
        panic!("router should fail when no branches are configured");
    };
    assert!(err.contains("no branches"));
}

#[tokio::test]
async fn test_router_zero_weight_error() {
    let router = ProbabilisticRouter {
        branches: vec![("alpha".to_string(), 0.0)],
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
    };
    let Err(err) = router.execute(&json!({ "omega_confidence": -1.0 })).await else {
        panic!("router should fail for invalid omega_confidence");
    };
    assert!(err.contains("omega_confidence"));
}
