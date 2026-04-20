use crate::contracts::{NodeDefinition, QianjiMechanism};
use std::sync::Arc;

use crate::engine::compiler::wendao_sql;

pub(in crate::engine::compiler) fn wendao_sql_discover(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = wendao_sql::discover_mechanism_config(node_def);
    Arc::new(crate::executors::WendaoSqlDiscoverMechanism {
        output_key: cfg.output_key,
        endpoint_key: cfg.endpoint_key,
        endpoint: cfg.endpoint,
        project_root_key: cfg.project_root_key,
        allowed_objects: cfg.allowed_objects,
        max_limit: cfg.max_limit,
        allowed_ops: cfg.allowed_ops,
        require_filter_for: cfg.require_filter_for,
    })
}

pub(in crate::engine::compiler) fn wendao_sql_validate(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = wendao_sql::validate_mechanism_config(node_def);
    Arc::new(crate::executors::WendaoSqlValidateMechanism {
        surface_bundle_key: cfg.surface_bundle_key,
        author_spec_key: cfg.author_spec_key,
        output_key: cfg.output_key,
        report_key: cfg.report_key,
        error_key: cfg.error_key,
        accepted_branch_label: cfg.accepted_branch_label,
        rejected_branch_label: cfg.rejected_branch_label,
    })
}

pub(in crate::engine::compiler) fn wendao_sql_execute(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = wendao_sql::execute_mechanism_config(node_def);
    Arc::new(crate::executors::WendaoSqlExecuteMechanism {
        sql_key: cfg.sql_key,
        output_key: cfg.output_key,
        report_key: cfg.report_key,
        error_key: cfg.error_key,
        endpoint_key: cfg.endpoint_key,
        endpoint: cfg.endpoint,
        success_branch_label: cfg.success_branch_label,
        error_branch_label: cfg.error_branch_label,
        max_report_rows: cfg.max_report_rows,
    })
}
