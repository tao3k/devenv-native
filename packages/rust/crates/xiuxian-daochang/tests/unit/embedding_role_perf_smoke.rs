//! Embedding role performance smoke tests and report generation harness.

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use reqwest::{Client, StatusCode, Url};
use serde::Serialize;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::Semaphore;

const DEFAULT_BASE_PORT: u16 = 18870;
const DEFAULT_SINGLE_RUNS: usize = 20;
const DEFAULT_BATCH_RUNS: usize = 10;
const DEFAULT_CONCURRENT_TOTAL: usize = 64;
const DEFAULT_CONCURRENT_WIDTH: usize = 8;
const DEFAULT_HEALTH_TIMEOUT_SECS: u64 = 120;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;
const DEFAULT_WARMUP_ATTEMPTS: usize = 4;
const DEFAULT_WARMUP_BACKOFF_MS: u64 = 1_500;
const DEFAULT_UPSTREAM_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_EMBEDDING_MODEL: &str = "qwen3-embedding:0.6b";
const DEFAULT_REPORT_REL_PATH: &str =
    ".run/reports/xiuxian-daochang-embedding-role-perf-smoke.json";
const DEFAULT_OLLAMA_AUTOSTART: bool = true;
const DEFAULT_OLLAMA_STARTUP_TIMEOUT_SECS: u64 = 45;
const DEFAULT_OLLAMA_LOG_REL_PATH: &str =
    ".run/logs/xiuxian-daochang-embedding-role-perf-ollama.log";

#[derive(Debug, Clone)]
struct PerfConfig {
    base_port: u16,
    single_runs: usize,
    batch_runs: usize,
    concurrent_total: usize,
    concurrent_width: usize,
    health_timeout_secs: u64,
    request_timeout_secs: u64,
    warmup_attempts: usize,
    warmup_backoff_ms: u64,
    upstream_base_url: String,
    embedding_model: String,
    report_path: PathBuf,
    max_single_p95_ms: Option<f64>,
    max_batch8_p95_ms: Option<f64>,
    min_concurrent_rps: Option<f64>,
    ollama_autostart: bool,
    ollama_models: String,
    ollama_startup_timeout_secs: u64,
}

#[derive(Debug, Clone)]
struct RoleConfig {
    name: &'static str,
    port: u16,
    model: String,
    upstream_model: String,
    settings_toml: String,
}

#[derive(Debug)]
struct ManagedProcess {
    child: Child,
}

impl ManagedProcess {
    fn is_running(&mut self) -> Result<bool> {
        Ok(self.child.try_wait()?.is_none())
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug, Serialize)]
struct SequentialReport {
    count: usize,
    ok: usize,
    err: usize,
    errors: Vec<String>,
    avg_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

#[derive(Debug, Serialize)]
struct ConcurrentReport {
    count: usize,
    ok: usize,
    err: usize,
    concurrency: usize,
    rps: f64,
    errors: Vec<String>,
    avg_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

#[derive(Debug, Serialize)]
struct RoleReport {
    role: String,
    endpoint: String,
    model: String,
    single: SequentialReport,
    batch8: SequentialReport,
    concurrent_single: ConcurrentReport,
    log_file: String,
}

#[derive(Debug, Serialize)]
struct PerfReport {
    schema: String,
    generated_at_epoch: u64,
    duration_secs: f64,
    base_port: u16,
    upstream_base_url: String,
    embedding_model: String,
    single_runs: usize,
    batch_runs: usize,
    concurrent_total: usize,
    concurrent_width: usize,
    roles: Vec<RoleReport>,
    status: String,
    failures: Vec<String>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "heavy benchmark; run with --ignored --nocapture to validate embedding role performance"]
async fn embedding_role_perf_smoke_reports_metrics() -> Result<()> {
    let workspace_root = workspace_root()?;
    let config = PerfConfig::from_env(&workspace_root)?;
    let client = build_http_client(config.request_timeout_secs)?;
    let _ollama_guard = ensure_local_ollama_if_needed(&workspace_root, &client, &config).await?;

    let temp_root = tempfile::tempdir().context("create temp dir for embedding role benchmark")?;
    let agent_bin = resolve_agent_binary(&workspace_root)?;

    let started = Instant::now();
    let mut role_reports = Vec::with_capacity(2);
    for role in build_role_configs(&config, temp_root.path()) {
        let role_report =
            run_role_benchmark(&workspace_root, &client, &config, &agent_bin, role).await?;
        role_reports.push(role_report);
    }

    let mut failures = Vec::new();
    for role in &role_reports {
        collect_failures_for_role(role, &config, &mut failures);
    }

    if let Some(parent) = config.report_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "create report parent directory: {}",
                parent.as_os_str().to_string_lossy()
            )
        })?;
    }

    let payload = PerfReport {
        schema: "xiuxian_daochang.embedding.role_perf_smoke.v1".to_string(),
        generated_at_epoch: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock before unix epoch")?
            .as_secs(),
        duration_secs: round2(started.elapsed().as_secs_f64()),
        base_port: config.base_port,
        upstream_base_url: config.upstream_base_url.clone(),
        embedding_model: config.embedding_model.clone(),
        single_runs: config.single_runs,
        batch_runs: config.batch_runs,
        concurrent_total: config.concurrent_total,
        concurrent_width: config.concurrent_width,
        roles: role_reports,
        status: if failures.is_empty() {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        failures: failures.clone(),
    };

    let mut file = File::create(&config.report_path).with_context(|| {
        format!(
            "create embedding perf report: {}",
            config.report_path.as_os_str().to_string_lossy()
        )
    })?;
    serde_json::to_writer_pretty(&mut file, &payload).context("serialize embedding perf report")?;
    file.write_all(b"\n")
        .context("append newline to embedding perf report")?;

    println!("{}", config.report_path.display());
    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("FAIL: {failure}");
        }
        bail!(
            "embedding role perf smoke failed with {} issue(s)",
            failures.len()
        );
    }

    Ok(())
}

impl PerfConfig {
    fn from_env(workspace_root: &Path) -> Result<Self> {
        let base_port = env_u16("OMNI_EMBED_BASE_PORT", DEFAULT_BASE_PORT)?;
        let single_runs = env_usize("OMNI_EMBED_SINGLE_RUNS", DEFAULT_SINGLE_RUNS)?;
        let batch_runs = env_usize("OMNI_EMBED_BATCH_RUNS", DEFAULT_BATCH_RUNS)?;
        let concurrent_total = env_usize("OMNI_EMBED_CONCURRENT_TOTAL", DEFAULT_CONCURRENT_TOTAL)?;
        let concurrent_width = env_usize("OMNI_EMBED_CONCURRENT_WIDTH", DEFAULT_CONCURRENT_WIDTH)?;
        let health_timeout_secs = env_u64(
            "OMNI_EMBED_HEALTH_TIMEOUT_SECS",
            DEFAULT_HEALTH_TIMEOUT_SECS,
        )?;
        let request_timeout_secs = env_u64(
            "OMNI_EMBED_REQUEST_TIMEOUT_SECS",
            DEFAULT_REQUEST_TIMEOUT_SECS,
        )?;
        let warmup_attempts = env_usize("OMNI_EMBED_WARMUP_ATTEMPTS", DEFAULT_WARMUP_ATTEMPTS)?;
        let warmup_backoff_ms = env_u64("OMNI_EMBED_WARMUP_BACKOFF_MS", DEFAULT_WARMUP_BACKOFF_MS)?;

        let upstream_base_url = env_string(
            "OMNI_EMBED_UPSTREAM_BASE_URL",
            DEFAULT_UPSTREAM_BASE_URL.to_string(),
        );
        let embedding_model =
            env_string("OMNI_EMBED_BASE_MODEL", DEFAULT_EMBEDDING_MODEL.to_string());

        let report_path = resolve_report_path(
            workspace_root,
            env::var("XIUXIAN_DAOCHANG_EMBED_ROLE_PERF_REPORT").ok(),
        );

        let max_single_p95_ms = env_optional_f64("OMNI_EMBED_MAX_SINGLE_P95_MS")?;
        let max_batch8_p95_ms = env_optional_f64("OMNI_EMBED_MAX_BATCH8_P95_MS")?;
        let min_concurrent_rps = env_optional_f64("OMNI_EMBED_MIN_CONCURRENT_RPS")?;

        let ollama_autostart =
            env_optional_bool("OMNI_EMBED_AUTOSTART_OLLAMA")?.unwrap_or(DEFAULT_OLLAMA_AUTOSTART);
        let ollama_startup_timeout_secs = env_u64(
            "OMNI_EMBED_OLLAMA_STARTUP_TIMEOUT_SECS",
            DEFAULT_OLLAMA_STARTUP_TIMEOUT_SECS,
        )?;
        let ollama_models = resolve_ollama_models(workspace_root);

        if single_runs == 0 || batch_runs == 0 || concurrent_total == 0 || concurrent_width == 0 {
            bail!(
                "run counts and concurrency must be positive: single_runs={single_runs}, batch_runs={batch_runs}, concurrent_total={concurrent_total}, concurrent_width={concurrent_width}"
            );
        }
        if concurrent_width > concurrent_total {
            bail!(
                "concurrent_width must be <= concurrent_total: width={concurrent_width}, total={concurrent_total}"
            );
        }
        if warmup_attempts == 0 {
            bail!("OMNI_EMBED_WARMUP_ATTEMPTS must be positive");
        }
        if request_timeout_secs == 0 || health_timeout_secs == 0 {
            bail!(
                "timeout values must be positive: request_timeout_secs={request_timeout_secs}, health_timeout_secs={health_timeout_secs}"
            );
        }
        if upstream_base_url.trim().is_empty() {
            bail!("OMNI_EMBED_UPSTREAM_BASE_URL must be non-empty");
        }
        if embedding_model.trim().is_empty() {
            bail!("OMNI_EMBED_BASE_MODEL must be non-empty");
        }

        Ok(Self {
            base_port,
            single_runs,
            batch_runs,
            concurrent_total,
            concurrent_width,
            health_timeout_secs,
            request_timeout_secs,
            warmup_attempts,
            warmup_backoff_ms,
            upstream_base_url: upstream_base_url.trim().trim_end_matches('/').to_string(),
            embedding_model: embedding_model.trim().to_string(),
            report_path,
            max_single_p95_ms,
            max_batch8_p95_ms,
            min_concurrent_rps,
            ollama_autostart,
            ollama_models,
            ollama_startup_timeout_secs,
        })
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(4)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            anyhow!(
                "failed to resolve workspace root from {}",
                manifest_dir.display()
            )
        })
}

fn build_role_configs(config: &PerfConfig, temp_root: &Path) -> Vec<RoleConfig> {
    let base_model = config.embedding_model.clone();
    let litellm_model = format!("ollama/{base_model}");

    let litellm_memory_path = temp_root.join("memory-litellm-rs");
    let litellm_settings = format!(
        "[agent]\n\
         llm_backend = \"litellm_rs\"\n\
         \n\
         [llm.embedding]\n\
         backend = \"litellm_rs\"\n\
         batch_max_size = 128\n\
         batch_max_concurrency = 1\n\
         litellm_model = \"{litellm_model}\"\n\
         model = \"{litellm_model}\"\n\
         litellm_api_base = \"{}\"\n\
         \n\
         [memory]\n\
         embedding_backend = \"litellm_rs\"\n\
         embedding_model = \"{litellm_model}\"\n\
         persistence_backend = \"local\"\n\
         path = \"{}\"\n\
         \n\
         [tool_runtime]\n\
         strict_startup = false\n",
        config.upstream_base_url,
        litellm_memory_path.display()
    );

    let openai_http_memory_path = temp_root.join("memory-openai-http");
    let openai_http_settings = format!(
        "[agent]\n\
         llm_backend = \"http\"\n\
         \n\
         [llm.embedding]\n\
         backend = \"openai_http\"\n\
         batch_max_size = 128\n\
         batch_max_concurrency = 1\n\
         model = \"{base_model}\"\n\
         litellm_api_base = \"{}\"\n\
         \n\
         [memory]\n\
         embedding_backend = \"openai_http\"\n\
         embedding_model = \"{base_model}\"\n\
         persistence_backend = \"local\"\n\
         path = \"{}\"\n\
         \n\
         [tool_runtime]\n\
         strict_startup = false\n",
        config.upstream_base_url,
        openai_http_memory_path.display()
    );

    vec![
        RoleConfig {
            name: "litellm_rs",
            port: config.base_port,
            model: litellm_model,
            upstream_model: base_model.clone(),
            settings_toml: litellm_settings,
        },
        RoleConfig {
            name: "openai_http",
            port: config.base_port + 1,
            model: base_model.clone(),
            upstream_model: base_model,
            settings_toml: openai_http_settings,
        },
    ]
}

fn resolve_report_path(workspace_root: &Path, report_override: Option<String>) -> PathBuf {
    if let Some(raw) = report_override
        && !raw.trim().is_empty()
    {
        let raw_path = PathBuf::from(raw.trim());
        if raw_path.is_absolute() {
            return raw_path;
        }
        return workspace_root.join(raw_path);
    }
    workspace_root.join(DEFAULT_REPORT_REL_PATH)
}

fn resolve_ollama_models(workspace_root: &Path) -> String {
    if let Ok(explicit) = env::var("OLLAMA_MODELS")
        && !explicit.trim().is_empty()
    {
        return explicit.trim().to_string();
    }

    if let Ok(prj_data_home) = env::var("PRJ_DATA_HOME")
        && !prj_data_home.trim().is_empty()
    {
        return format!("{}/models", prj_data_home.trim());
    }

    workspace_root.join(".data/models").display().to_string()
}

fn resolve_agent_binary(workspace_root: &Path) -> Result<PathBuf> {
    if let Ok(explicit) = env::var("XIUXIAN_DAOCHANG_BIN")
        && !explicit.trim().is_empty()
    {
        let explicit_path = PathBuf::from(explicit.trim());
        if explicit_path.is_file() {
            return Ok(explicit_path);
        }
        bail!(
            "XIUXIAN_DAOCHANG_BIN does not exist: {}",
            explicit_path.display()
        );
    }

    let target_dir = env::var("CARGO_TARGET_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map_or_else(
            || workspace_root.join("target"),
            |value| {
                let path = PathBuf::from(value);
                if path.is_absolute() {
                    path
                } else {
                    workspace_root.join(path)
                }
            },
        );
    let agent_bin = target_dir.join("debug/xiuxian-daochang");
    if agent_bin.is_file() {
        return Ok(agent_bin);
    }

    let status = Command::new("cargo")
        .current_dir(workspace_root)
        .args([
            "build",
            "-p",
            "xiuxian-daochang",
            "--bin",
            "xiuxian-daochang",
        ])
        .status()
        .context("run cargo build for xiuxian-daochang binary")?;
    if !status.success() {
        bail!("cargo build -p xiuxian-daochang --bin xiuxian-daochang failed with status {status}");
    }
    if !agent_bin.is_file() {
        bail!(
            "xiuxian-daochang binary missing after build: {}",
            agent_bin.display()
        );
    }
    Ok(agent_bin)
}

fn build_http_client(request_timeout_secs: u64) -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(request_timeout_secs))
        .pool_max_idle_per_host(32)
        .build()
        .context("build reqwest client for embedding perf smoke")
}

fn parse_upstream_local_bind(base_url: &str) -> Option<SocketAddr> {
    let parsed = Url::parse(base_url).ok()?;
    let host = parsed.host_str()?;
    if host != "127.0.0.1" && host != "localhost" {
        return None;
    }
    let port = parsed.port_or_known_default()?;
    format!("127.0.0.1:{port}").parse::<SocketAddr>().ok()
}

fn is_port_listening(addr: SocketAddr) -> bool {
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

async fn ensure_local_ollama_if_needed(
    workspace_root: &Path,
    client: &Client,
    config: &PerfConfig,
) -> Result<Option<ManagedProcess>> {
    if !config.ollama_autostart {
        return Ok(None);
    }
    let Some(bind_addr) = parse_upstream_local_bind(&config.upstream_base_url) else {
        return Ok(None);
    };
    if is_port_listening(bind_addr) {
        return Ok(None);
    }

    let log_path = workspace_root.join(DEFAULT_OLLAMA_LOG_REL_PATH);
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create ollama log dir: {}", parent.display()))?;
    }
    let stdout = File::create(&log_path)
        .with_context(|| format!("create ollama stdout log: {}", log_path.display()))?;
    let stderr = stdout
        .try_clone()
        .context("clone ollama stdout log fd for stderr")?;

    let mut command = Command::new("ollama");
    command
        .arg("serve")
        .env("OLLAMA_MODELS", &config.ollama_models)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let mut process = ManagedProcess {
        child: command
            .spawn()
            .context("spawn local ollama serve for embedding perf smoke")?,
    };

    let version_url = format!("{}/api/version", config.upstream_base_url);
    let deadline = Instant::now() + Duration::from_secs(config.ollama_startup_timeout_secs);
    loop {
        if !process.is_running()? {
            bail!(
                "local ollama process exited before startup completed; see {}",
                log_path.display()
            );
        }
        if let Ok(resp) = client.get(&version_url).send().await
            && resp.status().is_success()
        {
            return Ok(Some(process));
        }
        if Instant::now() >= deadline {
            bail!(
                "timed out waiting for local ollama startup at {version_url}; OLLAMA_MODELS='{}', log={}",
                config.ollama_models,
                log_path.display()
            );
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

async fn run_role_benchmark(
    workspace_root: &Path,
    client: &Client,
    config: &PerfConfig,
    agent_bin: &Path,
    role: RoleConfig,
) -> Result<RoleReport> {
    assert_upstream_embedding_ready(client, config, &role.upstream_model).await?;

    let temp = TempDir::new().context("create role benchmark temp dir")?;
    let conf_root = write_role_config(temp.path(), &role)?;
    let log_path = temp
        .path()
        .join(format!("xiuxian-daochang-{}.log", role.name));
    let gateway =
        spawn_gateway_process(workspace_root, agent_bin, &conf_root, role.port, &log_path)?;

    let health_url = format!("http://127.0.0.1:{}/health", role.port);
    wait_for_health(
        client,
        &health_url,
        Duration::from_secs(config.health_timeout_secs),
    )
    .await
    .with_context(|| format!("gateway health check failed for role '{}'", role.name))?;

    let endpoint = format!("http://127.0.0.1:{}/v1/embeddings", role.port);
    warm_up_role_endpoint(client, config, &endpoint, &role.model).await?;

    let single = run_sequential(
        client,
        &endpoint,
        &role.model,
        config.single_runs,
        1,
        "single",
    )
    .await;
    let batch8 = run_sequential(
        client,
        &endpoint,
        &role.model,
        config.batch_runs,
        8,
        "batch8",
    )
    .await;
    let concurrent = run_concurrent(
        client,
        &endpoint,
        &role.model,
        config.concurrent_total,
        config.concurrent_width,
    )
    .await;

    drop(gateway);
    // Preserve role log under project .run for post-run diagnosis.
    let role_log_target = workspace_root
        .join(".run/logs")
        .join(format!("xiuxian-daochang-embedding-role-{}.log", role.name));
    if let Some(parent) = role_log_target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create role log parent dir: {}", parent.display()))?;
    }
    fs::copy(&log_path, &role_log_target).with_context(|| {
        format!(
            "copy role log {} -> {}",
            log_path.display(),
            role_log_target.display()
        )
    })?;

    Ok(RoleReport {
        role: role.name.to_string(),
        endpoint,
        model: role.model.clone(),
        single,
        batch8,
        concurrent_single: concurrent,
        log_file: role_log_target.display().to_string(),
    })
}

fn write_role_config(temp_root: &Path, role: &RoleConfig) -> Result<PathBuf> {
    let conf_root = temp_root.join(format!("conf-{}", role.name));
    let conf_dir = conf_root.join("xiuxian-artisan-workshop");
    fs::create_dir_all(&conf_dir)
        .with_context(|| format!("create role config dir: {}", conf_dir.display()))?;
    let settings_path = conf_dir.join("xiuxian.toml");
    fs::write(&settings_path, role.settings_toml.as_bytes())
        .with_context(|| format!("write role settings toml: {}", settings_path.display()))?;
    Ok(conf_root)
}

fn spawn_gateway_process(
    workspace_root: &Path,
    agent_bin: &Path,
    conf_root: &Path,
    port: u16,
    log_path: &Path,
) -> Result<ManagedProcess> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create gateway log dir: {}", parent.display()))?;
    }
    let stdout = File::create(log_path)
        .with_context(|| format!("create gateway stdout log: {}", log_path.display()))?;
    let stderr = stdout
        .try_clone()
        .context("clone gateway stdout log fd for stderr")?;

    let mut command = Command::new(agent_bin);
    command
        .current_dir(workspace_root)
        .env("XIUXIAN_DAOCHANG_TOOL_STRICT_STARTUP", "false")
        .env(
            "RUST_LOG",
            env::var("RUST_LOG").unwrap_or_else(|_| "xiuxian_daochang=warn".to_string()),
        )
        .arg("--conf")
        .arg(conf_root)
        .arg("gateway")
        .arg("--bind")
        .arg(format!("127.0.0.1:{port}"))
        .arg("--tool-config")
        .arg(".tool.json")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));

    let child = command.spawn().with_context(|| {
        format!(
            "spawn gateway process for role benchmark on port {port} with conf root {}",
            conf_root.display()
        )
    })?;
    Ok(ManagedProcess { child })
}

async fn wait_for_health(client: &Client, health_url: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(resp) = client.get(health_url).send().await
            && resp.status().is_success()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("gateway health not ready within {timeout:?}: {health_url}");
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn assert_upstream_embedding_ready(
    client: &Client,
    config: &PerfConfig,
    upstream_model: &str,
) -> Result<()> {
    let models_url = format!("{}/v1/models", config.upstream_base_url);
    if let Ok(resp) = client.get(&models_url).send().await
        && resp.status().is_success()
    {
        let payload: Value = resp.json().await.unwrap_or_else(|_| json!({}));
        let available = extract_openai_model_ids(&payload);
        if !available.is_empty() && !available.iter().any(|id| id == upstream_model) {
            let joined = available.join(", ");
            bail!(
                "upstream model not found: required='{upstream_model}', available=[{joined}] (hint: verify OLLAMA_MODELS='{}')",
                config.ollama_models
            );
        }
    }

    let probe_payload = json!({
        "input": ["upstream readiness probe"],
        "model": upstream_model,
    });
    let endpoint = format!("{}/v1/embeddings", config.upstream_base_url);
    for attempt in 1..=config.warmup_attempts {
        match client.post(&endpoint).json(&probe_payload).send().await {
            Ok(resp) if resp.status() == StatusCode::OK => return Ok(()),
            Ok(resp) => {
                let status = resp.status();
                let body_preview = response_body_preview(resp).await;
                if attempt < config.warmup_attempts && status.is_server_error() {
                    tokio::time::sleep(Duration::from_millis(config.warmup_backoff_ms)).await;
                    continue;
                }
                bail!(
                    "upstream readiness probe failed status={status}, model='{upstream_model}', endpoint='{endpoint}', body='{body_preview}'"
                );
            }
            Err(error) => {
                if attempt < config.warmup_attempts {
                    tokio::time::sleep(Duration::from_millis(config.warmup_backoff_ms)).await;
                    continue;
                }
                bail!(
                    "upstream readiness probe request failed model='{upstream_model}', endpoint='{endpoint}', error={error}; hint: ensure Ollama is running and OLLAMA_MODELS='{}'",
                    config.ollama_models
                );
            }
        }
    }
    bail!("unreachable readiness probe flow")
}

async fn warm_up_role_endpoint(
    client: &Client,
    config: &PerfConfig,
    endpoint: &str,
    model: &str,
) -> Result<()> {
    for warmup_idx in 0..2 {
        let payload = json!({
            "input": [format!("warmup request #{warmup_idx}")],
            "model": model,
        });
        let mut success = false;
        for attempt in 1..=config.warmup_attempts {
            match client.post(endpoint).json(&payload).send().await {
                Ok(resp) if resp.status() == StatusCode::OK => {
                    success = true;
                    break;
                }
                Ok(resp) => {
                    if attempt < config.warmup_attempts && resp.status().is_server_error() {
                        tokio::time::sleep(Duration::from_millis(config.warmup_backoff_ms)).await;
                        continue;
                    }
                    let status = resp.status();
                    let body_preview = response_body_preview(resp).await;
                    bail!(
                        "role warmup failed endpoint='{endpoint}', model='{model}', status={status}, body='{body_preview}'"
                    );
                }
                Err(error) => {
                    if attempt < config.warmup_attempts {
                        tokio::time::sleep(Duration::from_millis(config.warmup_backoff_ms)).await;
                        continue;
                    }
                    bail!(
                        "role warmup request failed endpoint='{endpoint}', model='{model}', error={error}"
                    );
                }
            }
        }
        if !success {
            bail!("role warmup exhausted retries endpoint='{endpoint}', model='{model}'");
        }
    }
    Ok(())
}

async fn run_sequential(
    client: &Client,
    endpoint: &str,
    model: &str,
    count: usize,
    items_per_request: usize,
    tag: &str,
) -> SequentialReport {
    let mut ok = 0usize;
    let mut errors = Vec::new();
    let mut latencies_ms = Vec::with_capacity(count);

    for index in 0..count {
        let payload = json!({
            "input": build_inputs(tag, index, items_per_request),
            "model": model,
        });
        let started = Instant::now();
        match client.post(endpoint).json(&payload).send().await {
            Ok(resp) if resp.status() == StatusCode::OK => {
                ok += 1;
            }
            Ok(resp) => {
                let status = resp.status();
                let body_preview = response_body_preview(resp).await;
                errors.push(format!("status={status} body={body_preview}"));
            }
            Err(error) => {
                errors.push(error.to_string());
            }
        }
        latencies_ms.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    let err = count.saturating_sub(ok);
    let (avg_ms, p95_ms, max_ms) = calc_latency_stats(&latencies_ms);
    SequentialReport {
        count,
        ok,
        err,
        errors: truncate_errors(errors),
        avg_ms,
        p95_ms,
        max_ms,
    }
}

async fn run_concurrent(
    client: &Client,
    endpoint: &str,
    model: &str,
    total: usize,
    concurrency: usize,
) -> ConcurrentReport {
    let started = Instant::now();
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut tasks = tokio::task::JoinSet::new();

    for index in 0..total {
        let endpoint = endpoint.to_string();
        let model = model.to_string();
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        tasks.spawn(async move {
            let permit = semaphore
                .acquire_owned()
                .await
                .map_err(|error| anyhow!("concurrency semaphore closed: {error}"))?;
            let _permit = permit;
            let payload = json!({
                "input": build_inputs("concurrent", index, 1),
                "model": model,
            });
            let request_started = Instant::now();
            let request_outcome = match client.post(endpoint).json(&payload).send().await {
                Ok(resp) if resp.status() == StatusCode::OK => Ok(()),
                Ok(resp) => {
                    let status = resp.status();
                    let body_preview = response_body_preview(resp).await;
                    Err(format!("status={status} body={body_preview}"))
                }
                Err(error) => Err(error.to_string()),
            };
            Ok::<_, anyhow::Error>((request_outcome, request_started.elapsed()))
        });
    }

    let mut ok = 0usize;
    let mut errors = Vec::new();
    let mut latencies_ms = Vec::with_capacity(total);
    while let Some(join_result) = tasks.join_next().await {
        match join_result {
            Ok(Ok((request_outcome, elapsed))) => {
                latencies_ms.push(elapsed.as_secs_f64() * 1000.0);
                match request_outcome {
                    Ok(()) => ok += 1,
                    Err(error) => errors.push(error),
                }
            }
            Ok(Err(error)) => {
                errors.push(error.to_string());
            }
            Err(error) => {
                errors.push(format!("join error: {error}"));
            }
        }
    }

    let elapsed_secs = started.elapsed().as_secs_f64().max(1e-9);
    let err = total.saturating_sub(ok);
    let (avg_ms, p95_ms, max_ms) = calc_latency_stats(&latencies_ms);
    ConcurrentReport {
        count: total,
        ok,
        err,
        concurrency,
        rps: round2(usize_to_f64(total) / elapsed_secs),
        errors: truncate_errors(errors),
        avg_ms,
        p95_ms,
        max_ms,
    }
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn calc_latency_stats(latencies_ms: &[f64]) -> (f64, f64, f64) {
    if latencies_ms.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let avg = latencies_ms.iter().sum::<f64>() / usize_to_f64(latencies_ms.len());
    let p95 = percentile(latencies_ms, 0.95);
    let max = latencies_ms.iter().copied().fold(0.0, f64::max);
    (round2(avg), round2(p95), round2(max))
}

fn percentile(values: &[f64], pct: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    if values.len() == 1 {
        return values[0];
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let rank = usize_to_f64(ordered.len() - 1) * pct;
    let lower = (0..ordered.len())
        .rev()
        .find(|index| usize_to_f64(*index) <= rank)
        .unwrap_or(0);
    let upper = (0..ordered.len())
        .find(|index| usize_to_f64(*index) >= rank)
        .unwrap_or(ordered.len() - 1);
    if lower == upper {
        ordered[lower]
    } else {
        let weight = rank - usize_to_f64(lower);
        ordered[lower] * (1.0 - weight) + ordered[upper] * weight
    }
}

fn build_inputs(tag: &str, index: usize, items_per_request: usize) -> Vec<String> {
    (0..items_per_request)
        .map(|item_idx| format!("embedding role perf {tag} request #{index} item #{item_idx}"))
        .collect()
}

fn collect_failures_for_role(role: &RoleReport, config: &PerfConfig, failures: &mut Vec<String>) {
    if role.single.err > 0 {
        failures.push(format!("{}: single err={}", role.role, role.single.err));
    }
    if role.batch8.err > 0 {
        failures.push(format!("{}: batch8 err={}", role.role, role.batch8.err));
    }
    if role.concurrent_single.err > 0 {
        failures.push(format!(
            "{}: concurrent_single err={}",
            role.role, role.concurrent_single.err
        ));
    }

    if let Some(max_single_p95_ms) = config.max_single_p95_ms
        && role.single.p95_ms > max_single_p95_ms
    {
        failures.push(format!(
            "{}: single p95 {:.2}ms > {:.2}ms",
            role.role, role.single.p95_ms, max_single_p95_ms
        ));
    }
    if let Some(max_batch8_p95_ms) = config.max_batch8_p95_ms
        && role.batch8.p95_ms > max_batch8_p95_ms
    {
        failures.push(format!(
            "{}: batch8 p95 {:.2}ms > {:.2}ms",
            role.role, role.batch8.p95_ms, max_batch8_p95_ms
        ));
    }
    if let Some(min_concurrent_rps) = config.min_concurrent_rps
        && role.concurrent_single.rps < min_concurrent_rps
    {
        failures.push(format!(
            "{}: concurrent rps {:.2} < {:.2}",
            role.role, role.concurrent_single.rps, min_concurrent_rps
        ));
    }
}

async fn response_body_preview(resp: reqwest::Response) -> String {
    match resp.text().await {
        Ok(body) => body.chars().take(160).collect(),
        Err(_) => String::new(),
    }
}

fn truncate_errors(errors: Vec<String>) -> Vec<String> {
    errors
        .into_iter()
        .take(3)
        .map(|entry| entry.chars().take(220).collect())
        .collect()
}

fn extract_openai_model_ids(payload: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    let Some(items) = payload.get("data").and_then(Value::as_array) else {
        return ids;
    };
    for item in items {
        if let Some(id) = item
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            ids.push(id.to_string());
        } else if let Some(name) = item
            .get("model")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            ids.push(name.to_string());
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn env_string(key: &str, default: String) -> String {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
}

fn env_u16(key: &str, default: u16) -> Result<u16> {
    match env::var(key) {
        Ok(raw) => raw
            .trim()
            .parse::<u16>()
            .with_context(|| format!("parse {key}='{raw}' as u16")),
        Err(_) => Ok(default),
    }
}

fn env_u64(key: &str, default: u64) -> Result<u64> {
    match env::var(key) {
        Ok(raw) => raw
            .trim()
            .parse::<u64>()
            .with_context(|| format!("parse {key}='{raw}' as u64")),
        Err(_) => Ok(default),
    }
}

fn env_usize(key: &str, default: usize) -> Result<usize> {
    match env::var(key) {
        Ok(raw) => raw
            .trim()
            .parse::<usize>()
            .with_context(|| format!("parse {key}='{raw}' as usize")),
        Err(_) => Ok(default),
    }
}

fn env_optional_f64(key: &str) -> Result<Option<f64>> {
    match env::var(key) {
        Ok(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                let value = trimmed
                    .parse::<f64>()
                    .with_context(|| format!("parse {key}='{raw}' as f64"))?;
                Ok(Some(value))
            }
        }
        Err(_) => Ok(None),
    }
}

fn env_optional_bool(key: &str) -> Result<Option<bool>> {
    match env::var(key) {
        Ok(raw) => {
            let trimmed = raw.trim().to_ascii_lowercase();
            if trimmed.is_empty() {
                return Ok(None);
            }
            match trimmed.as_str() {
                "1" | "true" | "yes" | "on" => Ok(Some(true)),
                "0" | "false" | "no" | "off" => Ok(Some(false)),
                _ => bail!("parse {key}='{raw}' as bool"),
            }
        }
        Err(_) => Ok(None),
    }
}
