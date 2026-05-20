use super::rerank::{
    apply_plugin_rerank_scores, build_plugin_rerank_telemetry, collect_plugin_rerank_anchors,
    collect_plugin_rerank_trace_ids,
};
use crate::link_graph::models::{QuantumAnchorHit, QuantumContext};
use std::collections::BTreeMap;
use xiuxian_wendao_core::transport::PluginTransportKind;
use xiuxian_wendao_runtime::transport::NegotiatedTransportSelection;
use xiuxian_wendao_runtime::transport::PluginArrowScoreRow;

#[test]
fn apply_plugin_rerank_scores_overwrites_saliency_and_resorts_contexts() {
    let mut contexts = vec![
        QuantumContext {
            anchor_id: "doc-1#a".to_string(),
            doc_id: "doc-1".to_string(),
            path: "notes/doc-1.md".to_string(),
            semantic_path: vec![],
            trace_label: None,
            related_clusters: vec![],
            saliency_score: 0.2,
            vector_score: 0.6,
            topology_score: 0.1,
        },
        QuantumContext {
            anchor_id: "doc-2#b".to_string(),
            doc_id: "doc-2".to_string(),
            path: "notes/doc-2.md".to_string(),
            semantic_path: vec![],
            trace_label: None,
            related_clusters: vec![],
            saliency_score: 0.9,
            vector_score: 0.5,
            topology_score: 0.2,
        },
    ];
    let response_rows = BTreeMap::from([
        (
            "doc-1#a".to_string(),
            PluginArrowScoreRow {
                doc_id: "doc-1#a".to_string(),
                analyzer_score: 0.7,
                final_score: 0.95,
                trace_id: None,
            },
        ),
        (
            "doc-2#b".to_string(),
            PluginArrowScoreRow {
                doc_id: "doc-2#b".to_string(),
                analyzer_score: 0.3,
                final_score: 0.4,
                trace_id: None,
            },
        ),
    ]);

    let updated = apply_plugin_rerank_scores(&mut contexts, &response_rows);

    assert_eq!(updated, 2);
    assert_eq!(contexts[0].anchor_id, "doc-1#a");
    assert!((contexts[0].saliency_score - 0.95).abs() < f64::EPSILON);
    assert_eq!(contexts[1].anchor_id, "doc-2#b");
    assert!((contexts[1].saliency_score - 0.4).abs() < f64::EPSILON);
}

#[test]
fn collect_plugin_rerank_trace_ids_deduplicates_non_empty_values() {
    let response_rows = BTreeMap::from([
        (
            "doc-1#a".to_string(),
            PluginArrowScoreRow {
                doc_id: "doc-1#a".to_string(),
                analyzer_score: 0.7,
                final_score: 0.95,
                trace_id: Some("trace-123".to_string()),
            },
        ),
        (
            "doc-2#b".to_string(),
            PluginArrowScoreRow {
                doc_id: "doc-2#b".to_string(),
                analyzer_score: 0.3,
                final_score: 0.4,
                trace_id: Some("trace-123".to_string()),
            },
        ),
        (
            "doc-3#c".to_string(),
            PluginArrowScoreRow {
                doc_id: "doc-3#c".to_string(),
                analyzer_score: 0.2,
                final_score: 0.1,
                trace_id: Some("trace-456".to_string()),
            },
        ),
    ]);

    let trace_ids = collect_plugin_rerank_trace_ids(&response_rows);

    assert_eq!(
        trace_ids,
        vec!["trace-123".to_string(), "trace-456".to_string()]
    );
}

#[test]
fn collect_plugin_rerank_anchors_preserves_anchor_ids_and_scores() {
    let contexts = vec![
        QuantumContext {
            anchor_id: "doc-1#a".to_string(),
            doc_id: "doc-1".to_string(),
            path: "notes/doc-1.md".to_string(),
            semantic_path: vec![],
            trace_label: None,
            related_clusters: vec![],
            saliency_score: 0.2,
            vector_score: 0.6,
            topology_score: 0.1,
        },
        QuantumContext {
            anchor_id: "doc-2#b".to_string(),
            doc_id: "doc-2".to_string(),
            path: "notes/doc-2.md".to_string(),
            semantic_path: vec![],
            trace_label: None,
            related_clusters: vec![],
            saliency_score: 0.9,
            vector_score: 0.5,
            topology_score: 0.2,
        },
    ];

    let anchors = collect_plugin_rerank_anchors(&contexts);

    assert_eq!(
        anchors,
        vec![
            QuantumAnchorHit {
                anchor_id: "doc-1#a".to_string(),
                vector_score: 0.6,
            },
            QuantumAnchorHit {
                anchor_id: "doc-2#b".to_string(),
                vector_score: 0.5,
            },
        ]
    );
}

#[test]
fn build_plugin_rerank_telemetry_carries_transport_selection_and_fallback() {
    let telemetry = build_plugin_rerank_telemetry(
        Some(&NegotiatedTransportSelection {
            selected_transport: PluginTransportKind::ArrowFlight,
            fallback_from: None,
            fallback_reason: None,
        }),
        true,
        2,
        vec!["trace-123".to_string()],
        None,
    );

    assert!(telemetry.applied);
    assert_eq!(telemetry.response_row_count, 2);
    assert_eq!(
        telemetry.selected_transport,
        Some(PluginTransportKind::ArrowFlight)
    );
    assert_eq!(telemetry.fallback_from, None);
    assert_eq!(telemetry.fallback_reason, None);
    assert_eq!(telemetry.trace_ids, vec!["trace-123".to_string()]);
    assert_eq!(telemetry.error, None);
}
