#[cfg(feature = "wendao-integration")]
use crate::executors::KnowledgeSeeker;
#[cfg(feature = "wendao-integration")]
use std::sync::Arc;

use super::resolver_chain;
use crate::engine::compiler::{stateful_mechanisms, task_type};

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    let resolver_chain::DispatchContext {
        task_type,
        compiler,
        node_def,
    } = context;
    match task_type {
        #[cfg(feature = "wendao-integration")]
        task_type::TaskType::Knowledge => Some(Ok(Arc::new(KnowledgeSeeker {
            index: compiler.index.clone(),
        }))),
        task_type::TaskType::Annotation => Some(Ok(stateful_mechanisms::annotation(node_def))),
        _ => None,
    }
}
