use std::collections::BTreeMap;
use std::fmt::{Display, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(feature = "performance-stress")]
use std::future::Future;

#[cfg(feature = "performance-stress")]
use futures::future::join_all;
use serde::{Deserialize, Serialize};

const PERF_REPORT_SCHEMA_VERSION: &str = "wendao.perf-report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PerfBudget {
    pub(crate) max_p50_latency_ms: Option<f64>,
    pub(crate) max_p95_latency_ms: Option<f64>,
    pub(crate) max_p99_latency_ms: Option<f64>,
    pub(crate) min_throughput_qps: Option<f64>,
    pub(crate) max_error_rate: Option<f64>,
}

impl PerfBudget {
    pub(crate) const fn new() -> Self {
        Self {
            max_p50_latency_ms: None,
            max_p95_latency_ms: None,
            max_p99_latency_ms: None,
            min_throughput_qps: None,
            max_error_rate: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PerfRunConfig {
    pub(crate) warmup_samples: usize,
    pub(crate) samples: usize,
    pub(crate) timeout_ms: u64,
    pub(crate) concurrency: usize,
}

impl PerfRunConfig {
    pub(crate) fn normalized(&self) -> Self {
        Self {
            warmup_samples: self.warmup_samples,
            samples: self.samples.max(1),
            timeout_ms: self.timeout_ms.max(1),
            concurrency: self.concurrency.max(1),
        }
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms.max(1))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(clippy::struct_field_names)]
pub(crate) struct PerfQuantiles {
    pub(crate) min_ms: f64,
    pub(crate) mean_ms: f64,
    pub(crate) max_ms: f64,
    pub(crate) p50_ms: f64,
    pub(crate) p95_ms: f64,
    pub(crate) p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PerfSummary {
    pub(crate) total_ops: u64,
    pub(crate) success_ops: u64,
    pub(crate) timeout_ops: u64,
    pub(crate) error_ops: u64,
    pub(crate) error_rate: f64,
    pub(crate) throughput_qps: f64,
    pub(crate) elapsed_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PerfReport {
    pub(crate) schema_version: String,
    pub(crate) suite: String,
    pub(crate) case: String,
    pub(crate) mode: String,
    pub(crate) captured_at_unix_ms: u64,
    pub(crate) run_config: PerfRunConfig,
    pub(crate) summary: PerfSummary,
    pub(crate) quantiles: PerfQuantiles,
    pub(crate) sample_latency_ms: Vec<f64>,
    pub(crate) metadata: BTreeMap<String, String>,
    pub(crate) report_path: Option<String>,
}

impl PerfReport {
    pub(crate) fn add_metadata<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
    }
}

#[track_caller]
pub(crate) fn assert_perf_budget(report: &PerfReport, budget: &PerfBudget) {
    let mut violations = Vec::new();

    if let Some(limit) = budget.max_p50_latency_ms
        && report.quantiles.p50_ms > limit
    {
        violations.push(format!(
            "p50 latency exceeded: actual={:.3}ms budget<={:.3}ms",
            report.quantiles.p50_ms, limit
        ));
    }
    if let Some(limit) = budget.max_p95_latency_ms
        && report.quantiles.p95_ms > limit
    {
        violations.push(format!(
            "p95 latency exceeded: actual={:.3}ms budget<={:.3}ms",
            report.quantiles.p95_ms, limit
        ));
    }
    if let Some(limit) = budget.max_p99_latency_ms
        && report.quantiles.p99_ms > limit
    {
        violations.push(format!(
            "p99 latency exceeded: actual={:.3}ms budget<={:.3}ms",
            report.quantiles.p99_ms, limit
        ));
    }
    if let Some(limit) = budget.min_throughput_qps
        && report.summary.throughput_qps < limit
    {
        violations.push(format!(
            "throughput below floor: actual={:.3}qps budget>={:.3}qps",
            report.summary.throughput_qps, limit
        ));
    }
    if let Some(limit) = budget.max_error_rate
        && report.summary.error_rate > limit
    {
        violations.push(format!(
            "error rate exceeded: actual={:.5} budget<={:.5}",
            report.summary.error_rate, limit
        ));
    }

    if violations.is_empty() {
        return;
    }

    let mut message = String::new();
    let _ = writeln!(message, "performance budget gate failed");
    let _ = writeln!(message, "suite: {}", report.suite);
    let _ = writeln!(message, "case: {}", report.case);
    let _ = writeln!(message, "mode: {}", report.mode);
    let _ = writeln!(
        message,
        "summary: p50={:.3}ms p95={:.3}ms p99={:.3}ms throughput={:.3}qps error_rate={:.5}",
        report.quantiles.p50_ms,
        report.quantiles.p95_ms,
        report.quantiles.p99_ms,
        report.summary.throughput_qps,
        report.summary.error_rate
    );
    let _ = writeln!(
        message,
        "counts: total={} success={} timeout={} error={}",
        report.summary.total_ops,
        report.summary.success_ops,
        report.summary.timeout_ops,
        report.summary.error_ops
    );
    let _ = writeln!(message, "violations:");
    for violation in violations {
        let _ = writeln!(message, "- {violation}");
    }
    if let Some(path) = &report.report_path {
        let _ = writeln!(message, "report_path: {path}");
    }

    panic!("{message}");
}

pub(crate) fn run_sync_budget<T, E, F>(
    suite: &str,
    case: &str,
    config: &PerfRunConfig,
    mut operation: F,
) -> PerfReport
where
    F: FnMut() -> Result<T, E>,
    E: Display,
{
    let config = config.normalized();
    let timeout = config.timeout();

    for _ in 0..config.warmup_samples {
        for _ in 0..config.concurrency {
            let _ = operation();
        }
    }

    let started = Instant::now();
    let mut metrics = RunMetrics::with_capacity(config.samples * config.concurrency);

    for _ in 0..config.samples {
        for _ in 0..config.concurrency {
            let op_started = Instant::now();
            let result = operation();
            metrics.observe_result(op_started.elapsed(), timeout, result.is_err());
        }
    }

    finalize_report(suite, case, "sync", config, started.elapsed(), metrics)
}

#[cfg(feature = "performance-stress")]
pub(crate) async fn run_async_budget<T, E, Fut, F>(
    suite: &str,
    case: &str,
    config: &PerfRunConfig,
    operation: F,
) -> PerfReport
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let config = config.normalized();
    let timeout = config.timeout();

    for _ in 0..config.warmup_samples {
        for _ in 0..config.concurrency {
            let _ = tokio::time::timeout(timeout, operation()).await;
        }
    }

    let started = Instant::now();
    let mut metrics = RunMetrics::with_capacity(config.samples * config.concurrency);

    for _ in 0..config.samples {
        let mut batch = Vec::with_capacity(config.concurrency);
        for _ in 0..config.concurrency {
            batch.push(run_one_async(operation(), timeout));
        }

        for outcome in join_all(batch).await {
            metrics.observe_outcome(outcome);
        }
    }

    finalize_report(suite, case, "async", config, started.elapsed(), metrics)
}

struct RunMetrics {
    total_ops: u64,
    success_ops: u64,
    timeout_ops: u64,
    error_ops: u64,
    samples_ms: Vec<f64>,
}

impl RunMetrics {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            total_ops: 0,
            success_ops: 0,
            timeout_ops: 0,
            error_ops: 0,
            samples_ms: Vec::with_capacity(capacity),
        }
    }

    fn observe_result(&mut self, elapsed: Duration, timeout: Duration, failed: bool) {
        self.total_ops = self.total_ops.saturating_add(1);
        self.samples_ms.push(duration_to_ms(elapsed));
        if elapsed > timeout {
            self.timeout_ops = self.timeout_ops.saturating_add(1);
        } else if failed {
            self.error_ops = self.error_ops.saturating_add(1);
        } else {
            self.success_ops = self.success_ops.saturating_add(1);
        }
    }

    #[cfg(feature = "performance-stress")]
    fn observe_outcome(&mut self, outcome: AsyncOutcome) {
        self.total_ops = self.total_ops.saturating_add(1);
        self.samples_ms.push(duration_to_ms(outcome.elapsed));
        if outcome.timed_out {
            self.timeout_ops = self.timeout_ops.saturating_add(1);
        } else if outcome.failed {
            self.error_ops = self.error_ops.saturating_add(1);
        } else {
            self.success_ops = self.success_ops.saturating_add(1);
        }
    }
}

#[cfg(feature = "performance-stress")]
#[derive(Clone, Copy)]
struct AsyncOutcome {
    elapsed: Duration,
    timed_out: bool,
    failed: bool,
}

#[cfg(feature = "performance-stress")]
async fn run_one_async<T, E, Fut>(future: Fut, timeout: Duration) -> AsyncOutcome
where
    Fut: Future<Output = Result<T, E>>,
{
    let started = Instant::now();
    match tokio::time::timeout(timeout, future).await {
        Ok(Ok(_)) => AsyncOutcome {
            elapsed: started.elapsed(),
            timed_out: false,
            failed: false,
        },
        Ok(Err(_)) => AsyncOutcome {
            elapsed: started.elapsed(),
            timed_out: false,
            failed: true,
        },
        Err(_) => AsyncOutcome {
            elapsed: started.elapsed(),
            timed_out: true,
            failed: false,
        },
    }
}

fn finalize_report(
    suite: &str,
    case: &str,
    mode: &str,
    config: PerfRunConfig,
    elapsed: Duration,
    metrics: RunMetrics,
) -> PerfReport {
    let quantiles = summarize_quantiles(&metrics.samples_ms);
    let summary = build_summary(
        metrics.total_ops,
        metrics.success_ops,
        metrics.timeout_ops,
        metrics.error_ops,
        elapsed,
    );
    let mut report = PerfReport {
        schema_version: PERF_REPORT_SCHEMA_VERSION.to_string(),
        suite: suite.to_string(),
        case: case.to_string(),
        mode: mode.to_string(),
        captured_at_unix_ms: now_unix_ms(),
        run_config: config,
        summary,
        quantiles,
        sample_latency_ms: metrics.samples_ms,
        metadata: build_metadata(mode),
        report_path: None,
    };

    match persist_report(&mut report) {
        Ok(_) => {}
        Err(error) => report.add_metadata("report_write_error", error.to_string()),
    }

    report
}

fn summarize_quantiles(samples_ms: &[f64]) -> PerfQuantiles {
    if samples_ms.is_empty() {
        return PerfQuantiles::default();
    }

    let mut sorted = samples_ms.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let len = sorted.len();
    let sum: f64 = sorted.iter().sum();

    PerfQuantiles {
        min_ms: sorted[0],
        mean_ms: sum / bounded_usize_to_f64(len),
        max_ms: sorted[len - 1],
        p50_ms: sorted[percentile_index(len, 50, 100)],
        p95_ms: sorted[percentile_index(len, 95, 100)],
        p99_ms: sorted[percentile_index(len, 99, 100)],
    }
}

fn percentile_index(len: usize, numerator: usize, denominator: usize) -> usize {
    if len <= 1 {
        return 0;
    }

    let max_index = len.saturating_sub(1);
    let rounded = max_index
        .saturating_mul(numerator)
        .saturating_add(denominator / 2)
        / denominator;
    rounded.min(max_index)
}

fn build_summary(
    total_ops: u64,
    success_ops: u64,
    timeout_ops: u64,
    error_ops: u64,
    elapsed: Duration,
) -> PerfSummary {
    let failed_ops = timeout_ops.saturating_add(error_ops);
    let elapsed_secs = elapsed.as_secs_f64();
    let throughput_qps = if elapsed_secs > 0.0 {
        bounded_u64_to_f64(success_ops) / elapsed_secs
    } else {
        0.0
    };
    let error_rate = if total_ops > 0 {
        bounded_u64_to_f64(failed_ops) / bounded_u64_to_f64(total_ops)
    } else {
        0.0
    };

    PerfSummary {
        total_ops,
        success_ops,
        timeout_ops,
        error_ops,
        error_rate,
        throughput_qps,
        elapsed_ms: duration_to_ms(elapsed),
    }
}

fn build_metadata(mode: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("mode".to_string(), mode.to_string());

    if let Ok(value) = std::env::var("CARGO_PKG_NAME") {
        metadata.insert("crate".to_string(), value);
    }
    if let Ok(value) = std::env::var("PRJ_ROOT") {
        metadata.insert("project_root".to_string(), value);
    }
    if let Ok(value) = std::env::var("PRJ_RUNTIME_DIR") {
        metadata.insert("runtime_dir".to_string(), value);
    }

    metadata
}

fn persist_report(report: &mut PerfReport) -> std::io::Result<PathBuf> {
    let output_path = report_output_path(&report.suite, &report.case, report.captured_at_unix_ms);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let payload = serde_json::to_vec_pretty(report)
        .map_err(|error| std::io::Error::other(format!("serialize report: {error}")))?;
    std::fs::write(&output_path, payload)?;

    report.report_path = Some(output_path.display().to_string());
    Ok(output_path)
}

fn report_output_path(suite: &str, case: &str, captured_at_unix_ms: u64) -> PathBuf {
    let mut root = default_reports_root();

    let mut had_segment = false;
    for segment in suite.split('/') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        had_segment = true;
        root.push(sanitize_segment(segment, "suite"));
    }
    if !had_segment {
        root.push("default");
    }

    root.join(format!(
        "{}-{}.json",
        sanitize_segment(case, "case"),
        captured_at_unix_ms
    ))
}

fn default_reports_root() -> PathBuf {
    let runtime_dir = std::env::var("PRJ_RUNTIME_DIR").unwrap_or_else(|_| ".run".to_string());
    resolve_runtime_dir(&runtime_dir).join("reports")
}

fn resolve_runtime_dir(raw_runtime_dir: &str) -> PathBuf {
    let runtime_dir = Path::new(raw_runtime_dir);
    if runtime_dir.is_absolute() {
        return runtime_dir.to_path_buf();
    }

    if let Ok(project_root) = std::env::var("PRJ_ROOT") {
        return Path::new(&project_root).join(runtime_dir);
    }

    match std::env::current_dir() {
        Ok(current_dir) => current_dir.join(runtime_dir),
        Err(_) => PathBuf::from(raw_runtime_dir),
    }
}

fn sanitize_segment(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return fallback.to_string();
    }

    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else if ch == '/' || ch == '\\' {
            out.push('_');
        }
    }
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

fn now_unix_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        Err(_) => 0,
    }
}

fn duration_to_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn bounded_u64_to_f64(value: u64) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn bounded_usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}
