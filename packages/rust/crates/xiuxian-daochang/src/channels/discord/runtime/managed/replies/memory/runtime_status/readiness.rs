pub(super) fn is_backend_ready(
    enabled: bool,
    active_backend_present: bool,
    startup_load_status: &str,
) -> bool {
    enabled && active_backend_present && startup_load_status == "loaded"
}

pub(super) fn format_optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "-".to_string(), format_yes_no)
}

pub(super) fn format_optional_str(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_string(), ToString::to_string)
}

pub(super) fn format_optional_string(value: Option<String>) -> String {
    value.unwrap_or_else(|| "-".to_string())
}

pub(super) fn format_yes_no(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}
