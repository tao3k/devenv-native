#![allow(
    missing_docs,
    unused_imports,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown
)]

use std::sync::Arc;
use xiuxian_qianji::executors::MockMechanism;
use xiuxian_qianji::{QianjiEngine, QianjiSafetyGuard, QianjiScheduler};

#[tokio::test]
async fn test_qianji_safety_static_cycle_detection() {
    let mut engine = QianjiEngine::new();
    let mech = Arc::new(MockMechanism {
        name: "A".to_string(),
        weight: 1.0,
    });

    let a = engine.add_mechanism("A", mech.clone());
    let b = engine.add_mechanism("B", mech.clone());

    engine.add_link(a, b, None, 1.0);
    engine.add_link(b, a, None, 1.0);

    let guard = QianjiSafetyGuard::new(10);
    let result = guard.audit_topology(&engine);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Infinite cycle detected")
    );
}

#[tokio::test]
async fn test_qianji_runtime_loop_guard() {
    let mut engine = QianjiEngine::new();
    let mech = Arc::new(MockMechanism {
        name: "A".to_string(),
        weight: 1.0,
    });
    let _a = engine.add_mechanism("A", mech);

    let _scheduler = QianjiScheduler::new(engine);
}
