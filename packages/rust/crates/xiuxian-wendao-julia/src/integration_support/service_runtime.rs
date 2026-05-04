//! Shared process and path helpers for Julia integration-support services.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio::time::sleep;

const WENDAOSEARCH_WORKSPACE_PREFIX: &str = ".data/WendaoSearch.jl/";
const WENDAO_CODE_PARSER_WORKSPACE_PREFIX: &str = ".data/WendaoCodeParser.jl/";

/// Guard for a spawned Julia integration-support service process.
pub struct JuliaExampleServiceGuard {
    child: Option<Child>,
}

impl JuliaExampleServiceGuard {
    pub(crate) fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    pub(crate) fn external() -> Self {
        Self { child: None }
    }

    /// Terminates the spawned service if it is still running.
    ///
    /// # Panics
    ///
    /// Panics when polling or terminating the child process fails.
    pub fn kill(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if let Some(_status) = child
            .try_wait()
            .unwrap_or_else(|error| panic!("poll Julia example child: {error}"))
        {
            return;
        }
        child
            .kill()
            .unwrap_or_else(|error| panic!("kill Julia example child: {error}"));
        let _ = child.wait();
    }
}

impl Drop for JuliaExampleServiceGuard {
    fn drop(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if let Ok(None) = child.try_wait() {
            let _ = child.kill();
            let _ = child.wait();
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

pub(crate) fn wendaosearch_package_dir() -> PathBuf {
    if let Some(configured) = env::var_os("WENDAOSEARCH_PACKAGE_DIR") {
        return resolve_existing_path("WendaoSearch package dir", configured);
    }

    repo_root()
        .join(".data/WendaoSearch.jl")
        .canonicalize()
        .unwrap_or_else(|error| {
            panic!(
                "resolve WendaoSearch package dir: {error}; set WENDAOSEARCH_PACKAGE_DIR when WendaoSearch is installed by Julia Pkg"
            )
        })
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

pub(crate) fn wendaocodeparser_package_dir() -> PathBuf {
    if let Some(configured) = env::var_os("WENDAO_CODE_PARSER_PACKAGE_DIR") {
        return resolve_existing_path("WendaoCodeParser package dir", configured);
    }

    let candidate = repo_root().join(".data/WendaoCodeParser.jl");
    if candidate.is_dir() {
        return candidate
            .canonicalize()
            .unwrap_or_else(|error| panic!("resolve WendaoCodeParser package dir: {error}"));
    }

    wendaosearch_package_dir()
}

pub(crate) fn wendaocodeparser_julia_project() -> PathBuf {
    if let Some(configured) = env::var_os("WENDAO_CODE_PARSER_JULIA_PROJECT") {
        return resolve_existing_path("WendaoCodeParser Julia project dir", configured);
    }

    if env::var_os("WENDAO_CODE_PARSER_PACKAGE_DIR").is_some()
        || repo_root().join(".data/WendaoCodeParser.jl").is_dir()
    {
        return wendaocodeparser_package_dir();
    }

    wendaosearch_julia_project()
}

fn resolve_existing_path(label: &str, configured: impl Into<PathBuf>) -> PathBuf {
    let candidate = configured.into();
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        repo_root().join(candidate)
    };
    candidate
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve {label} `{}`: {error}", candidate.display()))
}

pub(crate) fn wendaosearch_config(name: &str) -> PathBuf {
    wendaosearch_package_dir()
        .join("config")
        .join("live")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch config `{name}`: {error}"))
}

pub(crate) fn wendaocodeparser_config(name: &str) -> PathBuf {
    let candidate = wendaocodeparser_package_dir()
        .join("config")
        .join("live")
        .join(name);
    if candidate.is_file() {
        return candidate
            .canonicalize()
            .unwrap_or_else(|error| panic!("resolve WendaoCodeParser config `{name}`: {error}"));
    }
    wendaosearch_config(name)
}

pub(crate) fn wendaosearch_script(name: &str) -> PathBuf {
    wendaosearch_package_dir()
        .join("scripts")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch script `{name}`: {error}"))
}

pub(crate) fn wendaocodeparser_script(name: &str) -> PathBuf {
    let candidate = wendaocodeparser_package_dir().join("scripts").join(name);
    if candidate.is_file() {
        return candidate
            .canonicalize()
            .unwrap_or_else(|error| panic!("resolve WendaoCodeParser script `{name}`: {error}"));
    }
    if name == "run_service.jl" {
        return wendaosearch_script("run_parser_summary_service.jl");
    }
    candidate
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoCodeParser script `{name}`: {error}"))
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
        resolve_wendaosearch_contract_path(&self.service.script, "script")
    }

    #[cfg(test)]
    pub(crate) fn config_path(&self) -> PathBuf {
        resolve_wendaosearch_contract_path(&self.service.config, "config")
    }

    #[cfg(test)]
    pub(crate) fn base_url(&self) -> String {
        format!("http://{}:{}", self.service.host, self.service.port)
    }
}

fn resolve_wendaosearch_contract_path(configured: &str, label: &str) -> PathBuf {
    if let Some(package_relative) = configured.strip_prefix(WENDAO_CODE_PARSER_WORKSPACE_PREFIX) {
        if label == "script" && package_relative == "scripts/run_service.jl" {
            return wendaocodeparser_script("run_service.jl");
        }
        if label == "config" && package_relative == "config/live/parser_summary.toml" {
            return wendaocodeparser_config("parser_summary.toml");
        }
        return wendaocodeparser_package_dir()
            .join(package_relative)
            .canonicalize()
            .unwrap_or_else(|error| {
                panic!("resolve WendaoCodeParser contract {label} `{configured}`: {error}")
            });
    }
    if let Some(package_relative) = configured.strip_prefix(WENDAOSEARCH_WORKSPACE_PREFIX) {
        return wendaosearch_package_dir()
            .join(package_relative)
            .canonicalize()
            .unwrap_or_else(|error| {
                panic!("resolve WendaoSearch contract {label} `{configured}`: {error}")
            });
    }

    repo_root()
        .join(configured)
        .canonicalize()
        .unwrap_or_else(|error| {
            panic!("resolve WendaoSearch parser-summary contract {label} `{configured}`: {error}")
        })
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
