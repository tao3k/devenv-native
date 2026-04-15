//! Focused integration coverage for the built-in `modularity` rule pack.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_testing::{CollectionContext, ModularityRulePack, RulePack};

mod file_shape;
mod internal_root_doc_flow;
mod internal_root_doc_owner;
mod internal_root_visibility;
mod mod_interface;
mod root_entry;
mod root_facade;
mod root_hint;

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn crate_src_root(temp_dir: &TempDir, crate_name: &str) -> PathBuf {
    temp_dir
        .path()
        .join("packages")
        .join("rust")
        .join("crates")
        .join(crate_name)
        .join("src")
}

fn write_rust_file(src_root: &Path, relative_path: &str, content: &str) {
    let path = src_root.join(relative_path);
    let parent = path
        .parent()
        .unwrap_or_else(|| panic!("target file should have parent: {}", path.display()));
    must_ok(
        fs::create_dir_all(parent),
        "should create parent directories for fixture file",
    );
    must_ok(
        fs::write(&path, content),
        "should write fixture rust source file",
    );
}

fn evaluate_fixture(crate_name: &str, temp_dir: &TempDir) -> Vec<xiuxian_testing::ContractFinding> {
    let ctx = CollectionContext {
        suite_id: "contracts".to_string(),
        crate_name: Some(crate_name.to_string()),
        workspace_root: Some(temp_dir.path().to_path_buf()),
        labels: std::collections::BTreeMap::new(),
    };

    let pack = ModularityRulePack;
    let artifacts = must_ok(pack.collect(&ctx), "modularity collect should succeed");
    must_ok(
        pack.evaluate(&artifacts),
        "modularity evaluation should succeed",
    )
}
