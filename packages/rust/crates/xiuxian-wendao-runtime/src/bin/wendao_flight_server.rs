//! Runtime-owned Arrow Flight server binary for the stable Wendao query and rerank routes.

#[cfg(not(feature = "transport"))]
use std::io;

#[cfg(not(feature = "transport"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err(io::Error::other("`wendao_flight_server` requires the `transport` feature").into())
}

#[cfg(feature = "transport")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    xiuxian_wendao_runtime::transport::run_wendao_flight_server_from_args(std::env::args().skip(1))
        .await
}
