use crate::channels::managed_runtime::parsing::matching::eq_any_ignore_ascii;
use crate::channels::managed_runtime::parsing::normalize::normalize_command_input;
use crate::channels::managed_runtime::parsing::{
    FeedbackDirection, OutputFormat, SessionFeedbackCommand, SessionMentionCommand,
    SessionMentionMode, SessionPartitionCommand,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionContextCommandKind {
    Status,
    Budget,
    Memory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SessionContextCommand {
    kind: SessionContextCommandKind,
    format: OutputFormat,
}

pub(crate) fn parse_session_context_status_command(input: &str) -> Option<OutputFormat> {
    let command = parse_session_context_command(input)?;
    if matches!(command.kind, SessionContextCommandKind::Status) {
        return Some(command.format);
    }
    None
}

pub(crate) fn parse_session_context_budget_command(input: &str) -> Option<OutputFormat> {
    let command = parse_session_context_command(input)?;
    if matches!(command.kind, SessionContextCommandKind::Budget) {
        return Some(command.format);
    }
    None
}

pub(crate) fn parse_session_context_memory_command(input: &str) -> Option<OutputFormat> {
    let command = parse_session_context_command(input)?;
    if matches!(command.kind, SessionContextCommandKind::Memory) {
        return Some(command.format);
    }
    None
}

pub(crate) fn parse_session_feedback_command(input: &str) -> Option<SessionFeedbackCommand> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let command = parts.next()?;

    let (direction_raw, format_raw) = if eq_any_ignore_ascii(command, &["feedback"]) {
        (parts.next()?, parts.next())
    } else {
        if !is_session_family_command(command) {
            return None;
        }
        let sub = parts.next()?;
        if !eq_any_ignore_ascii(sub, &["feedback"]) {
            return None;
        }
        (parts.next()?, parts.next())
    };
    if parts.next().is_some() {
        return None;
    }

    let direction = parse_feedback_direction(direction_raw)?;
    let format = match format_raw {
        None => OutputFormat::Dashboard,
        Some(value) if eq_any_ignore_ascii(value, &["json"]) => OutputFormat::Json,
        Some(_) => return None,
    };
    Some(SessionFeedbackCommand { direction, format })
}

pub(crate) fn parse_session_partition_command<Mode, F>(
    input: &str,
    parse_mode: F,
) -> Option<SessionPartitionCommand<Mode>>
where
    F: Fn(&str) -> Option<Mode>,
{
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let command = parts.next()?;
    if !is_session_family_command(command) {
        return None;
    }

    let sub = parts.next()?;
    if !is_partition_scope_alias(sub) {
        return None;
    }

    let arg = parts.next();
    let maybe_json = parts.next();
    if parts.next().is_some() {
        return None;
    }

    match (arg, maybe_json) {
        (None, None) => Some(SessionPartitionCommand {
            mode: None,
            format: OutputFormat::Dashboard,
        }),
        (Some(value), None) if eq_any_ignore_ascii(value, &["json"]) => {
            Some(SessionPartitionCommand {
                mode: None,
                format: OutputFormat::Json,
            })
        }
        (Some(value), None) => Some(SessionPartitionCommand {
            mode: Some(parse_mode(value)?),
            format: OutputFormat::Dashboard,
        }),
        (Some(value), Some(fmt)) if eq_any_ignore_ascii(fmt, &["json"]) => {
            Some(SessionPartitionCommand {
                mode: Some(parse_mode(value)?),
                format: OutputFormat::Json,
            })
        }
        _ => None,
    }
}

pub(crate) fn parse_session_mention_command(input: &str) -> Option<SessionMentionCommand> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let command = parts.next()?;
    if !is_session_family_command(command) {
        return None;
    }
    let sub = parts.next()?;
    if !eq_any_ignore_ascii(sub, &["mention"]) {
        return None;
    }

    let arg = parts.next();
    let maybe_json = parts.next();
    if parts.next().is_some() {
        return None;
    }

    match (arg, maybe_json) {
        (None, None) => Some(SessionMentionCommand {
            mode: None,
            format: OutputFormat::Dashboard,
        }),
        (Some(value), None) if eq_any_ignore_ascii(value, &["json", "status", "info", "stats"]) => {
            Some(SessionMentionCommand {
                mode: None,
                format: if eq_any_ignore_ascii(value, &["json"]) {
                    OutputFormat::Json
                } else {
                    OutputFormat::Dashboard
                },
            })
        }
        (Some(value), Some(fmt))
            if eq_any_ignore_ascii(value, &["status", "info", "stats"])
                && eq_any_ignore_ascii(fmt, &["json"]) =>
        {
            Some(SessionMentionCommand {
                mode: None,
                format: OutputFormat::Json,
            })
        }
        (Some(value), None) => Some(SessionMentionCommand {
            mode: Some(parse_session_mention_mode(value)?),
            format: OutputFormat::Dashboard,
        }),
        (Some(value), Some(fmt)) if eq_any_ignore_ascii(fmt, &["json"]) => {
            Some(SessionMentionCommand {
                mode: Some(parse_session_mention_mode(value)?),
                format: OutputFormat::Json,
            })
        }
        _ => None,
    }
}

fn is_partition_scope_alias(token: &str) -> bool {
    eq_any_ignore_ascii(token, &["partition", "scope"])
}

fn parse_session_context_command(input: &str) -> Option<SessionContextCommand> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let command = parts.next()?;
    if !is_session_family_command(command) {
        return None;
    }

    let sub = parts.next();
    let maybe_json = parts.next();
    if parts.next().is_some() {
        return None;
    }

    match (sub, maybe_json) {
        (None, None) => Some(SessionContextCommand {
            kind: SessionContextCommandKind::Status,
            format: OutputFormat::Dashboard,
        }),
        (Some(value), None) if eq_any_ignore_ascii(value, &["json"]) => {
            Some(SessionContextCommand {
                kind: SessionContextCommandKind::Status,
                format: OutputFormat::Json,
            })
        }
        (Some(value), None) if is_status_alias(value) => Some(SessionContextCommand {
            kind: SessionContextCommandKind::Status,
            format: OutputFormat::Dashboard,
        }),
        (Some(value), None) if eq_any_ignore_ascii(value, &["budget"]) => {
            Some(SessionContextCommand {
                kind: SessionContextCommandKind::Budget,
                format: OutputFormat::Dashboard,
            })
        }
        (Some(value), None) if is_memory_alias(value) => Some(SessionContextCommand {
            kind: SessionContextCommandKind::Memory,
            format: OutputFormat::Dashboard,
        }),
        (Some(value), Some(fmt)) if eq_any_ignore_ascii(fmt, &["json"]) => {
            if is_status_alias(value) {
                return Some(SessionContextCommand {
                    kind: SessionContextCommandKind::Status,
                    format: OutputFormat::Json,
                });
            }
            if eq_any_ignore_ascii(value, &["budget"]) {
                return Some(SessionContextCommand {
                    kind: SessionContextCommandKind::Budget,
                    format: OutputFormat::Json,
                });
            }
            if is_memory_alias(value) {
                return Some(SessionContextCommand {
                    kind: SessionContextCommandKind::Memory,
                    format: OutputFormat::Json,
                });
            }
            None
        }
        _ => None,
    }
}

fn is_session_family_command(command: &str) -> bool {
    eq_any_ignore_ascii(command, &["session", "window", "context"])
}

fn is_status_alias(token: &str) -> bool {
    eq_any_ignore_ascii(token, &["status", "stats", "info"])
}

fn is_memory_alias(token: &str) -> bool {
    eq_any_ignore_ascii(token, &["memory", "recall"])
}

fn parse_feedback_direction(raw: &str) -> Option<FeedbackDirection> {
    if eq_any_ignore_ascii(raw, &["up", "success", "positive", "good"]) || raw == "+" {
        return Some(FeedbackDirection::Up);
    }
    if eq_any_ignore_ascii(raw, &["down", "failure", "negative", "bad", "fail"]) || raw == "-" {
        return Some(FeedbackDirection::Down);
    }
    None
}

fn parse_session_mention_mode(raw: &str) -> Option<SessionMentionMode> {
    if eq_any_ignore_ascii(raw, &["on", "enable", "enabled", "require", "required"]) {
        return Some(SessionMentionMode::Require);
    }
    if eq_any_ignore_ascii(raw, &["off", "disable", "disabled", "open"]) {
        return Some(SessionMentionMode::Open);
    }
    if eq_any_ignore_ascii(raw, &["inherit", "default", "unset", "clear"]) {
        return Some(SessionMentionMode::Inherit);
    }
    None
}
