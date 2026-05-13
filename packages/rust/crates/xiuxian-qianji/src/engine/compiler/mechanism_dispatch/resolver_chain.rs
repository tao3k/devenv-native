use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::engine::compiler_api::QianjiCompiler;
use crate::error::QianjiError;
use std::sync::Arc;

use crate::engine::compiler::TaskType;

#[derive(Clone, Copy)]
pub(super) struct DispatchContext<'a> {
    pub(super) task_type: TaskType,
    pub(super) compiler: &'a QianjiCompiler,
    pub(super) node_def: &'a NodeDefinition,
}

pub(super) type ResolveOutcome = Result<Arc<dyn QianjiMechanism>, QianjiError>;
pub(super) type ResolverFn = for<'a> fn(DispatchContext<'a>) -> Option<ResolveOutcome>;

pub(super) fn run(
    resolvers: &[ResolverFn],
    context: DispatchContext<'_>,
) -> Option<ResolveOutcome> {
    resolvers.iter().find_map(|resolver| resolver(context))
}
