use include_dir::Dir;
use std::future::Future;

tokio::task_local! {
    static RUNTIME_WENDAO_MOUNTS: Vec<RuntimeWendaoMount>;
}

/// Runtime mount descriptor used by semantic URI resolution hooks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeWendaoMount {
    /// Semantic skill name (host segment in `wendao://skills/<name>/...`).
    pub(crate) semantic_name: &'static str,
    /// Relative references root inside mounted embedded directory.
    pub(crate) references_dir: &'static str,
    /// Embedded directory providing referenced resources.
    pub(crate) dir: &'static Dir<'static>,
}

/// Runs one future with task-local Wendao runtime mounts.
pub(crate) async fn with_runtime_wendao_mounts<F>(
    mounts: Vec<RuntimeWendaoMount>,
    future: F,
) -> F::Output
where
    F: Future,
{
    RUNTIME_WENDAO_MOUNTS.scope(mounts, future).await
}

pub(super) fn runtime_wendao_mounts_snapshot() -> Vec<RuntimeWendaoMount> {
    RUNTIME_WENDAO_MOUNTS
        .try_with(|mounts| mounts.clone())
        .unwrap_or_default()
}
