//! Shared-query `FlightSQL` server binary over the Wendao search-plane surface.

#[cfg(feature = "julia")]
use xiuxian_wendao_studio::bin_support::flightsql_server::run_search_flightsql_server;

#[cfg(feature = "julia")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_search_flightsql_server().await
}

#[cfg(not(feature = "julia"))]
fn main() {
    eprintln!("wendao_search_flightsql_server requires the `julia` feature");
    std::process::exit(1);
}
