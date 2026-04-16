use std::path::Path;

use crate::gateway::studio::{GatewayState, StudioApiError};
use crate::gateway::studio::search::handlers::queries::AstSearchQuery;
use crate::gateway::studio::search::project_scope::project_metadata_for_path;
use crate::gateway::studio::types::{AstSearchHit, AstSearchResponse, UiProjectConfig};
use crate::search::SearchCorpusKind;

pub(crate) async fn load_ast_search_response(
    state: &GatewayState,
    query: AstSearchQuery,
) -> Result<AstSearchResponse, StudioApiError> {
    let raw_query = query.q.unwrap_or_default();
    let query_text = raw_query.trim();
    if query_text.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_QUERY",
            "AST search requires a non-empty query",
        ));
    }

    let limit = query.limit.unwrap_or(20).max(1);
    state.studio.ensure_local_symbol_index_started()?;
    let status = state
        .studio
        .local_corpus_bootstrap_status(SearchCorpusKind::LocalSymbol, "ast_search");
    if !status.active_epoch_ready {
        state.studio.record_local_corpus_partial_search_response(
            SearchCorpusKind::LocalSymbol,
            "ast_search",
        );
        return Ok(AstSearchResponse {
            query: query_text.to_string(),
            hit_count: 0,
            hits: Vec::new(),
            selected_scope: "definitions".to_string(),
            partial: true,
            indexing_state: Some(status.indexing_state.to_string()),
            index_error: status.index_error,
        });
    }
    let ast_hits = state
        .studio
        .search_local_symbol_hits(query_text, limit)
        .await?;
    let projects = state.studio.configured_projects();
    let mut hits = ast_hits
        .iter()
        .map(|hit| {
            enrich_ast_hit(
                hit,
                state.studio.project_root.as_path(),
                state.studio.config_root.as_path(),
                projects.as_slice(),
            )
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line_start.cmp(&right.line_start))
    });
    hits.truncate(limit);

    state
        .studio
        .record_local_corpus_ready_search_response(SearchCorpusKind::LocalSymbol, "ast_search");
    Ok(AstSearchResponse {
        query: query_text.to_string(),
        hit_count: hits.len(),
        hits,
        selected_scope: "definitions".to_string(),
        partial: false,
        indexing_state: Some("ready".to_string()),
        index_error: None,
    })
}

fn enrich_ast_hit(
    hit: &AstSearchHit,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> AstSearchHit {
    let metadata =
        project_metadata_for_path(project_root, config_root, projects, hit.path.as_str());
    let mut navigation_target = hit.navigation_target.clone();
    navigation_target
        .project_name
        .clone_from(&metadata.project_name);
    navigation_target
        .root_label
        .clone_from(&metadata.root_label);

    let mut enriched = hit.clone();
    enriched.project_name = metadata.project_name;
    enriched.root_label = metadata.root_label;
    enriched.navigation_target = navigation_target;
    if enriched.score <= 0.0 {
        enriched.score = ast_hit_score(&enriched);
    }
    enriched
}

fn ast_hit_score(hit: &AstSearchHit) -> f64 {
    if hit.language != "markdown" {
        return 0.95;
    }

    match hit.node_kind.as_deref() {
        Some("task") => 0.88,
        Some("property" | "observation") => 0.8,
        _ => 0.95,
    }
}
