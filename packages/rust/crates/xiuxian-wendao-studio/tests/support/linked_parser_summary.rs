use std::fs;
use std::io::Error as IoError;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use toml::Value;
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::{
    JuliaServiceGuard, spawn_wendaosearch_all_parser_summary_service,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::{
    clear_modelica_parser_summary_transport_cache_for_tests,
    set_linked_julia_parser_summary_base_url_for_tests,
    set_linked_modelica_parser_summary_base_url_for_tests,
};

use super::repo_parser_summary;
use repo_parser_summary::{FakeParserSummaryServiceGuard, spawn_fake_julia_parser_summary_service};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV: &str = "RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST";
const WENDAOSEARCH_CONFIG_ENV: &str = "WENDAOSEARCH_CONFIG";
const WENDAOSEARCH_PACKAGE_DIR_ENV: &str = "WENDAOSEARCH_PACKAGE_DIR";
const PROCESS_MANAGED_PARSER_SUMMARY_SERVICE_NAME: &str = "wendaocodeparser-parser-summary";
const PROCESS_MANAGED_READY_ATTEMPTS: usize = 600;

struct LinkedParserSummaryService {
    base_url: String,
    _guard: Mutex<LinkedParserSummaryGuard>,
}

enum LinkedParserSummaryGuard {
    #[cfg(feature = "julia")]
    Real { _guard: JuliaServiceGuard },
    Fake {
        _guard: FakeParserSummaryServiceGuard,
    },
}

#[derive(Clone, Copy)]
enum ProcessManagedParserSummaryMode {
    Required,
    BestEffort,
}

impl ProcessManagedParserSummaryMode {
    fn already_running_attempts(self) -> usize {
        match self {
            Self::Required => 600,
            Self::BestEffort => 25,
        }
    }
}

static LINKED_PARSER_SUMMARY_SERVICE: OnceLock<Result<LinkedParserSummaryService, String>> =
    OnceLock::new();
static PROCESS_MANAGED_PARSER_SUMMARY_SERVICE: OnceLock<Result<(), String>> = OnceLock::new();

pub fn ensure_linked_parser_summary_service() -> TestResult {
    linked_parser_summary_base_url().map(|_| ())
}

pub fn ensure_linked_modelica_parser_summary_service() -> TestResult {
    linked_modelica_parser_summary_base_url().map(|_| ())
}

pub fn linked_modelica_parser_summary_base_url() -> Result<String, Box<dyn std::error::Error>> {
    linked_parser_summary_base_url()
}

pub fn linked_parser_summary_base_url() -> Result<String, Box<dyn std::error::Error>> {
    if process_managed_wendaosearch_test_enabled() {
        ensure_process_managed_parser_summary_service(ProcessManagedParserSummaryMode::Required)?;
        return process_managed_parser_summary_base_url()
            .map_err(|message| Box::new(IoError::other(message)) as Box<dyn std::error::Error>);
    }
    if process_managed_parser_summary_service_is_configured()
        && ensure_process_managed_parser_summary_service(
            ProcessManagedParserSummaryMode::BestEffort,
        )
        .is_ok()
    {
        return process_managed_parser_summary_base_url()
            .map_err(|message| Box::new(IoError::other(message)) as Box<dyn std::error::Error>);
    }
    let service = linked_parser_summary_service()?;
    configure_linked_parser_summary_base_url(service.base_url.as_str())
        .map_err(|message| Box::new(IoError::other(message)) as Box<dyn std::error::Error>)?;
    Ok(service.base_url.clone())
}

fn linked_parser_summary_service()
-> Result<&'static LinkedParserSummaryService, Box<dyn std::error::Error>> {
    let service = LINKED_PARSER_SUMMARY_SERVICE.get_or_init(|| {
        let (base_url, guard) = spawn_in_process_linked_parser_summary_service()?;
        configure_linked_parser_summary_base_url(base_url.as_str())?;
        Ok(LinkedParserSummaryService {
            base_url,
            _guard: Mutex::new(guard),
        })
    });
    match service.as_ref() {
        Ok(service) => Ok(service),
        Err(message) => {
            Err(Box::new(IoError::other(message.clone())) as Box<dyn std::error::Error>)
        }
    }
}

fn spawn_in_process_linked_parser_summary_service()
-> Result<(String, LinkedParserSummaryGuard), String> {
    #[cfg(not(feature = "julia"))]
    {
        spawn_fake_julia_parser_summary_service()
            .map(|(base_url, guard)| (base_url, LinkedParserSummaryGuard::Fake { _guard: guard }))
    }
    #[cfg(feature = "julia")]
    {
        if !real_parser_summary_service_is_available() {
            return spawn_fake_julia_parser_summary_service().map(|(base_url, guard)| {
                (base_url, LinkedParserSummaryGuard::Fake { _guard: guard })
            });
        }
        match std::thread::spawn(|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            Ok::<(String, JuliaServiceGuard), String>(
                runtime.block_on(spawn_wendaosearch_all_parser_summary_service()),
            )
        })
        .join()
        {
            Ok(Ok((base_url, guard))) => {
                Ok((base_url, LinkedParserSummaryGuard::Real { _guard: guard }))
            }
            Ok(Err(_)) | Err(_) => {
                spawn_fake_julia_parser_summary_service().map(|(base_url, guard)| {
                    (base_url, LinkedParserSummaryGuard::Fake { _guard: guard })
                })
            }
        }
    }
}

#[cfg(feature = "julia")]
fn real_parser_summary_service_is_available() -> bool {
    std::env::var_os(WENDAOSEARCH_PACKAGE_DIR_ENV)
        .filter(|value| !value.is_empty())
        .is_some_and(|path| Path::new(&path).exists())
}

fn configure_linked_parser_summary_base_url(base_url: &str) -> Result<(), String> {
    #[cfg(feature = "julia")]
    {
        set_linked_julia_parser_summary_base_url_for_tests(base_url)?;
        set_linked_modelica_parser_summary_base_url_for_tests(base_url)?;
    }
    #[cfg(not(feature = "julia"))]
    if base_url.trim().is_empty() {
        return Err("linked parser summary base URL must not be empty".to_string());
    }
    Ok(())
}

fn process_managed_wendaosearch_test_enabled() -> bool {
    std::env::var_os(RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV).is_some()
}

fn process_managed_parser_summary_service_is_configured() -> bool {
    process_managed_parser_summary_base_url().is_ok()
}

fn ensure_process_managed_parser_summary_service(
    mode: ProcessManagedParserSummaryMode,
) -> TestResult {
    let service = PROCESS_MANAGED_PARSER_SUMMARY_SERVICE.get_or_init(|| {
        let base_url = process_managed_parser_summary_base_url()?;
        if !service_is_ready(base_url.as_str())? {
            start_process_managed_parser_summary_service(base_url.as_str(), mode)?;
        }
        wait_for_service_ready(base_url.as_str(), PROCESS_MANAGED_READY_ATTEMPTS)?;
        configure_process_managed_parser_summary_base_url(base_url.as_str())?;
        Ok(())
    });
    match service.as_ref() {
        Ok(()) => Ok(()),
        Err(message) => {
            Err(Box::new(IoError::other(message.clone())) as Box<dyn std::error::Error>)
        }
    }
}

fn configure_process_managed_parser_summary_base_url(base_url: &str) -> Result<(), String> {
    #[cfg(feature = "julia")]
    {
        clear_modelica_parser_summary_transport_cache_for_tests();
        set_linked_julia_parser_summary_base_url_for_tests(base_url)
            .map_err(|error| error.clone())?;
        set_linked_modelica_parser_summary_base_url_for_tests(base_url)
            .map_err(|error| error.clone())?;
    }
    #[cfg(not(feature = "julia"))]
    if base_url.trim().is_empty() {
        return Err("process-managed parser summary base URL must not be empty".to_string());
    }
    Ok(())
}

fn start_process_managed_parser_summary_service(
    base_url: &str,
    mode: ProcessManagedParserSummaryMode,
) -> Result<(), String> {
    let output = devenv_processes_command([
        "up",
        "-d",
        PROCESS_MANAGED_PARSER_SUMMARY_SERVICE_NAME,
    ])
    .output()
    .map_err(|error| {
        format!(
            "start process-managed `{PROCESS_MANAGED_PARSER_SUMMARY_SERVICE_NAME}` service: {error}"
        )
    })?;
    if !output.status.success() {
        if output_mentions_processes_already_running(&output) {
            return wait_for_service_ready(base_url, mode.already_running_attempts());
        }
        return Err(format!(
            "start process-managed `{PROCESS_MANAGED_PARSER_SUMMARY_SERVICE_NAME}` service failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    wait_for_service_ready(base_url, PROCESS_MANAGED_READY_ATTEMPTS)
}

fn output_mentions_processes_already_running(output: &std::process::Output) -> bool {
    String::from_utf8_lossy(&output.stdout).contains("Processes already running")
        || String::from_utf8_lossy(&output.stderr).contains("Processes already running")
}

fn process_managed_parser_summary_base_url() -> Result<String, String> {
    let config_path = process_managed_parser_summary_config_path();
    let config_text = fs::read_to_string(&config_path)
        .map_err(|error| format!("read `{}`: {error}", config_path.display()))?;
    let config_value: Value = toml::from_str(&config_text)
        .map_err(|error| format!("parse `{}`: {error}", config_path.display()))?;
    let interface = config_value
        .get("interface")
        .and_then(Value::as_table)
        .ok_or_else(|| format!("`{}` is missing table `[interface]`", config_path.display()))?;
    let host = interface
        .get("host")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "`{}` is missing string `[interface].host`",
                config_path.display()
            )
        })?;
    let port = interface
        .get("port")
        .and_then(Value::as_integer)
        .ok_or_else(|| {
            format!(
                "`{}` is missing integer `[interface].port`",
                config_path.display()
            )
        })?;
    Ok(format!("http://{host}:{port}"))
}

fn process_managed_parser_summary_config_path() -> PathBuf {
    if let Some(path) = std::env::var_os(WENDAOSEARCH_CONFIG_ENV).filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }
    if let Some(package_dir) =
        std::env::var_os(WENDAOSEARCH_PACKAGE_DIR_ENV).filter(|value| !value.is_empty())
    {
        return PathBuf::from(package_dir)
            .join("config")
            .join("live")
            .join("parser_summary.toml");
    }
    repo_root()
        .join(".data")
        .join("WendaoCodeParser.jl")
        .join("config")
        .join("live")
        .join("parser_summary.toml")
}

fn wait_for_service_ready(base_url: &str, attempts: usize) -> Result<(), String> {
    for _ in 0..attempts {
        if service_is_ready(base_url)? {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(format!(
        "process-managed `{PROCESS_MANAGED_PARSER_SUMMARY_SERVICE_NAME}` did not become ready in time"
    ))
}

fn service_is_ready(base_url: &str) -> Result<bool, String> {
    let socket_addr = socket_addr_from_base_url(base_url)?;
    match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn socket_addr_from_base_url(base_url: &str) -> Result<SocketAddr, String> {
    let socket_addr = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))
        .unwrap_or(base_url);
    socket_addr
        .parse::<SocketAddr>()
        .map_err(|error| format!("parse socket address `{socket_addr}`: {error}"))
}

fn repo_root() -> PathBuf {
    if let Ok(project_root) = std::env::var("PRJ_ROOT") {
        let candidate = PathBuf::from(project_root);
        if repo_root_candidate_is_valid(candidate.as_path()) {
            return candidate;
        }
    }

    match Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4) {
        Some(path) if repo_root_candidate_is_valid(path) => path.to_path_buf(),
        Some(path) => panic!(
            "resolved workspace root candidate `{}` failed marker checks",
            path.display()
        ),
        None => panic!("workspace root"),
    }
}

fn repo_root_candidate_is_valid(candidate: &Path) -> bool {
    candidate.join("Cargo.lock").is_file()
        && candidate
            .join("packages/rust/crates/xiuxian-wendao/Cargo.toml")
            .is_file()
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
