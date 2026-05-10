//! Discord channel integration (skeleton).

mod acl_config;
mod channel;
mod client;
mod constants;
mod parsing;
pub(crate) mod runtime;
mod send;
mod serenity_payload;
mod session_partition;

pub use acl_config::{DiscordAclOverrides, build_discord_acl_overrides};
pub use channel::{
    DiscordBotUserId, DiscordChannel, DiscordCommandAdminRule, DiscordControlCommandPolicy,
    DiscordSlashCommandPolicy, build_discord_command_admin_rule,
};
pub use constants::DISCORD_MAX_MESSAGE_LENGTH;
pub use runtime::DiscordIngressRunRequest;
pub use runtime::{
    DiscordIngressApp, DiscordIngressBuildRequest, DiscordRuntimeConfig, build_discord_ingress_app,
    build_discord_ingress_app_with_control_command_policy,
    build_discord_ingress_app_with_partition_and_control_command_policy, run_discord_gateway,
    run_discord_ingress,
};
pub use send::split_message_for_discord;
pub use session_partition::DiscordSessionPartition;
