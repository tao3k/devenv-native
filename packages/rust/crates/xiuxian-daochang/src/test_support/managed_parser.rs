//! Managed-command detector helpers exposed for integration tests.

use crate::channels::managed_commands as managed;

use super::types::{
    ManagedControlCommand, ManagedSlashCommand, managed_control_command_from_internal,
    managed_slash_command_from_internal,
};

/// Detects a managed slash command in test input.
pub fn detect_managed_slash_command(input: &str) -> Option<ManagedSlashCommand> {
    managed::detect_managed_slash_command(input).map(managed_slash_command_from_internal)
}

/// Detects a managed control command in test input.
pub fn detect_managed_control_command(input: &str) -> Option<ManagedControlCommand> {
    managed::detect_managed_control_command(input).map(managed_control_command_from_internal)
}
