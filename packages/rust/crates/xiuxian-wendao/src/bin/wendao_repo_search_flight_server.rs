//! Thin entrypoint for the domain-owned `SearchStrategyFlow` Flight host.

#[tokio::main]
async fn main() -> xiuxian_wendao::flight_host::FlightHostResult<()> {
    xiuxian_wendao::flight_host::run_repo_search_flight_server_from_args(std::env::args().skip(1))
        .await
}
