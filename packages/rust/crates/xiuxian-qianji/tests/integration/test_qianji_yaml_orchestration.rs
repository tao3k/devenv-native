#![allow(
    missing_docs,
    unused_imports,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown
)]

use serde_json::json;
use std::sync::Arc;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

const DIAMOND_DAG_TOML: &str = include_str!("../../resources/tests/diamond_dag.toml");

#[tokio::test]
async fn test_qianji_native_toml_orchestration_diamond() {
    let temp = tempfile::tempdir().unwrap();
    let index = Arc::new(LinkGraphIndex::build(temp.path()).unwrap());
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    // Fix: Inject None for llm_client
    let compiler = QianjiCompiler::new(index, orchestrator, registry, None);
    let engine = compiler
        .compile(DIAMOND_DAG_TOML)
        .expect("Compilation failed");
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler.run(json!({})).await.expect("Execution failed");

    assert_eq!(result["A"], "done");
    assert_eq!(result["D"], "done");
}
