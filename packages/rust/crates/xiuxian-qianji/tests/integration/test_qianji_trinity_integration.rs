#![allow(
    missing_docs,
    unused_imports,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown
)]
#![cfg(feature = "wendao-integration")]

use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::executors::{ContextAnnotator, SynapseCalibrator};
use xiuxian_qianji::{NodeAnnotationExecutionMode, QianjiEngine, QianjiScheduler};

#[tokio::test]
async fn test_qianji_trinity_integration() {
    let mut engine = QianjiEngine::new();
    let annotator = Arc::new(ContextAnnotator {
        persona_id: "artisan-engineer".to_string(),
        template_target: None,
        execution_mode: NodeAnnotationExecutionMode::Isolated,
        input_keys: vec!["raw_facts".to_string()],
        history_key: "annotation_history".to_string(),
        output_key: "annotated_prompt".to_string(),
    });
    let calibrator = Arc::new(SynapseCalibrator {
        target_node_id: "Annotator".to_string(),
        drift_threshold: 0.5,
    });

    let a = engine.add_mechanism("Annotator", annotator);
    let c = engine.add_mechanism("Calibrator", calibrator);
    engine.add_link(a, c, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler.run(json!({
        "raw_facts": "Implementation ensures milimeter-level alignment and audit trail traceability.",
        "drift_score": 0.1
    })).await.unwrap();

    assert!(
        result["annotated_prompt"]
            .as_str()
            .unwrap()
            .contains("<system_prompt_injection>")
    );
}
