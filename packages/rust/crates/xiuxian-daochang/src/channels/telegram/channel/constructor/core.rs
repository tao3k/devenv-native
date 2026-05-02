use std::sync::RwLock;

use crate::channels::control_command_authorization::ControlCommandPolicy;
use crate::config::runtime_settings_paths;

use crate::channels::telegram::TelegramSessionPartition;
use crate::channels::telegram::channel::acl::{
    normalize_allowed_group_entries, normalize_allowed_user_entries_with_context,
    normalize_control_command_policy, normalize_slash_command_policy,
};
use crate::channels::telegram::channel::acl_reload::TelegramAclReloadState;
use crate::channels::telegram::channel::admin_rules::TelegramCommandAdminRule;
use crate::channels::telegram::channel::group_policy::TelegramGroupPolicyConfig;
use crate::channels::telegram::channel::policy::TelegramSlashCommandRule;
use crate::channels::telegram::channel::send_gate::{
    TelegramSendRateLimitBackend, TelegramSendRateLimitGateState,
};
use crate::channels::telegram::channel::state::TelegramChannel;

pub(super) struct TelegramChannelCoreInit {
    pub(super) bot_token: String,
    pub(super) allowed_users: Vec<String>,
    pub(super) allowed_groups: Vec<String>,
    pub(super) api_base_url: String,
    pub(super) control_command_policy: ControlCommandPolicy<TelegramCommandAdminRule>,
    pub(super) slash_command_policy: ControlCommandPolicy<TelegramSlashCommandRule>,
    pub(super) session_partition: TelegramSessionPartition,
    pub(super) client: reqwest::Client,
}

impl TelegramChannel {
    pub(super) fn new_with_base_url_and_partition_and_client_impl(
        init: TelegramChannelCoreInit,
    ) -> Self {
        let TelegramChannelCoreInit {
            bot_token,
            allowed_users,
            allowed_groups,
            api_base_url,
            control_command_policy,
            slash_command_policy,
            session_partition,
            client,
        } = init;

        let (system_settings_path, user_settings_path) = runtime_settings_paths();
        let control_command_policy = normalize_control_command_policy(control_command_policy);
        let slash_command_policy = normalize_slash_command_policy(slash_command_policy);
        Self {
            bot_token,
            api_base_url,
            allowed_users: RwLock::new(normalize_allowed_user_entries_with_context(
                allowed_users,
                "telegram.acl.allow.users",
            )),
            allowed_groups: RwLock::new(normalize_allowed_group_entries(allowed_groups)),
            control_command_policy: RwLock::new(control_command_policy),
            slash_command_policy: RwLock::new(slash_command_policy),
            group_policy_config: RwLock::new(TelegramGroupPolicyConfig::default()),
            session_admin_persist: RwLock::new(false),
            session_partition: RwLock::new(session_partition),
            acl_reload_state: RwLock::new(TelegramAclReloadState::new(
                system_settings_path,
                user_settings_path,
            )),
            send_rate_limit_gate: tokio::sync::Mutex::new(TelegramSendRateLimitGateState::default()),
            send_rate_limit_backend: TelegramSendRateLimitBackend::from_env(),
            client,
        }
    }

    #[doc(hidden)]
    pub fn new_with_base_url_and_send_rate_limit_valkey_for_test(
        bot_token: String,
        allowed_users: Vec<String>,
        allowed_groups: Vec<String>,
        api_base_url: String,
        redis_url: &str,
        key_prefix: &str,
    ) -> anyhow::Result<Self> {
        let mut channel =
            Self::new_with_base_url(bot_token, allowed_users, allowed_groups, api_base_url);
        channel.send_rate_limit_backend =
            TelegramSendRateLimitBackend::new_valkey_for_test(redis_url, key_prefix)?;
        Ok(channel)
    }
}
