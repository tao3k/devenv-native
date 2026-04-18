use super::QianjiScheduler;
use crate::telemetry::{NodeTransitionPhase, SwarmEvent, unix_millis_now};
use petgraph::stable_graph::NodeIndex;

mod alerts;
#[cfg(test)]
#[path = "../../../../tests/unit/scheduler/core/telemetry/node_transition.rs"]
mod node_transition;

impl QianjiScheduler {
    pub(in crate::scheduler::core) fn emit_event_non_blocking(&self, event: SwarmEvent) {
        let Some(emitter) = self.telemetry_emitter.clone() else {
            return;
        };
        std::mem::drop(tokio::spawn(async move {
            if let Err(error) = emitter.emit_pulse(event).await {
                log::debug!("scheduler telemetry emission skipped: {error}");
            }
        }));
    }

    pub(in crate::scheduler::core) async fn emit_node_transition(
        &self,
        node_idx: NodeIndex,
        phase: NodeTransitionPhase,
        session_id: Option<&str>,
    ) {
        let node_id = {
            let engine = self.engine.read().await;
            engine.graph.node_weight(node_idx).map_or_else(
                || format!("node#{}", node_idx.index()),
                |node| node.id.clone(),
            )
        };
        self.emit_event_non_blocking(SwarmEvent::NodeTransition {
            session_id: session_id.map(std::string::ToString::to_string),
            agent_id: self.execution_identity.agent_id.clone(),
            role_class: self.execution_identity.role_class.clone(),
            node_id,
            phase,
            timestamp_ms: unix_millis_now(),
        });
    }
}
