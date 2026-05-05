//! Process-managed `WendaoSearch` example services used by integration tests.

use std::env;
use std::process::{Command, Stdio};

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use super::service_runtime::{
    JuliaExampleServiceGuard, repo_root, reserve_service_port,
    wait_for_service_ready_with_attempts, wendaocodeparser_julia_project,
    wendaosearch_julia_project, wendaosearch_parser_summary_contract, wendaosearch_script,
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
const WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV: &str = "WENDAOSEARCH_SOLVER_DEMO_BASE_URL";
const JULIA_PARSER_SUMMARY_ROUTE_NAMES: &[&str] = &["julia_file_summary", "julia_root_summary"];
const MODELICA_PARSER_SUMMARY_ROUTE_NAMES: &[&str] =
    &["modelica_file_summary", "modelica_ast_query"];
const ALL_PARSER_SUMMARY_ROUTE_NAMES: &[&str] = &[
    "julia_file_summary",
    "julia_root_summary",
    "modelica_file_summary",
    "modelica_ast_query",
];

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
    if let Some(base_url) = configured_solver_demo_base_url() {
        wait_for_external_solver_demo_service(base_url.as_str()).await;
        return (base_url, JuliaExampleServiceGuard::external());
    }
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
    if let Some(base_url) = configured_solver_demo_base_url() {
        wait_for_external_solver_demo_service(base_url.as_str()).await;
        return (base_url, JuliaExampleServiceGuard::external());
    }
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
    spawn_wendaosearch_parser_summary_service(ready_attempts, JULIA_PARSER_SUMMARY_ROUTE_NAMES)
        .await
}

/// Spawns the official `WendaoSearch` parser-summary service with all native
/// Julia and Modelica summary routes mounted on the shared Flight endpoint.
///
/// # Panics
///
/// Panics when the service script cannot be resolved, the service fails to
/// start, or the Modelica parser-summary route readiness probe fails.
pub async fn spawn_wendaosearch_all_parser_summary_service() -> (String, JuliaExampleServiceGuard) {
    let (base_url, mut guard) =
        spawn_wendaosearch_parser_summary_service(1500, ALL_PARSER_SUMMARY_ROUTE_NAMES).await;
    probe_wendaosearch_modelica_parser_summary_route_for_tests(base_url.as_str()).unwrap_or_else(
        |error| {
            guard.kill();
            panic!("wait for WendaoSearch all-routes parser-summary readiness: {error}");
        },
    );
    (base_url, guard)
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
    let (base_url, mut guard) =
        spawn_wendaosearch_parser_summary_service(1500, MODELICA_PARSER_SUMMARY_ROUTE_NAMES).await;
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

fn project_julia_command() -> Command {
    if executable_on_path("direnv") {
        let mut command = Command::new("direnv");
        command.arg("exec").arg(".").arg("julia");
        return command;
    }
    Command::new("julia")
}

fn executable_on_path(name: &str) -> bool {
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|path| path.join(name).is_file()))
}

fn configured_solver_demo_base_url() -> Option<String> {
    env::var(WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV)
        .ok()
        .filter(|value| !value.is_empty())
}

async fn wait_for_external_solver_demo_service(base_url: &str) {
    wait_for_service_ready_with_attempts(base_url, 600)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "wait for externally managed WendaoSearch solver-demo service readiness: {error}"
            )
        });
}

async fn spawn_wendaosearch_service(
    route_name: &str,
    mode: &str,
) -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_service_with_code_parser_routes(route_name, mode, &[], 600).await
}

async fn spawn_wendaosearch_parser_summary_service(
    ready_attempts: usize,
    code_parser_route_names: &[&str],
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let contract = wendaosearch_parser_summary_contract();
    let base_url = format!("http://{}:{port}", contract.service.host);
    let script = contract.script_path();
    let child = project_julia_command()
        .arg(format!(
            "--project={}",
            wendaocodeparser_julia_project().display()
        ))
        .arg(script)
        .arg("--host")
        .arg(&contract.service.host)
        .arg("--port")
        .arg(port.to_string())
        .arg("--code-parser-route-names")
        .arg(code_parser_route_names.join(","))
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .env(
            "JULIA_NUM_THREADS",
            env::var("WENDAOSEARCH_JULIA_NUM_THREADS").unwrap_or_else(|_| "8".to_string()),
        )
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
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
        MODELICA_PARSER_SUMMARY_READY_SOURCE_ID.into(),
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
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
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
