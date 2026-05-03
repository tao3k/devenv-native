use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use anyhow::{Result, anyhow};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tokio::time::{Instant, sleep};
use tower::util::ServiceExt;
use xiuxian_wendao::analyzers::{RegisteredRepository, load_repo_intelligence_config};
use xiuxian_wendao::repo_index::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexStatusResponse};
use xiuxian_wendao_studio::studio::{
    perf_support::{GatewayPerfFixture, GatewayRepoIndexControllerDebugSnapshot},
    studio_effective_wendao_toml_path,
};

mod mixed_load;

pub(crate) use mixed_load::{
    RepoIndexMixedLoadAuditConfig, audit_real_workspace_repo_index_under_query_load,
    describe_repo_index_mixed_load_report,
};

const REPO_INDEX_STATUS_URI: &str = "/api/repo/index/status";
const DEFAULT_AUDIT_SAMPLE_COUNT: usize = 6;
const DEFAULT_AUDIT_INTERVAL_SECS: u64 = 10;
const DEFAULT_AUDIT_STALL_WINDOW: usize = 3;
const DEFAULT_FULL_AUDIT_TIMEOUT_SECS: u64 = 900;
const DEFAULT_FULL_AUDIT_INTERVAL_SECS: u64 = 15;
const DEFAULT_FULL_AUDIT_STALL_WINDOW: usize = 4;
const AUDIT_SAMPLE_COUNT_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_AUDIT_SAMPLES";
const AUDIT_INTERVAL_SECS_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_AUDIT_INTERVAL_SECS";
const AUDIT_STALL_WINDOW_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_AUDIT_STALL_WINDOW";
const FULL_AUDIT_TIMEOUT_SECS_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_FULL_AUDIT_TIMEOUT_SECS";
const FULL_AUDIT_INTERVAL_SECS_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_FULL_AUDIT_INTERVAL_SECS";
const FULL_AUDIT_STALL_WINDOW_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_FULL_AUDIT_STALL_WINDOW";

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum RepoIndexAuditSourceKind {
    LocalCheckout,
    ManagedRemote,
    #[default]
    Unknown,
}

impl RepoIndexAuditSourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::LocalCheckout => "local_checkout",
            Self::ManagedRemote => "managed_remote",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RepoIndexPhaseCounts {
    pub total: usize,
    pub queued: usize,
    pub checking: usize,
    pub syncing: usize,
    pub indexing: usize,
    pub ready: usize,
    pub unsupported: usize,
    pub failed: usize,
}

impl RepoIndexPhaseCounts {
    fn observe(&mut self, phase: RepoIndexPhase) {
        self.total = self.total.saturating_add(1);
        match phase {
            RepoIndexPhase::Idle => {}
            RepoIndexPhase::Queued => self.queued = self.queued.saturating_add(1),
            RepoIndexPhase::Checking => self.checking = self.checking.saturating_add(1),
            RepoIndexPhase::Syncing => self.syncing = self.syncing.saturating_add(1),
            RepoIndexPhase::Indexing => self.indexing = self.indexing.saturating_add(1),
            RepoIndexPhase::Ready => self.ready = self.ready.saturating_add(1),
            RepoIndexPhase::Unsupported => self.unsupported = self.unsupported.saturating_add(1),
            RepoIndexPhase::Failed => self.failed = self.failed.saturating_add(1),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoIndexLiveAuditConfig {
    pub sample_count: usize,
    pub interval: Duration,
    pub stall_window: usize,
    pub max_duration: Option<Duration>,
    pub emit_sample_logs: bool,
}

impl Default for RepoIndexLiveAuditConfig {
    fn default() -> Self {
        Self {
            sample_count: positive_usize_env(AUDIT_SAMPLE_COUNT_ENV)
                .unwrap_or(DEFAULT_AUDIT_SAMPLE_COUNT),
            interval: Duration::from_secs(
                positive_u64_env(AUDIT_INTERVAL_SECS_ENV).unwrap_or(DEFAULT_AUDIT_INTERVAL_SECS),
            ),
            stall_window: positive_usize_env(AUDIT_STALL_WINDOW_ENV)
                .unwrap_or(DEFAULT_AUDIT_STALL_WINDOW),
            max_duration: None,
            emit_sample_logs: false,
        }
    }
}

impl RepoIndexLiveAuditConfig {
    pub(crate) fn full_run() -> Self {
        Self::full_run_with_lookup(&|key| std::env::var(key).ok())
    }

    fn full_run_with_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Self {
        let timeout = lookup(FULL_AUDIT_TIMEOUT_SECS_ENV)
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map_or(
                Duration::from_secs(DEFAULT_FULL_AUDIT_TIMEOUT_SECS),
                Duration::from_secs,
            );
        let interval = lookup(FULL_AUDIT_INTERVAL_SECS_ENV)
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map_or(
                Duration::from_secs(DEFAULT_FULL_AUDIT_INTERVAL_SECS),
                Duration::from_secs,
            );
        let stall_window = lookup(FULL_AUDIT_STALL_WINDOW_ENV)
            .and_then(|raw| raw.trim().parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_FULL_AUDIT_STALL_WINDOW);
        Self {
            sample_count: sample_budget_for_duration(timeout, interval),
            interval,
            stall_window,
            max_duration: Some(timeout),
            emit_sample_logs: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoIndexLiveAuditSample {
    pub sample_index: usize,
    pub elapsed_secs: u64,
    pub total: usize,
    pub active: usize,
    pub queued: usize,
    pub checking: usize,
    pub syncing: usize,
    pub indexing: usize,
    pub ready: usize,
    pub unsupported: usize,
    pub failed: usize,
    pub target_concurrency: usize,
    pub max_concurrency: usize,
    pub sync_concurrency_limit: usize,
    pub current_repo_id: Option<String>,
    pub controller: GatewayRepoIndexControllerDebugSnapshot,
    pub by_source_kind: BTreeMap<RepoIndexAuditSourceKind, RepoIndexPhaseCounts>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoIndexLiveAuditReport {
    pub expected_repositories: usize,
    pub inventory_by_source_kind: BTreeMap<RepoIndexAuditSourceKind, usize>,
    pub samples: Vec<RepoIndexLiveAuditSample>,
    pub final_status: RepoIndexStatusResponse,
    pub final_by_source_kind: BTreeMap<RepoIndexAuditSourceKind, RepoIndexPhaseCounts>,
    pub top_unsupported_reasons: Vec<(String, usize)>,
    pub top_failed_reasons: Vec<(String, usize)>,
    pub syncing_repo_ids: Vec<String>,
    pub indexing_repo_ids: Vec<String>,
    pub queued_repo_ids: Vec<String>,
    pub stall_reason: Option<String>,
    pub reached_terminal_state: bool,
    pub timed_out: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RepoIndexAuditObservation {
    sample: RepoIndexLiveAuditSample,
    status: RepoIndexStatusResponse,
    completed: usize,
    previous_completed: usize,
    queued_repo_ids: Vec<String>,
    indexing_repo_ids: Vec<String>,
    syncing_repo_ids: Vec<String>,
    unsupported_reasons: Vec<(String, usize)>,
    failed_reasons: Vec<(String, usize)>,
}

pub(crate) async fn audit_real_workspace_repo_index(
    fixture: &GatewayPerfFixture,
    config: &RepoIndexLiveAuditConfig,
) -> Result<RepoIndexLiveAuditReport> {
    audit_real_workspace_repo_index_with_live_note(fixture, config, &|| None).await
}

async fn audit_real_workspace_repo_index_with_live_note<F>(
    fixture: &GatewayPerfFixture,
    config: &RepoIndexLiveAuditConfig,
    live_note: &F,
) -> Result<RepoIndexLiveAuditReport>
where
    F: Fn() -> Option<String>,
{
    let source_kind_by_repo_id = load_real_workspace_repo_source_kinds(fixture)?;
    let inventory_by_source_kind = summarize_inventory(&source_kind_by_repo_id);
    let expected_repositories = source_kind_by_repo_id.len();
    let start = Instant::now();
    let deadline = config.max_duration.map(|duration| start + duration);
    let mut samples: Vec<RepoIndexLiveAuditSample> = Vec::new();
    let mut final_status = None;
    let mut timed_out = false;
    let controller = fixture.repo_index_controller_debug_snapshot();

    log_audit_start(config, expected_repositories, &controller, live_note());

    for sample_index in 0..config.sample_count {
        if wait_for_next_sample(sample_index, deadline, config.interval).await {
            timed_out = true;
            break;
        }

        let status = request_repo_index_status(fixture).await?;
        let controller = fixture.repo_index_controller_debug_snapshot();
        let observation = build_audit_observation(
            sample_index,
            start,
            status,
            controller,
            previous_completed(&samples),
            &source_kind_by_repo_id,
        );
        log_audit_sample(config, &observation, live_note());
        let reached_terminal_state = repo_index_status_is_terminal(&observation.status);
        let RepoIndexAuditObservation { sample, status, .. } = observation;
        samples.push(sample);
        final_status = Some(status);
        if reached_terminal_state {
            log_terminal_state(config);
            break;
        }
    }

    if !timed_out
        && deadline.is_some_and(|deadline| Instant::now() >= deadline)
        && final_status
            .as_ref()
            .is_some_and(|status| !repo_index_status_is_terminal(status))
    {
        timed_out = true;
    }

    let final_status =
        final_status.ok_or_else(|| anyhow!("repo-index audit captured no samples"))?;
    log_audit_completion(
        config,
        timed_out,
        start,
        &final_status,
        &controller,
        live_note(),
    );

    Ok(build_audit_report(
        expected_repositories,
        inventory_by_source_kind,
        &source_kind_by_repo_id,
        samples,
        final_status,
        config.stall_window,
        timed_out,
    ))
}

fn log_audit_start(
    config: &RepoIndexLiveAuditConfig,
    expected_repositories: usize,
    controller: &GatewayRepoIndexControllerDebugSnapshot,
    live_note: Option<String>,
) {
    if config.emit_sample_logs {
        eprintln!(
            "[repo-index-audit] start expectedRepos={} sampleCount={} interval={}s timeout={}s stallWindow={} analysisTimeout={}s syncTimeout={}s syncRetryBudget={}{}",
            expected_repositories,
            config.sample_count,
            config.interval.as_secs(),
            config.max_duration.map_or(0, |duration| duration.as_secs()),
            config.stall_window,
            controller.analysis_timeout_secs,
            controller.sync_timeout_secs,
            controller.sync_retry_budget,
            describe_live_note_suffix(live_note),
        );
    }
}

async fn wait_for_next_sample(
    sample_index: usize,
    deadline: Option<Instant>,
    interval: Duration,
) -> bool {
    if sample_index == 0 {
        return false;
    }

    if let Some(deadline) = deadline {
        let now = Instant::now();
        if now >= deadline {
            return true;
        }
        sleep(interval.min(deadline.duration_since(now))).await;
        return false;
    }

    sleep(interval).await;
    false
}

fn build_audit_observation(
    sample_index: usize,
    start: Instant,
    status: RepoIndexStatusResponse,
    controller: GatewayRepoIndexControllerDebugSnapshot,
    previous_completed: usize,
    source_kind_by_repo_id: &BTreeMap<String, RepoIndexAuditSourceKind>,
) -> RepoIndexAuditObservation {
    let queued_repo_ids = queued_repo_ids(&status.repos);
    let indexing_repo_ids = repo_ids_for_phase(&status.repos, RepoIndexPhase::Indexing);
    let syncing_repo_ids = repo_ids_for_phase(&status.repos, RepoIndexPhase::Syncing);
    let unsupported_reasons = summarize_error_buckets(&status.repos, RepoIndexPhase::Unsupported);
    let failed_reasons = summarize_error_buckets(&status.repos, RepoIndexPhase::Failed);
    let sample = RepoIndexLiveAuditSample {
        sample_index,
        elapsed_secs: start.elapsed().as_secs(),
        total: status.total,
        active: status.active,
        queued: status.queued,
        checking: status.checking,
        syncing: status.syncing,
        indexing: status.indexing,
        ready: status.ready,
        unsupported: status.unsupported,
        failed: status.failed,
        target_concurrency: status.target_concurrency,
        max_concurrency: status.max_concurrency,
        sync_concurrency_limit: status.sync_concurrency_limit,
        current_repo_id: status.current_repo_id.clone(),
        controller,
        by_source_kind: summarize_phase_counts(&status.repos, source_kind_by_repo_id),
    };

    RepoIndexAuditObservation {
        completed: completed_repo_count(&status),
        previous_completed,
        queued_repo_ids,
        indexing_repo_ids,
        syncing_repo_ids,
        unsupported_reasons,
        failed_reasons,
        sample,
        status,
    }
}

fn log_audit_sample(
    config: &RepoIndexLiveAuditConfig,
    observation: &RepoIndexAuditObservation,
    live_note: Option<String>,
) {
    if !config.emit_sample_logs {
        return;
    }

    let sample = &observation.sample;
    eprintln!(
        "[repo-index-audit] sample={} elapsed={}s completed={}/{} deltaCompleted=+{} active={} queued={} checking={} syncing={} indexing={} ready={} unsupported={} failed={} targetConcurrency={} maxConcurrency={} syncLimit={} currentRepoId={} controllerLastAdjustment={} controllerSuccessStreak={} controllerReferenceLimit={} controllerIoPressureStreak={} controllerLastElapsedMs={} controllerEmaMs={} controllerBaselineMs={} controllerEfficiencyRatioPct={} syncingRepos={} indexingRepos={} queuedRepos={} unsupportedReasons={} failedReasons={}{}",
        sample.sample_index,
        sample.elapsed_secs,
        observation.completed,
        sample.total,
        observation
            .completed
            .saturating_sub(observation.previous_completed),
        sample.active,
        sample.queued,
        sample.checking,
        sample.syncing,
        sample.indexing,
        sample.ready,
        sample.unsupported,
        sample.failed,
        sample.target_concurrency,
        sample.max_concurrency,
        sample.sync_concurrency_limit,
        sample.current_repo_id.as_deref().unwrap_or("none"),
        sample.controller.last_adjustment,
        sample.controller.success_streak,
        sample.controller.reference_limit,
        sample.controller.io_pressure_streak,
        sample
            .controller
            .last_elapsed_ms
            .map_or("none".to_string(), |value| value.to_string()),
        sample
            .controller
            .ema_elapsed_ms
            .map_or("none".to_string(), |value| value.to_string()),
        sample
            .controller
            .baseline_elapsed_ms
            .map_or("none".to_string(), |value| value.to_string()),
        sample
            .controller
            .last_efficiency_ratio_pct
            .map_or("none".to_string(), |value| value.to_string()),
        observation.syncing_repo_ids.join(","),
        observation.indexing_repo_ids.join(","),
        observation.queued_repo_ids.join(","),
        describe_reason_counts(&observation.unsupported_reasons),
        describe_reason_counts(&observation.failed_reasons),
        describe_live_note_suffix(live_note),
    );
}

fn log_terminal_state(config: &RepoIndexLiveAuditConfig) {
    if config.emit_sample_logs {
        eprintln!("[repo-index-audit] terminal-state reached");
    }
}

fn log_audit_completion(
    config: &RepoIndexLiveAuditConfig,
    timed_out: bool,
    start: Instant,
    final_status: &RepoIndexStatusResponse,
    controller: &GatewayRepoIndexControllerDebugSnapshot,
    live_note: Option<String>,
) {
    if !config.emit_sample_logs {
        return;
    }

    let phase = if timed_out { "timeout" } else { "complete" };
    eprintln!(
        "[repo-index-audit] {} elapsed={}s analysisTimeout={}s syncTimeout={}s syncRetryBudget={} finalStatus={}{}",
        phase,
        start.elapsed().as_secs(),
        controller.analysis_timeout_secs,
        controller.sync_timeout_secs,
        controller.sync_retry_budget,
        describe_repo_index_status_summary(final_status),
        describe_live_note_suffix(live_note),
    );
}

fn describe_live_note_suffix(live_note: Option<String>) -> String {
    match live_note {
        Some(note) if !note.trim().is_empty() => format!(" {note}"),
        _ => String::new(),
    }
}

fn build_audit_report(
    expected_repositories: usize,
    inventory_by_source_kind: BTreeMap<RepoIndexAuditSourceKind, usize>,
    source_kind_by_repo_id: &BTreeMap<String, RepoIndexAuditSourceKind>,
    samples: Vec<RepoIndexLiveAuditSample>,
    final_status: RepoIndexStatusResponse,
    stall_window: usize,
    timed_out: bool,
) -> RepoIndexLiveAuditReport {
    let final_by_source_kind = summarize_phase_counts(&final_status.repos, source_kind_by_repo_id);
    let top_unsupported_reasons =
        summarize_error_buckets(&final_status.repos, RepoIndexPhase::Unsupported);
    let top_failed_reasons = summarize_error_buckets(&final_status.repos, RepoIndexPhase::Failed);
    let syncing_repo_ids = repo_ids_for_phase(&final_status.repos, RepoIndexPhase::Syncing);
    let indexing_repo_ids = repo_ids_for_phase(&final_status.repos, RepoIndexPhase::Indexing);
    let queued_repo_ids = queued_repo_ids(&final_status.repos);
    let stall_reason = detect_stall_reason(&samples, stall_window);
    let reached_terminal_state = repo_index_status_is_terminal(&final_status);

    RepoIndexLiveAuditReport {
        expected_repositories,
        inventory_by_source_kind,
        samples,
        final_status,
        final_by_source_kind,
        top_unsupported_reasons,
        top_failed_reasons,
        syncing_repo_ids,
        indexing_repo_ids,
        queued_repo_ids,
        stall_reason,
        reached_terminal_state,
        timed_out,
    }
}

fn previous_completed(samples: &[RepoIndexLiveAuditSample]) -> usize {
    samples.last().map_or(0, |sample| {
        sample.ready + sample.unsupported + sample.failed
    })
}

pub(crate) fn describe_repo_index_live_audit_report(report: &RepoIndexLiveAuditReport) -> String {
    let inventory = describe_source_kind_counts(&report.inventory_by_source_kind);
    let final_by_source_kind = describe_phase_counts_by_source_kind(&report.final_by_source_kind);
    let samples = report
        .samples
        .iter()
        .map(|sample| {
            format!(
                "t+{}s(total={} ready={} active={} queued={} checking={} syncing={} indexing={} unsupported={} failed={} targetConcurrency={} maxConcurrency={} syncLimit={} currentRepoId={} controllerLastAdjustment={} controllerReferenceLimit={} controllerIoPressureStreak={} controllerLastElapsedMs={} controllerEfficiencyRatioPct={})",
                sample.elapsed_secs,
                sample.total,
                sample.ready,
                sample.active,
                sample.queued,
                sample.checking,
                sample.syncing,
                sample.indexing,
                sample.unsupported,
                sample.failed,
                sample.target_concurrency,
                sample.max_concurrency,
                sample.sync_concurrency_limit,
                sample.current_repo_id.as_deref().unwrap_or("none"),
                sample.controller.last_adjustment,
                sample.controller.reference_limit,
                sample.controller.io_pressure_streak,
                sample
                    .controller
                    .last_elapsed_ms
                    .map_or("none".to_string(), |value| value.to_string()),
                sample
                    .controller
                    .last_efficiency_ratio_pct
                    .map_or("none".to_string(), |value| value.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(" -> ");

    let mut description = String::new();
    let _ = write!(
        description,
        "expectedRepos={} inventory={} finalStatus={} bySource={} reachedTerminalState={} timedOut={} samples=[{}]",
        report.expected_repositories,
        inventory,
        describe_repo_index_status_summary(&report.final_status),
        final_by_source_kind,
        report.reached_terminal_state,
        report.timed_out,
        samples
    );
    if !report.syncing_repo_ids.is_empty() {
        let _ = write!(
            description,
            " syncingRepos={}",
            report.syncing_repo_ids.join(",")
        );
    }
    if !report.indexing_repo_ids.is_empty() {
        let _ = write!(
            description,
            " indexingRepos={}",
            report.indexing_repo_ids.join(",")
        );
    }
    if !report.queued_repo_ids.is_empty() {
        let _ = write!(
            description,
            " queuedRepos={}",
            report.queued_repo_ids.join(",")
        );
    }
    if !report.top_unsupported_reasons.is_empty() {
        let _ = write!(
            description,
            " unsupportedReasons={}",
            describe_reason_counts(&report.top_unsupported_reasons)
        );
    }
    if !report.top_failed_reasons.is_empty() {
        let _ = write!(
            description,
            " failedReasons={}",
            describe_reason_counts(&report.top_failed_reasons)
        );
    }
    if let Some(stall_reason) = &report.stall_reason {
        let _ = write!(description, " stall={stall_reason}");
    }
    description
}

fn positive_usize_env(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
}

fn positive_u64_env(key: &str) -> Option<u64> {
    std::env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
}

fn sample_budget_for_duration(max_duration: Duration, interval: Duration) -> usize {
    let interval_secs = interval.as_secs().max(1);
    let max_duration_secs = max_duration.as_secs().max(1);
    let budget = max_duration_secs / interval_secs;
    usize::try_from(budget)
        .unwrap_or(usize::MAX)
        .saturating_add(1)
}

fn describe_reason_counts(counts: &[(String, usize)]) -> String {
    counts
        .iter()
        .map(|(reason, count)| format!("{reason}x{count}"))
        .collect::<Vec<_>>()
        .join("|")
}

fn describe_source_kind_counts(counts: &BTreeMap<RepoIndexAuditSourceKind, usize>) -> String {
    counts
        .iter()
        .map(|(kind, count)| format!("{}={count}", kind.as_str()))
        .collect::<Vec<_>>()
        .join(",")
}

fn describe_phase_counts_by_source_kind(
    counts: &BTreeMap<RepoIndexAuditSourceKind, RepoIndexPhaseCounts>,
) -> String {
    counts
        .iter()
        .map(|(kind, count)| {
            format!(
                "{}(total={} ready={} queued={} checking={} syncing={} indexing={} unsupported={} failed={})",
                kind.as_str(),
                count.total,
                count.ready,
                count.queued,
                count.checking,
                count.syncing,
                count.indexing,
                count.unsupported,
                count.failed
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn describe_repo_index_status_summary(status: &RepoIndexStatusResponse) -> String {
    format!(
        "total={} ready={} active={} queued={} checking={} syncing={} indexing={} unsupported={} failed={} targetConcurrency={} maxConcurrency={} syncLimit={} currentRepoId={}",
        status.total,
        status.ready,
        status.active,
        status.queued,
        status.checking,
        status.syncing,
        status.indexing,
        status.unsupported,
        status.failed,
        status.target_concurrency,
        status.max_concurrency,
        status.sync_concurrency_limit,
        status.current_repo_id.as_deref().unwrap_or("none")
    )
}

fn completed_repo_count(status: &RepoIndexStatusResponse) -> usize {
    status
        .ready
        .saturating_add(status.unsupported)
        .saturating_add(status.failed)
}

fn summarize_inventory(
    source_kind_by_repo_id: &BTreeMap<String, RepoIndexAuditSourceKind>,
) -> BTreeMap<RepoIndexAuditSourceKind, usize> {
    let mut counts = BTreeMap::new();
    for source_kind in source_kind_by_repo_id.values().copied() {
        *counts.entry(source_kind).or_insert(0) += 1;
    }
    counts
}

fn summarize_phase_counts(
    repos: &[RepoIndexEntryStatus],
    source_kind_by_repo_id: &BTreeMap<String, RepoIndexAuditSourceKind>,
) -> BTreeMap<RepoIndexAuditSourceKind, RepoIndexPhaseCounts> {
    let mut counts: BTreeMap<RepoIndexAuditSourceKind, RepoIndexPhaseCounts> = BTreeMap::new();
    for repo in repos {
        let source_kind = source_kind_by_repo_id
            .get(&repo.repo_id)
            .copied()
            .unwrap_or_default();
        counts.entry(source_kind).or_default().observe(repo.phase);
    }
    counts
}

fn summarize_error_buckets(
    repos: &[RepoIndexEntryStatus],
    phase: RepoIndexPhase,
) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::new();
    for repo in repos {
        if repo.phase != phase {
            continue;
        }
        let bucket = canonical_error_bucket(repo.last_error.as_deref().unwrap_or("unknown"));
        *counts.entry(bucket).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}

fn repo_ids_for_phase(repos: &[RepoIndexEntryStatus], phase: RepoIndexPhase) -> Vec<String> {
    repos
        .iter()
        .filter(|repo| repo.phase == phase)
        .map(|repo| repo.repo_id.clone())
        .take(5)
        .collect()
}

fn queued_repo_ids(repos: &[RepoIndexEntryStatus]) -> Vec<String> {
    let mut queued = repos
        .iter()
        .filter(|repo| repo.phase == RepoIndexPhase::Queued)
        .collect::<Vec<_>>();
    queued.sort_by_key(|repo| repo.queue_position.unwrap_or(usize::MAX));
    queued
        .into_iter()
        .map(|repo| repo.repo_id.clone())
        .take(5)
        .collect()
}

fn repo_index_status_is_terminal(status: &RepoIndexStatusResponse) -> bool {
    status.active == 0
        && status.queued == 0
        && status.checking == 0
        && status.syncing == 0
        && status.indexing == 0
}

fn detect_stall_reason(
    samples: &[RepoIndexLiveAuditSample],
    stall_window: usize,
) -> Option<String> {
    if stall_window == 0 || samples.len() < stall_window {
        return None;
    }
    let window = &samples[samples.len() - stall_window..];
    let first = window.first()?;
    let signature = (
        first.active,
        first.queued,
        first.checking,
        first.syncing,
        first.indexing,
        first.ready,
        first.unsupported,
        first.failed,
        first.current_repo_id.clone(),
    );
    if signature.0 == 0 {
        return None;
    }
    if window.iter().skip(1).all(|sample| {
        (
            sample.active,
            sample.queued,
            sample.checking,
            sample.syncing,
            sample.indexing,
            sample.ready,
            sample.unsupported,
            sample.failed,
            sample.current_repo_id.clone(),
        ) == signature
    }) {
        return Some(format!(
            "no repo-index progress for {} samples while currentRepoId={} and counts stayed queued={} checking={} syncing={} indexing={} ready={} unsupported={} failed={}",
            stall_window,
            signature.8.as_deref().unwrap_or("none"),
            signature.1,
            signature.2,
            signature.3,
            signature.4,
            signature.5,
            signature.6,
            signature.7
        ));
    }
    None
}

fn canonical_error_bucket(error: &str) -> String {
    let trimmed = error.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }
    for needle in [
        "missing Project.toml",
        "operation timed out",
        "timed out",
        "Too many open files",
        "failed to resolve address",
        "failed to connect to github.com",
        "can't assign requested address",
        "failed to acquire managed checkout lock",
    ] {
        if trimmed.contains(needle) {
            return needle.to_string();
        }
    }
    trimmed.to_string()
}

fn infer_repo_source_kind(repository: &RegisteredRepository) -> RepoIndexAuditSourceKind {
    if repository.url.is_some() {
        return RepoIndexAuditSourceKind::ManagedRemote;
    }
    if repository.path.is_some() {
        return RepoIndexAuditSourceKind::LocalCheckout;
    }
    RepoIndexAuditSourceKind::Unknown
}

fn load_real_workspace_repo_source_kinds(
    fixture: &GatewayPerfFixture,
) -> Result<BTreeMap<String, RepoIndexAuditSourceKind>> {
    let config_root = fixture.root();
    let config_path = studio_effective_wendao_toml_path(config_root);
    let config = load_repo_intelligence_config(Some(config_path.as_path()), config_root)?;
    Ok(config
        .repos
        .into_iter()
        .map(|repository| (repository.id.clone(), infer_repo_source_kind(&repository)))
        .collect())
}

async fn request_repo_index_status(
    fixture: &GatewayPerfFixture,
) -> Result<RepoIndexStatusResponse> {
    let response = fixture
        .router()
        .oneshot(
            Request::builder()
                .uri(REPO_INDEX_STATUS_URI)
                .body(Body::empty())?,
        )
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    if status != StatusCode::OK {
        return Err(anyhow!(
            "unexpected status {status} for {REPO_INDEX_STATUS_URI}"
        ));
    }
    Ok(serde_json::from_slice(&body)?)
}

#[cfg(test)]
mod tests {
    use super::{
        RepoIndexAuditSourceKind, RepoIndexLiveAuditConfig, RepoIndexLiveAuditSample,
        canonical_error_bucket, completed_repo_count, describe_phase_counts_by_source_kind,
        detect_stall_reason, sample_budget_for_duration,
    };
    use std::collections::BTreeMap;
    use std::time::Duration;
    use xiuxian_wendao_studio::studio::perf_support::GatewayRepoIndexControllerDebugSnapshot;

    use super::RepoIndexPhaseCounts;

    #[test]
    fn canonical_error_bucket_collapses_known_transport_failures() {
        assert_eq!(
            canonical_error_bucket(
                "failed to refresh managed mirror `A`: failed to connect to github.com: Can't assign requested address"
            ),
            "failed to connect to github.com"
        );
        assert_eq!(
            canonical_error_bucket(
                "failed to acquire managed checkout lock `/tmp/example.lock`: Too many open files (os error 24)"
            ),
            "Too many open files"
        );
    }

    #[test]
    fn detect_stall_reason_reports_repeated_sync_signature() {
        let sample = RepoIndexLiveAuditSample {
            sample_index: 0,
            elapsed_secs: 0,
            total: 177,
            active: 1,
            queued: 146,
            checking: 0,
            syncing: 1,
            indexing: 0,
            ready: 30,
            unsupported: 0,
            failed: 0,
            target_concurrency: 1,
            max_concurrency: 8,
            sync_concurrency_limit: 1,
            current_repo_id: Some("AutoOffload.jl".to_string()),
            controller: GatewayRepoIndexControllerDebugSnapshot::default(),
            by_source_kind: BTreeMap::new(),
        };
        let samples = vec![sample.clone(), sample.clone(), sample];
        let reason = detect_stall_reason(&samples, 3)
            .unwrap_or_else(|| panic!("stall reason should be detected"));
        assert!(reason.contains("AutoOffload.jl"));
        assert!(reason.contains("queued=146"));
    }

    #[test]
    fn describe_phase_counts_by_source_kind_formats_compact_summary() {
        let mut counts = BTreeMap::new();
        counts.insert(
            RepoIndexAuditSourceKind::ManagedRemote,
            RepoIndexPhaseCounts {
                total: 10,
                ready: 3,
                queued: 6,
                syncing: 1,
                ..RepoIndexPhaseCounts::default()
            },
        );
        assert_eq!(
            describe_phase_counts_by_source_kind(&counts),
            "managed_remote(total=10 ready=3 queued=6 checking=0 syncing=1 indexing=0 unsupported=0 failed=0)"
        );
    }

    #[test]
    fn sample_budget_for_duration_adds_terminal_sample() {
        assert_eq!(
            sample_budget_for_duration(Duration::from_mins(15), Duration::from_secs(15)),
            61
        );
    }

    #[test]
    fn full_run_config_uses_timeout_budget_to_derive_sample_count() {
        let config = RepoIndexLiveAuditConfig::full_run_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_PERF_FULL_AUDIT_TIMEOUT_SECS" => Some("120".to_string()),
            "XIUXIAN_WENDAO_GATEWAY_PERF_FULL_AUDIT_INTERVAL_SECS" => Some("20".to_string()),
            "XIUXIAN_WENDAO_GATEWAY_PERF_FULL_AUDIT_STALL_WINDOW" => Some("5".to_string()),
            _ => None,
        });

        assert_eq!(config.sample_count, 7);
        assert_eq!(config.interval, Duration::from_secs(20));
        assert_eq!(config.stall_window, 5);
        assert_eq!(config.max_duration, Some(Duration::from_mins(2)));
        assert!(config.emit_sample_logs);
    }

    #[test]
    fn completed_repo_count_sums_ready_unsupported_and_failed() {
        let status = xiuxian_wendao::repo_index::RepoIndexStatusResponse {
            total: 10,
            ready: 4,
            unsupported: 2,
            failed: 1,
            ..xiuxian_wendao::repo_index::RepoIndexStatusResponse::default()
        };

        assert_eq!(completed_repo_count(&status), 7);
    }
}
