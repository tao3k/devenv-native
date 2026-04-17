use std::path::PathBuf;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::{Value, json};
use xiuxian_qianji::{
    BootcampLlmMode, BootcampRunOptions, WENDAO_SQL_AUTHORING_V1_TOML, WorkflowReport,
    run_workflow_from_manifest_toml,
};

use super::format::render_search_report;
use super::runtime::WendaoSearchToolConfig;
use crate::agent::native_tools::registry::{NativeTool, NativeToolCallContext};
use crate::config::load_xiuxian_config;

/// Native direct-gateway bounded Wendao search tool.
#[derive(Clone)]
pub struct WendaoSearchTool {
    config_override: Option<WendaoSearchToolConfig>,
    llm_mode: BootcampLlmMode,
}

impl WendaoSearchTool {
    /// Creates one production-configured bounded Wendao search tool.
    #[must_use]
    pub fn new(config: WendaoSearchToolConfig) -> Self {
        Self::new_with_llm_mode(config, BootcampLlmMode::RuntimeDefault)
    }

    /// Creates one runtime-configured bounded Wendao search tool.
    #[must_use]
    pub fn new_runtime_default() -> Self {
        Self {
            config_override: None,
            llm_mode: BootcampLlmMode::RuntimeDefault,
        }
    }

    /// Creates one bounded Wendao search tool with an explicit bootcamp LLM mode.
    #[must_use]
    pub fn new_with_llm_mode(config: WendaoSearchToolConfig, llm_mode: BootcampLlmMode) -> Self {
        Self {
            config_override: Some(config),
            llm_mode,
        }
    }
}

#[async_trait]
impl NativeTool for WendaoSearchTool {
    fn name(&self) -> &'static str {
        "wendao.search"
    }

    fn description(&self) -> &'static str {
        "Compatibility alias for `knowledge.search`. Use `knowledge.search` for new repo-specific knowledge questions; this name remains available for callers that still reference `wendao.search`."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural-language knowledge search query. Examples: 'find Daochang architecture docs', 'list Wendao SQL tables', or 'what does the memory gate do?'."
                },
                "request": {
                    "type": "string",
                    "description": "Legacy alias for `query`. Prefer `query` for new tool calls.",
                    "deprecated": true
                },
                "project_root": {
                    "type": "string",
                    "description": "Optional explicit project-root override. Usually omit this unless the user explicitly wants a different repo or workspace."
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    async fn call(
        &self,
        arguments: Option<Value>,
        context: &NativeToolCallContext,
    ) -> Result<String> {
        let config = self.resolve_config()?;
        let request = required_query(arguments.as_ref())?;
        let explicit_project_root = optional_string(arguments.as_ref(), "project_root");
        let project_root = config.resolve_project_root(
            explicit_project_root.as_deref(),
            context.session_id.as_deref(),
        );

        let report = self
            .run_bounded_search(
                request.as_str(),
                project_root.as_str(),
                config.query_endpoint(),
                context,
            )
            .await?;
        Ok(render_search_report(
            request.as_str(),
            project_root.as_str(),
            &report,
        ))
    }
}

impl WendaoSearchTool {
    fn resolve_config(&self) -> Result<WendaoSearchToolConfig> {
        if let Some(config) = &self.config_override {
            return Ok(config.clone());
        }

        let config = load_xiuxian_config();
        WendaoSearchToolConfig::from_xiuxian_config(&config).ok_or_else(|| {
            anyhow!(
                "`wendao.search` requires `[wendao_gateway].query_endpoint` in xiuxian.toml runtime config"
            )
        })
    }

    async fn run_bounded_search(
        &self,
        request: &str,
        project_root: &str,
        query_endpoint: &str,
        context: &NativeToolCallContext,
    ) -> Result<WorkflowReport> {
        run_workflow_from_manifest_toml(
            WENDAO_SQL_AUTHORING_V1_TOML,
            json!({
                "request": request,
                "project_root": project_root,
                "wendao_query_endpoint": query_endpoint,
            }),
            BootcampRunOptions {
                repo_path: Some(PathBuf::from(project_root)),
                session_id: context.session_id.clone(),
                llm_mode: self.llm_mode.clone(),
                ..BootcampRunOptions::default()
            },
        )
        .await
        .map_err(|error| anyhow!("bounded Wendao search workflow failed: {error}"))
    }
}

fn required_query(arguments: Option<&Value>) -> Result<String> {
    optional_string(arguments, "query")
        .or_else(|| optional_string(arguments, "request"))
        .ok_or_else(|| anyhow!("Missing `query` argument"))
}

fn optional_string(arguments: Option<&Value>, key: &str) -> Option<String> {
    arguments
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
