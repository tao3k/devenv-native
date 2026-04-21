use super::assert_local_business_rule_task;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_evaluates_registered_dmn_decision_locally() {
    assert_local_business_rule_task(
        "dmn_local",
        "wf_dmn_local",
        "loan-decision",
        "simple-unique-eligibility.dmn",
        json!({ "tier": "gold" }),
        json!({ "tier": "gold", "approval": "approve" }),
        70,
    )
    .await;
}
