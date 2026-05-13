use std::sync::Arc;

use xiuxian_wendao_core::{
    CognitiveSessionId, CognitiveTraceRecord, LinkGraphSemanticDocument,
    LinkGraphSemanticDocumentKind,
};

#[test]
fn semantic_document_kind_summary_as_str() {
    assert_eq!(LinkGraphSemanticDocumentKind::Summary.as_str(), "summary");
}

#[test]
fn semantic_document_kind_section_as_str() {
    assert_eq!(LinkGraphSemanticDocumentKind::Section.as_str(), "section");
}

#[test]
fn semantic_document_kind_cognitive_trace_as_str() {
    assert_eq!(
        LinkGraphSemanticDocumentKind::CognitiveTrace.as_str(),
        "cognitive_trace"
    );
}

#[test]
fn semantic_document_kind_equality() {
    assert_eq!(
        LinkGraphSemanticDocumentKind::CognitiveTrace,
        LinkGraphSemanticDocumentKind::CognitiveTrace
    );
    assert_ne!(
        LinkGraphSemanticDocumentKind::CognitiveTrace,
        LinkGraphSemanticDocumentKind::Summary
    );
}

#[test]
fn cognitive_trace_record_new_creates_minimal_record() {
    let record = CognitiveTraceRecord::new(
        "trace-123".to_string(),
        Some("session-456".into()),
        "AuditNode".to_string(),
        "Critique the agenda".to_string(),
    );

    assert_eq!(record.trace_id, "trace-123");
    assert_eq!(
        record.session_id.as_ref().map(CognitiveSessionId::as_str),
        Some("session-456")
    );
    assert_eq!(record.node_id, "AuditNode");
    assert_eq!(record.intent, "Critique the agenda");
    assert_eq!(record.reasoning.as_ref(), "");
    assert!(record.outcome.is_none());
    assert!(record.commit_sha.is_none());
    assert!(record.timestamp_ms > 0);
    assert!(record.coherence_score.is_none());
    assert!(!record.early_halt_triggered);
}

#[test]
fn cognitive_trace_record_new_without_session() {
    let record = CognitiveTraceRecord::new(
        "trace-789".to_string(),
        None,
        "PlanNode".to_string(),
        "Generate plan".to_string(),
    );

    assert_eq!(record.trace_id, "trace-789");
    assert!(record.session_id.is_none());
    assert_eq!(record.node_id, "PlanNode");
}

#[test]
fn cognitive_trace_record_to_semantic_document() {
    let record = CognitiveTraceRecord {
        trace_id: "trace-abc".to_string().into(),
        session_id: Some("session-def".into()),
        node_id: "ExecutorNode".to_string().into(),
        intent: "Execute task".to_string(),
        reasoning: Arc::<str>::from("Step 1: Analyze\nStep 2: Execute"),
        outcome: Some(Arc::<str>::from("Task completed")),
        commit_sha: Some("abc123".to_string()),
        timestamp_ms: 1_700_000_000_000,
        coherence_score: Some(0.95),
        early_halt_triggered: false,
    };

    let doc = record.to_semantic_document("doc-123", "traces/executor.md");

    assert_eq!(doc.anchor_id, "trace:trace-abc");
    assert_eq!(doc.doc_id, "doc-123");
    assert_eq!(doc.path, "traces/executor.md");
    assert_eq!(doc.kind, LinkGraphSemanticDocumentKind::CognitiveTrace);
    assert_eq!(doc.semantic_path, vec!["Cognitive Traces", "ExecutorNode"]);
    assert_eq!(doc.content.as_ref(), "Step 1: Analyze\nStep 2: Execute");
    assert!(doc.line_range.is_none());
}

#[test]
fn cognitive_trace_record_with_early_halt() {
    let mut record = CognitiveTraceRecord::new(
        "trace-halt".to_string(),
        None,
        "MonitorNode".to_string(),
        "Monitor execution".to_string(),
    );
    record.coherence_score = Some(0.25);
    record.early_halt_triggered = true;

    assert_eq!(record.coherence_score, Some(0.25));
    assert!(record.early_halt_triggered);
}

#[test]
fn cognitive_trace_record_clone_preserves_values() {
    let record = CognitiveTraceRecord {
        trace_id: "trace-original".to_string().into(),
        session_id: None,
        node_id: "TestNode".to_string().into(),
        intent: "Test intent".to_string(),
        reasoning: Arc::<str>::from("Test reasoning"),
        outcome: Some(Arc::<str>::from("Test outcome")),
        commit_sha: None,
        timestamp_ms: 1_700_000_000_000,
        coherence_score: Some(0.85),
        early_halt_triggered: false,
    };

    let cloned = record.clone();

    assert_eq!(cloned.trace_id, record.trace_id);
    assert_eq!(cloned.node_id, record.node_id);
    assert_eq!(cloned.intent, record.intent);
    assert_eq!(cloned.reasoning, record.reasoning);
    assert_eq!(cloned.outcome, record.outcome);
    assert_eq!(cloned.coherence_score, record.coherence_score);
}

#[test]
fn cognitive_trace_record_partial_eq() {
    let record1 = CognitiveTraceRecord {
        trace_id: "trace-1".to_string().into(),
        session_id: Some("session-1".into()),
        node_id: "Node1".to_string().into(),
        intent: "Intent".to_string(),
        reasoning: Arc::<str>::from("Reasoning"),
        outcome: None,
        commit_sha: None,
        timestamp_ms: 1_700_000_000_000,
        coherence_score: None,
        early_halt_triggered: false,
    };

    let record2 = record1.clone();
    let record3 = CognitiveTraceRecord {
        trace_id: "trace-2".to_string().into(),
        ..record1.clone()
    };

    assert_eq!(record1, record2);
    assert_ne!(record1, record3);
}

#[test]
fn cognitive_trace_record_debug_format() {
    let record = CognitiveTraceRecord::new(
        "trace-debug".to_string(),
        None,
        "DebugNode".to_string(),
        "Debug intent".to_string(),
    );

    let debug_str = format!("{record:?}");
    assert!(debug_str.contains("trace-debug"));
    assert!(debug_str.contains("DebugNode"));
    assert!(debug_str.contains("Debug intent"));
}

#[test]
fn semantic_document_with_cognitive_trace_kind() {
    let doc = LinkGraphSemanticDocument {
        anchor_id: "trace:test-123".to_string().into(),
        doc_id: "doc-456".to_string().into(),
        path: "traces/test.md".to_string().into(),
        kind: LinkGraphSemanticDocumentKind::CognitiveTrace,
        semantic_path: vec!["Cognitive Traces".to_string(), "TestNode".to_string()],
        content: Arc::<str>::from("Test reasoning content"),
        line_range: None,
    };

    assert_eq!(doc.kind, LinkGraphSemanticDocumentKind::CognitiveTrace);
    assert_eq!(doc.kind.as_str(), "cognitive_trace");
}

#[test]
fn semantic_document_equality() {
    let doc1 = LinkGraphSemanticDocument {
        anchor_id: "anchor-1".to_string().into(),
        doc_id: "doc-1".to_string().into(),
        path: "path/1.md".to_string().into(),
        kind: LinkGraphSemanticDocumentKind::Section,
        semantic_path: vec!["Path".to_string()],
        content: Arc::<str>::from("Content"),
        line_range: Some((1, 10)),
    };

    let doc2 = doc1.clone();
    assert_eq!(doc1, doc2);
}
