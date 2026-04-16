use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use serde_json::Value;
use xiuxian_testing::{PerfBudget, PerfReport, PerfRunConfig, assert_perf_budget, run_sync_budget};

pub(crate) const REPO_MODULE_SEARCH_CASE: &str = "repo_module_search_formal";
pub(crate) const REPO_SYMBOL_SEARCH_CASE: &str = "repo_symbol_search_formal";
pub(crate) const REPO_EXAMPLE_SEARCH_CASE: &str = "repo_example_search_formal";
pub(crate) const REPO_PROJECTED_PAGE_SEARCH_CASE: &str = "repo_projected_page_search_formal";
pub(crate) const STUDIO_SEARCH_INDEX_STATUS_CASE: &str = "studio_search_index_status_formal";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GatewayMaintenancePressureSummary {
    pub prewarm_running_count: u64,
    pub prewarm_queued_corpus_count: u64,
    pub max_prewarm_queue_depth: u64,
    pub compaction_running_count: u64,
    pub compaction_queued_corpus_count: u64,
    pub max_compaction_queue_depth: u64,
    pub compaction_pending_count: u64,
    pub aged_compaction_queue_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GatewayRepoReadPressureSummary {
    pub budget: u64,
    pub in_flight: u64,
    pub requested_repo_count: Option<u64>,
    pub searchable_repo_count: Option<u64>,
    pub parallelism: Option<u64>,
    pub fanout_capped: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GatewayPerfCaseDiagnostics {
    pub uri: String,
    pub search_index: String,
    pub repo_index: String,
    pub extra: BTreeMap<String, String>,
}

const GATEWAY_URI_METADATA_KEY: &str = "gateway_uri";
const GATEWAY_SEARCH_INDEX_METADATA_KEY: &str = "gateway_search_index";
const GATEWAY_REPO_INDEX_METADATA_KEY: &str = "gateway_repo_index";
const GATEWAY_METADATA_PREFIX: &str = "gateway_";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GatewayTelemetryScopePressure {
    pub scope: String,
    pub corpus_count: u64,
    pub total_rows_scanned: u64,
    pub total_matched_rows: u64,
    pub total_result_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GatewayPerfBudgetProfile {
    Linux,
    Local,
    Other,
}

pub(crate) fn formal_gateway_perf_config() -> PerfRunConfig {
    PerfRunConfig {
        warmup_samples: 1,
        samples: 6,
        timeout_ms: 2_000,
        concurrency: 1,
    }
}

pub(crate) fn formal_gateway_perf_budget(case: &str) -> PerfBudget {
    formal_gateway_perf_budget_with_lookup(case, gateway_perf_budget_profile(), &|key| {
        std::env::var(key).ok()
    })
}

pub(crate) fn formal_gateway_perf_budget_with_lookup(
    case: &str,
    profile: GatewayPerfBudgetProfile,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> PerfBudget {
    let default_budget = default_perf_budget(case, profile);
    PerfBudget {
        max_p50_latency_ms: budget_override(lookup, case, "P50_MS")
            .or(default_budget.max_p50_latency_ms),
        max_p95_latency_ms: budget_override(lookup, case, "P95_MS")
            .or(default_budget.max_p95_latency_ms),
        max_p99_latency_ms: budget_override(lookup, case, "P99_MS")
            .or(default_budget.max_p99_latency_ms),
        min_throughput_qps: budget_override(lookup, case, "MIN_QPS")
            .or(default_budget.min_throughput_qps),
        max_error_rate: budget_override(lookup, case, "MAX_ERROR_RATE")
            .or(default_budget.max_error_rate),
    }
}

pub(crate) fn gateway_perf_budget_profile_for_runner_os(
    runner_os: Option<&str>,
) -> GatewayPerfBudgetProfile {
    match runner_os {
        Some("Linux") => GatewayPerfBudgetProfile::Linux,
        Some("local") | None => GatewayPerfBudgetProfile::Local,
        _ => GatewayPerfBudgetProfile::Other,
    }
}

fn gateway_perf_budget_profile() -> GatewayPerfBudgetProfile {
    gateway_perf_budget_profile_for_runner_os(std::env::var("RUNNER_OS").ok().as_deref())
}

fn budget_override(
    lookup: &dyn Fn(&str) -> Option<String>,
    case: &str,
    metric: &str,
) -> Option<f64> {
    let case_id = case
        .strip_suffix("_formal")
        .unwrap_or(case)
        .to_ascii_uppercase();
    let key = format!("XIUXIAN_WENDAO_GATEWAY_PERF_{case_id}_{metric}");
    lookup(&key).and_then(|raw| parse_positive_budget_value(&raw))
}

fn parse_positive_budget_value(raw: &str) -> Option<f64> {
    let parsed = raw.trim().parse::<f64>().ok()?;
    (parsed.is_finite() && parsed > 0.0).then_some(parsed)
}

fn default_perf_budget(case: &str, profile: GatewayPerfBudgetProfile) -> PerfBudget {
    match profile {
        GatewayPerfBudgetProfile::Linux => linux_perf_budget(case),
        GatewayPerfBudgetProfile::Local | GatewayPerfBudgetProfile::Other => {
            local_perf_budget(case)
        }
    }
}

fn linux_perf_budget(case: &str) -> PerfBudget {
    match case {
        REPO_MODULE_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(1.45),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(950.0),
            max_error_rate: Some(0.001),
        },
        REPO_SYMBOL_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(1.0),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(1_050.0),
            max_error_rate: Some(0.001),
        },
        REPO_EXAMPLE_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(0.8),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(1_200.0),
            max_error_rate: Some(0.001),
        },
        REPO_PROJECTED_PAGE_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(3.5),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(450.0),
            max_error_rate: Some(0.001),
        },
        STUDIO_SEARCH_INDEX_STATUS_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(0.3),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(1_900.0),
            max_error_rate: Some(0.001),
        },
        other => panic!("missing performance budget for formal gateway case `{other}`"),
    }
}

fn local_perf_budget(case: &str) -> PerfBudget {
    match case {
        REPO_MODULE_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(350.0),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(4.0),
            max_error_rate: Some(0.001),
        },
        REPO_SYMBOL_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(400.0),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(4.0),
            max_error_rate: Some(0.001),
        },
        REPO_EXAMPLE_SEARCH_CASE | STUDIO_SEARCH_INDEX_STATUS_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(350.0),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(5.0),
            max_error_rate: Some(0.001),
        },
        REPO_PROJECTED_PAGE_SEARCH_CASE => PerfBudget {
            max_p50_latency_ms: None,
            max_p95_latency_ms: Some(40.0),
            max_p99_latency_ms: None,
            min_throughput_qps: Some(35.0),
            max_error_rate: Some(0.001),
        },
        other => panic!("missing performance budget for formal gateway case `{other}`"),
    }
}

pub(crate) fn query_telemetry_scopes(summary: &Value) -> Vec<GatewayTelemetryScopePressure> {
    summary["scopes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some(GatewayTelemetryScopePressure {
                scope: entry["scope"].as_str()?.to_string(),
                corpus_count: entry["corpusCount"].as_u64()?,
                total_rows_scanned: entry["totalRowsScanned"].as_u64()?,
                total_matched_rows: entry["totalMatchedRows"].as_u64()?,
                total_result_count: entry["totalResultCount"].as_u64()?,
            })
        })
        .collect()
}

pub(crate) fn query_telemetry_scope(
    summary: &Value,
    scope: &str,
) -> Option<GatewayTelemetryScopePressure> {
    query_telemetry_scopes(summary)
        .into_iter()
        .find(|entry| entry.scope == scope)
}

pub(crate) fn describe_query_telemetry_scopes(summary: &Value) -> String {
    let scopes = query_telemetry_scopes(summary);
    if scopes.is_empty() {
        return "[]".to_string();
    }
    let joined = scopes
        .into_iter()
        .map(|entry| {
            format!(
                "{}(corpora={}, scanned={}, matched={}, results={})",
                entry.scope,
                entry.corpus_count,
                entry.total_rows_scanned,
                entry.total_matched_rows,
                entry.total_result_count
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

pub(crate) fn maintenance_summary(summary: &Value) -> Option<GatewayMaintenancePressureSummary> {
    Some(GatewayMaintenancePressureSummary {
        prewarm_running_count: summary["prewarmRunningCount"].as_u64()?,
        prewarm_queued_corpus_count: summary["prewarmQueuedCorpusCount"].as_u64()?,
        max_prewarm_queue_depth: summary["maxPrewarmQueueDepth"].as_u64()?,
        compaction_running_count: summary["compactionRunningCount"].as_u64()?,
        compaction_queued_corpus_count: summary["compactionQueuedCorpusCount"].as_u64()?,
        max_compaction_queue_depth: summary["maxCompactionQueueDepth"].as_u64()?,
        compaction_pending_count: summary["compactionPendingCount"].as_u64()?,
        aged_compaction_queue_count: summary["agedCompactionQueueCount"].as_u64()?,
    })
}

pub(crate) fn describe_maintenance_summary(summary: Option<&Value>) -> String {
    let Some(summary) = summary else {
        return "none".to_string();
    };
    let Some(summary) = maintenance_summary(summary) else {
        return "invalid".to_string();
    };
    format!(
        "prewarm(running={}, queuedCorpora={}, maxQueueDepth={}), compaction(running={}, queuedCorpora={}, maxQueueDepth={}, pending={}, agedQueued={})",
        summary.prewarm_running_count,
        summary.prewarm_queued_corpus_count,
        summary.max_prewarm_queue_depth,
        summary.compaction_running_count,
        summary.compaction_queued_corpus_count,
        summary.max_compaction_queue_depth,
        summary.compaction_pending_count,
        summary.aged_compaction_queue_count
    )
}

pub(crate) fn describe_gateway_perf_case_diagnostics(
    diagnostics: &GatewayPerfCaseDiagnostics,
) -> String {
    let mut message = format!(
        "uri={}; searchIndex={}; repoIndex={}",
        diagnostics.uri, diagnostics.search_index, diagnostics.repo_index
    );
    for (key, value) in &diagnostics.extra {
        message.push_str("; ");
        message.push_str(key);
        message.push('=');
        message.push_str(value);
    }
    message
}

pub(crate) fn repo_read_pressure(
    summary: Option<&Value>,
) -> Option<GatewayRepoReadPressureSummary> {
    let summary = summary?;
    Some(GatewayRepoReadPressureSummary {
        budget: summary["budget"].as_u64()?,
        in_flight: summary["inFlight"].as_u64()?,
        requested_repo_count: summary.get("requestedRepoCount").and_then(Value::as_u64),
        searchable_repo_count: summary.get("searchableRepoCount").and_then(Value::as_u64),
        parallelism: summary.get("parallelism").and_then(Value::as_u64),
        fanout_capped: summary["fanoutCapped"].as_bool()?,
    })
}

pub(crate) fn describe_repo_read_pressure(summary: Option<&Value>) -> String {
    let Some(summary) = summary else {
        return "none".to_string();
    };
    let Some(summary) = repo_read_pressure(Some(summary)) else {
        return "invalid".to_string();
    };
    format!(
        "budget={}, inFlight={}, requested={}, searchable={}, parallelism={}, fanoutCapped={}",
        summary.budget,
        summary.in_flight,
        summary
            .requested_repo_count
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        summary
            .searchable_repo_count
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        summary
            .parallelism
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        summary.fanout_capped
    )
}

pub(crate) fn describe_search_index_status_payload(payload: &Value) -> String {
    let status_reason = payload["statusReason"]["code"].as_str().unwrap_or("none");
    let maintenance = describe_maintenance_summary(payload.get("maintenanceSummary"));
    let repo_read = describe_repo_read_pressure(payload.get("repoReadPressure"));
    let query_summary = payload
        .get("queryTelemetrySummary")
        .filter(|value| !value.is_null())
        .map_or_else(
            || "none".to_string(),
            |summary| {
                format!(
                    "corpora={}, scanned={}, scopes={}",
                    summary["corpusCount"].as_u64().unwrap_or_default(),
                    summary["totalRowsScanned"].as_u64().unwrap_or_default(),
                    summary["scopes"].as_array().map_or(0, std::vec::Vec::len)
                )
            },
        );
    format!(
        "total={} idle={} indexing={} ready={} degraded={} failed={} compactionPending={} statusReason={} maintenance={} repoRead={} queryTelemetry={}",
        payload["total"].as_u64().unwrap_or_default(),
        payload["idle"].as_u64().unwrap_or_default(),
        payload["indexing"].as_u64().unwrap_or_default(),
        payload["ready"].as_u64().unwrap_or_default(),
        payload["degraded"].as_u64().unwrap_or_default(),
        payload["failed"].as_u64().unwrap_or_default(),
        payload["compactionPending"].as_u64().unwrap_or_default(),
        status_reason,
        maintenance,
        repo_read,
        query_summary
    )
}

pub(crate) fn describe_repo_index_status_payload(payload: &Value) -> String {
    let current_repo_id = payload["currentRepoId"].as_str().unwrap_or("none");
    format!(
        "total={} ready={} active={} queued={} checking={} syncing={} indexing={} unsupported={} failed={} currentRepoId={}",
        payload["total"].as_u64().unwrap_or_default(),
        payload["ready"].as_u64().unwrap_or_default(),
        payload["active"].as_u64().unwrap_or_default(),
        payload["queued"].as_u64().unwrap_or_default(),
        payload["checking"].as_u64().unwrap_or_default(),
        payload["syncing"].as_u64().unwrap_or_default(),
        payload["indexing"].as_u64().unwrap_or_default(),
        payload["unsupported"].as_u64().unwrap_or_default(),
        payload["failed"].as_u64().unwrap_or_default(),
        current_repo_id
    )
}

pub(crate) fn attach_gateway_perf_diagnostics(
    report: &mut PerfReport,
    diagnostics: &GatewayPerfCaseDiagnostics,
) {
    report.add_metadata(GATEWAY_URI_METADATA_KEY, diagnostics.uri.as_str());
    report.add_metadata(
        GATEWAY_SEARCH_INDEX_METADATA_KEY,
        diagnostics.search_index.as_str(),
    );
    report.add_metadata(
        GATEWAY_REPO_INDEX_METADATA_KEY,
        diagnostics.repo_index.as_str(),
    );
    for (key, value) in &diagnostics.extra {
        report.add_metadata(gateway_extra_metadata_key(key), value.as_str());
    }
    if let Err(error) = persist_gateway_perf_report(report) {
        report.add_metadata("gateway_diagnostics_write_error", error.to_string());
    }
}

pub(crate) fn gateway_perf_diagnostics_from_report(
    report: &PerfReport,
) -> Option<GatewayPerfCaseDiagnostics> {
    let uri = report.metadata.get(GATEWAY_URI_METADATA_KEY)?.clone();
    let search_index = report
        .metadata
        .get(GATEWAY_SEARCH_INDEX_METADATA_KEY)?
        .clone();
    let repo_index = report
        .metadata
        .get(GATEWAY_REPO_INDEX_METADATA_KEY)?
        .clone();
    let mut extra = BTreeMap::new();
    for (key, value) in &report.metadata {
        let Some(extra_key) = key.strip_prefix(GATEWAY_METADATA_PREFIX) else {
            continue;
        };
        if matches!(
            key.as_str(),
            GATEWAY_URI_METADATA_KEY
                | GATEWAY_SEARCH_INDEX_METADATA_KEY
                | GATEWAY_REPO_INDEX_METADATA_KEY
        ) {
            continue;
        }
        extra.insert(extra_key.to_string(), value.clone());
    }
    Some(GatewayPerfCaseDiagnostics {
        uri,
        search_index,
        repo_index,
        extra,
    })
}

pub(crate) fn load_gateway_perf_diagnostics_from_report_path(
    path: &Path,
) -> io::Result<Option<GatewayPerfCaseDiagnostics>> {
    let payload = fs::read_to_string(path)?;
    let report: PerfReport = serde_json::from_str(&payload)
        .map_err(|error| io::Error::other(format!("parse perf report: {error}")))?;
    Ok(gateway_perf_diagnostics_from_report(&report))
}

pub(crate) fn describe_gateway_perf_report_from_path(path: &Path) -> io::Result<Option<String>> {
    load_gateway_perf_diagnostics_from_report_path(path)
        .map(|diagnostics| diagnostics.map(|value| describe_gateway_perf_case_diagnostics(&value)))
}

fn persist_gateway_perf_report(report: &PerfReport) -> io::Result<()> {
    let Some(path) = report.report_path.as_deref() else {
        return Ok(());
    };
    let payload = serde_json::to_vec_pretty(report)
        .map_err(|error| io::Error::other(format!("serialize report: {error}")))?;
    fs::write(path, payload)
}

fn gateway_extra_metadata_key(key: &str) -> String {
    format!("{GATEWAY_METADATA_PREFIX}{key}")
}

#[track_caller]
pub(crate) fn assert_gateway_perf_budget_with_diagnostics(
    report: &PerfReport,
    budget: &PerfBudget,
    diagnostics: &str,
) {
    let panic_result = catch_unwind(AssertUnwindSafe(|| assert_perf_budget(report, budget)));
    let Err(payload) = panic_result else {
        return;
    };
    let message = panic_message(payload.as_ref());
    panic!("{message}\ndiagnostics: {diagnostics}");
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(msg) = payload.downcast_ref::<String>() {
        return msg.clone();
    }
    if let Some(msg) = payload.downcast_ref::<&str>() {
        return (*msg).to_string();
    }
    "unknown panic payload".to_string()
}

#[test]
fn gateway_perf_budget_profile_maps_runner_labels() {
    assert_eq!(
        gateway_perf_budget_profile_for_runner_os(Some("Linux")),
        GatewayPerfBudgetProfile::Linux
    );
    assert_eq!(
        gateway_perf_budget_profile_for_runner_os(Some("local")),
        GatewayPerfBudgetProfile::Local
    );
    assert_eq!(
        gateway_perf_budget_profile_for_runner_os(None),
        GatewayPerfBudgetProfile::Local
    );
    assert_eq!(
        gateway_perf_budget_profile_for_runner_os(Some("macOS")),
        GatewayPerfBudgetProfile::Other
    );
}

#[test]
fn gateway_perf_budget_lookup_applies_case_overrides() {
    let budget = formal_gateway_perf_budget_with_lookup(
        REPO_EXAMPLE_SEARCH_CASE,
        GatewayPerfBudgetProfile::Local,
        &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_PERF_REPO_EXAMPLE_SEARCH_P95_MS" => Some("4.25".to_string()),
            "XIUXIAN_WENDAO_GATEWAY_PERF_REPO_EXAMPLE_SEARCH_MIN_QPS" => Some("333.0".to_string()),
            _ => None,
        },
    );
    assert_eq!(budget.max_p95_latency_ms, Some(4.25));
    assert_eq!(budget.min_throughput_qps, Some(333.0));
}

#[test]
fn gateway_perf_budget_lookup_ignores_invalid_values() {
    let budget = formal_gateway_perf_budget_with_lookup(
        REPO_PROJECTED_PAGE_SEARCH_CASE,
        GatewayPerfBudgetProfile::Local,
        &|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_PERF_REPO_PROJECTED_PAGE_SEARCH_P95_MS" => {
                Some("not-a-number".to_string())
            }
            "XIUXIAN_WENDAO_GATEWAY_PERF_REPO_PROJECTED_PAGE_SEARCH_MIN_QPS" => {
                Some("-1".to_string())
            }
            _ => None,
        },
    );
    assert_eq!(budget.max_p95_latency_ms, Some(40.0));
    assert_eq!(budget.min_throughput_qps, Some(35.0));
}

#[test]
fn gateway_perf_budget_local_profile_uses_workstation_safe_defaults() {
    let budget = formal_gateway_perf_budget_with_lookup(
        REPO_MODULE_SEARCH_CASE,
        GatewayPerfBudgetProfile::Local,
        &|_| None,
    );
    assert_eq!(budget.max_p95_latency_ms, Some(350.0));
    assert_eq!(budget.min_throughput_qps, Some(4.0));
}

#[test]
fn gateway_perf_budget_local_profile_uses_workstation_safe_example_defaults() {
    let budget = formal_gateway_perf_budget_with_lookup(
        REPO_EXAMPLE_SEARCH_CASE,
        GatewayPerfBudgetProfile::Local,
        &|_| None,
    );
    assert_eq!(budget.max_p95_latency_ms, Some(350.0));
    assert_eq!(budget.min_throughput_qps, Some(5.0));
}

#[test]
fn gateway_perf_budget_local_profile_uses_workstation_safe_projected_page_defaults() {
    let budget = formal_gateway_perf_budget_with_lookup(
        REPO_PROJECTED_PAGE_SEARCH_CASE,
        GatewayPerfBudgetProfile::Local,
        &|_| None,
    );
    assert_eq!(budget.max_p95_latency_ms, Some(40.0));
    assert_eq!(budget.min_throughput_qps, Some(35.0));
}

#[test]
fn gateway_perf_budget_local_profile_uses_workstation_safe_status_defaults() {
    let budget = formal_gateway_perf_budget_with_lookup(
        STUDIO_SEARCH_INDEX_STATUS_CASE,
        GatewayPerfBudgetProfile::Local,
        &|_| None,
    );
    assert_eq!(budget.max_p95_latency_ms, Some(350.0));
    assert_eq!(budget.min_throughput_qps, Some(5.0));
}

#[test]
fn query_telemetry_scope_reads_specific_scope_bucket() {
    let summary = serde_json::json!({
        "scopes": [
            {
                "scope": "autocomplete",
                "corpusCount": 1,
                "totalRowsScanned": 25,
                "totalMatchedRows": 9,
                "totalResultCount": 5
            },
            {
                "scope": "gateway-sync",
                "corpusCount": 2,
                "totalRowsScanned": 120,
                "totalMatchedRows": 31,
                "totalResultCount": 11
            }
        ]
    });

    let scope = query_telemetry_scope(&summary, "gateway-sync")
        .unwrap_or_else(|| panic!("gateway-sync scope should be present"));
    assert_eq!(scope.corpus_count, 2);
    assert_eq!(scope.total_rows_scanned, 120);
    assert_eq!(scope.total_matched_rows, 31);
    assert_eq!(scope.total_result_count, 11);
}

#[test]
fn describe_query_telemetry_scopes_formats_scope_pressure() {
    let summary = serde_json::json!({
        "scopes": [
            {
                "scope": "search",
                "corpusCount": 2,
                "totalRowsScanned": 100,
                "totalMatchedRows": 27,
                "totalResultCount": 13
            }
        ]
    });

    assert_eq!(
        describe_query_telemetry_scopes(&summary),
        "[search(corpora=2, scanned=100, matched=27, results=13)]"
    );
}

#[test]
fn maintenance_summary_reads_aggregate_counts() {
    let summary = serde_json::json!({
        "prewarmRunningCount": 1,
        "prewarmQueuedCorpusCount": 2,
        "maxPrewarmQueueDepth": 3,
        "compactionRunningCount": 4,
        "compactionQueuedCorpusCount": 5,
        "maxCompactionQueueDepth": 6,
        "compactionPendingCount": 7,
        "agedCompactionQueueCount": 2
    });

    let parsed =
        maintenance_summary(&summary).unwrap_or_else(|| panic!("maintenance summary should parse"));
    assert_eq!(parsed.prewarm_running_count, 1);
    assert_eq!(parsed.prewarm_queued_corpus_count, 2);
    assert_eq!(parsed.max_prewarm_queue_depth, 3);
    assert_eq!(parsed.compaction_running_count, 4);
    assert_eq!(parsed.compaction_queued_corpus_count, 5);
    assert_eq!(parsed.max_compaction_queue_depth, 6);
    assert_eq!(parsed.compaction_pending_count, 7);
    assert_eq!(parsed.aged_compaction_queue_count, 2);
}

#[test]
fn describe_maintenance_summary_formats_aggregate_pressure() {
    let summary = serde_json::json!({
        "prewarmRunningCount": 1,
        "prewarmQueuedCorpusCount": 2,
        "maxPrewarmQueueDepth": 3,
        "compactionRunningCount": 4,
        "compactionQueuedCorpusCount": 5,
        "maxCompactionQueueDepth": 6,
        "compactionPendingCount": 7,
        "agedCompactionQueueCount": 2
    });

    assert_eq!(
        describe_maintenance_summary(Some(&summary)),
        "prewarm(running=1, queuedCorpora=2, maxQueueDepth=3), compaction(running=4, queuedCorpora=5, maxQueueDepth=6, pending=7, agedQueued=2)"
    );
    assert_eq!(describe_maintenance_summary(None), "none");
}

#[test]
fn describe_repo_read_pressure_formats_gate_surface() {
    let summary = serde_json::json!({
        "budget": 2,
        "inFlight": 1,
        "requestedRepoCount": 177,
        "searchableRepoCount": 96,
        "parallelism": 2,
        "fanoutCapped": true
    });

    assert_eq!(
        describe_repo_read_pressure(Some(&summary)),
        "budget=2, inFlight=1, requested=177, searchable=96, parallelism=2, fanoutCapped=true"
    );
    assert_eq!(describe_repo_read_pressure(None), "none");
}

#[test]
fn describe_search_index_status_payload_formats_aggregate_surface() {
    let payload = serde_json::json!({
        "total": 6,
        "idle": 1,
        "indexing": 2,
        "ready": 2,
        "degraded": 1,
        "failed": 0,
        "compactionPending": 1,
        "statusReason": {
            "code": "refreshing"
        },
        "maintenanceSummary": {
            "prewarmRunningCount": 1,
            "prewarmQueuedCorpusCount": 0,
            "maxPrewarmQueueDepth": 0,
            "compactionRunningCount": 0,
            "compactionQueuedCorpusCount": 1,
            "maxCompactionQueueDepth": 2,
            "compactionPendingCount": 1,
            "agedCompactionQueueCount": 0
        },
        "repoReadPressure": {
            "budget": 2,
            "inFlight": 1,
            "requestedRepoCount": 177,
            "searchableRepoCount": 96,
            "parallelism": 2,
            "fanoutCapped": true
        },
        "queryTelemetrySummary": {
            "corpusCount": 2,
            "totalRowsScanned": 120,
            "scopes": [
                {"scope": "gateway-sync"}
            ]
        }
    });

    assert_eq!(
        describe_search_index_status_payload(&payload),
        "total=6 idle=1 indexing=2 ready=2 degraded=1 failed=0 compactionPending=1 statusReason=refreshing maintenance=prewarm(running=1, queuedCorpora=0, maxQueueDepth=0), compaction(running=0, queuedCorpora=1, maxQueueDepth=2, pending=1, agedQueued=0) repoRead=budget=2, inFlight=1, requested=177, searchable=96, parallelism=2, fanoutCapped=true queryTelemetry=corpora=2, scanned=120, scopes=1"
    );
}

#[test]
fn describe_repo_index_status_payload_formats_runtime_counts() {
    let payload = serde_json::json!({
        "total": 4,
        "ready": 2,
        "active": 1,
        "queued": 1,
        "checking": 0,
        "syncing": 0,
        "indexing": 0,
        "unsupported": 1,
        "failed": 0,
        "currentRepoId": "gateway-sync"
    });

    assert_eq!(
        describe_repo_index_status_payload(&payload),
        "total=4 ready=2 active=1 queued=1 checking=0 syncing=0 indexing=0 unsupported=1 failed=0 currentRepoId=gateway-sync"
    );
}

#[test]
fn assert_gateway_perf_budget_with_diagnostics_appends_context() {
    let config = PerfRunConfig {
        warmup_samples: 0,
        samples: 3,
        timeout_ms: 1_000,
        concurrency: 1,
    };
    let report = run_sync_budget(
        "xiuxian-wendao/perf-gateway",
        "budget_context",
        &config,
        || {
            std::thread::sleep(std::time::Duration::from_millis(2));
            Ok::<(), &'static str>(())
        },
    );
    let budget = PerfBudget {
        max_p95_latency_ms: Some(0.1),
        ..PerfBudget::new()
    };

    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        assert_gateway_perf_budget_with_diagnostics(
            &report,
            &budget,
            "maintenance=none; scopes=[gateway-sync(corpora=1, scanned=10, matched=3, results=2)]",
        );
    }));
    let payload = match panic_result {
        Ok(()) => panic!("expected budget assertion to fail"),
        Err(payload) => payload,
    };
    let message = panic_message(payload.as_ref());
    assert!(message.contains("performance budget gate failed"));
    assert!(message.contains("diagnostics: maintenance=none; scopes=[gateway-sync"));
}

#[test]
fn describe_gateway_perf_case_diagnostics_formats_compact_context() {
    let diagnostics = GatewayPerfCaseDiagnostics {
        uri: "/api/repo/module-search?repo=gateway-sync&query=solve".to_string(),
        search_index: "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none".to_string(),
        repo_index: "total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none".to_string(),
        extra: BTreeMap::from([
            (
                "maintenancePressure".to_string(),
                "prewarm(running=0, queuedCorpora=0, maxQueueDepth=0), compaction(running=0, queuedCorpora=0, maxQueueDepth=0, pending=0, agedQueued=0)".to_string(),
            ),
            (
                "queryScopePressure".to_string(),
                "[gateway-sync(corpora=1, scanned=10, matched=3, results=2)]".to_string(),
            ),
        ]),
    };

    assert_eq!(
        describe_gateway_perf_case_diagnostics(&diagnostics),
        "uri=/api/repo/module-search?repo=gateway-sync&query=solve; searchIndex=total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none; repoIndex=total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none; maintenancePressure=prewarm(running=0, queuedCorpora=0, maxQueueDepth=0), compaction(running=0, queuedCorpora=0, maxQueueDepth=0, pending=0, agedQueued=0); queryScopePressure=[gateway-sync(corpora=1, scanned=10, matched=3, results=2)]"
    );
}

#[test]
fn attach_gateway_perf_diagnostics_persists_report_metadata() {
    let config = PerfRunConfig {
        warmup_samples: 0,
        samples: 2,
        timeout_ms: 1_000,
        concurrency: 1,
    };
    let mut report = run_sync_budget(
        "xiuxian-wendao/perf-gateway",
        "metadata_attach",
        &config,
        || Ok::<(), &'static str>(()),
    );
    let diagnostics = GatewayPerfCaseDiagnostics {
        uri: "/api/repo/module-search?repo=gateway-sync&query=solve".to_string(),
        search_index: "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none".to_string(),
        repo_index: "total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none".to_string(),
        extra: BTreeMap::from([(
            "queryScopePressure".to_string(),
            "[gateway-sync(corpora=1, scanned=10, matched=3, results=2)]".to_string(),
        )]),
    };

    attach_gateway_perf_diagnostics(&mut report, &diagnostics);

    let path = report
        .report_path
        .clone()
        .unwrap_or_else(|| panic!("report should be persisted"));
    let payload = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("expected report payload at {path}: {error}"));
    let value: Value = serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("expected valid json at {path}: {error}"));
    assert_eq!(
        value["metadata"]["gateway_uri"].as_str(),
        Some("/api/repo/module-search?repo=gateway-sync&query=solve")
    );
    assert_eq!(
        value["metadata"]["gateway_search_index"].as_str(),
        Some(
            "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none"
        )
    );
    assert_eq!(
        value["metadata"]["gateway_repo_index"].as_str(),
        Some(
            "total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none"
        )
    );
    assert_eq!(
        value["metadata"]["gateway_queryScopePressure"].as_str(),
        Some("[gateway-sync(corpora=1, scanned=10, matched=3, results=2)]")
    );
}

#[test]
fn gateway_perf_diagnostics_from_report_reads_base_fields_and_extras() {
    let config = PerfRunConfig {
        warmup_samples: 0,
        samples: 1,
        timeout_ms: 1_000,
        concurrency: 1,
    };
    let mut report = run_sync_budget(
        "xiuxian-wendao/perf-gateway",
        "metadata_read",
        &config,
        || Ok::<(), &'static str>(()),
    );
    let diagnostics = GatewayPerfCaseDiagnostics {
        uri: "/api/search/index/status".to_string(),
        search_index: "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none".to_string(),
        repo_index: "total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none".to_string(),
        extra: BTreeMap::from([("statusGatePressure".to_string(), "maintenance=none; scopes=[]".to_string())]),
    };
    attach_gateway_perf_diagnostics(&mut report, &diagnostics);

    let restored = gateway_perf_diagnostics_from_report(&report)
        .unwrap_or_else(|| panic!("gateway diagnostics should round-trip from report metadata"));
    assert_eq!(restored, diagnostics);
}

#[test]
fn load_gateway_perf_diagnostics_from_report_path_round_trips_persisted_metadata() {
    let config = PerfRunConfig {
        warmup_samples: 0,
        samples: 1,
        timeout_ms: 1_000,
        concurrency: 1,
    };
    let mut report = run_sync_budget(
        "xiuxian-wendao/perf-gateway",
        "metadata_load_from_path",
        &config,
        || Ok::<(), &'static str>(()),
    );
    let diagnostics = GatewayPerfCaseDiagnostics {
        uri: "/api/repo/index/status".to_string(),
        search_index: "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none".to_string(),
        repo_index: "total=4 ready=2 active=1 queued=1 checking=0 syncing=0 indexing=0 unsupported=1 failed=0 currentRepoId=gateway-sync".to_string(),
        extra: BTreeMap::from([("minRepos".to_string(), "150".to_string())]),
    };
    attach_gateway_perf_diagnostics(&mut report, &diagnostics);

    let path = report
        .report_path
        .clone()
        .unwrap_or_else(|| panic!("report should be persisted"));
    let restored = load_gateway_perf_diagnostics_from_report_path(Path::new(&path))
        .unwrap_or_else(|error| panic!("expected persisted diagnostics at {path}: {error}"))
        .unwrap_or_else(|| panic!("gateway diagnostics should be present at {path}"));
    assert_eq!(restored, diagnostics);
}

#[test]
fn describe_gateway_perf_report_from_path_formats_persisted_metadata() {
    let config = PerfRunConfig {
        warmup_samples: 0,
        samples: 1,
        timeout_ms: 1_000,
        concurrency: 1,
    };
    let mut report = run_sync_budget(
        "xiuxian-wendao/perf-gateway",
        "metadata_describe_from_path",
        &config,
        || Ok::<(), &'static str>(()),
    );
    let diagnostics = GatewayPerfCaseDiagnostics {
        uri: "/api/repo/module-search?repo=gateway-sync&query=solve".to_string(),
        search_index: "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none".to_string(),
        repo_index: "total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none".to_string(),
        extra: BTreeMap::from([("workspaceQuery".to_string(), "solve".to_string())]),
    };
    attach_gateway_perf_diagnostics(&mut report, &diagnostics);

    let path = report
        .report_path
        .clone()
        .unwrap_or_else(|| panic!("report should be persisted"));
    let summary = describe_gateway_perf_report_from_path(Path::new(&path))
        .unwrap_or_else(|error| panic!("expected summary at {path}: {error}"))
        .unwrap_or_else(|| panic!("gateway diagnostics summary should be present at {path}"));
    assert_eq!(
        summary,
        "uri=/api/repo/module-search?repo=gateway-sync&query=solve; searchIndex=total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=none queryTelemetry=none; repoIndex=total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none; workspaceQuery=solve"
    );
}
