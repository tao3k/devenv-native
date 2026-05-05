//! Telegram administrator rule parsing from channel configuration.

use crate::channels::control_command_rule_specs::{
    CommandSelectorAuthRule, parse_control_command_rule,
};
use anyhow::{Context, Result};

use crate::channels::telegram::channel::identity::normalize_user_identity;

/// Parsed Telegram command-admin rule for control-command authorization.
pub type TelegramCommandAdminRule = CommandSelectorAuthRule;

/// Build one Telegram command-admin rule from selectors and allowed users.
///
/// # Errors
/// Returns an error when selectors or users are invalid.
pub fn build_telegram_command_admin_rule(
    selectors: Vec<String>,
    allowed_users: Vec<String>,
) -> Result<TelegramCommandAdminRule> {
    parse_control_command_rule(
        selectors,
        allowed_users,
        "admin command rule",
        normalize_user_identity,
    )
}

pub(in crate::channels::telegram::channel) fn parse_admin_command_rule_specs(
    specs: Vec<String>,
) -> Result<Vec<TelegramCommandAdminRule>> {
    specs
        .into_iter()
        .map(|spec| parse_admin_command_rule_spec(spec.as_str()))
        .collect()
}

fn parse_admin_command_rule_spec(spec: &str) -> Result<TelegramCommandAdminRule> {
    let (selectors_raw, principals_raw) = spec
        .split_once("=>")
        .with_context(|| "invalid admin command rule; expected selectors=>allowed_users")?;
    let selectors = selectors_raw
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToString::to_string)
        .collect();
    let allowed_users = principals_raw
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToString::to_string)
        .collect();
    build_telegram_command_admin_rule(selectors, allowed_users)
}
