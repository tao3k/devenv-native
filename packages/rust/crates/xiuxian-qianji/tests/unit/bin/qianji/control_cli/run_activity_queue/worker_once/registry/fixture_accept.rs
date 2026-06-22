use crate::qianji_cli::test_exports::{
    ActivityExecutionRequest, ActivityExecutorAdapterKind, ActivityExecutorKindArg,
    ActivityExecutorOutcome, ActivityExecutorRegistry, ActivitySettleOutcomeArg,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;

use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    registry_worker_task, registry_worker_task_with,
};

#[test]
fn activity_executor_registry_builds_complete_fixture_outcome() {
    let task = registry_worker_task();
    let outcome = must_ok(
        ActivityExecutorRegistry::fixture_only().execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_ref_json: None,
            output_hash: Some("sha256:activity-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: Some("{\"rows\":3}"),
        }),
        "fixture executor should build complete outcome",
    );
    let ActivityExecutorOutcome::Complete { result } = outcome else {
        panic!("fixture complete should return complete outcome");
    };

    assert_eq!(
        result.output_hash.as_deref(),
        Some("sha256:activity-output")
    );
    assert_eq!(result.metadata["rows"], 3);
}

#[test]
fn activity_executor_registry_returns_fixture_contract_snapshot() {
    let task = registry_worker_task();
    let contract = must_ok(
        ActivityExecutorRegistry::fixture_only()
            .validate_task(ActivityExecutorKindArg::Fixture, Some(&task)),
        "fixture executor should return a validated contract snapshot",
    );

    assert_eq!(contract.executor, ActivityExecutorKindArg::Fixture);
    assert_eq!(contract.adapter, ActivityExecutorAdapterKind::Fixture);
    assert_eq!(
        contract.allowed_activity_types,
        &[
            "llm.plan",
            "llm.tool_select",
            "llm.repair",
            "episteme.ontology.reasoning_fill",
            "tool.github",
            "wendao.search"
        ]
    );
    assert_eq!(
        contract.allowed_task_queues,
        &[
            "llm.openai",
            "llm.anthropic",
            "llm.openrouter",
            "llm.local",
            "episteme.ontology.reasoning",
            "tool.github",
            "wendao.search"
        ]
    );
    assert!(!contract.requires_input_ref);
    assert!(!contract.requires_request_audit);
}

#[test]
fn activity_executor_registry_accepts_llm_tool_selection_route() {
    let task = registry_worker_task_with("llm.tool_select", "llm.openrouter");
    must_ok(
        ActivityExecutorRegistry::fixture_only()
            .validate_task(ActivityExecutorKindArg::Fixture, Some(&task)),
        "fixture executor should accept governed tool-selection LLM routes",
    );
}

#[test]
fn activity_executor_registry_accepts_llm_repair_route() {
    let task = registry_worker_task_with("llm.repair", "llm.local");
    let outcome = must_ok(
        ActivityExecutorRegistry::fixture_only().execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_ref_json: None,
            output_hash: Some("sha256:repair-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: Some("{\"repair\":true}"),
        }),
        "fixture executor should accept governed repair LLM routes",
    );
    let ActivityExecutorOutcome::Complete { result } = outcome else {
        panic!("fixture repair route should return complete outcome");
    };

    assert_eq!(result.output_hash.as_deref(), Some("sha256:repair-output"));
    assert_eq!(result.metadata["repair"], true);
}

#[test]
fn activity_executor_registry_accepts_episteme_reasoning_fixture_route() {
    let task = registry_worker_task_with(
        "episteme.ontology.reasoning_fill",
        "episteme.ontology.reasoning",
    );
    let outcome = must_ok(
        ActivityExecutorRegistry::fixture_only().execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_ref_json: None,
            output_hash: Some("sha256:episteme-review-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: Some("{\"review_only\":true,\"rdf_mutation\":false}"),
        }),
        "fixture executor should accept admitted Episteme reasoning-fill routes",
    );
    let ActivityExecutorOutcome::Complete { result } = outcome else {
        panic!("fixture Episteme route should return complete outcome");
    };

    assert_eq!(
        result.output_hash.as_deref(),
        Some("sha256:episteme-review-output")
    );
    assert_eq!(result.metadata["review_only"], true);
    assert_eq!(result.metadata["rdf_mutation"], false);
}
