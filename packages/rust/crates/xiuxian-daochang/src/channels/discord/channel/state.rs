//! Discord channel session state and interrupt bookkeeping.

use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

use crate::channels::control_command_authorization::ControlCommandPolicy;

use super::policy::{DiscordCommandAdminRule, DiscordSlashCommandRule};
use crate::channels::discord::session_partition::DiscordSessionPartition;

/// Discord channel transport state and ACL policy.
pub struct DiscordChannel {
    pub(in crate::channels::discord) bot_token: String,
    pub(super) api_base_url: String,
    pub(in crate::channels::discord) allowed_users: Vec<String>,
    pub(in crate::channels::discord) allowed_guilds: Vec<String>,
    pub(super) control_command_policy: ControlCommandPolicy<DiscordCommandAdminRule>,
    pub(super) slash_command_policy: ControlCommandPolicy<DiscordSlashCommandRule>,
    pub(super) session_partition: RwLock<DiscordSessionPartition>,
    pub(super) recipient_admin_users: RwLock<HashMap<String, Vec<String>>>,
    pub(super) sender_acl_identities: RwLock<HashMap<String, Vec<String>>>,
    pub(super) bot_user_id: RwLock<Option<String>>,
    pub(super) default_require_mention: RwLock<bool>,
    pub(super) require_mention_persist: RwLock<bool>,
    pub(super) recipient_require_mention: RwLock<HashMap<String, bool>>,
    pub(in crate::channels::discord) client: reqwest::Client,
}

impl DiscordChannel {
    /// Current session partition mode used by this channel.
    pub fn session_partition(&self) -> DiscordSessionPartition {
        *self
            .session_partition
            .read()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Update session partition mode at runtime.
    pub fn set_session_partition(&self, mode: DiscordSessionPartition) {
        *self
            .session_partition
            .write()
            .unwrap_or_else(PoisonError::into_inner) = mode;
    }
}

impl Clone for DiscordChannel {
    fn clone(&self) -> Self {
        let session_partition = *self
            .session_partition
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let recipient_admin_users = self
            .recipient_admin_users
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let sender_acl_identities = self
            .sender_acl_identities
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let bot_user_id = self
            .bot_user_id
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let default_require_mention = *self
            .default_require_mention
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let require_mention_persist = *self
            .require_mention_persist
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let recipient_require_mention = self
            .recipient_require_mention
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        Self {
            bot_token: self.bot_token.clone(),
            api_base_url: self.api_base_url.clone(),
            allowed_users: self.allowed_users.clone(),
            allowed_guilds: self.allowed_guilds.clone(),
            control_command_policy: self.control_command_policy.clone(),
            slash_command_policy: self.slash_command_policy.clone(),
            session_partition: RwLock::new(session_partition),
            recipient_admin_users: RwLock::new(recipient_admin_users),
            sender_acl_identities: RwLock::new(sender_acl_identities),
            bot_user_id: RwLock::new(bot_user_id),
            default_require_mention: RwLock::new(default_require_mention),
            require_mention_persist: RwLock::new(require_mention_persist),
            recipient_require_mention: RwLock::new(recipient_require_mention),
            client: self.client.clone(),
        }
    }
}
