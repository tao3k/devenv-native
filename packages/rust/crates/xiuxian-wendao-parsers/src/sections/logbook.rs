//! Markdown logbook drawer parsing.

use super::types::LogbookEntry;

/// Parse one `:LOGBOOK:` entry line.
#[must_use]
pub fn parse_logbook_entry(line: &str, line_number: usize) -> Option<LogbookEntry> {
    let trimmed = line.trim();

    if !trimmed.starts_with('-') {
        return None;
    }

    let rest = trimmed[1..].trim_start();

    if !rest.starts_with('[') {
        return None;
    }

    let close_bracket = rest.find(']')?;
    let timestamp = rest[1..close_bracket].trim().to_string();

    if timestamp.is_empty() {
        return None;
    }

    let message = rest[close_bracket + 1..].trim().to_string();

    if message.is_empty() {
        return None;
    }

    Some(LogbookEntry {
        timestamp,
        message,
        line_number,
    })
}

/// Extract all `:LOGBOOK:` entries from one section.
#[must_use]
pub fn extract_logbook_entries(lines: &[String], start_line: usize) -> Vec<LogbookEntry> {
    let mut entries = Vec::new();
    let mut in_logbook_block = false;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed == ":LOGBOOK:" {
            in_logbook_block = true;
            continue;
        }

        if in_logbook_block && trimmed == ":END:" {
            break;
        }

        if in_logbook_block {
            let line_number = start_line + idx + 1;
            if let Some(entry) = parse_logbook_entry(line, line_number) {
                entries.push(entry);
            }
        }
    }

    entries
}
