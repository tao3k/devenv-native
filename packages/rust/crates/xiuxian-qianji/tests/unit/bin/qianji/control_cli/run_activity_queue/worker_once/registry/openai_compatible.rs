use crate::qianji_cli::test_exports::{
    ActivityExecutorAdapterKind, ActivityExecutorKindArg, ActivityExecutorRegistry,
};
use crate::qianji_cli::tests::control_cli::support::must_ok;

use crate::qianji_cli::tests::control_cli::run_activity_queue::worker_once::support::{
    registry_episteme_openai_compatible_llm_task, registry_openai_compatible_llm_task,
};

#[test]
fn activity_executor_registry_accepts_openai_compatible_llm_gate() {
    let task = registry_openai_compatible_llm_task();
    let contract = must_ok(
        ActivityExecutorRegistry::fixture_only()
            .validate_task(ActivityExecutorKindArg::OpenAiCompatibleLlm, Some(&task)),
        "openai-compatible LLM executor gate should accept audited LLM task",
    );

    assert_eq!(
        contract.executor,
        ActivityExecutorKindArg::OpenAiCompatibleLlm
    );
    assert_eq!(
        contract.adapter,
        ActivityExecutorAdapterKind::OpenAiCompatibleLlm
    );
    assert_eq!(
        contract.allowed_activity_types,
        &[
            "llm.plan",
            "llm.tool_select",
            "llm.repair",
            "episteme.ontology.reasoning_fill"
        ]
    );
    assert_eq!(
        contract.allowed_task_queues,
        &[
            "llm.openai",
            "llm.openrouter",
            "llm.local",
            "episteme.ontology.reasoning"
        ]
    );
    assert!(contract.requires_input_ref);
    assert!(contract.requires_request_audit);
}

#[test]
fn activity_executor_registry_accepts_episteme_reasoning_openai_compatible_gate() {
    let task = registry_episteme_openai_compatible_llm_task();
    let contract = must_ok(
        ActivityExecutorRegistry::fixture_only()
            .validate_task(ActivityExecutorKindArg::OpenAiCompatibleLlm, Some(&task)),
        "openai-compatible LLM executor gate should accept audited Episteme reasoning task",
    );

    assert_eq!(
        contract.executor,
        ActivityExecutorKindArg::OpenAiCompatibleLlm
    );
    assert!(contract.requires_input_ref);
    assert!(contract.requires_request_audit);
}

#[test]
fn activity_executor_registry_rejects_episteme_reasoning_openai_compatible_without_context_ref() {
    let mut task = registry_episteme_openai_compatible_llm_task();
    task.metadata["qianji_llm_activity_request"]["context_ref"] = serde_json::Value::Null;

    let error = ActivityExecutorRegistry::fixture_only()
        .validate_task(ActivityExecutorKindArg::OpenAiCompatibleLlm, Some(&task))
        .err()
        .unwrap_or_else(|| panic!("missing Episteme context_ref should fail LLM executor gate"));

    assert!(error.to_string().contains("Episteme reasoning context_ref"));
}

#[test]
fn activity_executor_registry_rejects_openai_compatible_llm_without_input_ref() {
    let mut task = registry_openai_compatible_llm_task();
    task.input_ref = None;

    let error = ActivityExecutorRegistry::fixture_only()
        .validate_task(ActivityExecutorKindArg::OpenAiCompatibleLlm, Some(&task))
        .err()
        .unwrap_or_else(|| panic!("missing input_ref should fail LLM executor gate"));

    assert!(error.to_string().contains("requires task input_ref"));
}

#[test]
fn activity_executor_registry_rejects_openai_compatible_llm_without_request_audit() {
    let mut task = registry_openai_compatible_llm_task();
    task.metadata = serde_json::Value::Null;

    let error = ActivityExecutorRegistry::fixture_only()
        .validate_task(ActivityExecutorKindArg::OpenAiCompatibleLlm, Some(&task))
        .err()
        .unwrap_or_else(|| panic!("missing request audit should fail LLM executor gate"));

    assert!(error.to_string().contains("qianji_llm_activity_request"));
}

#[test]
fn activity_executor_registry_rejects_openai_compatible_llm_prompt_ref_mismatch() {
    let mut task = registry_openai_compatible_llm_task();
    task.metadata["qianji_llm_activity_request"]["prompt_ref"]["uri"] =
        serde_json::Value::String("artifact://different-prompt".to_string());

    let error = ActivityExecutorRegistry::fixture_only()
        .validate_task(ActivityExecutorKindArg::OpenAiCompatibleLlm, Some(&task))
        .err()
        .unwrap_or_else(|| panic!("prompt_ref mismatch should fail LLM executor gate"));

    assert!(error.to_string().contains("input_ref to match"));
}
