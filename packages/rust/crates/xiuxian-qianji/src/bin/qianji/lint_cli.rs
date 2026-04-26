use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use qianji_bpmn_engine::{
    BpmnSourceFile, DmnSourceFile, LintDomain, LintIssue, LintReport, lint_bpmn_source,
    lint_dmn_source,
};
use xiuxian_qianji::{
    WorkflowPlan, WorkflowPlanValidationReport, render_workflow_plan_validation_report,
    validate_workflow_plan,
};

use super::{invalid_input, parse_flag_value, resolve_cli_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LintCliCommand {
    Auto { path: PathBuf },
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
        LintCliCommand::Auto { path } => {
            let resolved = resolve_cli_path(&path)?;
            let contents = fs::read_to_string(&resolved)?;
            run_inferred_lint(&path, &resolved, contents)
        }
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
    let mut positional_path = None;
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
            other if other.starts_with('-') => {
                return Err(invalid_input(format!(
                    "unsupported `lint` option `{other}`"
                )));
            }
            other => {
                if positional_path.is_some() {
                    return Err(invalid_input(format!(
                        "`lint` command accepts only one positional path; unexpected `{other}`"
                    )));
                }
                positional_path = Some(PathBuf::from(other));
            }
        }

        index += 1;
    }

    match (positional_path, bpmn, dmn) {
        (Some(path), None, None) => Ok(Some(LintCliCommand::Auto { path })),
        (None, Some(path), None) => Ok(Some(LintCliCommand::Bpmn { path })),
        (None, None, Some(path)) => Ok(Some(LintCliCommand::Dmn { path })),
        (None, None, None) => Err(invalid_input("missing path for `lint` command")),
        _ => Err(invalid_input(
            "`lint` command requires exactly one target path",
        )),
    }
}

fn run_inferred_lint(
    source_path: &Path,
    resolved_path: &Path,
    contents: String,
) -> io::Result<LintCliOutput> {
    match source_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("bpmn") => {
            let report = lint_bpmn_source(&BpmnSourceFile::new(
                source_path.display().to_string(),
                contents,
            ));
            Ok(render_lint_output(&report, resolved_path))
        }
        Some("dmn") => {
            let report = lint_dmn_source(&DmnSourceFile::new(
                source_path.display().to_string(),
                contents,
            ));
            Ok(render_lint_output(&report, resolved_path))
        }
        Some("json") => {
            let source_id = source_path.display().to_string();
            match parse_workflow_plan(&contents) {
                Ok(plan) => {
                    let report = validate_workflow_plan(&plan);
                    Ok(render_workflow_plan_lint_output(
                        &report,
                        &source_id,
                        resolved_path,
                    ))
                }
                Err(error) => Ok(render_workflow_plan_parse_error(
                    &source_id,
                    resolved_path,
                    &error,
                )),
            }
        }
        _ => Err(invalid_input(format!(
            "cannot infer lint target for {}; use a .bpmn, .dmn, or WorkflowPlan .json file",
            source_path.display()
        ))),
    }
}

fn parse_workflow_plan(contents: &str) -> serde_json::Result<WorkflowPlan> {
    serde_json::from_str(contents)
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

fn render_workflow_plan_lint_output(
    report: &WorkflowPlanValidationReport,
    source_id: &str,
    resolved_path: &Path,
) -> LintCliOutput {
    if report.ok {
        return LintCliOutput {
            rendered: format!(
                "# Lint Passed\n\nSource: {source_id}\nPath: {}\nDomain: workflow-plan\nStatus: no blocking issues found in the bounded lint contract.\n",
                resolved_path.display(),
            ),
            exit_code: 0,
        };
    }

    LintCliOutput {
        rendered: format!(
            "# Lint Failed\n\nSource: {source_id}\nPath: {}\nDomain: workflow-plan\nIssues: {}\n\n{}",
            resolved_path.display(),
            report.diagnostics.len(),
            render_workflow_plan_validation_report(report),
        ),
        exit_code: 2,
    }
}

fn render_workflow_plan_parse_error(
    source_id: &str,
    resolved_path: &Path,
    error: &serde_json::Error,
) -> LintCliOutput {
    let rendered = format!(
        "# Lint Failed\n\nSource: {source_id}\nPath: {}\nDomain: workflow-plan\nIssues: 1\n\n",
        resolved_path.display(),
    );
    LintCliOutput {
        rendered: format!("{rendered}{}", workflow_plan_parse_error_body(error)),
        exit_code: 2,
    }
}

fn workflow_plan_parse_error_body(error: &serde_json::Error) -> String {
    format!(
        "## [construct_plan.invalid_json_shape] WorkflowPlan JSON shape is invalid\nSeverity: error\nSummary: failed to parse WorkflowPlan JSON: {error}\n\n### Repair Guidance\n- Emit one top-level WorkflowPlan object, not a wrapper such as `plan`.\n- Use numeric `\"version\": 1`, not string `\"1\"`.\n- Use `constructs`, `tasks`, and `edges`; do not use `nodes` or BPMN element names as the IR shape.\n- Each task must use `construct`, not `type`, and the construct value must come from `qianji construct index`.\n- Treat `constructs` as a set: list each selected construct id once.\n\n### Minimal Shape\n```json\n{{\n  \"version\": 1,\n  \"name\": \"example-plan\",\n  \"constructs\": [\"service-task.agent\"],\n  \"tasks\": [\n    {{\"id\": \"Task_DoWork\", \"construct\": \"service-task.agent\", \"outputs\": [\"result\"]}}\n  ],\n  \"edges\": [\n    {{\"from\": \"start\", \"to\": \"Task_DoWork\"}},\n    {{\"from\": \"Task_DoWork\", \"to\": \"end\"}}\n  ]\n}}\n```\n"
    )
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
