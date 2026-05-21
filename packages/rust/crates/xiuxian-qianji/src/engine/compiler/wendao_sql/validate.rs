use crate::contracts::NodeDefinition;

use super::params::string_param;

pub(in crate::engine::compiler) struct WendaoSqlValidateMechanismConfig {
    pub(in crate::engine::compiler) surface_bundle_key: String,
    pub(in crate::engine::compiler) author_spec_key: String,
    pub(in crate::engine::compiler) output_key: String,
    pub(in crate::engine::compiler) report_key: String,
    pub(in crate::engine::compiler) error_key: String,
    pub(in crate::engine::compiler) accepted_branch_label: Option<String>,
    pub(in crate::engine::compiler) rejected_branch_label: Option<String>,
}

pub(in crate::engine::compiler) fn mechanism_config(
    node_def: &NodeDefinition,
) -> WendaoSqlValidateMechanismConfig {
    WendaoSqlValidateMechanismConfig {
        surface_bundle_key: string_param(node_def, "surface_bundle_key")
            .unwrap_or_else(|| "surface_bundle_xml".to_string()),
        author_spec_key: string_param(node_def, "author_spec_key")
            .unwrap_or_else(|| "author_spec_xml".to_string()),
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "validated_sql".to_string()),
        report_key: string_param(node_def, "report_key")
            .unwrap_or_else(|| "validation_report_xml".to_string()),
        error_key: string_param(node_def, "error_key")
            .unwrap_or_else(|| "validation_error".to_string()),
        accepted_branch_label: string_param(node_def, "accepted_branch_label"),
        rejected_branch_label: string_param(node_def, "rejected_branch_label"),
    }
}
