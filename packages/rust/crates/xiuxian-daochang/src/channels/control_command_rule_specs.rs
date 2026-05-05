//! Shared parser and matcher for structured control-command rules.
//!
//! Rules are parsed from explicit selector and principal lists:
//! - selectors: `commands[]`
//! - principals: `allow.users[]` / `allow.roles[]`
//!
//! Supported selectors (left-hand side):
//! - Exact command path: `/session partition`, `session.partition`, `/resume drop`
//! - Group wildcard: `session.*` (matches `session.partition`, `session.reset`, ...)
//! - Global wildcard: `*`
//!
//! Notes:
//! - Matching is case-insensitive.
//! - Command path normalization strips leading `/` and optional `@bot` suffix.
//! - Control command matching is based on command + optional subcommand.

use anyhow::Result;

use crate::channels::control_command_authorization::ControlCommandAuthRule;

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandSelector {
    Any,
    Exact(String),
    Prefix(String),
}

impl CommandSelector {
    fn matches(&self, command_key: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(value) => command_key == value,
            Self::Prefix(prefix) => {
                command_key == prefix
                    || command_key
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with('.'))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Authorization rule built from command selectors and allowed identities.
pub struct CommandSelectorAuthRule {
    selectors: Vec<CommandSelector>,
    allowed_identities: Vec<String>,
}

impl CommandSelectorAuthRule {
    pub(crate) fn matches(&self, command_text: &str) -> bool {
        let Some(command_key) = extract_command_key(command_text) else {
            return false;
        };
        self.selectors
            .iter()
            .any(|selector| selector.matches(&command_key))
    }

    pub(crate) fn allows_normalized_identity(&self, normalized_identity: &str) -> bool {
        self.allowed_identities
            .iter()
            .any(|entry| entry == "*" || entry == normalized_identity)
    }
}

impl ControlCommandAuthRule for CommandSelectorAuthRule {
    fn matches(&self, command_text: &str) -> bool {
        CommandSelectorAuthRule::matches(self, command_text)
    }

    fn allows_identity(&self, identity: &str) -> bool {
        self.allows_normalized_identity(identity)
    }
}

pub(crate) fn parse_control_command_rule(
    selectors: Vec<String>,
    allowed_identities: Vec<String>,
    rule_label: &str,
    normalize_identity: fn(&str) -> String,
) -> Result<CommandSelectorAuthRule> {
    let selectors = parse_selectors_from_entries(selectors, rule_label)?;
    let allowed_identities =
        parse_allowed_identities_from_entries(allowed_identities, rule_label, normalize_identity)?;
    Ok(CommandSelectorAuthRule {
        selectors,
        allowed_identities,
    })
}

fn parse_selectors_from_entries(
    selectors: Vec<String>,
    rule_label: &str,
) -> Result<Vec<CommandSelector>> {
    let mut parsed_selectors = Vec::new();
    for selector in selectors {
        let parsed = parse_selector(&selector, rule_label)?;
        parsed_selectors.push(parsed);
    }
    if parsed_selectors.is_empty() {
        anyhow::bail!("invalid {rule_label}; command selector cannot be empty");
    }
    Ok(parsed_selectors)
}

fn parse_allowed_identities_from_entries(
    entries: Vec<String>,
    rule_label: &str,
    normalize_identity: fn(&str) -> String,
) -> Result<Vec<String>> {
    let allowed_identities: Vec<String> = entries
        .into_iter()
        .map(|entry| normalize_identity(&entry))
        .filter(|entry| !entry.is_empty())
        .collect();
    if allowed_identities.is_empty() {
        anyhow::bail!("invalid {rule_label}; allowed users cannot be empty");
    }
    Ok(allowed_identities)
}

fn parse_selector(raw_selector: &str, rule_label: &str) -> Result<CommandSelector> {
    let normalized = normalize_selector(raw_selector);
    if normalized.is_empty() {
        anyhow::bail!("invalid {rule_label}; command selector cannot be empty");
    }

    if normalized == "*" {
        return Ok(CommandSelector::Any);
    }

    if let Some(prefix) = normalized.strip_suffix(".*") {
        if prefix.is_empty() {
            anyhow::bail!("invalid {rule_label}; wildcard prefix cannot be empty");
        }
        if prefix.contains('*') {
            anyhow::bail!(
                "invalid {rule_label}; wildcard `*` is only allowed as full selector `*` or suffix `.*`"
            );
        }
        return Ok(CommandSelector::Prefix(prefix.to_string()));
    }

    if normalized.contains('*') {
        anyhow::bail!(
            "invalid {rule_label}; wildcard `*` is only allowed as full selector `*` or suffix `.*`"
        );
    }

    Ok(CommandSelector::Exact(normalized))
}

fn normalize_selector(value: &str) -> String {
    let trimmed = value.trim();
    let trimmed = trimmed.strip_prefix("cmd:").unwrap_or(trimmed);
    let trimmed = trimmed.trim_start_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.contains(char::is_whitespace) {
        return trimmed
            .split_whitespace()
            .map(normalize_token)
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>()
            .join(".");
    }

    trimmed
        .split('.')
        .map(normalize_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn normalize_token(token: &str) -> String {
    let token = token.trim();
    if token == "*" {
        return "*".to_string();
    }
    let token = token.split('@').next().unwrap_or(token);
    token
        .trim_start_matches('/')
        .replace('-', "_")
        .to_ascii_lowercase()
}

fn extract_command_key(command_text: &str) -> Option<String> {
    let mut parts = command_text.split_whitespace();
    let first = parts.next()?;
    if !first.starts_with('/') {
        return None;
    }

    let command = normalize_token(first);
    if command.is_empty() {
        return None;
    }

    let mut key_parts = vec![command];

    if let Some(second) = parts.next() {
        let subcommand = normalize_token(second);
        if is_subcommand_token(&subcommand) {
            key_parts.push(subcommand);
        }
    }

    Some(key_parts.join("."))
}

fn is_subcommand_token(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
}
