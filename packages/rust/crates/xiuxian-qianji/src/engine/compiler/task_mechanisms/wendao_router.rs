use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::error::QianjiError;
use std::sync::Arc;

use crate::engine::compiler::router;
#[cfg(feature = "wendao-integration")]
use crate::engine::compiler::{wendao_ingester, wendao_refresh};

#[cfg(feature = "wendao-integration")]
pub(in crate::engine::compiler) fn wendao_ingester(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = wendao_ingester::mechanism_config(node_def);
    Arc::new(crate::executors::WendaoIngesterMechanism {
        output_key: cfg.output_key,
        graph_scope: cfg.graph_scope,
        graph_scope_key: cfg.graph_scope_key,
        graph_dimension: cfg.graph_dimension,
        persist: cfg.persist,
        persist_best_effort: cfg.persist_best_effort,
    })
}

#[cfg(feature = "wendao-integration")]
pub(in crate::engine::compiler) fn wendao_refresh(
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = wendao_refresh::mechanism_config(node_def);
    Arc::new(crate::executors::WendaoRefreshMechanism {
        output_key: cfg.output_key,
        changed_paths_key: cfg.changed_paths_key,
        root_dir_key: cfg.root_dir_key,
        root_dir: cfg.root_dir,
        force_full: cfg.force_full,
        prefer_incremental: cfg.prefer_incremental,
        allow_full_fallback: cfg.allow_full_fallback,
        full_rebuild_threshold: cfg.full_rebuild_threshold,
        include_dirs: cfg.include_dirs,
        excluded_dirs: cfg.excluded_dirs,
    })
}

pub(in crate::engine::compiler) fn router(
    node_def: &NodeDefinition,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let cfg = router::config(node_def)?;
    Ok(Arc::new(crate::executors::ProbabilisticRouter {
        branches: cfg.branches,
        semantic_guard_route_key: cfg.semantic_guard_route_key,
    }))
}
