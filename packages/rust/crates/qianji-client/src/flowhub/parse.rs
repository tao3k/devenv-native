//! Argument parser for `qianji-client flowhub`.

use std::env;
use std::path::{Path, PathBuf};

use crate::QianjiClientError;

/// Qianji client Flowhub action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowhubAction {
    /// Materialize the agent planning surface.
    Init,
    /// Validate the generated agent planning surface.
    Check,
    /// List Flowhub Org+BPMN source pairs.
    Scenarios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlowhubOutputFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClientCommand {
    Flowhub(FlowhubCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubCommand {
    pub(crate) action: FlowhubAction,
    pub(crate) project_root: PathBuf,
    pub(crate) cache_home: PathBuf,
    pub(crate) flowhub_root: Option<PathBuf>,
    pub(crate) mode: String,
    pub(crate) scenario: String,
    pub(crate) slug: String,
    pub(crate) output_format: FlowhubOutputFormat,
}

pub(crate) fn parse_client_command(args: &[String]) -> Result<ClientCommand, QianjiClientError> {
    match args.get(1).map(String::as_str) {
        Some("flowhub") => parse_flowhub_command(args),
        Some("check") => {
            let rewritten = rewrite_default_flowhub_action(args, "check");
            parse_flowhub_command(&rewritten)
        }
        Some("init") => {
            let rewritten = rewrite_default_flowhub_action(args, "init");
            parse_flowhub_command(&rewritten)
        }
        Some("--help" | "-h") | None => Err(QianjiClientError::message(usage())),
        Some(other) => Err(QianjiClientError::message(format!(
            "unsupported qianji-client command `{other}`\n\n{}",
            usage()
        ))),
    }
}

fn parse_flowhub_command(args: &[String]) -> Result<ClientCommand, QianjiClientError> {
    let cwd = env::current_dir().map_err(|error| {
        QianjiClientError::message(format!("Failed to resolve current directory: {error}"))
    })?;
    let mut mode = None;
    let mut scenario = None;
    let mut project_root = None;
    let mut cache_home = None;
    let mut flowhub_root = None;
    let mut slug = None;
    let mut output_format = FlowhubOutputFormat::Markdown;
    let mut action = None;
    let mut index = 2;

    while index < args.len() {
        match args[index].as_str() {
            "init" => set_action(&mut action, FlowhubAction::Init)?,
            "check" => set_action(&mut action, FlowhubAction::Check)?,
            "scenarios" | "list" => set_action(&mut action, FlowhubAction::Scenarios)?,
            "--mode" => mode = Some(parse_value(args, &mut index, "--mode")?),
            "--scenario" | "--Scenario" => {
                scenario = Some(parse_value(args, &mut index, "--scenario")?);
            }
            "--project-root" => {
                project_root = Some(PathBuf::from(parse_value(
                    args,
                    &mut index,
                    "--project-root",
                )?));
            }
            "--cache-home" => {
                cache_home = Some(PathBuf::from(parse_value(
                    args,
                    &mut index,
                    "--cache-home",
                )?));
            }
            "--flowhub-root" => {
                flowhub_root = Some(PathBuf::from(parse_value(
                    args,
                    &mut index,
                    "--flowhub-root",
                )?));
            }
            "--slug" => slug = Some(parse_value(args, &mut index, "--slug")?),
            "--json" => output_format = FlowhubOutputFormat::Json,
            "--help" | "-h" => return Err(QianjiClientError::message(usage())),
            other => {
                return Err(QianjiClientError::message(format!(
                    "unsupported qianji-client flowhub argument `{other}`\n\n{}",
                    usage()
                )));
            }
        }
        index += 1;
    }

    let action = action.unwrap_or(FlowhubAction::Check);
    let mode = mode.unwrap_or_else(|| "plan".to_string());
    let scenario = scenario.unwrap_or_else(|| "agent-coding".to_string());
    if mode != "plan" {
        return Err(QianjiClientError::message(format!(
            "unsupported qianji-client flowhub mode `{mode}`; only `plan` is supported"
        )));
    }

    let project_root = resolve_project_root(project_root, &cwd);
    let cache_home = resolve_cache_home(cache_home, &project_root);
    let flowhub_root = resolve_optional_path(flowhub_root, &cwd)
        .or_else(|| resolve_env_path("QIANJI_FLOWHUB_ROOT", &cwd))
        .or_else(|| {
            let candidate = project_root.join("qianji-flowhub");
            candidate.is_dir().then_some(candidate)
        });

    Ok(ClientCommand::Flowhub(FlowhubCommand {
        action,
        project_root,
        cache_home,
        flowhub_root,
        mode,
        slug: slug.unwrap_or_else(|| scenario.clone()),
        scenario,
        output_format,
    }))
}

fn rewrite_default_flowhub_action(args: &[String], action: &str) -> Vec<String> {
    let mut rewritten = Vec::with_capacity(args.len() + 1);
    if let Some(binary) = args.first() {
        rewritten.push(binary.clone());
    }
    rewritten.push("flowhub".to_string());
    rewritten.push(action.to_string());
    rewritten.extend(args.iter().skip(2).cloned());
    rewritten
}

fn set_action(
    action: &mut Option<FlowhubAction>,
    value: FlowhubAction,
) -> Result<(), QianjiClientError> {
    if action.replace(value).is_some() {
        return Err(QianjiClientError::message(
            "qianji-client flowhub accepts exactly one action: `init`, `check`, or `scenarios`"
                .to_string(),
        ));
    }
    Ok(())
}

fn parse_value(
    args: &[String],
    index: &mut usize,
    flag: &str,
) -> Result<String, QianjiClientError> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| QianjiClientError::message(format!("missing value for {flag}")))
}

fn resolve_project_root(explicit: Option<PathBuf>, cwd: &Path) -> PathBuf {
    if let Some(project_root) = explicit {
        return resolve_path(project_root, cwd);
    }
    if let Some(project_root) = resolve_env_path("PRJ_ROOT", cwd) {
        return project_root;
    }
    cwd.to_path_buf()
}

fn resolve_cache_home(explicit: Option<PathBuf>, project_root: &Path) -> PathBuf {
    if let Some(cache_home) = explicit {
        return resolve_path(cache_home, project_root);
    }
    if let Some(cache_home) = resolve_env_path("PRJ_CACHE_HOME", project_root) {
        return cache_home;
    }
    project_root.join(".cache")
}

fn resolve_env_path(name: &str, base: &Path) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .map(|path| resolve_path(path, base))
}

fn resolve_optional_path(path: Option<PathBuf>, base: &Path) -> Option<PathBuf> {
    path.map(|path| resolve_path(path, base))
}

fn resolve_path(path: PathBuf, base: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn usage() -> String {
    "Usage: qianji-client flowhub --mode plan --scenario <id> init [--json]\n       qianji-client flowhub check [--json]\n       qianji-client flowhub scenarios [--json]\n       qianji-client check [--json]"
        .to_string()
}
