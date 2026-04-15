use crate::error::QianjiError;
use crate::runtime_config::resolve_process_project_root;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::link_graph::LinkGraphIndex;

pub(super) fn unix_timestamp_millis() -> Result<u128, QianjiError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| {
            QianjiError::Execution(format!("system clock drifted before UNIX_EPOCH: {error}"))
        })
}

fn resolve_repo_root_path(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }
    resolve_process_project_root().unwrap_or_else(std::env::temp_dir)
}

fn build_link_graph_index_for_root_with_builders<C, P>(
    root: &Path,
    cache_build: &C,
    plain_build: &P,
) -> Result<LinkGraphIndex, String>
where
    C: Fn(&Path) -> Result<LinkGraphIndex, String>,
    P: Fn(&Path) -> Result<LinkGraphIndex, String>,
{
    match cache_build(root) {
        Ok(index) => Ok(index),
        Err(cache_error) => plain_build(root).map_err(|plain_error| {
            format!("cache bootstrap failed ({cache_error}); build fallback failed ({plain_error})")
        }),
    }
}

fn build_link_graph_index_with_builders<C, P>(
    primary_root: &Path,
    fallback_root: &Path,
    cache_build: C,
    plain_build: P,
) -> Result<LinkGraphIndex, QianjiError>
where
    C: Fn(&Path) -> Result<LinkGraphIndex, String>,
    P: Fn(&Path) -> Result<LinkGraphIndex, String>,
{
    match build_link_graph_index_for_root_with_builders(primary_root, &cache_build, &plain_build) {
        Ok(index) => Ok(index),
        Err(primary_error) => {
            build_link_graph_index_for_root_with_builders(fallback_root, &cache_build, &plain_build)
                .map_err(|fallback_error| {
                    QianjiError::Topology(format!(
                        "failed to build LinkGraph index at `{}` ({primary_error}); \
fallback `{}` also failed ({fallback_error})",
                        primary_root.display(),
                        fallback_root.display()
                    ))
                })
        }
    }
}

pub(super) fn build_link_graph_index(
    explicit_repo_root: Option<&Path>,
) -> Result<LinkGraphIndex, QianjiError> {
    let primary_root = resolve_repo_root_path(explicit_repo_root);
    let fallback_root = std::env::temp_dir();
    build_link_graph_index_with_builders(
        primary_root.as_path(),
        fallback_root.as_path(),
        |root| LinkGraphIndex::build_with_cache(root, &[], &[]),
        LinkGraphIndex::build,
    )
}

#[cfg(test)]
#[path = "../../tests/unit/bootcamp/runtime.rs"]
mod tests;
