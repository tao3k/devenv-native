//! Process entrypoint for the `wendao-episteme` operator CLI.

fn main() -> anyhow::Result<()> {
    xiuxian_wendao_episteme::cli::wendao_episteme::run_from_env()
}
