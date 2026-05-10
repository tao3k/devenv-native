//! Example: run the agent as HTTP gateway or stdio.
//!
//! External tool servers from **tool.json** only (default `.tool.json`). Use `--tool-config <path>` to override.
//!
//! Subcommands:
//!   gateway  — HTTP server (POST /message). Default: --bind 0.0.0.0:8080
//!   stdio    — Read lines from stdin, run turn, print output. Optional --session-id
//!
//! Examples:
//!   cargo run -p xiuxian-daochang --example gateway -- gateway --bind 0.0.0.0:8080
//!   cargo run -p xiuxian-daochang --example gateway -- stdio --session-id s1

use std::path::PathBuf;

use xiuxian_daochang::{
    Agent, AgentConfig, DEFAULT_STDIO_SESSION_ID, StdioSessionId, load_tool_config, run_http,
    run_stdio,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, rest) = args
        .split_first()
        .map_or(("gateway", &[][..]), |(m, r)| (m.as_str(), r));
    if mode == "stdio" {
        let (session_id, tool_config_path) = parse_stdio_args(rest);
        Box::pin(run_stdio_mode(session_id, tool_config_path)).await
    } else {
        let (bind_addr, tool_config_path) = parse_gateway_args(rest);
        Box::pin(run_gateway(bind_addr, tool_config_path)).await
    }
}

fn parse_gateway_args(args: &[String]) -> (String, PathBuf) {
    let mut bind = "0.0.0.0:8080".to_string();
    let mut tool_config = PathBuf::from(".tool.json");
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--bind" && i + 1 < args.len() {
            bind.clone_from(&args[i + 1]);
            i += 2;
            continue;
        }
        if args[i] == "--tool-config" && i + 1 < args.len() {
            tool_config = PathBuf::from(&args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }
    (bind, tool_config)
}

fn parse_stdio_args(args: &[String]) -> (String, PathBuf) {
    let mut session_id = DEFAULT_STDIO_SESSION_ID.to_string();
    let mut tool_config = PathBuf::from(".tool.json");
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--session-id" && i + 1 < args.len() {
            session_id.clone_from(&args[i + 1]);
            i += 2;
            continue;
        }
        if args[i] == "--tool-config" && i + 1 < args.len() {
            tool_config = PathBuf::from(&args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }
    (session_id, tool_config)
}

async fn run_gateway(bind_addr: String, tool_config_path: PathBuf) -> anyhow::Result<()> {
    let tool_servers = load_tool_config(&tool_config_path)?;
    let config = AgentConfig {
        inference_url: std::env::var("LITELLM_PROXY_URL")
            .unwrap_or_else(|_| AgentConfig::default().inference_url),
        model: std::env::var("OMNI_AGENT_MODEL").unwrap_or_else(|_| AgentConfig::default().model),
        api_key: None,
        tool_servers,
        max_tool_rounds: 10,
        ..AgentConfig::default()
    };
    let agent = Agent::from_config(config).await?;
    Box::pin(run_http(agent, &bind_addr, None, None)).await
}

async fn run_stdio_mode(session_id: String, tool_config_path: PathBuf) -> anyhow::Result<()> {
    let tool_servers = load_tool_config(&tool_config_path)?;
    let config = AgentConfig {
        inference_url: std::env::var("LITELLM_PROXY_URL")
            .unwrap_or_else(|_| AgentConfig::default().inference_url),
        model: std::env::var("OMNI_AGENT_MODEL").unwrap_or_else(|_| AgentConfig::default().model),
        api_key: None,
        tool_servers,
        max_tool_rounds: 10,
        ..AgentConfig::default()
    };
    let agent = Agent::from_config(config).await?;
    Box::pin(run_stdio(agent, StdioSessionId::new(session_id))).await
}
