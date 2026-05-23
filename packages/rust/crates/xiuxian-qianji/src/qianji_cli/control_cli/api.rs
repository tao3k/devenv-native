use std::io;

use super::parse::parse_control_command_impl;
use super::run::run_control_command_impl;
use super::types::{ControlCliCommand, ControlCliOutput};

pub(crate) fn parse_control_command(args: &[String]) -> io::Result<Option<ControlCliCommand>> {
    parse_control_command_impl(args)
}

pub(crate) fn handle_control_command(command: &ControlCliCommand) -> io::Result<()> {
    let output = run_control_command(command)?;
    println!("{}", output.rendered);
    Ok(())
}

pub(crate) async fn handle_control_command_async(command: ControlCliCommand) -> io::Result<()> {
    tokio::task::spawn_blocking(move || handle_control_command(&command))
        .await
        .map_err(|error| io::Error::other(format!("control command task failed: {error}")))?
}

pub(crate) fn run_control_command(command: &ControlCliCommand) -> io::Result<ControlCliOutput> {
    run_control_command_impl(command)
}
