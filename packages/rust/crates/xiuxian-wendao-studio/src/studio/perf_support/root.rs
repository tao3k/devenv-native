use std::path::{Path, PathBuf};
use std::time::Duration;
use xiuxian_config_core::{lookup_positive_parsed, resolve_path_from_value};

pub(crate) const REAL_WORKSPACE_ROOT_ENV: &str = "XIUXIAN_WENDAO_GATEWAY_PERF_WORKSPACE_ROOT";
pub(crate) const REAL_WORKSPACE_READY_TIMEOUT_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_PERF_READY_TIMEOUT_SECS";
pub(crate) const DEFAULT_REAL_WORKSPACE_ROOT: &str = ".data/wendao-frontend";
pub(crate) const DEFAULT_REAL_WORKSPACE_READY_TIMEOUT_SECS: u64 = 900;

#[derive(Debug, Clone)]
pub(crate) enum GatewayPerfRoot {
    Owned(PathBuf),
    External(PathBuf),
}

pub(crate) fn create_perf_root() -> anyhow::Result<PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "xiuxian-wendao-gateway-perf-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

pub(crate) fn resolve_real_workspace_root() -> Option<PathBuf> {
    let project_root = xiuxian_io::PrjDirs::project_root();
    resolve_real_workspace_root_with_lookup(project_root.as_path(), &|key| std::env::var(key).ok())
}

pub(crate) fn resolve_real_workspace_root_with_lookup(
    project_root: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<PathBuf> {
    if let Some(resolved) = resolve_path_from_value(
        Some(project_root),
        lookup(REAL_WORKSPACE_ROOT_ENV).as_deref(),
    ) {
        return Some(resolved);
    }

    let fallback = project_root.join(DEFAULT_REAL_WORKSPACE_ROOT);
    fallback.exists().then_some(fallback)
}

pub(crate) fn real_workspace_ready_timeout() -> Duration {
    real_workspace_ready_timeout_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn real_workspace_ready_timeout_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Duration {
    Duration::from_secs(
        lookup_positive_parsed::<u64>(REAL_WORKSPACE_READY_TIMEOUT_ENV, lookup)
            .unwrap_or(DEFAULT_REAL_WORKSPACE_READY_TIMEOUT_SECS),
    )
}
