use std::path::{Path, PathBuf};
use std::process::Command;

use super::ExternalSymbol;

/// Find files using fd command.
/// pattern: glob pattern like "**/Cargo.toml" or "*.rs"
pub(super) fn find_files(pattern: &str, project_root: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();

    // Convert glob pattern to fd pattern
    // "**/Cargo.toml" -> "Cargo.toml" with --recursive
    // "packages/**/Cargo.toml" -> "Cargo.toml" in packages/
    let fd_pattern = if pattern.contains("**") {
        pattern
            .split("**")
            .last()
            .unwrap_or(pattern)
            .trim_start_matches('/')
            .to_string()
    } else {
        pattern.to_string()
    };

    let base_dir = if pattern.starts_with('/') {
        PathBuf::from(
            pattern
                .trim_start_matches('/')
                .split("**")
                .next()
                .unwrap_or("."),
        )
    } else {
        project_root.to_path_buf()
    };

    // Build fd command
    let output = Command::new("fd")
        .arg(&fd_pattern)
        .arg(&base_dir)
        .arg("--max-depth")
        .arg("10")
        .arg("-t")
        .arg("f")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                if !line.trim().is_empty() {
                    results.push(PathBuf::from(line.trim()));
                }
            }
        }
        Ok(_) | Err(_) => {
            // Fallback: direct path check
            let path = base_dir.join(&fd_pattern);
            if path.exists() && path.file_name().is_some_and(|n| n == "Cargo.toml") {
                results.push(path);
            }
        }
    }

    results
}

/// Find Rust source files using fd.
pub(super) fn find_rs_files(source_path: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();

    let output = Command::new("fd")
        .arg("\\.rs$")
        .arg(source_path)
        .arg("--max-depth")
        .arg("10")
        .arg("-t")
        .arg("f")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                if !line.trim().is_empty() {
                    results.push(PathBuf::from(line.trim()));
                }
            }
        }
        Ok(_) | Err(_) => {
            log::debug!("fd not available or failed for: {}", source_path.display());
        }
    }

    results
}

/// Extract symbols from Rust source files in a crate directory.
pub(super) fn extract_symbols_from_crate(
    source_path: &Path,
    crate_name: &str,
) -> Vec<ExternalSymbol> {
    use crate::dependency_indexer::symbols::extract_dependency_symbols;

    let mut all_symbols = Vec::new();

    // Use fd to find .rs files
    let rs_files = find_rs_files(source_path);

    for rs_file in &rs_files {
        match extract_dependency_symbols(rs_file, "rust") {
            Ok(mut symbols) => {
                for sym in &mut symbols {
                    sym.crate_name = crate_name.to_string();
                }
                all_symbols.extend(symbols);
            }
            Err(e) => {
                log::debug!(
                    "Failed to extract symbols from {}: {}",
                    rs_file.display(),
                    e
                );
            }
        }
    }

    all_symbols
}
