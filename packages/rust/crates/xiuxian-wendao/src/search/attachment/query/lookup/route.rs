use crate::duckdb::ParquetQueryEngine;
use crate::gateway::studio::types::AttachmentSearchHit;
use crate::search::ranking::sort_by_rank;
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::decode::decode_attachment_hits;
use super::scan::execute_attachment_search;
use super::scoring::{
    build_query_tokens, compare_candidates, normalize_extension_filters, normalize_kind_filters,
    retained_window,
};
use super::types::{AttachmentCandidateQuery, AttachmentSearchError};

#[derive(Clone)]
struct PreparedAttachmentRead {
    query_engine: ParquetQueryEngine,
    table_name: String,
}

pub(crate) async fn search_attachment_hits(
    service: &SearchPlaneService,
    query: &str,
    limit: usize,
    extensions: &[String],
    kinds: &[crate::link_graph::LinkGraphAttachmentKind],
    case_sensitive: bool,
) -> Result<Vec<AttachmentSearchHit>, AttachmentSearchError> {
    let query_text = query.trim();
    if query_text.is_empty() {
        return Ok(Vec::new());
    }

    let normalized_extensions = normalize_extension_filters(extensions);
    let normalized_kinds = normalize_kind_filters(kinds);
    let prepared = prepare_attachment_read(service).await?;

    let normalized_query = if case_sensitive {
        query_text.to_string()
    } else {
        query_text.to_ascii_lowercase()
    };
    let query_tokens = build_query_tokens(normalized_query.as_str());
    let candidate_query = AttachmentCandidateQuery {
        case_sensitive,
        normalized_query: normalized_query.as_str(),
        query_tokens: query_tokens.as_slice(),
        extensions: &normalized_extensions,
        kinds: &normalized_kinds,
        window: retained_window(limit),
    };
    let execution = execute_attachment_search(
        &prepared.query_engine,
        prepared.table_name.as_str(),
        &candidate_query,
    )
    .await?;
    let mut candidates = execution.candidates;
    sort_by_rank(&mut candidates, compare_candidates);
    candidates.truncate(limit);
    let hits = decode_attachment_hits(
        &prepared.query_engine,
        prepared.table_name.as_str(),
        candidates,
    )
    .await?;
    service.record_query_telemetry(
        SearchCorpusKind::Attachment,
        execution
            .telemetry
            .finish(execution.source, None, hits.len()),
    );
    Ok(hits)
}

async fn prepare_attachment_read(
    service: &SearchPlaneService,
) -> Result<PreparedAttachmentRead, AttachmentSearchError> {
    let status = service
        .coordinator()
        .status_for(SearchCorpusKind::Attachment);
    let Some(active_epoch) = status.active_epoch else {
        return Err(AttachmentSearchError::NotReady);
    };

    let parquet_path = service.local_epoch_parquet_path(SearchCorpusKind::Attachment, active_epoch);
    if !parquet_path.exists() {
        return Err(AttachmentSearchError::NotReady);
    }
    let table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::Attachment,
        active_epoch,
    );
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    query_engine
        .ensure_parquet_table_registered(table_name.as_str(), parquet_path.as_path())
        .await?;

    Ok(PreparedAttachmentRead {
        query_engine,
        table_name,
    })
}
