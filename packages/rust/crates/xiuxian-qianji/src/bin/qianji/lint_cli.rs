use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use qianji_bpmn_engine::{
    BpmnSourceFile, DmnSourceFile, LintDomain, LintIssue, LintReport, lint_bpmn_source,
    lint_dmn_source,
};

use super::{invalid_input, parse_flag_value, resolve_cli_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LintCliCommand {
    Bpmn { path: PathBuf },
    Dmn { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LintCliOutput {
    pub(crate) rendered: String,
    pub(crate) exit_code: i32,
}

pub(super) fn handle_lint_command(
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

pub(super) fn run_lint_command(command: LintCliCommand) -> io::Result<LintCliOutput> {
    match command {
        LintCliCommand::Bpmn { path } => {
            let resolved = resolve_cli_path(&path)?;
            let contents = fs::read_to_string(&resolved)?;
            let report =
                lint_bpmn_source(&BpmnSourceFile::new(path.display().to_string(), contents));
            Ok(render_lint_output(&report, &resolved))
        }
        LintCliCommand::Dmn { path } => {
            let resolved = resolve_cli_path(&path)?;
            let contents = fs::read_to_string(&resolved)?;
            let report = lint_dmn_source(&DmnSourceFile::new(path.display().to_string(), contents));
            Ok(render_lint_output(&report, &resolved))
        }
    }
}

pub(super) fn parse_lint_command(args: &[String]) -> io::Result<Option<LintCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };

    if command_name != "lint" && command_name != "linter" {
        return Ok(None);
    }

    let mut index = 2;
    let mut bpmn = None;
    let mut dmn = None;
    while index < args.len() {
        match args[index].as_str() {
            "--bpmn" => {
                bpmn = Some(PathBuf::from(parse_flag_value(args, &mut index, "--bpmn")?));
            }
            "--dmn" => {
                dmn = Some(PathBuf::from(parse_flag_value(args, &mut index, "--dmn")?));
            }
            other => {
                return Err(invalid_input(format!(
                    "unsupported `lint` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    match (bpmn, dmn) {
        (Some(path), None) => Ok(Some(LintCliCommand::Bpmn { path })),
        (None, Some(path)) => Ok(Some(LintCliCommand::Dmn { path })),
        (None, None) => Err(invalid_input(
            "missing `--bpmn <path>` or `--dmn <path>` for `lint` command",
        )),
        _ => Err(invalid_input(
            "`lint` command requires exactly one of `--bpmn <path>` or `--dmn <path>`",
        )),
    }
}

fn render_lint_output(report: &LintReport, resolved_path: &Path) -> LintCliOutput {
    if report.ok {
        return LintCliOutput {
            rendered: format!(
                "# Lint Passed\n\nSource: {}\nPath: {}\nDomain: {}\nStatus: no blocking issues found in the bounded lint contract.\n",
                report.source_id,
                resolved_path.display(),
                lint_domain_name(&report.domain),
            ),
            exit_code: 0,
        };
    }

    let mut rendered = format!(
        "# Lint Failed\n\nSource: {}\nPath: {}\nDomain: {}\nIssues: {}\n",
        report.source_id,
        resolved_path.display(),
        lint_domain_name(&report.domain),
        report.issues.len(),
    );

    for issue in &report.issues {
        append_issue_markdown(&mut rendered, issue);
    }

    LintCliOutput {
        rendered,
        exit_code: 2,
    }
}

fn lint_domain_name(domain: &LintDomain) -> &'static str {
    match domain {
        LintDomain::Bpmn => "bpmn",
        LintDomain::Dmn => "dmn",
    }
}

fn append_issue_markdown(rendered: &mut String, issue: &LintIssue) {
    let _ = writeln!(rendered, "\n## [{}] {}", issue.code, issue.title);
    let _ = writeln!(rendered, "Severity: error");
    let _ = writeln!(rendered, "Summary: {}", issue.summary);
    let _ = writeln!(rendered, "\n### Why It Failed");
    let _ = writeln!(rendered, "{}", issue.why_it_failed);
    let _ = writeln!(rendered, "\n### Repair Guidance");
    for step in &issue.repair_guidance {
        let _ = writeln!(rendered, "- {step}");
    }
    let _ = writeln!(rendered, "\n### LLM Fix Prompt");
    let _ = writeln!(rendered, "{}", issue.llm_fix_prompt);
    let _ = writeln!(rendered, "\n### Evidence");
    let _ = writeln!(rendered, "```json");
    let evidence = serde_json::to_string_pretty(&issue.evidence)
        .unwrap_or_else(|_error| "{\"error\":\"failed to render lint evidence\"}".to_string());
    let _ = writeln!(rendered, "{evidence}");
    let _ = writeln!(rendered, "```");
}
