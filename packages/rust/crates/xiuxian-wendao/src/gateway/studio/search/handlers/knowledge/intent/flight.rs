use std::sync::Arc;

#[cfg(test)]
use async_trait::async_trait;
use xiuxian_vector_store::{
    LanceArrayRef, LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema,
    LanceStringArray,
};
use xiuxian_wendao_runtime::transport::SearchFlightRouteResponse;
#[cfg(test)]
use xiuxian_wendao_runtime::transport::{SEARCH_INTENT_ROUTE, SearchFlightRouteProvider};

use super::entry::build_intent_search_response_with_metadata;
use crate::gateway::studio::types::{SearchHit, SearchResponse};
use crate::gateway::studio::{StudioApiError, StudioState};

/// Studio-backed Flight provider for the semantic `/search/intent` route.
#[derive(Clone)]
#[cfg(test)]
pub struct StudioIntentSearchFlightRouteProvider {
    studio: Arc<StudioState>,
}

#[cfg(test)]
impl std::fmt::Debug for StudioIntentSearchFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioIntentSearchFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
impl StudioIntentSearchFlightRouteProvider {
    /// Create one Studio-backed intent-search Flight provider.
    #[must_use]
    pub fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

#[async_trait]
#[cfg(test)]
impl SearchFlightRouteProvider for StudioIntentSearchFlightRouteProvider {
    async fn search_batch(
        &self,
        route: &str,
        query_text: &str,
        limit: usize,
        intent: Option<&str>,
        repo_hint: Option<&str>,
    ) -> Result<SearchFlightRouteResponse, String> {
        if route != SEARCH_INTENT_ROUTE {
            return Err(format!(
                "studio intent Flight provider only supports route `{SEARCH_INTENT_ROUTE}`, got `{route}`"
            ));
        }

        let (response, _transport_metadata) = build_intent_search_response_with_metadata(
            self.studio.as_ref(),
            query_text,
            query_text,
            repo_hint,
            limit,
            intent.map(ToString::to_string),
        )
        .await
        .map_err(|error| {
            format!(
                "studio intent Flight provider failed to build search response for `{query_text}`: {error:?}"
            )
        })?;

        let batch = search_hit_batch_from_hits(&response.hits)?;
        let app_metadata = search_response_flight_app_metadata(&response)?;
        Ok(SearchFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
    }
}

pub(crate) async fn load_intent_search_flight_response(
    studio: Arc<StudioState>,
    raw_query: &str,
    query_text: &str,
    repo_hint: Option<&str>,
    limit: usize,
    intent: Option<String>,
) -> Result<SearchFlightRouteResponse, StudioApiError> {
    let (response, _transport_metadata) = build_intent_search_response_with_metadata(
        studio.as_ref(),
        raw_query,
        query_text,
        repo_hint,
        limit,
        intent,
    )
    .await?;
    let batch = search_hit_batch_from_hits(&response.hits).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_INTENT_FLIGHT_BATCH_FAILED",
            "Failed to materialize intent hits through the Flight-backed provider",
            Some(error),
        )
    })?;
    let app_metadata = search_response_flight_app_metadata(&response).map_err(|error| {
        StudioApiError::internal(
            "SEARCH_INTENT_FLIGHT_METADATA_FAILED",
            "Failed to encode intent Flight app metadata",
            Some(error),
        )
    })?;
    Ok(SearchFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}

pub(crate) fn search_hit_batch_from_hits(hits: &[SearchHit]) -> Result<LanceRecordBatch, String> {
    let columns = SearchHitFlightColumns::from_hits(hits)?;
    LanceRecordBatch::try_new(
        search_hit_flight_schema(),
        columns.into_record_batch_columns(),
    )
    .map_err(|error| format!("failed to build search-hit Flight batch: {error}"))
}

fn search_hit_flight_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new("stem", LanceDataType::Utf8, false),
        LanceField::new("title", LanceDataType::Utf8, true),
        LanceField::new("path", LanceDataType::Utf8, false),
        LanceField::new("docType", LanceDataType::Utf8, true),
        LanceField::new("tagsJson", LanceDataType::Utf8, false),
        LanceField::new("score", LanceDataType::Float64, false),
        LanceField::new("bestSection", LanceDataType::Utf8, true),
        LanceField::new("matchReason", LanceDataType::Utf8, true),
        LanceField::new("hierarchicalUri", LanceDataType::Utf8, true),
        LanceField::new("hierarchyJson", LanceDataType::Utf8, true),
        LanceField::new("saliencyScore", LanceDataType::Float64, true),
        LanceField::new("auditStatus", LanceDataType::Utf8, true),
        LanceField::new("verificationState", LanceDataType::Utf8, true),
        LanceField::new("implicitBacklinksJson", LanceDataType::Utf8, true),
        LanceField::new("implicitBacklinkItemsJson", LanceDataType::Utf8, true),
        LanceField::new("navigationTargetJson", LanceDataType::Utf8, true),
    ]))
}

struct SearchHitFlightColumns<'a> {
    stems: Vec<&'a str>,
    titles: Vec<Option<&'a str>>,
    paths: Vec<&'a str>,
    doc_types: Vec<Option<&'a str>>,
    tags_json: Vec<String>,
    scores: Vec<f64>,
    best_sections: Vec<Option<&'a str>>,
    match_reasons: Vec<Option<&'a str>>,
    hierarchical_uris: Vec<Option<&'a str>>,
    hierarchy_json: Vec<Option<String>>,
    saliency_scores: Vec<Option<f64>>,
    audit_statuses: Vec<Option<&'a str>>,
    verification_states: Vec<Option<&'a str>>,
    implicit_backlinks_json: Vec<Option<String>>,
    implicit_backlink_items_json: Vec<Option<String>>,
    navigation_target_json: Vec<Option<String>>,
}

impl<'a> SearchHitFlightColumns<'a> {
    fn from_hits(hits: &'a [SearchHit]) -> Result<Self, String> {
        Ok(Self {
            stems: hits.iter().map(|hit| hit.stem.as_str()).collect(),
            titles: hits.iter().map(|hit| hit.title.as_deref()).collect(),
            paths: hits.iter().map(|hit| hit.path.as_str()).collect(),
            doc_types: hits.iter().map(|hit| hit.doc_type.as_deref()).collect(),
            tags_json: hits
                .iter()
                .map(|hit| serde_json::to_string(&hit.tags).map_err(|error| error.to_string()))
                .collect::<Result<Vec<_>, _>>()?,
            scores: hits.iter().map(|hit| hit.score).collect(),
            best_sections: hits.iter().map(|hit| hit.best_section.as_deref()).collect(),
            match_reasons: hits.iter().map(|hit| hit.match_reason.as_deref()).collect(),
            hierarchical_uris: hits
                .iter()
                .map(|hit| hit.hierarchical_uri.as_deref())
                .collect(),
            hierarchy_json: hits
                .iter()
                .map(|hit| optional_json_string(hit.hierarchy.as_ref()))
                .collect::<Result<Vec<_>, _>>()?,
            saliency_scores: hits.iter().map(|hit| hit.saliency_score).collect(),
            audit_statuses: hits.iter().map(|hit| hit.audit_status.as_deref()).collect(),
            verification_states: hits
                .iter()
                .map(|hit| hit.verification_state.as_deref())
                .collect(),
            implicit_backlinks_json: hits
                .iter()
                .map(|hit| optional_json_string(hit.implicit_backlinks.as_ref()))
                .collect::<Result<Vec<_>, _>>()?,
            implicit_backlink_items_json: hits
                .iter()
                .map(|hit| optional_json_string(hit.implicit_backlink_items.as_ref()))
                .collect::<Result<Vec<_>, _>>()?,
            navigation_target_json: hits
                .iter()
                .map(|hit| optional_json_string(hit.navigation_target.as_ref()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn into_record_batch_columns(self) -> Vec<LanceArrayRef> {
        vec![
            Arc::new(LanceStringArray::from(self.stems)),
            Arc::new(LanceStringArray::from(self.titles)),
            Arc::new(LanceStringArray::from(self.paths)),
            Arc::new(LanceStringArray::from(self.doc_types)),
            Arc::new(LanceStringArray::from(
                self.tags_json
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceFloat64Array::from(self.scores)),
            Arc::new(LanceStringArray::from(self.best_sections)),
            Arc::new(LanceStringArray::from(self.match_reasons)),
            Arc::new(LanceStringArray::from(self.hierarchical_uris)),
            Arc::new(LanceStringArray::from(
                self.hierarchy_json
                    .iter()
                    .map(|value| value.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceFloat64Array::from(self.saliency_scores)),
            Arc::new(LanceStringArray::from(self.audit_statuses)),
            Arc::new(LanceStringArray::from(self.verification_states)),
            Arc::new(LanceStringArray::from(
                self.implicit_backlinks_json
                    .iter()
                    .map(|value| value.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceStringArray::from(
                self.implicit_backlink_items_json
                    .iter()
                    .map(|value| value.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LanceStringArray::from(
                self.navigation_target_json
                    .iter()
                    .map(|value| value.as_deref())
                    .collect::<Vec<_>>(),
            )),
        ]
    }
}

fn optional_json_string<T>(value: Option<&T>) -> Result<Option<String>, String>
where
    T: serde::Serialize,
{
    value
        .map(|value| serde_json::to_string(value).map_err(|error| error.to_string()))
        .transpose()
}

pub(crate) fn search_response_flight_app_metadata(
    response: &SearchResponse,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "query": response.query,
        "hitCount": response.hit_count,
        "graphConfidenceScore": response.graph_confidence_score,
        "selectedMode": response.selected_mode,
        "intent": response.intent,
        "intentConfidence": response.intent_confidence,
        "searchMode": response.search_mode,
        "partial": response.partial,
        "indexingState": response.indexing_state,
        "pendingRepos": response.pending_repos,
        "skippedRepos": response.skipped_repos,
    }))
    .map_err(|error| format!("failed to encode search response Flight app_metadata: {error}"))
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/search/handlers/knowledge/intent/flight.rs"]
mod tests;
