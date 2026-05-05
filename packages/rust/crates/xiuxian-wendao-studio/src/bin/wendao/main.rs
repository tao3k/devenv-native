//! Command-line interface entrypoint for xiuxian-wendao link-graph operations.

use xiuxian_wendao_studio::bin_support::wendao::run_wendao;

fn main() -> anyhow::Result<()> {
    run_wendao()
}
