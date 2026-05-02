use crate::runtime::linear::dmn::{assert_local_business_rule_task, run_local_business_rule_task};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::BpmnEngineError;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_resolves_registered_decision_service_locally() {
    assert_local_business_rule_task(
        "dmn_decision_service",
        "wf_dmn_decision_service",
        "DecisionService_credit",
        "versioned-local-decision-service-runtime-20191111.dmn",
        json!({ "tier": "gold" }),
        json!({ "tier": "gold", "approval": "approve" }),
        84,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_validates_registered_decision_service_exposures_locally() {
    assert_local_business_rule_task(
        "dmn_decision_service_exposures",
        "wf_dmn_decision_service_exposures",
        "DecisionService_credit",
        "versioned-local-decision-service-exposure-runtime-20191111.dmn",
        json!({ "tier": "gold", "application": { "id": "app-1" } }),
        json!({ "tier": "gold", "application": { "id": "app-1" }, "approval": "approve" }),
        87,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_missing_input_decision_service_exposure_locally() {
    let error = run_local_business_rule_task(
        "dmn_decision_service_missing_input_exposure",
        "wf_dmn_decision_service_missing_input_exposure",
        "DecisionService_credit",
        "versioned-missing-input-decision-service-exposure-runtime-20191111.dmn",
        json!({ "tier": "gold", "application": { "id": "app-2" } }),
        88,
    )
    .await
    .must_err("missing local decision-service input decisions should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnDecisionServiceReferenceTarget {
            source_id: "versioned-missing-input-decision-service-exposure-runtime-20191111.dmn"
                .to_string(),
            decision_service_id: "DecisionService_credit".to_string(),
            reference_kind: "inputDecision".to_string(),
            href: "#Decision_missing".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_unsupported_input_data_exposure_href_locally() {
    let error = run_local_business_rule_task(
        "dmn_decision_service_unsupported_input_data_exposure",
        "wf_dmn_decision_service_unsupported_input_data_exposure",
        "DecisionService_credit",
        "versioned-unsupported-input-data-decision-service-exposure-runtime-20191111.dmn",
        json!({ "tier": "gold", "application": { "id": "app-3" } }),
        89,
    )
    .await
    .must_err("non-local decision-service input-data hrefs should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnDecisionServiceReferenceHref {
            source_id:
                "versioned-unsupported-input-data-decision-service-exposure-runtime-20191111.dmn"
                    .to_string(),
            decision_service_id: "DecisionService_credit".to_string(),
            reference_kind: "inputData".to_string(),
            href: "InputData_application".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_missing_decision_service_output_target_locally() {
    let error = run_local_business_rule_task(
        "dmn_decision_service_missing_output",
        "wf_dmn_decision_service_missing_output",
        "DecisionService_missing_output",
        "versioned-missing-output-decision-service-runtime-20191111.dmn",
        json!({ "tier": "gold" }),
        85,
    )
    .await
    .must_err("missing local decision-service outputs should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnDecisionServiceOutputTarget {
            source_id: "versioned-missing-output-decision-service-runtime-20191111.dmn".to_string(),
            decision_service_id: "DecisionService_missing_output".to_string(),
            href: "#Decision_missing".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_resolves_multi_output_decision_service_locally() {
    assert_local_business_rule_task(
        "dmn_decision_service_multi_output",
        "wf_dmn_decision_service_multi_output",
        "DecisionService_multi_output",
        "versioned-multi-output-decision-service-runtime-20191111.dmn",
        json!({ "tier": "gold" }),
        json!({ "tier": "gold", "approval": "approve", "review_state": "secondary" }),
        86,
    )
    .await;
}
