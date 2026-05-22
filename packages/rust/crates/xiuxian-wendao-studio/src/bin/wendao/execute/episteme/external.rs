use std::{path::Path, process::Command as ProcessCommand};

use anyhow::{Context, Result};
use serde::Serialize;

use super::root::path_display;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeExternalCommandSpec {
    pub(super) program: String,
    pub(super) args: Vec<String>,
    pub(super) current_dir: Option<String>,
}

pub(super) fn image_ocr_analyzer_command_spec(
    analyzer_command: &str,
    episteme_root: &Path,
    tasks_path: &Path,
    corpus_root: &Path,
    ocr_results_jsonl: &Path,
) -> EpistemeExternalCommandSpec {
    EpistemeExternalCommandSpec {
        program: analyzer_command.to_string(),
        args: vec![
            "--tasks".to_string(),
            path_display(tasks_path),
            "--corpus-root".to_string(),
            path_display(corpus_root),
            "--output-jsonl".to_string(),
            path_display(ocr_results_jsonl),
        ],
        current_dir: Some(path_display(episteme_root)),
    }
}

pub(super) fn docling_document_analyzer_command_spec(
    analyzer_command: &str,
    episteme_root: &Path,
    tasks_path: &Path,
    corpus_root: &Path,
    document_results_jsonl: &Path,
    docling_profile: &str,
) -> EpistemeExternalCommandSpec {
    EpistemeExternalCommandSpec {
        program: analyzer_command.to_string(),
        args: vec![
            "--tasks".to_string(),
            path_display(tasks_path),
            "--corpus-root".to_string(),
            path_display(corpus_root),
            "--output-jsonl".to_string(),
            path_display(document_results_jsonl),
            "--profile".to_string(),
            docling_profile.to_string(),
        ],
        current_dir: Some(path_display(episteme_root)),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeExternalCommandReport {
    pub(super) command: EpistemeExternalCommandSpec,
    pub(super) skipped: bool,
    pub(super) exit_code: Option<i32>,
}

pub(super) fn run_external_command_if_needed(
    skip_command: bool,
    spec: &EpistemeExternalCommandSpec,
    label: &str,
) -> Result<Option<i32>> {
    if skip_command {
        return Ok(None);
    }
    run_external_command(spec, label).map(Some)
}

pub(super) fn should_skip_analyzer(dry_run: bool, use_existing_results: bool) -> bool {
    dry_run || use_existing_results
}

fn run_external_command(spec: &EpistemeExternalCommandSpec, label: &str) -> Result<i32> {
    let mut command = ProcessCommand::new(&spec.program);
    command.args(&spec.args);
    if let Some(current_dir) = &spec.current_dir {
        command.current_dir(current_dir);
    }
    let status = command
        .status()
        .with_context(|| format!("failed to start {label} command `{}`", spec.program))?;
    let exit_code = status.code().unwrap_or(1);
    if !status.success() {
        anyhow::bail!("{label} command failed with exit code {exit_code}");
    }
    Ok(exit_code)
}
