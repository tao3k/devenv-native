use async_trait::async_trait;
use std::sync::Arc;

use crate::channels::traits::{
    Channel, ChannelMessage, RecipientCommandAdminUsersMutation, RecipientMentionPolicyStatus,
};

use super::auth::normalize_discord_identity;
use super::state::DiscordChannel;
use crate::channels::discord::session_partition::DiscordSessionPartition;

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &'static str {
        "discord"
    }

    fn session_partition_mode(&self) -> Option<String> {
        Some(self.session_partition().to_string())
    }

    fn set_session_partition_mode(&self, mode: &str) -> anyhow::Result<()> {
        let parsed = mode
            .parse::<DiscordSessionPartition>()
            .map_err(|_| anyhow::anyhow!("invalid discord session partition mode: {mode}"))?;
        self.set_session_partition(parsed);
        Ok(())
    }

    fn is_admin_user(&self, identity: &str) -> bool {
        let normalized = normalize_discord_identity(identity);
        self.control_command_policy
            .admin_users
            .iter()
            .any(|entry| entry == "*" || entry == &normalized)
    }

    fn is_authorized_for_control_command(&self, identity: &str, command_text: &str) -> bool {
        self.authorize_control_command(identity, command_text)
    }

    fn is_authorized_for_control_command_for_recipient(
        &self,
        identity: &str,
        command_text: &str,
        recipient: &str,
    ) -> bool {
        self.authorize_control_command_for_recipient(identity, command_text, recipient)
    }

    fn is_authorized_for_slash_command(&self, identity: &str, command_scope: &str) -> bool {
        self.authorize_slash_command(identity, command_scope)
    }

    fn is_authorized_for_slash_command_for_recipient(
        &self,
        identity: &str,
        command_scope: &str,
        recipient: &str,
    ) -> bool {
        self.authorize_slash_command_for_recipient(identity, command_scope, recipient)
    }

    fn recipient_command_admin_users(
        &self,
        recipient: &str,
    ) -> anyhow::Result<Option<Vec<String>>> {
        self.recipient_override_admin_users(recipient)
    }

    fn mutate_recipient_command_admin_users(
        &self,
        recipient: &str,
        mutation: RecipientCommandAdminUsersMutation,
    ) -> anyhow::Result<Option<Vec<String>>> {
        self.mutate_recipient_override_admin_users(recipient, mutation)
    }

    fn recipient_mention_policy_status(
        &self,
        recipient: &str,
    ) -> anyhow::Result<RecipientMentionPolicyStatus> {
        DiscordChannel::recipient_mention_policy_status(self, recipient)
    }

    fn set_recipient_require_mention(
        &self,
        recipient: &str,
        require_mention: Option<bool>,
    ) -> anyhow::Result<RecipientMentionPolicyStatus> {
        DiscordChannel::set_recipient_require_mention(self, recipient, require_mention)
    }

    async fn send(&self, message: &str, recipient: &str) -> anyhow::Result<()> {
        self.send_text(message, recipient).await
    }

    async fn listen(&self, tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> anyhow::Result<()> {
        super::super::runtime::run_discord_gateway_listener(Arc::new(self.clone()), tx).await
    }

    async fn start_typing(&self, recipient: &str) -> anyhow::Result<()> {
        self.start_typing_indicator(recipient).await
    }

    async fn health_check(&self) -> bool {
        false
    }
}
