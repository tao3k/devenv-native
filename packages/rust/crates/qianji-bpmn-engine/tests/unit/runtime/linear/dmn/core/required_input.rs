use crate::runtime::linear::dmn::{assert_local_business_rule_task, run_local_business_rule_task};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::BpmnEngineError;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_binds_required_input_alias_locally() {
    assert_local_business_rule_task(
        "dmn_required_input",
        "wf_dmn_required_input",
        "Decision_alias_required_input",
        "versioned-local-required-input-runtime-20191111.dmn",
        json!({ "applicant_input": { "age": 41 } }),
        json!({
            "applicant_input": { "age": 41 },
            "approval": "approve",
        }),
        79,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_missing_required_input_target_locally() {
    let error = run_local_business_rule_task(
        "dmn_required_input_missing",
        "wf_dmn_required_input_missing",
        "Decision_missing_required_input",
        "versioned-missing-required-input-runtime-20191111.dmn",
        json!({ "applicant_input": { "age": 41 } }),
        83,
    )
    .await
    .must_err("missing required input targets should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnRequiredInputTarget {
            source_id: "versioned-missing-required-input-runtime-20191111.dmn".to_string(),
            decision_id: "Decision_missing_required_input".to_string(),
            href: "#InputData_missing".to_string(),
        }
    );
}
