//! Markdown property drawer parsing.

use std::collections::HashMap;

/// Parse one property drawer line such as `:ID: value`.
#[must_use]
pub fn parse_property_drawer(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with(':') {
        return None;
    }

    let rest = &trimmed[1..];
    let colon_pos = rest.find(':')?;

    let key = rest[..colon_pos].trim().to_uppercase();
    if key.is_empty() {
        return None;
    }

    let value = rest[colon_pos + 1..].trim().to_string();
    if value.is_empty() {
        return None;
    }

    Some((key, value))
}

/// Extract section-leading property drawer attributes.
#[must_use]
pub fn extract_property_drawers(lines: &[String]) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    let mut in_properties_block = false;
    let mut block_ended = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == ":PROPERTIES:" {
            in_properties_block = true;
            continue;
        }

        if in_properties_block && trimmed == ":END:" {
            in_properties_block = false;
            block_ended = true;
            continue;
        }

        if in_properties_block {
            if let Some((key, value)) = parse_property_drawer(line) {
                attributes.insert(key, value);
            }
            continue;
        }

        if block_ended {
            break;
        }

        if let Some((key, value)) = parse_property_drawer(line) {
            attributes.insert(key, value);
        } else if trimmed.is_empty() {
            // Skip empty lines at the start of the section.
        } else {
            break;
        }
    }

    attributes
}
