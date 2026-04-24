use super::super::types::Cli;
use anyhow::Result;
use std::path::{Path, PathBuf};
use xiuxian_wendao::{LinkGraphIndex, resolve_link_graph_index_runtime};

pub(crate) fn build_index(cli: &Cli) -> Result<LinkGraphIndex> {
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

    build_index_with_optional_cache(&cli.root, &include_dirs, &exclude_dirs)
}

fn build_index_with_optional_cache(
    root: &Path,
    include_dirs: &[String],
    exclude_dirs: &[String],
) -> Result<LinkGraphIndex> {
    match LinkGraphIndex::build_with_cache(root, include_dirs, exclude_dirs) {
        Ok(index) => Ok(index),
        Err(error) if is_optional_link_graph_cache_failure(&error) => {
            eprintln!(
                "warning: link-graph cache unavailable; building index without cache: {error}"
            );
            LinkGraphIndex::build_with_filters(root, include_dirs, exclude_dirs)
                .map_err(anyhow::Error::msg)
        }
        Err(error) => Err(anyhow::Error::msg(error)),
    }
}

fn is_optional_link_graph_cache_failure(error: &str) -> bool {
    error.contains("link_graph cache valkey url is required")
        || error.contains("failed to connect valkey for link-graph cache")
        || error.contains("failed to GET link-graph cache from valkey")
        || error.contains("failed to SETEX link-graph cache to valkey")
        || error.contains("failed to SET link-graph cache to valkey")
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/helpers/index.rs"]
mod tests;
