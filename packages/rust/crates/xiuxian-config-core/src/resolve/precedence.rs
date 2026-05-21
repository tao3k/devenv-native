//! Scalar precedence helpers for TOML-first and environment-backed settings.

use std::str::FromStr;

/// Named scalar value selected from a precedence chain.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct NamedScalarValue {
    /// Name of the TOML setting or env-style lookup key that produced `value`.
    pub source_name: String,
    /// Trimmed scalar value selected from the source.
    pub value: String,
}

impl NamedScalarValue {
    /// Build a named scalar value from a source name and value.
    pub fn new(source_name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            value: value.into(),
        }
    }
}

/// Return a trimmed non-empty string candidate.
#[must_use]
pub fn trimmed_non_empty(candidate: Option<String>) -> Option<String> {
    candidate
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Parse a trimmed scalar value.
#[must_use]
pub fn parse_trimmed<T>(raw: &str) -> Option<T>
where
    T: FromStr,
{
    raw.trim().parse::<T>().ok()
}

/// Parse a positive numeric-like value from trimmed input.
#[must_use]
pub fn parse_positive<T>(raw: &str) -> Option<T>
where
    T: FromStr + Default + PartialOrd,
{
    let value = parse_trimmed::<T>(raw)?;
    (value > T::default()).then_some(value)
}

/// Parse a bool-style flag from trimmed input.
#[must_use]
pub fn parse_bool_flag(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Return the first non-empty environment-style value from a precedence list.
#[must_use]
pub fn first_non_empty_lookup(
    names: &[&str],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    names
        .iter()
        .find_map(|name| trimmed_non_empty(lookup(name)))
}

/// Return the first non-empty environment-style value and its lookup key.
#[must_use]
pub fn first_non_empty_named_lookup(
    names: &[&str],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<NamedScalarValue> {
    names.iter().find_map(|name| {
        trimmed_non_empty(lookup(name)).map(|value| NamedScalarValue::new(*name, value))
    })
}

/// Resolve a TOML-owned string first and otherwise return the first named
/// env-style lookup candidate.
#[must_use]
pub fn toml_first_named_string(
    setting_name: &str,
    setting_value: Option<String>,
    lookup: &dyn Fn(&str) -> Option<String>,
    env_names: &[&str],
) -> Option<NamedScalarValue> {
    trimmed_non_empty(setting_value)
        .map(|value| NamedScalarValue::new(setting_name, value))
        .or_else(|| first_non_empty_named_lookup(env_names, lookup))
}

/// Resolve a parsed env-style value from a single lookup key.
#[must_use]
pub fn lookup_parsed<T, F>(
    name: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
    parse: F,
) -> Option<T>
where
    F: Fn(&str) -> Option<T>,
{
    trimmed_non_empty(lookup(name)).and_then(|value| parse(value.as_str()))
}

/// Resolve a positive parsed env-style value from a single lookup key.
#[must_use]
pub fn lookup_positive_parsed<T>(name: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Option<T>
where
    T: FromStr + Default + PartialOrd,
{
    lookup_parsed(name, lookup, parse_positive::<T>)
}

/// Resolve a bool-style env flag from a single lookup key.
#[must_use]
pub fn lookup_bool_flag(name: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Option<bool> {
    lookup_parsed(name, lookup, parse_bool_flag)
}

/// Resolve a TOML-owned string first and fall back to env-style lookups.
///
/// Invalid or blank TOML values are treated as absent for this permissive
/// helper, which keeps optional runtime surfaces recoverable through the
/// fallback env chain.
#[must_use]
pub fn toml_first_env_string(
    setting_value: Option<String>,
    lookup: &dyn Fn(&str) -> Option<String>,
    env_names: &[&str],
) -> Option<String> {
    trimmed_non_empty(setting_value).or_else(|| first_non_empty_lookup(env_names, lookup))
}

/// Resolve a TOML-owned value first and fall back to env-style lookups.
///
/// Invalid or blank TOML values are treated as absent for this permissive
/// helper, which keeps optional runtime surfaces recoverable through the
/// fallback env chain.
#[must_use]
pub fn toml_first_env_parsed<T, F>(
    setting_value: Option<String>,
    lookup: &dyn Fn(&str) -> Option<String>,
    env_names: &[&str],
    parse: F,
) -> Option<T>
where
    F: Fn(&str) -> Option<T>,
{
    trimmed_non_empty(setting_value)
        .and_then(|value| parse(value.as_str()))
        .or_else(|| {
            first_non_empty_lookup(env_names, lookup).and_then(|value| parse(value.as_str()))
        })
}

#[cfg(test)]
#[path = "../../tests/unit/resolve/precedence.rs"]
mod tests;
