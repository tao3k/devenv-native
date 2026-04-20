use crate::error::QianjiError;
use crate::swarm::engine_orchestrator::SwarmEngine;
use crate::swarm::engine_types::{SwarmAgentConfig, SwarmAgentReport};
use xiuxian_window::SessionWindow;

impl SwarmEngine {
    pub(in crate::swarm::engine::worker) fn build_worker_report(
        identity: SwarmAgentConfig,
        role: Option<String>,
        session_id: &str,
        window: &mut SessionWindow,
        run_result: Result<serde_json::Value, QianjiError>,
    ) -> Result<SwarmAgentReport, QianjiError> {
        let context = match run_result {
            Ok(context) => {
                window.append_turn("assistant", "swarm_worker_completed", 0, Some(session_id));
                context
            }
            Err(error) => {
                window.append_turn("assistant", "swarm_worker_failed", 0, Some(session_id));
                return Err(error);
            }
        };
        let (window_turns, window_tool_calls, _ring_len) = window.get_stats();
        Ok(SwarmAgentReport {
            agent_id: identity.agent_id,
            role_class: role,
            success: true,
            context: Some(context),
            error: None,
            window_turns,
            window_tool_calls,
        })
    }
}
