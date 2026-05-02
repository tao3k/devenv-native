use std::collections::HashMap;

use crate::config::settings::{
    DiscordAclAllowSettings, DiscordAclControlSettings, DiscordAclPrincipalSettings,
    DiscordAclSettings, DiscordAclSlashSettings, DiscordChannelSettings, DiscordSettings,
};

impl DiscordSettings {
    pub(super) fn merge(self, overlay: Self) -> Self {
        Self {
            acl: self.acl.merge(overlay.acl),
            runtime_mode: overlay.runtime_mode.or(self.runtime_mode),
            require_mention: overlay.require_mention.or(self.require_mention),
            require_mention_persist: overlay
                .require_mention_persist
                .or(self.require_mention_persist),
            ingress_bind: overlay.ingress_bind.or(self.ingress_bind),
            ingress_path: overlay.ingress_path.or(self.ingress_path),
            ingress_secret_token: overlay.ingress_secret_token.or(self.ingress_secret_token),
            channels: merge_discord_channel_map(self.channels, overlay.channels),
            session_partition: overlay.session_partition.or(self.session_partition),
            session_partition_persist: overlay
                .session_partition_persist
                .or(self.session_partition_persist),
            inbound_queue_capacity: overlay
                .inbound_queue_capacity
                .or(self.inbound_queue_capacity),
            turn_timeout_secs: overlay.turn_timeout_secs.or(self.turn_timeout_secs),
            foreground_max_in_flight_messages: overlay
                .foreground_max_in_flight_messages
                .or(self.foreground_max_in_flight_messages),
            foreground_queue_mode: overlay.foreground_queue_mode.or(self.foreground_queue_mode),
        }
    }
}

impl DiscordAclSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            role_aliases: merge_string_map(self.role_aliases, overlay.role_aliases),
            allow: merge_option_discord_allow_settings(self.allow, overlay.allow),
            admin: merge_option_discord_principal_settings(self.admin, overlay.admin),
            control: merge_option_discord_control_settings(self.control, overlay.control),
            slash: merge_option_discord_slash_settings(self.slash, overlay.slash),
        }
    }
}

impl DiscordChannelSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            require_mention: overlay.require_mention.or(self.require_mention),
        }
    }
}

impl DiscordAclAllowSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            users: overlay.users.or(self.users),
            roles: overlay.roles.or(self.roles),
            guilds: overlay.guilds.or(self.guilds),
        }
    }
}

impl DiscordAclPrincipalSettings {
    fn merge(_base: Self, overlay: Self) -> Self {
        overlay
    }
}

impl DiscordAclControlSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            allow_from: merge_option_discord_principal_settings(
                self.allow_from,
                overlay.allow_from,
            ),
            rules: overlay.rules.or(self.rules),
        }
    }
}

impl DiscordAclSlashSettings {
    fn merge(self, overlay: Self) -> Self {
        Self {
            global: merge_option_discord_principal_settings(self.global, overlay.global),
            session_status: merge_option_discord_principal_settings(
                self.session_status,
                overlay.session_status,
            ),
            session_budget: merge_option_discord_principal_settings(
                self.session_budget,
                overlay.session_budget,
            ),
            session_memory: merge_option_discord_principal_settings(
                self.session_memory,
                overlay.session_memory,
            ),
            session_feedback: merge_option_discord_principal_settings(
                self.session_feedback,
                overlay.session_feedback,
            ),
            job_status: merge_option_discord_principal_settings(
                self.job_status,
                overlay.job_status,
            ),
            jobs_summary: merge_option_discord_principal_settings(
                self.jobs_summary,
                overlay.jobs_summary,
            ),
            background_submit: merge_option_discord_principal_settings(
                self.background_submit,
                overlay.background_submit,
            ),
        }
    }
}

fn merge_option_discord_allow_settings(
    base: Option<DiscordAclAllowSettings>,
    overlay: Option<DiscordAclAllowSettings>,
) -> Option<DiscordAclAllowSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => {
            Some(base_settings.merge(overlay_settings))
        }
    }
}

fn merge_option_discord_principal_settings(
    base: Option<DiscordAclPrincipalSettings>,
    overlay: Option<DiscordAclPrincipalSettings>,
) -> Option<DiscordAclPrincipalSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => Some(DiscordAclPrincipalSettings::merge(
            base_settings,
            overlay_settings,
        )),
    }
}

fn merge_option_discord_control_settings(
    base: Option<DiscordAclControlSettings>,
    overlay: Option<DiscordAclControlSettings>,
) -> Option<DiscordAclControlSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => {
            Some(base_settings.merge(overlay_settings))
        }
    }
}

fn merge_option_discord_slash_settings(
    base: Option<DiscordAclSlashSettings>,
    overlay: Option<DiscordAclSlashSettings>,
) -> Option<DiscordAclSlashSettings> {
    match (base, overlay) {
        (None, None) => None,
        (Some(settings), None) | (None, Some(settings)) => Some(settings),
        (Some(base_settings), Some(overlay_settings)) => {
            Some(base_settings.merge(overlay_settings))
        }
    }
}

fn merge_string_map(
    base: Option<HashMap<String, String>>,
    overlay: Option<HashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    match (base, overlay) {
        (None, None) => None,
        (Some(values), None) | (None, Some(values)) => Some(values),
        (Some(mut base_values), Some(overlay_values)) => {
            for (key, value) in overlay_values {
                base_values.insert(key, value);
            }
            Some(base_values)
        }
    }
}

fn merge_discord_channel_map(
    base: Option<HashMap<String, DiscordChannelSettings>>,
    overlay: Option<HashMap<String, DiscordChannelSettings>>,
) -> Option<HashMap<String, DiscordChannelSettings>> {
    match (base, overlay) {
        (None, None) => None,
        (Some(values), None) | (None, Some(values)) => Some(values),
        (Some(mut base_values), Some(overlay_values)) => {
            for (key, value) in overlay_values {
                match base_values.remove(&key) {
                    Some(base_value) => {
                        base_values.insert(key, base_value.merge(value));
                    }
                    None => {
                        base_values.insert(key, value);
                    }
                }
            }
            Some(base_values)
        }
    }
}
