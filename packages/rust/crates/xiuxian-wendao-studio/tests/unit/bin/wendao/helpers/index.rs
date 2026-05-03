use super::{build_index, resolve_index_filters};
use crate::bin_support::wendao::types::Cli;
use clap::Parser;
use std::fs;

#[test]
fn local_cli_index_build_uses_cli_filters_without_cache_runtime() {
    let cli = Cli::parse_from([
        "wendao",
        "--include-dir",
        "docs",
        "--exclude-dir",
        ".cache",
        "audit",
        "docs",
    ]);

    let (include_dirs, exclude_dirs) = resolve_index_filters(&cli);
    assert_eq!(include_dirs, vec!["docs"]);
    assert_eq!(exclude_dirs, vec![".cache"]);
}

#[test]
fn local_cli_index_build_uses_local_cache_entrypoint() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let docs = temp.path().join("docs");
    fs::create_dir_all(&docs).unwrap_or_else(|error| panic!("create docs dir: {error}"));
    fs::write(docs.join("alpha.md"), "# Alpha\n\nLocal cache proof.\n")
        .unwrap_or_else(|error| panic!("write note: {error}"));
    let root = temp.path().to_string_lossy().to_string();
    let cli = Cli::parse_from([
        "wendao",
        "--root",
        &root,
        "--include-dir",
        "docs",
        "audit",
        "docs",
    ]);

    let index = build_index(&cli).unwrap_or_else(|error| panic!("build local CLI index: {error}"));
    let (_, hits) = index.search_planned(
        "Alpha",
        5,
        xiuxian_wendao::LinkGraphSearchOptions::default(),
    );
    assert_eq!(hits.len(), 1);
}
