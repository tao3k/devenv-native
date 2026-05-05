//! Wendao search runtime loading and request execution.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::XiuxianConfig;

/// Runtime configuration for the bounded native `wendao.search` tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoSearchToolConfig {
    query_endpoint: String,
    default_project_root: Option<String>,
    session_project_roots: HashMap<String, String>,
}

impl WendaoSearchToolConfig {
    /// Builds one bounded search config from unified Xiuxian config.
    #[must_use]
    pub fn from_xiuxian_config(config: &XiuxianConfig) -> Option<Self> {
        let query_endpoint = normalized_non_empty(config.wendao_gateway.query_endpoint.as_deref())?;
        let default_project_root =
            normalized_non_empty(config.wendao_gateway.default_project_root.as_deref());
        let session_project_roots = config
            .wendao_gateway
            .session_project_roots
            .iter()
            .filter_map(|(session_id, project_root)| {
                let session_id = normalized_non_empty(Some(session_id.as_str()))?;
                let project_root = normalized_non_empty(Some(project_root.as_str()))?;
                Some((session_id, project_root))
            })
            .collect::<HashMap<_, _>>();

        Some(Self {
            query_endpoint,
            default_project_root,
            session_project_roots,
        })
    }

    /// Creates one explicit bounded search config.
    #[must_use]
    pub fn new(
        query_endpoint: impl Into<String>,
        default_project_root: Option<String>,
        session_project_roots: HashMap<String, String>,
    ) -> Self {
        let default_project_root =
            default_project_root.and_then(|value| normalized_non_empty(Some(value.as_str())));
        let session_project_roots = session_project_roots
            .into_iter()
            .filter_map(|(session_id, project_root)| {
                let session_id = normalized_non_empty(Some(session_id.as_str()))?;
                let project_root = normalized_non_empty(Some(project_root.as_str()))?;
                Some((session_id, project_root))
            })
            .collect::<HashMap<_, _>>();
        Self {
            query_endpoint: query_endpoint.into().trim().to_string(),
            default_project_root,
            session_project_roots,
        }
    }

    /// Returns the configured Wendao SQL/REST query endpoint.
    #[must_use]
    pub fn query_endpoint(&self) -> &str {
        self.query_endpoint.as_str()
    }

    /// Resolves the effective project root for one tool call.
    #[must_use]
    pub fn resolve_project_root(
        &self,
        explicit_project_root: Option<&str>,
        session_id: Option<&str>,
    ) -> String {
        normalized_non_empty(explicit_project_root)
            .or_else(|| {
                session_id
                    .and_then(|session_id| self.session_project_roots.get(session_id))
                    .cloned()
            })
            .or_else(|| self.default_project_root.clone())
            .unwrap_or_else(fallback_project_root)
    }
}

fn normalized_non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn fallback_project_root() -> String {
    std::env::var("PRJ_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|path| path.display().to_string())
        })
        .unwrap_or_else(|| PathBuf::from(".").display().to_string())
}
