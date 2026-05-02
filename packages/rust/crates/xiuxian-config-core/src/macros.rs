//! Exported config precedence macro definitions.

/// Resolve the first `Some(...)` candidate from an ordered precedence chain.
#[macro_export]
macro_rules! first_some {
    ($candidate:expr $(,)?) => {
        $candidate
    };
    ($first:expr, $($rest:expr),+ $(,)?) => {{
        let resolved = $first;
        if resolved.is_some() {
            resolved
        } else {
            $crate::first_some!($($rest),+)
        }
    }};
}

/// Resolve a TOML-owned setting first and then fall back to a precedence-ordered
/// env lookup chain.
///
/// The first form returns a trimmed string:
/// `toml_first_env!(settings, "path.to.key", lookup, ["ENV_A", "ENV_B"], get_setting)`
///
/// The second form additionally applies a parser closure and falls back to env
/// when the TOML value is blank or fails to parse:
/// `toml_first_env!(settings, "path.to.key", lookup, ["ENV"], get_setting, parse)`
#[macro_export]
macro_rules! toml_first_env {
    ($settings:expr, $setting_key:expr, $lookup:expr, [$($env:expr),+ $(,)?], $get_setting:path) => {{
        $crate::toml_first_env_string(
            $get_setting($settings, $setting_key),
            $lookup,
            &[$($env),+],
        )
    }};
    ($settings:expr, $setting_key:expr, $lookup:expr, [$($env:expr),+ $(,)?], $get_setting:path, $parse:expr) => {{
        $crate::toml_first_env_parsed(
            $get_setting($settings, $setting_key),
            $lookup,
            &[$($env),+],
            $parse,
        )
    }};
}
