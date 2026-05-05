//! Telegram session-admin command parser.

use super::{SessionAdminAction, SessionAdminCommand, SessionOutputFormat};
use crate::channels::managed_runtime::parsing::normalize_command_input;

/// Parse delegated session-admin command:
/// - `/session admin` or `/session admin list [json]`
/// - `/session admin set <user_ids...> [json]`
/// - `/session admin add <user_ids...> [json]`
/// - `/session admin remove <user_ids...> [json]`
/// - `/session admin clear [json]`
pub fn parse_session_admin_command(input: &str) -> Option<SessionAdminCommand> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let root = parts.next()?;
    if !root.eq_ignore_ascii_case("session")
        && !root.eq_ignore_ascii_case("window")
        && !root.eq_ignore_ascii_case("context")
    {
        return None;
    }
    let sub = parts.next()?;
    if !sub.eq_ignore_ascii_case("admin") {
        return None;
    }
    let tokens: Vec<&str> = parts.collect();
    if tokens.is_empty() {
        return Some(SessionAdminCommand {
            action: SessionAdminAction::List,
            format: SessionOutputFormat::Dashboard,
        });
    }
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("json") {
        return Some(SessionAdminCommand {
            action: SessionAdminAction::List,
            format: SessionOutputFormat::Json,
        });
    }

    let mut format = SessionOutputFormat::Dashboard;
    let args_end = if tokens
        .last()
        .is_some_and(|token| token.eq_ignore_ascii_case("json"))
    {
        format = SessionOutputFormat::Json;
        tokens.len().saturating_sub(1)
    } else {
        tokens.len()
    };
    if args_end == 0 {
        return None;
    }
    let command = tokens[0];
    let id_tokens = &tokens[1..args_end];
    let action = if command.eq_ignore_ascii_case("list") {
        if !id_tokens.is_empty() {
            return None;
        }
        SessionAdminAction::List
    } else if command.eq_ignore_ascii_case("clear") {
        if !id_tokens.is_empty() {
            return None;
        }
        SessionAdminAction::Clear
    } else if command.eq_ignore_ascii_case("set") {
        SessionAdminAction::Set(parse_admin_user_ids(id_tokens)?)
    } else if command.eq_ignore_ascii_case("add") {
        SessionAdminAction::Add(parse_admin_user_ids(id_tokens)?)
    } else if command.eq_ignore_ascii_case("remove")
        || command.eq_ignore_ascii_case("rm")
        || command.eq_ignore_ascii_case("del")
    {
        SessionAdminAction::Remove(parse_admin_user_ids(id_tokens)?)
    } else if command.eq_ignore_ascii_case("json") {
        return None;
    } else {
        SessionAdminAction::Set(parse_admin_user_ids(&tokens[..args_end])?)
    };

    Some(SessionAdminCommand { action, format })
}

fn parse_admin_user_ids(raw_tokens: &[&str]) -> Option<Vec<String>> {
    if raw_tokens.is_empty() {
        return None;
    }
    let values: Vec<String> = raw_tokens
        .iter()
        .flat_map(|token| token.split(','))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect();
    if values.is_empty() {
        return None;
    }
    Some(values)
}
