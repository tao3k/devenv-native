//! Integration tests for `ToolsScanner`.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_skills::{SkillScanner, ToolRecord, ToolsScanner};

mod annotations;
mod decorator_kwargs;
mod discovery;
mod enrichment;
mod parameters;
mod serialization;
mod structure_scan;

pub(super) type BoxError = Box<dyn std::error::Error>;
pub(super) type TestResult = Result<(), BoxError>;

pub(super) fn create_scripts_dir(
    temp_dir: &TempDir,
    skill_name: &str,
) -> Result<PathBuf, BoxError> {
    let scripts_dir = temp_dir.path().join(format!("{skill_name}/scripts"));
    fs::create_dir_all(&scripts_dir)?;
    Ok(scripts_dir)
}

pub(super) fn write_script(
    scripts_dir: &Path,
    file_name: &str,
    contents: &str,
) -> Result<PathBuf, BoxError> {
    let script_path = scripts_dir.join(file_name);
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&script_path, contents)?;
    Ok(script_path)
}

pub(super) fn scan_scripts(
    scripts_dir: &Path,
    skill_name: &str,
    routing_keywords: &[String],
) -> Result<Vec<ToolRecord>, BoxError> {
    ToolsScanner::new().scan_scripts(scripts_dir, skill_name, routing_keywords, &[])
}

pub(super) fn scan_with_structure(
    skill_path: &Path,
    skill_name: &str,
    routing_keywords: &[String],
) -> Result<Vec<ToolRecord>, BoxError> {
    let structure = SkillScanner::default_structure();
    ToolsScanner::new().scan_with_structure(
        skill_path,
        skill_name,
        routing_keywords,
        &[],
        &structure,
    )
}
