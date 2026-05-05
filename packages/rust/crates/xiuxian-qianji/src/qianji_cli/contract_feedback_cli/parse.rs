use std::io;
use std::path::PathBuf;

use super::types::{
    ContractFeedbackCliCommand, DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME, RestDocsCliCommand,
};
use crate::qianji_cli::common::{invalid_input, parse_flag_value};

pub(crate) fn parse_contract_feedback_command(
    args: &[String],
) -> io::Result<Option<ContractFeedbackCliCommand>> {
    if args.get(1).map(String::as_str) != Some("contract-feedback") {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("rest-docs") => Ok(Some(ContractFeedbackCliCommand::RestDocs(
            parse_rest_docs_cli_command(&args[3..])?,
        ))),
        Some(other) => Err(invalid_input(format!(
            "unsupported contract-feedback subcommand '{other}'"
        ))),
        None => Err(invalid_input(
            "missing contract-feedback subcommand; expected 'rest-docs'",
        )),
    }
}

fn parse_rest_docs_cli_command(args: &[String]) -> io::Result<RestDocsCliCommand> {
    let Some(openapi_path) = args.first() else {
        return Err(invalid_input(
            "missing OpenAPI path; expected 'contract-feedback rest-docs <openapi_path>'",
        ));
    };

    let mut command = RestDocsCliCommand {
        openapi_path: PathBuf::from(openapi_path),
        workspace_root: None,
        storage_path: None,
        table_name: DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME.to_string(),
        no_persist: false,
        live_advisory: false,
        roles: Vec::new(),
        model: None,
        temperature: None,
        cognitive_early_halt_threshold: None,
    };

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--workspace-root" => {
                command.workspace_root = Some(PathBuf::from(parse_flag_value(
                    args,
                    &mut index,
                    "--workspace-root",
                )?));
            }
            "--storage-path" => {
                command.storage_path = Some(PathBuf::from(parse_flag_value(
                    args,
                    &mut index,
                    "--storage-path",
                )?));
            }
            "--table-name" => {
                command.table_name = parse_flag_value(args, &mut index, "--table-name")?;
            }
            "--role" => {
                command
                    .roles
                    .push(parse_flag_value(args, &mut index, "--role")?);
            }
            "--model" => {
                command.model = Some(parse_flag_value(args, &mut index, "--model")?);
            }
            "--temperature" => {
                let raw = parse_flag_value(args, &mut index, "--temperature")?;
                command.temperature = Some(raw.parse::<f32>().map_err(|error| {
                    invalid_input(format!(
                        "failed to parse --temperature value '{raw}' as f32: {error}"
                    ))
                })?);
            }
            "--cognitive-threshold" => {
                let raw = parse_flag_value(args, &mut index, "--cognitive-threshold")?;
                command.cognitive_early_halt_threshold =
                    Some(raw.parse::<f32>().map_err(|error| {
                        invalid_input(format!(
                            "failed to parse --cognitive-threshold value '{raw}' as f32: {error}"
                        ))
                    })?);
            }
            "--no-persist" => {
                command.no_persist = true;
            }
            "--live-advisory" => {
                command.live_advisory = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported contract-feedback option '{other}'"
                )));
            }
        }

        index += 1;
    }

    Ok(command)
}
