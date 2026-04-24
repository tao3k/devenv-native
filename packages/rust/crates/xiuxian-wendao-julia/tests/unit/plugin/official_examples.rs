use std::future::Future;
use std::process::{Command, Stdio};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use toml::Value;

use super::common::{
    ChildGuard, julia_example_project_for_package, repo_root, reserve_test_port,
    wendaoanalyzer_package_dir, wendaoarrow_package_dir, wendaosearch_config,
    wendaosearch_julia_project, wendaosearch_script,
};

pub(crate) const LIVE_SERVICE_STARTUP_TIMEOUT_SECS: u64 = 150;
pub(crate) const LIVE_REQUEST_TIMEOUT_SECS: u64 = 90;
pub(crate) const RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV: &str =
    "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST";
const PROCESS_MANAGED_WENDAOSEARCH_SERVICE_NAME: &str = "wendaosearch-solver-demo";

pub(crate) struct ProcessManagedWendaoSearchGuard {
    owned: bool,
}

pub(crate) fn spawn_real_wendaoarrow_service(port: u16) -> ChildGuard {
    spawn_wendaoarrow_example(
        "examples/stream_scoring_flight_server.jl",
        port,
        "spawn real WendaoArrow service",
    )
}

pub(crate) fn spawn_real_wendaoarrow_metadata_service(port: u16) -> ChildGuard {
    spawn_wendaoarrow_example(
        "examples/stream_metadata_flight_server.jl",
        port,
        "spawn real WendaoArrow metadata service",
    )
}

pub(crate) fn spawn_real_wendaoarrow_bad_response_service(port: u16) -> ChildGuard {
    spawn_wendaoarrow_example(
        "examples/stream_scoring_bad_response_flight_server.jl",
        port,
        "spawn real WendaoArrow bad-response service",
    )
}

pub(crate) fn spawn_real_wendaoanalyzer_linear_blend_service(port: u16) -> ChildGuard {
    spawn_wendaoanalyzer_example(
        &["--service-mode", "stream"],
        port,
        "spawn real WendaoAnalyzer linear blend service",
    )
}

pub(crate) fn spawn_real_wendaosearch_demo_capability_manifest_service(port: u16) -> ChildGuard {
    spawn_real_wendaosearch_service("capability_manifest", "demo", port)
}

pub(crate) fn spawn_real_wendaosearch_demo_multi_route_service(port: u16) -> ChildGuard {
    spawn_real_wendaosearch_multi_route_service("demo", port)
}

pub(crate) fn spawn_real_wendaosearch_solver_demo_multi_route_service(port: u16) -> ChildGuard {
    spawn_real_wendaosearch_multi_route_service("solver_demo", port)
}

fn spawn_real_wendaosearch_multi_route_service(mode: &str, port: u16) -> ChildGuard {
    let script = wendaosearch_script("run_search_service.jl");
    let child = Command::new("julia")
        .arg(format!(
            "--project={}",
            wendaosearch_julia_project().display()
        ))
        .arg(script)
        .arg("--route-names")
        .arg("capability_manifest,structural_rerank,constraint_filter")
        .arg("--mode")
        .arg(mode)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| {
            panic!("spawn real WendaoSearch `{route_name}` `{mode}` service: {error}")
        });
    ChildGuard::new(child)
}

fn spawn_wendaoarrow_example(example_path: &str, port: u16, error_context: &str) -> ChildGuard {
    let package_dir = wendaoarrow_package_dir();
    let project = julia_example_project_for_package(package_dir.as_path());
    let script = package_dir.join("scripts").join("run_flight_example.jl");
    let child = Command::new("julia")
        .arg(format!("--project={}", project.display()))
        .arg(script)
        .arg(example_path)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("{error_context}: {error}"));
    ChildGuard::new(child)
}

fn spawn_wendaoanalyzer_example(args: &[&str], port: u16, error_context: &str) -> ChildGuard {
    let package_dir = wendaoanalyzer_package_dir();
    let project = julia_example_project_for_package(package_dir.as_path());
    let script = package_dir.join("scripts").join("run_analyzer_example.jl");
    let child = Command::new("julia")
        .arg(format!("--project={}", project.display()))
        .arg(script)
        .args(args)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("{error_context}: {error}"));
    ChildGuard::new(child)
}

pub(crate) async fn wait_for_service_ready(base_url: &str) -> Result<(), String> {
    wait_for_service_ready_with_attempts(base_url, 150).await
}

pub(crate) async fn wait_for_service_ready_with_attempts(
    base_url: &str,
    attempts: usize,
) -> Result<(), String> {
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

pub(crate) fn reserve_real_service_port() -> u16 {
    reserve_test_port()
}

pub(crate) fn process_managed_wendaosearch_test_enabled() -> bool {
    std::env::var_os(RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV).is_some()
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
    let socket_addr = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))
        .unwrap_or(base_url)
        .to_string();
    TcpStream::connect(&socket_addr).await.is_ok()
}
