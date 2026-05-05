use std::collections::HashMap;

use crate::config::{DiscordAclAllowSettings, DiscordAclPrincipalSettings};

use super::role_aliases::{resolve_principal_entry, resolve_role_principal};

pub(super) fn collect_principals(
    principal: &DiscordAclPrincipalSettings,
    role_aliases: &HashMap<String, String>,
) -> Option<Vec<String>> {
    let configured = principal.users.is_some() || principal.roles.is_some();
    if !configured {
        return None;
    }

    let mut resolved: Vec<String> = principal
        .users
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| resolve_principal_entry(&entry, role_aliases))
        .collect();
    resolved.extend(
        principal
            .roles
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| resolve_role_principal(&entry, role_aliases)),
    );

    Some(resolved)
}

pub(super) fn principal_list_from_allow(
    allow: &DiscordAclAllowSettings,
    role_aliases: &HashMap<String, String>,
) -> Option<Vec<String>> {
    let principal = DiscordAclPrincipalSettings {
        users: allow.users.clone(),
        roles: allow.roles.clone(),
    };
    collect_principals(&principal, role_aliases)
}

pub(super) fn guilds_list_from_allow(allow: &DiscordAclAllowSettings) -> Option<Vec<String>> {
    let guilds = allow.guilds.as_ref()?;
    Some(
        guilds
            .iter()
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .collect(),
    )
}
