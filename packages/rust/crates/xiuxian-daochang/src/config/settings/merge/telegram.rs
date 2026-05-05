use std::collections::HashMap;

use crate::config::settings::{
    TelegramAclAllowSettings, TelegramAclControlSettings, TelegramAclPrincipalSettings,
    TelegramAclSettings, TelegramAclSlashSettings, TelegramGroupSettings, TelegramSettings,
    TelegramTopicSettings,
};

impl TelegramSettings {
    pub(super) fn merge(self, overlay: Self) -> Self {
        Self {
            acl: self.acl.merge(overlay.acl),
            session_admin_persist: overlay.session_admin_persist.or(self.session_admin_persist),
            session_partition_persist: overlay
                .session_partition_persist
                .or(self.session_partition_persist),
            group_policy: overlay.group_policy.or(self.group_policy),
            group_allow_from: overlay.group_allow_from.or(self.group_allow_from),
            require_mention: overlay.require_mention.or(self.require_mention),
            groups: merge_telegram_groups(self.groups, overlay.groups),
            mode: overlay.mode.or(self.mode),
            webhook_bind: overlay.webhook_bind.or(self.webhook_bind),
            webhook_path: overlay.webhook_path.or(self.webhook_path),
            webhook_dedup_backend: overlay.webhook_dedup_backend.or(self.webhook_dedup_backend),
            webhook_dedup_ttl_secs: overlay
                .webhook_dedup_ttl_secs
                .or(self.webhook_dedup_ttl_secs),
            webhook_dedup_key_prefix: overlay
                .webhook_dedup_key_prefix
                .or(self.webhook_dedup_key_prefix),
            max_tool_rounds: overlay.max_tool_rounds.or(self.max_tool_rounds),
            session_partition: overlay.session_partition.or(self.session_partition),
            inbound_queue_capacity: overlay
                .inbound_queue_capacity
                .or(self.inbound_queue_capacity),
            foreground_queue_capacity: overlay
                .foreground_queue_capacity
                .or(self.foreground_queue_capacity),
            foreground_max_in_flight_messages: overlay
                .foreground_max_in_flight_messages
                .or(self.foreground_max_in_flight_messages),
            foreground_turn_timeout_secs: overlay
                .foreground_turn_timeout_secs
                .or(self.foreground_turn_timeout_secs),
            foreground_queue_mode: overlay.foreground_queue_mode.or(self.foreground_queue_mode),
            foreground_session_gate_backend: overlay
                .foreground_session_gate_backend
                .or(self.foreground_session_gate_backend),
            foreground_session_gate_key_prefix: overlay
                .foreground_session_gate_key_prefix
                .or(self.foreground_session_gate_key_prefix),
            foreground_session_gate_lease_ttl_secs: overlay
                .foreground_session_gate_lease_ttl_secs
                .or(self.foreground_session_gate_lease_ttl_secs),
            foreground_session_gate_acquire_timeout_secs: overlay
                .foreground_session_gate_acquire_timeout_secs
                .or(self.foreground_session_gate_acquire_timeout_secs),
            send_rate_limit_gate_key_prefix: overlay
                .send_rate_limit_gate_key_prefix
                .or(self.send_rate_limit_gate_key_prefix),
        }
    }
}

impl TelegramAclSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            allow: merge_option_telegram_allow_settings(self.allow, overlay.allow),
            admin: merge_option_telegram_principal_settings(self.admin, overlay.admin),
            control: merge_option_telegram_control_settings(self.control, overlay.control),
            slash: merge_option_telegram_slash_settings(self.slash, overlay.slash),
        }
    }
}

impl TelegramAclAllowSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            users: overlay.users.or(self.users),
            groups: overlay.groups.or(self.groups),
        }
    }
}

impl TelegramAclPrincipalSettings {
    fn merge(_base: Self, overlay: Self) -> Self {
        overlay
    }
}

impl TelegramAclControlSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            allow_from: merge_option_telegram_principal_settings(
                self.allow_from,
                overlay.allow_from,
            ),
            rules: overlay.rules.or(self.rules),
        }
    }
}

impl TelegramAclSlashSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            global: merge_option_telegram_principal_settings(self.global, overlay.global),
            session_status: merge_option_telegram_principal_settings(
                self.session_status,
                overlay.session_status,
            ),
            session_budget: merge_option_telegram_principal_settings(
                self.session_budget,
                overlay.session_budget,
            ),
            session_memory: merge_option_telegram_principal_settings(
                self.session_memory,
                overlay.session_memory,
            ),
            session_feedback: merge_option_telegram_principal_settings(
                self.session_feedback,
                overlay.session_feedback,
            ),
            job_status: merge_option_telegram_principal_settings(
                self.job_status,
                overlay.job_status,
            ),
            jobs_summary: merge_option_telegram_principal_settings(
                self.jobs_summary,
                overlay.jobs_summary,
            ),
            background_submit: merge_option_telegram_principal_settings(
                self.background_submit,
                overlay.background_submit,
            ),
        }
    }
}

fn merge_option_telegram_allow_settings(
    base: Option<TelegramAclAllowSettings>,
    overlay: Option<TelegramAclAllowSettings>,
) -> Option<TelegramAclAllowSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => {
            Some(base_settings.merge(overlay_settings))
        }
    }
}

fn merge_option_telegram_principal_settings(
    base: Option<TelegramAclPrincipalSettings>,
    overlay: Option<TelegramAclPrincipalSettings>,
) -> Option<TelegramAclPrincipalSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => Some(TelegramAclPrincipalSettings::merge(
            base_settings,
            overlay_settings,
        )),
    }
}

fn merge_option_telegram_control_settings(
    base: Option<TelegramAclControlSettings>,
    overlay: Option<TelegramAclControlSettings>,
) -> Option<TelegramAclControlSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => {
            Some(base_settings.merge(overlay_settings))
        }
    }
}

fn merge_option_telegram_slash_settings(
    base: Option<TelegramAclSlashSettings>,
    overlay: Option<TelegramAclSlashSettings>,
) -> Option<TelegramAclSlashSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => {
            Some(base_settings.merge(overlay_settings))
        }
    }
}

impl TelegramGroupSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            enabled: overlay.enabled.or(self.enabled),
            group_policy: overlay.group_policy.or(self.group_policy),
            allow_from: merge_option_telegram_principal_settings(
                self.allow_from,
                overlay.allow_from,
            ),
            admin_users: merge_option_telegram_principal_settings(
                self.admin_users,
                overlay.admin_users,
            ),
            require_mention: overlay.require_mention.or(self.require_mention),
            topics: merge_telegram_topics(self.topics, overlay.topics),
        }
    }
}

impl TelegramTopicSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            enabled: overlay.enabled.or(self.enabled),
            group_policy: overlay.group_policy.or(self.group_policy),
            allow_from: merge_option_telegram_principal_settings(
                self.allow_from,
                overlay.allow_from,
            ),
            admin_users: merge_option_telegram_principal_settings(
                self.admin_users,
                overlay.admin_users,
            ),
            require_mention: overlay.require_mention.or(self.require_mention),
        }
    }
}

fn merge_telegram_groups(
    base: Option<HashMap<String, TelegramGroupSettings>>,
    overlay: Option<HashMap<String, TelegramGroupSettings>>,
) -> Option<HashMap<String, TelegramGroupSettings>> {
    match (base, overlay) {
        (None, None) => None,
        (Some(groups), None) | (None, Some(groups)) => Some(groups),
        (Some(mut groups), Some(overlay_groups)) => {
            for (group_id, override_group) in overlay_groups {
                groups
                    .entry(group_id)
                    .and_modify(|existing| {
                        *existing = existing.clone().merge(override_group.clone());
                    })
                    .or_insert(override_group);
            }
            Some(groups)
        }
    }
}

fn merge_telegram_topics(
    base: Option<HashMap<String, TelegramTopicSettings>>,
    overlay: Option<HashMap<String, TelegramTopicSettings>>,
) -> Option<HashMap<String, TelegramTopicSettings>> {
    match (base, overlay) {
        (None, None) => None,
        (Some(topics), None) | (None, Some(topics)) => Some(topics),
        (Some(mut topics), Some(overlay_topics)) => {
            for (topic_id, override_topic) in overlay_topics {
                topics
                    .entry(topic_id)
                    .and_modify(|existing| {
                        *existing = existing.clone().merge(override_topic.clone());
                    })
                    .or_insert(override_topic);
            }
            Some(topics)
        }
    }
}
