//! Discord gateway coordinates bot identity, polling, foreground runtime, and managed command handling.

mod event_handler;
mod loop_control;
mod runner;

pub(in crate::channels::discord::runtime::gateway) use event_handler::DiscordGatewayEventHandler;
pub(in crate::channels::discord::runtime::gateway) use loop_control::drive_gateway_runtime_loop;
pub use runner::{run_discord_gateway, run_discord_gateway_listener};
