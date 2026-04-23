use super::RemotePossessionBus;
use super::keys::{request_key, response_channel, response_key};
use crate::swarm::possession_model::RemoteNodeResponse;
use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use tokio::time::{Duration, timeout};

impl RemotePossessionBus {
    pub(in crate::swarm) async fn submit_response_impl(
        &self,
        response: &RemoteNodeResponse,
        ttl_seconds: u64,
    ) -> Result<()> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        let response_json = serde_json::to_string(response)?;
        let response_storage_key = response_key(&response.request_id);
        let request_storage_key = request_key(&response.request_id);
        let channel = response_channel(&response.request_id);

        let _: () = self
            .run_command("remote_possession_set_response", || {
                let mut command = redis::cmd("SET");
                command.arg(&response_storage_key).arg(&response_json);
                command
            })
            .await?;
        let _: bool = self
            .run_command("remote_possession_expire_response", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&response_storage_key).arg(ttl_seconds);
                command
            })
            .await?;
        let _: i64 = self
            .run_command("remote_possession_hset_request_done", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&request_storage_key)
                    .arg("status")
                    .arg("completed")
                    .arg("finished_ms")
                    .arg(response.finished_ms.to_string());
                command
            })
            .await?;
        let _: i64 = self
            .run_command("remote_possession_publish_response", || {
                let mut command = redis::cmd("PUBLISH");
                command.arg(&channel).arg(&response_json);
                command
            })
            .await?;
        Ok(())
    }

    pub(in crate::swarm) async fn wait_response_impl(
        &self,
        request_id: &str,
        max_wait: Duration,
    ) -> Result<Option<RemoteNodeResponse>> {
        let response_storage_key = response_key(request_id);
        let existing: Option<String> = self
            .run_command("remote_possession_get_response", || {
                let mut command = redis::cmd("GET");
                command.arg(&response_storage_key);
                command
            })
            .await?;
        if let Some(payload) = existing {
            let parsed = serde_json::from_str::<RemoteNodeResponse>(&payload)?;
            return Ok(Some(parsed));
        }

        let client = redis::Client::open(self.redis_url.as_str())
            .context("Failed to open Valkey connection for remote possession wait")?;
        let mut pubsub = client.get_async_pubsub().await?;
        let channel = response_channel(request_id);
        pubsub.subscribe(channel).await?;
        let mut stream = pubsub.on_message();

        match timeout(max_wait, async {
            while let Some(message) = stream.next().await {
                let payload: String = message.get_payload()?;
                let parsed = serde_json::from_str::<RemoteNodeResponse>(&payload)?;
                if parsed.request_id == request_id {
                    return Ok(parsed);
                }
            }
            Err(anyhow!(
                "remote possession pubsub stream closed unexpectedly"
            ))
        })
        .await
        {
            Ok(inner) => inner.map(Some),
            Err(_elapsed) => Ok(None),
        }
    }
}
