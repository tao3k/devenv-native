use crate::runtime::linear::dmn::{assert_local_business_rule_task, run_local_business_rule_task};
use crate::test_support::MustExt as _;
use serde_json::json;
use xiuxian_qianji_bpmn_engine::BpmnEngineError;

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_resolves_required_knowledge_dependency_locally() {
    assert_local_business_rule_task(
        "dmn_required_knowledge",
        "wf_dmn_required_knowledge",
        "Decision_required_knowledge_runtime",
        "versioned-local-required-knowledge-runtime-20191111.dmn",
        json!({ "applicant": { "age": 41 } }),
        json!({
            "applicant": { "age": 41 },
            "Decision_required_knowledge_runtime": 42.0,
        }),
        80,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_missing_required_knowledge_target_locally() {
    let error = run_local_business_rule_task(
        "dmn_required_knowledge_missing",
        "wf_dmn_required_knowledge_missing",
        "Decision_missing_required_knowledge",
        "versioned-missing-required-knowledge-runtime-20191111.dmn",
        json!({ "applicant": { "age": 41 } }),
        81,
    )
    .await
    .must_err("missing required knowledge targets should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnRequiredKnowledgeTarget {
            source_id: ("versioned-missing-required-knowledge-runtime-20191111.dmn".to_string())
                .into(),
            decision_id: ("Decision_missing_required_knowledge".to_string()).into(),
            href: "#BKM_missing".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_business_rule_task_rejects_invocation_outside_required_knowledge_locally() {
    let error = run_local_business_rule_task(
        "dmn_required_knowledge_outside",
        "wf_dmn_required_knowledge_outside",
        "Decision_outside_required_knowledge",
        "versioned-outside-required-knowledge-runtime-20191111.dmn",
        json!({ "applicant": { "age": 41 } }),
        82,
    )
    .await
    .must_err("invocation targets outside required knowledge should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::UndeclaredDmnInvocationKnowledgeTarget {
            source_id: ("versioned-outside-required-knowledge-runtime-20191111.dmn".to_string())
                .into(),
            decision_id: ("Decision_outside_required_knowledge".to_string()).into(),
            target: "scoreCard".to_string(),
        }
    );
}
