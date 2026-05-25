//! Command-line implementation for the `WendaoGraph.jl` `SearchStrategyFlow` bridge.

use std::{
    env,
    io::{self, BufRead, Write},
    path::PathBuf,
    time::Instant,
};

use crate::integration_support::{
    SearchStrategyFlowFlightMaterializationConfig, SearchStrategyFlowPersistentBatchHost,
    SearchStrategyFlowPersistentHostStabilizationLimits, SearchStrategyFlowServiceExecutionRequest,
    SearchStrategyFlowSideTableExecutionRequest, SearchStrategyFlowSideTableRequest,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_side_tables,
    run_wendaograph_search_strategy_flow_json_with_service_and_flight_materialization,
};
use serde::Deserialize;
use serde_json::json;

const USAGE: &str = "usage: wendaograph_search_strategy_flow --intent <text> [--search-root <path>] [--flight-base-url <url> [--flight-repo <repo>]] [--strategy-flow-service-base-url <url>] [--query-understanding-arrow-ipc <path>] [--branch-judgements-arrow-ipc <path>] [--persistent-warm-samples <count>] [--serve-stdio]";
const STDIO_SESSION_RESPONSE_KIND: &str =
    "xiuxian_wendao.wendaograph.search_strategy_flow.persistent_stdio_response.v1";

/// Runs the `wendaograph_search_strategy_flow` command with process arguments.
///
/// This function exits the process with the command status code when argument
/// parsing or bridge execution fails.
pub async fn run_from_env() {
    let status = run_with_args(env::args().skip(1)).await;
    if status != 0 {
        std::process::exit(status);
    }
}

/// Runs the `wendaograph_search_strategy_flow` command with explicit arguments.
pub async fn run_with_args(args: impl Iterator<Item = String>) -> i32 {
    match parse_args(args) {
        Ok(args) => match run(args).await {
            Ok(trace) => {
                print!("{trace}");
                0
            }
            Err(error) => {
                eprintln!("{error}");
                1
            }
        },
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{USAGE}");
            64
        }
    }
}

async fn run(args: Args) -> Result<String, String> {
    let config = args.flight_materialization_config()?;
    if args.serve_stdio {
        return run_persistent_stdio_session(&args, config.as_ref()).await;
    }
    let intent = args
        .intent
        .as_deref()
        .ok_or_else(|| "missing --intent".to_owned())?;
    if let Some(sample_count) = args.persistent_warm_samples {
        let config = config
            .ok_or_else(|| "--persistent-warm-samples requires --flight-base-url".to_owned())?;
        return run_persistent_stabilization_report(&args, intent, &config, sample_count).await;
    }
    let query_understanding_arrow_ipc_path = args.query_understanding_arrow_ipc_path_arg();
    let branch_judgements_arrow_ipc_path = args.branch_judgements_arrow_ipc_path_arg();
    if let Some(service_base_url) = args.strategy_flow_service_base_url.as_ref() {
        return run_wendaograph_search_strategy_flow_json_with_service_and_flight_materialization(
            SearchStrategyFlowServiceExecutionRequest {
                intent: intent.to_owned(),
                search_root: args.search_root.clone(),
                flight_materialization_config: config,
                strategy_flow_service_base_url: service_base_url.clone(),
                strategy_flow_service_timeout_seconds: args.strategy_flow_service_timeout_seconds,
                query_understanding_arrow_ipc_path,
                branch_judgements_arrow_ipc_path,
            },
        )
        .await;
    }
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_side_tables(
        SearchStrategyFlowSideTableExecutionRequest::new(intent, args.search_root.as_path())
            .with_flight_materialization_config(config)
            .with_query_understanding_arrow_ipc_path(query_understanding_arrow_ipc_path)
            .with_branch_judgements_arrow_ipc_path(branch_judgements_arrow_ipc_path),
    )
    .await
}

async fn run_persistent_stabilization_report(
    args: &Args,
    intent: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
    sample_count: usize,
) -> Result<String, String> {
    let mut host = SearchStrategyFlowPersistentBatchHost::start(args.search_root.as_path())?;
    let report = host
        .stabilize_with_flight_materialization(
            intent,
            config,
            SearchStrategyFlowPersistentHostStabilizationLimits::default()
                .with_sample_count(sample_count),
        )
        .await;
    let finish = host.finish();
    match (report, finish) {
        (Ok(report), Ok(())) => {
            let value = json!({
                "kind": "xiuxian_wendao.wendaograph.search_strategy_flow.persistent_host_stabilization.v1",
                "intent": intent,
                "searchRoot": args.search_root.display().to_string(),
                "flight": {
                    "baseUrl": args.flight_base_url.as_deref().unwrap_or_default(),
                    "repo": config.repo_id,
                    "timeoutSeconds": args.flight_timeout_seconds,
                },
                "persistentHost": report.to_json_value(),
            });
            serde_json::to_string(&value)
                .map(|json| format!("{json}\n"))
                .map_err(|error| {
                    format!("serialize SearchStrategyFlow persistent host report: {error}")
                })
        }
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(report_error), Err(finish_error)) => Err(format!("{report_error}; {finish_error}")),
    }
}

async fn run_persistent_stdio_session(
    args: &Args,
    config: Option<&SearchStrategyFlowFlightMaterializationConfig>,
) -> Result<String, String> {
    let mut host = SearchStrategyFlowPersistentBatchHost::start(args.search_root.as_path())?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line =
            line.map_err(|error| format!("read SearchStrategyFlow stdio request: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let responses = match parse_stdio_session_input(&line) {
            Ok(StdioSessionInput::Single(request)) => {
                vec![submit_stdio_session_request(&mut host, args, config, request).await]
            }
            Ok(StdioSessionInput::Batch(request)) => {
                submit_stdio_session_batch(&mut host, args, config, request.requests).await
            }
            Err(error) => vec![stdio_session_error_response(None, Instant::now(), error)],
        };
        for response in responses {
            let response = serde_json::to_string(&response)
                .map_err(|error| format!("serialize SearchStrategyFlow stdio response: {error}"))?;
            writeln!(stdout, "{response}")
                .map_err(|error| format!("write SearchStrategyFlow stdio response: {error}"))?;
            stdout
                .flush()
                .map_err(|error| format!("flush SearchStrategyFlow stdio response: {error}"))?;
        }
    }
    host.finish()?;
    Ok(String::new())
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum StdioSessionInput {
    Batch(StdioSessionBatchRequest),
    Single(StdioSessionRequest),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StdioSessionBatchRequest {
    requests: Vec<StdioSessionRequest>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StdioSessionRequest {
    request_id: Option<String>,
    intent: String,
    query_understanding_arrow_ipc_path: Option<String>,
    branch_judgements_arrow_ipc_path: Option<String>,
    ontology_registry_arrow_ipc_path: Option<String>,
}

impl StdioSessionRequest {
    fn query_understanding_arrow_ipc_path_arg(&self) -> &str {
        self.query_understanding_arrow_ipc_path
            .as_deref()
            .unwrap_or_default()
    }

    fn branch_judgements_arrow_ipc_path_arg(&self) -> &str {
        self.branch_judgements_arrow_ipc_path
            .as_deref()
            .unwrap_or_default()
    }

    fn ontology_registry_arrow_ipc_path_arg(&self) -> &str {
        self.ontology_registry_arrow_ipc_path
            .as_deref()
            .unwrap_or_default()
    }
}

#[cfg(test)]
fn parse_stdio_session_request(line: &str) -> Result<StdioSessionRequest, String> {
    let request = serde_json::from_str::<StdioSessionRequest>(line)
        .map_err(|error| format!("invalid SearchStrategyFlow stdio request JSON: {error}"))?;
    normalize_stdio_session_request(request)
}

fn parse_stdio_session_input(line: &str) -> Result<StdioSessionInput, String> {
    let input = serde_json::from_str::<StdioSessionInput>(line)
        .map_err(|error| format!("invalid SearchStrategyFlow stdio request JSON: {error}"))?;
    match input {
        StdioSessionInput::Single(request) => {
            normalize_stdio_session_request(request).map(StdioSessionInput::Single)
        }
        StdioSessionInput::Batch(request) => {
            if request.requests.is_empty() {
                return Err("SearchStrategyFlow stdio batch request must not be empty".to_owned());
            }
            request
                .requests
                .into_iter()
                .map(normalize_stdio_session_request)
                .collect::<Result<Vec<_>, _>>()
                .map(|requests| StdioSessionInput::Batch(StdioSessionBatchRequest { requests }))
        }
    }
}

fn normalize_stdio_session_request(
    request: StdioSessionRequest,
) -> Result<StdioSessionRequest, String> {
    if request.intent.trim().is_empty() {
        return Err("SearchStrategyFlow stdio request intent must not be blank".to_owned());
    }
    Ok(StdioSessionRequest {
        request_id: request.request_id,
        intent: request.intent.trim().to_owned(),
        query_understanding_arrow_ipc_path: request.query_understanding_arrow_ipc_path,
        branch_judgements_arrow_ipc_path: request.branch_judgements_arrow_ipc_path,
        ontology_registry_arrow_ipc_path: request.ontology_registry_arrow_ipc_path,
    })
}

async fn submit_stdio_session_request(
    host: &mut SearchStrategyFlowPersistentBatchHost,
    args: &Args,
    config: Option<&SearchStrategyFlowFlightMaterializationConfig>,
    request: StdioSessionRequest,
) -> serde_json::Value {
    let started = Instant::now();
    let result = if let Some(config) = config {
        host.submit_with_flight_materialization_and_side_tables(
            &request.intent,
            config,
            request.query_understanding_arrow_ipc_path_arg(),
            request.branch_judgements_arrow_ipc_path_arg(),
            request.ontology_registry_arrow_ipc_path_arg(),
        )
        .await
    } else {
        host.submit_with_markdown_candidates_and_side_tables(
            &request.intent,
            args.search_root.as_path(),
            request.query_understanding_arrow_ipc_path_arg(),
            request.branch_judgements_arrow_ipc_path_arg(),
            request.ontology_registry_arrow_ipc_path_arg(),
        )
    };
    stdio_session_response(request.request_id.as_deref(), started, result)
}

async fn submit_stdio_session_batch(
    host: &mut SearchStrategyFlowPersistentBatchHost,
    args: &Args,
    config: Option<&SearchStrategyFlowFlightMaterializationConfig>,
    requests: Vec<StdioSessionRequest>,
) -> Vec<serde_json::Value> {
    let started = Instant::now();
    let request_ids = requests
        .iter()
        .map(|request| request.request_id.clone())
        .collect::<Vec<_>>();
    let side_table_requests = requests
        .into_iter()
        .map(|request| {
            SearchStrategyFlowSideTableRequest::new(
                request.intent,
                request
                    .query_understanding_arrow_ipc_path
                    .unwrap_or_default(),
                request.branch_judgements_arrow_ipc_path.unwrap_or_default(),
                request.ontology_registry_arrow_ipc_path.unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    let result = if let Some(config) = config {
        host.submit_batch_with_flight_materialization_and_side_tables(side_table_requests, config)
            .await
    } else {
        host.submit_batch_with_markdown_candidates_and_side_tables(
            args.search_root.as_path(),
            side_table_requests,
        )
    };
    match result {
        Ok(traces) if traces.len() == request_ids.len() => request_ids
            .iter()
            .zip(traces)
            .map(|(request_id, trace)| {
                stdio_session_response(request_id.as_deref(), started, Ok(trace))
            })
            .collect(),
        Ok(traces) => {
            let error = format!(
                "SearchStrategyFlow stdio batch expected {} trace(s), got {}",
                request_ids.len(),
                traces.len()
            );
            request_ids
                .iter()
                .map(|request_id| {
                    stdio_session_error_response(request_id.as_deref(), started, error.clone())
                })
                .collect()
        }
        Err(error) => request_ids
            .iter()
            .map(|request_id| {
                stdio_session_error_response(request_id.as_deref(), started, error.clone())
            })
            .collect(),
    }
}

fn stdio_session_response(
    request_id: Option<&str>,
    started: Instant,
    result: Result<String, String>,
) -> serde_json::Value {
    match result {
        Ok(trace) => match serde_json::from_str::<serde_json::Value>(&trace) {
            Ok(trace) => json!({
                "kind": STDIO_SESSION_RESPONSE_KIND,
                "requestId": request_id,
                "ok": true,
                "elapsedMs": started.elapsed().as_secs_f64() * 1000.0,
                "trace": trace,
            }),
            Err(error) => stdio_session_error_response(
                request_id,
                started,
                format!("parse SearchStrategyFlow trace JSON: {error}"),
            ),
        },
        Err(error) => stdio_session_error_response(request_id, started, error),
    }
}

fn stdio_session_error_response(
    request_id: Option<&str>,
    started: Instant,
    error: impl Into<String>,
) -> serde_json::Value {
    json!({
        "kind": STDIO_SESSION_RESPONSE_KIND,
        "requestId": request_id,
        "ok": false,
        "elapsedMs": started.elapsed().as_secs_f64() * 1000.0,
        "error": error.into(),
    })
}

struct Args {
    intent: Option<String>,
    search_root: PathBuf,
    flight_base_url: Option<String>,
    flight_repo: Option<String>,
    flight_timeout_seconds: u64,
    strategy_flow_service_base_url: Option<String>,
    strategy_flow_service_timeout_seconds: u64,
    persistent_warm_samples: Option<usize>,
    serve_stdio: bool,
    query_understanding_arrow_ipc_path: Option<PathBuf>,
    branch_judgements_arrow_ipc_path: Option<PathBuf>,
}

impl Args {
    fn flight_materialization_config(
        &self,
    ) -> Result<Option<SearchStrategyFlowFlightMaterializationConfig>, String> {
        let Some(base_url) = self.flight_base_url.as_ref() else {
            return Ok(None);
        };
        let config = match self.flight_repo.as_ref() {
            Some(repo) => SearchStrategyFlowFlightMaterializationConfig::new(base_url, repo),
            None => SearchStrategyFlowFlightMaterializationConfig::new_with_backend_default_repo(
                base_url,
            ),
        }
        .map_err(|error| format!("invalid SearchStrategyFlow Flight config: {error}"))?
        .with_timeout_seconds(self.flight_timeout_seconds);
        Ok(Some(config))
    }

    fn branch_judgements_arrow_ipc_path_arg(&self) -> String {
        self.branch_judgements_arrow_ipc_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    fn query_understanding_arrow_ipc_path_arg(&self) -> String {
        self.query_understanding_arrow_ipc_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

#[derive(Default)]
struct ArgsDraft {
    intent: Option<String>,
    search_root: Option<PathBuf>,
    flight_base_url: Option<String>,
    flight_repo: Option<String>,
    flight_timeout_seconds: u64,
    strategy_flow_service_base_url: Option<String>,
    strategy_flow_service_timeout_seconds: u64,
    persistent_warm_samples: Option<usize>,
    serve_stdio: bool,
    query_understanding_arrow_ipc_path: Option<PathBuf>,
    branch_judgements_arrow_ipc_path: Option<PathBuf>,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut draft = ArgsDraft {
        flight_timeout_seconds: 30,
        strategy_flow_service_timeout_seconds: 30,
        ..ArgsDraft::default()
    };
    let mut args = args;
    while let Some(arg) = args.next() {
        parse_arg(&mut draft, arg.as_str(), &mut args)?;
    }
    finish_args(draft)
}

fn parse_arg(
    draft: &mut ArgsDraft,
    arg: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<(), String> {
    match arg {
        "--intent" => draft.intent = Some(take_required_value(args, "--intent")?),
        "--search-root" => {
            draft.search_root = Some(PathBuf::from(take_required_value(args, "--search-root")?));
        }
        "--flight-base-url" => {
            draft.flight_base_url = Some(take_required_value(args, "--flight-base-url")?);
        }
        "--flight-repo" => {
            draft.flight_repo = Some(take_required_value(args, "--flight-repo")?);
        }
        "--flight-timeout-seconds" => {
            draft.flight_timeout_seconds =
                parse_non_zero_u64(take_required_value(args, arg)?.as_str(), arg)?;
        }
        "--strategy-flow-service-base-url" => {
            draft.strategy_flow_service_base_url = Some(take_required_value(
                args,
                "--strategy-flow-service-base-url",
            )?);
        }
        "--strategy-flow-service-timeout-seconds" => {
            draft.strategy_flow_service_timeout_seconds =
                parse_non_zero_u64(take_required_value(args, arg)?.as_str(), arg)?;
        }
        "--branch-judgements-arrow-ipc" => {
            draft.branch_judgements_arrow_ipc_path = Some(PathBuf::from(take_required_value(
                args,
                "--branch-judgements-arrow-ipc",
            )?));
        }
        "--query-understanding-arrow-ipc" => {
            draft.query_understanding_arrow_ipc_path = Some(PathBuf::from(take_required_value(
                args,
                "--query-understanding-arrow-ipc",
            )?));
        }
        "--persistent-warm-samples" => {
            let sample_count = take_required_value(args, "--persistent-warm-samples")?
                .parse::<usize>()
                .map_err(|error| format!("invalid --persistent-warm-samples: {error}"))?;
            if sample_count == 0 {
                return Err("--persistent-warm-samples must be greater than zero".to_owned());
            }
            draft.persistent_warm_samples = Some(sample_count);
        }
        "--serve-stdio" => draft.serve_stdio = true,
        "--help" | "-h" => {
            return Err("WendaoGraph SearchStrategyFlow Rust bridge".to_owned());
        }
        _ => return Err(format!("unknown argument `{arg}`")),
    }
    Ok(())
}

fn take_required_value(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {option}"))
}

fn parse_non_zero_u64(value: &str, option: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|error| format!("invalid {option}: {error}"))
        .map(|value| value.max(1))
}

fn finish_args(draft: ArgsDraft) -> Result<Args, String> {
    let search_root = draft.search_root.map_or_else(
        || env::current_dir().map_err(|error| format!("resolve current dir: {error}")),
        Ok,
    )?;
    if draft.serve_stdio && draft.persistent_warm_samples.is_some() {
        return Err("--serve-stdio cannot be combined with --persistent-warm-samples".to_owned());
    }
    if draft.serve_stdio && draft.strategy_flow_service_base_url.is_some() {
        return Err(
            "--serve-stdio cannot be combined with --strategy-flow-service-base-url".to_owned(),
        );
    }
    if draft.persistent_warm_samples.is_some() && draft.strategy_flow_service_base_url.is_some() {
        return Err(
            "--persistent-warm-samples cannot be combined with --strategy-flow-service-base-url"
                .to_owned(),
        );
    }
    if !draft.serve_stdio && draft.intent.is_none() {
        return Err("missing --intent".to_owned());
    }
    if draft.flight_base_url.is_none() && draft.flight_repo.is_some() {
        return Err("missing --flight-base-url".to_owned());
    }
    Ok(Args {
        intent: draft.intent,
        search_root,
        flight_base_url: draft.flight_base_url,
        flight_repo: draft.flight_repo,
        flight_timeout_seconds: draft.flight_timeout_seconds,
        strategy_flow_service_base_url: draft.strategy_flow_service_base_url,
        strategy_flow_service_timeout_seconds: draft.strategy_flow_service_timeout_seconds,
        persistent_warm_samples: draft.persistent_warm_samples,
        serve_stdio: draft.serve_stdio,
        query_understanding_arrow_ipc_path: draft.query_understanding_arrow_ipc_path,
        branch_judgements_arrow_ipc_path: draft.branch_judgements_arrow_ipc_path,
    })
}

#[cfg(test)]
#[path = "../tests/unit/wendaograph_search_strategy_flow_cli/mod.rs"]
mod wendaograph_search_strategy_flow_cli_unit_tests;
