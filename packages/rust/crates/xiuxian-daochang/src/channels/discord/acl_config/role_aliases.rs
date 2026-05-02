use std::collections::HashMap;

use crate::config::DiscordAclSettings;

pub(super) fn normalize_role_aliases(acl: &DiscordAclSettings) -> HashMap<String, String> {
    let mut normalized = HashMap::new();
    let Some(role_aliases) = acl.role_aliases.as_ref() else {
        return normalized;
    };
    for (alias, raw_role_value) in role_aliases {
        let key = alias.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        let Some(role_id) = parse_role_id(raw_role_value) else {
            tracing::warn!(
                alias = %key,
                value = %raw_role_value,
                "discord acl role_aliases entry ignored: invalid role id"
            );
            continue;
        };
        normalized.insert(key, format!("role:{role_id}"));
    }
    normalized
}

fn parse_role_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("role:") {
        return parse_role_id(rest);
    }
    if let Some(rest) = trimmed
        .strip_prefix("<@&")
        .and_then(|value| value.strip_suffix('>'))
    {
        return parse_role_id(rest);
    }
    if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(trimmed.to_string());
    }
    None
}

pub(super) fn resolve_role_principal(
    raw_role: &str,
    role_aliases: &HashMap<String, String>,
) -> Option<String> {
    let role = raw_role.trim();
    if role.is_empty() {
        return None;
    }
    if let Some(role_id) = parse_role_id(role) {
        return Some(format!("role:{role_id}"));
    }

    let alias_key = role
        .strip_prefix("role:")
        .map_or(role, str::trim)
        .to_ascii_lowercase();
    if let Some(role_principal) = role_aliases.get(&alias_key) {
        return Some(role_principal.clone());
    }

    tracing::warn!(
        role = %role,
        "discord acl role entry ignored: role id or alias not found"
    );
    None
}

pub(super) fn resolve_principal_entry(
    raw_entry: &str,
    role_aliases: &HashMap<String, String>,
) -> Option<String> {
    let entry = raw_entry.trim();
    if entry.is_empty() {
        return None;
    }
    if entry.starts_with("role:") || entry.starts_with("<@&") {
        return resolve_role_principal(entry, role_aliases);
    }
    Some(entry.to_string())
}
