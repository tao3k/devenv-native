//! Integration coverage for Qianji layout-to-BPMN export.

use async_trait::async_trait;
use std::sync::Arc;
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use xiuxian_qianji::engine::QianjiEngine;
use xiuxian_qianji::layout::{QgsTheme, QianjiLayoutEngine, generate_bpmn_xml};

#[tokio::test]
async fn test_omg_standard_compliance_branching_flow() {
    let mut engine = QianjiEngine::new();

    // Construct a branching flow: Start -> Gateway -> [A (80%), B (20%)] -> End
    let start = engine.add_mechanism("start", Arc::new(MockMechanism));
    let gateway = engine.add_mechanism("decide_logic", Arc::new(MockMechanism));
    let task_a = engine.add_mechanism("audit_high", Arc::new(MockMechanism));
    let task_b = engine.add_mechanism("audit_low", Arc::new(MockMechanism));
    let end = engine.add_mechanism("end", Arc::new(MockMechanism));

    engine.add_link(start, gateway, None, 1.0);
    engine.add_link(gateway, task_a, Some("HighRisk"), 0.8);
    engine.add_link(gateway, task_b, Some("LowRisk"), 0.2);
    engine.add_link(task_a, end, None, 1.0);
    engine.add_link(task_b, end, None, 1.0);

    let layout_engine = QianjiLayoutEngine::new(QgsTheme::default());
    let layout = layout_engine.compute_from_engine(&engine);
    let xml = generate_bpmn_xml(&layout);

    // 1. Check Gateway Mapping
    assert!(xml.contains("bpmn:exclusiveGateway"));

    // 2. Check Probability Labels (Weight Mapping)
    assert!(xml.contains("80%"));
    assert!(xml.contains("20%"));

    // 3. Check Orthogonal Waypoints (at least 4 waypoints for an S-shape)
    assert!(xml.contains("<di:waypoint"));

    // 4. Check DI Labels
    assert!(xml.contains("<bpmndi:BPMNLabel>"));
}

struct MockMechanism;

#[async_trait]
impl QianjiMechanism for MockMechanism {
    async fn execute(&self, _context: &serde_json::Value) -> Result<QianjiOutput, String> {
        Ok(QianjiOutput {
            data: serde_json::json!({}),
            instruction: FlowInstruction::Continue,
        })
    }
    fn weight(&self) -> f32 {
        1.0
    }
}
