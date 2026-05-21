use crate::transmuter::streaming::{
    CognitiveDistribution, StreamProvider, ZhenfaPipeline, ZhenfaPipelineOptions,
};

#[test]
fn pipeline_creates_correct_parser() {
    let claude = ZhenfaPipeline::new(StreamProvider::Claude);
    assert_eq!(claude.provider(), StreamProvider::Claude);

    let gemini = ZhenfaPipeline::new(StreamProvider::Gemini);
    assert_eq!(gemini.provider(), StreamProvider::Gemini);

    let codex = ZhenfaPipeline::new(StreamProvider::Codex);
    assert_eq!(codex.provider(), StreamProvider::Codex);
}

#[test]
fn pipeline_parses_claude_text_delta() {
    let mut pipeline = ZhenfaPipeline::new(StreamProvider::Claude);
    let line =
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;

    let outputs = match pipeline.process_line(line) {
        Ok(outputs) => outputs,
        Err(err) => panic!("parse should succeed: {err}"),
    };
    assert!(!outputs.is_empty());
}

#[test]
fn pipeline_tracks_cognitive_state() {
    let mut pipeline = ZhenfaPipeline::with_options(
        ZhenfaPipelineOptions::new(StreamProvider::Claude).with_xsd_validation(false),
    );

    let line = r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"Let me plan my approach."}}"#;
    let _ = pipeline.process_line(line);

    let dist = pipeline.cognitive_distribution();
    assert!(dist.meta > 0 || dist.operational > 0 || dist.epistemic > 0);
}

#[test]
fn pipeline_respects_early_halt() {
    let mut pipeline = ZhenfaPipeline::with_options(
        ZhenfaPipelineOptions::new(StreamProvider::Claude)
            .with_xsd_validation(false)
            .with_early_halt_threshold(0.5),
    );

    for _ in 0..10 {
        let line = r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"I'm not sure about this."}}"#;
        let _ = pipeline.process_line(line);

        if pipeline.should_halt() {
            break;
        }
    }

    let score = pipeline.coherence_score();
    assert!(score < 1.0);
}

#[test]
fn cognitive_distribution_calculates_balance() {
    let dist = CognitiveDistribution {
        meta: 3,
        operational: 7,
        epistemic: 2,
        instrumental: 1,
        system: 0,
    };

    assert_eq!(dist.total(), 13);
    assert!((dist.balance() - 0.3).abs() < 0.01);
    assert!((dist.uncertainty_ratio() - 2.0 / 13.0).abs() < 0.01);
}

#[test]
fn pipeline_reset_clears_state() {
    let mut pipeline = ZhenfaPipeline::new(StreamProvider::Claude);
    let line =
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
    let _ = pipeline.process_line(line);

    pipeline.reset();
    assert!(!pipeline.should_halt());
}

#[test]
fn pipeline_disables_validation() {
    let mut pipeline = ZhenfaPipeline::with_options(
        ZhenfaPipelineOptions::new(StreamProvider::Claude)
            .with_xsd_validation(false)
            .with_cognitive_monitoring(false)
            .with_early_halt_threshold(0.0),
    );
    let line = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Any text"}}"#;

    let outputs = match pipeline.process_line(line) {
        Ok(outputs) => outputs,
        Err(err) => panic!("should succeed: {err}"),
    };
    assert!(!outputs.is_empty());

    for output in outputs {
        assert!(output.validation.is_empty());
        assert!(output.cognitive.is_none());
    }
}
