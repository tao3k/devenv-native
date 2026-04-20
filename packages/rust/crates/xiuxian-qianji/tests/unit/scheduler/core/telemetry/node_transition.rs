use crate::telemetry::NodeTransitionPhase;
use crate::{QianjiEngine, QianjiScheduler};
use petgraph::stable_graph::NodeIndex;

#[tokio::test]
async fn emit_node_transition_is_callable_without_emitter() {
    let scheduler = QianjiScheduler::new(QianjiEngine::default());
    scheduler
        .emit_node_transition(NodeIndex::new(0), NodeTransitionPhase::Entering, None)
        .await;
}
