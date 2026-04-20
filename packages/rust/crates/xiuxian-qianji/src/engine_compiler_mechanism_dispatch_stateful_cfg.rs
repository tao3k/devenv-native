use super::resolver_chain;
use crate::engine::compiler::task_type;

#[path = "engine/compiler/mechanism_dispatch/stateful_cfg/formal_audit.rs"]
mod formal_audit;
#[path = "engine/compiler/mechanism_dispatch/stateful_cfg/llm.rs"]
mod llm;

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Option<resolver_chain::ResolveOutcome> {
    match context.task_type {
        task_type::TaskType::FormalAudit => Some(formal_audit::build(context)),
        task_type::TaskType::Llm => Some(llm::build(context)),
        _ => None,
    }
}
