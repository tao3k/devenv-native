use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

pub(super) fn create_file(path: &Path) -> Result<fs::File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    fs::File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

pub(super) fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(payload)
        .with_context(|| format!("failed to serialize `{}`", path.display()))?;
    fs::write(path, format!("{raw}\n"))
        .with_context(|| format!("failed to write `{}`", path.display()))
}

pub(super) fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))
}
