use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use super::common::{
    JuliaExampleServiceGuard, repo_root, reserve_service_port, wait_for_service_ready,
    wait_for_service_ready_with_attempts, wendaoanalyzer_script, wendaoarrow_script,
    wendaosearch_package_dir, wendaosearch_parser_summary_contract, wendaosearch_script,
};
use crate::compatibility::link_graph::{
    DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH, LinkGraphJuliaAnalyzerLaunchManifest,
    LinkGraphJuliaDeploymentArtifact, LinkGraphJuliaRerankRuntimeConfig,
};
use crate::{
    fetch_modelica_ast_query_analysis_blocking_for_repository,
    fetch_modelica_parser_file_summary_blocking_for_repository,
};

const MODELICA_PARSER_SUMMARY_READY_SOURCE_ID: &str = "Warmup/package.mo";
const MODELICA_PARSER_SUMMARY_READY_SOURCE: &str = r"
within Warmup;
package Warmup
  import Modelica.Units.SI;

  model Probe
    parameter SI.Time tau = 1;
  end Probe;
end Warmup;
";
const MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS: u64 = 60;

/// Spawns the official `WendaoArrow` stream-scoring Flight example service.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaoarrow_stream_scoring_service() -> (String, JuliaExampleServiceGuard) {
    spawn_script_service(
        wendaoarrow_script("run_stream_scoring_flight_server.sh"),
        "spawn real WendaoArrow service",
    )
    .await
}

/// Spawns the official `WendaoArrow` stream-metadata Flight example service.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaoarrow_stream_metadata_service() -> (String, JuliaExampleServiceGuard) {
    spawn_script_service(
        wendaoarrow_script("run_stream_metadata_flight_server.sh"),
        "spawn real WendaoArrow metadata service",
    )
    .await
}

/// Spawns the official `WendaoAnalyzer` linear-blend example service.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaoanalyzer_stream_linear_blend_service() -> (String, JuliaExampleServiceGuard)
{
    spawn_wendaoanalyzer_example_service(
        &[
            "--service-mode",
            "stream",
            "--analyzer-strategy",
            "linear_blend",
        ],
        "spawn real WendaoAnalyzer linear blend service",
    )
    .await
}

/// Spawns the official `WendaoSearch` structural-rerank example in `demo`
/// mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_demo_structural_rerank_service()
-> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_service("structural_rerank", "demo").await
}

/// Spawns the official `WendaoSearch` structural-rerank example in
/// `solver_demo` mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_solver_demo_structural_rerank_service()
-> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_service("structural_rerank", "solver_demo").await
}

/// Spawns the official same-port multi-route `WendaoSearch` example in `demo`
/// mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_demo_multi_route_service() -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_multi_route_service("demo").await
}

/// Spawns the official same-port multi-route `WendaoSearch` example in
/// `solver_demo` mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_solver_demo_multi_route_service()
-> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_multi_route_service("solver_demo").await
}

/// Spawns the official `WendaoSearch` parser-summary service with the native
/// summary routes mounted on the shared Flight endpoint.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_julia_parser_summary_service() -> (String, JuliaExampleServiceGuard)
{
    spawn_wendaosearch_julia_parser_summary_service_with_attempts(1500).await
}

/// Spawns the official `WendaoSearch` parser-summary service with one explicit
/// readiness
/// attempt budget.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_julia_parser_summary_service_with_attempts(
    ready_attempts: usize,
) -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_parser_summary_service(ready_attempts).await
}

/// Spawns the official `WendaoSearch` parser-summary service for the Modelica
/// summary route.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_modelica_parser_summary_service()
-> (String, JuliaExampleServiceGuard) {
    let (base_url, mut guard) = spawn_wendaosearch_parser_summary_service(1500).await;
    probe_wendaosearch_modelica_parser_summary_route_for_tests(base_url.as_str()).unwrap_or_else(
        |error| {
            guard.kill();
            panic!("wait for WendaoSearch Modelica parser-summary route readiness: {error}");
        },
    );
    (base_url, guard)
}

/// Probe the `WendaoSearch` Modelica parser-summary service on one explicit base
/// URL using the same file-summary plus ast-query warmup fixture as the linked
/// Julia integration helpers.
///
/// # Errors
///
/// Returns an error string when the fixed service does not accept the warmup
/// Modelica file-summary and ast-query requests on the configured Flight
/// endpoint.
pub fn probe_wendaosearch_modelica_parser_summary_route_for_tests(
    base_url: &str,
) -> Result<(), String> {
    wait_for_modelica_parser_summary_route_ready(base_url)
}

/// Materializes a Julia deployment artifact from runtime-config values.
#[must_use]
pub fn wendaoanalyzer_deployment_artifact_from_runtime(
    runtime: &LinkGraphJuliaRerankRuntimeConfig,
) -> LinkGraphJuliaDeploymentArtifact {
    runtime.deployment_artifact()
}

/// Spawns a `WendaoAnalyzer` service from an explicit Julia launch manifest.
///
/// # Panics
///
/// Panics when the launcher path cannot be resolved, the child process cannot
/// be spawned, or the service never becomes ready.
pub async fn spawn_wendaoanalyzer_service_from_manifest(
    manifest: &LinkGraphJuliaAnalyzerLaunchManifest,
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut command = if manifest.launcher_path == DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH {
        let mut command = Command::new("julia");
        command.arg(wendaoanalyzer_script("run_analyzer_example.jl"));
        for argument in &manifest.args {
            command.arg(argument);
        }
        command
    } else {
        let script = repo_root().join(&manifest.launcher_path);
        let mut command = Command::new("bash");
        command.arg(script);
        for argument in &manifest.args {
            command.arg(argument);
        }
        command
    };
    command.arg("--port").arg(port.to_string());

    let child = command
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("spawn WendaoAnalyzer service: {error}"));
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready(base_url.as_str())
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoAnalyzer service readiness: {error}");
        });

    (base_url, guard)
}

/// Spawns a `WendaoAnalyzer` service from a rendered deployment artifact.
///
/// # Panics
///
/// Panics when the deployment artifact launcher cannot be spawned or the
/// service never becomes ready.
pub async fn spawn_wendaoanalyzer_service_from_artifact(
    artifact: &LinkGraphJuliaDeploymentArtifact,
) -> (String, JuliaExampleServiceGuard) {
    spawn_wendaoanalyzer_service_from_manifest(&artifact.launch).await
}

async fn spawn_wendaoanalyzer_example_service(
    args: &[&str],
    error_context: &str,
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let child = Command::new("julia")
        .arg(wendaoanalyzer_script("run_analyzer_example.jl"))
        .args(args)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("{error_context}: {error}"));
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready(base_url.as_str())
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoAnalyzer service readiness: {error}");
        });

    (base_url, guard)
}

async fn spawn_script_service(
    script: PathBuf,
    error_context: &str,
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let child = Command::new("bash")
        .arg(script)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("{error_context}: {error}"));
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready(base_url.as_str())
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for Julia official example service readiness: {error}");
        });

    (base_url, guard)
}

fn project_environment_is_ready() -> bool {
    [
        "PRJ_ROOT",
        "PRJ_CACHE_HOME",
        "PRJ_DATA_HOME",
        "PRJ_RUNTIME_DIR",
    ]
    .into_iter()
    .all(|name| env::var_os(name).is_some())
}

fn project_julia_command() -> Command {
    if project_environment_is_ready() {
        Command::new("julia")
    } else {
        let mut command = Command::new("direnv");
        command.arg("exec").arg(".").arg("julia");
        command
    }
}

async fn spawn_wendaosearch_service(
    route_name: &str,
    mode: &str,
) -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_service_with_code_parser_routes(route_name, mode, &[], 600).await
}

async fn spawn_wendaosearch_parser_summary_service(
    ready_attempts: usize,
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let contract = wendaosearch_parser_summary_contract();
    let base_url = format!("http://{}:{port}", contract.service.host);
    let script = contract.script_path();
    let child = project_julia_command()
        .arg(format!(
            "--project={}",
            wendaosearch_package_dir().display()
        ))
        .arg(script)
        .arg("--host")
        .arg(&contract.service.host)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("spawn real WendaoSearch parser-summary service: {error}"));
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), ready_attempts)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch parser-summary service readiness: {error}");
        });

    (base_url, guard)
}

fn wait_for_modelica_parser_summary_route_ready(base_url: &str) -> Result<(), String> {
    let repository = modelica_parser_summary_ready_repository(base_url);
    let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
        &repository,
        MODELICA_PARSER_SUMMARY_READY_SOURCE_ID,
        MODELICA_PARSER_SUMMARY_READY_SOURCE,
    )
    .map_err(|error| {
        format!("Modelica file-summary readiness probe failed for `{base_url}`: {error}")
    })?;
    if summary.class_name.as_deref() != Some("Warmup") {
        return Err(format!(
            "Modelica file-summary readiness probe returned unexpected class_name {:?} for `{base_url}`",
            summary.class_name
        ));
    }

    let analysis = fetch_modelica_ast_query_analysis_blocking_for_repository(
        &repository,
        MODELICA_PARSER_SUMMARY_READY_SOURCE_ID,
        MODELICA_PARSER_SUMMARY_READY_SOURCE,
    )
    .map_err(|error| {
        format!("Modelica ast-query readiness probe failed for `{base_url}`: {error}")
    })?;
    if !analysis
        .modules
        .iter()
        .any(|module| module.qualified_name == "Warmup")
    {
        return Err(format!(
            "Modelica ast-query readiness probe returned no Warmup module for `{base_url}`"
        ));
    }

    Ok(())
}

fn modelica_parser_summary_ready_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "linked-modelica-ready".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: json!({
                "parser_summary_transport": {
                    "base_url": base_url,
                    "file_summary": {
                        "timeout_secs": MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS,
                    },
                    "ast_query": {
                        "timeout_secs": MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS,
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    }
}

async fn spawn_wendaosearch_service_with_code_parser_routes(
    route_name: &str,
    mode: &str,
    code_parser_route_names: &[&str],
    ready_attempts: usize,
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let script = wendaosearch_script("run_search_service.jl");
    let mut command = project_julia_command();
    command
        .arg(format!(
            "--project={}",
            wendaosearch_package_dir().display()
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
        .stderr(Stdio::null());
    if !code_parser_route_names.is_empty() {
        command
            .arg("--code-parser-route-names")
            .arg(code_parser_route_names.join(","));
    }
    let child = command.spawn().unwrap_or_else(|error| {
        panic!("spawn real WendaoSearch `{route_name}` `{mode}` service: {error}")
    });
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), ready_attempts)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch service readiness: {error}");
        });

    (base_url, guard)
}

async fn spawn_wendaosearch_multi_route_service(mode: &str) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let script = wendaosearch_script("run_search_service.jl");
    let child = project_julia_command()
        .arg(format!(
            "--project={}",
            wendaosearch_package_dir().display()
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
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), 600)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch multi-route service readiness: {error}");
        });

    (base_url, guard)
}
