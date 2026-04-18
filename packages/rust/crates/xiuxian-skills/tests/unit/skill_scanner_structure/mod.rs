//! Behavior and structural tests for `SkillScanner`.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
mod scan_all;
mod sniffer_rules;
mod structure;

type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) fn create_skill_dir(temp_dir: &TempDir, name: &str) -> PathBuf {
    let skill_path = temp_dir.path().join(name);
    fs::create_dir_all(&skill_path).unwrap_or_else(|error| panic!("create skill dir: {error}"));
    skill_path
}

pub(super) fn write_skill_md(skill_path: &Path, contents: &str) -> TestResult {
    fs::write(skill_path.join("SKILL.md"), contents)?;
    Ok(())
}
