//! Telegram ACL override parsing for control-command authorization.

use anyhow::Result;

use crate::config::{
    RuntimeSettings, TelegramAclControlSettings, TelegramAclRuleSettings, TelegramSettings,
};

use super::TelegramCommandAdminRule;
use super::acl::{
    normalize_allowed_group_entries, normalize_allowed_user_entries_with_context,
    normalize_optional_allowed_user_entries_with_context,
};
use super::admin_rules::build_telegram_command_admin_rule;

const TELEGRAM_ACL_FIELD_ALLOWED_USERS: &str = "telegram.acl.allow.users";
const TELEGRAM_ACL_FIELD_ADMIN_USERS: &str = "telegram.acl.admin.users";
const TELEGRAM_ACL_FIELD_CONTROL_COMMAND_ALLOW_FROM: &str = "telegram.acl.control.allow_from.users";
const TELEGRAM_ACL_FIELD_SLASH_COMMAND_ALLOW_FROM: &str = "telegram.acl.slash.global.users";
const TELEGRAM_ACL_FIELD_SLASH_SESSION_STATUS_ALLOW_FROM: &str =
    "telegram.acl.slash.session_status.users";
const TELEGRAM_ACL_FIELD_SLASH_SESSION_BUDGET_ALLOW_FROM: &str =
    "telegram.acl.slash.session_budget.users";
const TELEGRAM_ACL_FIELD_SLASH_SESSION_MEMORY_ALLOW_FROM: &str =
    "telegram.acl.slash.session_memory.users";
const TELEGRAM_ACL_FIELD_SLASH_SESSION_FEEDBACK_ALLOW_FROM: &str =
    "telegram.acl.slash.session_feedback.users";
const TELEGRAM_ACL_FIELD_SLASH_JOB_ALLOW_FROM: &str = "telegram.acl.slash.job_status.users";
const TELEGRAM_ACL_FIELD_SLASH_JOBS_ALLOW_FROM: &str = "telegram.acl.slash.jobs_summary.users";
const TELEGRAM_ACL_FIELD_SLASH_BG_ALLOW_FROM: &str = "telegram.acl.slash.background_submit.users";

/// Runtime ACL overrides derived from Telegram configuration.
#[derive(Debug, Clone, Default)]
pub struct TelegramAclOverrides {
    /// Explicitly allowed Telegram user identifiers.
    pub allowed_users: Vec<String>,
    /// Explicitly allowed Telegram group or chat identifiers.
    pub allowed_groups: Vec<String>,
    /// Telegram users allowed to execute privileged admin commands.
    pub admin_users: Vec<String>,
    /// Optional allow-list for privileged text control commands.
    pub control_command_allow_from: Option<Vec<String>>,
    /// Per-command admin rules compiled for runtime checks.
    pub control_command_rules: Vec<TelegramCommandAdminRule>,
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

/// Builds Telegram ACL runtime overrides from the root runtime settings.
///
/// # Errors
///
/// Returns an error when control-command rule parsing fails.
pub fn build_telegram_acl_overrides(settings: &RuntimeSettings) -> Result<TelegramAclOverrides> {
    build_telegram_acl_overrides_from_settings(&settings.telegram)
}

/// Builds Telegram ACL runtime overrides from Telegram-only settings.
///
/// # Errors
///
/// Returns an error when control-command rule parsing fails.
pub fn build_telegram_acl_overrides_from_settings(
    settings: &TelegramSettings,
) -> Result<TelegramAclOverrides> {
    let allow = settings.acl.allow.as_ref();
    let admin = settings.acl.admin.as_ref();
    let control = settings.acl.control.as_ref();
    let slash = settings.acl.slash.as_ref();

    Ok(TelegramAclOverrides {
        allowed_users: normalize_allowed_user_entries_with_context(
            allow.and_then(|acl| acl.users.clone()).unwrap_or_default(),
            TELEGRAM_ACL_FIELD_ALLOWED_USERS,
        ),
        allowed_groups: normalize_allowed_group_entries(
            allow.and_then(|acl| acl.groups.clone()).unwrap_or_default(),
        ),
        admin_users: normalize_allowed_user_entries_with_context(
            admin.and_then(|acl| acl.users.clone()).unwrap_or_default(),
            TELEGRAM_ACL_FIELD_ADMIN_USERS,
        ),
        control_command_allow_from: normalize_optional_allowed_user_entries_with_context(
            control
                .and_then(|acl| acl.allow_from.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_CONTROL_COMMAND_ALLOW_FROM,
        ),
        control_command_rules: build_control_command_rules(control)?,
        slash_command_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.global.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_COMMAND_ALLOW_FROM,
        ),
        slash_session_status_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.session_status.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_SESSION_STATUS_ALLOW_FROM,
        ),
        slash_session_budget_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.session_budget.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_SESSION_BUDGET_ALLOW_FROM,
        ),
        slash_session_memory_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.session_memory.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_SESSION_MEMORY_ALLOW_FROM,
        ),
        slash_session_feedback_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.session_feedback.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_SESSION_FEEDBACK_ALLOW_FROM,
        ),
        slash_job_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.job_status.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_JOB_ALLOW_FROM,
        ),
        slash_jobs_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.jobs_summary.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_JOBS_ALLOW_FROM,
        ),
        slash_bg_allow_from: normalize_optional_allowed_user_entries_with_context(
            slash
                .and_then(|acl| acl.background_submit.as_ref())
                .and_then(|principals| principals.users.clone()),
            TELEGRAM_ACL_FIELD_SLASH_BG_ALLOW_FROM,
        ),
    })
}

fn build_control_command_rules(
    control: Option<&TelegramAclControlSettings>,
) -> Result<Vec<TelegramCommandAdminRule>> {
    control.and_then(|acl| acl.rules.as_ref()).map_or_else(
        || Ok(Vec::new()),
        |rules| {
            rules
                .iter()
                .enumerate()
                .map(build_control_command_rule)
                .collect()
        },
    )
}

fn build_control_command_rule(
    (index, rule): (usize, &TelegramAclRuleSettings),
) -> Result<TelegramCommandAdminRule> {
    build_telegram_command_admin_rule(
        rule.commands.clone(),
        rule.allow.users.clone().unwrap_or_default(),
    )
    .map_err(|error| anyhow::anyhow!("telegram.acl.control.rules[{index}].commands: {error}"))
}
