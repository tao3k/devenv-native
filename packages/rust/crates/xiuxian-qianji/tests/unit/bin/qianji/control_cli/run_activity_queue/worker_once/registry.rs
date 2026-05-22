use crate::qianji_cli::test_exports::{
    ActivityExecutionRequest, ActivityExecutorAdapterKind, ActivityExecutorKindArg,
    ActivityExecutorOutcome, ActivityExecutorRegistry, ActivitySettleOutcomeArg,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;

use super::support::{registry_worker_task, registry_worker_task_with};

#[test]
fn activity_executor_registry_builds_complete_fixture_outcome() {
    let task = registry_worker_task();
    let outcome = must_ok(
        ActivityExecutorRegistry::fixture_only().execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
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
            "tool.github",
            "wendao.search"
        ]
    );
    assert!(!contract.requires_input_ref);
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
fn activity_executor_registry_rejects_failed_fixture_without_retryable() {
    let task = registry_worker_task();
    let error = ActivityExecutorRegistry::fixture_only()
        .execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Fail,
            output_hash: None,
            error_code: Some("rate_limited"),
            message: Some("provider rejected request"),
            retryable: None,
            metadata: None,
        })
        .err()
        .unwrap_or_else(|| panic!("missing retryable should fail fixture execution"));

    assert!(
        error
            .to_string()
            .contains("missing fixture `retryable` for failed activity execution"),
        "unexpected error: {error}"
    );
}

#[test]
fn activity_executor_registry_rejects_missing_worker_task() {
    let error = ActivityExecutorRegistry::fixture_only()
        .execute(ActivityExecutionRequest {
            task: None,
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_hash: Some("sha256:activity-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
        })
        .err()
        .unwrap_or_else(|| panic!("missing worker task should fail fixture execution"));

    assert!(
        error
            .to_string()
            .contains("activity executor requires a claimed worker activity task"),
        "unexpected error: {error}"
    );
}

#[test]
fn activity_executor_registry_rejects_disallowed_activity_type() {
    let task = registry_worker_task_with("provider.unknown", "llm.openai");
    let error = ActivityExecutorRegistry::fixture_only()
        .execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_hash: Some("sha256:activity-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
        })
        .err()
        .unwrap_or_else(|| panic!("disallowed activity type should fail fixture execution"));

    assert!(
        error.to_string().contains(
            "activity executor `Fixture` does not allow activity_type `provider.unknown`"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn activity_executor_registry_rejects_disallowed_task_queue() {
    let task = registry_worker_task_with("llm.plan", "provider.unknown");
    let error = ActivityExecutorRegistry::fixture_only()
        .execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_hash: Some("sha256:activity-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
        })
        .err()
        .unwrap_or_else(|| panic!("disallowed task queue should fail fixture execution"));

    assert!(
        error
            .to_string()
            .contains("activity executor `Fixture` does not allow task_queue `provider.unknown`"),
        "unexpected error: {error}"
    );
}
