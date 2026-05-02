use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::QianjiCompiler;
use xiuxian_qianji::scheduler::QianjiScheduler;
use xiuxian_wendao::LinkGraphIndex;

#[tokio::test]
async fn test_smart_commit_workflow_mechanisms() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;

    // Load the miniature version of the smart commit manifest to test the mechanisms.
    let manifest_content = include_str!("../../resources/test_smart_commit_mock.toml");

    let index = Arc::new(LinkGraphIndex::build(temp.path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    let compiler = QianjiCompiler::new(index, orchestrator, registry, None);

    let engine = compiler.compile(manifest_content)?;
    let scheduler = QianjiScheduler::new(engine);

    // Turn 1: Start workflow. It should pause at UserApproval.
    let initial_context = json!({});
    let result_1 = scheduler.run(initial_context).await?;

    // The execution should suspend at UserApproval and return the context with the prompt.
    assert!(
        result_1.get("suspend_prompt").is_some(),
        "Workflow should suspend at UserApproval and yield prompt"
    );
    assert_eq!(result_1["suspend_prompt"], "Please review.");

    // Turn 2: Resume workflow. Provide the resume_key "final_message".
    // Since our test runner doesn't use the actual redis checkpointer in memory inside run()
    // if we don't supply a session_id, we need to manually simulate the node statuses for this test,
    // OR we can just use the checkpointer with a local session!
    // But since `run` without session_id starts fresh, let's use the checkpointer.
    Ok(())
}
