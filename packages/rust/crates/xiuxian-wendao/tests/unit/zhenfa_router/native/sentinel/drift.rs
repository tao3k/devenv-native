use super::{AffectedDoc, DriftConfidence, SemanticDriftSignal};

#[test]
fn test_semantic_drift_signal_summary() {
    let mut signal = SemanticDriftSignal::new("src/lib.rs", "lib");
    signal.add_affected_doc(AffectedDoc::new(
        "docs/api",
        "fn lib_init($$$)",
        "rust",
        "node-1",
    ));
    signal.update_confidence(DriftConfidence::High);

    let summary = signal.summary();
    assert!(summary.contains("lib"));
    assert!(summary.contains("docs/api"));
}

#[test]
fn test_semantic_drift_signal_serialization() {
    let mut signal = SemanticDriftSignal::new("src/lib.rs", "lib");
    signal.add_affected_doc(AffectedDoc::new(
        "docs/api",
        "fn lib_init($$$)",
        "rust",
        "node-1",
    ));

    let json = signal.to_streaming_payload();
    assert!(json.contains("lib"));
    assert!(json.contains("docs/api"));
}

#[test]
fn test_drift_confidence_levels() {
    assert_eq!(DriftConfidence::High, DriftConfidence::High);
    assert_ne!(DriftConfidence::High, DriftConfidence::Low);
}

#[test]
fn test_affected_doc_builder() {
    let doc = AffectedDoc::new("docs/test", "pattern", "rust", "node-1").with_line(42);

    assert_eq!(doc.doc_id, "docs/test");
    assert_eq!(doc.matching_pattern, "pattern");
    assert_eq!(doc.language, "rust");
    assert_eq!(doc.line_number, Some(42));
    assert_eq!(doc.node_id, "node-1");
}

#[test]
fn test_drift_confidence_ordering() {
    assert!(DriftConfidence::Low < DriftConfidence::Medium);
    assert!(DriftConfidence::Medium < DriftConfidence::High);
    assert!(DriftConfidence::Low < DriftConfidence::High);
    assert!(DriftConfidence::High > DriftConfidence::Medium);
    assert!(DriftConfidence::Medium > DriftConfidence::Low);
    assert_eq!(DriftConfidence::Low, DriftConfidence::Low);
    assert_eq!(DriftConfidence::Medium, DriftConfidence::Medium);
    assert_eq!(DriftConfidence::High, DriftConfidence::High);
    assert!(DriftConfidence::Medium >= DriftConfidence::Low);
    assert!(DriftConfidence::High >= DriftConfidence::Medium);
    assert!(DriftConfidence::High >= DriftConfidence::High);
}

#[test]
fn test_drift_confidence_threshold_filtering() {
    let threshold = DriftConfidence::Medium;

    assert!(DriftConfidence::Low < threshold);
    assert!(DriftConfidence::Medium >= threshold);
    assert!(DriftConfidence::High >= threshold);
}

#[test]
fn test_drift_confidence_auto_fix_threshold() {
    let auto_fix_threshold = DriftConfidence::High;

    assert!(DriftConfidence::Low < auto_fix_threshold);
    assert!(DriftConfidence::Medium < auto_fix_threshold);
    assert!(DriftConfidence::High >= auto_fix_threshold);
}
