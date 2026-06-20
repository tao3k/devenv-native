use super::{leaf_dispatch, resolver_chain, stateful_cfg, stateless};

use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::engine::compiler_api::QianjiCompiler;
use crate::error::QianjiError;
use std::sync::Arc;

use crate::engine::compiler::task_type;

const ROOT_RESOLVERS: [resolver_chain::ResolverFn; 3] =
    [stateless::build, stateful_cfg::build, leaf_dispatch::build];

pub(crate) fn build(
    compiler: &QianjiCompiler,
    node_def: &NodeDefinition,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let task_type = task_type::TaskType::parse(node_def.task_type.as_str())?;
    #[cfg(not(feature = "wendao-integration"))]
    let _ = compiler;
    let context = resolver_chain::DispatchContext {
        task_type,
        #[cfg(feature = "wendao-integration")]
        compiler,
        node_def,
    };
    resolver_chain::run(&ROOT_RESOLVERS, context).unwrap_or_else(|| {
        Err(QianjiError::Topology(format!(
            "Internal dispatch chain produced no resolver for task type: {task_type:?}"
        )))
    })
}
