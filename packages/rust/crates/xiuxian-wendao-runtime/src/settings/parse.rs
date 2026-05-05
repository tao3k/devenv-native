//! Scalar parsing helpers for runtime configuration values.

/// Parse a positive integer in `u64` form.
#[must_use]
pub fn parse_positive_u64(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

/// Parse a positive integer in `usize` form.
#[must_use]
pub fn parse_positive_usize(raw: &str) -> Option<usize> {
    raw.trim().parse::<usize>().ok().filter(|value| *value > 0)
}

/// Parse a positive finite float.
#[must_use]
pub fn parse_positive_f64(raw: &str) -> Option<f64> {
    raw.trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
}

/// Parse a conventional truthy/falsy string into a boolean.
#[must_use]
pub fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Return the first non-empty string after trimming whitespace.
#[must_use]
pub fn first_non_empty(values: &[Option<String>]) -> Option<String> {
    values.iter().flatten().find_map(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
#[path = "../../tests/unit/settings/parse.rs"]
mod tests;
