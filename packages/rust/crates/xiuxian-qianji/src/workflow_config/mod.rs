//! Workflow/task-level configuration for Qianji-owned execution routes.
//!
//! Server/global defaults live in `resources/config/qianji.toml`; route
//! contracts for concrete workflow task families live under
//! `resources/config/workflows/` and user overlays under
//! `$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/workflows/`.

mod loader;
mod model;

pub use loader::{
    DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE, resolve_qianji_workflow_llm_task_config,
    resolve_qianji_workflow_llm_task_config_with_env,
};
pub use model::{
    QianjiWorkflowLlmEndpointConfig, QianjiWorkflowLlmTaskConfig, QianjiWorkflowLlmTaskRetryConfig,
    QianjiWorkflowLlmTaskRouteConfig,
};

#[cfg(test)]
#[path = "../../tests/unit/workflow_config/mod.rs"]
mod tests;
