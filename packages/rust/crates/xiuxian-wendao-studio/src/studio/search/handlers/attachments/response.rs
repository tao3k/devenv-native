use crate::studio::search::handlers::queries::AttachmentSearchQuery;
use crate::studio::types::AttachmentSearchResponse;
use crate::studio::{StudioApiError, StudioState};
use xiuxian_wendao::link_graph::LinkGraphAttachmentKind;
use xiuxian_wendao::search::{SearchCorpusKind, SearchPlaneCacheTtl};

const ATTACHMENT_SEARCH_CACHE_SCOPE: &str = "attachment";

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
            indexing_state: Some(status.indexing_state.into()),
            index_error: status.index_error,
        });
    }
    let cache_key = attachment_search_cache_key(
        studio,
        query_text,
        limit,
        extensions.as_slice(),
        kinds.as_slice(),
        query.case_sensitive,
    );
    if let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = studio
            .search_plane
            .cache_get_json::<AttachmentSearchResponse>(cache_key)
            .await
    {
        return Ok(cached);
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
    let response = AttachmentSearchResponse {
        query: query_text.to_string(),
        hit_count: hits.len(),
        hits,
        selected_scope: "attachments".to_string(),
        partial: false,
        indexing_state: Some("ready".into()),
        index_error: None,
    };
    if let Some(cache_key) = cache_key.as_ref() {
        studio
            .search_plane
            .cache_set_json(cache_key, SearchPlaneCacheTtl::HotQuery, &response)
            .await;
    }
    Ok(response)
}

fn attachment_search_cache_key(
    studio: &StudioState,
    query_text: &str,
    limit: usize,
    extensions: &[String],
    kinds: &[LinkGraphAttachmentKind],
    case_sensitive: bool,
) -> Option<String> {
    let intent = attachment_search_cache_intent(extensions, kinds, case_sensitive);
    studio.search_plane.search_query_cache_key(
        ATTACHMENT_SEARCH_CACHE_SCOPE,
        &[SearchCorpusKind::Attachment],
        query_text,
        limit,
        Some(intent.as_str()),
        None,
    )
}

fn attachment_search_cache_intent(
    extensions: &[String],
    kinds: &[LinkGraphAttachmentKind],
    case_sensitive: bool,
) -> String {
    let mut extensions = extensions.to_vec();
    extensions.sort_unstable();
    extensions.dedup();
    let mut kinds = kinds
        .iter()
        .map(|kind| attachment_kind_cache_label(*kind))
        .collect::<Vec<_>>();
    kinds.sort_unstable();
    kinds.dedup();
    format!(
        "ext:{}|kind:{}|case:{}",
        extensions.join(","),
        kinds.join(","),
        case_sensitive
    )
}

fn attachment_kind_cache_label(kind: LinkGraphAttachmentKind) -> &'static str {
    match kind {
        LinkGraphAttachmentKind::Image => "image",
        LinkGraphAttachmentKind::Pdf => "pdf",
        LinkGraphAttachmentKind::Gpg => "gpg",
        LinkGraphAttachmentKind::Document => "document",
        LinkGraphAttachmentKind::Archive => "archive",
        LinkGraphAttachmentKind::Audio => "audio",
        LinkGraphAttachmentKind::Video => "video",
        LinkGraphAttachmentKind::Other => "other",
    }
}
