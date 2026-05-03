use std::path::Path;
use std::sync::Arc;

use crate::studio::router::error::StudioApiError;
use crate::studio::router::state::helpers::graph_include_dirs;
use crate::studio::router::state::types::{
    GatewayState, GraphIndexCacheEntry, GraphSourceSignature, StudioState,
};
#[cfg(test)]
use crate::studio::symbol_index::{SymbolIndexPhase, SymbolIndexStatus};
use crate::studio::types::SearchIndexStatusResponse;
use walkdir::WalkDir;
use xiuxian_wendao::link_graph::LinkGraphIndex;
use xiuxian_wendao::parsers::markdown::is_supported_note;
#[cfg(test)]
use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

impl GatewayState {
    pub(crate) async fn link_graph_index(&self) -> Result<Arc<LinkGraphIndex>, StudioApiError> {
        self.studio.graph_index().await
    }
}

impl StudioState {
    pub(crate) async fn graph_index(&self) -> Result<Arc<LinkGraphIndex>, StudioApiError> {
        if let Some(index) = self.cached_graph_index() {
            return Ok(index);
        }

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
        self.store_graph_index(Arc::clone(&index));
        Ok(index)
    }

    fn cached_graph_index(&self) -> Option<Arc<LinkGraphIndex>> {
        let entry = self
            .graph_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()?;
        if self.graph_source_signature() != entry.source_signature {
            return None;
        }
        Some(entry.index)
    }

    fn store_graph_index(&self, index: Arc<LinkGraphIndex>) {
        let source_signature = self.graph_source_signature();
        let mut guard = self
            .graph_index
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(GraphIndexCacheEntry {
            index,
            source_signature,
        });
    }

    fn graph_source_signature(&self) -> GraphSourceSignature {
        let include_dirs = graph_include_dirs(
            self.project_root.as_path(),
            self.config_root.as_path(),
            &self.configured_projects(),
        );
        graph_source_signature(self.project_root.as_path(), &include_dirs)
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
        let current_status = self.symbol_index_coordinator.status();
        let current_index = self.current_symbol_index();
        self.ensure_local_symbol_index_started()?;
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

fn graph_source_signature(root: &Path, include_dirs: &[String]) -> GraphSourceSignature {
    include_dirs
        .iter()
        .map(|include_dir| graph_source_signature_in_dir(root, include_dir))
        .fold(
            GraphSourceSignature::default(),
            merge_graph_source_signature,
        )
}

fn graph_source_signature_in_dir(root: &Path, include_dir: &str) -> GraphSourceSignature {
    let base = if include_dir == "." {
        root.to_path_buf()
    } else {
        root.join(include_dir)
    };
    let mut signature = GraphSourceSignature::default();
    for entry in WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_supported_note(entry.path()))
    {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        signature.note_count = signature.note_count.saturating_add(1);
        signature.total_size_bytes = signature.total_size_bytes.saturating_add(metadata.len());
        if let Ok(modified_at) = metadata.modified() {
            signature.latest_modified_at = Some(
                signature
                    .latest_modified_at
                    .map_or(modified_at, |current| current.max(modified_at)),
            );
        }
    }
    signature
}

fn merge_graph_source_signature(
    mut left: GraphSourceSignature,
    right: GraphSourceSignature,
) -> GraphSourceSignature {
    left.note_count = left.note_count.saturating_add(right.note_count);
    left.total_size_bytes = left.total_size_bytes.saturating_add(right.total_size_bytes);
    left.latest_modified_at = match (left.latest_modified_at, right.latest_modified_at) {
        (Some(left_ts), Some(right_ts)) => Some(left_ts.max(right_ts)),
        (Some(left_ts), None) => Some(left_ts),
        (None, Some(right_ts)) => Some(right_ts),
        (None, None) => None,
    };
    left
}
