//! Stateful-configuration branch for compiler mechanism dispatch.

use super::resolver_chain;
use crate::engine::compiler::TaskType;

#[path = "engine/compiler/mechanism_dispatch/stateful_cfg/formal_audit.rs"]
mod formal_audit;

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    match context.task_type {
        TaskType::FormalAudit => Some(formal_audit::build(context)),
        _ => None,
    }
}
