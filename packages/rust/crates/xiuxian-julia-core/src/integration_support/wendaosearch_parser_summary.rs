//! Parser-summary service helpers for managed `WendaoSearch` services.

use std::env;
use std::process::Stdio;
#[cfg(test)]
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use crate::integration_support::service_runtime::{
    JuliaServiceGuard, repo_root, reserve_service_port, wait_for_service_ready_with_attempts,
    wendaocodeparser_julia_project, wendaosearch_parser_summary_contract,
};
use crate::{
    fetch_modelica_ast_query_analysis_blocking_for_repository,
    fetch_modelica_parser_file_summary_blocking_for_repository,
};

use super::project_julia_command;

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
const MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS: u64 = 4;
const MODELICA_PARSER_SUMMARY_READY_ROUTE_RETRIES: usize = 6;
const MODELICA_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS: u64 = 250;
const WENDAOSEARCH_PARSER_SUMMARY_READY_TIMEOUT_SECS_ENV: &str =
    "WENDAOSEARCH_PARSER_SUMMARY_READY_TIMEOUT_SECS";
const WENDAOSEARCH_PARSER_SUMMARY_READY_ROUTE_RETRIES_ENV: &str =
    "WENDAOSEARCH_PARSER_SUMMARY_READY_ROUTE_RETRIES";
const WENDAOSEARCH_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS_ENV: &str =
    "WENDAOSEARCH_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS";
const JULIA_PARSER_SUMMARY_ROUTE_NAMES: &[&str] = &["julia_file_summary", "julia_root_summary"];
const MODELICA_PARSER_SUMMARY_ROUTE_NAMES: &[&str] =
    &["modelica_file_summary", "modelica_ast_query"];
const ALL_PARSER_SUMMARY_ROUTE_NAMES: &[&str] = &[
    "julia_file_summary",
    "julia_root_summary",
    "modelica_file_summary",
    "modelica_ast_query",
];
const DEFAULT_PARSER_SUMMARY_WENDAOSEARCH_READY_ATTEMPTS: usize = 300;
const WENDAOSEARCH_PARSER_SUMMARY_READY_ATTEMPTS_ENV: &str =
    "WENDAOSEARCH_PARSER_SUMMARY_READY_ATTEMPTS";
#[cfg(test)]
static PARSER_SUMMARY_TEST_ENVS: OnceLock<Mutex<HashMap<&'static str, Option<String>>>> =
    OnceLock::new();

#[cfg(test)]
fn parser_summary_test_envs() -> &'static Mutex<HashMap<&'static str, Option<String>>> {
    PARSER_SUMMARY_TEST_ENVS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
pub(crate) fn set_parser_summary_env_for_tests(name: &str, value: Option<&str>) {
    let mut envs = parser_summary_test_envs()
        .lock()
        .unwrap_or_else(|error| panic!("parser summary test env lock failed: {error}"));
    let key: &'static str = match name {
        WENDAOSEARCH_PARSER_SUMMARY_READY_TIMEOUT_SECS_ENV => {
            WENDAOSEARCH_PARSER_SUMMARY_READY_TIMEOUT_SECS_ENV
        }
        WENDAOSEARCH_PARSER_SUMMARY_READY_ROUTE_RETRIES_ENV => {
            WENDAOSEARCH_PARSER_SUMMARY_READY_ROUTE_RETRIES_ENV
        }
        WENDAOSEARCH_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS_ENV => {
            WENDAOSEARCH_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS_ENV
        }
        WENDAOSEARCH_PARSER_SUMMARY_READY_ATTEMPTS_ENV => {
            WENDAOSEARCH_PARSER_SUMMARY_READY_ATTEMPTS_ENV
        }
        _ => panic!("unknown parser summary env key in test override: {name}"),
    };
    envs.insert(
        key,
        value
            .map(std::string::ToString::to_string)
            .filter(|value| !value.is_empty()),
    );
}

fn parser_summary_env_value(name: &str) -> Option<String> {
    #[cfg(test)]
    {
        if let Some(override_value) = parser_summary_test_envs()
            .lock()
            .ok()
            .and_then(|envs| envs.get(name).cloned().flatten())
        {
            return Some(override_value);
        }
    }

    env::var(name).ok()
}

/// Spawns the managed `WendaoSearch` parser-summary service with the native
/// summary routes mounted on the shared Flight endpoint.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_julia_parser_summary_service() -> (String, JuliaServiceGuard) {
    spawn_wendaosearch_julia_parser_summary_service_with_attempts(parser_summary_ready_attempts())
        .await
}

/// Spawns the managed `WendaoSearch` parser-summary service with one explicit
/// readiness attempt budget.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_julia_parser_summary_service_with_attempts(
    ready_attempts: usize,
) -> (String, JuliaServiceGuard) {
    spawn_wendaosearch_parser_summary_service(ready_attempts, JULIA_PARSER_SUMMARY_ROUTE_NAMES)
        .await
}

/// Spawns the managed `WendaoSearch` parser-summary service with all native
/// Julia and Modelica summary routes mounted on the shared Flight endpoint.
///
/// # Panics
///
/// Panics when the service script cannot be resolved, the service fails to
/// start, or the Modelica parser-summary route readiness probe fails.
pub async fn spawn_wendaosearch_all_parser_summary_service() -> (String, JuliaServiceGuard) {
    let (base_url, mut guard) = spawn_wendaosearch_parser_summary_service(
        parser_summary_ready_attempts(),
        ALL_PARSER_SUMMARY_ROUTE_NAMES,
    )
    .await;
    probe_wendaosearch_modelica_parser_summary_route_for_tests(base_url.as_str()).unwrap_or_else(
        |error| {
            guard.kill();
            panic!("wait for WendaoSearch all-routes parser-summary readiness: {error}");
        },
    );
    (base_url, guard)
}

/// Spawns the managed `WendaoSearch` parser-summary service for the Modelica
/// summary route.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_modelica_parser_summary_service() -> (String, JuliaServiceGuard) {
    let (base_url, mut guard) = spawn_wendaosearch_parser_summary_service(
        parser_summary_ready_attempts(),
        MODELICA_PARSER_SUMMARY_ROUTE_NAMES,
    )
    .await;
    probe_wendaosearch_modelica_parser_summary_route_for_tests(base_url.as_str()).unwrap_or_else(
        |error| {
            guard.kill();
            panic!("wait for WendaoSearch Modelica parser-summary route readiness: {error}");
        },
    );
    (base_url, guard)
}

/// Probe the `WendaoSearch` Modelica parser-summary service on one explicit
/// base URL using the same file-summary plus ast-query warmup fixture as the
/// linked Julia integration helpers.
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

fn parser_summary_ready_attempts() -> usize {
    parser_summary_env_value(WENDAOSEARCH_PARSER_SUMMARY_READY_ATTEMPTS_ENV)
        .and_then(|value: String| value.parse::<usize>().ok())
        .filter(|attempts| *attempts > 0)
        .unwrap_or(DEFAULT_PARSER_SUMMARY_WENDAOSEARCH_READY_ATTEMPTS)
}

async fn spawn_wendaosearch_parser_summary_service(
    ready_attempts: usize,
    code_parser_route_names: &[&str],
) -> (String, JuliaServiceGuard) {
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
    let mut guard = JuliaServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), ready_attempts)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch parser-summary service readiness: {error}");
        });

    (base_url, guard)
}

fn wait_for_modelica_parser_summary_route_ready(base_url: &str) -> Result<(), String> {
    let mut last_error = "route readiness probe not started".to_string();
    for _ in 0..parser_summary_ready_route_retries() {
        let repository = modelica_parser_summary_ready_repository(base_url);
        match check_modelica_parser_summary_route_ready_once(base_url, &repository) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = error;
                std::thread::sleep(std::time::Duration::from_millis(
                    parser_summary_ready_retry_delay_millis(),
                ));
            }
        }
    }
    Err(format!(
        "Modelica parser-summary route was not ready for `{base_url}` after bounded retries: {last_error}"
    ))
}

fn parser_summary_ready_route_retries() -> usize {
    parser_summary_env_value(WENDAOSEARCH_PARSER_SUMMARY_READY_ROUTE_RETRIES_ENV)
        .and_then(|value: String| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(MODELICA_PARSER_SUMMARY_READY_ROUTE_RETRIES)
}

fn parser_summary_ready_retry_delay_millis() -> u64 {
    parser_summary_env_value(WENDAOSEARCH_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS_ENV)
        .and_then(|value: String| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(MODELICA_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS)
}

fn parser_summary_ready_timeout_secs() -> u64 {
    parser_summary_env_value(WENDAOSEARCH_PARSER_SUMMARY_READY_TIMEOUT_SECS_ENV)
        .and_then(|value: String| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS)
}

fn check_modelica_parser_summary_route_ready_once(
    base_url: &str,
    repository: &RegisteredRepository,
) -> Result<(), String> {
    let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
        repository,
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
        repository,
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
                        "timeout_secs": parser_summary_ready_timeout_secs(),
                    },
                    "ast_query": {
                        "timeout_secs": parser_summary_ready_timeout_secs(),
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaosearch_parser_summary.rs"]
mod tests;
