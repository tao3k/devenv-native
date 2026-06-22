use crate::engine::compiler::mechanism_dispatch::resolver_chain;
use crate::engine::compiler::{task_mechanisms, task_type};

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    let resolver_chain::DispatchContext {
        task_type,
        node_def,
        ..
    } = context;
    match task_type {
        #[cfg(feature = "wendao-integration")]
        task_type::TaskType::WendaoIngester => Some(Ok(task_mechanisms::wendao_ingester(node_def))),
        #[cfg(feature = "wendao-integration")]
        task_type::TaskType::WendaoRefresh => Some(Ok(task_mechanisms::wendao_refresh(node_def))),
        task_type::TaskType::Router => Some(task_mechanisms::router(node_def)),
        _ => None,
    }
}
