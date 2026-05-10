//! Discord channel adapter with shared control-command authorization policy.

mod auth;
mod bot_identity;
mod constructor;
mod mention_policy;
mod mention_policy_persistence;
mod policy;
mod policy_builders;
mod recipient_admin;
mod state;
mod trait_impl;

pub use bot_identity::DiscordBotUserId;
pub use policy::{DiscordCommandAdminRule, DiscordControlCommandPolicy, DiscordSlashCommandPolicy};
pub use policy_builders::build_discord_command_admin_rule;
pub use state::DiscordChannel;
