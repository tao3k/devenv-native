//! Runtime-backed Wendao Flight server binary that reads repo-search data from
//! the active search-plane store.

#[cfg(feature = "zhenfa-router")]
use xiuxian_wendao::bin_support::flight_server::run_search_flight_server;

#[cfg(feature = "zhenfa-router")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_search_flight_server().await
}

#[cfg(not(feature = "zhenfa-router"))]
fn main() {
    eprintln!("wendao_search_flight_server requires the `zhenfa-router` feature");
    std::process::exit(1);
}
