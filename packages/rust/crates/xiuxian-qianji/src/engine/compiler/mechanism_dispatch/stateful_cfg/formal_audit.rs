use crate::contracts::QianjiMechanism;
use crate::error::QianjiError;
use std::sync::Arc;

use crate::engine::compiler::formal_audit as formal_audit_cfg;
use crate::engine::compiler::mechanism_dispatch::resolver_chain;
use crate::engine::compiler::stateful_mechanisms;

pub(super) fn build(
    context: resolver_chain::DispatchContext<'_>,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    let resolver_chain::DispatchContext { node_def, .. } = context;
    stateful_mechanisms::formal_audit_requires_llm_guard(node_def)?;
    formal_audit_cfg::ensure_native_retry_budget_not_configured(node_def)?;
    Ok(stateful_mechanisms::formal_audit_native(node_def))
}
