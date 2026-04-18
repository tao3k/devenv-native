use crate::env_parse::lookup_env;

pub(in crate::channels) fn resolve_runtime_snapshot_interval_secs<F>(
    lookup: F,
    env_var: &str,
    default_secs: u64,
) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    let Some(raw) = lookup_env(&lookup, env_var) else {
        return Some(default_secs);
    };
    match raw.trim().parse::<u64>() {
        Ok(0) => None,
        Ok(value) => Some(value),
        Err(_) => {
            tracing::warn!(
                env_var,
                value = %raw,
                default_secs,
                "invalid runtime snapshot interval; using default"
            );
            Some(default_secs)
        }
    }
}
