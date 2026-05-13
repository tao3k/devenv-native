use std::io;
use std::path::PathBuf;

use super::types::{DirCliCommand, MaterializeCliTarget, ShowCliTarget};
use crate::qianji_cli::input::invalid_input;

pub(crate) fn parse_dir_command(args: &[String]) -> io::Result<Option<DirCliCommand>> {
    match args.get(1).map(String::as_str) {
        Some("show") => Ok(Some(DirCliCommand::Show {
            target: parse_show_target(&args[2..])?,
        })),
        Some("check") => Ok(Some(DirCliCommand::Check {
            dir: parse_dir_flag(&args[2..], "check")?,
        })),
        Some("materialize") => Ok(Some(DirCliCommand::Materialize {
            target: parse_materialize_target(&args[2..])?,
        })),
        Some("advance") => {
            let (dir, to) = parse_advance_command(&args[2..])?;
            Ok(Some(DirCliCommand::Advance { dir, to }))
        }
        _ => Ok(None),
    }
}

fn parse_show_target(args: &[String]) -> io::Result<ShowCliTarget> {
    let mut index = 0;
    let mut dir = None;
    let mut graph = None;
    let mut contract = None;
    let mut anchor = None;
    let mut scenario = None;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --dir in `show` command"))?;
                dir = Some(PathBuf::from(value));
            }
            "--graph" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --graph in `show` command"))?;
                graph = Some(PathBuf::from(value));
            }
            "--contract" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --contract in `show` command")
                })?;
                contract = Some(value.clone());
            }
            "--anchor" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --anchor in `show` command"))?;
                anchor = Some(PathBuf::from(value));
            }
            "--scenario" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --scenario in `show` command")
                })?;
                scenario = Some(value.clone());
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `show` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    match (anchor, scenario, dir, graph, contract) {
        (Some(anchor), Some(scenario), dir, None, None) => Ok(ShowCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
        }),
        (Some(_), None, _, None, None) | (None, Some(_), _, None, None) => Err(invalid_input(
            "`show --anchor` requires `--scenario <ref>` and `show --scenario` requires `--anchor <path>`",
        )),
        (None, None, Some(dir), None, None) => Ok(ShowCliTarget::Dir(dir)),
        (None, None, None, Some(graph), None) => Ok(ShowCliTarget::Graph(graph)),
        (None, None, None, None, Some(contract_name)) => Ok(ShowCliTarget::Contract(contract_name)),
        (None, None, None, None, None) => Err(invalid_input(
            "missing `--dir <path>`, `--graph <path>`, `--contract <name>`, or `--anchor <path> --scenario <ref>` for `show` command",
        )),
        _ => Err(invalid_input(
            "`show` command requires exactly one of `--dir <path>`, `--graph <path>`, `--contract <name>`, or `--anchor <path> --scenario <ref>`",
        )),
    }
}

fn parse_dir_flag(args: &[String], command: &str) -> io::Result<PathBuf> {
    let mut index = 0;
    let mut dir = None;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input(format!("missing value for --dir in `{command}` command"))
                })?;
                dir = Some(PathBuf::from(value));
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `{command}` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    dir.ok_or_else(|| invalid_input(format!("missing `--dir <path>` for `{command}` command")))
}

fn parse_materialize_target(args: &[String]) -> io::Result<MaterializeCliTarget> {
    let mut index = 0;
    let mut anchor = None;
    let mut scenario = None;
    let mut dir = None;
    let mut current_node = None;
    while index < args.len() {
        match args[index].as_str() {
            "--anchor" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --anchor in `materialize` command")
                })?;
                anchor = Some(PathBuf::from(value));
            }
            "--scenario" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --scenario in `materialize` command")
                })?;
                scenario = Some(value.clone());
            }
            "--dir" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --dir in `materialize` command")
                })?;
                dir = Some(PathBuf::from(value));
            }
            "--current-node" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    invalid_input("missing value for --current-node in `materialize` command")
                })?;
                current_node = Some(value.clone());
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `materialize` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    match (anchor, scenario, dir) {
        (Some(anchor), Some(scenario), Some(dir)) => Ok(MaterializeCliTarget::AnchoredScenario {
            anchor,
            scenario,
            dir,
            current_node,
        }),
        _ => Err(invalid_input(
            "missing `--anchor <path> --scenario <ref> --dir <path>` for `materialize` command",
        )),
    }
}

fn parse_advance_command(args: &[String]) -> io::Result<(PathBuf, String)> {
    let mut index = 0;
    let mut dir = None;
    let mut to = None;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --dir in `advance` command"))?;
                dir = Some(PathBuf::from(value));
            }
            "--to" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("missing value for --to in `advance` command"))?;
                to = Some(value.clone());
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `advance` option `{other}`"
                )));
            }
        }
        index += 1;
    }

    match (dir, to) {
        (Some(dir), Some(to)) => Ok((dir, to)),
        _ => Err(invalid_input(
            "missing `--dir <path> --to <node>` for `advance` command",
        )),
    }
}
