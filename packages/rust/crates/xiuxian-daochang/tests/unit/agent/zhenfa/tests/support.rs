use std::sync::Arc;

use async_trait::async_trait;
use num_traits::ToPrimitive;
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, ZhenfaSignal, ZhenfaTool};

pub(super) struct RewardEmitterTool;

#[async_trait]
impl ZhenfaTool for RewardEmitterTool {
    fn id(&self) -> &'static str {
        "reward.emitter"
    }

    fn definition(&self) -> serde_json::Value {
        json!({
            "name": "reward.emitter",
            "description": "Emit one reward signal for memory sink tests",
            "parameters": {
                "type": "object",
                "properties": {
                    "episode_id": { "type": "string" },
                    "value": { "type": "number" }
                }
            }
        })
    }

    async fn call_native(
        &self,
        ctx: &ZhenfaContext,
        args: serde_json::Value,
    ) -> Result<String, ZhenfaError> {
        let episode_id = args
            .get("episode_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let value = args
            .get("value")
            .and_then(serde_json::Value::as_f64)
            .and_then(|raw| raw.to_f32())
            .unwrap_or(0.0);
        let signal_sender = ctx
            .get_extension::<UnboundedSender<ZhenfaSignal>>()
            .ok_or_else(|| ZhenfaError::execution("signal sender missing from context"))?;
        signal_sender
            .send(ZhenfaSignal::Reward {
                episode_id,
                value,
                source: "test.reward_emitter".to_string(),
            })
            .map_err(|_| ZhenfaError::execution("signal receiver closed"))?;
        Ok("<ok/>".to_string())
    }
}

pub(super) fn build_manifestation_manager() -> Arc<ManifestationManager> {
    Arc::new(
        ManifestationManager::new_with_embedded_templates(&[], &[("bootstrap.md", "bootstrap")])
            .unwrap_or_else(|error| panic!("build manifestation manager: {error}")),
    )
}
