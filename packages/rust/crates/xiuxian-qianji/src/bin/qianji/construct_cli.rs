use std::io;

use xiuxian_qianji::{
    construct_cards, find_construct_card, render_construct_card, render_construct_card_json,
    render_construct_index, render_construct_index_json,
};

use super::invalid_input;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConstructCliCommand {
    Index { json: bool },
    Show { id: String, json: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstructCliOutput {
    pub(crate) rendered: String,
}

pub(super) fn handle_construct_command(command: &ConstructCliCommand) -> io::Result<()> {
    let output = run_construct_command(command)?;
    println!("{}", output.rendered);
    Ok(())
}

pub(super) fn run_construct_command(
    command: &ConstructCliCommand,
) -> io::Result<ConstructCliOutput> {
    let rendered = match command {
        ConstructCliCommand::Index { json } => {
            if *json {
                render_construct_index_json(construct_cards()).map_err(json_error)?
            } else {
                render_construct_index(construct_cards())
            }
        }
        ConstructCliCommand::Show { id, json } => {
            let Some(card) = find_construct_card(id) else {
                return Err(invalid_input(format!(
                    "unknown qianji construct `{id}`; available constructs: {}",
                    available_construct_ids()
                )));
            };
            if *json {
                render_construct_card_json(card).map_err(json_error)?
            } else {
                render_construct_card(card)
            }
        }
    };
    Ok(ConstructCliOutput { rendered })
}

pub(super) fn parse_construct_command(args: &[String]) -> io::Result<Option<ConstructCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };
    if command_name != "construct" {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("index") => {
            let json = parse_optional_json_flag(args, 3, "`construct index`")?;
            Ok(Some(ConstructCliCommand::Index { json }))
        }
        Some("show") => {
            let Some(id) = args.get(3) else {
                return Err(invalid_input("missing construct id for `construct show`"));
            };
            let json = parse_optional_json_flag(args, 4, "`construct show <id>`")?;
            Ok(Some(ConstructCliCommand::Show {
                id: id.clone(),
                json,
            }))
        }
        Some(other) => Err(invalid_input(format!(
            "unsupported `construct` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `construct` subcommand; expected `index` or `show <id>`",
        )),
    }
}

fn parse_optional_json_flag(args: &[String], start: usize, context: &str) -> io::Result<bool> {
    let mut json = false;
    for value in &args[start..] {
        match value.as_str() {
            "--json" => json = true,
            other => {
                return Err(invalid_input(format!(
                    "{context} does not accept argument `{other}`"
                )));
            }
        }
    }
    Ok(json)
}

fn json_error(error: serde_json::Error) -> io::Error {
    io::Error::other(error)
}

fn available_construct_ids() -> String {
    construct_cards()
        .iter()
        .map(|card| card.id)
        .collect::<Vec<_>>()
        .join(", ")
}
