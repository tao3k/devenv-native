#[cfg(test)]
use crate::studio::search::handlers::knowledge::intent::types::configured_parquet_query_engine_label;
use crate::studio::search::handlers::knowledge::intent::types::{
    IntentIndexState, IntentSourceHits,
};
use crate::studio::search::handlers::knowledge::intent_policy::is_index_not_ready;
use crate::studio::{StudioApiError, StudioState};

pub(crate) async fn search_intent_sources(
    studio: &StudioState,
    query_text: &str,
    candidate_limit: usize,
    index_state: &IntentIndexState,
) -> Result<IntentSourceHits, StudioApiError> {
    #[cfg(test)]
    let shared_query_engine =
        if index_state.knowledge_config_missing && index_state.symbol_config_missing {
            None
        } else {
            #[cfg(feature = "duckdb")]
            {
                Some(resolve_intent_source_query_engine_label(studio)?)
            }
            #[cfg(not(feature = "duckdb"))]
            {
                Some(resolve_intent_source_query_engine_label(studio))
            }
        };
    let (knowledge_result, symbol_result) = tokio::join!(
        async {
            if index_state.knowledge_config_missing {
                Ok(Vec::new())
            } else {
                studio
                    .search_knowledge_sections(query_text, candidate_limit)
                    .await
            }
        },
        async {
            if index_state.symbol_config_missing {
                Ok(Vec::new())
            } else {
                studio
                    .search_local_symbol_hits(query_text, candidate_limit)
                    .await
            }
        }
    );

    let (knowledge_hits, knowledge_indexing) = decode_intent_source_result(knowledge_result)?;
    let (local_symbol_hits, local_symbol_indexing) = decode_intent_source_result(symbol_result)?;
    Ok(IntentSourceHits {
        knowledge_hits,
        local_symbol_hits,
        knowledge_indexing,
        local_symbol_indexing,
        #[cfg(test)]
        knowledge_query_engine: (!index_state.knowledge_config_missing)
            .then_some(shared_query_engine)
            .flatten(),
        #[cfg(test)]
        local_symbol_query_engine: (!index_state.symbol_config_missing)
            .then_some(shared_query_engine)
            .flatten(),
    })
}

#[cfg(all(test, feature = "duckdb"))]
fn resolve_intent_source_query_engine_label(
    studio: &StudioState,
) -> Result<&'static str, StudioApiError> {
    configured_parquet_query_engine_label(&studio.search_plane).map_err(|error| {
        StudioApiError::internal(
            "INTENT_SOURCE_QUERY_ENGINE_FAILED",
            "Failed to resolve intent source query-engine metadata",
            Some(error),
        )
    })
}

#[cfg(all(test, not(feature = "duckdb")))]
fn resolve_intent_source_query_engine_label(studio: &StudioState) -> &'static str {
    configured_parquet_query_engine_label(&studio.search_plane)
}

fn decode_intent_source_result<T>(
    result: Result<Vec<T>, StudioApiError>,
) -> Result<(Vec<T>, bool), StudioApiError> {
    match result {
        Ok(hits) => Ok((hits, false)),
        Err(error) if is_index_not_ready(&error) => Ok((Vec::new(), true)),
        Err(error) => Err(error),
    }
}
