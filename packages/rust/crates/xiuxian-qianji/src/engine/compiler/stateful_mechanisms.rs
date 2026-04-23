use crate::contracts::{NodeDefinition, QianjiMechanism};
use crate::error::QianjiError;
use crate::executors::{ContextAnnotator, FormalAuditMechanism};
use std::sync::Arc;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::PersonaRegistry;

use super::{annotation, formal_audit};

#[cfg(feature = "llm")]
use super::llm_node;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::LlmClient;

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

#[cfg(feature = "llm")]
pub(super) fn formal_audit_with_llm(
    orchestrator: &Arc<ThousandFacesOrchestrator>,
    registry: &Arc<PersonaRegistry>,
    node_def: &NodeDefinition,
    client: Arc<dyn LlmClient>,
) -> Result<Arc<dyn QianjiMechanism>, QianjiError> {
    formal_audit::ensure_llm_retry_targets(node_def)?;
    let threshold_score = formal_audit::threshold_score(node_def)?;
    let max_retries = formal_audit::max_retries(node_def)?;
    let llm_config = llm_node::mechanism_config(node_def);
    let annotation = annotation::mechanism_config(node_def);
    let retry_target_ids = formal_audit::retry_targets(node_def);

    Ok(Arc::new(crate::executors::LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator: Arc::clone(orchestrator),
            registry: Arc::clone(registry),
            persona_id: annotation.persona_id,
            template_target: annotation.template_target,
            execution_mode: annotation.execution_mode,
            input_keys: annotation.input_keys,
            history_key: annotation.history_key,
            output_key: annotation.output_key,
        },
        client,
        model: llm_config.model,
        threshold_score,
        max_retries,
        retry_target_ids,
        retry_counter_key: formal_audit::retry_counter_key(node_def),
        output_key: formal_audit::output_key(node_def),
        score_key: formal_audit::score_key(node_def),
        cognitive_early_halt_threshold: formal_audit::cognitive_early_halt_threshold(node_def),
        enable_cognitive_supervision: formal_audit::enable_cognitive_supervision(node_def),
    }))
}

pub(super) fn formal_audit_native(node_def: &NodeDefinition) -> Arc<dyn QianjiMechanism> {
    Arc::new(FormalAuditMechanism {
        invariants: vec![crate::safety::logic::Invariant::MustBeGrounded],
        retry_target_ids: formal_audit::retry_targets(node_def),
    })
}

#[cfg(not(feature = "llm"))]
pub(super) fn formal_audit_requires_llm_guard(
    node_def: &NodeDefinition,
) -> Result<(), QianjiError> {
    if formal_audit::uses_llm_controller(node_def) {
        return Err(QianjiError::Topology(
            "Task type `formal_audit` with `[nodes.qianhuan] + [nodes.llm]` requires enabling feature `llm` for xiuxian-qianji.".to_string(),
        ));
    }
    Ok(())
}

#[cfg(feature = "llm")]
pub(super) fn llm(
    _orchestrator: &Arc<ThousandFacesOrchestrator>,
    _registry: &Arc<PersonaRegistry>,
    node_def: &NodeDefinition,
    client: Arc<dyn LlmClient>,
) -> Arc<dyn QianjiMechanism> {
    let llm_cfg = llm_node::mechanism_config(node_def);

    Arc::new(
        crate::executors::StreamingLlmAnalyzer::builder()
            .client(client)
            .model(llm_cfg.model)
            .context_keys(llm_cfg.context_keys)
            .prompt_template(llm_cfg.prompt_template)
            .output_key(llm_cfg.output_key)
            .parse_json_output(llm_cfg.parse_json_output)
            .fallback_repo_tree(llm_cfg.fallback_repo_tree_on_parse_failure)
            .build(),
    )
}
