//! `search::repo_search::buffered` owns Wendao search repo search buffered behavior.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinSet;
use tokio::time::{Instant, timeout_at};

use crate::parsers::search::repo_code_query::parse_repo_code_search_query;
use crate::query_core::{
    InMemoryWendaoExplainSink, RepoCodeQueryRequest, RetrievalCorpus, query_repo_code_relation,
};
use crate::search::contracts::SearchHit;
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::content::search_repo_content_hits_for_query;
use super::dispatch::{RepoSearchTarget, repo_search_parallelism};
use super::entity::search_repo_entity_hits_for_query;
use super::entity::{record_query_core_telemetry, relation_to_search_hits};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Per-repository result limits for buffered repository code search.
pub struct RepoSearchResultLimits {
    /// Maximum entity hits to return for each repository.
    pub entity_limit: usize,
    /// Maximum content hits to return for each repository.
    pub content_limit: usize,
}

#[derive(Debug, Default)]
pub(crate) struct BufferedRepoSearchResult {
    pub(crate) hits: Vec<SearchHit>,
    pub(crate) partial_timeout: bool,
}

pub(crate) async fn search_repo_intent_hits_buffered(
    search_plane: SearchPlaneService,
    targets: Vec<RepoSearchTarget>,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }

    let mut queued = VecDeque::from(targets);
    let mut join_set = JoinSet::new();
    let raw_query = raw_query.to_string();
    let parallelism = repo_search_parallelism(&search_plane, queued.len());
    for _ in 0..parallelism {
        if let Some(target) = queued.pop_front() {
            spawn_repo_intent_search_task(
                &mut join_set,
                search_plane.clone(),
                target,
                raw_query.clone(),
                limit,
            );
        }
    }

    let mut hits = Vec::new();
    while let Some(result) = join_set.join_next().await {
        let repository_hits =
            result.map_err(|error| format!("repo intent-search task failed: {error}"))??;
        hits.extend(repository_hits);
        if let Some(target) = queued.pop_front() {
            spawn_repo_intent_search_task(
                &mut join_set,
                search_plane.clone(),
                target,
                raw_query.clone(),
                limit,
            );
        }
    }
    Ok(hits)
}

pub(crate) async fn search_repo_code_hits_buffered(
    search_plane: SearchPlaneService,
    targets: Vec<RepoSearchTarget>,
    raw_query: &str,
    per_repo_limits: RepoSearchResultLimits,
    repo_wide_budget: Option<Duration>,
) -> Result<BufferedRepoSearchResult, String> {
    if targets.is_empty() {
        return Ok(BufferedRepoSearchResult::default());
    }

    let mut queued = VecDeque::from(targets);
    let mut join_set = JoinSet::new();
    let raw_query = raw_query.to_string();
    let parallelism = repo_search_parallelism(&search_plane, queued.len());
    let deadline = repo_wide_budget.map(|budget| Instant::now() + budget);
    for _ in 0..parallelism {
        if let Some(target) = queued.pop_front() {
            spawn_repo_code_search_task(
                &mut join_set,
                search_plane.clone(),
                target,
                raw_query.clone(),
                per_repo_limits,
            );
        }
    }

    let mut hits = Vec::new();
    while !join_set.is_empty() {
        let next_result = if let Some(deadline) = deadline {
            let Ok(result) = timeout_at(deadline, join_set.join_next()).await else {
                join_set.abort_all();
                while join_set.join_next().await.is_some() {}
                return Ok(BufferedRepoSearchResult {
                    hits,
                    partial_timeout: true,
                });
            };
            result
        } else {
            join_set.join_next().await
        };
        let Some(result) = next_result else {
            break;
        };
        let repository_hits =
            result.map_err(|error| format!("repo code-search task failed: {error}"))??;
        hits.extend(repository_hits);
        if let Some(target) = queued.pop_front() {
            spawn_repo_code_search_task(
                &mut join_set,
                search_plane.clone(),
                target,
                raw_query.clone(),
                per_repo_limits,
            );
        }
    }

    Ok(BufferedRepoSearchResult {
        hits,
        partial_timeout: false,
    })
}

fn spawn_repo_intent_search_task(
    join_set: &mut JoinSet<Result<Vec<SearchHit>, String>>,
    search_plane: SearchPlaneService,
    target: RepoSearchTarget,
    raw_query: String,
    limit: usize,
) {
    join_set.spawn(async move {
        let mut hits = Vec::new();
        if target.publication_state.entity_published {
            hits.extend(
                search_repo_entity_hits_for_query(
                    &search_plane,
                    target.repo_id.as_str(),
                    raw_query.as_str(),
                    limit,
                )
                .await?,
            );
        }
        if target.publication_state.content_published {
            hits.extend(
                search_repo_content_hits_for_query(
                    &search_plane,
                    target.repo_id.as_str(),
                    raw_query.as_str(),
                    limit,
                )
                .await?,
            );
        }
        Ok(hits)
    });
}

fn spawn_repo_code_search_task(
    join_set: &mut JoinSet<Result<Vec<SearchHit>, String>>,
    search_plane: SearchPlaneService,
    target: RepoSearchTarget,
    raw_query: String,
    per_repo_limits: RepoSearchResultLimits,
) {
    join_set.spawn(async move {
        search_repo_code_hits(&search_plane, &target, raw_query.as_str(), per_repo_limits).await
    });
}

async fn search_repo_code_hits(
    search_plane: &SearchPlaneService,
    target: &RepoSearchTarget,
    raw_query: &str,
    per_repo_limits: RepoSearchResultLimits,
) -> Result<Vec<SearchHit>, String> {
    let parsed = parse_repo_code_search_query(raw_query);
    let Some(search_term) = parsed.search_term() else {
        return Ok(Vec::new());
    };
    let explain_sink = Arc::new(InMemoryWendaoExplainSink::new());
    let query_limit = if target.publication_state.entity_published {
        per_repo_limits.entity_limit
    } else {
        per_repo_limits.content_limit
    };
    let query = RepoCodeQueryRequest::new(
        target.repo_id.as_str(),
        search_term,
        &parsed.language_filters,
        &parsed.kind_filters,
        target.publication_state.entity_published,
        target.publication_state.content_published,
        query_limit,
    );
    let result = query_repo_code_relation(search_plane, &query, Some(explain_sink.clone()))
        .await
        .map_err(|error| {
            format!(
                "repo code-search query failed for repo `{}`: {error}",
                target.repo_id
            )
        })?;

    let corpus = match result.corpus {
        RetrievalCorpus::RepoEntity => SearchCorpusKind::RepoEntity,
        RetrievalCorpus::RepoContent => SearchCorpusKind::RepoContentChunk,
    };
    let telemetry_limit = match result.corpus {
        RetrievalCorpus::RepoEntity => per_repo_limits.entity_limit,
        RetrievalCorpus::RepoContent => per_repo_limits.content_limit,
    };
    record_query_core_telemetry(
        search_plane,
        corpus,
        target.repo_id.as_str(),
        telemetry_limit,
        explain_sink.events().as_slice(),
    );

    let mut repository_hits = relation_to_search_hits(target.repo_id.as_str(), &result.relation)
        .map_err(|error| {
            format!(
                "repo code-search decode failed for repo `{}`: {error}",
                target.repo_id
            )
        })?;

    if result.corpus == RetrievalCorpus::RepoContent
        && repository_hits.len() > per_repo_limits.content_limit
    {
        repository_hits.truncate(per_repo_limits.content_limit);
    }

    Ok(repository_hits)
}
