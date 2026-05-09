//! CLI proof surface for the Rust-owned `WendaoGraph.jl` SearchStrategyFlow bridge.

use std::env;
use std::path::PathBuf;

use xiuxian_wendao_julia::integration_support::{
    SearchStrategyFlowFlightMaterializationConfig,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization,
};

#[tokio::main]
async fn main() {
    match parse_args(env::args().skip(1)) {
        Ok(args) => {
            let config = match args.flight_materialization_config() {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(64);
                }
            };
            match run_wendaograph_search_strategy_flow_json_with_flight_materialization(
                &args.intent,
                args.search_root,
                config,
            )
            .await
            {
                Ok(trace) => print!("{trace}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!(
                "usage: wendaograph_search_strategy_flow --intent <text> --search-root <path> [--flight-base-url <url> --flight-repo <repo>]"
            );
            std::process::exit(64);
        }
    }
}

struct Args {
    intent: String,
    search_root: PathBuf,
    flight_base_url: Option<String>,
    flight_repo: Option<String>,
    flight_timeout_seconds: u64,
}

impl Args {
    fn flight_materialization_config(
        &self,
    ) -> Result<Option<SearchStrategyFlowFlightMaterializationConfig>, String> {
        let Some(base_url) = self.flight_base_url.as_ref() else {
            return Ok(None);
        };
        let Some(repo) = self.flight_repo.as_ref() else {
            return Ok(None);
        };
        Ok(Some(
            SearchStrategyFlowFlightMaterializationConfig::new(base_url, repo)
                .map_err(|error| format!("invalid SearchStrategyFlow Flight config: {error}"))?
                .with_timeout_seconds(self.flight_timeout_seconds),
        ))
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut intent = None;
    let mut search_root = None;
    let mut flight_base_url = None;
    let mut flight_repo = None;
    let mut flight_timeout_seconds = 30;
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
            "--help" | "-h" => {
                return Err("WendaoGraph SearchStrategyFlow Rust bridge".to_owned());
            }
            _ => return Err(format!("unknown argument `{arg}`")),
        }
    }

    let intent = intent.ok_or_else(|| "missing --intent".to_owned())?;
    let search_root = search_root.ok_or_else(|| "missing --search-root".to_owned())?;
    match (&flight_base_url, &flight_repo) {
        (Some(_), Some(_)) | (None, None) => {}
        (Some(_), None) => return Err("missing --flight-repo".to_owned()),
        (None, Some(_)) => return Err("missing --flight-base-url".to_owned()),
    }
    Ok(Args {
        intent,
        search_root,
        flight_base_url,
        flight_repo,
        flight_timeout_seconds,
    })
}
