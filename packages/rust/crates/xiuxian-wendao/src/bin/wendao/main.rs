//! Command-line interface entrypoint for xiuxian-wendao link-graph operations.

use xiuxian_wendao::bin_support::wendao::run_wendao;

fn main() -> anyhow::Result<()> {
    run_wendao()
}
