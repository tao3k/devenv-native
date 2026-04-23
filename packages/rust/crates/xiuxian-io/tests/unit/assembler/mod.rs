//! Tests for `ContextAssembler` - Parallel I/O + templating + token counting.

#![cfg(feature = "assembler")]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use tempfile::TempDir;

use crate::{AssemblyResult, ContextAssembler};

mod basic;
mod errors;
mod markdown;
mod references;

pub(super) fn temp_dir() -> TempDir {
    TempDir::new().unwrap_or_else(|error| panic!("operation should succeed: {error}"))
}

pub(super) fn write_main_file(temp_dir: &TempDir, content: &str) -> PathBuf {
    write_file(temp_dir, "SKILL.md", content)
}

pub(super) fn write_file(temp_dir: &TempDir, relative_path: &str, content: &str) -> PathBuf {
    let path = temp_dir.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("operation should succeed: {error}"));
    }
    fs::write(&path, content).unwrap_or_else(|error| panic!("operation should succeed: {error}"));
    path
}

pub(super) fn assemble(
    main_path: PathBuf,
    references: Vec<PathBuf>,
    variables: Value,
) -> AssemblyResult {
    ContextAssembler::assemble_skill(main_path, references, variables)
        .unwrap_or_else(|error| panic!("operation should succeed: {error}"))
}
