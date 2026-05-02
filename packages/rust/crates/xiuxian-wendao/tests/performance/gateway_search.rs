use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::support::{PerfRunConfig, run_async_budget};
use anyhow::{Result, anyhow};
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use serial_test::file_serial;
use tower::util::ServiceExt;
use xiuxian_wendao::gateway::studio::perf_support::{
    GatewayPerfFixture, prepare_gateway_perf_fixture, prepare_gateway_real_workspace_perf_fixture,
};

use crate::performance::support::gateway::{
    GatewayPerfCaseDiagnostics, REPO_EXAMPLE_SEARCH_CASE, REPO_MODULE_SEARCH_CASE,
    REPO_PROJECTED_PAGE_SEARCH_CASE, REPO_SYMBOL_SEARCH_CASE, STUDIO_SEARCH_INDEX_STATUS_CASE,
    assert_gateway_perf_budget_with_diagnostics, attach_gateway_perf_diagnostics,
    describe_gateway_perf_case_diagnostics, describe_maintenance_summary,
    describe_query_telemetry_scopes, describe_repo_index_status_payload,
    describe_repo_read_pressure, describe_search_index_status_payload, formal_gateway_perf_budget,
    formal_gateway_perf_config, maintenance_summary, query_telemetry_scope, repo_read_pressure,
};
use crate::performance::support::repo_index_audit::{
    RepoIndexLiveAuditConfig, RepoIndexMixedLoadAuditConfig, audit_real_workspace_repo_index,
    audit_real_workspace_repo_index_under_query_load, describe_repo_index_live_audit_report,
    describe_repo_index_mixed_load_report,
};

const SUITE: &str = "xiuxian-wendao/perf-gateway";
const REAL_WORKSPACE_SUITE: &str = "xiuxian-wendao/perf-gateway-real-workspace";
const REPO_MODULE_SEARCH_URI: &str =
    "/api/repo/module-search?repo=gateway-sync&query=GatewaySyncPkg&limit=5";
const REPO_SYMBOL_SEARCH_URI: &str =
    "/api/repo/symbol-search?repo=gateway-sync&query=solve&limit=5";
const REPO_EXAMPLE_SEARCH_URI: &str =
    "/api/repo/example-search?repo=gateway-sync&query=solve&limit=5";
const REPO_PROJECTED_PAGE_SEARCH_URI: &str =
    "/api/repo/projected-page-search?repo=gateway-sync&query=solve&limit=5";
const STUDIO_SEARCH_INDEX_STATUS_URI: &str = "/api/search/index/status";
const REPO_INDEX_STATUS_URI: &str = "/api/repo/index/status";
const REAL_WORKSPACE_REPO_INDEX_STATUS_CASE: &str = "repo_index_status_real_workspace_sample";
const REAL_WORKSPACE_DEFAULT_MIN_REPOS: u64 = 150;

fn real_workspace_perf_config() -> PerfRunConfig {
    PerfRunConfig {
        warmup_samples: 1,
        samples: 3,
        timeout_ms: 30_000,
        concurrency: 1,
    }
}

fn real_workspace_min_repos() -> u64 {
    std::env::var("XIUXIAN_WENDAO_GATEWAY_PERF_MIN_REPO_COUNT")
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(REAL_WORKSPACE_DEFAULT_MIN_REPOS)
}

async fn request_status(router: axum::Router, uri: &str) -> Result<StatusCode> {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let _ = to_bytes(response.into_body(), usize::MAX).await?;
    Ok(status)
}

async fn request_json(router: axum::Router, uri: &str) -> Result<(StatusCode, Value)> {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload = serde_json::from_slice(&body)?;
    Ok((status, payload))
}

async fn assert_status_perf_case(
    fixture: &GatewayPerfFixture,
    case: &str,
    uri: &'static str,
) -> Result<()> {
    let mut report = run_async_budget(SUITE, case, &formal_gateway_perf_config(), || {
        let router = fixture.router();
        async move {
            let status = request_status(router, uri).await?;
            if status == StatusCode::OK {
                Ok::<_, anyhow::Error>(())
            } else {
                Err(anyhow!("unexpected status {status} for {uri}"))
            }
        }
    })
    .await;
    let diagnostics = collect_gateway_case_diagnostics(fixture, uri).await;
    attach_gateway_perf_diagnostics(&mut report, &diagnostics);
    let diagnostics = describe_gateway_perf_case_diagnostics(&diagnostics);
    assert_gateway_perf_budget_with_diagnostics(
        &report,
        &formal_gateway_perf_budget(case),
        diagnostics.as_str(),
    );
    Ok(())
}

async fn collect_gateway_case_diagnostics(
    fixture: &GatewayPerfFixture,
    uri: &str,
) -> GatewayPerfCaseDiagnostics {
    let search_index = match request_json(fixture.router(), STUDIO_SEARCH_INDEX_STATUS_URI).await {
        Ok((StatusCode::OK, payload)) => describe_search_index_status_payload(&payload),
        Ok((status, _)) => format!("status={status}"),
        Err(error) => format!("error={error}"),
    };
    let repo_index = match request_json(fixture.router(), REPO_INDEX_STATUS_URI).await {
        Ok((StatusCode::OK, payload)) => describe_repo_index_status_payload(&payload),
        Ok((status, _)) => format!("status={status}"),
        Err(error) => format!("error={error}"),
    };
    GatewayPerfCaseDiagnostics {
        uri: uri.to_string(),
        search_index,
        repo_index,
        extra: BTreeMap::new(),
    }
}

async fn collect_gateway_case_diagnostics_with_extra(
    fixture: &GatewayPerfFixture,
    uri: &str,
    extra: BTreeMap<String, String>,
) -> GatewayPerfCaseDiagnostics {
    let mut diagnostics = collect_gateway_case_diagnostics(fixture, uri).await;
    diagnostics.extra.extend(extra);
    diagnostics
}

async fn collect_gateway_case_diagnostics_with_repo_read_pressure(
    fixture: &GatewayPerfFixture,
    uri: &str,
    mut extra: BTreeMap<String, String>,
) -> GatewayPerfCaseDiagnostics {
    let repo_read = match request_json(fixture.router(), STUDIO_SEARCH_INDEX_STATUS_URI).await {
        Ok((StatusCode::OK, payload)) => {
            describe_repo_read_pressure(payload.get("repoReadPressure"))
        }
        Ok((status, _)) => format!("status={status}"),
        Err(error) => format!("error={error}"),
    };
    extra.insert("repoReadPressure".to_string(), repo_read);
    collect_gateway_case_diagnostics_with_extra(fixture, uri, extra).await
}

async fn assert_real_workspace_repo_index_status_sample(
    fixture: &GatewayPerfFixture,
) -> Result<()> {
    let min_repos = real_workspace_min_repos();
    let mut report = run_async_budget(
        REAL_WORKSPACE_SUITE,
        REAL_WORKSPACE_REPO_INDEX_STATUS_CASE,
        &real_workspace_perf_config(),
        || {
            let router = fixture.router();
            async move {
                let (status, payload) = request_json(router, REPO_INDEX_STATUS_URI).await?;
                if status != StatusCode::OK {
                    return Err(anyhow!(
                        "unexpected status {status} for {REPO_INDEX_STATUS_URI}"
                    ));
                }
                let total = payload["total"]
                    .as_u64()
                    .ok_or_else(|| anyhow!("repo index status total should be u64"))?;
                let ready = payload["ready"]
                    .as_u64()
                    .ok_or_else(|| anyhow!("repo index status ready should be u64"))?;
                if total < min_repos {
                    return Err(anyhow!(
                        "repo index status should report at least {min_repos} repositories, got {total}"
                    ));
                }
                if ready == 0 {
                    return Err(anyhow!(
                        "repo index status should report at least one ready repository"
                    ));
                }
                Ok::<_, anyhow::Error>(())
            }
        },
    )
    .await;
    let diagnostics = collect_gateway_case_diagnostics_with_repo_read_pressure(
        fixture,
        REPO_INDEX_STATUS_URI,
        BTreeMap::from([("minRepos".to_string(), min_repos.to_string())]),
    )
    .await;
    attach_gateway_perf_diagnostics(&mut report, &diagnostics);
    if report.summary.error_rate > 0.0 {
        return Err(anyhow!(
            "real-workspace repo-index perf sample recorded error_rate={}; diagnostics={}",
            report.summary.error_rate,
            describe_gateway_perf_case_diagnostics(&diagnostics)
        ));
    }
    Ok(())
}

#[test]
fn real_workspace_min_repos_defaults_to_large_fixture_floor() {
    assert_eq!(real_workspace_min_repos(), REAL_WORKSPACE_DEFAULT_MIN_REPOS);
}

#[tokio::test]
async fn collect_gateway_case_diagnostics_with_extra_merges_runtime_pressure() -> Result<()> {
    let fixture = prepare_gateway_perf_fixture().await?;
    let diagnostics = collect_gateway_case_diagnostics_with_extra(
        &fixture,
        REPO_MODULE_SEARCH_URI,
        BTreeMap::from([("scope".to_string(), "gateway-sync".to_string())]),
    )
    .await;

    assert_eq!(diagnostics.uri, REPO_MODULE_SEARCH_URI);
    assert!(diagnostics.search_index.contains("total="));
    assert!(diagnostics.repo_index.contains("total="));
    assert_eq!(
        diagnostics.extra.get("scope").map(String::as_str),
        Some("gateway-sync")
    );
    Ok(())
}

#[tokio::test]
async fn collect_gateway_case_diagnostics_with_repo_read_pressure_records_gate_state() -> Result<()>
{
    let fixture = prepare_gateway_perf_fixture().await?;
    fixture
        .warm_repo_scope_query("gateway-sync", "solve")
        .await?;
    let diagnostics = collect_gateway_case_diagnostics_with_repo_read_pressure(
        &fixture,
        REPO_SYMBOL_SEARCH_URI,
        BTreeMap::from([("workspaceQuery".to_string(), "solve".to_string())]),
    )
    .await;

    assert_eq!(diagnostics.uri, REPO_SYMBOL_SEARCH_URI);
    assert!(diagnostics.search_index.contains("repoRead="));
    assert!(diagnostics.repo_index.contains("total="));
    assert_eq!(
        diagnostics.extra.get("workspaceQuery").map(String::as_str),
        Some("solve")
    );
    assert!(
        diagnostics
            .extra
            .get("repoReadPressure")
            .is_some_and(|value| value.contains("budget=") && value.contains("fanoutCapped="))
    );
    Ok(())
}

#[tokio::test]
#[file_serial(wendao_perf_gate)]
async fn repo_module_search_perf_gate_reports_warm_cache_latency_formal_gate() -> Result<()> {
    let fixture = prepare_gateway_perf_fixture().await?;
    assert_status_perf_case(&fixture, REPO_MODULE_SEARCH_CASE, REPO_MODULE_SEARCH_URI).await
}

#[tokio::test]
#[file_serial(wendao_perf_gate)]
async fn repo_symbol_search_perf_gate_reports_warm_cache_latency_formal_gate() -> Result<()> {
    let fixture = prepare_gateway_perf_fixture().await?;
    assert_status_perf_case(&fixture, REPO_SYMBOL_SEARCH_CASE, REPO_SYMBOL_SEARCH_URI).await
}

#[tokio::test]
#[file_serial(wendao_perf_gate)]
async fn repo_example_search_perf_gate_reports_warm_cache_latency_formal_gate() -> Result<()> {
    let fixture = prepare_gateway_perf_fixture().await?;
    assert_status_perf_case(&fixture, REPO_EXAMPLE_SEARCH_CASE, REPO_EXAMPLE_SEARCH_URI).await
}

#[tokio::test]
#[file_serial(wendao_perf_gate)]
async fn repo_projected_page_search_perf_gate_reports_warm_cache_latency_formal_gate() -> Result<()>
{
    let fixture = prepare_gateway_perf_fixture().await?;
    assert_status_perf_case(
        &fixture,
        REPO_PROJECTED_PAGE_SEARCH_CASE,
        REPO_PROJECTED_PAGE_SEARCH_URI,
    )
    .await
}

#[tokio::test]
#[file_serial(wendao_perf_gate)]
async fn search_index_status_perf_gate_reports_query_telemetry_summary_formal_gate() -> Result<()> {
    let fixture = prepare_gateway_perf_fixture().await?;
    fixture
        .warm_repo_scope_query("gateway-sync", "solve")
        .await?;
    let diagnostics = Arc::new(Mutex::new(
        "maintenance=none; repoRead=none; scopes=<missing>".to_string(),
    ));

    let mut report = run_async_budget(
        SUITE,
        STUDIO_SEARCH_INDEX_STATUS_CASE,
        &formal_gateway_perf_config(),
        || {
            let router = fixture.router();
            let diagnostics = Arc::clone(&diagnostics);
            async move {
                let (status, payload) =
                    request_json(router, STUDIO_SEARCH_INDEX_STATUS_URI).await?;
                if status != StatusCode::OK {
                    return Err(anyhow!(
                        "unexpected status {status} for {STUDIO_SEARCH_INDEX_STATUS_URI}"
                    ));
                }
                let maintenance_summary_value =
                    payload.get("maintenanceSummary").filter(|value| !value.is_null());
                let maintenance_pressure =
                    describe_maintenance_summary(maintenance_summary_value);
                let repo_read_pressure_value =
                    payload.get("repoReadPressure").filter(|value| !value.is_null());
                let repo_read_pressure_text =
                    describe_repo_read_pressure(repo_read_pressure_value);
                {
                    let mut latest = diagnostics
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    *latest = format!(
                        "maintenance={maintenance_pressure}; repoRead={repo_read_pressure_text}; scopes=<missing>"
                    );
                }
                let maintenance = maintenance_summary_value
                    .map(|summary| {
                        maintenance_summary(summary).ok_or_else(|| {
                            anyhow!(
                                "maintenanceSummary should be fully populated when present; maintenance={maintenance_pressure}"
                            )
                        })
                    })
                    .transpose()?;
                let repo_read = repo_read_pressure_value
                    .map(|summary| {
                        repo_read_pressure(Some(summary)).ok_or_else(|| {
                            anyhow!(
                                "repoReadPressure should be fully populated when present; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}"
                            )
                        })
                    })
                    .transpose()?;
                let summary = payload
                    .get("queryTelemetrySummary")
                    .filter(|value| !value.is_null())
                    .ok_or_else(|| {
                        anyhow!(
                            "missing queryTelemetrySummary; maintenance={maintenance_pressure}"
                        )
                    })?;
                let corpus_count = summary["corpusCount"]
                    .as_u64()
                    .ok_or_else(|| {
                        anyhow!(
                            "queryTelemetrySummary.corpusCount should be u64; maintenance={maintenance_pressure}"
                        )
                    })?;
                let total_rows_scanned = summary["totalRowsScanned"].as_u64().ok_or_else(|| {
                    anyhow!(
                        "queryTelemetrySummary.totalRowsScanned should be u64; maintenance={maintenance_pressure}"
                    )
                })?;
                let scope_pressure = query_telemetry_scope(summary, "gateway-sync")
                    .ok_or_else(|| {
                        anyhow!(
                            "queryTelemetrySummary should retain `gateway-sync` scope after repo-scoped warmup; scopes={}; maintenance={maintenance_pressure}",
                            describe_query_telemetry_scopes(summary),
                        )
                    })?;
                {
                    let mut latest = diagnostics
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    *latest = format!(
                        "maintenance={maintenance_pressure}; repoRead={repo_read_pressure_text}; scopes={}",
                        describe_query_telemetry_scopes(summary)
                    );
                }
                if corpus_count == 0 {
                    return Err(anyhow!(
                        "queryTelemetrySummary should report at least one corpus; maintenance={maintenance_pressure}"
                    ));
                }
                if total_rows_scanned == 0 {
                    return Err(anyhow!(
                        "queryTelemetrySummary should report scanned rows after warmup; maintenance={maintenance_pressure}"
                    ));
                }
                if scope_pressure.corpus_count == 0 || scope_pressure.total_rows_scanned == 0 {
                    return Err(anyhow!(
                        "`gateway-sync` scope bucket should report scanned rows after repo-scoped warmup; scopes={}; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}",
                        describe_query_telemetry_scopes(summary),
                    ));
                }
                if let Some(repo_read) = repo_read {
                    if repo_read.budget == 0 {
                        return Err(anyhow!(
                            "repoReadPressure.budget should stay positive; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}"
                        ));
                    }
                    if repo_read.in_flight > repo_read.budget {
                        return Err(anyhow!(
                            "repoReadPressure.inFlight cannot exceed budget; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}"
                        ));
                    }
                    if let Some(parallelism) = repo_read.parallelism
                        && (parallelism == 0 || parallelism > repo_read.budget)
                    {
                        return Err(anyhow!(
                            "repoReadPressure.parallelism should stay within the shared repo-read budget; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}"
                        ));
                    }
                    if let (Some(searchable_repo_count), Some(parallelism)) =
                        (repo_read.searchable_repo_count, repo_read.parallelism)
                    {
                        if searchable_repo_count > parallelism && !repo_read.fanout_capped {
                            return Err(anyhow!(
                                "repoReadPressure.fanoutCapped should be true when searchable repos exceed parallelism; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}"
                            ));
                        }
                        if searchable_repo_count <= parallelism && repo_read.fanout_capped {
                            return Err(anyhow!(
                                "repoReadPressure.fanoutCapped should stay false when searchable repos fit within parallelism; repoRead={repo_read_pressure_text}; maintenance={maintenance_pressure}"
                            ));
                        }
                    }
                }
                if let Some(maintenance) = maintenance {
                    let signal_count = maintenance.prewarm_running_count
                        + maintenance.prewarm_queued_corpus_count
                        + maintenance.compaction_running_count
                        + maintenance.compaction_queued_corpus_count
                        + maintenance.compaction_pending_count
                        + maintenance.aged_compaction_queue_count;
                    if signal_count == 0 {
                        return Err(anyhow!(
                            "maintenanceSummary should be omitted when no maintenance pressure is present; maintenance={maintenance_pressure}"
                        ));
                    }
                    if maintenance.prewarm_queued_corpus_count > 0
                        && maintenance.max_prewarm_queue_depth == 0
                    {
                        return Err(anyhow!(
                            "maintenanceSummary queued prewarm count requires positive maxPrewarmQueueDepth; maintenance={maintenance_pressure}"
                        ));
                    }
                    if maintenance.prewarm_queued_corpus_count == 0
                        && maintenance.max_prewarm_queue_depth > 0
                    {
                        return Err(anyhow!(
                            "maintenanceSummary maxPrewarmQueueDepth should stay zero without queued prewarm backlog; maintenance={maintenance_pressure}"
                        ));
                    }
                    if maintenance.compaction_queued_corpus_count > 0
                        && maintenance.max_compaction_queue_depth == 0
                    {
                        return Err(anyhow!(
                            "maintenanceSummary queued compaction count requires positive maxCompactionQueueDepth; maintenance={maintenance_pressure}"
                        ));
                    }
                    if maintenance.compaction_queued_corpus_count == 0
                        && maintenance.max_compaction_queue_depth > 0
                    {
                        return Err(anyhow!(
                            "maintenanceSummary maxCompactionQueueDepth should stay zero without queued compaction backlog; maintenance={maintenance_pressure}"
                        ));
                    }
                    if maintenance.aged_compaction_queue_count
                        > maintenance.compaction_queued_corpus_count
                    {
                        return Err(anyhow!(
                            "maintenanceSummary aged queued compaction count cannot exceed queued compaction corpus count; maintenance={maintenance_pressure}"
                        ));
                    }
                }
                Ok::<_, anyhow::Error>(())
            }
        },
    )
    .await;
    let diagnostics = diagnostics
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let mut gateway_diagnostics =
        collect_gateway_case_diagnostics(&fixture, STUDIO_SEARCH_INDEX_STATUS_URI).await;
    gateway_diagnostics
        .extra
        .insert("statusGatePressure".to_string(), diagnostics);
    attach_gateway_perf_diagnostics(&mut report, &gateway_diagnostics);
    let diagnostics = describe_gateway_perf_case_diagnostics(&gateway_diagnostics);
    assert_gateway_perf_budget_with_diagnostics(
        &report,
        &formal_gateway_perf_budget(STUDIO_SEARCH_INDEX_STATUS_CASE),
        diagnostics.as_str(),
    );
    Ok(())
}

#[tokio::test]
#[ignore = "manual large-workspace gateway perf scenario"]
#[file_serial(wendao_perf_gate)]
async fn gateway_perf_reports_real_workspace_scale_real_workspace() -> Result<()> {
    let fixture = prepare_gateway_real_workspace_perf_fixture().await?;
    assert_real_workspace_repo_index_status_sample(&fixture).await
}

#[tokio::test]
#[ignore = "manual large-workspace repo-index live audit"]
#[file_serial(wendao_perf_gate)]
async fn gateway_perf_audits_real_workspace_repo_index_distinctions_live() -> Result<()> {
    let fixture = prepare_gateway_real_workspace_perf_fixture().await?;
    let report =
        audit_real_workspace_repo_index(&fixture, &RepoIndexLiveAuditConfig::default()).await?;
    let diagnostics = describe_repo_index_live_audit_report(&report);
    eprintln!("{diagnostics}");

    if report.expected_repositories == 0 {
        return Err(anyhow!(
            "real-workspace repo-index audit found no configured repositories"
        ));
    }
    if report.final_status.total < report.expected_repositories {
        return Err(anyhow!(
            "repo-index audit should report all configured repositories (expected={}, got={}); diagnostics={diagnostics}",
            report.expected_repositories,
            report.final_status.total
        ));
    }
    if report.final_status.ready == 0 {
        return Err(anyhow!(
            "repo-index audit should retain at least one ready repository; diagnostics={diagnostics}"
        ));
    }
    if let Some(stall_reason) = &report.stall_reason {
        return Err(anyhow!(
            "repo-index audit detected a live stall signature: {stall_reason}; diagnostics={diagnostics}"
        ));
    }
    Ok(())
}

#[tokio::test]
#[ignore = "manual large-workspace repo-index full live audit"]
#[file_serial(wendao_perf_gate)]
async fn gateway_perf_audits_real_workspace_repo_index_full_run_live() -> Result<()> {
    let fixture = prepare_gateway_real_workspace_perf_fixture().await?;
    let report =
        audit_real_workspace_repo_index(&fixture, &RepoIndexLiveAuditConfig::full_run()).await?;
    let diagnostics = describe_repo_index_live_audit_report(&report);
    eprintln!("{diagnostics}");

    if report.expected_repositories == 0 {
        return Err(anyhow!(
            "real-workspace repo-index full audit found no configured repositories"
        ));
    }
    if report.final_status.total < report.expected_repositories {
        return Err(anyhow!(
            "repo-index full audit should report all configured repositories (expected={}, got={}); diagnostics={diagnostics}",
            report.expected_repositories,
            report.final_status.total
        ));
    }
    if let Some(stall_reason) = &report.stall_reason {
        return Err(anyhow!(
            "repo-index full audit detected a live stall signature: {stall_reason}; diagnostics={diagnostics}"
        ));
    }
    if report.timed_out {
        return Err(anyhow!(
            "repo-index full audit hit its timeout before reaching terminal state; diagnostics={diagnostics}"
        ));
    }
    if !report.reached_terminal_state {
        return Err(anyhow!(
            "repo-index full audit should reach a terminal state; diagnostics={diagnostics}"
        ));
    }
    if report.final_status.failed > 0 {
        return Err(anyhow!(
            "repo-index full audit should finish without failed repositories; diagnostics={diagnostics}"
        ));
    }
    Ok(())
}

#[tokio::test]
#[ignore = "manual large-workspace repo-index mixed gateway load audit"]
#[file_serial(wendao_perf_gate)]
async fn gateway_perf_audits_real_workspace_repo_index_mixed_gateway_load_live() -> Result<()> {
    let fixture = prepare_gateway_real_workspace_perf_fixture().await?;
    let report = audit_real_workspace_repo_index_under_query_load(
        &fixture,
        &RepoIndexMixedLoadAuditConfig::full_run(),
    )
    .await?;
    let diagnostics = describe_repo_index_mixed_load_report(&report);
    eprintln!("{diagnostics}");

    if report.audit.expected_repositories == 0 {
        return Err(anyhow!(
            "real-workspace repo-index mixed-load audit found no configured repositories"
        ));
    }
    if report.audit.final_status.total < report.audit.expected_repositories {
        return Err(anyhow!(
            "repo-index mixed-load audit should report all configured repositories (expected={}, got={}); diagnostics={diagnostics}",
            report.audit.expected_repositories,
            report.audit.final_status.total
        ));
    }
    if let Some(stall_reason) = &report.audit.stall_reason {
        return Err(anyhow!(
            "repo-index mixed-load audit detected a live stall signature: {stall_reason}; diagnostics={diagnostics}"
        ));
    }
    if report.audit.timed_out {
        return Err(anyhow!(
            "repo-index mixed-load audit hit its timeout before reaching terminal state; diagnostics={diagnostics}"
        ));
    }
    if !report.audit.reached_terminal_state {
        return Err(anyhow!(
            "repo-index mixed-load audit should reach a terminal state; diagnostics={diagnostics}"
        ));
    }
    if report.audit.final_status.failed > 0 {
        return Err(anyhow!(
            "repo-index mixed-load audit should finish without failed repositories; diagnostics={diagnostics}"
        ));
    }
    if report.query_load.total_requests == 0 {
        return Err(anyhow!(
            "repo-index mixed-load audit should exercise gateway queries during the run; diagnostics={diagnostics}"
        ));
    }
    if report.query_load.failed_requests > 0 {
        return Err(anyhow!(
            "repo-index mixed-load audit should keep gateway queries successful under load; diagnostics={diagnostics}"
        ));
    }
    Ok(())
}
