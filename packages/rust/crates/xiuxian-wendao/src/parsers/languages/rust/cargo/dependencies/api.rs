//! `parsers::languages::rust::cargo::dependencies::api` owns Wendao cargo dependencies api behavior.

use super::regex::{RE_DEP_COMPLEX, RE_DEP_SIMPLE};
use super::types::CargoDependency;
use std::fs::read_to_string;
use std::path::Path;

/// Parse dependencies from a Cargo.toml file.
/// Priority: If this is a workspace root, parse [workspace.dependencies].
/// Otherwise, parse [dependencies] section.
///
/// # Errors
///
/// Returns I/O errors when reading the `Cargo.toml` file fails.
pub fn parse_cargo_dependencies(path: &Path) -> Result<Vec<CargoDependency>, std::io::Error> {
    let content = read_to_string(path)?;

    let is_workspace =
        content.contains("[workspace]") || content.contains("[workspace.dependencies]");

    let deps = if is_workspace {
        parse_workspace_dependencies(&content)
    } else {
        parse_regular_dependencies(&content)
    };

    Ok(deps)
}

fn section_slice<'a>(content: &'a str, section_header: &str) -> Option<&'a str> {
    let section_start = content.find(section_header)?;
    let section_content = &content[section_start..];

    let mut depth = 0;
    let mut in_content = false;
    let mut section_end = section_content.len();

    for (i, c) in section_content.char_indices() {
        if !in_content {
            if c == '\n' {
                in_content = true;
            }
            continue;
        }

        if c == '{' {
            depth += 1;
        } else if c == '}' {
            if depth > 0 {
                depth -= 1;
            }
        } else if (c == '[' && depth == 0) || c == '\0' {
            section_end = i;
            break;
        }
    }

    Some(&section_content[..section_end])
}

fn parse_dep_lines(dep_content: &str, skip_path_git: bool) -> Vec<CargoDependency> {
    let mut deps = Vec::new();

    for line in dep_content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('[') || trimmed.starts_with('#') {
            continue;
        }

        if let Some(cap) = RE_DEP_COMPLEX.captures(trimmed) {
            let name = cap[1].to_string();
            let version = cap[2].to_string();
            deps.push(CargoDependency::new(name, version));
            continue;
        }

        if let Some(cap) = RE_DEP_SIMPLE.captures(trimmed) {
            let name = cap[1].to_string();
            let version = cap[2].to_string();

            if skip_path_git && (version.starts_with("path") || version.starts_with("git")) {
                continue;
            }

            deps.push(CargoDependency::new(name, version));
        }
    }

    deps
}

fn parse_workspace_dependencies(content: &str) -> Vec<CargoDependency> {
    if let Some(dep_content) = section_slice(content, "[workspace.dependencies]") {
        return parse_dep_lines(dep_content, false);
    }

    parse_regular_dependencies(content)
}

fn parse_regular_dependencies(content: &str) -> Vec<CargoDependency> {
    if let Some(dep_content) = section_slice(content, "[dependencies]") {
        return parse_dep_lines(dep_content, true);
    }

    Vec::new()
}
