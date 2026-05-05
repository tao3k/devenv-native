use std::path::PathBuf;

use crate::{RuntimeSettings, build_agent, run_http};

pub(crate) async fn run_gateway_mode(
    bind_addr: String,
    turn_timeout: Option<u64>,
    max_concurrent: Option<usize>,
    tool_config_path: PathBuf,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let agent = build_agent(&tool_config_path, runtime_settings).await?;
    run_http(agent, &bind_addr, turn_timeout, max_concurrent).await
}
