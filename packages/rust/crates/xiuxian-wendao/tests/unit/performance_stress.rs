//! Performance guardrails for narrator throughput.

use std::time::Instant;
use xiuxian_wendao::LinkGraphHit;

#[tokio::test]
async fn test_narrator_performance_scaling() {
    // 1. Setup a synthetic large subgraph (100 hits)
    let mut hits = Vec::new();
    for i in 0..100 {
        hits.push(LinkGraphHit {
            stem: format!("node_{i}"),
            score: 1.0 - (f64::from(i) * 0.01),
            title: format!("Deep Scaling Analysis node {i}"),
            path: format!("path/{i}.md").into(),
            doc_type: None,
            tags: vec!["doc".to_string()],
            best_section: None,
            match_reason: None,
        });
    }

    // 2. Measure narration latency
    let start = Instant::now();
    let _output = xiuxian_wendao::narrate_subgraph(&hits);
    let duration = start.elapsed();

    println!("Narration of 100 hits took: {duration:?}");

    // Artisan Threshold: Narration should be < 5ms for 100 hits
    assert!(
        duration.as_millis() < 5,
        "Narration is too slow for large subgraphs"
    );
}
