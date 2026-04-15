use std::sync::Arc;

use crate::gateway::studio::router::error::StudioApiError;
use crate::gateway::studio::router::state::helpers::graph_include_dirs;
use crate::gateway::studio::router::state::types::{GatewayState, StudioState};
#[cfg(test)]
use crate::gateway::studio::symbol_index::{SymbolIndexPhase, SymbolIndexStatus};
use crate::gateway::studio::types::SearchIndexStatusResponse;
use crate::link_graph::LinkGraphIndex;
#[cfg(test)]
use crate::unified_symbol::UnifiedSymbolIndex;

impl GatewayState {
    pub(crate) async fn link_graph_index(&self) -> Result<Arc<LinkGraphIndex>, StudioApiError> {
        self.studio.graph_index().await
    }
}

impl StudioState {
    pub(crate) async fn graph_index(&self) -> Result<Arc<LinkGraphIndex>, StudioApiError> {
        let project_root = self.project_root.clone();
        let config_root = self.config_root.clone();
        let configured_projects = self.configured_projects();
        if configured_projects.is_empty() {
            return Err(StudioApiError::bad_request(
                "UI_CONFIG_REQUIRED",
                "Studio graph access requires configured link_graph.projects",
            ));
        }

        let build = tokio::task::spawn_blocking(move || {
            let include_dirs = graph_include_dirs(
                project_root.as_path(),
                config_root.as_path(),
                &configured_projects,
            );
            if include_dirs.is_empty() {
                Err(
                    "configured link_graph.projects did not produce any graph include dirs"
                        .to_string(),
                )
            } else {
                LinkGraphIndex::build_with_cache_with_meta(
                    project_root.as_path(),
                    &include_dirs,
                    &[],
                )
                .map(|(index, _meta)| index)
                .or_else(|_| {
                    LinkGraphIndex::build_with_filters(project_root.as_path(), &include_dirs, &[])
                })
            }
        })
        .await
        .map_err(|error: tokio::task::JoinError| {
            StudioApiError::internal(
                "LINK_GRAPH_BUILD_PANIC",
                "Failed to build link graph index",
                Some(error.to_string()),
            )
        })?;
        let index = Arc::new(build.map_err(|error: String| {
            StudioApiError::internal(
                "LINK_GRAPH_BUILD_FAILED",
                "Failed to build link graph index",
                Some(error),
            )
        })?);
        Ok(index)
    }

    #[cfg(test)]
    pub(crate) fn current_symbol_index(&self) -> Option<Arc<UnifiedSymbolIndex>> {
        self.symbol_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(Arc::clone)
    }

    #[cfg(test)]
    pub(crate) fn symbol_index_status(&self) -> Result<SymbolIndexStatus, StudioApiError> {
        let configured_projects = self.configured_projects();

        if configured_projects.is_empty() {
            return Err(StudioApiError::bad_request(
                "UI_CONFIG_REQUIRED",
                "Studio symbol search requires configured link_graph.projects",
            ));
        }
        self.ensure_local_symbol_index_started()?;

        let current_status = self.symbol_index_coordinator.status();
        let current_index = self.current_symbol_index();
        if current_index.is_none() && matches!(current_status.phase, SymbolIndexPhase::Idle) {
            self.record_deferred_bootstrap_background_indexing_activation("symbol_index_status");
        }

        self.symbol_index_coordinator
            .ensure_started(configured_projects, Arc::clone(&self.symbol_index));
        Ok(self.symbol_index_coordinator.status())
    }

    pub(crate) async fn search_index_status(&self) -> SearchIndexStatusResponse {
        let snapshot = self.search_plane.status_with_repo_runtime().await;
        self.record_local_corpus_ready_observations_from_snapshot(&snapshot, "search_index_status");
        SearchIndexStatusResponse::from_snapshot_with_diagnostics(&snapshot).await
    }
}
