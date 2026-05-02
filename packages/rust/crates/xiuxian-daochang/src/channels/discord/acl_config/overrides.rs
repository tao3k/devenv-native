//! Discord ACL override builder.

use crate::config::RuntimeSettings;

use super::control_rules::control_rules;
use super::principals::{collect_principals, guilds_list_from_allow, principal_list_from_allow};
use super::role_aliases::normalize_role_aliases;
use super::slash::slash_overrides;
use crate::channels::discord::channel::DiscordCommandAdminRule;

/// Runtime ACL overrides derived from the Discord configuration surface.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscordAclOverrides {
    /// Explicitly allowed Discord user IDs after principal expansion.
    pub allowed_users: Vec<String>,
    /// Explicitly allowed Discord guild IDs after principal expansion.
    pub allowed_guilds: Vec<String>,
    /// Optional global admin principal list after alias expansion.
    pub admin_users: Option<Vec<String>>,
    /// Optional global allow-list for privileged text control commands.
    pub control_command_allow_from: Option<Vec<String>>,
    /// Per-command control ACL rules compiled for runtime checks.
    pub control_command_rules: Vec<DiscordCommandAdminRule>,
    /// Optional allow-list for all slash commands.
    pub slash_command_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/session status`.
    pub slash_session_status_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/session budget`.
    pub slash_session_budget_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/session memory`.
    pub slash_session_memory_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/session feedback`.
    pub slash_session_feedback_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/job`.
    pub slash_job_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/jobs`.
    pub slash_jobs_allow_from: Option<Vec<String>>,
    /// Optional allow-list for `/bg`.
    pub slash_bg_allow_from: Option<Vec<String>>,
}

/// Build Discord runtime ACL overrides from settings.
///
/// Normalizes configured principals, role aliases, and slash-command overrides
/// into one runtime structure used by Discord authorization checks.
///
/// # Errors
///
/// Returns an error when ACL command-rule parsing fails.
pub fn build_discord_acl_overrides(
    settings: &RuntimeSettings,
) -> anyhow::Result<DiscordAclOverrides> {
    let acl = &settings.discord.acl;
    let role_aliases = normalize_role_aliases(acl);

    let allowed_users = acl
        .allow
        .as_ref()
        .and_then(|allow| principal_list_from_allow(allow, &role_aliases))
        .unwrap_or_default();
    let allowed_guilds = acl
        .allow
        .as_ref()
        .and_then(guilds_list_from_allow)
        .unwrap_or_default();
    let admin_users = acl
        .admin
        .as_ref()
        .and_then(|principal| collect_principals(principal, &role_aliases));
    let control_command_allow_from = acl
        .control
        .as_ref()
        .and_then(|control| control.allow_from.as_ref())
        .and_then(|allow_from| collect_principals(allow_from, &role_aliases));
    let control_command_rules = acl
        .control
        .as_ref()
        .map(|control| control_rules(control, &role_aliases))
        .transpose()?
        .unwrap_or_default();

    let slash_overrides = slash_overrides(acl.slash.as_ref(), &role_aliases);

    Ok(DiscordAclOverrides {
        allowed_users,
        allowed_guilds,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        slash_command_allow_from: slash_overrides.command,
        slash_session_status_allow_from: slash_overrides.session_status,
        slash_session_budget_allow_from: slash_overrides.session_budget,
        slash_session_memory_allow_from: slash_overrides.session_memory,
        slash_session_feedback_allow_from: slash_overrides.session_feedback,
        slash_job_allow_from: slash_overrides.job_status,
        slash_jobs_allow_from: slash_overrides.jobs_summary,
        slash_bg_allow_from: slash_overrides.background_submit,
    })
}
