use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tokio::select;
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tower::util::ServiceExt;
use xiuxian_wendao::gateway::studio::perf_support::GatewayPerfFixture;
use xiuxian_wendao::repo_index::RepoIndexPhase;

use super::{
    REPO_INDEX_STATUS_URI, RepoIndexLiveAuditConfig, RepoIndexLiveAuditReport,
    audit_real_workspace_repo_index_with_live_note, describe_repo_index_live_audit_report,
    request_repo_index_status,
};

const SEARCH_INDEX_STATUS_URI: &str = "/api/search/index/status";
const DEFAULT_QUERY_WORKERS_MIN: usize = 2;
const DEFAULT_QUERY_WORKERS_MAX: usize = 12;
const DEFAULT_QUERY_PAUSE_MS_MIN: u64 = 5;
const DEFAULT_QUERY_PAUSE_MS_MAX: u64 = 25;
const MIXED_QUERY_WORKERS_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_MIXED_QUERY_WORKERS";
const MIXED_QUERY_PAUSE_MS_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_MIXED_QUERY_PAUSE_MS";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoIndexMixedLoadAuditConfig {
    pub audit: RepoIndexLiveAuditConfig,
    pub query_workers: usize,
    pub query_pause: Duration,
}

impl Default for RepoIndexMixedLoadAuditConfig {
    fn default() -> Self {
        Self::with_lookup(
            &|key| std::env::var(key).ok(),
            RepoIndexLiveAuditConfig::default(),
        )
    }
}

impl RepoIndexMixedLoadAuditConfig {
    pub(crate) fn full_run() -> Self {
        Self::with_lookup(
            &|key| std::env::var(key).ok(),
            RepoIndexLiveAuditConfig::full_run(),
        )
    }

    fn with_lookup(
        lookup: &dyn Fn(&str) -> Option<String>,
        audit: RepoIndexLiveAuditConfig,
    ) -> Self {
        Self {
            audit,
            query_workers: lookup(MIXED_QUERY_WORKERS_ENV)
                .and_then(|raw| raw.trim().parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or_else(default_query_workers),
            query_pause: Duration::from_millis(
                lookup(MIXED_QUERY_PAUSE_MS_ENV)
                    .and_then(|raw| raw.trim().parse::<u64>().ok())
                    .unwrap_or_else(default_query_pause_ms),
            ),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RepoIndexQueryPlan {
    uris: Arc<Vec<String>>,
    skipped_uris: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoIndexMixedLoadReport {
    pub audit: RepoIndexLiveAuditReport,
    pub query_load: RepoIndexQueryLoadSummary,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RepoIndexQueryLoadSummary {
    pub target_repo_id: Option<String>,
    pub worker_count: usize,
    pub query_pause_ms: u64,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub p50_latency_ms: Option<u64>,
    pub p95_latency_ms: Option<u64>,
    pub max_latency_ms: Option<u64>,
    pub last_error: Option<String>,
    pub skipped_uris: Vec<String>,
    pub by_uri: Vec<RepoIndexQueryUriSummary>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RepoIndexQueryUriSummary {
    pub uri: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub p95_latency_ms: Option<u64>,
    pub max_latency_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RepoIndexQueryLoadSnapshot {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    p95_latency_ms: Option<u64>,
    max_latency_ms: Option<u64>,
    busiest_uri: Option<String>,
    last_error: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct RepoIndexQueryLoadTracker {
    inner: Arc<Mutex<RepoIndexQueryLoadState>>,
}

#[derive(Clone, Debug, Default)]
struct RepoIndexQueryLoadState {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    last_error: Option<String>,
    latencies_ms: Vec<u64>,
    by_uri: BTreeMap<String, RepoIndexQueryUriState>,
}

#[derive(Clone, Debug, Default)]
struct RepoIndexQueryUriState {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    latencies_ms: Vec<u64>,
}

pub(crate) async fn audit_real_workspace_repo_index_under_query_load(
    fixture: &GatewayPerfFixture,
    config: &RepoIndexMixedLoadAuditConfig,
) -> Result<RepoIndexMixedLoadReport> {
    let target_repo_id = resolve_query_target_repo_id(fixture).await?;
    let query_plan = resolve_query_plan(fixture, target_repo_id.as_deref()).await?;
    let query_tracker = RepoIndexQueryLoadTracker::default();
    let cancellation = CancellationToken::new();
    let tasks = TaskTracker::new();
    let router = fixture.router();

    for worker_index in 0..config.query_workers {
        tasks.spawn(run_query_worker(
            router.clone(),
            query_plan.uris.clone(),
            worker_index,
            query_tracker.clone(),
            cancellation.child_token(),
            config.query_pause,
        ));
    }

    let live_note = {
        let query_tracker = query_tracker.clone();
        let target_repo_id = target_repo_id.clone();
        let query_workers = config.query_workers;
        let query_pause_ms = duration_millis_u64(config.query_pause);
        let planned_query_count = query_plan.uris.len();
        let skipped_query_count = query_plan.skipped_uris.len();
        move || {
            Some(query_tracker.describe_live_note(
                target_repo_id.as_deref(),
                query_workers,
                query_pause_ms,
                planned_query_count,
                skipped_query_count,
            ))
        }
    };

    let audit =
        audit_real_workspace_repo_index_with_live_note(fixture, &config.audit, &live_note).await;

    cancellation.cancel();
    tasks.close();
    tasks.wait().await;

    let audit = audit?;
    let query_load = query_tracker.summary(
        query_plan.uris.as_ref(),
        target_repo_id,
        config.query_workers,
        duration_millis_u64(config.query_pause),
        query_plan.skipped_uris,
    );

    Ok(RepoIndexMixedLoadReport { audit, query_load })
}

pub(crate) fn describe_repo_index_mixed_load_report(report: &RepoIndexMixedLoadReport) -> String {
    format!(
        "{} queryLoad={}",
        describe_repo_index_live_audit_report(&report.audit),
        describe_query_load_summary(&report.query_load)
    )
}

async fn resolve_query_target_repo_id(fixture: &GatewayPerfFixture) -> Result<Option<String>> {
    let status = request_repo_index_status(fixture).await?;
    Ok(status
        .repos
        .iter()
        .find(|repo| repo.phase == RepoIndexPhase::Ready)
        .map(|repo| repo.repo_id.clone())
        .or(status.current_repo_id))
}

fn build_query_uris(target_repo_id: Option<&str>) -> Vec<String> {
    let mut uris = vec![
        REPO_INDEX_STATUS_URI.to_string(),
        SEARCH_INDEX_STATUS_URI.to_string(),
    ];
    if let Some(repo_id) = target_repo_id {
        uris.extend([
            format!("/api/repo/module-search?repo={repo_id}&query=solve&limit=5"),
            format!("/api/repo/symbol-search?repo={repo_id}&query=solve&limit=5"),
            format!("/api/repo/example-search?repo={repo_id}&query=solve&limit=5"),
            format!(
                "/api/repo/projected-page-search?repo={repo_id}&query=solve&kind=reference&limit=5"
            ),
        ]);
    }
    uris
}

async fn resolve_query_plan(
    fixture: &GatewayPerfFixture,
    target_repo_id: Option<&str>,
) -> Result<RepoIndexQueryPlan> {
    let candidate_uris = build_query_uris(target_repo_id);
    let router = fixture.router();
    let mut accepted = Vec::new();
    let mut skipped = Vec::new();

    for (index, uri) in candidate_uris.into_iter().enumerate() {
        if index < 2 {
            accepted.push(uri);
            continue;
        }
        let status = request_status(router.clone(), &uri).await?;
        if status == StatusCode::OK {
            accepted.push(uri);
        } else {
            skipped.push(format!("{uri}=>status={status}"));
        }
    }

    Ok(RepoIndexQueryPlan {
        uris: Arc::new(accepted),
        skipped_uris: skipped,
    })
}

async fn request_status(router: Router, uri: &str) -> Result<StatusCode> {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let _ = to_bytes(response.into_body(), usize::MAX).await?;
    Ok(status)
}

async fn run_query_worker(
    router: Router,
    uris: Arc<Vec<String>>,
    worker_index: usize,
    tracker: RepoIndexQueryLoadTracker,
    cancellation: CancellationToken,
    query_pause: Duration,
) {
    if uris.is_empty() {
        return;
    }

    let mut uri_index = worker_index % uris.len();
    loop {
        let uri = uris[uri_index].clone();
        let started = Instant::now();
        let request = Request::builder().uri(uri.as_str()).body(Body::empty());
        match request {
            Ok(request) => match router.clone().oneshot(request).await {
                Ok(response) => {
                    let status = response.status();
                    match to_bytes(response.into_body(), usize::MAX).await {
                        Ok(_) if status == StatusCode::OK => {
                            tracker.record_success(&uri, elapsed_ms(started));
                        }
                        Ok(_) => {
                            tracker.record_failure(
                                &uri,
                                elapsed_ms(started),
                                format!("status={status}"),
                            );
                        }
                        Err(error) => {
                            tracker.record_failure(&uri, elapsed_ms(started), error.to_string());
                        }
                    }
                }
                Err(error) => tracker.record_failure(&uri, elapsed_ms(started), error.to_string()),
            },
            Err(error) => tracker.record_failure(&uri, elapsed_ms(started), error.to_string()),
        }

        uri_index = (uri_index + 1) % uris.len();
        select! {
            () = cancellation.cancelled() => break,
            () = sleep(query_pause) => {}
        }
    }
}

impl RepoIndexQueryLoadTracker {
    fn record_success(&self, uri: &str, latency_ms: u64) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.total_requests = guard.total_requests.saturating_add(1);
        guard.successful_requests = guard.successful_requests.saturating_add(1);
        guard.latencies_ms.push(latency_ms);
        let entry = guard.by_uri.entry(uri.to_string()).or_default();
        entry.total_requests = entry.total_requests.saturating_add(1);
        entry.successful_requests = entry.successful_requests.saturating_add(1);
        entry.latencies_ms.push(latency_ms);
    }

    fn record_failure(&self, uri: &str, latency_ms: u64, error: String) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.total_requests = guard.total_requests.saturating_add(1);
        guard.failed_requests = guard.failed_requests.saturating_add(1);
        guard.last_error = Some(error);
        guard.latencies_ms.push(latency_ms);
        let entry = guard.by_uri.entry(uri.to_string()).or_default();
        entry.total_requests = entry.total_requests.saturating_add(1);
        entry.failed_requests = entry.failed_requests.saturating_add(1);
        entry.latencies_ms.push(latency_ms);
    }

    fn snapshot(&self) -> RepoIndexQueryLoadSnapshot {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let busiest_uri = guard
            .by_uri
            .iter()
            .max_by_key(|(_, state)| state.total_requests)
            .map(|(uri, _)| uri.clone());
        RepoIndexQueryLoadSnapshot {
            total_requests: guard.total_requests,
            successful_requests: guard.successful_requests,
            failed_requests: guard.failed_requests,
            p95_latency_ms: percentile_ms(&guard.latencies_ms, 95),
            max_latency_ms: guard.latencies_ms.iter().copied().max(),
            busiest_uri,
            last_error: guard.last_error.clone(),
        }
    }

    fn describe_live_note(
        &self,
        target_repo_id: Option<&str>,
        query_workers: usize,
        query_pause_ms: u64,
        planned_query_count: usize,
        skipped_query_count: usize,
    ) -> String {
        let snapshot = self.snapshot();
        format!(
            "queryWorkers={} queryPauseMs={} queryPlannedUris={} querySkippedUris={} queryTargetRepoId={} queryTotal={} queryOk={} queryFailed={} queryP95Ms={} queryMaxMs={} queryBusiestUri={} queryLastError={}",
            query_workers,
            query_pause_ms,
            planned_query_count,
            skipped_query_count,
            target_repo_id.unwrap_or("none"),
            snapshot.total_requests,
            snapshot.successful_requests,
            snapshot.failed_requests,
            snapshot
                .p95_latency_ms
                .map_or_else(|| "none".to_string(), |value| value.to_string()),
            snapshot
                .max_latency_ms
                .map_or_else(|| "none".to_string(), |value| value.to_string()),
            snapshot.busiest_uri.as_deref().unwrap_or("none"),
            snapshot.last_error.as_deref().unwrap_or("none"),
        )
    }

    fn summary(
        &self,
        query_uris: &[String],
        target_repo_id: Option<String>,
        worker_count: usize,
        query_pause_ms: u64,
        skipped_uris: Vec<String>,
    ) -> RepoIndexQueryLoadSummary {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut by_uri = query_uris
            .iter()
            .map(|uri| {
                let state = guard.by_uri.get(uri).cloned().unwrap_or_default();
                RepoIndexQueryUriSummary {
                    uri: uri.clone(),
                    total_requests: state.total_requests,
                    successful_requests: state.successful_requests,
                    failed_requests: state.failed_requests,
                    p95_latency_ms: percentile_ms(&state.latencies_ms, 95),
                    max_latency_ms: state.latencies_ms.iter().copied().max(),
                }
            })
            .collect::<Vec<_>>();
        by_uri.sort_by_key(|entry| std::cmp::Reverse(entry.total_requests));
        RepoIndexQueryLoadSummary {
            target_repo_id,
            worker_count,
            query_pause_ms,
            total_requests: guard.total_requests,
            successful_requests: guard.successful_requests,
            failed_requests: guard.failed_requests,
            p50_latency_ms: percentile_ms(&guard.latencies_ms, 50),
            p95_latency_ms: percentile_ms(&guard.latencies_ms, 95),
            max_latency_ms: guard.latencies_ms.iter().copied().max(),
            last_error: guard.last_error.clone(),
            skipped_uris,
            by_uri,
        }
    }
}

fn describe_query_load_summary(summary: &RepoIndexQueryLoadSummary) -> String {
    format!(
        "targetRepoId={} workers={} queryPauseMs={} total={} ok={} failed={} p50Ms={} p95Ms={} maxMs={} lastError={} skippedUris={} byUri={}",
        summary.target_repo_id.as_deref().unwrap_or("none"),
        summary.worker_count,
        summary.query_pause_ms,
        summary.total_requests,
        summary.successful_requests,
        summary.failed_requests,
        summary
            .p50_latency_ms
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        summary
            .p95_latency_ms
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        summary
            .max_latency_ms
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        summary.last_error.as_deref().unwrap_or("none"),
        if summary.skipped_uris.is_empty() {
            "none".to_string()
        } else {
            summary.skipped_uris.join("|")
        },
        summary
            .by_uri
            .iter()
            .map(|entry| {
                format!(
                    "{}(total={} ok={} failed={} p95Ms={} maxMs={})",
                    entry.uri,
                    entry.total_requests,
                    entry.successful_requests,
                    entry.failed_requests,
                    entry
                        .p95_latency_ms
                        .map_or_else(|| "none".to_string(), |value| value.to_string()),
                    entry
                        .max_latency_ms
                        .map_or_else(|| "none".to_string(), |value| value.to_string()),
                )
            })
            .collect::<Vec<_>>()
            .join("|"),
    )
}

fn default_query_workers() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .map_or(
            DEFAULT_QUERY_WORKERS_MIN,
            default_query_workers_for_parallelism,
        )
}

fn default_query_workers_for_parallelism(parallelism: usize) -> usize {
    parallelism
        .div_ceil(2)
        .clamp(DEFAULT_QUERY_WORKERS_MIN, DEFAULT_QUERY_WORKERS_MAX)
}

fn default_query_pause_ms() -> u64 {
    std::thread::available_parallelism()
        .map(usize::from)
        .map_or(
            DEFAULT_QUERY_PAUSE_MS_MAX,
            default_query_pause_ms_for_parallelism,
        )
}

fn default_query_pause_ms_for_parallelism(parallelism: usize) -> u64 {
    let raw = 80_u64 / u64::try_from(parallelism.max(1)).unwrap_or(1);
    raw.clamp(DEFAULT_QUERY_PAUSE_MS_MIN, DEFAULT_QUERY_PAUSE_MS_MAX)
}

fn percentile_ms(samples: &[u64], percentile: usize) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let last_index = sorted.len().saturating_sub(1);
    let bounded_percentile = percentile.min(100);
    let rank = if bounded_percentile == 0 {
        0
    } else {
        last_index
            .saturating_mul(bounded_percentile)
            .saturating_add(99)
            / 100
    };
    sorted.get(rank).copied()
}

fn elapsed_ms(start: Instant) -> u64 {
    duration_millis_u64(start.elapsed())
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        RepoIndexMixedLoadAuditConfig, RepoIndexQueryLoadSummary, RepoIndexQueryLoadTracker,
        RepoIndexQueryUriSummary, build_query_uris, default_query_pause_ms_for_parallelism,
        default_query_workers_for_parallelism, describe_query_load_summary, percentile_ms,
    };
    use std::time::Duration;

    #[test]
    fn mixed_load_config_respects_env_overrides() {
        let config = RepoIndexMixedLoadAuditConfig::with_lookup(
            &|key| match key {
                "XIUXIAN_WENDAO_GATEWAY_PERF_MIXED_QUERY_WORKERS" => Some("9".to_string()),
                "XIUXIAN_WENDAO_GATEWAY_PERF_MIXED_QUERY_PAUSE_MS" => Some("11".to_string()),
                _ => None,
            },
            super::RepoIndexLiveAuditConfig::default(),
        );

        assert_eq!(config.query_workers, 9);
        assert_eq!(config.query_pause, Duration::from_millis(11));
    }

    #[test]
    fn machine_aware_query_defaults_scale_with_parallelism() {
        assert_eq!(default_query_workers_for_parallelism(1), 2);
        assert_eq!(default_query_workers_for_parallelism(12), 6);
        assert_eq!(default_query_workers_for_parallelism(32), 12);
        assert_eq!(default_query_pause_ms_for_parallelism(1), 25);
        assert_eq!(default_query_pause_ms_for_parallelism(12), 6);
        assert_eq!(default_query_pause_ms_for_parallelism(32), 5);
    }

    #[test]
    fn build_query_uris_adds_repo_backed_load_when_target_present() {
        let uris = build_query_uris(Some("Sundials.jl"));
        assert_eq!(uris.len(), 6);
        assert!(uris.iter().any(|uri| uri.contains("repo=Sundials.jl")));
    }

    #[test]
    fn percentile_ms_uses_sorted_rank() {
        assert_eq!(percentile_ms(&[9, 3, 7, 1, 5], 50), Some(5));
        assert_eq!(percentile_ms(&[9, 3, 7, 1, 5], 95), Some(9));
        assert_eq!(percentile_ms(&[], 95), None);
    }

    #[test]
    fn query_tracker_summary_retains_all_uris() {
        let tracker = RepoIndexQueryLoadTracker::default();
        tracker.record_success("/api/repo/index/status", 3);
        tracker.record_failure("/api/search/index/status", 9, "status=503".to_string());
        let summary = tracker.summary(
            &[
                "/api/repo/index/status".to_string(),
                "/api/search/index/status".to_string(),
            ],
            Some("Sundials.jl".to_string()),
            6,
            8,
            Vec::new(),
        );

        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.failed_requests, 1);
        assert_eq!(summary.target_repo_id.as_deref(), Some("Sundials.jl"));
        assert_eq!(summary.by_uri.len(), 2);
        assert_eq!(summary.by_uri[0].total_requests, 1);
        assert_eq!(summary.by_uri[1].total_requests, 1);
    }

    #[test]
    fn describe_query_load_summary_includes_latency_and_error_fields() {
        let text = describe_query_load_summary(&RepoIndexQueryLoadSummary {
            target_repo_id: Some("Sundials.jl".to_string()),
            worker_count: 6,
            query_pause_ms: 8,
            total_requests: 12,
            successful_requests: 11,
            failed_requests: 1,
            p50_latency_ms: Some(3),
            p95_latency_ms: Some(7),
            max_latency_ms: Some(11),
            last_error: Some("status=503".to_string()),
            skipped_uris: vec![
                "/api/repo/projected-page-search?repo=Sundials.jl&query=solve&kind=reference&limit=5=>status=409 Conflict".to_string(),
            ],
            by_uri: vec![RepoIndexQueryUriSummary {
                uri: "/api/repo/index/status".to_string(),
                total_requests: 6,
                successful_requests: 5,
                failed_requests: 1,
                p95_latency_ms: Some(7),
                max_latency_ms: Some(11),
            }],
        });

        assert!(text.contains("targetRepoId=Sundials.jl"));
        assert!(text.contains("workers=6"));
        assert!(text.contains("failed=1"));
        assert!(text.contains("lastError=status=503"));
        assert!(text.contains("skippedUris=/api/repo/projected-page-search"));
    }
}
