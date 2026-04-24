use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

use crate::compatibility::link_graph::{
    DEFAULT_JULIA_ANALYZER_PACKAGE_DIR, DEFAULT_JULIA_ARROW_PACKAGE_DIR,
};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio::time::sleep;

/// Guard for a spawned Julia integration-support service process.
pub struct JuliaExampleServiceGuard {
    child: Child,
}

impl JuliaExampleServiceGuard {
    pub(crate) fn new(child: Child) -> Self {
        Self { child }
    }

    /// Terminates the spawned service if it is still running.
    ///
    /// # Panics
    ///
    /// Panics when polling or terminating the child process fails.
    pub fn kill(&mut self) {
        if let Some(_status) = self
            .child
            .try_wait()
            .unwrap_or_else(|error| panic!("poll Julia example child: {error}"))
        {
            return;
        }
        self.child
            .kill()
            .unwrap_or_else(|error| panic!("kill Julia example child: {error}"));
        let _ = self.child.wait();
    }
}

impl Drop for JuliaExampleServiceGuard {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

pub(crate) fn reserve_service_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map_or_else(
            |error| panic!("reserve Julia example service port: {error}"),
            |address| address.port(),
        )
}

pub(crate) fn repo_root() -> PathBuf {
    if let Ok(project_root) = env::var("PRJ_ROOT") {
        let candidate = PathBuf::from(project_root);
        if repo_root_candidate_is_valid(candidate.as_path()) {
            return candidate;
        }
    }

    match Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4) {
        Some(path) if repo_root_candidate_is_valid(path) => path.to_path_buf(),
        Some(path) => panic!(
            "resolved repo root candidate `{}` failed marker checks",
            path.display()
        ),
        None => panic!("resolve repo root"),
    }
}

fn repo_root_candidate_is_valid(candidate: &Path) -> bool {
    candidate.join("Cargo.lock").is_file()
        && candidate
            .join("packages/rust/crates/xiuxian-wendao-julia/Cargo.toml")
            .is_file()
}

pub(crate) fn project_cache_dir() -> PathBuf {
    let configured = env::var_os("PRJ_CACHE_HOME").unwrap_or_else(|| {
        panic!("PRJ_CACHE_HOME must be set; run Julia integration support via `direnv exec . ...`")
    });
    let configured = PathBuf::from(configured);
    if configured.is_absolute() {
        configured
    } else {
        panic!(
            "PRJ_CACHE_HOME must be absolute in the project environment, got `{}`",
            configured.display()
        )
    }
}

fn resolve_linked_package_dir(relative_path: &str, label: &str) -> Option<PathBuf> {
    let candidate = repo_root().join(relative_path);
    if !candidate.is_dir() {
        return None;
    }
    Some(
        candidate
            .canonicalize()
            .unwrap_or_else(|error| panic!("resolve {label} package dir: {error}")),
    )
}

fn resolve_project_package_dir(package_name: &str) -> Option<PathBuf> {
    let project = env::var_os("WENDAOSEARCH_JULIA_PROJECT")?;
    let project = PathBuf::from(project);
    let project = if project.is_absolute() {
        project
    } else {
        repo_root().join(project)
    };
    let output = Command::new("julia")
        .arg(format!("--project={}", project.display()))
        .arg("-e")
        .arg(format!(
            "using {package_name}; print(dirname(dirname(pathof({package_name}))))"
        ))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let resolved = String::from_utf8(output.stdout).ok()?;
    let resolved = resolved.trim();
    if resolved.is_empty() {
        return None;
    }
    PathBuf::from(resolved).canonicalize().ok()
}

pub(crate) fn julia_example_project_for_package(package_dir: &Path) -> PathBuf {
    if package_dir.starts_with(repo_root().join(".data")) {
        return package_dir.to_path_buf();
    }
    wendaosearch_julia_project()
}

pub(crate) fn wendaoarrow_package_dir() -> PathBuf {
    resolve_linked_package_dir(DEFAULT_JULIA_ARROW_PACKAGE_DIR, "WendaoArrow")
        .or_else(|| resolve_project_package_dir("WendaoArrow"))
        .unwrap_or_else(|| panic!("resolve WendaoArrow package dir"))
}

pub(crate) fn wendaoarrow_script(name: &str) -> PathBuf {
    wendaoarrow_package_dir()
        .join("scripts")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoArrow script `{name}`: {error}"))
}

pub(crate) fn wendaoanalyzer_package_dir() -> PathBuf {
    resolve_linked_package_dir(DEFAULT_JULIA_ANALYZER_PACKAGE_DIR, "WendaoAnalyzer")
        .or_else(|| resolve_project_package_dir("WendaoAnalyzer"))
        .unwrap_or_else(|| panic!("resolve WendaoAnalyzer package dir"))
}

pub(crate) fn wendaoanalyzer_script(name: &str) -> PathBuf {
    wendaoanalyzer_package_dir()
        .join("scripts")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoAnalyzer script `{name}`: {error}"))
}

pub(crate) fn wendaosearch_package_dir() -> PathBuf {
    repo_root()
        .join(".data/WendaoSearch.jl")
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch package dir: {error}"))
}

pub(crate) fn wendaosearch_julia_project() -> PathBuf {
    let Some(configured) = env::var_os("WENDAOSEARCH_JULIA_PROJECT") else {
        return wendaosearch_package_dir();
    };
    let candidate = PathBuf::from(configured);
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        repo_root().join(candidate)
    };
    candidate
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch Julia project dir: {error}"))
}

#[cfg(test)]
pub(crate) fn wendaosearch_config(name: &str) -> PathBuf {
    wendaosearch_package_dir()
        .join("config")
        .join("live")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch config `{name}`: {error}"))
}

pub(crate) fn wendaosearch_script(name: &str) -> PathBuf {
    wendaosearch_package_dir()
        .join("scripts")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch script `{name}`: {error}"))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct WendaoSearchParserSummaryServiceContract {
    pub(crate) script: String,
    pub(crate) config: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) default_code_parser_route_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct WendaoSearchModelicaTransportContract {
    pub(crate) schema_version: String,
    pub(crate) file_summary_route_name: String,
    pub(crate) ast_query_route_name: String,
    pub(crate) file_summary_path: String,
    pub(crate) ast_query_path: String,
    pub(crate) readiness_route_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct WendaoSearchParserSummaryContract {
    pub(crate) contract_version: u32,
    pub(crate) service: WendaoSearchParserSummaryServiceContract,
    pub(crate) modelica_transport: WendaoSearchModelicaTransportContract,
}

impl WendaoSearchParserSummaryContract {
    pub(crate) fn script_path(&self) -> PathBuf {
        repo_root()
            .join(&self.service.script)
            .canonicalize()
            .unwrap_or_else(|error| {
                panic!(
                    "resolve WendaoSearch parser-summary contract script `{}`: {error}",
                    self.service.script
                )
            })
    }

    #[cfg(test)]
    pub(crate) fn config_path(&self) -> PathBuf {
        repo_root()
            .join(&self.service.config)
            .canonicalize()
            .unwrap_or_else(|error| {
                panic!(
                    "resolve WendaoSearch parser-summary contract config `{}`: {error}",
                    self.service.config
                )
            })
    }

    #[cfg(test)]
    pub(crate) fn base_url(&self) -> String {
        format!("http://{}:{}", self.service.host, self.service.port)
    }
}

pub(crate) fn wendaosearch_parser_summary_contract_path() -> PathBuf {
    repo_root()
        .join(
            "packages/rust/crates/xiuxian-wendao-julia/contracts/wendaosearch_parser_summary.toml",
        )
        .canonicalize()
        .unwrap_or_else(|error| {
            panic!("resolve WendaoSearch parser-summary contract path: {error}")
        })
}

pub(crate) fn wendaosearch_parser_summary_contract() -> WendaoSearchParserSummaryContract {
    let contract_path = wendaosearch_parser_summary_contract_path();
    let contract_text = fs::read_to_string(&contract_path).unwrap_or_else(|error| {
        panic!(
            "read WendaoSearch parser-summary contract `{}`: {error}",
            contract_path.display()
        )
    });
    toml::from_str(&contract_text).unwrap_or_else(|error| {
        panic!(
            "parse WendaoSearch parser-summary contract `{}`: {error}",
            contract_path.display()
        )
    })
}

#[cfg(test)]
pub(crate) fn expected_wendaosearch_modelica_transport_contract()
-> WendaoSearchModelicaTransportContract {
    use crate::modelica_plugin::{
        MODELICA_AST_QUERY_ROUTE, MODELICA_FILE_SUMMARY_ROUTE,
        MODELICA_PARSER_SUMMARY_SCHEMA_VERSION,
    };

    WendaoSearchModelicaTransportContract {
        schema_version: MODELICA_PARSER_SUMMARY_SCHEMA_VERSION.to_string(),
        file_summary_route_name: "modelica_file_summary".to_string(),
        ast_query_route_name: "modelica_ast_query".to_string(),
        file_summary_path: MODELICA_FILE_SUMMARY_ROUTE.to_string(),
        ast_query_path: MODELICA_AST_QUERY_ROUTE.to_string(),
        readiness_route_names: vec![
            "modelica_file_summary".to_string(),
            "modelica_ast_query".to_string(),
        ],
    }
}

pub(crate) async fn wait_for_service_ready(base_url: &str) -> Result<(), String> {
    wait_for_service_ready_with_attempts(base_url, 450).await
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

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaosearch_contract.rs"]
mod wendaosearch_contract;
