use crate::QianjiLlmClient;
use crate::contracts::NodeDefinition;
use crate::error::QianjiError;
use std::sync::Arc;
use xiuxian_llm::llm::{LlmClient, OpenAIClient};

use super::llm_node;

pub(super) fn resolve_for_node(
    node_def: &NodeDefinition,
    global_client: Option<Arc<QianjiLlmClient>>,
) -> Result<Arc<dyn LlmClient>, QianjiError> {
    let binding = node_def.llm.as_ref();
    if let Some((base_url, api_key)) = llm_node::resolve_node_llm_endpoint(binding)? {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_mins(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        return Ok(Arc::new(OpenAIClient {
            api_key,
            base_url,
            http,
        }));
    }

    global_client.ok_or(QianjiError::Topology(
        "LLM client not provided to compiler".to_string(),
    ))
}
