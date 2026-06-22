use crate::runtime_config::constants::WORKFLOW_STATE_DUCKDB_FILE_NAME;
use crate::runtime_config::env_vars::{env_var_or_override, normalize_non_empty};
use crate::runtime_config::model::{QianjiRuntimeEnv, QianjiRuntimeWorkflowStateConfig};
use crate::runtime_config::pathing::resolve_qianji_data_root;
use crate::runtime_config::toml_config::QianjiTomlWorkflowState;
use std::path::{Path, PathBuf};

pub(super) fn resolve_qianji_runtime_workflow_state(
    file_workflow_state: &QianjiTomlWorkflowState,
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
) -> QianjiRuntimeWorkflowStateConfig {
    let local_duckdb_path = runtime_env
        .qianji_workflow_state_duckdb_path
        .clone()
        .or_else(|| {
            normalize_non_empty(file_workflow_state.local_duckdb_path.clone()).map(PathBuf::from)
        })
        .or_else(|| {
            env_var_or_override(runtime_env, "QIANJI_WORKFLOW_STATE_DUCKDB_PATH")
                .and_then(|value| normalize_non_empty(Some(value)))
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| default_workflow_state_duckdb_path(runtime_env, project_root));

    QianjiRuntimeWorkflowStateConfig {
        local_duckdb_path: resolve_against_project_root(project_root, local_duckdb_path),
    }
}

fn default_workflow_state_duckdb_path(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
) -> PathBuf {
    resolve_qianji_data_root(runtime_env, project_root)
        .join("duckdb")
        .join(WORKFLOW_STATE_DUCKDB_FILE_NAME)
}

fn resolve_against_project_root(project_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}
