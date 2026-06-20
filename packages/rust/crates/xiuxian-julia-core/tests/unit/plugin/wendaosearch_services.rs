use std::future::Future;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use toml::Value;

use super::common::{
    ChildGuard, repo_root, reserve_test_port, wendaosearch_config, wendaosearch_julia_project,
    wendaosearch_package_dir, wendaosearch_script,
};

pub(crate) const LIVE_SERVICE_STARTUP_TIMEOUT_SECS: u64 = 150;
pub(crate) const LIVE_REQUEST_TIMEOUT_SECS: u64 = 90;
pub(crate) const RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV: &str =
    "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST";
const WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV: &str = "WENDAOSEARCH_SOLVER_DEMO_BASE_URL";
const PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME: &str = "wendaosearch-solver-demo";

pub(crate) struct ProcessManagedWendaoSearchGuard {
    owned: bool,
}

pub(crate) fn spawn_real_wendaosearch_demo_capability_manifest_service(port: u16) -> ChildGuard {
    spawn_real_wendaosearch_service("capability_manifest", "demo", port)
}

pub(crate) fn spawn_real_wendaosearch_demo_multi_route_service(port: u16) -> ChildGuard {
    spawn_real_wendaosearch_multi_route_service("demo", port)
}

pub(crate) fn spawn_real_wendaosearch_solver_demo_multi_route_service(port: u16) -> ChildGuard {
    spawn_real_wendaosearch_solver_demo_multi_route_service_with_options(port, true, None)
}

pub(crate) fn spawn_real_wendaosearch_solver_demo_multi_route_service_with_options(
    port: u16,
    warmup_on_start: bool,
    thread_pinning_policy: Option<&str>,
) -> ChildGuard {
    if configured_solver_demo_base_url().is_some() {
        return ChildGuard::external();
    }
    spawn_real_wendaosearch_multi_route_service_with_options(
        "solver_demo",
        port,
        warmup_on_start,
        thread_pinning_policy,
    )
}

fn spawn_real_wendaosearch_multi_route_service(mode: &str, port: u16) -> ChildGuard {
    spawn_real_wendaosearch_multi_route_service_with_options(mode, port, true, None)
}

fn spawn_real_wendaosearch_multi_route_service_with_options(
    mode: &str,
    port: u16,
    warmup_on_start: bool,
    thread_pinning_policy: Option<&str>,
) -> ChildGuard {
    let script = wendaosearch_script("run_search_service.jl");
    let mut command = Command::new("julia");
    command
        .arg(format!(
            "--project={}",
            wendaosearch_julia_project().display()
        ))
        .arg(script)
        .args([
            "--route-names",
            "capability_manifest,structural_rerank,constraint_filter",
            "--mode",
            mode,
            "--host",
            "127.0.0.1",
            "--port",
        ])
        .arg(port.to_string());
    if warmup_on_start {
        command.arg("--warmup-on-start");
    } else {
        command.arg("--no-warmup-on-start");
    }
    if let Some(policy) = thread_pinning_policy {
        command.args(["--thread-pinning-policy", policy]);
    }
    let child = command
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| {
            panic!("spawn real WendaoSearch multi-route `{mode}` service: {error}")
        });
    ChildGuard::new(child)
}

fn spawn_real_wendaosearch_service(route_name: &str, mode: &str, port: u16) -> ChildGuard {
    let script = wendaosearch_script("run_search_service.jl");
    let child = Command::new("julia")
        .arg(format!(
            "--project={}",
            wendaosearch_julia_project().display()
        ))
        .arg(script)
        .arg("--route-name")
        .arg(route_name)
        .arg("--mode")
        .arg(mode)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| {
            panic!("spawn real WendaoSearch `{route_name}` `{mode}` service: {error}")
        });
    ChildGuard::new(child)
}

pub(crate) async fn wait_for_service_ready_with_attempts(
    base_url: &str,
    attempts: usize,
) -> Result<(), String> {
    if let Some((host, port)) = parse_base_url_host_port(base_url) {
        for attempt in 0..attempts {
            if service_tcp_ready(&host, port).await
                && let Ok(true) = probe_wendaosearch_service(&host, port)
            {
                return Ok(());
            }
            if attempt + 1 == attempts {
                break;
            }
            sleep(Duration::from_millis(200)).await;
        }
        return Err(
            "real Julia WendaoSearch service did not become query-ready in time".to_string(),
        );
    }

    let socket_addr = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))
        .unwrap_or(base_url)
        .to_string();

    for _ in 0..attempts {
        if TcpStream::connect(&socket_addr).await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(200)).await;
    }
    Err("real Julia Flight service did not become ready in time".to_string())
}

fn parse_base_url_host_port(base_url: &str) -> Option<(String, u16)> {
    let socket_addr = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))
        .unwrap_or(base_url);
    let mut split = socket_addr.split(':');
    let host = split.next()?.to_string();
    let port = split.next()?.parse::<u16>().ok()?;
    if split.next().is_some() {
        return None;
    }
    Some((host, port))
}

async fn service_tcp_ready(host: &str, port: u16) -> bool {
    TcpStream::connect(format!("{host}:{port}")).await.is_ok()
}

fn probe_wendaosearch_service(host: &str, port: u16) -> Result<bool, String> {
    if !wendaosearch_probe_script_available() {
        return Ok(true);
    }

    let probe_script = wendaosearch_script("probe_search_service.jl");
    let mut command = Command::new("julia");
    command
        .arg(format!(
            "--project={}",
            wendaosearch_julia_project().display()
        ))
        .arg(probe_script)
        .arg("--route-name")
        .arg("capability_manifest")
        .arg("--host")
        .arg(host)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .current_dir(repo_root());

    let output = command
        .output()
        .map_err(|error| format!("spawn WendaoSearch probe command: {error}"))?;

    if output.status.success() {
        return Ok(true);
    }

    Ok(false)
}

fn wendaosearch_probe_script_available() -> bool {
    let probe_script = wendaosearch_package_dir().join("scripts/probe_search_service.jl");
    probe_script.is_file()
}

pub(crate) fn reserve_real_service_port() -> u16 {
    reserve_test_port()
}

pub(crate) fn solver_demo_multi_route_base_url_for_port(port: u16) -> String {
    configured_solver_demo_base_url().unwrap_or_else(|| format!("http://127.0.0.1:{port}"))
}

pub(crate) fn process_managed_wendaosearch_test_enabled() -> bool {
    std::env::var_os(RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV).is_some()
}

pub(crate) fn local_wendaosearch_package_available() -> bool {
    if let Some(configured) = std::env::var_os("WENDAOSEARCH_PACKAGE_DIR") {
        return existing_path(configured).is_some();
    }

    repo_root().join(".data/WendaoSearch.jl").is_dir()
}

pub(crate) fn solver_demo_wendaosearch_service_available() -> bool {
    configured_solver_demo_base_url().is_some() || local_wendaosearch_package_available()
}

fn existing_path(configured: impl Into<PathBuf>) -> Option<PathBuf> {
    let candidate = configured.into();
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        repo_root().join(candidate)
    };
    candidate.canonicalize().ok()
}

impl Drop for ProcessManagedWendaoSearchGuard {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }

        let output = devenv_processes_command(["down", PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME])
            .output()
            .unwrap_or_else(|error| {
                panic!(
                    "stop process-managed `{PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME}` service: {error}"
                )
            });
        if !output.status.success() {
            eprintln!(
                "warning: failed to stop process-managed `{PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME}` service:\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }
}

pub(crate) fn process_managed_wendaosearch_solver_demo_base_url() -> Result<String, String> {
    if let Some(base_url) = configured_solver_demo_base_url() {
        return Ok(base_url);
    }

    let config_path = wendaosearch_config("solver_demo.toml");
    let config_text = std::fs::read_to_string(&config_path)
        .map_err(|error| format!("read `{}`: {error}", config_path.display()))?;
    let config_value: Value = toml::from_str(&config_text)
        .map_err(|error| format!("parse `{}`: {error}", config_path.display()))?;
    let host = config_value
        .get("host")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("`{}` is missing string `host`", config_path.display()))?;
    let port = config_value
        .get("port")
        .and_then(Value::as_integer)
        .ok_or_else(|| format!("`{}` is missing integer `port`", config_path.display()))?;
    Ok(format!("http://{host}:{port}"))
}

fn configured_solver_demo_base_url() -> Option<String> {
    std::env::var(WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV)
        .ok()
        .filter(|value| !value.is_empty())
}

pub(crate) async fn ensure_process_managed_wendaosearch_solver_demo_service()
-> Result<ProcessManagedWendaoSearchGuard, String> {
    let base_url = process_managed_wendaosearch_solver_demo_base_url()?;
    if service_is_ready(&base_url).await {
        return Ok(ProcessManagedWendaoSearchGuard { owned: false });
    }

    let output = devenv_processes_command(["up", "-d", PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME])
        .output()
        .map_err(|error| {
            format!(
                "start process-managed `{PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME}` service: {error}"
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "start process-managed `{PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME}` service failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    timeout(
        Duration::from_secs(LIVE_SERVICE_STARTUP_TIMEOUT_SECS),
        wait_for_service_ready_with_attempts(&base_url, 600),
    )
    .await
    .map_err(|error| {
        format!(
            "wait for process-managed `{PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME}` service startup timeout: {error}"
        )
    })?
    .map_err(|error| {
        format!(
            "wait for process-managed `{PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME}` service: {error}"
        )
    })?;

    Ok(ProcessManagedWendaoSearchGuard { owned: true })
}

pub(crate) async fn await_live_step<F, T>(future: F, timeout_secs: u64, context: &str) -> T
where
    F: Future<Output = T>,
{
    match timeout(Duration::from_secs(timeout_secs), future).await {
        Ok(value) => value,
        Err(timeout_error) => {
            panic!("{context} timed out after {timeout_secs}s: {timeout_error}")
        }
    }
}

fn devenv_processes_command<const N: usize>(args: [&str; N]) -> Command {
    let mut command = Command::new("devenv");
    command
        .arg("processes")
        .args(args)
        .current_dir(repo_root())
        .env_remove("PC_CONFIG_FILES")
        .env_remove("PC_SOCKET_PATH");
    command
}

async fn service_is_ready(base_url: &str) -> bool {
    if let Some((host, port)) = parse_base_url_host_port(base_url) {
        if !service_tcp_ready(&host, port).await {
            return false;
        }
        return probe_wendaosearch_service(&host, port).unwrap_or(false);
    }

    let socket_addr = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))
        .unwrap_or(base_url)
        .to_string();
    TcpStream::connect(&socket_addr).await.is_ok()
}
