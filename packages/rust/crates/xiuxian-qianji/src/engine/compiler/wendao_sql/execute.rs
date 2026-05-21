use crate::contracts::NodeDefinition;

use super::params::{string_param, usize_param};

pub(in crate::engine::compiler) struct WendaoSqlExecuteMechanismConfig {
    pub(in crate::engine::compiler) sql_key: String,
    pub(in crate::engine::compiler) output_key: String,
    pub(in crate::engine::compiler) report_key: String,
    pub(in crate::engine::compiler) error_key: String,
    pub(in crate::engine::compiler) endpoint_key: Option<String>,
    pub(in crate::engine::compiler) endpoint: Option<String>,
    pub(in crate::engine::compiler) success_branch_label: Option<String>,
    pub(in crate::engine::compiler) error_branch_label: Option<String>,
    pub(in crate::engine::compiler) max_report_rows: usize,
}

pub(in crate::engine::compiler) fn mechanism_config(
    node_def: &NodeDefinition,
) -> WendaoSqlExecuteMechanismConfig {
    WendaoSqlExecuteMechanismConfig {
        sql_key: string_param(node_def, "sql_key").unwrap_or_else(|| "validated_sql".to_string()),
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "sql_query_payload".to_string()),
        report_key: string_param(node_def, "report_key")
            .unwrap_or_else(|| "execution_report_xml".to_string()),
        error_key: string_param(node_def, "error_key")
            .unwrap_or_else(|| "execution_error".to_string()),
        endpoint_key: string_param(node_def, "endpoint_key"),
        endpoint: string_param(node_def, "endpoint"),
        success_branch_label: string_param(node_def, "success_branch_label"),
        error_branch_label: string_param(node_def, "error_branch_label"),
        max_report_rows: usize_param(node_def, "max_report_rows").unwrap_or(3),
    }
}
