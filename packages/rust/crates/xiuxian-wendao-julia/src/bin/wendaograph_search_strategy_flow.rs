//! CLI entry point for the Rust-owned `WendaoGraph.jl` `SearchStrategyFlow` bridge.

use xiuxian_wendao_julia::wendaograph_search_strategy_flow_cli::run_from_env;

#[tokio::main]
async fn main() {
    run_from_env().await;
}
