use crate::bin_support::wendao::types::Cli;
use crate::{LinkGraphIndex, resolve_link_graph_index_runtime};
use anyhow::Result;
use std::path::PathBuf;

pub(crate) fn build_index(cli: &Cli) -> Result<LinkGraphIndex> {
    let (include_dirs, exclude_dirs) = resolve_index_filters(cli);
    LinkGraphIndex::build_with_local_cache(&cli.root, &include_dirs, &exclude_dirs)
        .map_err(anyhow::Error::msg)
}

fn resolve_index_filters(cli: &Cli) -> (Vec<String>, Vec<String>) {
    let (include_dirs, exclude_dirs) = if cli.config_file.is_some() {
        let root_for_scope = if cli.root.is_absolute() {
            cli.root.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&cli.root)
        };
        let runtime_scope = resolve_link_graph_index_runtime(&root_for_scope);
        let include = if cli.include_dirs.is_empty() {
            runtime_scope.include_dirs
        } else {
            cli.include_dirs.clone()
        };
        let exclude = if cli.exclude_dirs.is_empty() {
            runtime_scope.exclude_dirs
        } else {
            cli.exclude_dirs.clone()
        };
        (include, exclude)
    } else {
        (cli.include_dirs.clone(), cli.exclude_dirs.clone())
    };

    (include_dirs, exclude_dirs)
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/helpers/index.rs"]
mod tests;
