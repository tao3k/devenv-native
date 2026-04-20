use std::path::PathBuf;

use xiuxian_db_store::EngineRecordBatch;

use crate::duckdb::{LocalRelationEngineKind, ParquetQueryEngine};
use crate::search::{SearchCorpusKind, SearchPlaneService};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PublishedParquetQueryExecution {
    pub(crate) corpus: SearchCorpusKind,
    pub(crate) table_name: String,
    pub(crate) engine_kind: LocalRelationEngineKind,
    pub(crate) batches: Vec<EngineRecordBatch>,
}

#[cfg(feature = "duckdb")]
pub(crate) fn configured_parquet_query_engine(
    _service: &SearchPlaneService,
) -> Result<ParquetQueryEngine, String> {
    ParquetQueryEngine::configured().map_err(|error| {
        format!("shared published parquet query-engine configuration failed: {error}")
    })
}

#[cfg(not(feature = "duckdb"))]
pub(crate) fn configured_parquet_query_engine(
    service: &SearchPlaneService,
) -> Result<ParquetQueryEngine, String> {
    if !service.project_root().exists() {
        return Err(format!(
            "shared published parquet query-engine configuration failed: project root `{}` does not exist",
            service.project_root().display()
        ));
    }
    Ok(ParquetQueryEngine::configured(
        service.datafusion_query_engine().clone(),
    ))
}

pub(crate) async fn try_execute_published_parquet_query(
    service: &SearchPlaneService,
    query_engine: Option<&ParquetQueryEngine>,
    query_text: &str,
) -> Result<Option<PublishedParquetQueryExecution>, String> {
    let Some(target) = resolve_published_parquet_statement_target(service, query_text).await else {
        return Ok(None);
    };
    let query_engine = query_engine
        .cloned()
        .map_or_else(|| configured_parquet_query_engine(service), Ok)?;
    let engine_kind = query_engine.kind();
    query_engine
        .ensure_parquet_table_registered(target.table_name.as_str(), target.parquet_path.as_path())
        .await
        .map_err(|error| {
            format!(
                "shared published parquet query failed to register table `{}`: {error}",
                target.table_name
            )
        })?;
    let batches = query_engine
        .query_batches(query_text)
        .await
        .map_err(|error| {
            format!("shared published parquet SQL execution failed for `{query_text}`: {error}")
        })?;
    Ok(Some(PublishedParquetQueryExecution {
        corpus: target.corpus,
        table_name: target.table_name,
        engine_kind,
        batches,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedPublishedParquetTarget {
    corpus: SearchCorpusKind,
    table_name: String,
    parquet_path: PathBuf,
}

async fn resolve_published_parquet_statement_target(
    service: &SearchPlaneService,
    query_text: &str,
) -> Option<ResolvedPublishedParquetTarget> {
    let table_name = extract_single_source_table_name(query_text)?;
    if let Some(target) = resolve_active_local_corpus(service, table_name.as_str()) {
        return Some(target);
    }
    if let Some(target) =
        resolve_local_symbol_source_table_statement_target(service, table_name.as_str())
    {
        return Some(target);
    }
    resolve_repo_source_table_statement_target(service, table_name.as_str()).await
}

fn resolve_active_local_corpus(
    service: &SearchPlaneService,
    table_name: &str,
) -> Option<ResolvedPublishedParquetTarget> {
    let corpus = match table_name {
        "reference_occurrence" => SearchCorpusKind::ReferenceOccurrence,
        "attachment" => SearchCorpusKind::Attachment,
        "knowledge_section" => SearchCorpusKind::KnowledgeSection,
        _ => return None,
    };
    let active_epoch = service.coordinator().status_for(corpus).active_epoch?;
    let parquet_path = service.local_epoch_parquet_path(corpus, active_epoch);
    if !parquet_path.exists() {
        return None;
    }
    Some(ResolvedPublishedParquetTarget {
        corpus,
        table_name: table_name.to_string(),
        parquet_path,
    })
}

fn resolve_local_symbol_source_table_statement_target(
    service: &SearchPlaneService,
    table_name: &str,
) -> Option<ResolvedPublishedParquetTarget> {
    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol)
        .active_epoch?;
    let source_table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    if !source_table_names.iter().any(|source| source == table_name) {
        return None;
    }
    let parquet_path = service.local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name);
    if !parquet_path.exists() {
        return None;
    }
    Some(ResolvedPublishedParquetTarget {
        corpus: SearchCorpusKind::LocalSymbol,
        table_name: table_name.to_string(),
        parquet_path,
    })
}

async fn resolve_repo_source_table_statement_target(
    service: &SearchPlaneService,
    table_name: &str,
) -> Option<ResolvedPublishedParquetTarget> {
    if !table_name.starts_with("repo_entity_repo_")
        && !table_name.starts_with("repo_content_chunk_repo_")
    {
        return None;
    }

    let repo_records = service.repo_corpus_snapshot_for_reads().await;
    for ((corpus, repo_id), record) in repo_records {
        let expected_table_name = match corpus {
            SearchCorpusKind::RepoEntity => {
                SearchPlaneService::repo_entity_table_name(repo_id.as_str())
            }
            SearchCorpusKind::RepoContentChunk => {
                SearchPlaneService::repo_content_chunk_table_name(repo_id.as_str())
            }
            SearchCorpusKind::LocalSymbol
            | SearchCorpusKind::KnowledgeSection
            | SearchCorpusKind::Attachment
            | SearchCorpusKind::ReferenceOccurrence => continue,
        };
        if expected_table_name != table_name {
            continue;
        }

        let publication = record.publication?;
        if !publication.is_parquet_query_readable() {
            return None;
        }
        let parquet_path =
            service.repo_publication_parquet_path(corpus, publication.table_name.as_str());
        if !parquet_path.exists() {
            return None;
        }

        return Some(ResolvedPublishedParquetTarget {
            corpus,
            table_name: table_name.to_string(),
            parquet_path,
        });
    }

    None
}

fn extract_single_source_table_name(query_text: &str) -> Option<String> {
    let trimmed = query_text.trim();
    if trimmed.is_empty() || (trimmed.contains(';') && !trimmed.ends_with(';')) {
        return None;
    }

    let tokens = trimmed
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>();
    let first = tokens.first()?;
    if !first.eq_ignore_ascii_case("select") {
        return None;
    }

    for token in &tokens {
        if token.eq_ignore_ascii_case("join")
            || token.eq_ignore_ascii_case("union")
            || token.eq_ignore_ascii_case("intersect")
            || token.eq_ignore_ascii_case("except")
        {
            return None;
        }
    }

    let from_index = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("from"))?;
    let table_token = tokens.get(from_index + 1)?.trim_end_matches(',');
    if table_token.is_empty()
        || table_token.starts_with('(')
        || table_token.contains(',')
        || table_token.contains('.')
    {
        return None;
    }
    normalize_sql_identifier(table_token)
}

fn normalize_sql_identifier(token: &str) -> Option<String> {
    if token.len() >= 2 && token.starts_with('"') && token.ends_with('"') {
        let inner = token[1..token.len() - 1].replace("\"\"", "\"");
        if inner.is_empty() || inner.contains('.') {
            return None;
        }
        return Some(inner);
    }

    if token
        .chars()
        .all(|char| char == '_' || char.is_ascii_alphanumeric())
    {
        return Some(token.to_ascii_lowercase());
    }

    None
}
