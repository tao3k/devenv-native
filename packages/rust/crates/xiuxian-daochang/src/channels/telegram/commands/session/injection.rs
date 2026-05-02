//! Telegram session prompt-injection command parser.

use super::{SessionInjectionAction, SessionInjectionCommand, SessionOutputFormat};
use crate::channels::managed_runtime::parsing::{
    normalize_command_input, slice_original_command_suffix,
};

/// Parse session system prompt injection command:
/// - `/session inject` or `/session inject status [json]`
/// - `/session inject clear [json]`
/// - `/session inject set <xml>`
/// - `/session inject <xml>`
pub fn parse_session_injection_command(input: &str) -> Option<SessionInjectionCommand> {
    let normalized = normalize_command_input(input);
    let lowered = normalized.to_ascii_lowercase();
    let prefixes = [
        "session inject",
        "window inject",
        "context inject",
        "session injection",
        "window injection",
        "context injection",
    ];

    let rest = prefixes.iter().find_map(|prefix| {
        lowered.strip_prefix(prefix).and_then(|suffix| {
            if suffix.trim().is_empty() {
                Some(String::new())
            } else {
                slice_original_command_suffix(normalized, suffix).map(ToString::to_string)
            }
        })
    })?;

    let tail = rest.trim();
    if tail.is_empty() {
        return Some(SessionInjectionCommand {
            action: SessionInjectionAction::Status,
            format: SessionOutputFormat::Dashboard,
        });
    }
    if tail.eq_ignore_ascii_case("json") {
        return Some(SessionInjectionCommand {
            action: SessionInjectionAction::Status,
            format: SessionOutputFormat::Json,
        });
    }
    if tail.eq_ignore_ascii_case("status") {
        return Some(SessionInjectionCommand {
            action: SessionInjectionAction::Status,
            format: SessionOutputFormat::Dashboard,
        });
    }
    if tail.eq_ignore_ascii_case("status json") {
        return Some(SessionInjectionCommand {
            action: SessionInjectionAction::Status,
            format: SessionOutputFormat::Json,
        });
    }
    if tail.eq_ignore_ascii_case("clear") {
        return Some(SessionInjectionCommand {
            action: SessionInjectionAction::Clear,
            format: SessionOutputFormat::Dashboard,
        });
    }
    if tail.eq_ignore_ascii_case("clear json") {
        return Some(SessionInjectionCommand {
            action: SessionInjectionAction::Clear,
            format: SessionOutputFormat::Json,
        });
    }
    if tail.eq_ignore_ascii_case("set") {
        return None;
    }
    let lowered_tail = tail.to_ascii_lowercase();
    if lowered_tail.starts_with("status ") || lowered_tail.starts_with("clear ") {
        return None;
    }

    let payload = if lowered_tail.starts_with("set ") {
        tail[4..].trim()
    } else {
        tail
    };
    if payload.is_empty() {
        return None;
    }
    Some(SessionInjectionCommand {
        action: SessionInjectionAction::SetXml(payload.to_string()),
        format: SessionOutputFormat::Dashboard,
    })
}
