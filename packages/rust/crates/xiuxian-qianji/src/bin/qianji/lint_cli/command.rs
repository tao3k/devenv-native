use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use qianji_bpmn_engine::{BpmnSourceFile, DmnSourceFile, lint_bpmn_source, lint_dmn_source};

use super::bpmn_json::render_bpmn_lint_json_output;
use super::llm::render_lint_llm_output;
use super::render::render_lint_json_output;
use super::workflow_plan::run_workflow_plan_lint;
use crate::{invalid_input, parse_flag_value, resolve_cli_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LintCliCommand {
    Auto { path: PathBuf },
    AutoJson { path: PathBuf },
    Bpmn { path: PathBuf },
    BpmnJson { path: PathBuf },
    Dmn { path: PathBuf },
    DmnJson { path: PathBuf },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum LintOutputFormat {
    Json,
    #[default]
    Llm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LintCliOutput {
    pub(crate) rendered: String,
    pub(crate) exit_code: i32,
}

pub(crate) fn handle_lint_command(
    command: LintCliCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_lint_command(command)?;
    println!("{}", output.rendered);
    if output.exit_code == 0 {
        Ok(())
    } else {
        std::process::exit(output.exit_code);
    }
}

pub(crate) fn run_lint_command(command: LintCliCommand) -> io::Result<LintCliOutput> {
    match command {
        LintCliCommand::Auto { path } => run_auto_lint(&path, LintOutputFormat::Llm),
        LintCliCommand::AutoJson { path } => run_auto_lint(&path, LintOutputFormat::Json),
        LintCliCommand::Bpmn { path } => run_bpmn_lint(&path, LintOutputFormat::Llm),
        LintCliCommand::BpmnJson { path } => run_bpmn_lint(&path, LintOutputFormat::Json),
        LintCliCommand::Dmn { path } => run_dmn_lint(&path, LintOutputFormat::Llm),
        LintCliCommand::DmnJson { path } => run_dmn_lint(&path, LintOutputFormat::Json),
    }
}

pub(crate) fn parse_lint_command(args: &[String]) -> io::Result<Option<LintCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };

    if command_name != "lint" && command_name != "linter" {
        return Ok(None);
    }

    let parsed = parse_lint_args(args)?;
    match (parsed.positional_path, parsed.bpmn, parsed.dmn) {
        (Some(path), None, None) if parsed.format == LintOutputFormat::Json => {
            Ok(Some(LintCliCommand::AutoJson { path }))
        }
        (Some(path), None, None) => Ok(Some(LintCliCommand::Auto { path })),
        (None, Some(path), None) if parsed.format == LintOutputFormat::Json => {
            Ok(Some(LintCliCommand::BpmnJson { path }))
        }
        (None, Some(path), None) => Ok(Some(LintCliCommand::Bpmn { path })),
        (None, None, Some(path)) if parsed.format == LintOutputFormat::Json => {
            Ok(Some(LintCliCommand::DmnJson { path }))
        }
        (None, None, Some(path)) => Ok(Some(LintCliCommand::Dmn { path })),
        (None, None, None) => Err(invalid_input("missing path for `lint` command")),
        _ => Err(invalid_input(
            "`lint` command requires exactly one target path",
        )),
    }
}

fn run_auto_lint(path: &Path, format: LintOutputFormat) -> io::Result<LintCliOutput> {
    let resolved = resolve_cli_path(path)?;
    let contents = fs::read_to_string(&resolved)?;
    run_inferred_lint(path, &resolved, &contents, format)
}

fn parse_lint_args(args: &[String]) -> io::Result<ParsedLintArgs> {
    let mut parsed = ParsedLintArgs::default();
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => parsed.set_format(LintOutputFormat::Json)?,
            "--llm" => parsed.set_format(LintOutputFormat::Llm)?,
            "--bpmn" => {
                parsed.bpmn = Some(PathBuf::from(parse_flag_value(args, &mut index, "--bpmn")?));
            }
            "--dmn" => {
                parsed.dmn = Some(PathBuf::from(parse_flag_value(args, &mut index, "--dmn")?));
            }
            other if other.starts_with('-') => {
                return Err(invalid_input(format!(
                    "unsupported `lint` option `{other}`"
                )));
            }
            other => {
                if parsed.positional_path.is_some() {
                    return Err(invalid_input(format!(
                        "`lint` command accepts only one positional path; unexpected `{other}`"
                    )));
                }
                parsed.positional_path = Some(PathBuf::from(other));
            }
        }

        index += 1;
    }
    Ok(parsed)
}

fn run_bpmn_lint(path: &Path, format: LintOutputFormat) -> io::Result<LintCliOutput> {
    let resolved = resolve_cli_path(path)?;
    let contents = fs::read_to_string(&resolved)?;
    let report = lint_bpmn_source(&BpmnSourceFile::new(
        path.display().to_string(),
        contents.clone(),
    ));
    match format {
        LintOutputFormat::Json => render_bpmn_lint_json_output(&report, &resolved, &contents),
        LintOutputFormat::Llm => render_lint_llm_output(&report, &resolved, Some(&contents)),
    }
}

fn run_dmn_lint(path: &Path, format: LintOutputFormat) -> io::Result<LintCliOutput> {
    let resolved = resolve_cli_path(path)?;
    let contents = fs::read_to_string(&resolved)?;
    let report = lint_dmn_source(&DmnSourceFile::new(path.display().to_string(), contents));
    match format {
        LintOutputFormat::Json => render_lint_json_output(&report, &resolved),
        LintOutputFormat::Llm => render_lint_llm_output(&report, &resolved, None),
    }
}

fn run_inferred_lint(
    source_path: &Path,
    resolved_path: &Path,
    contents: &str,
    format: LintOutputFormat,
) -> io::Result<LintCliOutput> {
    match source_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("bpmn") => run_bpmn_lint(source_path, format),
        Some("dmn") => run_dmn_lint(source_path, format),
        Some("json") => run_workflow_plan_lint(source_path, resolved_path, contents, format),
        _ => Err(invalid_input(format!(
            "cannot infer lint target for {}; use a .bpmn, .dmn, or WorkflowPlan .json file",
            source_path.display()
        ))),
    }
}

#[derive(Default)]
struct ParsedLintArgs {
    positional_path: Option<PathBuf>,
    bpmn: Option<PathBuf>,
    dmn: Option<PathBuf>,
    format: LintOutputFormat,
    format_explicit: bool,
}

impl ParsedLintArgs {
    fn set_format(&mut self, format: LintOutputFormat) -> io::Result<()> {
        if self.format_explicit && self.format != format {
            return Err(invalid_input(
                "`lint` command accepts only one output format",
            ));
        }
        self.format = format;
        self.format_explicit = true;
        Ok(())
    }
}
