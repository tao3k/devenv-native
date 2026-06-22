//! Opt-in audit for the root `wendao.toml` Repo Intelligence bridge.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use chrono::Utc;
use serde::Serialize;
use toml::Value;
use xiuxian_config_core::load_toml_value_with_imports;
use xiuxian_wendao::analyzers::{
    RegisteredRepository, RepoSourceKind, RepoSyncHealthState, RepoSyncMode, RepoSyncQuery,
    RepoSyncState, load_repo_intelligence_config, repo_sync_for_registered_repository,
};

const RUN_ENV: &str = "RUN_WENDAO_REAL_REPO_BRIDGE_AUDIT_TEST";
const MODE_ENV: &str = "WENDAO_REAL_REPO_BRIDGE_AUDIT_MODE";
const LIMIT_ENV: &str = "WENDAO_REAL_REPO_BRIDGE_AUDIT_LIMIT";
const REPOS_ENV: &str = "WENDAO_REAL_REPO_BRIDGE_AUDIT_REPOS";
const WORKERS_ENV: &str = "WENDAO_REAL_REPO_BRIDGE_AUDIT_WORKERS";
const REPORT_ENV: &str = "WENDAO_REAL_REPO_BRIDGE_AUDIT_REPORT";

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn root_wendao_toml_repo_bridge_audit_when_enabled() -> TestResult {
    if env::var_os(RUN_ENV).is_none() {
        eprintln!("skipping root wendao.toml repo bridge audit; set {RUN_ENV}=1 to run it");
        return Ok(());
    }

    let audit_started_at = Instant::now();
    let project_root = project_root();
    let config_path = project_root.join("wendao.toml");
    let surface = inspect_effective_project_surface(config_path.as_path())?;
    let config =
        load_repo_intelligence_config(Some(config_path.as_path()), project_root.as_path())?;
    let mode = parse_mode()?;
    let limit = parse_limit()?;
    let repo_filter = parse_repo_filter();
    let repos = select_repositories(&config.repos, &repo_filter, limit)?;
    let workers = parse_workers()?.min(repos.len().max(1));

    let rows = audit_registered_repositories(&repos, mode, &project_root, workers);

    let mut summary = BridgeSummary {
        effective_project_count: surface.effective_project_count,
        projects_with_url: surface.projects_with_url,
        projects_with_root: surface.projects_with_root,
        projects_with_dirs_only: surface.projects_with_dirs_only,
        registered_repo_resource_count: config.repos.len(),
        attempted_repo_count: rows.len(),
        ..BridgeSummary::default()
    };
    for row in &rows {
        summary.record(row);
    }

    let report = BridgeAuditReport {
        schema: "xiuxian_wendao.repo_bridge_real_scenario_audit.v1",
        generated_at_utc: Utc::now().to_rfc3339(),
        config_path: config_path.display().to_string(),
        project_root: project_root.display().to_string(),
        mode,
        limit,
        repo_filter,
        workers,
        wall_elapsed_ms: audit_started_at.elapsed().as_millis(),
        surface,
        summary,
        rows,
    };

    let report_path = report_path(&project_root);
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    eprintln!("wrote repo bridge audit report: {}", report_path.display());
    eprintln!("{}", serde_json::to_string_pretty(&report.summary)?);

    Ok(())
}

fn audit_registered_repositories(
    repositories: &[RegisteredRepository],
    mode: RepoSyncMode,
    project_root: &Path,
    workers: usize,
) -> Vec<BridgeRepoRow> {
    let next_index = AtomicUsize::new(0);
    let (row_sender, row_receiver) = mpsc::channel();

    thread::scope(|scope| {
        for _worker_index in 0..workers {
            let next_index = &next_index;
            let row_sender = row_sender.clone();
            scope.spawn(move || {
                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    let Some(repository) = repositories.get(index) else {
                        break;
                    };
                    let row = audit_registered_repository(repository, mode, project_root);
                    if row_sender.send(row).is_err() {
                        break;
                    }
                }
            });
        }
    });
    drop(row_sender);

    let mut rows = row_receiver.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.repo_id.cmp(&right.repo_id));
    rows
}

fn audit_registered_repository(
    repository: &RegisteredRepository,
    mode: RepoSyncMode,
    project_root: &Path,
) -> BridgeRepoRow {
    let started_at = Instant::now();
    let query = RepoSyncQuery {
        repo_id: repository.id.clone(),
        mode,
    };

    match repo_sync_for_registered_repository(&query, repository, project_root) {
        Ok(result) => BridgeRepoRow {
            repo_id: repository.id.clone(),
            config_has_url: BridgeFlag::new(repository.url.is_some()),
            config_has_root: BridgeFlag::new(repository.path.is_some()),
            bridge_ok: BridgeFlag::new(true),
            source_kind: Some(result.source_kind),
            health_state: Some(result.health_state),
            mirror_state: Some(result.mirror_state),
            checkout_state: Some(result.checkout_state),
            checkout_path: Some(result.checkout_path),
            mirror_path: result.mirror_path,
            upstream_url: result.upstream_url,
            elapsed_ms: started_at.elapsed().as_millis(),
            benchmark_eligible: BridgeFlag::new(benchmark_eligible_for(
                true,
                Some(result.health_state),
                Some(result.mirror_state),
                Some(result.checkout_state),
            )),
            prewarm_action: prewarm_action_for(
                true,
                Some(result.health_state),
                Some(result.mirror_state),
                Some(result.checkout_state),
                None,
            ),
            error: None,
        },
        Err(error) => {
            let error = error.to_string();
            BridgeRepoRow {
                repo_id: repository.id.clone(),
                config_has_url: BridgeFlag::new(repository.url.is_some()),
                config_has_root: BridgeFlag::new(repository.path.is_some()),
                bridge_ok: BridgeFlag::new(false),
                source_kind: None,
                health_state: None,
                mirror_state: None,
                checkout_state: None,
                checkout_path: None,
                mirror_path: None,
                upstream_url: repository.url.clone(),
                elapsed_ms: started_at.elapsed().as_millis(),
                benchmark_eligible: BridgeFlag::new(benchmark_eligible_for(
                    false, None, None, None,
                )),
                prewarm_action: prewarm_action_for(false, None, None, None, Some(error.as_str())),
                error: Some(error),
            }
        }
    }
}

fn inspect_effective_project_surface(config_path: &Path) -> TestResult<EffectiveProjectSurface> {
    let value = load_toml_value_with_imports(config_path)?;
    let projects = value
        .get("link_graph")
        .and_then(|value| value.get("projects"))
        .and_then(Value::as_table)
        .ok_or_else(|| {
            format!(
                "`{}` does not contain a [sources.projects] table",
                config_path.display()
            )
        })?;

    let mut surface = EffectiveProjectSurface {
        effective_project_count: projects.len(),
        ..EffectiveProjectSurface::default()
    };

    for (id, project) in projects {
        let Some(project) = project.as_table() else {
            continue;
        };
        let has_url = non_empty_string(project.get("url"));
        let has_root = non_empty_string(project.get("root"));
        let has_dirs = project
            .get("dirs")
            .and_then(Value::as_array)
            .is_some_and(|dirs| !dirs.is_empty());

        if has_url {
            surface.projects_with_url += 1;
        }
        if has_root {
            surface.projects_with_root += 1;
        }
        if has_dirs && !has_url && !has_root {
            surface.projects_with_dirs_only += 1;
            surface.dirs_only_project_ids.push(id.clone());
        }
        if !has_url && !has_root {
            surface.non_repo_resource_project_ids.push(id.clone());
        }
    }

    Ok(surface)
}

fn non_empty_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn parse_mode() -> TestResult<RepoSyncMode> {
    match env::var(MODE_ENV)
        .unwrap_or_else(|_| "status".to_string())
        .as_str()
    {
        "status" => Ok(RepoSyncMode::Status),
        "ensure" => Ok(RepoSyncMode::Ensure),
        "refresh" => Ok(RepoSyncMode::Refresh),
        value => Err(format!(
            "unsupported {MODE_ENV} value `{value}`; expected status, ensure, or refresh"
        )
        .into()),
    }
}

fn parse_limit() -> TestResult<Option<usize>> {
    env::var(LIMIT_ENV)
        .ok()
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("failed to parse {LIMIT_ENV} value `{value}`: {error}"))
        })
        .transpose()
        .map_err(Into::into)
}

fn parse_repo_filter() -> Vec<String> {
    env::var(REPOS_ENV)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn select_repositories(
    repositories: &[RegisteredRepository],
    repo_filter: &[String],
    limit: Option<usize>,
) -> TestResult<Vec<RegisteredRepository>> {
    let filtered = if repo_filter.is_empty() {
        repositories.to_vec()
    } else {
        let requested = repo_filter.iter().cloned().collect::<BTreeSet<_>>();
        let selected = repositories
            .iter()
            .filter(|repository| requested.contains(&repository.id))
            .cloned()
            .collect::<Vec<_>>();
        let selected_ids = selected
            .iter()
            .map(|repository| repository.id.as_str())
            .collect::<BTreeSet<_>>();
        let missing = requested
            .iter()
            .filter(|repo_id| !selected_ids.contains(repo_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(format!(
                "{REPOS_ENV} contains repository ids that are not registered in wendao.toml: {}",
                missing.join(", ")
            )
            .into());
        }
        selected
    };

    Ok(filtered
        .into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .collect())
}

fn parse_workers() -> TestResult<usize> {
    let default_workers = thread::available_parallelism()
        .map_or(4, usize::from)
        .clamp(1, 6);
    env::var(WORKERS_ENV)
        .ok()
        .map_or(Ok(default_workers), |value| {
            let workers = value.parse::<usize>().map_err(|error| {
                format!("failed to parse {WORKERS_ENV} value `{value}`: {error}")
            })?;
            if workers == 0 {
                return Err(format!("{WORKERS_ENV} must be greater than zero"));
            }
            Ok(workers)
        })
        .map_err(Into::into)
}

fn report_path(project_root: &Path) -> PathBuf {
    env::var_os(REPORT_ENV).map_or_else(
        || {
            project_root
                .join(".cache")
                .join("agent")
                .join("reports")
                .join("2026-05-10-wendaograph-real-scenario-rust-bridge-resource-audit.json")
        },
        |value| {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                project_root.join(path)
            }
        },
    )
}

fn project_root() -> PathBuf {
    env::var_os("PRJ_ROOT").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../../")
                .clean()
        },
        PathBuf::from,
    )
}

trait CleanPath {
    fn clean(self) -> PathBuf;
}

impl CleanPath for PathBuf {
    fn clean(self) -> PathBuf {
        self.canonicalize().unwrap_or(self)
    }
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct EffectiveProjectSurface {
    effective_project_count: usize,
    projects_with_url: usize,
    projects_with_root: usize,
    projects_with_dirs_only: usize,
    dirs_only_project_ids: Vec<String>,
    non_repo_resource_project_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeAuditReport {
    schema: &'static str,
    generated_at_utc: String,
    config_path: String,
    project_root: String,
    mode: RepoSyncMode,
    limit: Option<usize>,
    repo_filter: Vec<String>,
    workers: usize,
    wall_elapsed_ms: u128,
    surface: EffectiveProjectSurface,
    summary: BridgeSummary,
    rows: Vec<BridgeRepoRow>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct BridgeSummary {
    effective_project_count: usize,
    projects_with_url: usize,
    projects_with_root: usize,
    projects_with_dirs_only: usize,
    registered_repo_resource_count: usize,
    attempted_repo_count: usize,
    bridge_ok_count: usize,
    bridge_error_count: usize,
    benchmark_eligible_count: usize,
    prewarm_backlog_count: usize,
    retry_backlog_count: usize,
    source_kind_counts: BTreeMap<String, usize>,
    health_state_counts: BTreeMap<String, usize>,
    mirror_state_counts: BTreeMap<String, usize>,
    checkout_state_counts: BTreeMap<String, usize>,
    prewarm_action_counts: BTreeMap<String, usize>,
}

impl BridgeSummary {
    fn record(&mut self, row: &BridgeRepoRow) {
        if row.bridge_ok.get() {
            self.bridge_ok_count += 1;
        } else {
            self.bridge_error_count += 1;
        }
        if row.benchmark_eligible.get() {
            self.benchmark_eligible_count += 1;
        }
        if matches!(row.prewarm_action, PrewarmAction::PrewarmRequired) {
            self.prewarm_backlog_count += 1;
        }
        if matches!(row.prewarm_action, PrewarmAction::RetryRequired) {
            self.retry_backlog_count += 1;
        }
        bump(
            &mut self.prewarm_action_counts,
            prewarm_action_label(row.prewarm_action),
        );
        if let Some(source_kind) = row.source_kind {
            bump(&mut self.source_kind_counts, source_kind_label(source_kind));
        }
        if let Some(health_state) = row.health_state {
            bump(
                &mut self.health_state_counts,
                health_state_label(health_state),
            );
        }
        if let Some(mirror_state) = row.mirror_state {
            bump(
                &mut self.mirror_state_counts,
                sync_state_label(mirror_state),
            );
        }
        if let Some(checkout_state) = row.checkout_state {
            bump(
                &mut self.checkout_state_counts,
                sync_state_label(checkout_state),
            );
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeRepoRow {
    repo_id: String,
    config_has_url: BridgeFlag,
    config_has_root: BridgeFlag,
    bridge_ok: BridgeFlag,
    source_kind: Option<RepoSourceKind>,
    health_state: Option<RepoSyncHealthState>,
    mirror_state: Option<RepoSyncState>,
    checkout_state: Option<RepoSyncState>,
    checkout_path: Option<String>,
    mirror_path: Option<String>,
    upstream_url: Option<String>,
    elapsed_ms: u128,
    benchmark_eligible: BridgeFlag,
    prewarm_action: PrewarmAction,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
struct BridgeFlag(bool);

impl BridgeFlag {
    const fn new(value: bool) -> Self {
        Self(value)
    }

    const fn get(self) -> bool {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PrewarmAction {
    BenchmarkReady,
    PrewarmRequired,
    RefreshRecommended,
    RetryRequired,
    InvestigateError,
}

fn bump(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}

fn source_kind_label(kind: RepoSourceKind) -> &'static str {
    match kind {
        RepoSourceKind::LocalCheckout => "local_checkout",
        RepoSourceKind::ManagedRemote => "managed_remote",
    }
}

fn health_state_label(state: RepoSyncHealthState) -> &'static str {
    match state {
        RepoSyncHealthState::Healthy => "healthy",
        RepoSyncHealthState::MissingAssets => "missing_assets",
        RepoSyncHealthState::NeedsRefresh => "needs_refresh",
        RepoSyncHealthState::HasLocalCommits => "has_local_commits",
        RepoSyncHealthState::Diverged => "diverged",
        RepoSyncHealthState::Unknown => "unknown",
    }
}

fn benchmark_eligible_for(
    bridge_ok: bool,
    health_state: Option<RepoSyncHealthState>,
    _mirror_state: Option<RepoSyncState>,
    checkout_state: Option<RepoSyncState>,
) -> bool {
    if !bridge_ok {
        return false;
    }
    matches!(health_state, Some(RepoSyncHealthState::Healthy))
        && !matches!(checkout_state, Some(RepoSyncState::Missing) | None)
}

fn prewarm_action_for(
    bridge_ok: bool,
    health_state: Option<RepoSyncHealthState>,
    mirror_state: Option<RepoSyncState>,
    checkout_state: Option<RepoSyncState>,
    error: Option<&str>,
) -> PrewarmAction {
    if !bridge_ok {
        return if error.is_some_and(timeout_like_error) {
            PrewarmAction::RetryRequired
        } else {
            PrewarmAction::InvestigateError
        };
    }
    if matches!(
        health_state,
        Some(RepoSyncHealthState::MissingAssets) | None
    ) || matches!(mirror_state, Some(RepoSyncState::Missing))
        || matches!(checkout_state, Some(RepoSyncState::Missing) | None)
    {
        return PrewarmAction::PrewarmRequired;
    }
    if matches!(health_state, Some(RepoSyncHealthState::NeedsRefresh)) {
        return PrewarmAction::RefreshRecommended;
    }
    if benchmark_eligible_for(bridge_ok, health_state, mirror_state, checkout_state) {
        PrewarmAction::BenchmarkReady
    } else {
        PrewarmAction::InvestigateError
    }
}

fn timeout_like_error(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("timed out") || error.contains("timeout")
}

fn prewarm_action_label(action: PrewarmAction) -> &'static str {
    match action {
        PrewarmAction::BenchmarkReady => "benchmark_ready",
        PrewarmAction::PrewarmRequired => "prewarm_required",
        PrewarmAction::RefreshRecommended => "refresh_recommended",
        PrewarmAction::RetryRequired => "retry_required",
        PrewarmAction::InvestigateError => "investigate_error",
    }
}

fn sync_state_label(state: RepoSyncState) -> &'static str {
    match state {
        RepoSyncState::NotApplicable => "not_applicable",
        RepoSyncState::Missing => "missing",
        RepoSyncState::Validated => "validated",
        RepoSyncState::Observed => "observed",
        RepoSyncState::Created => "created",
        RepoSyncState::Reused => "reused",
        RepoSyncState::Refreshed => "refreshed",
    }
}

#[test]
fn repo_bridge_prewarm_action_classifies_missing_assets_as_backlog() {
    assert_eq!(
        prewarm_action_for(
            true,
            Some(RepoSyncHealthState::MissingAssets),
            Some(RepoSyncState::Missing),
            Some(RepoSyncState::Missing),
            None,
        ),
        PrewarmAction::PrewarmRequired
    );
    assert!(!benchmark_eligible_for(
        true,
        Some(RepoSyncHealthState::MissingAssets),
        Some(RepoSyncState::Missing),
        Some(RepoSyncState::Missing),
    ));
}

#[test]
fn repo_bridge_prewarm_action_classifies_timeout_errors_as_retry_required() {
    assert_eq!(
        prewarm_action_for(
            false,
            None,
            None,
            None,
            Some("remote operation `clone bare mirror` timed out after 66s"),
        ),
        PrewarmAction::RetryRequired
    );
    assert!(!benchmark_eligible_for(false, None, None, None));
}

#[test]
fn repo_bridge_prewarm_action_classifies_healthy_checkout_as_benchmark_ready() {
    assert_eq!(
        prewarm_action_for(
            true,
            Some(RepoSyncHealthState::Healthy),
            Some(RepoSyncState::Observed),
            Some(RepoSyncState::Observed),
            None,
        ),
        PrewarmAction::BenchmarkReady
    );
    assert!(benchmark_eligible_for(
        true,
        Some(RepoSyncHealthState::Healthy),
        Some(RepoSyncState::Observed),
        Some(RepoSyncState::Observed),
    ));
}
