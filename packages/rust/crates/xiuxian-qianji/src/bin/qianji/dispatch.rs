use std::env;

use xiuxian_logging::{init, split_logging_args};

use super::bpmn_cli::{handle_bpmn_command, parse_bpmn_command};
use super::contract_feedback_cli::{
    handle_contract_feedback_command, parse_contract_feedback_command,
};
use super::dir_cli::{handle_dir_command, parse_dir_command};
use super::graph_export::handle_graph_export;
use super::lint_cli::{handle_lint_command, parse_lint_command};
use super::manifest_exec::run_manifest_execution;
use super::template_cli::{handle_template_command, parse_template_command};
use super::usage::print_qianji_usage;

pub(crate) async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = env::args().collect();
    let (log_settings, args) = split_logging_args(&raw_args);
    init("xiuxian_qianji", &log_settings)?;

    if let Some(command) = parse_dir_command(&args)? {
        return handle_dir_command(command);
    }

    if let Some(command) = parse_bpmn_command(&args)? {
        return handle_bpmn_command(command).await;
    }

    if args.len() >= 4 && args[1] == "graph" {
        return handle_graph_export(&args[2], &args[3]);
    }

    if let Some(command) = parse_contract_feedback_command(&args)? {
        return handle_contract_feedback_command(command).await;
    }

    if let Some(command) = parse_lint_command(&args)? {
        return handle_lint_command(command);
    }

    if let Some(command) = parse_template_command(&args)? {
        handle_template_command(&command);
        return Ok(());
    }

    if args.len() < 4 {
        print_qianji_usage();
        std::process::exit(1);
    }

    run_manifest_execution(&args).await
}
