use std::fmt::Display;
use std::io::Error as IoError;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use crate::integration_support::{
    JuliaExampleServiceGuard, probe_wendaosearch_modelica_parser_summary_route_for_tests,
    spawn_wendaosearch_julia_parser_summary_service,
    spawn_wendaosearch_modelica_parser_summary_service,
};
use crate::plugin::parser_summary::{
    fetch_julia_parser_file_summary_blocking_for_repository,
    fetch_julia_parser_root_summary_blocking_for_repository,
};
use crate::{
    set_linked_julia_parser_summary_base_url_for_tests,
    set_linked_modelica_parser_summary_base_url_for_tests,
};

pub(crate) struct ChildGuard {
    child: Option<Child>,
}

struct LinkedJuliaParserSummaryService {
    _guard: Option<Mutex<JuliaExampleServiceGuard>>,
}

struct LinkedModelicaParserSummaryService {
    _guard: Option<Mutex<JuliaExampleServiceGuard>>,
}

static LINKED_JULIA_PARSER_SUMMARY_SERVICE: OnceLock<
    Result<LinkedJuliaParserSummaryService, String>,
> = OnceLock::new();
static LINKED_MODELICA_PARSER_SUMMARY_SERVICE: OnceLock<
    Result<LinkedModelicaParserSummaryService, String>,
> = OnceLock::new();

const SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL: &str = "http://127.0.0.1:41081";

pub(crate) trait ResultTestExt<T, E> {
    fn or_panic(self, context: &str) -> T;
    fn err_or_panic(self, context: &str) -> E;
}

impl<T, E> ResultTestExt<T, E> for Result<T, E>
where
    E: Display,
{
    fn or_panic(self, context: &str) -> T {
        self.unwrap_or_else(|error| panic!("{context}: {error}"))
    }

    fn err_or_panic(self, context: &str) -> E {
        let Err(error) = self else {
            panic!("{context}");
        };
        error
    }
}

pub(crate) trait OptionTestExt<T> {
    fn or_panic(self, context: &str) -> T;
}

impl<T> OptionTestExt<T> for Option<T> {
    fn or_panic(self, context: &str) -> T {
        let Some(value) = self else {
            panic!("{context}");
        };
        value
    }
}

impl ChildGuard {
    pub(crate) fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    pub(crate) fn external() -> Self {
        Self { child: None }
    }

    pub(crate) fn kill(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if let Some(_status) = child
            .try_wait()
            .unwrap_or_else(|error| panic!("poll Julia child: {error}"))
        {
            return;
        }
        child
            .kill()
            .unwrap_or_else(|error| panic!("kill Julia child: {error}"));
        let _ = child.wait();
    }
}

impl Drop for ChildGuard {
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

pub(crate) fn reserve_test_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map_or_else(
            |error| panic!("reserve test port: {error}"),
            |address| address.port(),
        )
}

pub(crate) fn assert_f64_eq(actual: f64, expected: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 1.0e-12,
        "expected `{expected}` but got `{actual}` (delta: {delta})"
    );
}

pub(crate) fn assert_sorted_json_snapshot(name: &str, value: impl Serialize) {
    let payload = canonicalize_json(
        serde_json::to_value(value)
            .unwrap_or_else(|error| panic!("serialize snapshot `{name}`: {error}")),
    );
    insta::with_settings!({
        snapshot_path => "snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_json_snapshot!(name, payload);
    });
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonicalize_json(value)))
                    .collect(),
            )
        }
        other => other,
    }
}

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../")
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve repo root: {error}"))
}

pub(crate) fn wendaosearch_package_dir() -> PathBuf {
    if let Some(configured) = std::env::var_os("WENDAOSEARCH_PACKAGE_DIR") {
        return resolve_existing_path("WendaoSearch package dir", configured);
    }

    repo_root()
        .join(".data/WendaoSearch.jl")
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch package dir: {error}"))
}

pub(crate) fn wendaosearch_julia_project() -> PathBuf {
    let Some(configured) = std::env::var_os("WENDAOSEARCH_JULIA_PROJECT") else {
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

pub(crate) fn wendaosearch_script(name: &str) -> PathBuf {
    wendaosearch_package_dir()
        .join("scripts")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve WendaoSearch script `{name}`: {error}"))
}

pub(crate) fn ensure_linked_julia_parser_summary_service() -> Result<(), Box<dyn std::error::Error>>
{
    let service = LINKED_JULIA_PARSER_SUMMARY_SERVICE.get_or_init(|| {
        let (base_url, guard) = ensure_process_managed_julia_parser_summary_service()
            .map(|base_url| (base_url, None))
            .or_else(|process_error| {
                let (base_url, guard) = std::thread::spawn(|| {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|error| error.to_string())?;
                    Ok::<(String, JuliaExampleServiceGuard), String>(
                        runtime.block_on(spawn_wendaosearch_julia_parser_summary_service()),
                    )
                })
                .join()
                .map_err(|_| "linked Julia parser-summary service thread panicked".to_string())?
                .map_err(|spawn_error| {
                    format!(
                        "{process_error}; fallback direct Julia parser-summary spawn also failed: {spawn_error}"
                    )
                })?;
                Ok::<(String, Option<JuliaExampleServiceGuard>), String>((base_url, Some(guard)))
            })?;
        set_linked_julia_parser_summary_base_url_for_tests(base_url.as_str())?;
        Ok::<LinkedJuliaParserSummaryService, String>(LinkedJuliaParserSummaryService {
            _guard: guard.map(Mutex::new),
        })
    });
    service
        .as_ref()
        .map(|_| ())
        .map_err(|message| Box::new(IoError::other(message.clone())) as Box<dyn std::error::Error>)
}

pub(crate) fn ensure_linked_modelica_parser_summary_service()
-> Result<(), Box<dyn std::error::Error>> {
    let service = LINKED_MODELICA_PARSER_SUMMARY_SERVICE.get_or_init(|| {
        let (base_url, guard) = ensure_process_managed_modelica_parser_summary_service()
            .map(|base_url| (base_url, None))
            .or_else(|process_error| {
                let (base_url, guard) = std::thread::spawn(|| {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|error| error.to_string())?;
                    Ok::<(String, JuliaExampleServiceGuard), String>(
                        runtime.block_on(spawn_wendaosearch_modelica_parser_summary_service()),
                    )
                })
                .join()
                .map_err(|_| "linked Modelica parser-summary service thread panicked".to_string())?
                .map_err(|spawn_error| {
                    format!("{process_error}; fallback direct spawn also failed: {spawn_error}")
                })?;
                Ok::<(String, Option<JuliaExampleServiceGuard>), String>((base_url, Some(guard)))
            })?;
        set_linked_modelica_parser_summary_base_url_for_tests(base_url.as_str())?;
        Ok::<LinkedModelicaParserSummaryService, String>(LinkedModelicaParserSummaryService {
            _guard: guard.map(Mutex::new),
        })
    });
    service
        .as_ref()
        .map(|_| ())
        .map_err(|message| Box::new(IoError::other(message.clone())) as Box<dyn std::error::Error>)
}

fn ensure_process_managed_modelica_parser_summary_service() -> Result<String, String> {
    if probe_wendaosearch_modelica_parser_summary_route_for_tests(
        SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL,
    )
    .is_ok()
    {
        return Ok(SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL.to_string());
    }

    let status = Command::new("direnv")
        .arg("exec")
        .arg(".")
        .arg("devenv")
        .arg("processes")
        .arg("up")
        .arg("-d")
        .arg("wendaocodeparser-parser-summary")
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| {
            format!("spawn process-managed wendaocodeparser-parser-summary service: {error}")
        })?;
    if !status.success() {
        return Err(format!(
            "process-managed wendaocodeparser-parser-summary exited with status {status}"
        ));
    }

    wait_for_parser_summary_socket_ready("127.0.0.1:41081", 600)?;
    wait_for_modelica_parser_summary_route_ready(120)?;

    Ok(SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL.to_string())
}

fn wait_for_parser_summary_socket_ready(socket_addr: &str, attempts: usize) -> Result<(), String> {
    for _ in 0..attempts {
        if TcpStream::connect(socket_addr).is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(200));
    }

    Err(format!(
        "shared wendaosearch parser-summary socket `{socket_addr}` did not become ready in time"
    ))
}

fn wait_for_modelica_parser_summary_route_ready(attempts: usize) -> Result<(), String> {
    let mut last_error = None;
    for _ in 0..attempts {
        match probe_wendaosearch_modelica_parser_summary_route_for_tests(
            SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL,
        ) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                sleep(Duration::from_millis(500));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        "shared Modelica parser-summary route did not become ready in time".to_string()
    }))
}

fn ensure_process_managed_julia_parser_summary_service() -> Result<String, String> {
    if probe_wendaosearch_julia_parser_summary_route_for_tests(
        SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL,
    )
    .is_ok()
    {
        return Ok(SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL.to_string());
    }

    let status = Command::new("direnv")
        .arg("exec")
        .arg(".")
        .arg("devenv")
        .arg("processes")
        .arg("up")
        .arg("-d")
        .arg("wendaocodeparser-parser-summary")
        .current_dir(repo_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| {
            format!("spawn process-managed wendaocodeparser-parser-summary service: {error}")
        })?;
    if !status.success() {
        return Err(format!(
            "process-managed wendaocodeparser-parser-summary exited with status {status}"
        ));
    }

    wait_for_parser_summary_socket_ready("127.0.0.1:41081", 600)?;
    wait_for_julia_parser_summary_route_ready(120)?;

    Ok(SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL.to_string())
}

fn wait_for_julia_parser_summary_route_ready(attempts: usize) -> Result<(), String> {
    let mut last_error = None;
    for _ in 0..attempts {
        match probe_wendaosearch_julia_parser_summary_route_for_tests(
            SHARED_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL,
        ) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                sleep(Duration::from_millis(500));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        "shared Julia parser-summary route did not become ready in time".to_string()
    }))
}

fn probe_wendaosearch_julia_parser_summary_route_for_tests(base_url: &str) -> Result<(), String> {
    let repository = RegisteredRepository {
        id: "linked-julia-ready".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "parser_summary_transport": {
                    "base_url": base_url,
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let file_summary = fetch_julia_parser_file_summary_blocking_for_repository(
        &repository,
        "src/Warmup.jl",
        "module Warmup\nexport answer\nanswer() = 42\nend\n",
    )
    .map_err(|error| {
        format!("Julia file-summary readiness probe failed for `{base_url}`: {error}")
    })?;
    if file_summary.module_name.as_deref() != Some("Warmup") {
        return Err(format!(
            "Julia file-summary readiness probe returned unexpected module_name {:?} for `{base_url}`",
            file_summary.module_name
        ));
    }

    let root_summary = fetch_julia_parser_root_summary_blocking_for_repository(
        &repository,
        "src/Warmup.jl",
        "module Warmup\nexport answer\nanswer() = 42\nend\n",
    )
    .map_err(|error| {
        format!("Julia root-summary readiness probe failed for `{base_url}`: {error}")
    })?;
    if root_summary.module_name != "Warmup" {
        return Err(format!(
            "Julia root-summary readiness probe returned unexpected module_name {:?} for `{base_url}`",
            root_summary.module_name
        ));
    }

    Ok(())
}
