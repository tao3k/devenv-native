use crate::agent::SessionRecallFeedbackDirection;
use crate::channels::managed_runtime::replies as shared_replies;

const PERMISSION_HINTS: shared_replies::PermissionHints<'static> =
    shared_replies::PermissionHints {
        control_command_hint: "Ask an identity allowed by `discord.control_command_allow_from` (or matching `discord.admin_command_rules` / `discord.admin_users`) to run this command.",
        slash_command_hint: "Ask an admin to grant this command via `discord.slash_*_allow_from` settings.",
    };

pub(in super::super) fn format_session_feedback(
    direction: SessionRecallFeedbackDirection,
    previous_bias: f32,
    updated_bias: f32,
) -> String {
    shared_replies::format_session_feedback(direction, previous_bias, updated_bias)
}

pub(in super::super) fn format_session_feedback_json(
    direction: SessionRecallFeedbackDirection,
    previous_bias: f32,
    updated_bias: f32,
) -> String {
    shared_replies::format_session_feedback_json(direction, previous_bias, updated_bias)
}

pub(in super::super) fn format_session_feedback_unavailable_json() -> String {
    shared_replies::format_session_feedback_unavailable_json()
}

pub(in super::super) fn format_control_command_admin_required(
    command: &str,
    sender: &str,
) -> String {
    shared_replies::format_control_command_admin_required(command, sender, PERMISSION_HINTS)
}

pub(in super::super) fn format_slash_command_permission_required(
    command: &str,
    sender: &str,
) -> String {
    shared_replies::format_slash_command_permission_required(command, sender, PERMISSION_HINTS)
}

pub(in super::super) fn format_slash_help() -> String {
    shared_replies::format_slash_help()
}

pub(in super::super) fn format_slash_help_json() -> String {
    shared_replies::format_slash_help_json()
}

pub(in super::super) fn format_command_error_json(command: &str, error: &str) -> String {
    shared_replies::format_command_error_json(command, error)
}

pub(in super::super) fn format_optional_usize(value: Option<usize>) -> String {
    shared_replies::format_optional_usize(value)
}

pub(in super::super) fn format_optional_f32(value: Option<f32>) -> String {
    shared_replies::format_optional_f32(value)
}
