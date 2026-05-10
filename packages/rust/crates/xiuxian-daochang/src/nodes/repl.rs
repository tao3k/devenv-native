use std::path::PathBuf;

use crate::{Agent, RuntimeSettings, StdioSessionId, run_stdio};

use crate::agent_builder::build_agent;

pub(crate) async fn run_repl_mode(
    query: Option<String>,
    session_id: String,
    tool_config_path: PathBuf,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let agent: Agent = build_agent(&tool_config_path, runtime_settings).await?;
    if let Some(q) = query {
        let out = agent.run_turn(&session_id, q.trim()).await?;
        println!("{out}");
        Ok(())
    } else {
        run_stdio(agent, StdioSessionId::new(session_id)).await
    }
}
