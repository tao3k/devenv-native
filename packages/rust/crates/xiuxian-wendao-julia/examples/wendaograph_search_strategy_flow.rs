//! CLI proof surface for the Rust-owned `WendaoGraph.jl` SearchStrategyFlow bridge.

use std::env;
use std::path::PathBuf;

use xiuxian_wendao_julia::integration_support::run_wendaograph_search_strategy_flow_json;

fn main() {
    match parse_args(env::args().skip(1)) {
        Ok(args) => match run_wendaograph_search_strategy_flow_json(&args.intent, args.search_root)
        {
            Ok(trace) => print!("{trace}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("{error}");
            eprintln!(
                "usage: wendaograph_search_strategy_flow --intent <text> --search-root <path>"
            );
            std::process::exit(64);
        }
    }
}

struct Args {
    intent: String,
    search_root: PathBuf,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut intent = None;
    let mut search_root = None;
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
            "--help" | "-h" => {
                return Err("WendaoGraph SearchStrategyFlow Rust bridge".to_owned());
            }
            _ => return Err(format!("unknown argument `{arg}`")),
        }
    }

    let intent = intent.ok_or_else(|| "missing --intent".to_owned())?;
    let search_root = search_root.ok_or_else(|| "missing --search-root".to_owned())?;
    Ok(Args {
        intent,
        search_root,
    })
}
