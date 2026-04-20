//! Mechanism dispatch chain.
//!
//! Start in `resolver_chain`; `stateless`, `stateful_cfg`, and `leaf_dispatch`
//! contribute ordered resolver stages.

use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::engine::compiler_api::QianjiCompiler;
use crate::error::QianjiError;
use std::sync::Arc;

use super::task_type;

#[path = "../../engine_compiler_mechanism_dispatch_leaf_dispatch.rs"]
mod leaf_dispatch;
mod resolver_chain;
#[path = "../../engine_compiler_mechanism_dispatch_stateful_cfg.rs"]
mod stateful_cfg;
mod stateless;

const ROOT_RESOLVERS: [resolver_chain::ResolverFn; 3] =
    [stateless::build, stateful_cfg::build, leaf_dispatch::build];

pub(super) fn build(
    compiler: &QianjiCompiler,
    node_def: &NodeDefinition,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let task_type = task_type::TaskType::parse(node_def.task_type.as_str())?;
    let context = resolver_chain::DispatchContext {
        task_type,
        compiler,
        node_def,
    };
    resolver_chain::run(&ROOT_RESOLVERS, context).unwrap_or_else(|| {
        Err(QianjiError::Topology(format!(
            "Internal dispatch chain produced no resolver for task type: {task_type:?}"
        )))
    })
}
