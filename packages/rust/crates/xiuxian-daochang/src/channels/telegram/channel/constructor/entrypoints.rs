use crate::channels::control_command_authorization::ControlCommandPolicy;
use crate::env_parse::read_non_empty_env;

use crate::channels::telegram::TelegramSessionPartition;
use crate::channels::telegram::channel::TELEGRAM_API_BASE_ENV;
use crate::channels::telegram::channel::acl::build_slash_command_policy;
use crate::channels::telegram::channel::constants::TELEGRAM_DEFAULT_API_BASE;
use crate::channels::telegram::channel::state::TelegramChannel;
use crate::channels::telegram::channel::{
    TelegramCommandAdminRule, TelegramControlCommandPolicy, TelegramSlashCommandPolicy,
};

impl TelegramChannel {
    #[must_use]
    fn default_api_base_url() -> String {
        read_non_empty_env(TELEGRAM_API_BASE_ENV)
            .unwrap_or_else(|| TELEGRAM_DEFAULT_API_BASE.to_string())
    }

    /// Create a new Telegram channel.
    #[must_use]
    pub fn new(bot_token: String, allowed_users: Vec<String>, allowed_groups: Vec<String>) -> Self {
        Self::new_with_partition(
            bot_token,
            allowed_users,
            allowed_groups,
            TelegramSessionPartition::from_env(),
        )
    }

    /// Create a new Telegram channel with explicit session partition strategy.
    #[must_use]
    pub fn new_with_partition(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        session_partition: TelegramSessionPartition,
    ) -> Self {
        let admin_users = Vec::new();
        Self::new_with_partition_and_admin_users(
            bot_token,
            allowed_users,
            allowed_groups,
            admin_users,
            session_partition,
        )
    }

    /// Create a new Telegram channel with explicit session partition and admin user allowlist.
    #[must_use]
    pub fn new_with_partition_and_admin_users(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        admin_users: Vec<String>,
        session_partition: TelegramSessionPartition,
    ) -> Self {
        let slash_command_policy =
            build_slash_command_policy(admin_users.clone(), TelegramSlashCommandPolicy::default());
        Self::new_with_base_url_and_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_groups,
            Self::default_api_base_url(),
            ControlCommandPolicy::new(admin_users, None, Vec::new()),
            slash_command_policy,
            session_partition,
        )
    }

    /// Create a new Telegram channel with explicit session partition and structured control-command
    /// authorization policy.
    #[must_use]
    pub fn new_with_partition_and_control_command_policy(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        control_command_policy: TelegramControlCommandPolicy,
        session_partition: TelegramSessionPartition,
    ) -> Self {
        let TelegramControlCommandPolicy {
            admin_users,
            control_command_allow_from,
            control_command_rules,
            slash_command_policy,
        } = control_command_policy;
        let slash_command_policy =
            build_slash_command_policy(admin_users.clone(), slash_command_policy);
        Self::new_with_base_url_and_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_groups,
            Self::default_api_base_url(),
            ControlCommandPolicy::new(
                admin_users,
                control_command_allow_from,
                control_command_rules,
            ),
            slash_command_policy,
            session_partition,
        )
    }

    /// Create a new Telegram channel with explicit session partition, optional control-command
    /// allowlist override, admin user allowlist, and typed per-command admin authorization rules.
    #[must_use]
    pub fn new_with_partition_and_admin_users_and_control_command_allow_from_and_command_rules(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        admin_users: Vec<String>,
        control_command_allow_from: Option<Vec<String>>,
        control_command_rules: Vec<TelegramCommandAdminRule>,
        session_partition: TelegramSessionPartition,
    ) -> Self {
        Self::new_with_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_groups,
            TelegramControlCommandPolicy::new(
                admin_users,
                control_command_allow_from,
                control_command_rules,
            ),
            session_partition,
        )
    }

    #[doc(hidden)]
    #[must_use]
    pub fn expect(self, _message: &str) -> Self {
        self
    }
}
