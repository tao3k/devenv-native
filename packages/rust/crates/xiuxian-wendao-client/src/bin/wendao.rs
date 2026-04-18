//! Standalone lightweight `wendao-client` binary backed by `xiuxian-wendao-client`.

use std::process::ExitCode;
use xiuxian_logging::init_from_cli;
use xiuxian_wendao_client::{ClientCli, ClientContext, run_command};

#[allow(clippy::print_stderr)]
fn main() -> ExitCode {
    use clap::Parser;

    let cli = ClientCli::parse();
    if let Err(error) = init_from_cli("xiuxian_wendao_client", &cli.logging) {
        eprintln!("{error}");
        return ExitCode::from(1);
    }

    let context = ClientContext::new(cli.root.as_path(), cli.output);
    match run_command(&cli.command, &context) {
        Ok(outcome) => ExitCode::from(outcome.exit_code()),
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::from(1)
        }
    }
}
