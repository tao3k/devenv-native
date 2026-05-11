use super::{
    AffectedDoc, DriftConfidence, ObservationBus, ObservationRef, ObservationSignal,
    SemanticDriftSignal, mpsc, signals_to_status_batch,
};

#[test]
fn test_observation_signal_stale_from_drift() {
    let mut drift = SemanticDriftSignal::new("src/lib.rs", "lib");
    drift.add_affected_doc(AffectedDoc::new(
        "docs/api",
        "fn lib_init($$$)",
        "rust",
        "node-1",
    ));
    drift.update_confidence(DriftConfidence::High);

    let signals = ObservationSignal::stale_from_drift(&drift);
    assert_eq!(signals.len(), 1);

    match &signals[0] {
        ObservationSignal::Stale {
            doc_id,
            observation,
            trigger_source,
            confidence,
        } => {
            assert_eq!(doc_id, "docs/api");
            assert_eq!(observation.pattern, "fn lib_init($$$)");
            assert_eq!(observation.language, "rust");
            assert_eq!(*trigger_source, "src/lib.rs");
            assert_eq!(*confidence, DriftConfidence::High);
        }
        _ => panic!("Expected Stale signal"),
    }
}

#[test]
fn test_observation_signal_to_status_message() {
    let signal = ObservationSignal::Stale {
        doc_id: "docs/api".to_string().into(),
        observation: ObservationRef {
            pattern: "fn test()".to_string(),
            language: "rust".to_string(),
            line_number: 42,
            node_id: "node-1".to_string().into(),
        },
        trigger_source: "src/lib.rs".to_string(),
        confidence: DriftConfidence::High,
    };

    let msg = signal.to_status_message();
    assert!(msg.contains("Stale"));
    assert!(msg.contains("docs/api"));
    assert!(msg.contains("fn test()"));
    assert!(msg.contains("High"));
}

#[test]
fn test_observation_signal_requires_attention() {
    let high_stale = ObservationSignal::Stale {
        doc_id: "docs/api".to_string().into(),
        observation: ObservationRef {
            pattern: "fn test()".to_string(),
            language: "rust".to_string(),
            line_number: 1,
            node_id: "n1".to_string().into(),
        },
        trigger_source: "src/lib.rs".to_string(),
        confidence: DriftConfidence::High,
    };
    assert!(high_stale.requires_attention());

    let low_stale = ObservationSignal::Stale {
        doc_id: "docs/api".to_string().into(),
        observation: ObservationRef {
            pattern: "fn test()".to_string(),
            language: "rust".to_string(),
            line_number: 1,
            node_id: "n1".to_string().into(),
        },
        trigger_source: "src/lib.rs".to_string(),
        confidence: DriftConfidence::Low,
    };
    assert!(!low_stale.requires_attention());

    let broken = ObservationSignal::Broken {
        doc_id: "docs/api".to_string().into(),
        observation: ObservationRef {
            pattern: "fn test()".to_string(),
            language: "rust".to_string(),
            line_number: 1,
            node_id: "n1".to_string().into(),
        },
        error: "Pattern not found".to_string(),
    };
    assert!(broken.requires_attention());
}

#[test]
fn test_observation_bus_emit() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut bus = ObservationBus::new();
    assert!(!bus.is_connected());

    bus.connect(tx);
    assert!(bus.is_connected());

    let signal = ObservationSignal::Stale {
        doc_id: "docs/api".to_string().into(),
        observation: ObservationRef {
            pattern: "fn test()".to_string(),
            language: "rust".to_string(),
            line_number: 1,
            node_id: "n1".to_string().into(),
        },
        trigger_source: "src/lib.rs".to_string(),
        confidence: DriftConfidence::High,
    };

    let id = bus.emit(signal);
    assert!(id.is_some());

    let received = rx.try_recv();
    assert!(received.is_ok());
}

#[test]
fn test_observation_bus_emit_drift_signals() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut bus = ObservationBus::new();
    bus.connect(tx);

    let mut drift = SemanticDriftSignal::new("src/lib.rs", "lib");
    drift.add_affected_doc(AffectedDoc::new("docs/a", "p1", "rust", "n1"));
    drift.add_affected_doc(AffectedDoc::new("docs/b", "p2", "rust", "n2"));

    let ids = bus.emit_drift_signals(&drift);
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_signals_to_status_batch() {
    let signals = vec![
        ObservationSignal::Stale {
            doc_id: "docs/a".to_string().into(),
            observation: ObservationRef {
                pattern: "fn a()".to_string(),
                language: "rust".to_string(),
                line_number: 1,
                node_id: "n1".to_string().into(),
            },
            trigger_source: "src/a.rs".to_string(),
            confidence: DriftConfidence::High,
        },
        ObservationSignal::Broken {
            doc_id: "docs/b".to_string().into(),
            observation: ObservationRef {
                pattern: "fn b()".to_string(),
                language: "rust".to_string(),
                line_number: 2,
                node_id: "n2".to_string().into(),
            },
            error: "Not found".to_string(),
        },
    ];

    let batch = signals_to_status_batch(&signals);
    assert!(batch.contains("Observation Signal Batch"));
    assert!(batch.contains("2 signal(s)"));
    assert!(batch.contains("2 require immediate attention"));
}
