use crate::qianji_cli::test_exports::{
    ActivityExecutionRequest, ActivityExecutorKindArg, ActivityExecutorRegistry,
    ActivitySettleOutcomeArg,
};

use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    registry_worker_task, registry_worker_task_with,
};

#[test]
fn activity_executor_registry_rejects_blank_output_ref_uri() {
    let task = registry_worker_task();
    let error = ActivityExecutorRegistry::fixture_only()
        .execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_ref_json: Some(
                r#"{"artifact_id":"artifact-worker-output","artifact_kind":"llm.output","uri":"   "}"#,
            ),
            output_hash: Some("sha256:activity-output"),
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
        })
        .err()
        .unwrap_or_else(|| panic!("blank output ref uri should fail"));

    assert!(
        error
            .to_string()
            .contains("activity output ArtifactRef uri must not be blank"),
        "unexpected error: {error}"
    );
}

#[test]
fn activity_executor_registry_rejects_failed_fixture_without_retryable() {
    let task = registry_worker_task();
    let error = ActivityExecutorRegistry::fixture_only()
        .execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::Fixture,
            outcome: ActivitySettleOutcomeArg::Fail,
            output_ref_json: None,
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
            output_ref_json: None,
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
            output_ref_json: None,
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
            output_ref_json: None,
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
