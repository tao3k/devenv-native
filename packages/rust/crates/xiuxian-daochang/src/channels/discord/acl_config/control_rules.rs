use std::collections::HashMap;

use anyhow::Result;

use crate::channels::discord::channel::{
    DiscordCommandAdminRule, build_discord_command_admin_rule,
};
use crate::config::DiscordAclControlSettings;

use super::principals::collect_principals;

pub(super) fn control_rules(
    control: &DiscordAclControlSettings,
    role_aliases: &HashMap<String, String>,
) -> Result<Vec<DiscordCommandAdminRule>> {
    let Some(rules) = control.rules.as_ref() else {
        return Ok(Vec::new());
    };
    let mut parsed_rules = Vec::new();
    for (index, rule) in rules.iter().enumerate() {
        let commands: Vec<String> = rule
            .commands
            .iter()
            .map(|command| command.trim().to_string())
            .filter(|command| !command.is_empty())
            .collect();
        if commands.is_empty() {
            tracing::warn!("discord acl control rule ignored: empty commands");
            continue;
        }
        let Some(principals) = collect_principals(&rule.allow, role_aliases) else {
            tracing::warn!(
                commands = %commands.join(","),
                "discord acl control rule ignored: no allow principals configured"
            );
            continue;
        };
        if principals.is_empty() {
            tracing::warn!(
                commands = %commands.join(","),
                "discord acl control rule ignored: allow principals resolved to empty set"
            );
            continue;
        }
        let parsed_rule =
            build_discord_command_admin_rule(commands, principals).map_err(|error| {
                anyhow::anyhow!("discord.acl.control.rules[{index}].commands: {error}")
            })?;
        parsed_rules.push(parsed_rule);
    }
    Ok(parsed_rules)
}
