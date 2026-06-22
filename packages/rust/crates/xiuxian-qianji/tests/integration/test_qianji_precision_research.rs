#![allow(missing_docs, clippy::doc_markdown)]
#![cfg(feature = "wendao-integration")]

use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

const PRECISION_RESEARCH_TOML: &str = include_str!("../../resources/tests/precision_research.toml");

#[tokio::test]
async fn test_qianji_high_precision_research_loop()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);

    let compiler = QianjiCompiler::new(index);
    let engine = compiler.compile(PRECISION_RESEARCH_TOML)?;
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler
        .run(json!({
            "raw_facts": "Implementation ensures milimeter-level alignment and audit trail.",
            "drift_score": 0.01
        }))
        .await?;

    let annotated = result["annotated_prompt"].as_str().unwrap_or("");
    assert!(
        annotated.contains("<system_prompt_injection>"),
        "Annotation failed"
    );
    assert_eq!(result["calibration_status"], "passed");
    Ok(())
}
