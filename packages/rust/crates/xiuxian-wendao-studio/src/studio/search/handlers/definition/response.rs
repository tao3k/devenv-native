use std::sync::Arc;

use xiuxian_wendao_server::transport::DefinitionFlightRouteResponse;

use super::batch::{definition_hit_batch, definition_response_flight_app_metadata};
use super::path::normalize_source_path;
use crate::studio::search::observation_hints::definition_observation_hints;
use crate::studio::search::{
    DefinitionResolveOptions, resolve_best_definition, resolve_definition_candidates,
};
use crate::studio::types::DefinitionResolveResponse;
use crate::studio::{StudioApiError, StudioState};

pub(crate) async fn build_definition_response(
    studio: &StudioState,
    query_text: &str,
    source_path: Option<&str>,
    source_line: Option<usize>,
) -> Result<DefinitionResolveResponse, StudioApiError> {
    let query_text = query_text.trim();
    if query_text.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_QUERY",
            "Definition search requires a non-empty query",
        ));
    }

    let normalized_source_path =
        source_path.map(|path| normalize_source_path(studio.project_root.as_path(), path));
    let source_paths = normalized_source_path
        .as_ref()
        .map(std::slice::from_ref)
        .filter(|paths| !paths.is_empty());
    let observation_hints =
        definition_observation_hints(studio, source_paths, source_line, query_text).await;
    studio.ensure_local_symbol_index_ready().await?;
    let ast_hits = studio.search_local_symbol_hits(query_text, 256).await?;
    let projects = studio.configured_projects();
    let options = DefinitionResolveOptions {
        scope_patterns: observation_hints.as_ref().and_then(|hints| {
            (!hints.scope_patterns.is_empty()).then_some(hints.scope_patterns.clone())
        }),
        languages: observation_hints
            .as_ref()
            .and_then(|hints| (!hints.languages.is_empty()).then_some(hints.languages.clone())),
        preferred_source_path: normalized_source_path.clone(),
        ..DefinitionResolveOptions::default()
    };
    let candidates = resolve_definition_candidates(
        query_text,
        ast_hits.as_slice(),
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        projects.as_slice(),
        &options,
    );
    let Some(definition) = resolve_best_definition(
        query_text,
        ast_hits.as_slice(),
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        projects.as_slice(),
        &options,
    ) else {
        return Err(StudioApiError::not_found("Definition not found"));
    };
    let navigation_target = definition.navigation_target.clone();

    Ok(DefinitionResolveResponse {
        query: query_text.to_string(),
        source_path: normalized_source_path,
        source_line,
        candidate_count: candidates.len(),
        selected_scope: "definition".to_string(),
        navigation_target: navigation_target.clone(),
        definition: definition.clone(),
        resolved_target: Some(navigation_target),
        resolved_hit: Some(definition),
    })
}

pub(super) async fn load_definition_flight_response(
    studio: Arc<StudioState>,
    query_text: &str,
    source_path: Option<&str>,
    source_line: Option<usize>,
) -> Result<DefinitionFlightRouteResponse, StudioApiError> {
    let response =
        build_definition_response(studio.as_ref(), query_text, source_path, source_line).await?;
    let batch = definition_hit_batch(&response.definition).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_DEFINITION_FLIGHT_BATCH_FAILED",
            "Failed to materialize definition result through the Flight-backed provider",
            Some(error),
        )
    })?;
    let app_metadata = definition_response_flight_app_metadata(&response).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_DEFINITION_FLIGHT_METADATA_FAILED",
            "Failed to encode definition Flight app metadata",
            Some(error),
        )
    })?;
    Ok(DefinitionFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}
