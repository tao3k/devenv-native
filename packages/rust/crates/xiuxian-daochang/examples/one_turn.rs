//! Example: one user turn with LLM + optional external tools.
//!
//! Inference: set `OPENAI_API_KEY` (or use `LiteLLM`: `litellm --port 4000` and
//! `LITELLM_PROXY_URL=http://127.0.0.1:4000/v1/chat/completions`). Optional tool server:
//! `OMNI_TOOL_URL=http://127.0.0.1:3002/sse`.
//!
//! Run: `cargo run -p xiuxian-daochang --example one_turn -- "Your message here"`

use xiuxian_daochang::{Agent, AgentConfig, ToolServerEntry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let message = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Say hello in one sentence.".to_string());

    let tool_url = std::env::var("OMNI_TOOL_URL").ok();
    // Prefer LiteLLM when LITELLM_PROXY_URL is set (one endpoint for 100+ providers).
    let mut config = if std::env::var("LITELLM_PROXY_URL").is_ok() {
        AgentConfig::litellm("gpt-4o-mini")
    } else {
        AgentConfig {
            inference_url: std::env::var("OMNI_AGENT_INFERENCE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
            model: std::env::var("OMNI_AGENT_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            api_key: None,
            max_tool_rounds: 10,
            ..AgentConfig::default()
        }
    };
    if let Some(url) = tool_url {
        config.tool_servers = vec![ToolServerEntry {
            name: "local".to_string(),
            url: Some(url),
            command: None,
            args: None,
        }];
    }

    let agent = Agent::from_config(config).await?;
    let out = agent.run_turn("example-session", &message).await?;
    println!("{out}");
    Ok(())
}
