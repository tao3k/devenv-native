use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji::executors::MockMechanism;
use xiuxian_qianji::{QianjiEngine, QianjiScheduler};

#[tokio::test]
async fn test_qianji_dag_parallel_execution() -> Result<(), xiuxian_qianji::error::QianjiError> {
    let mut engine = QianjiEngine::new();

    let a_mech = Arc::new(MockMechanism {
        name: "A".to_string(),
        weight: 1.0,
    });
    let b_mech = Arc::new(MockMechanism {
        name: "B".to_string(),
        weight: 1.0,
    });
    let c_mech = Arc::new(MockMechanism {
        name: "C".to_string(),
        weight: 1.0,
    });
    let d_mech = Arc::new(MockMechanism {
        name: "D".to_string(),
        weight: 1.0,
    });

    let a = engine.add_mechanism("A", a_mech);
    let b = engine.add_mechanism("B", b_mech);
    let c = engine.add_mechanism("C", c_mech);
    let d = engine.add_mechanism("D", d_mech);

    // Use standardized add_link API
    engine.add_link(a, b, None, 1.0);
    engine.add_link(a, c, None, 1.0);
    engine.add_link(b, d, None, 1.0);
    engine.add_link(c, d, None, 1.0);

    let scheduler = QianjiScheduler::new(engine);
    let result = scheduler.run(json!({})).await?;

    assert_eq!(result["A"], "done");
    assert_eq!(result["B"], "done");
    assert_eq!(result["C"], "done");
    assert_eq!(result["D"], "done");
    Ok(())
}
