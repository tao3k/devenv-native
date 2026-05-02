//! Discord ingress runtime coordinates HTTP ingress, foreground turn execution, telemetry, and server shutdown.

mod ingress;
mod loop_control;

pub use ingress::{DiscordIngressRunRequest, run_discord_ingress};
