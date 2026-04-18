pub(super) use crate::settings::get_setting_string;
pub(super) use crate::settings::merged_wendao_settings;

/// Override the global Wendao configuration home directory for link-graph
/// runtime resolution.
pub fn set_link_graph_config_home_override(path: &str) {
    crate::settings::set_wendao_config_home_override(path);
}

/// Clear the global Wendao configuration home override for link-graph runtime
/// resolution.
pub fn clear_link_graph_config_home_override() {
    crate::settings::clear_wendao_config_home_override();
}

/// Override the global Wendao configuration file path for link-graph runtime
/// resolution.
pub fn set_link_graph_wendao_config_override(path: &str) {
    crate::settings::set_wendao_config_override(path);
}

/// Clear the global Wendao configuration file override for link-graph runtime
/// resolution.
pub fn clear_link_graph_wendao_config_override() {
    crate::settings::clear_wendao_config_override();
}
