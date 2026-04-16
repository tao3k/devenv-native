use crate::gateway::studio::search::handlers::queries::AttachmentSearchQuery;
use crate::gateway::studio::types::AttachmentSearchResponse;
use crate::gateway::studio::{StudioApiError, StudioState};
use crate::link_graph::LinkGraphAttachmentKind;
use crate::search::SearchCorpusKind;

pub(crate) async fn load_attachment_search_response_from_studio(
    studio: &StudioState,
    query: AttachmentSearchQuery,
) -> Result<AttachmentSearchResponse, StudioApiError> {
    let raw_query = query.q.unwrap_or_default();
    let query_text = raw_query.trim();
    if query_text.is_empty() {
        return Err(StudioApiError::bad_request(
            "MISSING_QUERY",
            "Attachment search requires a non-empty query",
        ));
    }

    let limit = query.limit.unwrap_or(20).max(1);
    let extensions = query
        .ext
        .iter()
        .map(|value| value.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let kinds = query
        .kind
        .iter()
        .map(|value| LinkGraphAttachmentKind::from_alias(value))
        .collect::<Vec<_>>();
    studio.ensure_attachment_index_started()?;
    let status =
        studio.local_corpus_bootstrap_status(SearchCorpusKind::Attachment, "attachment_search");
    if !status.active_epoch_ready {
        studio.record_local_corpus_partial_search_response(
            SearchCorpusKind::Attachment,
            "attachment_search",
        );
        return Ok(AttachmentSearchResponse {
            query: query_text.to_string(),
            hit_count: 0,
            hits: Vec::new(),
            selected_scope: "attachments".to_string(),
            partial: true,
            indexing_state: Some(status.indexing_state.to_string()),
            index_error: status.index_error,
        });
    }
    let hits = studio
        .search_attachment_hits(
            query_text,
            limit,
            extensions.as_slice(),
            kinds.as_slice(),
            query.case_sensitive,
        )
        .await?;

    studio.record_local_corpus_ready_search_response(
        SearchCorpusKind::Attachment,
        "attachment_search",
    );
    Ok(AttachmentSearchResponse {
        query: query_text.to_string(),
        hit_count: hits.len(),
        hits,
        selected_scope: "attachments".to_string(),
        partial: false,
        indexing_state: Some("ready".to_string()),
        index_error: None,
    })
}
