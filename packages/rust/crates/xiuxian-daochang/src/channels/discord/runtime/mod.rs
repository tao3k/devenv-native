//! Discord runtime wiring (ingress + foreground turn execution).

mod config;
mod dispatch;
mod foreground;
mod gateway;
mod ingress;
mod interrupt;
mod managed;
mod run;
mod telemetry;
mod test_api;

pub use config::DiscordRuntimeConfig;
pub use gateway::{run_discord_gateway, run_discord_gateway_listener};
pub use ingress::{
    DiscordIngressApp, DiscordIngressBuildRequest, build_discord_ingress_app,
    build_discord_ingress_app_with_control_command_policy,
    build_discord_ingress_app_with_partition_and_control_command_policy,
};
pub use run::{DiscordIngressRunRequest, run_discord_ingress};

pub(crate) use foreground::{
    DiscordForegroundRuntime, DiscordForegroundRuntime as TestDiscordForegroundRuntime,
    build_foreground_runtime, build_foreground_runtime as test_build_discord_foreground_runtime,
};
pub(crate) use interrupt::ForegroundInterruptController;
pub(crate) use telemetry::snapshot_interval_from_env;
pub(crate) use test_api::{
    test_interrupted_reply_is_suppressed, test_process_discord_message,
    test_process_discord_message_with_interrupt, test_push_background_completion,
    test_resolve_snapshot_interval_secs,
};
