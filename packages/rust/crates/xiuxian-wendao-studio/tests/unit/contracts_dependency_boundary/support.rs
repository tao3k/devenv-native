use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn cargo_tree<const N: usize>(
    workspace_root: &Path,
    args: [&str; N],
) -> std::process::Output {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo)
        .current_dir(workspace_root)
        .arg("tree")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run cargo tree: {error}"))
}

pub(super) fn collect_domain_contract_imports(
    root: &Path,
    allowed_reexports: &[PathBuf],
    needle: &str,
    offenders: &mut Vec<String>,
) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));

    for entry in entries {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_domain_contract_imports(path.as_path(), allowed_reexports, needle, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if allowed_reexports
            .iter()
            .any(|allowed| path.as_path() == allowed.as_path())
        {
            continue;
        }
        let source = fs::read_to_string(path.as_path())
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if source.contains(needle) {
            offenders.push(path.display().to_string());
        }
    }
}

pub(super) fn collect_rust_source_occurrences(
    root: &Path,
    needle: &str,
    offenders: &mut Vec<String>,
) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));

    for entry in entries {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", root.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_occurrences(path.as_path(), needle, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(path.as_path())
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if source.contains(needle) {
            offenders.push(format!("{} contains {needle}", path.display()));
        }
    }
}

pub(super) fn workspace_root(manifest_dir: &Path) -> &Path {
    match manifest_dir.ancestors().nth(4) {
        Some(path) => path,
        None => panic!("xiuxian-wendao-studio should live under packages/rust/crates"),
    }
}
