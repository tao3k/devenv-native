use crate::qianji_cli::test_exports::{
    ActivityExecutionRequest, ActivityExecutorAdapterKind, ActivityExecutorKindArg,
    ActivityExecutorOutcome, ActivityExecutorRegistry, ActivitySettleOutcomeArg,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;

use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::registry_flowhub_service_task;

#[test]
fn activity_executor_registry_executes_flowhub_service_contract_completion() {
    let task = registry_flowhub_service_task();
    let outcome = must_ok(
        ActivityExecutorRegistry::fixture_only().execute(ActivityExecutionRequest {
            task: Some(&task),
            executor: ActivityExecutorKindArg::FlowhubService,
            outcome: ActivitySettleOutcomeArg::Complete,
            output_ref_json: None,
            output_hash: None,
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
        }),
        "Flowhub service executor should derive deterministic completion metadata",
    );
    let ActivityExecutorOutcome::Complete { result } = outcome else {
        panic!("Flowhub service executor should complete");
    };

    assert!(result.output_ref.is_none());
    assert!(result.output_hash.is_none());
    assert_eq!(
        result.metadata["qianji_flowhub_service_completion"]["schema"],
        "xiuxian_qianji.flowhub.service_completion.v1"
    );
    assert_eq!(
        result.metadata["qianji_flowhub_service_completion"]["data"]["projectResolved"],
        true
    );
}

#[test]
fn activity_executor_registry_returns_flowhub_service_contract_snapshot() {
    let task = registry_flowhub_service_task();
    let contract = must_ok(
        ActivityExecutorRegistry::fixture_only()
            .validate_task(ActivityExecutorKindArg::FlowhubService, Some(&task)),
        "Flowhub service executor should accept Flowhub service tasks",
    );

    assert_eq!(contract.executor, ActivityExecutorKindArg::FlowhubService);
    assert_eq!(
        contract.adapter,
        ActivityExecutorAdapterKind::FlowhubService
    );
    assert_eq!(contract.allowed_activity_types, &["flowhub.service"]);
    assert_eq!(contract.allowed_task_queues, &["flowhub.*"]);
    assert!(contract.requires_input_ref);
    assert!(!contract.requires_request_audit);
}
