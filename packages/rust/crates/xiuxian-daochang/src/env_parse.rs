//! Environment parsing helpers shared across runtime configuration.

#[must_use]
/// Looks up a raw environment variable through an injected lookup function.
pub fn lookup_env<F>(lookup: &F, name: &str) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(name)
}

#[must_use]
/// Looks up a non-empty environment variable through an injected lookup function.
pub fn lookup_non_empty_env<F>(lookup: &F, name: &str) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup_env(lookup, name).and_then(|value| trim_non_empty(&value))
}

#[must_use]
/// Reads a raw process environment variable.
pub fn read_env(name: &str) -> Option<String> {
    lookup_env(&|env_name| std::env::var(env_name).ok(), name)
}

#[must_use]
/// Reads a non-empty process environment variable.
pub fn read_non_empty_env(name: &str) -> Option<String> {
    lookup_non_empty_env(&|env_name| std::env::var(env_name).ok(), name)
}

#[must_use]
/// Parses a positive `u32` from the named process environment variable.
pub fn parse_positive_u32_from_env(name: &str) -> Option<u32> {
    parse_env_value(
        name,
        |raw| raw.parse::<u32>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

#[must_use]
/// Parses a positive `usize` from the named process environment variable.
pub fn parse_positive_usize_from_env(name: &str) -> Option<usize> {
    parse_env_value(
        name,
        |raw| raw.parse::<usize>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

#[must_use]
/// Parses a positive `u64` from the named process environment variable.
pub fn parse_positive_u64_from_env(name: &str) -> Option<u64> {
    parse_env_value(
        name,
        |raw| raw.parse::<u64>().ok().filter(|value| *value > 0),
        "invalid positive integer env value",
    )
}

#[must_use]
/// Parses a positive `f32` from the named process environment variable.
pub fn parse_positive_f32_from_env(name: &str) -> Option<f32> {
    parse_env_value(
        name,
        |raw| raw.parse::<f32>().ok().filter(|value| *value > 0.0),
        "invalid positive float env value",
    )
}

#[must_use]
/// Parses a unit-range `f32` from the named process environment variable.
pub fn parse_unit_f32_from_env(name: &str) -> Option<f32> {
    parse_env_value(
        name,
        |raw| {
            raw.parse::<f32>()
                .ok()
                .filter(|value| (0.0..=1.0).contains(value))
        },
        "invalid unit float env value (expected 0.0..=1.0)",
    )
}

#[must_use]
/// Parses a boolean from the named process environment variable.
pub fn parse_bool_from_env(name: &str) -> Option<bool> {
    parse_env_value(
        name,
        |raw| match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        },
        "invalid boolean env value",
    )
}

#[must_use]
/// Resolves the configured valkey URL from supported environment variables.
pub fn resolve_valkey_url_env() -> Option<String> {
    std::env::var("XIUXIAN_WENDAO_VALKEY_URL")
        .ok()
        .as_deref()
        .and_then(trim_non_empty)
        .or_else(|| {
            std::env::var("VALKEY_URL")
                .ok()
                .as_deref()
                .and_then(trim_non_empty)
        })
}

fn parse_env_value<T>(
    name: &str,
    parser: impl FnOnce(&str) -> Option<T>,
    invalid_message: &'static str,
) -> Option<T> {
    let raw = read_env(name)?;
    if let Some(value) = parser(raw.as_str()) {
        Some(value)
    } else {
        tracing::warn!(env_var = %name, value = %raw, "{invalid_message}");
        None
    }
}

fn trim_non_empty(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
