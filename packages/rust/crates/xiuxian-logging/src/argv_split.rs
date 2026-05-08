//! CLI argument splitter for shared `xiuxian-logging` flags.

use crate::types::{LogColor, LogFormat, LogLevel, LogSettings};

fn parse_verbose_flag(arg: &str) -> Option<u8> {
    if !arg.starts_with('-') || arg.starts_with("--") {
        return None;
    }
    let trimmed = arg.trim_start_matches('-');
    if trimmed.is_empty() || !trimmed.chars().all(|ch| ch == 'v') {
        return None;
    }
    Some(u8::try_from(trimmed.len()).unwrap_or(u8::MAX))
}

fn take_value<'a>(args: &'a [String], index: usize, flag: &str) -> Option<(&'a str, usize)> {
    let arg = args.get(index)?.as_str();
    if arg == flag {
        return args.get(index + 1).map(|value| (value.as_str(), 2));
    }
    let value = arg
        .strip_prefix(flag)
        .and_then(|rest| rest.strip_prefix('='));
    value.map(|value| (value, 1))
}

/// Split logging args from an argument vector, returning settings and remaining args.
#[must_use]
pub fn split_logging_args(raw: &[String]) -> (LogSettings, Vec<String>) {
    let mut settings = LogSettings::default();
    let mut remaining = Vec::with_capacity(raw.len());
    let mut index = 0;

    while index < raw.len() {
        let arg = raw[index].as_str();

        if let Some(count) = parse_verbose_flag(arg) {
            settings.verbose = settings.verbose.saturating_add(count);
            index += 1;
            continue;
        }

        if arg == "--log-verbose" {
            settings.verbose = settings.verbose.saturating_add(1);
            index += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--log-verbose=")
            && let Ok(count) = value.parse::<u8>()
        {
            settings.verbose = settings.verbose.saturating_add(count);
            index += 1;
            continue;
        }

        if let Some((value, consumed)) = take_value(raw, index, "--log-format")
            && let Ok(format) = value.parse::<LogFormat>()
        {
            settings.format = format;
            index += consumed;
            continue;
        }

        if let Some((value, consumed)) = take_value(raw, index, "--log-color")
            && let Ok(color) = value.parse::<LogColor>()
        {
            settings.color = color;
            index += consumed;
            continue;
        }

        if let Some((value, consumed)) = take_value(raw, index, "--log-level")
            && let Ok(level) = value.parse::<LogLevel>()
        {
            settings.level = Some(level);
            index += consumed;
            continue;
        }

        if let Some((value, consumed)) = take_value(raw, index, "--log-filter") {
            settings.filter = Some(value.to_string());
            index += consumed;
            continue;
        }

        remaining.push(raw[index].clone());
        index += 1;
    }

    (settings, remaining)
}
