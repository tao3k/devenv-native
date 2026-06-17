use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::error::QianjiError;
use crate::executors::{ContextAnnotator, FormalAuditMechanism};
use std::sync::Arc;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::PersonaRegistry;

use super::{annotation, formal_audit};

pub(super) fn annotation(
    orchestrator: &Arc<ThousandFacesOrchestrator>,
    registry: &Arc<PersonaRegistry>,
    node_def: &NodeDefinition,
) -> Arc<dyn QianjiMechanism> {
    let cfg = annotation::mechanism_config(node_def);
    Arc::new(ContextAnnotator {
        orchestrator: Arc::clone(orchestrator),
        registry: Arc::clone(registry),
        persona_id: cfg.persona_id,
        template_target: cfg.template_target,
        execution_mode: cfg.execution_mode,
        input_keys: cfg.input_keys,
        history_key: cfg.history_key,
        output_key: cfg.output_key,
    })
}

pub(super) fn formal_audit_native(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
    Arc::new(FormalAuditMechanism {
        invariants: vec![crate::safety::logic::Invariant::MustBeGrounded],
        retry_target_ids: formal_audit::retry_targets(node_def),
    })
}

pub(super) fn formal_audit_requires_llm_guard(
    node_def: &NodeDefinition,
) -> Result<(), QianjiError> {
    if formal_audit::uses_llm_controller(node_def) {
        return Err(QianjiError::Topology(
            "Task type `formal_audit` with `[nodes.qianhuan] + [nodes.llm]` requires external LLM execution; local Qianji LLM execution is retired, use marlin-agent-core.".to_string(),
        ));
    }
    Ok(())
}
