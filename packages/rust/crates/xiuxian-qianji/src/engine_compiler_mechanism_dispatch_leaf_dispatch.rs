//! Leaf-dispatch branch for compiler task mechanism resolution.

use crate::QianjiError;

use super::resolver_chain;

#[path = "engine/compiler/mechanism_dispatch/leaf_dispatch/io_control.rs"]
mod io_control;
#[path = "engine/compiler/mechanism_dispatch/leaf_dispatch/quality_guard.rs"]
mod quality_guard;
#[path = "engine/compiler/mechanism_dispatch/leaf_dispatch/wendao_router.rs"]
mod wendao_router;
#[cfg(feature = "wendao-integration")]
#[path = "engine/compiler/mechanism_dispatch/leaf_dispatch/wendao_sql.rs"]
mod wendao_sql;

#[cfg(feature = "wendao-integration")]
const LEAF_RESOLVERS: [resolver_chain::ResolverFn; 4] = [
    io_control::build,
    quality_guard::build,
    wendao_router::build,
    wendao_sql::build,
];

#[cfg(not(feature = "wendao-integration"))]
const LEAF_RESOLVERS: [resolver_chain::ResolverFn; 3] = [
    io_control::build,
    quality_guard::build,
    wendao_router::build,
];

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    resolver_chain::run(&LEAF_RESOLVERS, context).or_else(|| {
        let task_type = context.task_type;
        Some(Err(QianjiError::Topology(format!(
            "Internal dispatch mismatch for leaf task routing: {task_type:?}"
        ))))
    })
}
