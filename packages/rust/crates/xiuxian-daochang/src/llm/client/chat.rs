use anyhow::Result;

#[cfg(feature = "agent-provider-litellm")]
use crate::llm::backend::LlmBackendMode;
use crate::llm::client::LlmClient;
#[cfg(feature = "agent-provider-litellm")]
use crate::llm::compat::litellm::LiteLlmDispatchConfig;
use crate::llm::tools::{PreparedTool, parse_tools_json};
use crate::llm::types::{AssistantMessage, ChatCompletionRequest, ChatCompletionResponse};
use crate::session::ChatMessage;

impl LlmClient {
    /// Send messages and optionally tool definitions; returns content and/or tool calls.
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools_json: Option<Vec<serde_json::Value>>,
    ) -> Result<AssistantMessage> {
        let tools = parse_tools_json(tools_json);
        match self.backend_mode {
            #[cfg(feature = "agent-provider-litellm")]
            LlmBackendMode::LiteLlmRs => self.chat_via_litellm_rs(messages, tools).await,
            _ => self.chat_via_http(messages, tools).await,
        }
    }

    async fn chat_via_http(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<PreparedTool>,
    ) -> Result<AssistantMessage> {
        let tools = if tools.is_empty() {
            None
        } else {
            Some(tools.iter().map(PreparedTool::to_http_tool_def).collect())
        };
        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.inference_max_tokens,
            tool_choice: tools.as_ref().map(|_| "auto".to_string()),
            tools,
        };
        let mut req = self
            .client
            .post(&self.inference_url)
            .json(&body)
            .header("Content-Type", "application/json");
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }
        let res = req.send().await?;
        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!("LLM API error {status}: {text}"));
        }
        let parsed: ChatCompletionResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("LLM response parse error: {e}; body: {text}"))?;
        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("LLM response has no choices"))?;
        Ok(choice.message)
    }

    #[cfg(feature = "agent-provider-litellm")]
    async fn chat_via_litellm_rs(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<PreparedTool>,
    ) -> Result<AssistantMessage> {
        self.litellm_runtime
            .chat(
                LiteLlmDispatchConfig {
                    provider_mode: self.litellm_provider_mode,
                    wire_api: self.litellm_wire_api,
                    model: self.model.as_str(),
                    max_tokens: self.inference_max_tokens,
                    api_key: self.api_key.as_deref(),
                    litellm_api_key_env: self.litellm_api_key_env.as_str(),
                    inference_api_base: self.inference_api_base.as_str(),
                    minimax_api_base: self.minimax_api_base.as_str(),
                    timeout_secs: self.inference_timeout_secs,
                },
                messages,
                tools,
            )
            .await
    }
}
