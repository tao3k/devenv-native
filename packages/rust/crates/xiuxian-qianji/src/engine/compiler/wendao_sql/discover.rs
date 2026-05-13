use crate::contracts::NodeDefinition;

use super::params::{string_list_param, string_param, usize_param};

pub(in crate::engine::compiler) struct WendaoSqlDiscoverMechanismConfig {
    pub(in crate::engine::compiler) output_key: String,
    pub(in crate::engine::compiler) endpoint_key: Option<String>,
    pub(in crate::engine::compiler) endpoint: Option<String>,
    pub(in crate::engine::compiler) project_root_key: Option<String>,
    pub(in crate::engine::compiler) allowed_objects: Vec<String>,
    pub(in crate::engine::compiler) max_limit: usize,
    pub(in crate::engine::compiler) allowed_ops: Vec<String>,
    pub(in crate::engine::compiler) require_filter_for: Vec<String>,
}

pub(in crate::engine::compiler) fn mechanism_config(
    node_def: &NodeDefinition,
) -> WendaoSqlDiscoverMechanismConfig {
    let allowed_ops = string_list_param(node_def, "allowed_ops");
    WendaoSqlDiscoverMechanismConfig {
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "surface_bundle_xml".to_string()),
        endpoint_key: string_param(node_def, "endpoint_key"),
        endpoint: string_param(node_def, "endpoint"),
        project_root_key: string_param(node_def, "project_root_key")
            .or_else(|| Some("project_root".to_string())),
        allowed_objects: string_list_param(node_def, "allowed_objects"),
        max_limit: usize_param(node_def, "max_limit").unwrap_or(8),
        allowed_ops: if allowed_ops.is_empty() {
            vec!["eq".to_string(), "contains".to_string()]
        } else {
            allowed_ops
        },
        require_filter_for: string_list_param(node_def, "require_filter_for"),
    }
}
