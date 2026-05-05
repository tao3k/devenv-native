use std::path::PathBuf;

use crate::{RuntimeSettings, run_stdio};

use crate::build_agent;

pub(crate) async fn run_stdio_mode(
    session_id: String,
    tool_config_path: PathBuf,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let agent = build_agent(&tool_config_path, runtime_settings).await?;
    Box::pin(run_stdio(agent, session_id)).await
}
