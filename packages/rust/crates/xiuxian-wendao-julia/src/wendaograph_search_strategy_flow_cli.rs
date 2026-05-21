//! Command-line implementation for the `WendaoGraph.jl` `SearchStrategyFlow` bridge.

use std::{
    env,
    io::{self, BufRead, Write},
    path::PathBuf,
    time::Instant,
};

use crate::integration_support::{
    SearchStrategyFlowFlightMaterializationConfig, SearchStrategyFlowPersistentBatchHost,
    SearchStrategyFlowPersistentHostStabilizationLimits,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_branch_judgements,
};
use serde::Deserialize;
use serde_json::json;

const USAGE: &str = "usage: wendaograph_search_strategy_flow --intent <text> [--search-root <path>] [--flight-base-url <url> [--flight-repo <repo>]] [--branch-judgements-tsv <tsv>] [--persistent-warm-samples <count>] [--serve-stdio]";
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
        let config = config.ok_or_else(|| "--serve-stdio requires --flight-base-url".to_owned())?;
        return run_persistent_stdio_session(&args, &config).await;
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
    run_wendaograph_search_strategy_flow_json_with_flight_materialization_and_branch_judgements(
        intent,
        args.search_root.as_path(),
        config,
        args.branch_judgements_tsv.as_deref().unwrap_or(""),
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
    config: &SearchStrategyFlowFlightMaterializationConfig,
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
        let started = Instant::now();
        let response = match parse_stdio_session_request(&line) {
            Ok(request) => {
                let result = host
                    .submit_with_flight_materialization_and_branch_judgements_and_ontology_registry(
                        &request.intent,
                        config,
                        request.branch_judgements_tsv.as_deref().unwrap_or(""),
                        request.ontology_registry_tsv.as_deref().unwrap_or(""),
                    )
                    .await;
                stdio_session_response(request.request_id.as_deref(), started, result)
            }
            Err(error) => stdio_session_error_response(None, started, error),
        };
        let response = serde_json::to_string(&response)
            .map_err(|error| format!("serialize SearchStrategyFlow stdio response: {error}"))?;
        writeln!(stdout, "{response}")
            .map_err(|error| format!("write SearchStrategyFlow stdio response: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("flush SearchStrategyFlow stdio response: {error}"))?;
    }
    host.finish()?;
    Ok(String::new())
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StdioSessionRequest {
    request_id: Option<String>,
    intent: String,
    branch_judgements_tsv: Option<String>,
    ontology_registry_tsv: Option<String>,
}

fn parse_stdio_session_request(line: &str) -> Result<StdioSessionRequest, String> {
    let request = serde_json::from_str::<StdioSessionRequest>(line)
        .map_err(|error| format!("invalid SearchStrategyFlow stdio request JSON: {error}"))?;
    if request.intent.trim().is_empty() {
        return Err("SearchStrategyFlow stdio request intent must not be blank".to_owned());
    }
    Ok(StdioSessionRequest {
        request_id: request.request_id,
        intent: request.intent.trim().to_owned(),
        branch_judgements_tsv: request.branch_judgements_tsv,
        ontology_registry_tsv: request.ontology_registry_tsv,
    })
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
    persistent_warm_samples: Option<usize>,
    serve_stdio: bool,
    branch_judgements_tsv: Option<String>,
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
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut intent = None;
    let mut search_root = None;
    let mut flight_base_url = None;
    let mut flight_repo = None;
    let mut flight_timeout_seconds = 30;
    let mut persistent_warm_samples = None;
    let mut serve_stdio = false;
    let mut branch_judgements_tsv = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--intent" => {
                intent = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --intent".to_owned())?,
                );
            }
            "--search-root" => {
                search_root =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "missing value for --search-root".to_owned()
                    })?));
            }
            "--flight-base-url" => {
                flight_base_url = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --flight-base-url".to_owned())?,
                );
            }
            "--flight-repo" => {
                flight_repo = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --flight-repo".to_owned())?,
                );
            }
            "--flight-timeout-seconds" => {
                flight_timeout_seconds = args
                    .next()
                    .ok_or_else(|| "missing value for --flight-timeout-seconds".to_owned())?
                    .parse::<u64>()
                    .map_err(|error| format!("invalid --flight-timeout-seconds: {error}"))?
                    .max(1);
            }
            "--branch-judgements-tsv" => {
                branch_judgements_tsv = Some(
                    args.next()
                        .ok_or_else(|| "missing value for --branch-judgements-tsv".to_owned())?,
                );
            }
            "--persistent-warm-samples" => {
                let sample_count = args
                    .next()
                    .ok_or_else(|| "missing value for --persistent-warm-samples".to_owned())?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --persistent-warm-samples: {error}"))?;
                if sample_count == 0 {
                    return Err("--persistent-warm-samples must be greater than zero".to_owned());
                }
                persistent_warm_samples = Some(sample_count);
            }
            "--serve-stdio" => {
                serve_stdio = true;
            }
            "--help" | "-h" => {
                return Err("WendaoGraph SearchStrategyFlow Rust bridge".to_owned());
            }
            _ => return Err(format!("unknown argument `{arg}`")),
        }
    }

    let search_root = search_root.map_or_else(
        || env::current_dir().map_err(|error| format!("resolve current dir: {error}")),
        Ok,
    )?;
    if serve_stdio && persistent_warm_samples.is_some() {
        return Err("--serve-stdio cannot be combined with --persistent-warm-samples".to_owned());
    }
    if !serve_stdio && intent.is_none() {
        return Err("missing --intent".to_owned());
    }
    if flight_base_url.is_none() && flight_repo.is_some() {
        return Err("missing --flight-base-url".to_owned());
    }
    Ok(Args {
        intent,
        search_root,
        flight_base_url,
        flight_repo,
        flight_timeout_seconds,
        persistent_warm_samples,
        serve_stdio,
        branch_judgements_tsv,
    })
}

#[cfg(test)]
#[path = "../tests/unit/wendaograph_search_strategy_flow_cli.rs"]
mod wendaograph_search_strategy_flow_cli_unit_tests;
