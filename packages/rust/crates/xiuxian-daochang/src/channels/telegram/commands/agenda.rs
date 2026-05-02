//! Telegram agenda command parser.

use super::shared::normalize_command_input;

/// Parse `/agenda` (or `agenda`) as a managed command.
pub fn is_agenda_command(input: &str) -> bool {
    normalize_command_input(input).eq_ignore_ascii_case("agenda")
}
