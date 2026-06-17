#![allow(missing_docs, unused_imports, clippy::doc_markdown)]
#![cfg(feature = "wendao-integration")]

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
use xiuxian_qianji::{QianjiCompiler, QianjiEngine, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

const PRECISION_RESEARCH_TOML: &str = include_str!("../../resources/tests/precision_research.toml");

#[tokio::test]
async fn test_qianji_high_precision_research_loop()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "artisan-engineer".to_string(),
        name: "Artisan".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: vec![
            "milimeter-level alignment".to_string(),
            "audit trail".to_string(),
        ],
        cot_template: "T".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
    });
    let registry_arc = Arc::new(registry);

    let compiler = QianjiCompiler::new(index, orchestrator, registry_arc);
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
