use std::sync::Arc;
use std::sync::RwLock as StdRwLock;
use std::{collections::HashMap, sync::PoisonError};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock, mpsc};
pub(super) use xiuxian_daochang::test_support::{
    DiscordForegroundInterruptController as ForegroundInterruptController,
    build_discord_foreground_runtime, process_discord_message,
    process_discord_message_with_interrupt,
};
use xiuxian_daochang::{
    Agent, AgentConfig, Channel, ChannelMessage, JobManager, JobManagerConfig,
    RecipientCommandAdminUsersMutation, RecipientMentionPolicyStatus, TurnRunner,
};

#[derive(Default)]
pub(super) struct MockChannel {
    sent: Mutex<Vec<(String, String)>>,
    partition_mode: RwLock<String>,
    allow_control_commands: bool,
    denied_slash_scopes: Vec<String>,
    recipient_admin_users: StdRwLock<HashMap<String, Vec<String>>>,
    default_require_mention: StdRwLock<bool>,
    persist_enabled: StdRwLock<bool>,
    recipient_require_mention: StdRwLock<HashMap<String, bool>>,
}

impl MockChannel {
    pub(super) fn with_acl(
        allow_control_commands: bool,
        denied_slash_scopes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        Self {
            sent: Mutex::new(Vec::new()),
            partition_mode: RwLock::new("guild_channel_user".to_string()),
            allow_control_commands,
            denied_slash_scopes: denied_slash_scopes
                .into_iter()
                .map(|scope| scope.as_ref().to_string())
                .collect(),
            recipient_admin_users: StdRwLock::new(HashMap::new()),
            default_require_mention: StdRwLock::new(false),
            persist_enabled: StdRwLock::new(false),
            recipient_require_mention: StdRwLock::new(HashMap::new()),
        }
    }

    pub(super) async fn sent_messages(&self) -> Vec<(String, String)> {
        self.sent.lock().await.clone()
    }

    pub(super) async fn partition_mode(&self) -> String {
        self.partition_mode.read().await.clone()
    }
}

#[async_trait]
impl Channel for MockChannel {
    fn name(&self) -> &'static str {
        "discord-runtime-mock"
    }

    fn session_partition_mode(&self) -> Option<String> {
        Some(
            self.partition_mode
                .try_read()
                .map_or_else(|_| "guild_channel_user".to_string(), |guard| guard.clone()),
        )
    }

    fn set_session_partition_mode(&self, mode: &str) -> anyhow::Result<()> {
        if let Ok(mut guard) = self.partition_mode.try_write() {
            *guard = mode.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("failed to acquire partition write lock"))
        }
    }

    fn is_authorized_for_control_command(&self, _identity: &str, _command_text: &str) -> bool {
        self.allow_control_commands
    }

    fn is_authorized_for_control_command_for_recipient(
        &self,
        identity: &str,
        command_text: &str,
        recipient: &str,
    ) -> bool {
        self.is_authorized_for_control_command(identity, command_text)
            || self
                .recipient_command_admin_users(recipient)
                .ok()
                .flatten()
                .is_some_and(|entries| {
                    entries
                        .iter()
                        .any(|entry| entry == "*" || entry == identity)
                })
    }

    fn is_authorized_for_slash_command(&self, _identity: &str, command_scope: &str) -> bool {
        !self
            .denied_slash_scopes
            .iter()
            .any(|scope| scope == command_scope)
    }

    fn is_authorized_for_slash_command_for_recipient(
        &self,
        identity: &str,
        command_scope: &str,
        recipient: &str,
    ) -> bool {
        self.is_authorized_for_slash_command(identity, command_scope)
            || self
                .recipient_command_admin_users(recipient)
                .ok()
                .flatten()
                .is_some_and(|entries| {
                    entries
                        .iter()
                        .any(|entry| entry == "*" || entry == identity)
                })
    }

    fn recipient_command_admin_users(
        &self,
        recipient: &str,
    ) -> anyhow::Result<Option<Vec<String>>> {
        Ok(self
            .recipient_admin_users
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(recipient)
            .cloned())
    }

    fn mutate_recipient_command_admin_users(
        &self,
        recipient: &str,
        mutation: RecipientCommandAdminUsersMutation,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let recipient = recipient.trim();
        if recipient.is_empty() {
            return Err(anyhow::anyhow!(
                "recipient-scoped admin override requires a non-empty recipient key"
            ));
        }

        let mut overrides = self
            .recipient_admin_users
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let current = overrides.get(recipient).cloned();
        let next = match mutation {
            RecipientCommandAdminUsersMutation::Clear => None,
            RecipientCommandAdminUsersMutation::Set(entries) => {
                let filtered: Vec<String> = entries
                    .into_iter()
                    .map(|entry| entry.trim().to_string())
                    .filter(|entry| !entry.is_empty())
                    .collect();
                (!filtered.is_empty()).then_some(filtered)
            }
            RecipientCommandAdminUsersMutation::Add(entries) => {
                let mut merged = current.unwrap_or_default();
                for entry in entries {
                    let trimmed = entry.trim();
                    if !trimmed.is_empty() && !merged.iter().any(|existing| existing == trimmed) {
                        merged.push(trimmed.to_string());
                    }
                }
                (!merged.is_empty()).then_some(merged)
            }
            RecipientCommandAdminUsersMutation::Remove(entries) => {
                let removals: Vec<String> = entries
                    .into_iter()
                    .map(|entry| entry.trim().to_string())
                    .filter(|entry| !entry.is_empty())
                    .collect();
                let filtered: Vec<String> = current
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|entry| !removals.iter().any(|removal| removal == entry))
                    .collect();
                (!filtered.is_empty()).then_some(filtered)
            }
        };

        match next.clone() {
            Some(entries) => {
                overrides.insert(recipient.to_string(), entries);
            }
            None => {
                overrides.remove(recipient);
            }
        }
        Ok(next)
    }

    fn recipient_mention_policy_status(
        &self,
        recipient: &str,
    ) -> anyhow::Result<RecipientMentionPolicyStatus> {
        let recipient_override = self
            .recipient_require_mention
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(recipient)
            .copied();
        let default_require_mention = *self
            .default_require_mention
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let persist_enabled = *self
            .persist_enabled
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        Ok(RecipientMentionPolicyStatus {
            default_require_mention,
            recipient_override,
            effective_require_mention: recipient_override.unwrap_or(default_require_mention),
            persist_enabled,
        })
    }

    fn set_recipient_require_mention(
        &self,
        recipient: &str,
        require_mention: Option<bool>,
    ) -> anyhow::Result<RecipientMentionPolicyStatus> {
        let mut overrides = self
            .recipient_require_mention
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        match require_mention {
            Some(value) => {
                overrides.insert(recipient.to_string(), value);
            }
            None => {
                overrides.remove(recipient);
            }
        }
        drop(overrides);
        self.recipient_mention_policy_status(recipient)
    }

    async fn send(&self, message: &str, recipient: &str) -> Result<()> {
        self.sent
            .lock()
            .await
            .push((message.to_string(), recipient.to_string()));
        Ok(())
    }

    async fn listen(&self, _tx: mpsc::Sender<ChannelMessage>) -> Result<()> {
        Ok(())
    }
}

pub(super) fn inbound(content: &str) -> ChannelMessage {
    ChannelMessage {
        id: "discord_msg_1".to_string(),
        sender: "1001".to_string(),
        recipient: "2001".to_string(),
        session_key: "3001:2001:1001".to_string(),
        content: content.to_string(),
        attachments: Vec::new(),
        channel: "discord".to_string(),
        timestamp: 0,
    }
}

pub(super) async fn build_agent() -> Result<Arc<Agent>> {
    build_agent_with_inference_url("http://127.0.0.1:1/v1/chat/completions").await
}

pub(super) async fn build_agent_with_inference_url(inference_url: &str) -> Result<Arc<Agent>> {
    let config = AgentConfig {
        inference_url: inference_url.to_string(),
        model: "gpt-4o-mini".to_string(),
        api_key: None,
        max_tool_rounds: 1,
        ..AgentConfig::default()
    };
    Ok(Arc::new(Agent::from_config(config).await?))
}

pub(super) fn inbound_for_session(content: &str, session_key: &str) -> ChannelMessage {
    let mut message = inbound(content);
    message.session_key = session_key.to_string();
    message
}

pub(super) fn start_job_manager(agent: &Arc<Agent>) -> Arc<JobManager> {
    let runner: Arc<dyn TurnRunner> = agent.clone();
    let (manager, _completion_rx) = JobManager::start(runner, JobManagerConfig::default());
    manager
}
