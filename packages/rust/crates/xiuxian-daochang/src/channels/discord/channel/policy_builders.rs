//! Discord channel policy builders from ACL config.

use crate::channels::control_command_authorization::ControlCommandPolicy;
use crate::channels::control_command_rule_specs::parse_control_command_rule;
use crate::channels::managed_commands::{
    SLASH_SCOPE_BACKGROUND_SUBMIT, SLASH_SCOPE_JOB_STATUS, SLASH_SCOPE_JOBS_SUMMARY,
    SLASH_SCOPE_SESSION_BUDGET, SLASH_SCOPE_SESSION_FEEDBACK, SLASH_SCOPE_SESSION_MEMORY,
    SLASH_SCOPE_SESSION_STATUS,
};

use super::auth::normalize_discord_identity;
use super::policy::{DiscordCommandAdminRule, DiscordSlashCommandPolicy, DiscordSlashCommandRule};

/// Result surface for Discord command policy construction.
pub type DiscordCommandPolicyBuildResult<T> = anyhow::Result<T>;

/// Build one Discord command-admin rule from selectors and allowlist identities.
///
/// # Errors
/// Returns an error when rule selectors or identity entries are invalid.
pub fn build_discord_command_admin_rule(
    selectors: Vec<String>,
    allowed_identities: Vec<String>,
) -> DiscordCommandPolicyBuildResult<DiscordCommandAdminRule> {
    parse_control_command_rule(
        selectors,
        allowed_identities,
        "discord admin command rule",
        normalize_discord_identity,
    )
}

pub(super) fn normalize_allowed_user_entries(entries: Vec<String>) -> Vec<String> {
    entries
        .into_iter()
        .map(|entry| normalize_discord_identity(&entry))
        .filter(|entry| !entry.is_empty())
        .collect()
}

pub(super) fn normalize_allowed_guild_entries(entries: Vec<String>) -> Vec<String> {
    entries
        .into_iter()
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

pub(super) fn build_slash_command_policy(
    admin_users: Vec<String>,
    slash_policy: DiscordSlashCommandPolicy,
) -> ControlCommandPolicy<DiscordSlashCommandRule> {
    let slash_policy = normalize_slash_command_policy(slash_policy);
    let mut rules = Vec::new();

    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_SESSION_STATUS,
        slash_policy.session_status,
        &admin_users,
    );
    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_SESSION_BUDGET,
        slash_policy.session_budget,
        &admin_users,
    );
    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_SESSION_MEMORY,
        slash_policy.session_memory,
        &admin_users,
    );
    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_SESSION_FEEDBACK,
        slash_policy.session_feedback,
        &admin_users,
    );
    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_JOB_STATUS,
        slash_policy.job_status,
        &admin_users,
    );
    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_JOBS_SUMMARY,
        slash_policy.jobs_summary,
        &admin_users,
    );
    add_slash_rule(
        &mut rules,
        SLASH_SCOPE_BACKGROUND_SUBMIT,
        slash_policy.background_submit,
        &admin_users,
    );

    ControlCommandPolicy::new(admin_users, slash_policy.global, rules)
}

fn normalize_optional_allowed_user_entries(entries: Option<Vec<String>>) -> Option<Vec<String>> {
    entries.map(normalize_allowed_user_entries)
}

fn normalize_slash_command_policy(policy: DiscordSlashCommandPolicy) -> DiscordSlashCommandPolicy {
    DiscordSlashCommandPolicy {
        global: normalize_optional_allowed_user_entries(policy.global),
        session_status: normalize_optional_allowed_user_entries(policy.session_status),
        session_budget: normalize_optional_allowed_user_entries(policy.session_budget),
        session_memory: normalize_optional_allowed_user_entries(policy.session_memory),
        session_feedback: normalize_optional_allowed_user_entries(policy.session_feedback),
        job_status: normalize_optional_allowed_user_entries(policy.job_status),
        jobs_summary: normalize_optional_allowed_user_entries(policy.jobs_summary),
        background_submit: normalize_optional_allowed_user_entries(policy.background_submit),
    }
}

fn add_slash_rule(
    rules: &mut Vec<DiscordSlashCommandRule>,
    command_scope: &'static str,
    allow_from: Option<Vec<String>>,
    admin_users: &[String],
) {
    if let Some(mut allowed_identities) = allow_from {
        allowed_identities.extend(admin_users.iter().cloned());
        rules.push(DiscordSlashCommandRule::new(
            command_scope,
            allowed_identities,
        ));
    }
}
