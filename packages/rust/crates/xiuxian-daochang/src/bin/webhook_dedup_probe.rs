//! Process-level webhook dedup probe server used by integration tests.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    xiuxian_daochang::webhook_dedup_probe::run_from_cli().await
}
