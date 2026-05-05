use crate::channels::control_command_authorization::ControlCommandPolicy;

use super::core::TelegramChannelCoreInit;
use crate::channels::telegram::TelegramSessionPartition;
use crate::channels::telegram::channel::acl::build_slash_command_policy;
use crate::channels::telegram::channel::state::TelegramChannel;
use crate::channels::telegram::channel::{
    TelegramSlashCommandPolicy, policy::TelegramSlashCommandRule,
};

impl TelegramChannel {
    /// Create a Telegram channel with a custom API base URL (useful for tests/proxies).
    #[must_use]
    pub fn new_with_base_url(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        api_base_url: String,
    ) -> Self {
        let admin_users = Vec::new();
        Self::new_with_base_url_and_partition(
            bot_token,
            allowed_users,
            allowed_groups,
            api_base_url,
            admin_users,
            TelegramSessionPartition::from_env(),
        )
    }

    /// Create a Telegram channel with custom API base URL and explicit session partition.
    #[must_use]
    pub fn new_with_base_url_and_partition(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        api_base_url: String,
        admin_users: Vec<String>,
        session_partition: TelegramSessionPartition,
    ) -> Self {
        let slash_command_policy =
            build_slash_command_policy(admin_users.clone(), TelegramSlashCommandPolicy::default());
        Self::new_with_base_url_and_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_groups,
            api_base_url,
            ControlCommandPolicy::new(admin_users, None, Vec::new()),
            slash_command_policy,
            session_partition,
        )
    }

    /// Create a Telegram channel with custom API base URL, explicit session partition, and HTTP client.
    #[doc(hidden)]
    #[must_use]
    pub fn new_with_base_url_and_partition_and_client(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        api_base_url: String,
        admin_users: Vec<String>,
        session_partition: TelegramSessionPartition,
        client: reqwest::Client,
    ) -> Self {
        let slash_command_policy =
            build_slash_command_policy(admin_users.clone(), TelegramSlashCommandPolicy::default());
        Self::new_with_base_url_and_partition_and_client_impl(TelegramChannelCoreInit {
            bot_token,
            allowed_users,
            allowed_groups,
            api_base_url,
            control_command_policy: ControlCommandPolicy::new(admin_users, None, Vec::new()),
            slash_command_policy,
            session_partition,
            client,
        })
    }

    pub(super) fn new_with_base_url_and_partition_and_control_command_policy(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        api_base_url: String,
        control_command_policy: ControlCommandPolicy<
            crate::channels::telegram::channel::admin_rules::TelegramCommandAdminRule,
        >,
        slash_command_policy: ControlCommandPolicy<TelegramSlashCommandRule>,
        session_partition: TelegramSessionPartition,
    ) -> Self {
        Self::new_with_base_url_and_partition_and_client_impl(TelegramChannelCoreInit {
            bot_token,
            allowed_users,
            allowed_groups,
            api_base_url,
            control_command_policy,
            slash_command_policy,
            session_partition,
            client: crate::channels::telegram::channel::client::build_telegram_http_client(),
        })
    }
}
